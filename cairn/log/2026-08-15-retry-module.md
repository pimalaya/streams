---
cairn: log
change: retry-module
landed: 2026-08-15
---

# Give the retry strategy its own module

The stream module had two subjects in it. Beside the transport it exists for, it carried `StreamRetry`, the default timeout, the two backoff bounds and the loop honoring all four, none of which is about opening a socket or wrapping it in TLS. Those four now live in a `retry` module of their own, and the stream module is back to one subject.

The loop stayed a method on `Stream`, since running an operation belongs to the thing holding the socket, but it is written in the retry module as an `impl Stream` block. That works because its operation takes the stream back rather than the socket: `self.retry(|stream| stream.kind.read_once(buf))` closes over nothing private, so the loop can sit outside the module `kind` is private to, and no field had to be widened to `pub(crate)` to let it.

The tests were already split out as tests/retry.rs, so they only changed an import. They still drive the public API over a real socket, which is the only way to see a private loop work.

Capabilities moved: none. This change moved code, not behaviour.
