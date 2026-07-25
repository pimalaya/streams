#![cfg_attr(docsrs, feature(doc_cfg))]

//! # pimalaya-stream
//!
//! Stream, TLS and SASL utils shared by the Pimalaya io-* protocol
//! crates. Published for internal Pimalaya usage: the API follows the
//! needs of its consumers (io-imap, io-smtp, io-http and friends) and
//! may change without notice.
//!
//! ## Layout
//!
//! The crate is deliberately std: it wraps TLS providers and sockets
//! and exposes no I/O-free coroutines, so the no_std conventions of
//! the io-* family do not apply here.
//!
//! [`tls`] holds the provider-agnostic TLS options consumed by connect
//! and upgrade: the provider choice (Rustls with ring or aws crypto,
//! Native TLS), ALPN identifiers and an optional extra trust anchor.
//! [`sasl`] holds the credential configuration of the SASL mechanisms
//! the protocol crates implement (ANONYMOUS, LOGIN, PLAIN,
//! OAUTHBEARER, XOAUTH2, SCRAM-SHA-256). The [`std`] module (`std`
//! feature) is the blocking runtime layer: the [`std::stream`] transport
//! — one `Read + Write` handle over TCP, Unix sockets or a TLS session,
//! with the plain-to-TLS upgrade STARTTLS flows need — and the
//! [`std::proxy`] selector (SOCKS5, HTTP CONNECT, or environment-resolved)
//! that every connect funnels through, defaulting to the ambient proxy
//! variables. The module is deliberately named after its runtime: a
//! future async runtime would gain a sibling module (tokio) next to it.
//!
//! ## Conventions
//!
//! The conventions every Pimalaya repository shares are described in
//! the org
//! [ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md)
//! and
//! [GUIDELINES](https://github.com/pimalaya/.github/blob/master/GUIDELINES.md).
//! Logging follows the library rules: debug marks the lifecycle points
//! (connect, upgrade), trace carries the data.

pub mod sasl;
#[cfg(feature = "std")]
pub mod std;
pub mod tls;
