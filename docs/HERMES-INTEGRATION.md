# Hermes Agent → OmegaOS Integration Report

> **Historical research, not runtime documentation.** This May 2026 document
> proposes mechanisms that were not necessarily implemented. The current CLI
> adapter lives in `crates/omega-core/src/agents.rs`; supported versions and
> commands are documented in
> [PROVIDER-COMPATIBILITY.md](PROVIDER-COMPATIBILITY.md).

**Source**: https://github.com/nousresearch/hermes-agent
**Target**: OmegaOS (Rust + Bun)
**Date**: 2026-05-28
**Author**: research worker

---

## 1. Hermes self-improvement systems — inventory

Hermes is a Python agent, not a Rust one. Most of what is interesting for OmegaOS is **mechanism-level** (algorithms, prompts, data shapes), not code we can vendor. Below is every distinct self-improvement / introspection / learning subsystem I found, with its source file.

### 1.1 Background review (the actual "self-improvement loop")
- **What**: After each conversation turn, a forked agent instance replays the recent snapshot with a whitelisted toolset (memory + skill management only) and asks "should any skill / memory be saved or patched?". It runs in a daemon thread, inherits the parent's provider + cached system prompt (so it hits the prefix cache), and writes directly to the memory/skill stores. The user only sees a compact line like `💾 Self-improvement review: Memory updated · Skill patched`.
- **Triggers**: user revealed persona, expressed expectations, corrected style/tone, frustration signals ("stop doing X"), non-trivial workaround discovered, previously loaded skill turned out wrong.
- **Update preference order**: patch loaded skill → update umbrella skill → add support file → create new class-level skill (last resort).
- **File**: `agent/background_review.py` (functions `_run_review_in_thread`, `spawn_background_review_thread`, `summarize_background_review_actions`).
- **Generic?**: Yes. Just needs a memory store, a skill store, and a second LLM call. No Nous-specific dependency.

### 1.2 Procedural memory / skill library
- **What**: Skills are markdown files (`SKILL.md` + support assets) under `~/.hermes/skills/<bundle>/<skill>/`. A bundle is a YAML grouping (`~/.hermes/skill-bundles/*.yaml`). Hot-swap is mtime-based — the disk is the source of truth. Skills are invoked as slash commands (`/skill-name`) and resolved hyphen-or-underscore.
- **Preprocessing**: template substitution (`${HERMES_SKILL_DIR}`, `${HERMES_SESSION_ID}`) and inline shell expansion (``!`cmd` `` is replaced by stdout, run in the skill dir).
- **Selection**: name-based, not embedding-based — keyword/slug match. (I expected vector retrieval; there isn't one in `skill_preprocessing.py` or `skill_commands.py`. The model picks which `/skill` to invoke via tool calls.)
- **Files**: `agent/skill_commands.py`, `agent/skill_bundles.py`, `agent/skill_preprocessing.py`, `agent/skill_utils.py`, plus the 24 bundle directories under `/skills/` (apple, devops, github, red-teaming/godmode, software-development, …).
- **Generic?**: Yes. Identical to `~/.claude/skills/` and `~/.claude/commands/` shape. OmegaOS already has the substrate.

### 1.3 Trajectory capture (ShareGPT JSONL)
- **What**: Every conversation can be saved in **ShareGPT format** for downstream evaluation or fine-tuning. `<REASONING_SCRATCHPAD>` is converted to `<think>` for standardization. Incomplete scratchpads are detected and routed to `failed_trajectories.jsonl` instead of `trajectory_samples.jsonl`.
- **Fields**: `conversations` (ShareGPT list), `timestamp`, `model`, `completed`.
- **File**: `agent/trajectory.py`.
- **Generic?**: Yes — but it's only valuable if you actually train. For Omega, this is the foundation of a **prompt-evolution corpus**, not a model-training one.

### 1.4 Insights engine (SQLite analytics)
- **What**: Reads a SQLite session DB and emits a token/cost/tool-usage/skill-usage/activity-pattern report. Functions: `_get_sessions`, `_get_tool_usage`, `_get_skill_usage`, `_get_message_stats`, `_compute_overview`, `_compute_model_breakdown`, `_compute_platform_breakdown`, `_compute_activity_patterns`. Two formatters: terminal and gateway (markdown).
- **File**: `agent/insights.py`. Entrypoint `generate(days=30, source=None)`.
- **Generic?**: Yes. Drop-in equivalent for `~/.omega/state/metrics.db` (Omega already has `metrics.rs`).

### 1.5 Error classifier + failover taxonomy
- **What**: `FailoverReason` enum with ~20 codes (auth, billing, rate_limit, overloaded, context_overflow, payload_too_large, thinking_signature, long_context_tier, oauth_long_context_beta_forbidden, llama_cpp_grammar_pattern, …). `classify_api_error(exc)` returns `(reason, retryable, should_compress, should_rotate_credential, should_fallback)`. Priority pipeline: provider-specific patterns → HTTP status → structured codes → message patterns → transport/SSL → server disconnect → timeout → unknown.
- **File**: `agent/error_classifier.py`.
- **Generic?**: Yes, and it's the most directly portable file in the whole repo.

### 1.6 Context compressor
- **What**: Triggers when token count > threshold AND last two passes each saved ≥10% (anti-thrashing). Preserves system prompt + first exchange (head) and ~20K tokens of tail. Middle is summarized to a `Goal / Completed Actions / Active State / Remaining Work` template. Old tool outputs become `[read_file] read config.py from line 1 (1,200 chars)`. Images become `[Attached image — stripped after compression]`. Secrets are redacted before summarization.
- **File**: `agent/context_compressor.py`.
- **Generic?**: Yes. Highly valuable for Omega oracle sessions that hit Claude's 200K limit.

### 1.7 Iteration budget
- **What**: Thread-safe counter. Parent default 90, subagent default 50. Programmatic tool calls (e.g. `execute_code`) refund their iteration. `consume()` returns False when exhausted.
- **File**: `agent/iteration_budget.py`.
- **Generic?**: Yes. Maps cleanly to Omega's existing `--max-turns` / `/goal` budget controls.

### 1.8 Prompt caching strategy (Anthropic)
- **What**: `system_and_3` layout — 4 cache_control breakpoints (system + last 3 non-system messages) at one TTL (5m or 1h). Claims ~75% input-token cost reduction on multi-turn.
- **File**: `agent/prompt_caching.py`.
- **Generic?**: Yes. Useful for the AISB Master session and every Oracle.

### 1.9 Memory provider abstraction
- **What**: Single integration point with at most ONE external plugin provider + the built-in. Interface: `prefetch(query)`, `sync_turn(user, assistant)`, `get_tool_schemas()`, `handle_tool_call(name, args)`. A `StreamingContextScrubber` strips memory-context fence tags from streamed output.
- **File**: `agent/memory_manager.py`, `agent/memory_provider.py`.
- **Generic?**: Yes. Clean trait shape — translates directly to a Rust `trait MemoryProvider`.

### 1.10 Conversation loop with self-healing retries
- **What**: One `while` loop, multiple retry classes: empty-content, invalid-JSON tool args (with `_sanitize_tool_call_arguments` repair), incomplete-scratchpad, length-continuation, plus proactive `should_compress` check before each call. Interrupts allow breaking on new user input.
- **File**: `agent/conversation_loop.py`.
- **Generic?**: Yes — but Omega already has equivalents via `claude_stream.rs` and dispatch retries. Worth comparing line-by-line, not vendoring.

### 1.11 What is NOT here
There is **no training loop**, **no RL/SFT pipeline**, **no embedding-based skill retrieval**, **no model-rewriting-itself loop**. Hermes's "self-improvement" is **memory + skill curation by an LLM-as-curator**, not gradient updates. Trajectory JSONL exists but the training is presumably out-of-band on Nous's own infra. Don't expect a magic auto-evolution box.

---

## 2. Mapping table — Hermes → OmegaOS

| # | Hermes mechanism | OmegaOS surface | New file path (proposed) | Effort | Dependencies / blockers |
|---|---|---|---|---|---|
| 1 | Background review thread | Post-`oracle-*.done.json` hook → forked Claude session with whitelisted skill+memory tools | `crates/omega-core/src/review.rs` + `~/.omega/skills/_meta/curate.md` | M | Decide store format (JSON files vs SQLite); whitelist enforcement |
| 2 | Skill library (markdown bundles) | Already maps to `~/.claude/skills/` and `~/.claude/commands/` | `~/.omega/skills/<bundle>/<skill>/SKILL.md` + `crates/omega-core/src/skills.rs` | S | Convention; mtime hot-reload |
| 3 | Skill bundles (YAML grouping) | New: per-project bundle for Causio, Kommu, etc. | `~/.omega/skill-bundles/*.yaml` + loader in `skills.rs` | S | None |
| 4 | Inline shell preprocessing (``!`cmd` ``) | Skill rendering before injection into worker prompt | extend `dispatch.rs` prompt builder | S | Safety — must run only in skill dir, with timeout |
| 5 | Trajectory ShareGPT JSONL | Capture every Oracle + Worker session for the eval / prompt-evolution corpus | `crates/omega-core/src/trajectory.rs` writing to `~/.omega/trajectories/*.jsonl` | M | Storage budget; secret redaction |
| 6 | Insights engine | Replace / extend existing `metrics.rs` with a `omega insights --days 30` CLI | `crates/omega-cli/src/insights.rs` | S | `metrics.db` schema must record tool_name + skill invocations (small migration) |
| 7 | Error classifier + failover taxonomy | Port the enum + pipeline to Rust; wrap every provider call | `crates/omega-core/src/failover.rs` (extend `providers.rs`) | M | Need provider-specific error mapping for Claude/Codex/Gemini/GLM |
| 8 | Context compressor | Long-running Oracle sessions hitting 150K+ tokens | `crates/omega-core/src/compactor.rs` invoked by `claude_stream.rs` | L | Needs a "summarizer" sub-call; anti-thrash counter; image stripping |
| 9 | Iteration budget (thread-safe) | Already partially present (`--max-turns`); add per-worker quotas tracked centrally | extend `dispatch.rs` + `gate.rs` | S | None |
| 10 | Anthropic prompt cache `system_and_3` | AISB Master + every Oracle session | extend `claude_stream.rs` request builder | S | Must verify Claude Code CLI exposes cache breakpoints; if not, only applies to direct API calls |
| 11 | Memory provider trait | New `trait MemoryProvider` for AISB; built-in JSONL provider + optional Convex provider | `crates/omega-core/src/memory.rs` | M | Decide whether Convex is in-scope (docs/plans/CONCEPT.md says optional) |
| 12 | Streaming context scrubber | Strip `<MEMORY_CONTEXT>...</MEMORY_CONTEXT>` from Telegram stream | extend `formatting.rs` | S | None |
| 13 | Conversation loop self-healing retries | Already in `claude_stream.rs`; cross-check missing retry classes (invalid JSON args, length continuation) | patch `claude_stream.rs` | S | None |
| 14 | Tool guardrails (whitelist per role) | Restrict review-agent + worker to a tool subset declared in `brief.json` | extend `dispatch.rs` brief schema | M | Requires per-session config plumbing |

---

## 3. Top 5 highest-value integrations

### #1 — Background review (skill + memory curator)
- **Why**: This is the actual self-improvement loop. Every time an Oracle finishes a mission, a forked review session asks "what did we learn?" and persists deltas as patched skills or memory snippets. Compounds quality over weeks. Nothing else in the repo gives this.
- **Where**: Hook into the existing `~/.omega/state/oracle-*.done.json` patrol. When `status=done_clean`, spawn a tiny ephemeral Claude session with whitelist `{skill_view, skill_manage, memory_add, memory_patch}` and the curate prompt (port verbatim from Hermes).
- **Effort**: M (~1 day for the spawn + whitelist, ~1 day for the curate prompt + store format, ~1 day to wire mtime hot-reload into worker prompt builder).
- **Risk**: A noisy curator can pollute the skill library with junk. Mitigate with: (a) require 2 separate triggers before creating a NEW skill (rule R-21 multi-grader consensus already exists), (b) a `~/.omega/skills/_quarantine/` dir for first 24h before promotion.
- **Open questions**: Does Omega curate **global** skills (`~/.claude/skills/`) or **project-scoped** ones (`~/VibeCoding/clients/<proj>/.omega/skills/`)? My recommendation: project-scoped by default, manual promote-to-global.

### #2 — Error classifier + failover taxonomy
- **Why**: Omega currently has scattered retry/fallback logic per provider. A central `FailoverReason` enum + classification pipeline is a clean win for reliability and observability. Cheapest port in the repo.
- **Where**: New `crates/omega-core/src/failover.rs`. Every provider in `providers.rs` returns `Result<T, ClassifiedError>` instead of opaque strings. `dispatch.rs` reads the hints (retryable, should_rotate_credential, should_fallback) to pick the next action.
- **Effort**: M (~2 days: enum + classifier + per-provider mapping).
- **Risk**: Low. Pure code refactor.
- **Open questions**: Should this also drive automatic provider switching (Claude → Codex fallback)? Currently dispatch is explicit.

### #3 — Context compressor for long Oracles
- **Why**: Real oracles hit 150K+ tokens. Right now we either restart them (losing context) or let them error out. Hermes's compressor with head/tail preservation + structured middle summary is the right shape, including the anti-thrashing guard (skip if last two passes saved <10%).
- **Where**: `crates/omega-core/src/compactor.rs` invoked by `claude_stream.rs` before each request. Summarizer = small Haiku call.
- **Effort**: L (~3-5 days: token counter + summary prompt + tool-output redactor + image stripping + thrash guard + tests against real Oracle transcripts).
- **Risk**: Summarization loses information. Mitigate by storing the full transcript verbatim in `~/.omega/trajectories/` so nothing is destroyed, only what's in-context shrinks.
- **Open questions**: Does Claude Code CLI expose a hook to inject compressed history mid-session, or do we have to restart the session with the compressed prompt? If only the latter, this is fundamentally a session-fork, not an in-place compression.

### #4 — Trajectory capture (ShareGPT JSONL)
- **Why**: It's the foundation for everything else: prompt evolution, regression detection, audit playback. Even if we never train, having every Oracle/Worker run as a replayable JSONL is gold.
- **Where**: `crates/omega-core/src/trajectory.rs`. Writes `~/.omega/trajectories/<project>/<oracle>/<session>.jsonl`. Append-only. Same redaction as Hermes (secret patterns).
- **Effort**: M (~2 days: writer + redactor + path layout + cleanup cron).
- **Risk**: Disk growth. Mitigate with a 30-day retention crontab and gzip after 7 days.
- **Open questions**: ShareGPT format vs OpenAI-messages format? ShareGPT is the de-facto training format; pick it unless we have a strong reason not to.

### #5 — Skill bundles (YAML grouping) + inline shell preprocessing
- **Why**: Today `~/.claude/skills/` is a flat list. Bundles let us say "Causio dev mode = load these 8 skills" or "Linear feedback mission = load these 5 skills". Inline shell (``!`gh issue view 42` ``) makes skills dynamic — they pull live data at injection time instead of being static templates.
- **Where**: `crates/omega-core/src/skills.rs` (new), loaded by `dispatch.rs` when building worker prompts.
- **Effort**: S (~1 day for the YAML loader + mtime hot-reload, ~1 day for the bash expander with `flock` + timeout).
- **Risk**: Inline shell is dangerous. Constrain: cwd = skill dir, timeout 10s, no network, deny `rm`/`curl http*` outside an allow-list. This is a hard requirement — never ship the expander without a sandbox.
- **Open questions**: Bun vs `std::process::Command`? Bun gives us a JS-level sandbox; Rust is faster but the sandbox is harder. Recommend `tokio::process::Command` + AppArmor profile.

---

## 4. Anti-patterns to avoid

1. **Don't vendor Python**. Hermes is ~50K lines of Python with deep coupling to `httpx`, `anyio`, Anthropic/Gemini/Bedrock SDKs, SQLite, Honcho, FTS5, agentskills.io. Porting line-by-line breaks rule RUST-BUN-DEFAULT and introduces a Python runtime we explicitly do not want. Port **mechanisms and prompts**, not source.
2. **Don't import the `/skills/` directory wholesale**. Bundles like `apple`, `gaming`, `gifs`, `smart-home`, `yuanbao` are irrelevant to OmegaOS. Cherry-pick `software-development`, `devops`, `github`, `research`, `red-teaming/godmode`, `mlops` only — and even then, audit each `SKILL.md` for Hermes-specific assumptions.
3. **Don't add embedding-based skill retrieval just because it sounds smart**. Hermes doesn't have it; name-based slash-command resolution works fine when bundles are small. Omega's `~/.claude/commands/` already proves this scale.
4. **Don't add the Honcho user-modeling dialectic framework**. It's a Python service + LLM-driven persona model. For OmegaOS the equivalent is the existing `MEMORY.md` file at `~/VibeCoding/agentic/agentik-monitor/bot/MEMORY.md` — much simpler, already loaded every session.
5. **Don't auto-create new skills aggressively**. The Hermes prompt order (patch > update > add file > create) is correct, but in a Rust agent OS with audit trails we should be even more conservative: log proposed new skills to `~/.omega/skills/_proposals/` and require a human nod (or 2-grader consensus per rule R-21) before promoting.
6. **Don't use the conversation_loop self-healing retries verbatim**. Omega already routes through `claude_stream.rs` + `dispatch.rs` + `gate.rs`. Adding a parallel loop creates two retry surfaces. Cross-check what's missing (invalid-JSON repair, length continuation) and patch existing code instead.
7. **Don't fork the agent for review on EVERY turn**. Hermes does this per-turn; for an oracle that may be 1000+ turns this would burn tokens. In Omega, trigger review only on `done.json` events and on explicit user "lesson learned" signals via Telegram.
8. **No training loop**. Nous trains models. We don't. The trajectory JSONL is for replay, debugging, prompt evolution — not gradient updates. If we ever want training, it's a separate decision with its own infra.

---

## 5. Open questions for the human

1. **Scope of self-improvement**: global skills (`~/.claude/skills/`) or per-project (`~/VibeCoding/clients/<proj>/.omega/skills/`)? My recommendation: per-project default + manual promote.
2. **Curator model**: should the background-review forked agent use the SAME model as the parent (Opus, expensive) or a cheaper one (Haiku/Sonnet)? Hermes uses the same to hit the prefix cache. For Omega I'd start with Sonnet.
3. **Training**: do we ever want a real fine-tuning loop, or is this strictly prompt-evolution / skill curation? Affects whether trajectory format must be ShareGPT-strict.
4. **Memory backend**: JSON files (simple, git-friendly) or SQLite (queryable, FTS5)? Hermes uses SQLite. Rust has `rusqlite` already. Probably SQLite + a `~/.omega/memory.db`.
5. **Sandbox for inline shell**: AppArmor / firejail / bubblewrap / nsjail? Or limit to a fixed allow-list of binaries? This blocks the skill-bundle quick win.
6. **Convex dependency**: docs/plans/CONCEPT.md says optional. If we want the memory provider plugin pattern, do we ship a Convex provider in core or as an external plugin?
7. **Compressor placement**: in-process via Claude Code hooks, or as a fork-restart with a compressed prompt? Determines feasibility of integration #3.
8. **Skill quarantine**: do auto-created skills go live immediately (Hermes default) or to `_quarantine/` for 24h pending review (safer)?
9. **Telegram surfacing**: do we send a `💾 Self-improvement review` line to Telegram after every Oracle done, or only on significant updates? Noise budget matters.
10. **Where does this live in the repo?**: new crate (`crates/omega-self/`), or extend `omega-core`? My recommendation: new crate, keeps `omega-core` lean.

---

## 6. Quick wins (S effort, <1 day each)

| # | Win | What | File |
|---|---|---|---|
| QW1 | Anthropic prompt cache breakpoints | Implement `system_and_3` layout for AISB Master + Oracle sessions. ~75% cost reduction. | extend `claude_stream.rs` request builder |
| QW2 | Iteration budget centralization | Single `IterationBudget` struct, parent=90/worker=50, refund for programmatic tool calls. | new `crates/omega-core/src/budget.rs` |
| QW3 | Streaming context scrubber | Strip `<MEMORY_CONTEXT>` / `<REASONING_SCRATCHPAD>` fences from Telegram output. | extend `formatting.rs` |
| QW4 | Insights CLI | `omega insights --days 30 [--project Causio]` reading `~/.omega/state/metrics.db`. Tables: tokens, cost, tool usage, skill usage, activity. | new `crates/omega-cli/src/insights.rs` |
| QW5 | Skill bundles (YAML) | `~/.omega/skill-bundles/*.yaml` with name + skills[] + extra_instructions. mtime hot-reload. NO inline shell yet (that needs the sandbox). | new `crates/omega-core/src/skills.rs` |
| QW6 | Error classifier enum stub | Port `FailoverReason` enum + a SKELETON classifier with HTTP-status + message-pattern lanes. Per-provider patterns come later. | new `crates/omega-core/src/failover.rs` |
| QW7 | Trajectory writer (append-only JSONL) | Capture every dispatched session as ShareGPT JSONL. No redaction yet (mark TODO). 30-day retention cron. | new `crates/omega-core/src/trajectory.rs` |
| QW8 | Curate-prompt skill | Port the Hermes background-review prompt verbatim into `~/.omega/skills/_meta/curate.md`. Even before the auto-trigger, an oracle can invoke `/curate` manually after a mission. | content-only, no code |

QW1–QW3 + QW8 are all under 2 hours each. Doing all eight in a focused day gets us 80% of the self-improvement substrate without touching the dangerous bits (inline shell, auto-skill-creation, in-place compaction).

---

## Appendix — citations

| Claim | Hermes file |
|---|---|
| Background review whitelist + forked agent + prefix cache reuse | `agent/background_review.py` (`_run_review_in_thread`, `spawn_background_review_thread`, `summarize_background_review_actions`) |
| Update preference order (patch > update > add > create) | `agent/background_review.py` docstring/prompts |
| Skill bundles YAML at `~/.hermes/skill-bundles/` | `agent/skill_bundles.py` (`get_skill_bundles`, `resolve_bundle_command_key`, `build_bundle_invocation_message`) |
| Inline shell ``!`cmd` `` and `${HERMES_SKILL_DIR}` substitution | `agent/skill_preprocessing.py` (`substitute_template_vars`, `expand_inline_shell`, `preprocess_skill_content`) |
| Skill loading + scan + slash-command resolution | `agent/skill_commands.py` (`_load_skill_payload`, `scan_skill_commands`, `get_skill_commands`, `reload_skills`, `build_skill_invocation_message`, `resolve_skill_command_key`) |
| Trajectory JSONL ShareGPT, scratchpad→think conversion | `agent/trajectory.py` (`save_trajectory`, `convert_scratchpad_to_think`, `has_incomplete_scratchpad`) |
| Insights SQLite analytics, `generate(days=30, source=None)` | `agent/insights.py` (all `_get_*`, `_compute_*`, `format_terminal`, `format_gateway`) |
| Error taxonomy `FailoverReason` 20+ codes + priority pipeline | `agent/error_classifier.py` (`classify_api_error`, `ClassifiedError`) |
| Compressor head/tail preservation + anti-thrash + tool-output summary | `agent/context_compressor.py` (`should_compress`, `_prune_old_tool_results`, `_generate_summary`, `_serialize_for_summary`) |
| Iteration budget parent=90/sub=50 + refund | `agent/iteration_budget.py` (`consume`, `refund`, `used`, `remaining`) |
| `system_and_3` 4-breakpoint prompt cache | `agent/prompt_caching.py` (`apply_anthropic_cache_control`, `_apply_cache_marker`, `_build_marker`) |
| Memory provider trait (one external max) | `agent/memory_manager.py` (`MemoryManager`, `StreamingContextScrubber`, `MemoryProvider`) |
| Conversation loop retries (empty content / invalid JSON / scratchpad / length / compression preflight) | `agent/conversation_loop.py` (`run_conversation`) |

---

*Bottom line: Hermes's "self-improvement" is a memory-and-skills curator running in a forked LLM call after each turn, plus a clean failure taxonomy, plus disciplined context compression. It's mechanism we can port, not code. Start with QW1-QW8 (1 day), then build integration #1 (background review) on top. Skip the training loop, skip embeddings, skip Honcho, skip the irrelevant skill bundles.*
