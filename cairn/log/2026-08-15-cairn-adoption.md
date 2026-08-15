---
cairn: log
change: cairn-adoption
landed: 2026-08-15
---

# Adopt Cairn, retiring docs/

`docs/` held a README saying there was nothing to hold: the crate was small enough that the src/lib.rs header covered its whole architecture. That stayed true for the architecture and stopped being true for the behaviour, which is a different axis. This repository now keeps `cairn/spec/` (current truth, one file per capability), `cairn/changes/` (in-flight proposals) and `cairn/log/` (dated history), with cairn.toml as the root marker and AGENTS.md (plus CLAUDE.md) as the activation stanza. `docs/` was deleted.

The spec was seeded from the code that already exists, which Cairn normally discourages, done once here because the [stream-retry](./2026-08-15-stream-retry.md) change landing beside it needed something to state its delta against. Three capabilities were backfilled: `connect` (the three transports, the STARTTLS upgrade, the single dial path), `proxy` (tunnels only, resolution from the environment, bypass, proxy URLs) and `tls` (provider selection, trust, ALPN). Each requirement was read out of the code rather than out of intent.

The src/lib.rs header remains the entry point for the code. The spec is the behavioural truth behind it, and the forcing rule now applies: a behaviour change is not done until its delta is folded into the spec and an entry is appended here. It matters more here than in a leaf crate: every io- protocol crate sits on this transport, so a change to it reaches all of them at once.

Capabilities moved: none. This change moved documentation, not behaviour.
