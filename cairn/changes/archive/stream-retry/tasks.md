---
cairn: tasks
change: stream-retry
---

# Tasks

- [x] Add `StreamRetry` and `DEFAULT_RETRY_TIMEOUT` to the std stream module.
- [x] Split the per-variant I/O into one-attempt `read_once` / `write_once` / `flush_once`, and put the strategy in one `retrying` helper the three `Read`/`Write` methods share.
- [x] Arm the socket read deadline the strategy implies when a stream is opened, and again whenever the strategy is set.
- [x] Expose the strategy on the builder and on a live stream.
- [x] Drop the strategy to `Never` when non-blocking mode is turned on.
- [x] Cover the policy with tests: a transient failure retried, a stream that never becomes ready, a broken stream reported on the spot, `Never` passing the failure through, and the default budget.
- [x] Fold the delta into cairn/spec/retry.md and write the log entry.
- [x] Record the user-facing half in CHANGELOG.md.
- [ ] Follow-up, outside this repository: io-imap's mailbox watcher and io-jmap's watch loop select `StreamRetry::Never` when they are bumped, their `WouldBlock` wakeup being a shutdown poll rather than a failure.
