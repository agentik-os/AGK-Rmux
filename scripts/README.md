# Scripts directory — DEPRECATED

The shell scripts that used to live here (`dispatch-to-oracle.sh`,
`dispatch-to-session.sh`, `worker-mark-done.sh`, `close-gate.sh`) have
been **replaced by native Rust** in `crates/omega-core/`.

## Migration map

| Old shell script                | New native command                     |
|---------------------------------|----------------------------------------|
| `dispatch-to-oracle.sh P "M"`   | `omega dispatch P "M"`                 |
| `dispatch-to-session.sh S "P"`  | `omega spawn-worker S "P"`             |
| `worker-mark-done.sh S X "M"`   | `omega done S X "M"`                   |
| `close-gate.sh check-worker S`  | `omega gate <oracle>` (rubric-based)   |

## The full orchestrated pipeline

For end-to-end mission execution (classify → plan → dispatch → monitor
→ quality gate → outcome report), use:

```bash
omega orchestrate <Project> "<mission>"
```

This is the new canonical entry point. It uses
`omega_core::orchestration::Orchestrator` — fully typed, async,
event-driven worker monitoring via the rmux SDK, no shell glue.

## Why this matters

- **No more `bash -c "echo ... | python -c json.dumps"` escaping hell**
- **No more race conditions** between fork/exec/write
- **Real types**: `Mission`, `Plan`, `Outcome`, `WorkerResult`, `GateResult`
- **Real error handling**: `OrchestrationError` with proper variants
- **Tests**: `cargo test -p omega-core` covers the orchestrator
- **One binary** with everything inside — easier to ship and audit
