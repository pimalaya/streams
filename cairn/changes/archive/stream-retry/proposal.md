---
cairn: change
id: stream-retry
status: landed
created: 2026-08-15
---

# Retry a stream that is not ready, instead of ending the exchange

## Why

Two himalaya reports, [#731](https://github.com/pimalaya/himalaya/issues/731) and [#732](https://github.com/pimalaya/himalaya/issues/732), end the same way: a bare `Resource temporarily unavailable (os error 35)`, once from an IMAP `SORT` fallback fetching a large result, once from a slow `AUTHENTICATE` against a 260k-message Gmail mailbox. Both reporters are on macOS, both have large mailboxes, and both look like a timeout that nothing in the stack actually sets.

Nothing does set one. The sockets this crate opens are blocking and carry no deadline, so the `EAGAIN` comes from outside: the macOS network stack, or something in the path. What curl does with it, and what this crate did not, is issue the read again. curl drives non-blocking sockets in a select loop and never notices; every io- protocol crate above this one propagated the failure and ended the exchange.

Fixing it in io-imap alone would have left it in place everywhere else. io-smtp and io-http have the same loops, and io-gmail, io-gcal, io-people, io-msgraph and neverest each arm a thirty second socket read deadline and then treat its expiry as fatal, which builds the same failure in on purpose. A grep for `WouldBlock` across those crates returns nothing.

## What

Put the policy in the transport, inside `Read` and `Write` on `Stream`, so no consumer has to call anything to get it.

- A `StreamRetry` strategy on the stream: `Never` hands failures back untouched, `Until(Duration)` retries until that long passes without progress. Default one minute.
- `Read`, `Write` and `flush` honor it. `EAGAIN`, `EINTR` and the Windows spelling of an expired deadline are retried with a small growing pause; everything else is handed back. Exhausting the budget yields `TimedOut` with a message, never a raw errno.
- Selecting `Until` arms the socket read deadline to the same value, without which the budget is unenforceable against a server that goes silent.
- `set_nonblocking(true)` drops the strategy to `Never`, the two settings being contradictory.

Consumers whose loops want the not-ready failures (the io-imap mailbox watcher, the io-jmap SSE loop) select `Never` when they are bumped to this version. Everyone else changes nothing.
