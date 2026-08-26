> **Historical (2026-05) — kept for context.** The original Rust-rewrite plan. The rewrite shipped; current reference: [ARCHITECTURE.md](../ARCHITECTURE.md) and [MAP.md](../MAP.md).

# OmegaOS Rust Rewrite — Implementation Plan

> Rewrite the entire VPS Python/bash AISB system in Rust.
> Better, faster, more reliable. Zero Python dependency.

## What Exists (13,352 lines Rust)

Already built and working:
- rmux SDK integration (sessions, panes, send/capture)
- TUI session manager (6 tabs, auto-scroll, mouse, Ctrl+L)
- Agent registry (Claude/Codex/Gemini/Pi/Hermes/GLM)
- AISB Master (auto-spawn, --continue, healthcheck)
- Telegram bridge (long-poll, typing, auto-restart Master)
- PDF generator (4 templates, Telegram delivery)
- Rules registry (6 Laws + 20 Rules, export, sync to all LLMs)
- Config system (config.toml, providers.toml, OMEGA.md)
- Basic dispatch, ship, inbox, patrol, audit, orchestration stubs

## What's Missing (from VPS system)

### Phase 1: Intent & Routing Engine
- Intent parser: text → structured JSON (action, target, criteria, verify_commands)
- Smart routing: keyword detection, multi-project, topic mapping
- Prompt enhancement: casual text → structured oracle prompt with git context
- Project registry: projects.json equivalent, auto-discovery

### Phase 2: Oracle Lifecycle
- Oracle spawn with generated system prompt (per-project context)
- 5-step workflow: Analyze → Dispatch → Monitor → Verify → Report
- Oracle signal file (result.md) and watcher
- God Mode state machine (WORK → VERIFYING → DONE loop)
- Multi-oracle: concurrent oracles, no limit, lazy spawn

### Phase 3: Worker Pipeline  
- Worker dispatch with fresh context template (files, criteria, verify cmd)
- Scope-claim enforcement at dispatch time
- Worker stall detection (30s idle = nudge, 5min = escalate)
- Worker blocked protocol (.worker-blocked.json + fallback action)
- Done signal with structured report

### Phase 4: Quality Gates & Ship
- Rubric generation before execution (R-19)
- Multi-grader consensus (R-21): 3 independent lenses
- Popper falsification (R-30): ≥12 adversarial challenges
- Ship pipeline: build → gitleaks → commit → push → deploy → verify
- Freeze-don't-rollback on deploy failure
- Intent verifier: compare intent JSON vs actual outcome

### Phase 5: Telegram Bot (Full)
- Message handler chain: text, voice, documents, photos, callbacks
- Intent classification: regex fast-path + LLM fallback
- Project detection from DM keywords and topic IDs
- Reply-based routing (message_id → project)
- Inline keyboards (oracle actions, stop workers, close)
- Report pipeline: oracle result → format → send → track replies
- HTML formatting with blockquotes, code blocks, tables

### Phase 6: Skill System
- Skill registry: discover from ~/.omega/skills/
- 17 audit skills as typed invocable units
- Audit orchestrator: auto-select relevant audits per mission
- Audit tracker: freshness, scores dashboard
- Oracle end-of-mission audit hook
- Custom skill creation API

## Oracle Assignment (4 parallel, scope-disjoint)

### Oracle A: Intent & Routing
Files: NEW `crates/omega-core/src/intent.rs`, `crates/omega-core/src/router.rs`
       MODIFY `crates/omega-core/src/routing.rs`
Ref: docs/ARCHITECTURE.md + docs/ORCHESTRATION.md

### Oracle B: Oracle Lifecycle & Workers  
Files: MODIFY `crates/omega-core/src/dispatch.rs`, `crates/omega-core/src/patrol.rs`
       MODIFY `crates/omega-core/src/mission.rs`, `crates/omega-core/src/done.rs`
       NEW `crates/omega-core/src/oracle_lifecycle.rs`
Ref: docs/ARCHITECTURE.md + docs/ORCHESTRATION.md

### Oracle C: Quality Gates & Ship
Files: MODIFY `crates/omega-core/src/gate.rs`, `crates/omega-core/src/ship.rs`
       NEW `crates/omega-core/src/rubric.rs`, `crates/omega-core/src/verifier.rs`
Ref: docs/ARCHITECTURE.md + docs/ORCHESTRATION.md

### Oracle D: Telegram Bot & Skills
Files: MODIFY `crates/omega-cli/src/telegram_bridge.rs`
       MODIFY `crates/omega-core/src/audit.rs`
       NEW `crates/omega-core/src/skill_registry.rs`
       NEW `crates/omega-core/src/formatting.rs`
Ref: docs/ARCHITECTURE.md + docs/ORCHESTRATION.md

## Verification

After all 4 oracles complete:
1. `cargo build --release` = 0 errors
2. `cargo test` = all pass
3. Logic audit of the full pipeline
4. E2E test: Telegram message → intent → oracle → worker → done → report
