# Contributing guide

Thank you for investing your time in contributing to pimalaya-stream.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide, which chains to the shared architecture and guidelines;
3. the inline header documentation, starting with src/lib.rs: it is the architecture document of this crate;
4. the [cairn/](./cairn) folder for the development history and living plans (the Cairn convention: spec/, changes/, log/).

Everything below documents only what differs from the Pimalaya standards.

## Cairn

This repository follows [Cairn](https://github.com/pimalaya/cairn): a living spec, reviewable change proposals and a dated log, kept next to the code. Non-trivial work starts with a change folder under cairn/changes, and nothing behavioural is done until its delta is folded into cairn/spec and an entry is appended to cairn/log. The activation stanza is [AGENTS.md](./AGENTS.md).

Every io- protocol crate sits on this transport, so a behaviour change here reaches all of them at once: name the consumers it moves in the proposal.

## Deliberately std

pimalaya-stream wraps TLS providers and sockets and exposes no I/O-free coroutines, so the no_std layer checks of the org guide do not apply: there is no coroutine core to keep std-free. The layers to build against are the feature-gated transport and TLS providers:

```sh
cargo build --no-default-features                          # tls config vocabulary only
cargo build --no-default-features --features std           # + transport and proxy, no TLS
cargo build                                                # rustls-ring (default)
cargo build --no-default-features --features rustls-aws
cargo build --no-default-features --features native-tls
```

There is no async twin planned, so the modules sit flat at the crate root rather than under a runtime one. Should one ever land, that is the decision to revisit first.
