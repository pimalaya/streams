---
cairn: tasks
change: connect-options
---

# Tasks

- [x] Add the three connect options structs, public fields with `Default`.
- [x] Take them in `connect_tcp`, `connect_tls` and `connect_unix`, each dialling and wrapping for itself.
- [x] Delete `StreamBuilder`, `StreamStd::builder`, `open` and `armed`, leaving a private `new` that builds the handle and arms the deadline.
- [x] Make `retry` a public field on `Stream` and delete `set_retry`.
- [x] Give `set_nonblocking` its `&self` back and stop mutating the strategy from it.
- [x] Move the retry loop onto `Stream` as a private method taking the operation to attempt, inlining the one-call-site transient check.
- [x] Rewrite the retry tests over a real socket pair, the loop no longer being a free function a test can call on its own.
- [x] Rename `StreamStd` to `Stream`, the private enum to `StreamKind` and its field to `kind`, and name the retry loop `retry` after the verb it performs.
- [x] Merge the three `impl` blocks into one, public surface first and private helpers under it, with `_upgrade_tls` returning a `StreamKind` so construction happens in one place.
- [x] Update the three examples, which were the only in-repo consumers of the builder.
- [x] Fold the delta into cairn/spec/connect.md and cairn/spec/retry.md, and write the log entry.
- [x] Record the user-facing half in CHANGELOG.md.
- [ ] Follow-up, outside this repository: move every consumer's connect site onto the options structs when it is bumped.
