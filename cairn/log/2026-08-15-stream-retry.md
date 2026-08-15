---
cairn: log
change: stream-retry
landed: 2026-08-15
---

# Retry a stream that is not ready, instead of ending the exchange

A stream reporting `EAGAIN` used to end whatever exchange was in flight above it. It now carries a `StreamRetry` strategy that `Read`, `Write` and `flush` honor: `Until(Duration)` retries a not-ready stream until that long passes without progress, `Never` hands the failure back to a caller whose loop wants it. Streams open on `Until` one minute.

The evidence came from two himalaya reports of the same bare `Resource temporarily unavailable (os error 35)`, one from an IMAP `SORT` fallback fetching a large result and one from a slow `AUTHENTICATE` against a 260k-message mailbox, both on macOS. Nothing in the stack armed a deadline, so the failure came from outside the process; what curl does with it, and what this crate did not, is issue the read again. The policy landed here rather than in io-imap because io-smtp and io-http carry the same loops, and because io-gmail, io-gcal, io-people, io-msgraph and neverest each arm a thirty second read deadline and then treat its expiry as fatal, which is the same bug written on purpose. None of them changes a line to be fixed.

Three decisions are worth keeping. The strategy arms the socket read deadline itself, because a budget cannot be enforced against a server that goes silent while a blocking read waits forever, and a deadline is what turns that silence into something to count. `Interrupted` is retried without spending budget, a signal saying nothing about the stream. And non-blocking mode drops the strategy to `Never`, the two settings contradicting each other outright.

Exhausting the budget now says so, with a `TimedOut` failure naming the elapsed time, where callers used to surface a raw errno. Each retry logs the failure's kind and raw errno at debug level, which is how a spurious kernel `EAGAIN` will be told apart from a TLS layer reporting it has no plaintext ready the next time this is reported.

One follow-up leaves this repository: io-imap's mailbox watcher and io-jmap's watch loop arm a five second deadline to poll a shutdown flag, and select `StreamRetry::Never` when they are bumped, or their wakeup gets absorbed by the retry.

Capabilities moved: retry (new).
