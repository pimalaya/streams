---
cairn: change
id: flatten-modules
status: landed
created: 2026-08-15
---

# Flatten the runtime module away

## Why

`std::stream` and `std::proxy` sat under a module named after their runtime, so that an async runtime could one day gain a `tokio` sibling beside them. That day is not coming: Pimalaya has no async client to serve and no appetite for maintaining a second transport, so the nesting buys a path segment and nothing else. Every consumer writes `pimalaya_stream::std::stream::Stream` today, with a `std` in the middle that says only what the crate already is.

## What

The three modules sit flat at the crate root: `stream`, `proxy`, `tls`. `src/std/` is gone, along with the aggregator that held it together.

The `std` cargo feature stays. It gates the transport and its proxy selector, so a consumer wanting only the TLS configuration vocabulary still gets it without pulling in sockets, which io-gmail's default-featureless build does today. It names what it enables rather than where the code lives.

## Cost

Every import moves: `pimalaya_stream::std::stream::…` becomes `pimalaya_stream::stream::…`, and the same for `proxy`. Mechanical, and it lands in the same release as the connect surface it accompanies, so a consumer edits its imports once.
