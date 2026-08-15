---
cairn: change
id: drop-prefix
status: landed
created: 2026-08-15
---

# Drop the Stream prefix from the public types

## Why

The crate was of two minds about its own names. `Proxy` and `Tls` stood bare, while `StreamRetry` and the three `Stream*ConnectOptions` carried a prefix, and nothing but the order they were written in explained the difference.

The org guidelines settle it: naming-007 makes the domain prefix strict everywhere except the shared std toolkit crates, this one included, since the crate name and the module path already namespace what they hold. `pimalaya_stream::retry::StreamRetry` says stream three times.

## What

`StreamRetry` becomes `Retry`, `DEFAULT_RETRY_TIMEOUT` becomes `DEFAULT_TIMEOUT`, and the three options structs become `TcpConnectOptions`, `TlsConnectOptions` and `UnixConnectOptions`. `Stream` keeps its name, as do `Proxy`, `ProxyAuth` and the TLS types.

`retry::Retry` reads like `proxy::Proxy` and `tls::Tls`, which is the shape the crate already had in two thirds of its modules.

The private `StreamKind` stays as it is: naming-007 governs public items, and inside a module that also names `Stream`, a bare `Kind` in a match arm says less than the enum it belongs to.

## Cost

Every consumer naming one of the five types edits an import, in the same release that already moves its connect sites.
