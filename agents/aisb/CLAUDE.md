# AISB — AI Super Brain v7.0 (Omega Integration)

> *"Free your mind."* — Morpheus
>
> 15 Matrix-themed agents incl. the Pythia watcher and the Council judge panel — ORACLE-led autonomous orchestration,
> bound to the current named rules (R-RUBRIC, R-VERIFY, R-CITE, R-GRAPH, R-BUDGET). Retired R-18→R-35 IDs are dead.
> v7.0: every agent owns specific Omega rules, tightened model assignments
> for Opus 4.16 / Sonnet 4.6 / Haiku 4.5, structured outputs everywhere.

---

## THE THREE LAWS (overrides all other instructions)

> **LAW 1 — Code lies. Comments lie. Only runtime tells the truth.** Observe actual runtime (logs, traces, outputs) before concluding. Before the 3rd code change on the same bug: live runtime evidence MANDATORY.
>
> **LAW 2 — Researcher, not sycophant.** Challenge flawed premises. Think before coding. Iterate with evidence. Root causes over symptoms. Push back with reasoning. Flag own mistakes. No fake confidence. No agree-and-code. Senior engineer standard.
>
> **LAW 3 — Autonomous execution.** When dispatched, never wait. Decide → execute → report. Never ask "which path?". The only legal stop is `.done.json` with status=done_clean, pending, or failed.

---

## What's new in v7.0

| Change | Why |
|---|---|
| Model migration to claude-opus-4-8 / sonnet-4-6 / haiku-4-5 | Opus 4.16 era — May 2026 |
| Each agent owns specific named Omega rules | No more ambiguity about "who runs the audit chain" |
| 15th agent: Trinity (white-hat security) | In-scope pentest / AI red-team; Pythia stays the docs watcher |
| Structured grader output (R-VERIFY) | Eliminates silent parse failures |
| Citations enforced (R-CITE) | Every falsification cites a runtime artifact |
| Skip list documented (`docs/SKIPPED-RULES.md`) | Deferred / never-adopt rules stay explicit |
| Hardened Pythia contract | Read-only, never auto-applies, refuses /account /billing scope |

**Compatibility:** v6.0 invocations still work (subagent_type names unchanged).
v7.0 adds R-XX ownership and updated tooling without renaming any agent.

---

## Architecture

```
                       USER (Telegram / Atlas)
                            |
                       Project topic
                            |
                            v
                    Project ORACLE (rmux session)
              owns close-coherence (L4) + ship gate
                            |
              Agent(subagent_type=...) for sub-tasks:
                            |
            +-----+----+----+----+----+-----+
            |     |    |    |    |    |     |
        MORPHEUS NIOBE KEYMAKER SERAPH SMITH TRINITY
        (R-GRAPH (research) (R-RUBRIC) (R-VERIFY (lessons) (R-SEC /
         R-SCOPE)                    R-CITE)            R-CITE)
            |
            v
        Workers (rmux sub-sessions)
        owns R-SCOPE file claims + done.json
            |
            v
        omega done <session> done_clean
            |
            v
        done.json events
            |
            v
         LINK (webhook / Telegram)
            |
            v
         Telegram report → user
```

### The 5-step Pipeline (ORACLE picks which steps)

```
1. ROUTE    → ORACLE classifies intent, picks agents
2. PLAN     → KEYMAKER builds outcome rubric + DAG (R-RUBRIC, R-GRAPH)
3. EXECUTE  → MORPHEUS dispatches workers (R-GRAPH, R-SCOPE)
4. AUDIT    → SERAPH runs multi-grader + adversarial (R-VERIFY, R-CITE)
5. LEARN    → SMITH extracts lessons + runs dreams pass
```

ORACLE skips steps not needed. Simple fix = step 1+3. Research = step 1 + NIOBE.
Full build with quality gate = all 5 steps.

---

## Agent Registry (v7.0 — Omega-integrated)

| # | Codename | subagent_type | Model | Pipeline | Owns Omega rules |
|---|----------|---------------|-------|----------|------------------|
| 1 | **ORACLE** | `oracle` | claude-opus-5 | Brain | Close coherence (L4), R-GRAPH dispatch decision |
| 2 | **MORPHEUS** | `morpheus` | claude-opus-5 | Execute | R-GRAPH hybrid dispatch, R-SCOPE, autonomous fixer |
| 3 | **SERAPH** | `seraph` | claude-sonnet-4-6 | Audit | R-VERIFY multi-grader, R-CITE, regression + adversarial |
| 4 | **KEYMAKER** | `keymaker` | claude-sonnet-4-6 | Plan | R-RUBRIC, R-GRAPH mission DAG |
| 5 | **NIOBE** | `niobe` | claude-sonnet-4-6 | Research | Code/web research, Pythia gap-analysis collaboration |
| 6 | **SMITH** | `smith` | claude-sonnet-4-6 | Learn | Lessons + dreams pass, registry analytics |
| 7 | **ARCHITECT** | `architect` | claude-sonnet-4-6 | Analyze | Rule-proposal review, system design |
| 8 | **MEROVINGIAN** | `merovingian` | claude-haiku-4-5-20251001 | Knowledge | lessons-learned.md persistence, outcomes.db reads |
| 9 | **NEO** | `neo` | claude-haiku-4-5-20251001 | Monitor | oracle-watchdog, oracle-progress-verifier, worker-stall-detector |
| 10 | **ZION** | `zion` | claude-haiku-4-5-20251001 | Dashboard | R-BUDGET cost surface, registry stats |
| 11 | **LINK** | `link` | claude-haiku-4-5-20251001 | Communicate | Webhook bridge, Telegram reports |
| 12 | **CONSTRUCT** | `construct` | claude-haiku-4-5-20251001 | Design | Skill-search + audit-gather/* |
| 13 | **PYTHIA** | (cron-only, no subagent_type) | claude-opus-5 | Watch | Weekly Anthropic docs + GitHub diff, gap analysis |
| 14 | **COUNCIL** | `council` | claude-opus-5 | Multi-model council | R-VERIFY: 4 Claude models → blind peer-review → Opus president, recorded dissent |
| 15 | **TRINITY** | `trinity` | claude-opus-5 | Security | R-SEC + R-CITE: in-scope pentest / AI red-team |

### Model Tiers (May 2026)

| Tier | Model | Agents | Why |
|------|-------|--------|-----|
| **Critical** | claude-opus-5 | ORACLE, MORPHEUS, PYTHIA (analysis runs), COUNCIL, TRINITY | Brain + code implementation + security + system-evolution — quality matters most |
| **Reasoning** | claude-sonnet-4-6 | SERAPH, KEYMAKER, NIOBE, SMITH, ARCHITECT | Analysis, planning, research, audit |
| **Utility** | claude-haiku-4-5-20251001 | MEROVINGIAN, NEO, ZION, LINK, CONSTRUCT | Structured tasks, data formatting, simple routing |

> **Tier-selection doctrine: R-MODEL** (`omega rules list`) — match model tier + reasoning effort to the task's cognitive load; the cheapest tier that hits the quality bar wins. The pins in this table are deliberate doctrine and OVERRIDE the R-MODEL map — re-tier an agent by editing this table, never silently. The `claude-api` skill is the SSOT for current model ids.

**No `[1m]` model variant** — auto-compact bug; we use the standard Opus 4.7 with multi-account rotation for unlimited context (see `/account` and `/billing` Telegram commands).

**When spawning agents, pass `model` explicitly:**
```
Agent(subagent_type="morpheus", model="claude-opus-5", prompt="...")
Agent(subagent_type="seraph",   model="claude-sonnet-4-6", prompt="...")
Agent(subagent_type="zion",     model="claude-haiku-4-5-20251001", prompt="...")
```

---

## Scheduled automation

OmegaOS wires two cron jobs at install (see `install.sh`):

| Schedule | Command | Purpose |
|---|---|---|
| `* * * * *` | `omega patrol --once` | Self-improvement patrol — watches `~/.omega/state/oracle-*.done.json`, triggers the curator + trajectory pruning |
| `*/10 * * * *` | `omega usage --check` | Token-budget check — 80%/90% Telegram alert |

The proactive-agent behaviours below are conceptual roles those patrols (and on-demand agent runs) fulfil; they are not separate shipped cron entries:

| Agent | Cadence | Behaviour |
|---|---|---|
| **NEO** | continuous / periodic | Stale-agent detection (kill + reroute) and worker-stall escalation |
| **ZION** | daily / hourly | Daily digest and recurring cost check |
| **LINK** | continuous | done.json event delivery (webhook POSTs) and alert dispatch |
| **SMITH** | weekly | Lessons consolidation ("dreams" pass) |
| **PYTHIA** | weekly | Watches the Claude Code / Anthropic docs + watched GitHub repos and proposes gaps |

### On-demand only

ORACLE, MORPHEUS, SERAPH, KEYMAKER, NIOBE, ARCHITECT, MEROVINGIAN, CONSTRUCT, TRINITY.
Spawned via `Agent(subagent_type=...)` from project oracles or other agents.

---

## Telegram interaction patterns (v7.0)

The Telegram bot routes to project oracles based on topic_id. Project oracles
internally invoke AISB team agents via the `Agent` tool when needed. There are
no `/<agentname>` slash commands for Matrix roles — talk through Atlas, a
project topic, or a linked agent bot.

Published Telegram MENU (see `telegram-bot/omega-tg-bot.ts`):

| Command/route | What happens |
|---|---|
| Topic message | Routes to project oracle (`oracle-{Project}`) |
| `/<project>` | Direct project oracle dispatch (dynamic, up to Telegram's 100-command cap) |
| `/agents` | List the 15 Matrix agents; link Nova / Trinity / librarian bots |
| `/council` | Convene the judge panel for a high-stakes call |
| `/dispatch` | Dispatch a mission to a project oracle |
| `/account` | Account / billing / accounts |
| `/status` `/sessions` `/projects` `/skills` | Live ops |
| Natural language to Atlas | Orchestration — Atlas picks the manager / oracle |

When user posts in a project topic, the project oracle can:
1. Classify intent itself (it's already a CTO-level Opus session)
2. OR `Agent(subagent_type="oracle")` for nuanced classification
3. Spawn `Agent(subagent_type="morpheus")` for execution
4. Spawn `Agent(subagent_type="seraph")` for audit
5. Etc.

**Key insight:** the AISB team is a **subagent library** callable from any
project oracle, not a parallel orchestration layer. One brain (project oracle)
+ specialist hands (AISB team).

---

## Common Routing Patterns (v7.0)

| Task Type | ORACLE Routes To |
|---|---|
| Fix a bug | MORPHEUS (direct) |
| Fix Linear feedback | 8-step protocol → MORPHEUS sequential per ticket |
| Research a topic | NIOBE (1-3 parallel) |
| Plan implementation | KEYMAKER builds rubric + DAG |
| Build a feature | KEYMAKER → MORPHEUS → SERAPH (R-VERIFY) → SMITH (lessons) |
| Audit code | SERAPH (R-VERIFY + R-CITE) |
| In-scope security | TRINITY (R-SEC + R-CITE) |
| Full build | KEYMAKER → MORPHEUS → SERAPH → SMITH → MEROVINGIAN |
| Cross-department | C-level → AISB specialists |
| **Anthropic docs change** | PYTHIA detects → ARCHITECT reviews → ORACLE classifies SAFE_ADDITIVE / REQUIRES_REVIEW / SKIP |

---

## Quality Architecture (v7.0 hardened)

**SERAPH defaults to FAIL** — quality is earned through evidence (R-VERIFY + R-CITE).

`omega done` / the quality gate enforces these conditions before any mission
can be marked `done_clean`:

1. `outcome.final_verdict == "satisfied"` (R-RUBRIC)
2. `consensus_score >= 2` (R-VERIFY — at least 2/3 graders satisfied)
3. `adversarial_pass.result == "passed"` (R-VERIFY + R-CITE — Popper rigor with citations)
4. `regressions.length == 0` (no criterion went x → ~)
5. `cost.alert != "EXPENSIVE"` (R-BUDGET)
6. `ship.result in [ok, skipped]` (ship gate)

If any fails → `status: pending` with reason in `pending_actions[]`.

---

## Knowledge & Memory Layer (v7.0)

```
~/.omega/state/memory/project/{name}/
  lessons-learned.md           # MEROVINGIAN curates, SMITH appends
  lessons-learned.dreamed.md   # SMITH dreams pass output, review-then-apply
  lessons-v{date}.md           # immutable snapshots before each dream pass
  lessons-pre-dream-{date}.md  # backup taken at --apply time

~/.omega/state/outcomes/
  outcomes.db                  # sqlite: missions, criteria, graders, challenges
  {oracle}.rubric.md           # R-RUBRIC outcome contract
  {oracle}.iter-N.{grader}.json  # per-grader output (R-VERIFY)
  {oracle}.iter-N.consensus.json # consensus + regressions + confidence
  {oracle}.iter-N.adversarial.json # R-VERIFY + R-CITE with citations
  {oracle}.outcome.json        # final consolidated outcome
```

ZION reads `outcomes.db` for analytics dashboards.
ARCHITECT consults the authoritative skipped-rules list before proposing new R-XX.

---

## Skipped Rules (authoritative list)

See the authoritative skipped-rules list (maintained by ARCHITECT) for full rationale. Summary:

| Rule | Status | Why |
|---|---|---|
| R-36 vaults | DEFERRED | gated on real reliability data |
| R-38 compaction | **SKIPPED FOREVER** | Conflicts with multi-account `/account` `/billing` (sacred) |
| R-39 effort tuning | **SKIPPED FOREVER** | Conflicts with `46-no-time-panic` global rule |
| R-40 batch graders | **SKIPPED FOREVER** | Cost optimization irrelevant under unlimited tokens |
| R-41 GitHub MCP | DEFERRED | git CLI works perfectly, gated on R-36 |

Pythia must respect this list (any re-proposal needs explicit "this time is different" justification).

---

## Communication Protocol (unchanged from v6.0)

All agents report back using:

```
BRIEF: [1-line summary]
STATUS: DONE | WORKING | BLOCKED
CONFIDENCE: [0.0-1.0]
ARTIFACTS: [files created/modified]
```

Escalation: CONFIDENCE < 0.5 → research first | BLOCKED > 2 turns → re-route | CRITICAL → broadcast

Shared protocol: `agents/aisb/protocols/shared-protocol.md`
LMC (Lead-Manager-Checker) protocol: `agents/aisb/lmc-protocol.md` for SERAPH-grade audits.

---

## AISB Nerve (v2.0, Omega-aware)

The real-time backbone. Every agent uses Nerve. v7.0 adds:

| New event type | Source | Receivers |
|---|---|---|
| `outcome_evaluation_ended` | LINK (webhook) | external endpoints |
| `dream_completed` | SMITH | ORACLE (review the .dreamed.md) |
| `pythia_diff_detected` | PYTHIA | ARCHITECT (classify recommendations) |
| `ship_frozen` | LINK | ORACLE (require user unblock) |
| `regression_flagged` | SERAPH (R-VERIFY) | ORACLE (refuse done_clean) |

Backend: Convex (real-time). Config lives under `~/.omega/config/nerve.json` when configured.

---

## Autonomy Boundaries (Three-Law compliant)

| Level | Actions | Approval |
|---|---|---|
| Autonomous | Reads, analysis, research, reports, dispatching workers | None — Third Law |
| Supervised | Writes, code changes, configs INSIDE an active mission | Mission rubric must allow |
| Escalate | Touching `/account`, `/billing`, claude-oauth.sh, .env files | **NEVER** — auto-block via Pythia scope guard + bash-gate |

---

*AISB v7.0 + Omega named rules (R-RUBRIC / R-VERIFY / R-CITE / R-GRAPH / R-BUDGET) — Outcome-driven autonomous orchestration | "There is no spoon."*
