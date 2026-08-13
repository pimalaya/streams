# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added examples for the three shapes a consumer opens a connection in: implicit TLS, a STARTTLS upgrade, and a connect through an explicitly configured proxy.

### Changed

- Rescoped the crate as public, dropping the internal-usage disclaimer from the README and the crate documentation.

  The blocking stream, the TLS options and the proxy selector are documented for any consumer of the io- protocol crates, not only for Pimalaya's own products. The crate stays on 0.x: breaking changes land in minor bumps and never in patch bumps.

### Removed

- Removed the `sasl` module. **Breaking.**

  `Sasl`, `SaslMechanism` and the six credential structs moved to io-sasl, which is no_std, opens nothing and pairs each credential type with the coroutine that computes its payloads. Callers import them from there, where the structs carry a `Creds` suffix (`SaslPlainCreds`, `SaslLoginCreds`, ...) and SCRAM gained the client nonce and the channel binding the mechanism needs.

  This is what the extraction was for: a transport crate wrapping sockets and TLS sessions had become the home of a vocabulary that a no_std protocol core needed, so io-imap and io-smtp had to depend on it just to name a mechanism. Both now depend on it only for their std clients, and their coroutine cores are free of it.

## [0.1.2] - 2026-07-26

### Added

- Added the `Proxy` selector and blocking `dial` under the `std::proxy` module (`std` feature), letting every `StreamStd` connect tunnel through a proxy.

  Variants: `None` (direct), `Socks5` and `Http` (`CONNECT`) with optional username/password auth, and the default `System`, which resolves `all_proxy`/`https_proxy` from the environment at dial time, honouring `no_proxy` and always bypassing loopback. `Proxy::from_url` parses `socks5://`, `socks5h://`, `socks://`, `http://` and `https://` URLs. The handshakes run on the I/O-free `io-proxy` coroutines.

## [0.1.1] - 2026-07-25

### Fixed

- Fixed the `tls.cert` option being unusable for self-signed servers (Proton Bridge, self-hosted).

  Under rustls a configured certificate is now pinned to the server's leaf, instead of being registered as a CA trust anchor that rejected the self-signed leaf with `CaUsedAsEndEntity`. A server presenting a different leaf still falls back to using the certificate as an extra trust anchor.

## [0.1.0] - 2026-07-15

### Changed

- Aligned logging with the Pimalaya guidelines: connect and upgrade lifecycle points now log a short debug phrase followed by trace lines carrying the data (host, port, path), instead of single data-inlined traces.

### Removed

- Removed the unused serde dependency, along with the serde feature of secrecy, lightening the dependency tree.

## [0.0.1] - 2026-06-03

### Added

- Added the protocol-agnostic `Sasl` enum and `SaslMechanism` tag.

  Variants `Anonymous`, `Login`, `Plain`, `OAuthBearer`, `XOAuth2`, `ScramSha256` wrap per-mechanism structs (`SaslAnonymous`, `SaslLogin`, `SaslPlain`, `SaslOauthbearer`, `SaslXoauth2`, `SaslScramSha256`) carrying only the bits each mechanism actually transmits. Consumer crates translate them into the appropriate protocol framing.

- Added the `Tls` configuration struct.

  Carries a `provider: Option<TlsProvider>` selector (`Rustls` / `NativeTls`, falling back to the first enabled feature in the order `rustls-ring`, `rustls-aws`, `native-tls`), a `Rustls` sub-struct (crypto provider + ALPN list) and an optional PEM trust anchor path.

- Added the `StreamStd` blocking transport behind the `std` feature.

  Single `Read + Write` handle wrapping a TCP socket, a Unix-domain socket, a `rustls` TLS session or a `native-tls` TLS session. Constructors: `connect_tcp`, `connect_unix`, `connect_tls` (implicit TLS), `upgrade_tls` (STARTTLS).

- Added the `rustls-ring` cargo feature (default).

  Enables the rustls TLS backend with the ring crypto provider; pulls in `rustls`, `rustls-platform-verifier` and gates the `Rustls(...)` `StreamStd` variant.

- Added the `rustls-aws` cargo feature.

  Same rustls backend but with the aws-lc-rs crypto provider.

- Added the `native-tls` cargo feature.

  Enables the platform-backed `native-tls` TLS backend and gates the `NativeTls(...)` `StreamStd` variant.

- Added the `vendored` cargo feature.

  Forwarded to `native-tls/vendored` so consumers can compile the underlying TLS dependencies in vendored mode.

[unreleased]: https://github.com/pimalaya/stream/compare/v0.1.2..HEAD
[0.1.2]: https://github.com/pimalaya/stream/compare/v0.1.1..v0.1.2
[0.1.1]: https://github.com/pimalaya/stream/compare/v0.1.0..v0.1.1
[0.1.0]: https://github.com/pimalaya/stream/compare/v0.0.1..v0.1.0
[0.0.1]: https://github.com/pimalaya/stream/compare/root...v0.0.1
