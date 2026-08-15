---
cairn: tasks
change: retry-module
---

# Tasks

- [x] Move `StreamRetry`, its `Default`, `DEFAULT_RETRY_TIMEOUT` and the two backoff bounds into a `retry` module.
- [x] Move the retry loop with them, as an `impl Stream` block whose operation takes the stream rather than the socket, so the socket stays private to the stream module.
- [x] Declare the module in src/lib.rs, gated once on the `std` feature, and rewrite the Layout paragraph around three modules.
- [x] Repoint the stream module: the import, the option field docs, and the three `Read`/`Write` call sites.
- [x] Repoint the integration tests, which now name the module they cover.
