> **Historical (2026-05) — kept for context.** A point-in-time gap analysis against the pre-rewrite concept doc ("Archidoc"), which is not in this repo. Current reference: [ARCHITECTURE.md](../ARCHITECTURE.md).

# OmegaOS — Alignment with Archidoc (Concept Document)

> How much of the conceptual architecture (Archidoc.md) is actually built,
> what's missing, and whether each gap is worth closing.

Legend: ✅ done · 🟡 partial · ❌ not built · ⚪ optional/future

## 1. The Three Laws

| Concept | Status | Where |
|---------|--------|-------|
| L1 — Code lies, runtime tells truth | ✅ | rules/L1-runtime-truth.md + gate.rs requires evidence |
| L2 — Researcher not sycophant | ✅ | rules/L2-researcher-not-sycophant.md |
| L3 — Decide and proceed | ✅ | rules/L3-decide-and-proceed.md + oracle_lifecycle.rs |
| Laws embedded in canonical file | ✅ | OMEGA.md §"The Three Laws" |
| Audit that scans prompts for law refs | ❌ | Not built — would need a meta-audit |

## 2. Multi-Agent Hierarchy (L0–L5)

| Level | Concept | Status | Notes |
|-------|---------|--------|-------|
| L0 | Governance (Paperclip registry, fixed reporting lines) | 🟡 | aisb_agents.rs has 14 typed agents but no explicit "governance registry" with reporting lines |
| L1 | Human (Telegram/CLI/TUI) | ✅ | telegram_bridge.rs + omega-cli + omega-tui |
| L2 | Hermès isolated companion (own API key) | ❌ | Hermes is an *installable agent* but NOT wired as an isolated-budget L2 companion |
| L3 | AISB Master | ✅ | aisb.rs — persistent, auto-restart, --continue |
| L4 | Oracle (1 per project) | ✅ | oracle_lifecycle.rs |
| L5 | Workers (ephemeral, scoped) | ✅ | dispatch.rs + scope.rs |

**Gap:** L0 governance registry and L2 Hermès-as-companion are not implemented.
The 14 Matrix agents exist as typed data but the "Paperclip" governance layer
(who reports to whom, fixed lines) is conceptual only.

## 3. The 14 Agents

| Agent | Status | Where |
|-------|--------|-------|
| AISB (Lead) | ✅ | aisb_agents.rs |
| Oracle (Manager) | ✅ | aisb_agents.rs + oracle_lifecycle.rs |
| Morpheus / Construct / Architect / Keymaker / Niobe / Smith / Merovingian / Neo / Zion / Link / Seraph / Pythia / Council | ✅ | aisb_agents.rs (14 typed, with prompts) |
| Hermès (L2 companion) | 🟡 | exists as installable agent, not as isolated companion |

The 14 Matrix agents are present as typed registry entries with prompts.
What's missing is the *runtime wiring* — most are conceptual roles the AISB
Master plays, not separate spawned processes yet.

## 4. Canonical File → Dialects

| Concept | Status | Notes |
|---------|--------|-------|
| OMEGA.md as single source of truth | ✅ | ~/.omega/OMEGA.md |
| Sync to Claude (~/.claude/rules/omega-*.md) | ✅ | omega sync |
| Sync to Gemini (GEMINI.md @import) | ✅ | omega sync |
| Sync to Codex (AGENTS.md symlink) | ✅ | omega sync |
| Per-session chat-contexts/<session>/ with each dialect | ❌ | We do GLOBAL sync, not per-session context dirs |
| Qwen / Aider / OpenCode / Continue.dev / Ollama / LM Studio dialects | ❌ | Not in the sync targets |

**Gap:** The Archidoc envisions per-session `chat-contexts/<label>/` folders each
containing CLAUDE.md/GEMINI.md/QWEN.md/etc. We currently sync globally to each
LLM's home config dir. The global approach is simpler and works; per-session
contexts would allow different rules per session (future enhancement).

## 5. Completion is Derived, Never Declared

| Concept | Status | Where |
|---------|--------|-------|
| .done.json signal | ✅ | done.rs |
| consensus_score ≥ 2/3 (R-21) | ✅ | gate.rs MultiGrader |
| adversarial_pass ≥12 Popper (R-30) | ✅ | gate.rs PopperFalsifier |
| regressions = 0 (R-22) | ✅ | gate.rs RegressionDetector |
| audit_score ≥ 85 gate | ✅ | gate.rs |
| Engine derives completion, agent can't declare | ✅ | oracle_lifecycle.rs reads .done.json |

## 6. rmux Substrate

| Concept | Status |
|---------|--------|
| rmux SDK (typed, no subprocess hangs) | ✅ |
| Persistent detachable sessions | ✅ |
| Each hierarchy level = an rmux session | ✅ |
| Inspectable structured snapshots | ✅ session.rs capture_pane |

## 7. Master Directory Mapping (Python → Rust)

| Concept | Status |
|---------|--------|
| OMEGA.md canonical | ✅ |
| agents/ | ✅ |
| rules/ | ✅ |
| skills/audits/ (18 audits) | ✅ 17 audits |
| providers.toml catalog | ✅ |
| credentials/ (all LLM creds) | ✅ |
| config.toml active provider | ✅ |
| state/ runtime + .done.json | ✅ |
| crates/omega-core (32 modules) | ✅ |
| ~/.local/bin/omega binary | ✅ |

## 8. Mission Lifecycle (10 steps)

| Step | Concept | Status |
|------|---------|--------|
| 1 INTENT | intent.rs classifies | ✅ |
| 2 ROUTE | router.rs | ✅ |
| 3 PLAN | Keymaker rubric + DAG | ✅ rubric.rs + planner.rs |
| 4 DISPATCH | scope-locked worker | ✅ dispatch.rs + scope.rs |
| 5 EXECUTE | Morpheus + .done.json | ✅ |
| 6 AUDIT | Seraph independent | ✅ gate.rs |
| 7 VERIFY | gate computes score | ✅ |
| 8 SHIP | build→deploy→200 (R-14) | ✅ ship.rs |
| 9 REPORT | Link → Telegram | ✅ telegram_bridge.rs |
| 10 LEARN | Smith → Merovingian | 🟡 audit.rs has hooks, learning loop not closed |

## Summary: Alignment Score

| Area | Alignment |
|------|-----------|
| Three Laws | 95% (missing law-ref audit) |
| Hierarchy L0-L5 | 70% (L0 governance + L2 Hermès not wired) |
| 14 Agents | 80% (typed but not all runtime-spawned) |
| Canonical → Dialects | 75% (global sync, not per-session) |
| Completion derived | 100% |
| rmux substrate | 100% |
| Master dir mapping | 95% |
| Mission lifecycle | 90% (learn loop partial) |

**Overall: ~85% aligned with the Archidoc concept.**

## What's Worth Building Next

### High value
1. **Per-session chat-contexts** — let each rmux session have its own
   OMEGA.md mirror, so different sessions can run different rules.
2. **Close the learn loop** (step 10) — Smith extracts lessons, Merovingian
   persists them, feed back into future rubrics.

### Medium value
3. **L0 governance registry** — explicit "who reports to whom" so the
   AISB Master routing is data-driven, not hardcoded.
4. **More dialects in sync** — Qwen, Aider, OpenCode, Continue.dev,
   Ollama, LM Studio (each LLM CLI that reads a context file).

### Optional / future
5. **L2 Hermès as isolated companion** — separate API-key budget so a
   runaway loop doesn't burn the Max subscription.
6. **Law-reference audit** — meta-audit that scans agent prompts to
   confirm they reference the Three Laws.

## What's NOT in the Archidoc but we built anyway (bonus)

- Telegram bot in pure Rust with interactive button menus (/account, /model, /projects, /sessions)
- PDF generator (whitepaper/audit/marketing/doc) with Telegram delivery
- Multi-provider credential centralization with symlink compat
- TUI with 7 tabs (Sessions/Menu/Monitor/Projects/Settings/Agentic/Help)
- OAuth flow with auto-Enter on "Login successful"
- Auto-restart of AISB Master with --continue
