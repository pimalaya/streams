---
cairn: delta
change: stream-retry
---

## ADDED Requirements

### Requirement: The strategy is the only choice

A stream SHALL carry a retry strategy, and `Read` and `Write` SHALL honor it. No method of its own SHALL be exposed for it: a caller picks the strategy, and every read, write and flush it already writes behaves accordingly.

`StreamRetry::Never` SHALL hand every failure back untouched. `StreamRetry::Until(d)` SHALL retry until `d` passes without the stream making progress. A stream SHALL open on the default strategy, which retries for one minute.

### Requirement: What counts as not ready

`WouldBlock` (the Unix `EAGAIN`) and `TimedOut` (the Windows spelling, and how an expired read deadline surfaces there) SHALL be retried. `Interrupted` SHALL be retried too, and SHALL NOT count against the budget, a signal saying nothing about the stream. Every other failure SHALL be handed back on the spot.

### Requirement: The budget

Each read and each write SHALL carry its own budget, so a slow but progressing transfer never runs out of it.

Exhausting the budget SHALL fail with `TimedOut` and a message naming the elapsed budget, never a raw errno. Retries SHALL pause between attempts, from one millisecond up to 250, and SHALL log the failure's kind and raw errno at debug level.

### Requirement: The deadline the strategy arms

Selecting `StreamRetry::Until(d)` SHALL arm the socket read deadline to `d`, at open time and whenever the strategy is set afterwards. `StreamRetry::Never` SHALL leave the deadline alone.

### Requirement: Non-blocking mode wins

Turning non-blocking mode on SHALL drop the strategy to `StreamRetry::Never`.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
