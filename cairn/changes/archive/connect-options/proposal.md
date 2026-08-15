---
cairn: change
id: connect-options
status: landed
created: 2026-08-15
---

# One way to open a stream, with options per transport

## Why

Opening a stream had grown four entry points saying the same thing: three `connect_*` constructors, a `builder()` with a setter per field, a private `open()` behind both, and a private `armed()` behind that. Adding the retry strategy meant threading it through all four, which is the moment the duplication stopped being tolerable.

The builder also had the wrong shape for this repository. Nothing else in Pimalaya configures a call with chained setters: a command takes an options struct of public fields, `ImapSessionOpenOptions` and `ImapMessageFetchOptions` being the pattern a consumer already knows. A builder is a second vocabulary for the same job, and one nobody arrives expecting.

## What

One options struct per transport, each holding what that transport actually has, and one `connect_*` method per transport taking it.

- `StreamTcpConnectOptions { proxy, retry }`, `StreamTlsConnectOptions { tls, proxy, retry }`, `StreamUnixConnectOptions { retry }`, all public fields with `Default`.
- `connect_tcp`, `connect_tls` and `connect_unix` each take theirs by value. `upgrade_tls` keeps its bare `&Tls`: a proxy means nothing once the socket is open, and the strategy comes from the stream being consumed.
- `StreamBuilder`, `StreamStd::builder`, `open` and `armed` are gone. A private `new` builds the handle and arms the deadline, which is the only thing every path must do.
- `retry` becomes a public field on `Stream`, replacing `set_retry`. The one mid-connection switch that exists in the org is to `Never`, which arms nothing, so a setter that quietly re-armed the socket bought nothing and could clobber a deadline the caller had set.
- `set_nonblocking` goes back to `&self` and stops mutating the strategy. With `retry` public, a caller writes `stream.retry = StreamRetry::Never` beside it, in one visible line, rather than having it done behind its back.
- `StreamStd` becomes `Stream` and the private enum behind it becomes `StreamKind`. The module already says which transport this is, so the type was repeating it; the enum is a kind, and now says so.
- One `impl` block instead of three, public surface first and the private helpers under it.

## Cost

Every connect site in the org moves: io-imap, io-smtp, io-http, io-gmail, io-gcal, io-people, io-msgraph, io-jmap, io-oauth, io-pim-discovery, io-webdav, neverest, mirador, carillon, ortie, m2m, cardamum, linux, sirup. The move is mechanical, and this is the release to make it in.
