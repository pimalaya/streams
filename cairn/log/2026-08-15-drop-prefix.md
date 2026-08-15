---
cairn: log
change: drop-prefix
landed: 2026-08-15
---

# Drop the Stream prefix from the public types

The crate had been of two minds about its own names. `Proxy` and `Tls` stood bare, while `StreamRetry` and the three `Stream*ConnectOptions` carried a prefix, and nothing but the order they were written in explained why. They are now `Retry`, `DEFAULT_TIMEOUT`, `TcpConnectOptions`, `TlsConnectOptions` and `UnixConnectOptions`.

The org guidelines settle which way to resolve it. naming-007 makes the domain prefix strict everywhere except the shared std toolkit crates, this one included, on the grounds that the crate name and the module path already namespace what they hold: `pimalaya_stream::retry::StreamRetry` said stream three times. What is left reads like the two thirds of the crate that already worked this way, `retry::Retry` beside `proxy::Proxy` and `tls::Tls`.

`Stream` kept its name, and so did the private `StreamKind`. The rule governs public items, and inside a module that also names `Stream`, a bare `Kind` in a match arm would say less than the enum it belongs to.

Two guideline rules were repaired on the way, both describing a crate that had moved on. crate-002 still promised that this crate's modules are named after their runtime, which the flatten-modules change ended, and naming-007 still cited `pimalaya_stream::StreamStd` as its example of the exemption.

Capabilities moved: none. This change moved names, not behaviour.
