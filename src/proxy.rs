//! How a connection reaches its target.
//!
//! [`Proxy`] selects it: directly, through a SOCKS5 proxy,
//! through an HTTP `CONNECT` proxy, or, the default, resolved from the
//! environment at connect time. Every transport (IMAP, SMTP, HTTP, ...)
//! funnels through [`Proxy::connect`], so proxy support is
//! uniform across protocols.
//!
//! Only the tunnel model is implemented: the target host and port stay
//! opaque to the proxy, TLS still terminates at the target, and
//! certificate validation stays bound to the target host (the connect
//! returns a plain [`TcpStream`] to the target; the caller upgrades it).
//! Plaintext HTTP forward proxying (absolute-URI request lines) is
//! intentionally unsupported: it would leak proxy awareness into the HTTP
//! layer, and every Pimalaya HTTP backend is HTTPS.
//!
//! The handshakes themselves live in the `io-proxy` crate (I/O-free
//! coroutines for SOCKS5 and HTTP `CONNECT`); this module only owns the
//! config concerns: resolving [`Proxy::System`] from the
//! environment, `no_proxy` bypass and URL parsing, and drives those
//! coroutines over the socket.
//!
//! Logging follows the crate rules: debug marks the connect decision
//! (kind, proxy endpoint, source). Credentials never reach the logs.

use std::{env, net::TcpStream};

use anyhow::{Context, Result, bail};
use io_proxy::{
    client::{connect_http, connect_socks5},
    http::connect::HttpCredentials,
    socks::v5::{address::Socks5Address, auth::Socks5Credentials},
};
use log::debug;
use secrecy::{ExposeSecret, SecretString};
use url::Url;

/// Username/password credentials for a proxy.
///
/// The password is held in a [`SecretString`] and redacted from the
/// [`Debug`] output.
#[derive(Clone, Debug)]
pub struct ProxyAuth {
    /// Proxy username.
    pub user: String,
    /// Proxy password.
    pub pass: SecretString,
}

/// How a TCP connection reaches its target.
#[derive(Clone, Debug, Default)]
pub enum Proxy {
    /// Direct connection, no proxy.
    None,
    /// Tunnel through a SOCKS5 proxy (RFC 1928); the proxy resolves the
    /// target hostname (socks5h semantics).
    Socks5 {
        /// Proxy host.
        host: String,
        /// Proxy port.
        port: u16,
        /// Optional username/password authentication (RFC 1929).
        auth: Option<ProxyAuth>,
    },
    /// Tunnel through an HTTP proxy via the `CONNECT` method.
    Http {
        /// Proxy host.
        host: String,
        /// Proxy port.
        port: u16,
        /// Optional `Proxy-Authorization: Basic` credentials.
        auth: Option<ProxyAuth>,
    },
    /// Resolve from the environment at connect time (default).
    ///
    /// A `no_proxy` match (plus loopback, always) bypasses the proxy;
    /// otherwise `all_proxy` wins, then `https_proxy`. Both the lowercase
    /// and uppercase spellings are read, lowercase first.
    #[default]
    System,
}

impl Proxy {
    /// Parses a proxy URL such as `socks5://user:pass@host:1080` or
    /// `http://proxy.corp:3128`.
    ///
    /// `socks5`, `socks5h` and `socks` map to [`Self::Socks5`]; `http`
    /// and `https` map to [`Self::Http`] (`CONNECT` tunnelling).
    pub fn from_url(raw: &str) -> Result<Self> {
        let url = Url::parse(raw).with_context(|| format!("parse proxy url {raw}"))?;

        let host = url
            .host_str()
            .with_context(|| format!("proxy url {raw} has no host"))?
            .to_string();

        let auth = match url.username() {
            "" => None,
            user => Some(ProxyAuth {
                user: user.to_string(),
                pass: SecretString::from(url.password().unwrap_or_default().to_string()),
            }),
        };

        match url.scheme() {
            "socks5" | "socks5h" | "socks" => {
                let port = url.port().unwrap_or(1080);
                Ok(Self::Socks5 { host, port, auth })
            }
            "http" | "https" => {
                let port = url.port().unwrap_or(8080);
                Ok(Self::Http { host, port, auth })
            }
            other => bail!("unsupported proxy scheme `{other}` in {raw}"),
        }
    }

    /// Opens a plain [`TcpStream`] to `host:port`, tunnelling through
    /// this proxy.
    ///
    /// [`Self::System`] is resolved against the environment here (never
    /// before), so `no_proxy` sees the actual target. On success the
    /// returned stream is positioned at the start of the tunnel, ready
    /// for the caller's TLS handshake or plaintext protocol.
    pub fn connect(&self, host: &str, port: u16) -> Result<TcpStream> {
        let (proxy, source) = match self {
            Self::System => Self::from_env(host),
            other => (other.clone(), "config"),
        };

        match &proxy {
            Self::None => {
                debug!("connect {host}:{port} directly (source: {source})");
                Ok(TcpStream::connect((host, port))
                    .with_context(|| format!("connect {host}:{port}"))?)
            }
            Self::Socks5 {
                host: phost,
                port: pport,
                auth,
            } => {
                debug!("connect {host}:{port} via socks5 proxy {phost}:{pport} (source: {source})");
                let mut tcp = TcpStream::connect((phost.as_str(), *pport))
                    .with_context(|| format!("connect socks5 proxy {phost}:{pport}"))?;
                let target = Socks5Address::new(host, port)?;
                let credentials = match auth {
                    Some(auth) => Some(Socks5Credentials::new(
                        &auth.user,
                        auth.pass.expose_secret(),
                    )?),
                    None => None,
                };
                connect_socks5(&mut tcp, target, credentials)
                    .with_context(|| format!("socks5 handshake to {host}:{port}"))?;
                Ok(tcp)
            }
            Self::Http {
                host: phost,
                port: pport,
                auth,
            } => {
                debug!("connect {host}:{port} via http proxy {phost}:{pport} (source: {source})");
                let mut tcp = TcpStream::connect((phost.as_str(), *pport))
                    .with_context(|| format!("connect http proxy {phost}:{pport}"))?;
                let credentials = auth
                    .as_ref()
                    .map(|auth| HttpCredentials::new(&auth.user, auth.pass.expose_secret()));
                connect_http(&mut tcp, host, port, credentials)
                    .with_context(|| format!("http connect to {host}:{port}"))?;
                Ok(tcp)
            }
            Self::System => unreachable!("System proxy resolved before match"),
        }
    }

    /// Resolves [`Self::System`] against the environment for
    /// `target_host`, returning the concrete proxy and a label naming
    /// the source (for logs).
    fn from_env(target_host: &str) -> (Self, &'static str) {
        if Self::bypasses(target_host) {
            return (Self::None, "no_proxy");
        }

        for (var, label) in [("all_proxy", "all_proxy"), ("https_proxy", "https_proxy")] {
            let Some(raw) = Self::env_var(var) else {
                continue;
            };
            match Self::from_url(&raw) {
                Ok(proxy) => return (proxy, label),
                // NOTE: a malformed variable must not abort the connection
                Err(err) => debug!("ignoring invalid {label}: {err:#}"),
            }
        }

        (Self::None, "direct")
    }

    /// Whether `host` bypasses the proxy: loopback always, plus any
    /// `no_proxy` entry (suffix match, or `*` for everything).
    fn bypasses(host: &str) -> bool {
        let host = host.trim().trim_end_matches('.').to_ascii_lowercase();

        // NOTE: a proxy cannot reach the caller's own loopback
        if host == "localhost"
            || host.ends_with(".localhost")
            || host == "127.0.0.1"
            || host == "::1"
        {
            return true;
        }

        let Some(no_proxy) = Self::env_var("no_proxy") else {
            return false;
        };

        for entry in no_proxy.split(',') {
            let entry = entry
                .trim()
                .trim_start_matches('.')
                .trim_end_matches('.')
                .to_ascii_lowercase();

            if entry.is_empty() {
                continue;
            }
            if entry == "*" {
                return true;
            }
            if host == entry || host.ends_with(&format!(".{entry}")) {
                return true;
            }
        }

        false
    }

    /// Reads an environment proxy variable, lowercase spelling first
    /// then uppercase, treating an empty value as unset.
    fn env_var(lower: &str) -> Option<String> {
        let non_empty = |v: String| if v.is_empty() { None } else { Some(v) };
        env::var(lower)
            .ok()
            .and_then(non_empty)
            .or_else(|| env::var(lower.to_uppercase()).ok().and_then(non_empty))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_url_parses_socks5_with_auth() {
        let proxy = Proxy::from_url("socks5://alice:secret@10.0.0.1:1080").unwrap();
        match proxy {
            Proxy::Socks5 { host, port, auth } => {
                assert_eq!(host, "10.0.0.1");
                assert_eq!(port, 1080);
                let auth = auth.unwrap();
                assert_eq!(auth.user, "alice");
                assert_eq!(auth.pass.expose_secret(), "secret");
            }
            other => panic!("expected socks5, got {other:?}"),
        }
    }

    #[test]
    fn from_url_defaults_http_port_and_no_auth() {
        let proxy = Proxy::from_url("http://proxy.corp").unwrap();
        match proxy {
            Proxy::Http { host, port, auth } => {
                assert_eq!(host, "proxy.corp");
                assert_eq!(port, 8080);
                assert!(auth.is_none());
            }
            other => panic!("expected http, got {other:?}"),
        }
    }

    #[test]
    fn from_url_rejects_unknown_scheme() {
        assert!(Proxy::from_url("ftp://proxy.corp:21").is_err());
    }

    #[test]
    fn loopback_is_always_bypassed() {
        assert!(Proxy::bypasses("localhost"));
        assert!(Proxy::bypasses("127.0.0.1"));
        assert!(Proxy::bypasses("::1"));
        assert!(Proxy::bypasses("dev.localhost"));
    }
}
