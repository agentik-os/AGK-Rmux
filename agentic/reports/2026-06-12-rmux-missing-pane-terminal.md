# rmux daemon loses pane terminal while pane process lives ("missing pane terminal")

**Date:** 2026-06-12 · **rmux:** 0.3.1 (rev 2488ef5) · **Host:** macOS Darwin 25.2.0 arm64
**Status:** mitigated (patrol self-heal, OmegaOS 781cb8b) — root cause in rmux NOT fixed; issues disabled on agentik-os/rmux, tracked here.

## Symptom

Sessions `LiquidPad` and `Verba` listed normally in `rmux ls`, but every `capture-pane` / attach / `omega status` failed with:

```
rmux protocol error: server error: missing pane terminal for LiquidPad:0.0
```

`rmux list-panes -a` showed an empty history field for the broken panes only:

```
LiquidPad:0.0: [126x66] [history /,  bytes] %4 (active)     <- broken
Verba:0.0:     [126x66] [history /,  bytes] %5 (active)     <- broken
os:0.0:        [126x66] [history 7/500000, 109714 bytes] %6 <- healthy (same daemon, same minute)
```

The pane child processes were ALIVE and parented to the daemon (ppid == daemon pid 56606; PTYs ttys012/ttys013 existed). Only the daemon's in-memory `PaneTerminal` entry was gone. In the omega TUI this reads as "my project session shows nothing".

## Evidence / timeline

- Daemon started 15:54:13.
- Both broken sessions were RE-CREATES of just-killed names (18:04:04 / 18:04:18); the fresh-named `os` session created 18:04:51 by the same daemon was fine.
- A defunct (zombie) daemon child existed, spawned 18:03:44, ~20 s before the first broken create.

## Suspected mechanism (unconfirmed)

`crates/rmux-server/src/pane_terminal_store.rs` keys terminals by **SessionName** (mutable, reusable), not a stable session id:

- `insert_session` (line ~36) inserts the new map over a stale same-name entry **before** returning the "already exist" error — a half-completed state.
- `remove_session` / `remove_session_terminals` (session_runtime.rs:103) remove **by name**; in a kill+recreate window the old session's cleanup can erase the new session's entry.
- The runtime-owner rename indirection (grouped sessions) adds more name-keyed transitions.

## Repro attempts (all clean — simple races don't trigger it)

- 40 sequential kill + immediate same-name recreate + capture iterations.
- Concurrent stress: 3 create/kill workers + 1 capture/resize prober on a scratch daemon (`-L`).

Real trigger likely involves the SDK path (omega TUI + patrol + Telegram bot were all active at incident time).

## Recovery (runtime-proven)

```
rmux respawn-pane -k -t <session>     # rebuilds the terminal, keeps session + start dir
# then relaunch the agent in the pane (claude --continue picks the conversation back up)
```

## Mitigation shipped

OmegaOS patrol (`crates/omega-core/src/patrol.rs`, commit 781cb8b) sweeps every session each tick; a capture failing with this exact error triggers the respawn + agent `--continue` relaunch automatically.

## Next step for a real fix

Audit the name-keyed lifecycle in `pane_terminal_store.rs` — make `insert_session` atomic (don't insert before erroring), order `remove_session` against same-name re-creates, or key the store by a stable session id.
