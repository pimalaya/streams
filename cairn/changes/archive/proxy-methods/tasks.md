---
cairn: tasks
change: proxy-methods
---

# Tasks

- [x] Turn `dial` into `Proxy::connect`, taking the target host and port on the proxy it tunnels through.
- [x] Make `resolve_from_env`, `is_bypassed` and `env_var` private associated functions, as `from_env`, `bypasses` and `env_var`.
- [x] Repoint the two connects in the stream module, and drop the `dial` import with them.
- [x] Retire the dial vocabulary from the module header and from cairn/spec/proxy.md, which described the same act in a word the API no longer uses.
- [x] Keep the unit tests in the module, `bypasses` being reachable from them as an associated function.
