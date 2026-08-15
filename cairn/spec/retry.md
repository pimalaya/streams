---
cairn: spec
capability: retry
status: current
---

# Retry

What a stream does when a read or a write reports it is not ready. A blocking socket is not supposed to report `EAGAIN`, yet callers do see one surface mid-exchange, macOS especially and the more readily the longer the exchange runs, and any socket carrying a read deadline reports its expiry the same way. Neither says the session is over.

### Requirement: The strategy is the only choice

A stream SHALL carry a retry strategy, and `Read` and `Write` SHALL honor it. No method of its own SHALL be exposed for it: a caller picks the strategy, and every read, write and flush it already writes behaves accordingly.

`StreamRetry::Never` SHALL hand every failure back untouched. `StreamRetry::Until(d)` SHALL retry until `d` passes without the stream making progress. A stream SHALL open on the default strategy, which retries for one minute.

#### Scenario: A protocol crate that knows nothing about this
- GIVEN a client looping over a coroutine, reading and writing the stream it was handed
- WHEN the socket reports it is not ready in the middle of the exchange
- THEN the read is issued again, and the client sees only the bytes that eventually arrive

### Requirement: What counts as not ready

`WouldBlock` (the Unix `EAGAIN`) and `TimedOut` (the Windows spelling, and how an expired read deadline surfaces there) SHALL be retried. `Interrupted` SHALL be retried too, and SHALL NOT count against the budget, a signal saying nothing about the stream. Every other failure SHALL be handed back on the spot.

#### Scenario: A broken connection
- GIVEN a stream whose peer reset the connection
- WHEN a read is issued
- THEN the failure is reported immediately, without a retry

### Requirement: The budget

Each read and each write SHALL carry its own budget, so a slow but progressing transfer never runs out of it.

Exhausting the budget SHALL fail with `TimedOut` and a message naming the elapsed budget, never a raw errno. Retries SHALL pause between attempts, from one millisecond up to 250, and SHALL log the failure's kind and raw errno at debug level, which is what tells a spurious kernel `EAGAIN` from a TLS layer reporting it has no plaintext ready.

#### Scenario: A server that goes silent
- GIVEN a connection that stays open while the server stops answering
- WHEN the budget elapses
- THEN the caller is told the stream stopped responding after that long

### Requirement: The deadline the strategy arms

Connecting with `StreamRetry::Until(d)` SHALL arm the socket read deadline to `d`. Without it the budget is not enforceable: a server that goes silent on an otherwise healthy connection blocks the caller in `read` forever.

`StreamRetry::Never` SHALL arm nothing, a caller asking for the not-ready failures being one that arms its own deadline. A caller MAY arm a shorter deadline than its budget, which only means more wakeups.

The strategy SHALL be readable and assignable on a live stream. Assigning `Never` is how a caller takes the not-ready failures back mid-connection; assigning a different `Until` changes the budget alone, the deadline having been armed at connect time, so a caller wanting a matching one sets it beside.

#### Scenario: A watcher polling a shutdown flag
- GIVEN a stream whose strategy was set to `Never`, with a five second read deadline of the caller's own
- WHEN nothing arrives for five seconds
- THEN the caller gets the `WouldBlock` it armed the deadline for, and checks its flag

### Requirement: Non-blocking mode

Non-blocking mode SHALL be a plain socket toggle. It contradicts a retrying strategy, a caller reaching for it wanting the `WouldBlock` failures that a strategy would spend its whole budget hiding, so such a caller SHALL assign `StreamRetry::Never` beside it.
