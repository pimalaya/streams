---
cairn: change
id: retry-module
status: landed
created: 2026-08-15
---

# Give the retry strategy its own module

## Why

The stream module had grown a second subject. Beside the transport it exists for, it carried the retry strategy, the two backoff bounds, the default timeout and the loop honoring all four, none of which is about opening a socket or wrapping it in TLS. A reader looking for the connect surface had to scroll past a policy, and a reader looking for the policy had to know it was hiding in the transport.

## What

A `retry` module holding everything that answers "what does a stream do when the socket is not ready yet": `StreamRetry`, `DEFAULT_RETRY_TIMEOUT`, the backoff bounds, and the loop itself.

The loop stays a method on `Stream`, since running an operation is the stream's job and not the strategy's, but it is written in the retry module now, as an `impl Stream` block over there. Its `op` takes the stream back rather than the socket, which is what lets it live outside the module the socket is private to.

The tests followed: tests/retry.rs was already the retry file, and now names the module it covers.

## Cost

`StreamRetry` and `DEFAULT_RETRY_TIMEOUT` move one module across for consumers, from `stream` to `retry`.
