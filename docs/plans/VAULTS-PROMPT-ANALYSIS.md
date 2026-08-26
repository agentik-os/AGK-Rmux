> **Historical (2026-05-28) — kept for context.** Working analysis behind `crates/omega-core/src/amplify.rs`. Not maintained as reference.

# Vaults AISB → OmegaOS: Prompt-Amplification Analysis & Reapplication Plan

**Date:** 2026-05-28
**Scope:** Reverse-engineer how the live VPS "Vaults" Telegram bot (Python, `~/VibeCoding/agentic/agentik-monitor/bot/aisb/`) amplifies short user messages into structured Oracle/Worker briefs, injects rules/audits dynamically, and dispatches — then map the gap vs the new Rust OmegaOS and give a concrete port plan.

> **One-line verdict:** OmegaOS already *has* the amplification machinery (`PromptEnhancer`, `SmartRouter`, `classify_mission`, `detect_audit_skills`). The live AISB-Master conversation path **never calls it** — it pipes raw user text straight to its own Claude brain (`claude_stream.ask`). Vaults instead ran a deterministic `enhance_prompt()` Brain pass *before* dispatch. That single missing wire is the whole "copy-paste instead of structured brief" problem.

---

## 1. How Vaults builds prompts (the mechanism)

Vaults uses a **two-LLM-pass + 3-layer prompt** architecture.

### Pass 1 — the "Brain" amplifier (deterministic, runs BEFORE dispatch)
`enhance_prompt(user_prompt, project_name, project_path)` — `intelligence.py:124`.

1. **Skip gates** so cheap prompts don't pay the Brain cost:
   - ultra-trivial (`<8` chars, no space) → return raw (`intelligence.py:132-139`)
   - already-structured + long (`>=500` chars AND has `##`/`1.`/`- `/```` ```) → return raw (`intelligence.py:146-158`)
   - LRU cache `(project, hash)` → 64 entries (`intelligence.py:27, 161-169, 304-310`)
2. **Gather real project context** — `_gather_project_context()` (`intelligence.py:37`): recent git log (`git log --oneline -5 --since=24h`), branch + `git status --short`, last `.oracles/oracle-*.md` report, active tmux sessions for the project. Each section is independently fault-tolerant (`intelligence.py:52-121`).
3. **Spawn a separate one-shot Claude** (`claude --print --max-turns 1 --model claude-opus-4-7 --allowedTools ""`, `ANTHROPIC_API_KEY` stripped, 60s timeout, 2 attempts) with a strict instruction template (`intelligence.py:184-218, 224-256`). The Brain is told to output EXACTLY: `## Mission / ## Context / ## Tasks / ## Success Criteria / ## Constraints`, ≤350 words, **no tools allowed**, "STRUCTURE the message, don't REWRITE it", "NEVER invent terminology", "Linear /project/ URL = spec not ticket list", "Google Drive link = read content".
4. **Assemble final brief**: Brain output (or a deterministic fallback skeleton, `intelligence.py:262-268`) + appended `## Project State` (git/sessions) + `## Planner` status from `.planner/tracker.json` (`intelligence.py:270-286`).
5. Returns an `EnhancedPrompt(str)` subclass carrying `.original_human` and `.was_enhanced` so the dispatch layer can show the operator both the raw and enriched versions (`intelligence.py:295-299`).

### Pass 2 — wrap the amplified brief in the dispatch template (Layer 3)
`_build_oracle_dispatch_prompt()` — `prompts.py:365`. Critically, this is documented (`prompts.py:368-398`) as **Layer 3 of 3**:
- **Layer 1** = `CLAUDE.md` + `~/.claude/rules/` (auto-loaded by Claude Code — Three Laws, Karpathy, audit-keyword tables). Never repeated.
- **Layer 2** = oracle system prompt `~/.aisb/prompts/<Project>-oracle.md` via `--append-system-prompt-file` (identity, R-0..R-16, dispatch commands, worker template, Quality Arsenal). Loaded once at boot.
- **Layer 3 (this f-string)** = ONLY the mission: `> **the operator (original):** …` + enhanced brief + optional Linear protocol + a 1-line reminder. The docstring explicitly warns: "Repeating [rules] here wastes ~3000 tokens and creates confusion when versions drift between layers" (`prompts.py:393-397`).

`build_oracle_dispatch_prompt_async()` (`prompts.py:456`) optionally probes a `/dispatch-oracle` skill for an even richer body, with a hard fallback to the legacy f-string if it returns `<80` chars or errors (`prompts.py:478-551`).

### The AISB-Master *conversation* system prompt
`_build_system_prompt()` — `prompts.py:126`. This is the bot's OWN brain (handling DM/topic chat before it dispatches). It is large and static: Two Laws, "TU NE TRAVAILLES JAMAIS SUR LES PROJETS / tu DISPATCHES à l'oracle" (`prompts.py:164-168`), SOUL.md persona (`_get_soul_content`, mtime-cached, `prompts.py:94`), conversation-memory rules, anti-stage-direction output rules (`prompts.py:208-219`). Topic sessions get a tiny extra `PROJECT CONTEXT` block (`prompts.py:133-144`).

### Dispatch wiring (where Pass 1 → Pass 2 connect)
`process_prompt.py:133-137` — the only place that matters:
```python
enriched = await enhance_prompt(user_prompt, pname, project_path)          # Pass 1
user_prompt = await build_oracle_dispatch_prompt_async(pname, project_path, # Pass 2
                                                       work_sess_name, enriched)
user_prompt = f"{sessions_ctx}\n\n{user_prompt}"                            # + live sessions
```
then handed to `dispatch-to-session.sh` (`process_prompt.py:153-156`). **Oracle vs Worker differ here:** the Oracle gets the amplified-mission brief (Pass 1+2); the Worker gets a Fresh-Context template (Mission/Purpose/Context/What's Done/Current Task/Done Criteria/Verify Command/Files in Scope), the oracle builds that per sub-task and ships it via `worker.md` protocol (`~/.aisb/prompts/worker.md:1-77`).

---

## 2. Dynamic rule + audit injection logic

**Rule injection in Vaults is mostly *static* (Layer 1/2 auto-load), with ONE dynamic injection:** the Linear protocol.
- `_is_linear_feedback_task(user_task)` (`prompts.py:224`) — exact-phrase signals + `"linear"`+action-word co-occurrence (handles typos like "feedbaclk"). If true, the full 9-step `LINEAR_FEEDBACK_PROTOCOL` (`prompts.py:264-362`) is appended to the dispatch prompt (`prompts.py:403-405`).
- Everything else (Three Laws, Karpathy, audit-keyword tables) is **not** dynamically selected — it's permanently present in `CLAUDE.md`/rules and the oracle system prompt. The oracle then matches keywords itself at runtime.

**Audit selection is genuinely dynamic but lives at the *oracle/verification* layer, not the dispatch layer:** `~/.aisb/lib/audit-selector.py`. It picks audits from three signals (`audit-selector.py:5-16`):
- a `BASELINE_ALWAYS = [codeaudit, logicaudit, debugaudit]` floor (`:58`) with `MIN_AUDITS=5` padding (`:64`) and a special 8-audit `AUDIT_INREVIEW_BASELINE` mode (`:69`);
- **mission-text keyword regexes** — `KEYWORD_TRIGGERS` (`:87-`): e.g. `\bauth\b|\blogin\b|\bjwt\b|\bclerk\b → secaudit`, `\bbutton\b|\bui\b|\bmodal\b → uiuxaudit`, `\bslow\b|\blcp\b|\bbundle\b → perfaudit`;
- **files-modified patterns** + **project-type defaults** (webapp/backend/cli/saas).
Each selected audit is then dispatched as **one worker per audit, in parallel** (the oracle's job, gated by file-disjointness), never combined.

---

## 3. Oracle → Worker decomposition & dispatch (Vaults)

- The Oracle reads its Layer-2 system prompt (the per-project `*-oracle.md`, ~26KB) which contains the dispatch commands, the worker template, and the Quality Arsenal.
- For each sub-task it writes a **Fresh Context** worker prompt (Mission / Purpose / Context / What's Done / Current Task with file:line / Done Criteria / Verify Command / Files in Scope / Key Decisions) and dispatches via `dispatch-to-session.sh <Project>-<task>`.
- Parallelism: file-disjoint "narrow" tickets batch up to N (3-4); "broad"/"terminal" tickets run alone (`linear-mission.sh next-batch`). Code fixes on shared files are serialized (`prompts.py:293-299`).
- Worker closes with `worker.md` protocol: TodoWrite mirror → `omega-todo.sh declare` → execute → write 7-section `report.md` → `omega-todo.sh finish` writes `done.json` and self-kills (`worker.md:33-57`).

---

## 4. Gap analysis — OmegaOS (Rust) vs Vaults

| Capability | Vaults (Python) | OmegaOS (Rust) today | Gap |
|---|---|---|---|
| **Amplify raw msg → structured brief** | `enhance_prompt()` runs a dedicated Brain LLM pass **before** dispatch (`intelligence.py:124`) | `PromptEnhancer::enhance` exists (`router.rs:380-454`) but is **template-only (no LLM pass)** AND **never called on the live Master path** | **THE core gap.** See below. |
| **Master path wiring** | DM/topic → `enhance_prompt` → `build_oracle_dispatch_prompt_async` → dispatch (`process_prompt.py:133-156`) | Master path pipes raw `final_prompt` (reply-ctx + raw text) straight to `claude_stream.ask()` (`telegram_bridge.rs:635-656`). `SmartRouter`/`PromptEnhancer` are dead code outside `router.rs` tests. | Master's Claude brain gets RAW text + a static `aisb-master.md` system prompt; amplification is left to LLM whim → "copy-paste" behavior |
| **Project context injection** | git log/branch/status, last oracle report, planner status, active sessions (`intelligence.py:37`) | `WorkerContext::with_git_context` (branch + last 5 commits, `dispatch.rs:110`). No planner/last-report/sessions. Master injects none. | Medium — Master & oracle brief lack live project state |
| **Oracle brief structure** | Layer-3 mission-only, rules in Layer 1/2 (`prompts.py:365`) | `OraclePromptGenerator::generate` builds a full structured oracle system prompt incl. Three Laws, 5-step, dispatch rules, ship/godmode, quality gate (`oracle_lifecycle.rs:296-389`) — **good, arguably already better-structured than Vaults Layer 3** | Small — solid; just isn't fed an *amplified* mission |
| **Dynamic audit selection** | `audit-selector.py` (keyword + files + project-type + baseline floor + padding) at verify layer | `detect_audit_skills` (mission-keyword table only, `routing.rs:60`) → appended to oracle prompt (`dispatch.rs:189-197`). No files/project-type/baseline-floor signals. | Medium — keyword-only; no baseline floor, no file-pattern triggers |
| **Complexity classification** | implicit in oracle prompt | `classify_mission` scoring + effort/budget/turn scaling (`routing.rs:118`, `dispatch.rs:210-237`) — **better than Vaults** | None |
| **Dynamic rule injection (Linear etc.)** | `_is_linear_feedback_task` → append 9-step protocol (`prompts.py:404`) | Scoped rules block via `rules_prompt_block(scope)` (`rules.rs:307`) injected into Oracle+Worker briefs (`dispatch.rs:335,410`). **No Master injection, no Linear/task-specific protocol injection.** | Medium — no per-mission protocol packs; Master gets no rules block |
| **Brief preamble (safety surface)** | n/a | `brief_preamble()` prepended to Oracle+Worker (`dispatch.rs:329,404`) — **better than Vaults** | None |
| **Skip gates / caching** | trivial-skip + structured-skip + LRU cache (`intelligence.py:132-169`) | none (no amplifier to gate) | Comes free once amplifier is added |

### The crux (cite)
`telegram_bridge.rs:635-656`:
```rust
let final_prompt: String = match &reply_context {
    Some(ctx) => format!("{}{}", ctx, text),   // <- reply ctx + RAW user text
    None => text.to_string(),
};
... self.claude_stream.ask(&final_prompt).await   // <- straight to the brain, unamplified
```
There is no `classify_mission`, no `enhance`, no project-state gather, no rules block on this path. Compare Vaults `process_prompt.py:133-137` which always amplifies first. **OmegaOS built the parts and forgot the assembly line on the hottest path.**

---

## 5. Concrete reapplication plan for OmegaOS (Rust + Bun only)

Priority order. Effort: S ≤ ~50 LOC, M ≈ 50-200 LOC, L > 200 LOC.

| # | Port | Where (file) | Effort | Notes |
|---|---|---|---|---|
| **1** | **Wire amplification into the Master dispatch decision.** When the Master's brain decides to dispatch (it runs `omega dispatch <Project> "<mission>"`), the mission string passed to `dispatch_oracle` should already be amplified. Cheapest correct fix: amplify **inside `Dispatcher::dispatch_oracle`** before `OraclePromptGenerator::generate`, so EVERY dispatch (Master, CLI, topic) benefits. | `crates/omega-core/src/dispatch.rs` (around `:152-186`) | **M** | Single choke-point. Keeps Master prompt dumb; the pipeline does the work. |
| **2** | **Add a real Brain pass = `amplify_mission()`** mirroring `enhance_prompt`: spawn `claude --print --max-turns 1 --allowedTools "" --model <opus>` with the strict `## Mission/Context/Tasks/Success Criteria/Constraints` template + "structure don't rewrite / don't invent terminology / Linear-url=spec". | new `crates/omega-core/src/amplify.rs` | **M** | Reuse `agents::Agent` launch plumbing. Output is plain text injected as the `mission` body. Bun alternative only if a TS LLM client is preferred — Rust `std::process::Command` is simpler here. |
| **3** | **Skip gates + LRU cache** on `amplify_mission` (trivial `<8` no-space; structured+`>=500`; `(project,hash)` cache, cap 64). | `amplify.rs` | **S** | Direct port of `intelligence.py:132-169`. Avoids paying Brain cost on `/status`, long pre-written briefs. |
| **4** | **Enrich `WorkerContext`/oracle brief with live project state**: planner `.planner/tracker.json`, last `.oracles/oracle-*.md`, active sessions — extend `with_git_context`. | `dispatch.rs:110` + helper | **M** | Port `_gather_project_context` (`intelligence.py:37`). Each section fault-tolerant. |
| **5** | **Upgrade `detect_audit_skills`** to add a baseline floor (`[codeaudit, logicaudit, debugaudit]`, MIN=5 padding) + file-pattern triggers (`schema.ts→dataaudit`, route handlers→apiaudit) + project-type defaults. | `routing.rs:60-116` | **M** | Port the *logic* of `audit-selector.py:54-120`, not the Python. Keep it keyword/file-driven, no LLM. |
| **6** | **Dynamic protocol packs** (Linear etc.): a `mission_protocol_block(mission) -> Option<String>` that detects Linear-feedback intent and appends the multi-step protocol to the oracle brief. | new `crates/omega-core/src/protocols.rs` + call in `dispatch.rs` | **S/M** | Mirror `_is_linear_feedback_task` + `LINEAR_FEEDBACK_PROTOCOL`. Generalize to a small map of `intent → protocol.md`. |
| **7** | **Inject Master-scoped rules block** into the Master system prompt assembly (currently only `aisb-master.md` static). | `crates/omega-core/src/aisb.rs:34-70` | **S** | Call `rules::rules_prompt_block(RuleScope::Master)` + optional `brief_preamble()` when writing `aisb-master.system.md`. The scoping already exists (`rules.rs:252-254`); it's just not consumed. |
| **8** | **Delete or wire `SmartRouter`/`PromptEnhancer`.** They're unused on the live path. Either route the Master through them or remove to kill confusion (FILE-SIZE / dead-code hygiene). Recommend: fold the useful bits (audit-skill listing, success-criteria) into #2/#5 and delete the template-only `PromptEnhancer::enhance`. | `router.rs:376-454` | **S** | Avoids two competing "enhancer" concepts. |

**Minimum viable fix (ship first):** #1 + #2 + #3. That alone replaces the copy-paste with a real amplified brief on every dispatch, with skip gates so it stays cheap. #5/#6/#7 are quality multipliers; #4/#8 are hygiene.

---

## 6. What to AVOID (Vaults was "brouillon" — do NOT reproduce)

1. **Do NOT over-fetch rules into the brief.** Vaults' own docstring (`prompts.py:393-397`) admits repeating rules across layers "wastes ~3000 tokens and creates confusion when versions drift." OmegaOS's typed `rules_for_scope` is *better* — keep rules in the scoped block, never inline them into the amplified mission body. The amplifier output must stay **mission-only** (`## Mission/Context/Tasks/Success Criteria/Constraints`).
2. **Do NOT let the giant static persona prompt creep.** Vaults `_build_system_prompt` (`prompts.py:148-220`) is a 70-line wall mixing persona, laws, ecosystem cat-commands, safety, and anti-stage-direction patches accreted over months. OmegaOS's `aisb-master.md` is clean — resist bolting fixes onto it; put behavior in typed code (rules.rs) and keep the prompt declarative.
3. **Do NOT scatter amplification across handlers.** Vaults amplifies inside `process_prompt._send_and_poll_oracle` (`process_prompt.py:133`), but also has parallel godmode-Brain calls in `intelligence.py:683-740` with their own fallbacks and a *third* skill-probe path (`prompts.py:478`). Three places that build prompts = drift. **OmegaOS: one choke-point (`dispatch_oracle`).**
4. **Do NOT make the Brain use tools.** Vaults correctly forces `--allowedTools ""` + strips `ANTHROPIC_API_KEY` (`intelligence.py:208, 221, 227`). Keep that — the amplifier must NOT read files or fetch URLs; it only structures the text. A tool-enabled amplifier becomes a slow, nondeterministic mini-agent.
5. **Do NOT hand-roll tmux-pane scraping for results** (Vaults `process_prompt.py:201-254`: 1200×1s polling, idle-count heuristics, marker parsing). OmegaOS already has `done.json` signals + `patrol` — keep the structured signal path; don't regress to screen-scraping.
6. **Do NOT keep two enhancer abstractions.** `PromptEnhancer` (template) vs the new LLM `amplify_mission` will confuse future agents. Pick one (the LLM one), delete the other.

---

### Appendix — primary citations
- Amplifier: `aisb/intelligence.py:124` (`enhance_prompt`), `:37` (`_gather_project_context`), `:132-169` (skip/cache), `:184-218` (Brain template).
- Dispatch wrap: `aisb/prompts.py:365` (`_build_oracle_dispatch_prompt`, 3-layer doc `:368-398`), `:456` (async + skill probe), `:224` (`_is_linear_feedback_task`), `:264` (Linear protocol).
- Master persona: `aisb/prompts.py:126` (`_build_system_prompt`).
- Wiring: `aisb/process_prompt.py:133-156`.
- Audit selector: `~/.aisb/lib/audit-selector.py:54-120`.
- Worker protocol: `~/.aisb/prompts/worker.md:1-77`.
- OmegaOS Master raw path: `crates/omega-cli/src/telegram_bridge.rs:635-656`.
- OmegaOS dispatch: `crates/omega-core/src/dispatch.rs:152-280` (oracle), `:319-441` (worker).
- OmegaOS oracle prompt: `crates/omega-core/src/oracle_lifecycle.rs:296-410`.
- OmegaOS dead enhancer: `crates/omega-core/src/router.rs:376-454`.
- OmegaOS routing/audit: `crates/omega-core/src/routing.rs:60-189`.
- OmegaOS rules: `crates/omega-core/src/rules.rs:247-323`.
- OmegaOS master prompt source: `agents/aisb-master.md` (loaded `crates/omega-core/src/aisb.rs:11,34-70`).
