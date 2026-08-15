---
cairn: log
change: proxy-methods
landed: 2026-08-15
---

# Hang the proxy module off the type it is about

The proxy module was a type and four loose functions, `dial`, `resolve_from_env`, `is_bypassed` and `env_var`, every one of them about a `Proxy` and nothing else. They are now one `impl Proxy` block: `from_url` and `connect` public, `from_env`, `bypasses` and `env_var` private beneath them. Nothing floats at module level any more, so what a caller is meant to reach for is the two public methods and the rest is visibly plumbing.

`dial` went with them, and took its verb along. A stream connects; a proxy dialled; the same act had two names depending on which layer you were reading. `dial(host, port, &proxy)` is now `proxy.connect(host, port)`, which also reads in the order it happens. The word is gone from the module header and from cairn/spec/proxy.md, which had described the behaviour in a vocabulary the API no longer uses.

The type kept its name after a look at `StreamProxy`. `Proxy` sits beside `Tls` as a configuration vocabulary in its own right, not a part of the stream, and neither wants the prefix that `StreamRetry` and the connect options carry precisely because those are the stream's own.

Capabilities moved: none. The proxy spec changed wording, not requirements.
