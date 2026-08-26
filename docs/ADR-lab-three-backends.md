# ADR — Omega orchestration mirrors Cursor Cloud Agent

Status: accepted (2026-08-24)
Informed by: Gareth's control-plane direction (2026-08-24). Not a Cursor UI
clone. No Discord connector.

## Decision

Omega orchestration mirrors **Cursor Cloud Agent**:

- **One mission owner.** A follow-up is a `reply` into the live oracle (same
  session / same branch). Never spawn a sibling that edits the same files
  (`oracle-*-2` is a bug, not a fallback).
- **Launch is durable.** The agent session stays in the agent. Death is a
  `failed` session with a reason in JSON — not a silent `bash-5.3$` that
  Omega still calls `running`.
- **Observe without attaching.** `omega oracles`, `omega workers`,
  `omega status --json` (lifecycle only, no pane dump), `omega progress`,
  `omega capture`. Grok Bot is an external orchestrator like the Cursor
  sidebar. It must not need `omega attach` and must not type into OAuth
  wizards.
- **Workers are scoped peers** (files / worktree). Each emits a finish
  report (`done_clean` | `failed` | `blocked` + evidence). They report
  when they finish. The parent oracle verifies. The writer cannot
  gate-accept itself (`omega done` is a candidate; a human runs
  `omega gate --accept`).
- **Done is a visible terminal state + evidence**, not the writer saying
  done.

Cloud / Claude Code / Codex / Hermes are **backends**. The orchestration
API is the same for all of them.

| Backend | Role |
|---|---|
| **Codex** | Mac/VPS writer and default oracle. `omega new --agent codex` must keep Codex alive (same command as TUI New Codex). |
| **Claude / GLM** | Writers and workers. Same launch contract. |
| **Hermes** | Home only (`omega new --agent hermes`). Never `dispatch --agent hermes`, never a worker. |
| **Cloud** | This Cursor Cloud Agent. Writer for OmegaOS itself. Not `omega dispatch`. |

**Grok Bot is an external orchestrator.** Atlas/Telegram is optional. One
oracle per project. Review is outside Omega.

Grok loop (do not skip steps):

1. Observe: `omega oracles`, `omega workers`, `omega status --json`,
   `omega progress` (read-back only). Never dump a pane on `--json`.
2. Write a plan, then `omega dispatch <PROJECT> "<MISSION>"` (default
   Codex). Never `--agent hermes`. Never launch a provider setup wizard.
3. The oracle plans/verifies and never edits. It calls `omega spawn-worker`
   (claude | codex | glm only) with `--dir <project>` and a filled R-RUBRIC
   (Done Criteria + Verify Command). The worker pane starts in that project
   directory: `--dir`, else the oracle `working_dir` (unless that is `$HOME`
   and the project is registered), else the registered project path — never
   rmux `.` / `$HOME`. The parent does not eval Verify Command at spawn.
   Grok must not spawn workers. `--force` is not the path.
4. Reap finish reports. Writer `omega done` is a candidate, not a verdict.
5. A fresh Reviewer lists reasons NOT to merge; Audit if infra/auth/secrets/CI;
   then Afterwork. Gareth alone may `omega gate --accept`.
6. Kill / close the mission.

`omega send` / `omega attach` into a provider/OAuth wizard is forbidden.
Grok Bot is not a fourth Omega backend and not a substitute for `omega send`.

## Launch contract

The pane **is** the agent. Agent exit = session death. Never `; exec bash`
after the agent. `omega new --agent {codex,claude,hermes}` uses the same
`SessionManager::create_session_with_agent` entry as TUI New Codex /
New Claude / New Hermes (`Action::CreateSessionAutoName`). Same argv
(`Agent::try_launch`), no dispatch-authority env on the Home pane,
`--dir ~/…` expanded (a missing directory is a hard error). Codex
itself is not broken: the operator's menu launch stays in the TUI.

Codex unattended pair (0.149+):

```
codex --sandbox workspace-write --ask-for-approval never
```

Invalid: `--approve-for-me` together with `--sandbox` (CLI or
`~/.codex/config.toml` `sandbox_mode`).

Hermes Home: `hermes chat --yolo` (and `HERMES_YOLO_MODE=1` + `HERMES_HOME`).
`install.sh` installs the CLI; `omega sync` wires `~/.hermes`. Never `-q`
for a pane launch.

## Follow-up

Second `omega dispatch <project> "<msg>"` without `--new` replies into the
live oracle. Composer not ready → JSON `followup_pane_not_ready`. The
delivery is persisted and visible in `status --json` (`delivery.tag`).

## Out of scope

Publishing, merge, npm, `omega ship`, second writer, CLIENT tenants,
rewriting the provider catalog again, Discord connector.
