# OmegaOS Orchestration V3: forensic baseline

Date: 2026-07-30
Ticket: OMEGA-ORCH-V3
User need: raise the laws, rules, skills, Oracle sessions, workers, plans, gates, recovery, and reporting to a verifiable state-of-the-art quality bar.
Hinge point: the transition from a worker claiming completion to OmegaOS accepting that completion.

## Executive verdict

The current system has substantial components and a rich operating doctrine, but its critical control path is fail-open. The three independent audit tracks score the affected subsystems between 31/100 and 35/100. They converge on one root cause: mission state, plan state, skill state, and verification state are inferred from several text and file projections instead of being owned by one typed, append-only mission ledger.

The phrase "100% quality" is therefore a release target, not a truthful description of the baseline. It may only be claimed after every acceptance scenario in this report passes at runtime.

## Independent lenses

| Lens | Baseline | Hinge | Primary evidence |
|---|---:|---|---|
| Laws, rules, hooks, context | 34/100 | Replace transcript scraping with a provider-neutral mission ledger | `scripts/hooks/omega_plan_state.py:35-43,107-145`; `crates/omega-core/src/doctor.rs:109-145` |
| Skill lifecycle | 31/100 | Compile every skill surface from one recursive typed catalog | `install.sh:1286`; `crates/omega-core/src/skill_registry.rs:76`; `scripts/omega-skills-atlas.py:53`; `scripts/omega-skills-rag.py:36` |
| Runtime, Oracle, worker, gate | 35/100 | Make `done` a candidate event and require independent acceptance | `crates/omega-core/src/done.rs:218-220,290-313`; `crates/omega-cli/src/main.rs:5756-5811` |

## Reproduced failures

### F1. A no-op worker is accepted

Falsifiable proposition: a worker that changed nothing and supplied no executable proof must not become complete.

Probe:

```bash
probe_dir=$(mktemp -d /tmp/omega-done-probe.XXXXXX)
git init -q "$probe_dir/repo"
git -C "$probe_dir/repo" -c user.name=Probe -c user.email=probe@example.invalid \
  commit --allow-empty -qm baseline
OMEGA_DIR="$probe_dir/state" omega done no-op-worker done_clean \
  "claimed complete without doing work"
```

Observed: the signal was written as `done_clean`, with `todos_total=0`, `todos_completed=0`, the unchanged current Git SHA, and a filesystem check. This follows from `DoneSignal::is_complete` accepting `0 >= 0` and from command, URL, and note citations being treated as true without execution (`crates/omega-core/src/done.rs:218-220,290-313`). The CLI also releases scope immediately (`crates/omega-cli/src/main.rs:5804-5811`).

Verdict: confirmed P0.

### F2. The Codex finish guard does not see Codex plans

Falsifiable proposition: a real Codex session containing `update_plan` calls must be recognized as having a plan.

Probe:

```bash
python3 -c \
  'import sys;sys.path.insert(0,"scripts/hooks");import omega_plan_state as p;print(p.analyze(sys.argv[1]))' \
  "$HOME/.codex/sessions/2026/07/30/rollout-2026-07-30T10-12-08-019fb214-b14c-7390-a667-1822ab319d00.jsonl"
```

Observed:

```text
plan_ever=False total_tasks=0 tool_calls=0
```

The parser expects Claude-style `tool_use` objects while Codex records `function_call` payloads with JSON arguments stored as text (`scripts/hooks/omega_plan_state.py:35-43,107-145`).

Verdict: confirmed P0.

### F3. Equivalent French and English architecture missions are under-routed

Falsifiable proposition: equivalent French and English mission descriptions must receive equivalent routing.

Observed:

```text
French:  SIMPLE, 1 agent, 5 minutes, no decomposition
English: MEDIUM, 1 agent, 20 minutes, no decomposition
```

Neither form correctly recognizes a repository-wide architecture refactor. The classifier scores a small keyword table and stops after the first match in each class (`crates/omega-core/src/routing.rs:153-230`).

Verdict: confirmed P0 for the requested mission.

### F4. The skill catalog is not the installation catalog

Falsifiable proposition: an installed skill must be discoverable by exact semantic lookup and by every supported provider.

Observed:

```text
Repository SKILL.md files: 230
Installed SKILL.md files:   231
Exact query high-end-visual-design: target absent from returned results
Codex shared skill directory ~/.agents/skills: absent
```

The Atlas scanner has special-case traversal rather than a recursive canonical catalog (`scripts/omega-skills-atlas.py:53-78`). `omega sync` wires Claude skills but the Codex branch only generates instructions (`crates/omega-cli/src/main.rs:7636-7721`).

Verdict: confirmed P0.

### F5. Context injection is above a safe operating budget

Observed from the installed runtime:

```text
master: 64,498 bytes
oracle: 72,008 bytes
worker: 71,258 bytes
```

This is before overlapping platform-level instructions are considered. The mission reducer cannot eliminate duplication introduced through a global `AGENTS.md`.

Verdict: confirmed P1.

## Structural contradictions

1. L5 says tokens are unlimited while R-BUDGET imposes a 500K cap (`crates/omega-core/src/rules.rs:150-159,271-280`).
2. R-TEST requires production-only testing, conflicting with safe local/static, preview/integration, and final production verification stages (`crates/omega-core/src/rules.rs:391-400`).
3. The Oracle role says it never edits code, then directs easy work to be done directly (`agents/oracle.md:3-5,123-132`).
4. The canonical dispatch stores only a minimal Oracle state with an empty mission (`crates/omega-core/src/dispatch.rs:66-92`; `crates/omega-core/src/oracle_lifecycle.rs:139-165`).
5. `omega orchestrate` waits for a worker done file while an Oracle writes an Oracle done file (`crates/omega-core/src/orchestration.rs:293-418`; `crates/omega-core/src/session.rs:102-107`).
6. The real quality gate only runs in the duplicate orchestration path, not the canonical Telegram dispatch path (`crates/omega-core/src/orchestration.rs:624-634`; `crates/omega-cli/src/main.rs:6039-6098`).
7. Three plan authorities coexist: harness tasks, progress JSON, and planner tracker (`agents/oracle.md:123-132`; `crates/omega-cli/src/main.rs:5465-5526`; `crates/omega-core/src/planner.rs:133-160`).
8. Skill discovery, Atlas generation, RAG generation, audit registry, install, and provider activation each interpret the catalog independently.

## State-of-the-art constraints

The target architecture follows primary guidance rather than a multi-agent-at-all-costs design:

- Start with the simplest orchestration pattern that satisfies the task, then add agents only when measurable workload or specialization requires it. Source: [OpenAI, A practical guide to building agents](https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/) and [Anthropic, Building effective agents](https://www.anthropic.com/engineering/building-effective-agents).
- Support both manager-as-tools and handoff patterns, selected by task topology. Source: [OpenAI Agents SDK, multi-agent orchestration](https://openai.github.io/openai-agents-js/guides/multi-agent/).
- Bound turns and retries. Source: [OpenAI Agents SDK, running agents](https://openai.github.io/openai-agents-js/guides/running-agents/).
- Pause risky actions as typed approval requests that can be serialized and resumed. Source: [OpenAI Agents SDK, human in the loop](https://openai.github.io/openai-agents-python/human_in_the_loop/).
- Trace agent, tool, handoff, and guardrail events as first-class records. Source: [OpenAI Agents SDK, tracing](https://openai.github.io/openai-agents-js/guides/tracing/).
- Evaluate outcomes over repeated trials using task, transcript, result, and explicit graders. Source: [Anthropic, Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents/).
- Treat MCP tool annotations as untrusted unless the server itself is trusted, and apply least privilege. Source: [MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) and [MCP authorization](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization).

## Target acceptance matrix

| ID | Scenario | Required result |
|---|---|---|
| A1 | No-op worker calls `omega done` | State becomes `candidate_done`, then `correction_required`; scope remains held |
| A2 | Command or URL evidence is declared | Verifier reruns/probes it with timeout and records fresh output |
| A3 | Complex/Epic mission runs end to end | One MissionEngine reaches `accepted` without mismatched done files |
| A4 | Crash at every transition | Exactly-once local transition by idempotency key, replay-safe state, and effectively-once operator-visible delivery |
| A5 | French and English equivalent missions | Same topology, risk, quality gate, and agent budget |
| A6 | Scope aliases and directory overlap | All normalized variants conflict; parallel isolation failure aborts |
| A7 | Provider conformance suite | Identity, permissions, tools, timeout, resume, progress, cancel, and completion pass |
| A8 | Skill catalog compilation | Repo, install, Atlas, RAG, Claude, Codex, and docs have exact parity |
| A9 | Hook fixtures | At least 12 Claude, Codex, and Gemini fixtures pass, including negative verification |
| A10 | Context budget | Active doctrine is below 24 KB with no duplicate rule body |
| A11 | Reporting | No 100% or delivery event exists before `accepted` with fresh evidence |
| A12 | Clean installation | Fresh install, sync, build, tests, and provider smoke checks reproduce the state |

## Before and target architecture

| Surface | Before | Target |
|---|---|---|
| Mission state | Inferred from session names and unrelated JSON projections | SQLite WAL event store as the sole write authority; `MissionRecord` is a rebuildable projection |
| Completion | Worker self-report can close work | Candidate event, independent verifier, fail-closed acceptance |
| Plans | Three authorities | One typed DAG, other surfaces are projections |
| Routing | Keyword score | Bilingual deterministic feature extraction plus risk and topology |
| Provider support | Provider-specific branches and doctrine | Capability contract and conformance suite per adapter |
| Skills | Several independent scanners | One recursive typed `SkillCatalog` compiler |
| Rules | Long global doctrine | Short normative kernel, scoped profiles, runbook references, enforcement metadata |
| Context | 64 to 72 KB before platform overlap | Less than 24 KB, hashed and measured |
| Recovery | Partial session resurrection | Event replay with idempotency and leases |
| Evaluation | Single-run narration | Outcome-based scenario corpus with repeated trials and graders |

## Decision

Proceed with a staged compatibility refactor, not a flag-day replacement:

1. Add the typed contracts and fail-closed primitives behind backward-compatible serialization.
2. Persist events, fenced leases, and an outbox transactionally; use expected-version CAS and idempotency keys.
3. Keep existing CLI and Telegram files as one-way projections only.
4. Add conformance tests that prove the P0 failures are closed.
5. Migrate installation, catalogs, prompts, and provider adapters.
6. Deprecate duplicate protocols only after equivalence tests pass.

The external Claude `/logicaudit` worker could not start because the installed Claude CLI required a fresh OAuth authorization. That integration failure is recorded as evidence of provider readiness, not treated as a passing audit. The actual logicaudit protocol was executed in this session from its full `SKILL.md`, with three independent read-only lenses and root-level reproduction of the P0 probes.

## Remediation status

The release-candidate implementation closes the reproduced control-path defects:

- Completion now enters a task-attempt state machine and requires independent verification before acceptance.
- The finish guard parses Claude tool events and Codex function/custom tool events.
- French and English routing use provider-neutral mission features and risk signals.
- Skill discovery compiles one deterministic `SkillCatalogV1` for install, Atlas, RAG, and provider activation.
- Rule context is compact, provider-aware, measured, and rejected above 24 KB.
- Audit metadata comes from one TOML registry, and the runner fails closed on missing contracts, failed gatherers, invalid evidence, and invalid final verdicts.

This section records implementation, not a final score. The final grade remains evidence-bound to the workspace tests, install verification, runtime smoke checks, report delivery, and pushed commit.
