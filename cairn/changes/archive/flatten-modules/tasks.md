---
cairn: tasks
change: flatten-modules
---

# Tasks

- [x] Move src/std/stream.rs and src/std/proxy.rs to the crate root, deleting the aggregator with the directory.
- [x] Declare both modules in src/lib.rs, each gated once on the `std` feature at its declaration.
- [x] Repoint the intra-crate paths and doc links: the proxy import in stream, the connect and upgrade links in tls.
- [x] Update the three examples.
- [x] Rewrite the Layout section of the crate header, which explained the runtime nesting that no longer exists.
- [x] Split the tests: the retry behaviour needs a socket, so it lives in tests/retry.rs and drives the public API, leaving the parsing tests as the only unit tests.
- [x] Write the log entry.
- [x] Record the user-facing half in CHANGELOG.md.
- [ ] Follow-up, outside this repository: move every import when the consumer is bumped.
