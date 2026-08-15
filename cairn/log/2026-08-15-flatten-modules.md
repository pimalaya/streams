---
cairn: log
change: flatten-modules
landed: 2026-08-15
---

# Flatten the runtime module away

`stream` and `proxy` lived under a module named after their runtime, held there by a plan for an async twin that would have sat beside them as `tokio`. There is no such twin coming: Pimalaya has no async client to serve and no reason to maintain a second transport, so the nesting was costing every consumer a path segment saying what the crate already says. The three modules now sit flat at the root, `stream`, `proxy` and `tls`, and `src/std/` is gone with the aggregator that held it.

The `std` cargo feature stays and keeps its name. It gates the transport and the proxy selector, so a consumer that wants only the TLS configuration vocabulary still gets it without sockets, which is what io-gmail's default-featureless build takes. The feature names what it enables; it no longer names where the code sits.

The tests split along the same seam. The retry strategy cannot be seen working without a socket, the loop being private, so those four moved to tests/retry.rs and now drive the public API the way a consumer does: a Unix-domain listener, a stream connected to it, and reads and writes across it. A Unix socket rather than TCP because a peer that hung up reports a broken pipe on the first write, where TCP waits for the kernel to stop buffering. What stayed in the crate are the parsing tests in proxy, which need no I/O at all. One assertion did not survive the move: that the default strategy retries for a minute restated its own one-line implementation, and a test that can only fail when someone edits the line above it is not worth the run.

Capabilities moved: none. This change moved paths, not behaviour.
