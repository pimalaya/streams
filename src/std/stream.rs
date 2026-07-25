//! Blocking std transport handle.
//!
//! [`StreamStd`] is a single `Read + Write` type wrapping a TCP socket, a
//! Unix-domain socket or a TLS session (`rustls` or `native-tls`). TLS
//! options (provider, crypto, ALPN, a pinned certificate) come from
//! [`Tls`](crate::tls::Tls).

#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    path::Path,
    time::Duration,
};

use anyhow::{Result, bail};
use log::{debug, trace};
#[cfg(windows)]
use uds_windows::UnixStream;

use crate::{
    std::proxy::{Proxy, dial},
    tls::Tls,
};

#[derive(Debug)]
enum Stream {
    Tcp(TcpStream),
    Unix(UnixStream),
    #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
    Rustls(rustls::StreamOwned<rustls::ClientConnection, TcpStream>),
    #[cfg(feature = "native-tls")]
    NativeTls(native_tls::TlsStream<TcpStream>),
}

/// Blocking transport handle: TCP, Unix-domain or TLS, behind one
/// `Read + Write`.
#[derive(Debug)]
pub struct StreamStd {
    inner: Stream,
    host: String,
}

impl StreamStd {
    /// Opens a Unix-domain socket at `path`.
    pub fn connect_unix<P: AsRef<Path>>(path: P) -> Result<StreamStd> {
        debug!("connect unix stream");
        trace!("path: {}", path.as_ref().display());

        let inner = Stream::Unix(UnixStream::connect(path)?);
        let host = String::from("127.0.0.1");

        debug!("unix stream connected");
        Ok(Self { inner, host })
    }

    /// Opens a plain TCP connection to `host:port`.
    ///
    /// Routed through the ambient proxy ([`Proxy::System`]); use
    /// [`StreamStd::builder`] to select a proxy explicitly.
    pub fn connect_tcp(host: impl ToString, port: u16) -> Result<StreamStd> {
        let host = host.to_string();

        debug!("connect tcp stream");
        trace!("host: {host}");
        trace!("port: {port}");

        Self::open(host, port, None, &Proxy::System)
    }

    /// Opens a TCP connection and runs the TLS handshake (implicit TLS).
    ///
    /// Routed through the ambient proxy ([`Proxy::System`]); use
    /// [`StreamStd::builder`] to select a proxy explicitly.
    pub fn connect_tls(host: impl ToString, port: u16, tls: &Tls) -> Result<StreamStd> {
        let host = host.to_string();

        debug!("connect tls stream");
        trace!("host: {host}");
        trace!("port: {port}");

        Self::open(host, port, Some(tls), &Proxy::System)
    }

    /// Starts building a TCP connection to `host:port`, optionally wrapped
    /// in implicit TLS and/or routed through a chosen proxy. Terminates
    /// with [`StreamBuilder::connect`].
    ///
    /// The plain constructors ([`connect_tcp`](Self::connect_tcp),
    /// [`connect_tls`](Self::connect_tls)) are shorthands for this with the
    /// default [`Proxy::System`]; reach for the builder when a call site
    /// needs to override the proxy from its own configuration.
    pub fn builder(host: impl ToString, port: u16) -> StreamBuilder {
        StreamBuilder {
            host: host.to_string(),
            port,
            tls: None,
            proxy: Proxy::System,
        }
    }

    /// Dials `host:port` through `proxy`, optionally upgrading to implicit
    /// TLS. The single connect path shared by the constructors and the
    /// builder.
    fn open(host: String, port: u16, tls: Option<&Tls>, proxy: &Proxy) -> Result<StreamStd> {
        let tcp = dial(&host, port, proxy)?;

        match tls {
            Some(tls) => Self::_upgrade_tls(host, tcp, tls),
            None => {
                debug!("tcp stream connected");
                Ok(Self {
                    inner: Stream::Tcp(tcp),
                    host,
                })
            }
        }
    }

    /// Wraps a plain TCP stream in a TLS session (STARTTLS upgrade).
    ///
    /// Fails on Unix-domain or already-TLS variants.
    pub fn upgrade_tls(self, tls: &Tls) -> Result<StreamStd> {
        match self.inner {
            Stream::Tcp(tcp) => {
                debug!("upgrade tcp stream to tls");
                trace!("host: {}", self.host);
                Self::_upgrade_tls(self.host, tcp, tls)
            }
            Stream::Unix(_) => bail!("cannot upgrade Unix-domain stream to TLS"),
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            Stream::Rustls(_) => bail!("stream is already wrapped in rustls"),
            #[cfg(feature = "native-tls")]
            Stream::NativeTls(_) => bail!("stream is already wrapped in native-tls"),
        }
    }

    #[cfg(not(feature = "rustls-aws"))]
    #[cfg(not(feature = "rustls-ring"))]
    #[cfg(not(feature = "native-tls"))]
    fn _upgrade_tls(_: String, _: TcpStream, _: &Tls) -> Result<StreamStd> {
        bail!("missing cargo feature: `rustls-aws`, `rustls-ring` or `native-tls`")
    }

    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    fn _upgrade_tls(host: String, tcp: TcpStream, tls: &Tls) -> Result<StreamStd> {
        use crate::tls::TlsProvider;

        let provider = match &tls.provider {
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            Some(TlsProvider::Rustls) => TlsProvider::Rustls,
            #[cfg(not(feature = "rustls-aws"))]
            #[cfg(not(feature = "rustls-ring"))]
            Some(TlsProvider::Rustls) => {
                bail!("missing cargo feature: `rustls-aws` or `rustls-ring`")
            }
            #[cfg(feature = "native-tls")]
            Some(TlsProvider::NativeTls) => TlsProvider::NativeTls,
            #[cfg(not(feature = "native-tls"))]
            Some(TlsProvider::NativeTls) => bail!("missing cargo feature: `native-tls`"),
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            None => TlsProvider::Rustls,
            #[cfg(not(feature = "rustls-aws"))]
            #[cfg(not(feature = "rustls-ring"))]
            #[cfg(feature = "native-tls")]
            None => TlsProvider::NativeTls,
        };

        match provider {
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            TlsProvider::Rustls => {
                use std::{fs, sync::Arc};

                use rustls::{
                    ClientConfig, ClientConnection, StreamOwned,
                    crypto::{self, CryptoProvider},
                    pki_types::{CertificateDer, pem::PemObject},
                };
                use rustls_platform_verifier::{ConfigVerifierExt, Verifier};

                use crate::tls::RustlsCrypto;

                let crypto_provider = match &tls.rustls.crypto {
                    #[cfg(feature = "rustls-aws")]
                    Some(RustlsCrypto::Aws) => crypto::aws_lc_rs::default_provider(),
                    #[cfg(not(feature = "rustls-aws"))]
                    Some(RustlsCrypto::Aws) => bail!("missing cargo feature: `rustls-aws`"),
                    #[cfg(feature = "rustls-ring")]
                    Some(RustlsCrypto::Ring) => crypto::ring::default_provider(),
                    #[cfg(not(feature = "rustls-ring"))]
                    Some(RustlsCrypto::Ring) => bail!("missing cargo feature: `rustls-ring`"),
                    #[cfg(feature = "rustls-ring")]
                    None => crypto::ring::default_provider(),
                    #[cfg(not(feature = "rustls-ring"))]
                    #[cfg(feature = "rustls-aws")]
                    None => crypto::aws_lc_rs::default_provider(),
                    #[cfg(not(feature = "rustls-ring"))]
                    #[cfg(not(feature = "rustls-aws"))]
                    None => bail!("missing cargo feature: `rustls-aws` or `rustls-ring`"),
                };

                let crypto_provider = match crypto_provider.install_default() {
                    Ok(()) => CryptoProvider::get_default().unwrap().clone(),
                    Err(crypto_provider) => crypto_provider,
                };

                let mut config = if let Some(pem_path) = &tls.cert {
                    trace!("using TLS cert at {}", pem_path.display());
                    let pem = fs::read(pem_path)?;

                    let Some(cert) = CertificateDer::pem_slice_iter(&pem).next() else {
                        bail!("empty TLS cert at {}", pem_path.display())
                    };
                    let cert = cert?;

                    // NOTE: pin the leaf; a self-signed CA-marked leaf
                    // (Proton Bridge) fails a normal chain build with
                    // CaUsedAsEndEntity.
                    let fallback = Verifier::new_with_extra_roots(
                        vec![cert.clone()],
                        crypto_provider.clone(),
                    )?;

                    let verifier = pinned::PinnedServerCertVerifier::new(
                        cert,
                        Arc::new(fallback),
                        crypto_provider,
                    );

                    ClientConfig::builder()
                        .dangerous()
                        .with_custom_certificate_verifier(Arc::new(verifier))
                        .with_no_client_auth()
                } else {
                    trace!("using platform TLS certs");
                    ClientConfig::with_platform_verifier()?
                };

                config.alpn_protocols = tls
                    .rustls
                    .alpn
                    .iter()
                    .map(|p| p.as_bytes().to_vec())
                    .collect();

                let server_name = host.to_string().try_into()?;
                let conn = ClientConnection::new(Arc::new(config), server_name)?;
                let inner = Stream::Rustls(StreamOwned::new(conn, tcp));

                debug!("tls stream connected");
                Ok(StreamStd { inner, host })
            }

            #[cfg(feature = "native-tls")]
            TlsProvider::NativeTls => {
                use std::fs;

                use native_tls::{Certificate, TlsConnector};

                let mut builder = TlsConnector::builder();

                if let Some(pem_path) = &tls.cert {
                    trace!("using TLS cert at {}", pem_path.display());
                    let pem = fs::read(pem_path)?;
                    let cert = Certificate::from_pem(&pem)?;
                    builder.add_root_certificate(cert);
                } else {
                    trace!("using platform TLS certs");
                }

                let connector = builder.build()?;
                let inner = Stream::NativeTls(connector.connect(host.as_str(), tcp)?);

                debug!("tls stream connected");
                Ok(StreamStd { inner, host })
            }

            // NOTE: every provider is matched above; the pattern only
            // remains reachable on partial feature sets.
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

/// Builder for [`StreamStd`]: the proxy-aware connect entry point.
///
/// Created by [`StreamStd::builder`]. Set implicit TLS with [`tls`](Self::tls)
/// and/or a proxy with [`proxy`](Self::proxy), then open the connection with
/// the terminal [`connect`](Self::connect).
#[derive(Debug)]
pub struct StreamBuilder {
    host: String,
    port: u16,
    tls: Option<Tls>,
    proxy: Proxy,
}

impl StreamBuilder {
    /// Runs an implicit-TLS handshake once connected (omit for plaintext
    /// or a later STARTTLS [`upgrade_tls`](StreamStd::upgrade_tls)).
    pub fn tls(mut self, tls: Tls) -> Self {
        self.tls = Some(tls);
        self
    }

    /// Selects the proxy. Defaults to [`Proxy::System`] (resolved from the
    /// environment at connect time).
    pub fn proxy(mut self, proxy: Proxy) -> Self {
        self.proxy = proxy;
        self
    }

    /// Opens the connection (blocking). Terminal.
    pub fn connect(self) -> Result<StreamStd> {
        debug!("connect stream");
        trace!("host: {}", self.host);
        trace!("port: {}", self.port);

        StreamStd::open(self.host, self.port, self.tls.as_ref(), &self.proxy)
    }
}

impl Read for StreamStd {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.inner {
            Stream::Tcp(s) => s.read(buf),
            Stream::Unix(s) => s.read(buf),
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            Stream::Rustls(s) => s.read(buf),
            #[cfg(feature = "native-tls")]
            Stream::NativeTls(s) => s.read(buf),
        }
    }
}

impl Write for StreamStd {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.inner {
            Stream::Tcp(s) => s.write(buf),
            Stream::Unix(s) => s.write(buf),
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            Stream::Rustls(s) => s.write(buf),
            #[cfg(feature = "native-tls")]
            Stream::NativeTls(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.inner {
            Stream::Tcp(s) => s.flush(),
            Stream::Unix(s) => s.flush(),
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            Stream::Rustls(s) => s.flush(),
            #[cfg(feature = "native-tls")]
            Stream::NativeTls(s) => s.flush(),
        }
    }
}

/// Socket-level tuning shared by every variant.
impl StreamStd {
    /// Sets the read timeout on the underlying socket; `None` blocks
    /// forever.
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        match &self.inner {
            Stream::Tcp(s) => s.set_read_timeout(timeout),
            Stream::Unix(s) => s.set_read_timeout(timeout),
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            Stream::Rustls(s) => s.sock.set_read_timeout(timeout),
            #[cfg(feature = "native-tls")]
            Stream::NativeTls(s) => s.get_ref().set_read_timeout(timeout),
        }
    }

    /// Toggles non-blocking mode on the underlying socket. Under a TLS
    /// variant it applies to the socket beneath the session, so reads and
    /// writes surface `WouldBlock` reliably (unlike a read timeout, which
    /// the TLS layer does not always propagate).
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        match &self.inner {
            Stream::Tcp(s) => s.set_nonblocking(nonblocking),
            Stream::Unix(s) => s.set_nonblocking(nonblocking),
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            Stream::Rustls(s) => s.sock.set_nonblocking(nonblocking),
            #[cfg(feature = "native-tls")]
            Stream::NativeTls(s) => s.get_ref().set_nonblocking(nonblocking),
        }
    }
}

/// Certificate pinning for the rustls TLS branch.
#[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
mod pinned {
    use std::sync::Arc;

    use rustls::{
        DigitallySignedStruct, Error, SignatureScheme,
        client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
        crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature},
        pki_types::{CertificateDer, ServerName, UnixTime},
    };
    use rustls_platform_verifier::Verifier;

    /// A rustls verifier that pins one server certificate.
    ///
    /// The pin is trusted when the server presents it verbatim as its leaf,
    /// the model self-signed servers need (e.g. Proton Bridge): such a
    /// certificate is often a CA, which a normal chain build rejects in the
    /// leaf position (`CaUsedAsEndEntity`).
    ///
    /// A different leaf falls back to the pin as an extra trust anchor;
    /// handshake signatures are always verified through the active crypto
    /// provider.
    #[derive(Debug)]
    pub struct PinnedServerCertVerifier {
        pinned: CertificateDer<'static>,
        fallback: Arc<Verifier>,
        provider: Arc<CryptoProvider>,
    }

    impl PinnedServerCertVerifier {
        /// Builds a pinning verifier for `pinned`.
        ///
        /// Handshake signatures are verified through `provider`; `fallback`
        /// handles a server presenting a different leaf.
        pub fn new(
            pinned: CertificateDer<'static>,
            fallback: Arc<Verifier>,
            provider: Arc<CryptoProvider>,
        ) -> Self {
            Self {
                pinned,
                fallback,
                provider,
            }
        }
    }

    impl ServerCertVerifier for PinnedServerCertVerifier {
        fn verify_server_cert(
            &self,
            end_entity: &CertificateDer<'_>,
            intermediates: &[CertificateDer<'_>],
            server_name: &ServerName<'_>,
            ocsp_response: &[u8],
            now: UnixTime,
        ) -> Result<ServerCertVerified, Error> {
            if end_entity.as_ref() == self.pinned.as_ref() {
                return Ok(ServerCertVerified::assertion());
            }

            self.fallback.verify_server_cert(
                end_entity,
                intermediates,
                server_name,
                ocsp_response,
                now,
            )
        }

        fn verify_tls12_signature(
            &self,
            message: &[u8],
            cert: &CertificateDer<'_>,
            dss: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            verify_tls12_signature(
                message,
                cert,
                dss,
                &self.provider.signature_verification_algorithms,
            )
        }

        fn verify_tls13_signature(
            &self,
            message: &[u8],
            cert: &CertificateDer<'_>,
            dss: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            verify_tls13_signature(
                message,
                cert,
                dss,
                &self.provider.signature_verification_algorithms,
            )
        }

        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            self.provider
                .signature_verification_algorithms
                .supported_schemes()
        }
    }
}
