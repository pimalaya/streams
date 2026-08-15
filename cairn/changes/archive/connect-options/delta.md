---
cairn: delta
change: connect-options
---

## ADDED Requirements

### Requirement: Options per transport

Each connect SHALL take an options struct holding what its transport has and nothing more: a proxy where there is one to route through, TLS settings where there is a session to secure, and the retry strategy everywhere.

Options SHALL be plain structs of public fields with a `Default`, filled in as a struct literal, the shape a consumer already meets in the io- protocol crates.

## MODIFIED Requirements

### Requirement: One connect path

Opening a stream SHALL have exactly one form: a `connect_*` method per transport, taking that transport's options. No builder, and no second entry point wrapping the first.

Every path SHALL end in the same private constructor, the one place that builds the handle and arms the socket read deadline its retry strategy implies.

### Requirement: The deadline the strategy arms

Connecting with `StreamRetry::Until(d)` SHALL arm the socket read deadline to `d`. Without it the budget is not enforceable: a server that goes silent on an otherwise healthy connection blocks the caller in `read` forever.

`StreamRetry::Never` SHALL arm nothing, a caller asking for the not-ready failures being one that arms its own deadline. A caller MAY arm a shorter deadline than its budget, which only means more wakeups.

The strategy SHALL be readable and assignable on a live stream. Assigning `Never` is how a caller takes the not-ready failures back mid-connection; assigning a different `Until` changes the budget alone, the deadline having been armed at connect time, so a caller wanting a matching one sets it beside.

### Requirement: Non-blocking mode

Non-blocking mode SHALL be a plain socket toggle. It contradicts a retrying strategy, a caller reaching for it wanting the `WouldBlock` failures that a strategy would spend its whole budget hiding, so such a caller SHALL assign `StreamRetry::Never` beside it.

## REMOVED Requirements

None.
