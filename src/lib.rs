#![cfg_attr(docsrs, feature(doc_cfg))]

//! # pimalaya-stream
//!
//! Opens and upgrades blocking streams for the Pimalaya io-* protocol
//! crates: TCP connections, TLS connections, Unix-domain sockets,
//! proxy resolution and the plain-to-TLS upgrade STARTTLS flows need.
//! It also carries the TLS configuration vocabulary those crates share.
//!
//! The crate is public. Every io-* protocol client needs a stream
//! handed to it, and this is the one Pimalaya's own clients are built
//! on, so a third party wiring its own client against io-imap,
//! io-smtp, io-http or a sibling is a first-class consumer rather than
//! a tolerated one. Reusing it is the supported path, reimplementing
//! TCP, TLS, proxy resolution and the STARTTLS upgrade is not.
//!
//! ## Stability
//!
//! The crate stays on 0.x. Breaking changes land in minor bumps and
//! never in patch bumps, which is what Cargo already assumes for a 0.x
//! version, so pinning a minor keeps a consumer building. Every break
//! is spelled out in the
//! [CHANGELOG](https://github.com/pimalaya/stream/blob/master/CHANGELOG.md).
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
//! The [`std`] module (`std` feature) is the blocking runtime layer. It
//! carries the [`std::stream`] transport, one `Read + Write` handle
//! over TCP, Unix sockets or a TLS session, with the upgrade from plain
//! to TLS, and the [`std::proxy`] selector (SOCKS5, HTTP CONNECT, or
//! environment-resolved) that every connect funnels through,
//! defaulting to the ambient proxy variables. The module is
//! deliberately named after its runtime: a future async runtime would
//! gain a sibling module (tokio) next to it.
//!
//! SASL is not here. The credential vocabulary and the mechanisms that
//! compute payloads from it live in io-sasl, which is no_std and knows
//! nothing about sockets; a protocol crate reaching for authentication
//! reaches there, and this crate stays what its name says.
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

#[cfg(feature = "std")]
pub mod std;
pub mod tls;
