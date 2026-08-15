---
cairn: change
id: proxy-methods
status: landed
created: 2026-08-15
---

# Hang the proxy module off the type it is about

## Why

The proxy module was a type and four loose functions: `dial`, `resolve_from_env`, `is_bypassed` and `env_var`. Every one of them was about a `Proxy` and nothing else, so each was a method that had not been written, and a reader had to read the whole file to learn which of them a caller was meant to use.

`dial` was also the odd verb out. A stream connects, a proxy dialled, and the same act had two names depending on which layer you were reading.

## What

One `impl Proxy` block: `from_url` and `connect` public, `from_env`, `bypasses` and `env_var` private beneath them. No free functions left in the module.

`dial(host, port, &proxy)` becomes `proxy.connect(host, port)`, which is the verb the stream uses for the same act and reads in the order it happens.

The type keeps its name. `Proxy` sits beside `Tls` as a configuration vocabulary of its own, and the crate name plus the module path already say whose proxy it is.

## Cost

`pimalaya_stream::proxy::dial` is gone; callers reaching for it directly call `Proxy::connect` instead. Inside this crate the connects were the only two callers.
