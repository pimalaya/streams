---
cairn: tasks
change: drop-prefix
---

# Tasks

- [x] Rename `StreamRetry` to `Retry` and `DEFAULT_RETRY_TIMEOUT` to `DEFAULT_TIMEOUT`.
- [x] Rename the three options structs to `TcpConnectOptions`, `TlsConnectOptions` and `UnixConnectOptions`.
- [x] Repoint the doc links, the examples and the integration tests.
- [x] Fold the new names into cairn/spec, the archived changes and the log entries, none of which had shipped yet.
- [x] Record the rename in CHANGELOG.md.
- [x] Fix the two guideline rules this crate had outgrown: crate-002 still promised a runtime-named module, and naming-007 still cited `pimalaya_stream::StreamStd`.
