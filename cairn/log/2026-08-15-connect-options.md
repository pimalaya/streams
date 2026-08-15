---
cairn: log
change: connect-options
landed: 2026-08-15
---

# One way to open a stream, with options per transport

Opening a stream had four entry points for one job: three `connect_*` constructors, a `builder()` with a setter per field, a private `open()` behind both and a private `armed()` behind that. Threading the retry strategy through all four is what made the duplication plain. It is now one `connect_*` method per transport, each taking an options struct of public fields, `TcpConnectOptions { proxy, retry }`, `TlsConnectOptions { tls, proxy, retry }` and `UnixConnectOptions { retry }`, each holding what its transport has and nothing more.

The builder is gone, and so is the vocabulary it introduced. Nothing else in Pimalaya configures a call with chained setters: a command takes an options struct, which is the shape a consumer already meets in `ImapSessionOpenOptions` and its siblings. `open` and `armed` are gone with it, leaving a private `new` that builds the handle and arms the read deadline, the one thing every path must do.

Two mutators went the same way for the same reason. `set_retry` became a public `retry` field: the only mid-connection switch in the org is to `Never`, which arms nothing, so a setter that quietly re-armed the socket bought nothing and could clobber a deadline the caller had just set. `set_nonblocking` went back to `&self` and stopped dropping the strategy behind the caller's back, since with the field public that is one visible line at the call site. The retry loop itself became a private `StreamStd` method taking the operation to attempt, where a module-level function had been standing. It belongs to the stream rather than to the strategy: a strategy is data saying what to do, and running an operation under it is something only the thing holding the socket can do. Its tests moved with it, onto a real socket pair rather than a scripted closure, which is the better bargain anyway: they now cover the whole path from `Read` down to the syscall.

`upgrade_tls` keeps its bare `&Tls`. A proxy means nothing once the socket is open, and the strategy comes from the stream being consumed, so an options struct there would have been a one-field wrapper.

The names moved with the shape. `StreamStd` is now `Stream`: the module already says which transport this is, so the type was repeating it. The private enum behind it is `StreamKind`, which is what it is, and the field holding it is `kind`. Three `impl` blocks became one, public surface first and private helpers under it, and `_upgrade_tls` now returns a `StreamKind` rather than a whole stream, so every connect ends at the same constructor.

That constructor stayed private and did not become public fields. `kind` cannot be public without publishing the `rustls` and `native_tls` types it wraps, which is the one thing the TLS layer of this crate promises never to do, and the constructor is also where the read deadline is armed: a connect that skipped it would promise a budget it could not keep.

The three examples were the only consumers in this repository and moved with it. Everything else moves when it is bumped: io-imap, io-smtp, io-http, io-gmail, io-gcal, io-people, io-msgraph, io-jmap, io-oauth, io-pim-discovery, io-webdav, neverest, mirador, carillon, ortie, m2m, cardamum, linux and sirup all name a connect.

Capabilities moved: connect, retry.
