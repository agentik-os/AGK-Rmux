# The OmegaOS Guide

> The operator manual — for a human receiving OmegaOS for the first time AND for
> a fresh LLM agent operating it. Everything here is checked against the live
> source-built `omega`; check the installed version with `omega -V`. When this
> document and your runtime disagree, trust
> `omega --help` and `omega doctor`. Setup steps live in
> [docs/GETTING-STARTED.md](docs/GETTING-STARTED.md) (`omega guide`).

## 1. What OmegaOS is

OmegaOS is a provider-neutral control plane for a fleet of coding agents on one box. New sessions use OpenAI Codex by default, while Claude Code, Google Antigravity, enterprise/API-key Gemini CLI, OpenRouter/Pi, Hermes, GLM, and Kimi remain selectable. It
turns a Linux machine (typically a VPS) into a place where you dispatch work in
one sentence and a hierarchy of agents: the Atlas service routes, an oracle per
project that plans, ephemeral workers that edit — executes it in parallel under
a single typed rulebook, verifies it adversarially, and reports back to your
terminal or your phone. You drive it from three cockpits: a TUI, the `omega`
CLI, and a Telegram hub.

## 2. The vocabulary (read this first)

Every term below is load-bearing. Missions fail when these get conflated.

| Term | Definition |
|---|---|
| **Session** | An rmux PTY with a role: **Oracle** (per-project orchestrator), **Worker** (ephemeral editor), **Home** (your own interactive shells, e.g. `claude-1`), **System** (daemons and viewers). The legacy `aisb-master` name is a read-only Telegram conversation viewer, not a routing agent. `omega list` shows sessions. |
| **Mission** | A request dispatched to an oracle (`omega dispatch <Project> "<mission>"` or a message in the project's Telegram topic). The SQLite mission ledger is authoritative; JSON, timeline, plan, and Telegram surfaces are projections. |
| **Oracle** | One per project. It classifies, plans, dispatches workers, runs the quality gate, and reports. It **never edits project code itself** — the grader and the writer are different agents. |
| **Worker** | Ephemeral, parallel, file-scope-locked editor. Named `<Project>-worker-<task>`. It does ONE task, verifies against runtime, and signals completion with `omega done`. |
| **Workflow** | In-process fan-out *inside* one agent: spawn parallel sub-agents, adversarially verify their outputs, synthesize one answer. Cheaper than dispatching a worker per subtask; the default for review/research/audit/design work. |
| **Goal** | ONE shell-verifiable condition looped until true (the `/omg-goal` skill inside a Claude session). A thermostat, not a campaign — never wrap a multi-step mission in one goal. |
| **Plan** | A typed DAG of single-worker steps in `.planner/tracker.json`, written by `/omg-planner` and executed by `omega plan-run` with **Gate** (structural can't-skip) + **Guardian** (independent verify command) enforcement. |
| **Atlas** | The persistent Telegram orchestration service. Atlas receives messages, resolves their project/topic context, and dispatches work; the legacy `aisb-master` session is only a read-only conversation viewer. |
| **Skill** | A shipped `/omg-*` protocol (e.g. `/omg-planner`, `/omg-llm-council`). Installed under `~/.omega/skills/`; invoked by name inside a Claude session. |
| **Audit** | A forensic quality skill from the 23-audit Quality Arsenal (`omega audit list`). Gestalt clarity gate + Popper falsification + 10x scrutiny on the hinge point. |
| **done.json** | A compatibility projection of a candidate completion claim. `omega done` cannot self-accept work; the verifier and mission gate must record an accepted task attempt before delivery. |

## 3. The doctrine in 1 page

Seven Laws bind every agent, at every level, always. From the registry
(`omega rules list` — source: `crates/omega-core/src/rules.rs`):

- **L0 — Ship the truth, reproducible & pushed.** A change isn't done until it
  survives a clean rebuild and is pushed. For OmegaOS itself: a fresh
  `git clone && ./install.sh` must reproduce it. Secrets live outside the repo,
  always.
- **L1 — Runtime is the only truth.** Code and comments state intent; only
  running the program reveals reality. When code and runtime disagree, runtime
  wins.
- **L2 — Researcher, not sycophant.** Challenge a flawed premise with reasoning
  before acting — never agree-and-code. "This should work" without evidence is
  a lie.
- **L3 — Decide and proceed.** A dispatched agent is autonomous: it never asks
  "should I continue?". Detect the flaw, state the corrected premise, pick the
  best path, execute, report after.
- **L4 — Done means 100%, verified.** Enumerate every task in the prompt,
  finish each, self-verify each against runtime. 92% is not done.
- **L5 — Quality over speed.** Tokens are unlimited; quality is the only
  constraint. Never a "streamlined/quick" variant of a real protocol. A
  403/401/blocked surface is an ABORT, never a PASS.
- **L6 — Finish the mission.** Enumerate every deliverable, track it, execute
  through the last item, verify each result, and only then report.

The operational Rules reported by `omega rules list` implement the Laws, one
category at a time. One example each:

| Category | Example rule |
|---|---|
| Universal | **R-KARPATHY** — think before coding, simplicity first, surgical changes, goal-driven execution. |
| QualityGate | **R-VERIFY** — a delegate's own "done" is an input, never the verdict; verify adversarially, ≥2-of-3 consensus. |
| Orchestration | **R-ORCH** — workflow-first: fan out file-disjoint work in parallel, verify adversarially, synthesize yourself. |
| Reporting | **R-CITE** — evidence or it didn't happen: every claim carries a file:line, log line, or screenshot. |
| Safety | **R-SCOPE** — one writer per file: declare each worker's file scope; overlap → serialize or worktree-isolate. |

Each Rule is scoped by role, domain, risk, enforcement mode, and provider
capability. The compiler emits a compact context below 24 KB and a deterministic
digest. Nobody can spawn a child that quietly drops a Law, and provider-specific
mechanics no longer pollute the universal kernel.

## 4. Driving it — the three cockpits

### 4a. The TUI (`omega`)

Seven tabs (cycle with arrow keys; `Tab` toggles focus between panels):

| Tab | What it does |
|---|---|
| **Sessions** | Live session list with roles and progress; the right panel mirrors the selected pane and accepts chat input. Codex/OpenAI panes preserve ANSI color, reflow long Unicode input to the visible width, paint the real cursor row, and show the persisted provider identity. Kill, lock, rename, attach. |
| **Projects** | Registered projects, planner creation, Telegram topic controls, open/dispatch, and guarded deletion. |
| **OS** | The 24-product operative-system registry with static readiness evidence and MASTER prompt launch. |
| **Menu** | Launch actions: new Claude/Codex/Gemini/Antigravity/Pi/Hermes/GLM/Kimi/terminal session, **[N] New Project**, dispatch to an oracle, refresh, protection toggle, kill / kill-all / nuclear cleanup, restart, quit. |
| **System** | Laws, rules, agent roles, skills, and documentation. |
| **Help** | Keybindings and usage hints. |
| **Settings** | Theme gallery (live preview — see [docs/THEMES.md](docs/THEMES.md)), provider/model config, API keys, agent installs, the Monitor group (billing, accounts, bot status, provisioning keys wizard). |

**Scrolling & mouse — know your transport.** Over **plain SSH** the mouse works
end-to-end: wheel scrolls the session mirror (and the pane scrollback in a
direct `rmux attach`), click focuses, and drag-selecting text in the mirror
copies it to your local clipboard via OSC 52. Over **mosh** the mouse is dead
by design — mosh does not replicate the mouse-mode handshake to your terminal
([mosh#101](https://github.com/mobile-shell/mosh/issues/101)) — so keep a plain
SSH profile for mouse-heavy work and use the keyboard over mosh: `PgUp`/`PgDn`
(full-page mirror scroll, chat focus included), `Home`/`End` (top/tail),
`Alt+↑/↓` (line scroll — also works attached directly to a session). Claude
sessions scroll deep because OmegaOS runs them on the normal screen
(`CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1`) with a 500k-line rmux history.

### 4b. The CLI commands you'll use daily

```bash
omega doctor                                  # whole-stack health; --fix repairs mechanical issues
omega                                         # the TUI (alias: omega menu)
omega rules list                              # the Laws + Rules, role-scoped registry
omega dispatch MyApp "fix the signup flow"    # send a mission to MyApp's oracle
omega spawn-worker fix-auth "..." --files src/auth.rs --worktree   # worker, scope-locked + isolated
omega done <session> done_clean "what+proof"  # worker completion signal (done_clean|pending|failed|blocked)
omega progress <session> --plan "audit|fix N+1|merge"              # oracle progress → Telegram card
omega status <session>                        # session status + pane content
omega plan-create [dir]                       # open /omg-planner in that project
omega plan-run [dir]                          # execute .planner/tracker.json (Gate + Guardian)
omega patrol                                  # session-health watchdog (also runs from cron)
omega cleanup                                 # prune stray sessions, stale state, /tmp scratch
omega kill-all                                # kill everything except you + infrastructure
omega sync                                    # symlink OmegaOS config into every LLM config dir
OMEGA_TG_TOKEN=<TOKEN> omega telegram setup <ID> --user-id <ID>    # wire the Telegram bot
omega guide                                   # re-print the getting-started steps
```

(A goal loop is a Claude-session primitive, not a CLI command: type `/omg-goal
<one shell-verifiable condition>` inside a session.)

The full surface, one line per subcommand (from `omega --help`):

The table below is a curated operator reference, not an exhaustive command
inventory. Run `omega --help` for the current top-level surface and
`omega <command> --help` for authoritative arguments.

| Command | One line |
|---|---|
| `menu` | Launch the TUI session manager. |
| `guide` | Print the getting-started guide. |
| `new` | Create a new rmux session (`--agent claude\|codex\|…`). |
| `new-project` | Bootstrap a brand-new project (provision + scaffold + vision/PRD/plan); `--dry-run` prints the plan. |
| `agents` | List supported agent CLIs and their availability. |
| `clean-junk` | Remove rmux sessions omega could not have created (mangled-paste names); dry-run by default. |
| `clock` | Print the localized wall-clock for the rmux status bar. |
| `projects` | Auto-discover projects on this machine. |
| `install` | Run the official installer for an agent (pi, hermes, codex, gemini, glm, claude). |
| `aisb-view` | Open the read-only Telegram conversation viewer (`master` and `aisb` are compatibility aliases). |
| `config` | Get/set provider configuration values. |
| `monitor` | Show billing / accounts / bot status (one-shot). |
| `telegram` | Manage the Telegram bridge (setup/run/enable/disable). |
| `pdf` | Generate a branded PDF report (whitepaper/audit/marketing/doc). |
| `rules` | List, export, or manage operational rules. |
| `audit` | Manage the 23-audit Quality Arsenal (`list` / `select` / `results` / `run`). |
| `sync` | Symlink OmegaOS config into all LLM config directories. |
| `install-bindings` | Install Ctrl+Space plus prefix `o` / `z` rmux menu bindings. |
| `list` | List all sessions. |
| `attach` | Attach to a session. |
| `kill` | Kill a session. |
| `dispatch` | Dispatch a mission to an oracle. |
| `orchestrate` | Full mission end-to-end: classify → plan → dispatch → monitor → gate. |
| `spawn-worker` | Spawn a worker under the current oracle (`--files`, `--worktree`). |
| `team` | Spawn a team of agents in split panes. |
| `done` | Signal task completion (called by workers). |
| `progress` | Report live mission progress (renders the Telegram task checklist + bar). |
| `inbox` | Read/drain oracle inbox events (JSONL queue). |
| `ship` | Ship pipeline: build → commit → push → deploy → verify. |
| `patrol` | Run the session-health watchdog daemon. |
| `usage` | Token-budget monitor + Telegram 80%/90% alerts. |
| `kill-all` | Kill all sessions except yours + infrastructure. |
| `cleanup` | Nuclear cleanup: stray sessions, stale state, /tmp, page cache. |
| `doctor` | One-shot health check of the whole stack (`--fix`, `--pre-reset`). |
| `backup` | Back up irreproducible state (`~/.omega` + crontab) to one `.tgz`. |
| `timeline` | Replay an oracle's dispatch→done history. |
| `resurrect` | Re-spawn crashed oracles from persisted state (no arg = all dead ones). |
| `provision` | Manage provisioning credential groups (per-client accounts). |
| `aisb-chat` | Interactive local chat REPL routed through the Telegram orchestration service. |
| `gate` | Check the quality gate for an oracle. |
| `scope` | Check scope-claim conflicts. |
| `status` | Show session status and pane content. |
| `send` | Send text to a session. |
| `capture` | Capture pane content from a session. |
| `log` | Show session log (JSONL history). |
| `rpc` | JSONL stdin/stdout mode for external orchestration. |
| `route` | Classify a mission's complexity (SIMPLE/MEDIUM/COMPLEX/EPIC). |
| `completions` | Generate shell completions. |
| `init` | Initialize OmegaOS configuration. |
| `plan-create` | Open `/omg-planner` in a project-scoped agent session. |
| `plan-status` | Show plan progress from `.planner/tracker.json` (read-only). |
| `plan-run` | Drive a plan to completion (spawns real workers per step). |
| `claude-login` / `claude-login-code` | Headless Claude OAuth re-login (start / finish). |

### 4c. The Telegram hub

One command bot (systemd service `omega-tg-bot`) plus a topics group:

- **Atlas topic** — discussion with the orchestrator brain. Ask questions,
  give direction; Atlas classifies and dispatches. Reports do not land here.
- **Project topics** — one per project (`/setupgroup` then `/sync`). Message a
  topic = dispatch a mission to that project's oracle. While it runs you get a
  **live progress card** (task checklist + bar, edited in place), and at the
  end a **mission PDF report** with a "steps to verify" section.
- **Alerts topic** — operational alerts only (stuck oracle, token budget,
  catastrophic failures). Auto-recreated if deleted.
- **Deposit / inbox bot** — a private second bot (`inbox-bot-up <TOKEN>`):
  anything you send from your phone (photos, files, notes) lands in
  `~/.omega/inbox/`, timestamped and indexed, where any agent can read it.
- **Per-project agent bots** — optionally give a project its own bot: register
  the token via the command bot's project menu; it's stored in
  `~/.omega/agent-bots.json` and runs as `omega-tg-agent-<project>.service`.
- **Companion bots** — an instant lightweight co-worker (Haiku) on its own bot
  for chat-speed assistance, separate from the heavy mission pipeline.

The command bot's slash surface: `/menu /account /model /projects /sessions
/audits /skills /status /dispatch /login /setupgroup /sync` — plain text goes
to the Atlas brain.

## 5. Your first 3 missions (copy-paste)

**1. A dispatch.** Drop any repo under `~/Station/<Category>/<Name>` (it's
auto-discovered), then:

```bash
omega dispatch MyApp "Add input validation to the signup form: email format, password >= 12 chars, inline errors. Verify with the existing test suite."
omega timeline oracle-MyApp     # watch what the oracle did, step by step
```

The oracle plans, spawns workers, gates the result, and (with Telegram wired)
posts the progress card + final PDF in the project's topic.

**2. A new project, end to end.**

```bash
omega new-project acme --dry-run    # print the plan first
omega new-project acme              # provision -> scaffold -> vision -> PRD -> plan -> build
```

Or in any Claude session: `/omg-new-project`. With service keys in
`~/.omega/provisioning/services.env` it auto-provisions Vercel / Convex /
GitHub / Clerk / Stripe; without them it pauses honestly and tells you what it
needs. The build phase is a typed plan executed by `omega plan-run`.

**3. An audit.**

```bash
omega audit list                  # the 23-audit arsenal
omega audit run codeaudit --dir . # prints the exact spawn-worker command to run it
omega audit select "fix auth"     # which audits a mission would auto-trigger
```

Or inside a Claude session, invoke the skill directly: `/omg-codeaudit`,
`/omg-secaudit`, `/omg-uiuxaudit`, …

## 6. The skill catalog

Shipped in `skills/` (repo) → `~/.omega/skills/` (installed); invoked as
`/omg-<name>` in any Claude session. Grouped:

| Group | Skill | One line |
|---|---|---|
| Pipeline | `vision` | Product identity + emotional positioning via Socratic discovery → VISION.md. |
| | `prd` | Full product-docs suite (requirements, stories, stack, milestones) for agent implementation. |
| | `brand-identity` | Complete brand system → deployed interactive brand book (opt-in step). |
| | `planner` | Decompose work into a typed DAG → `.planner/tracker.json` for `omega plan-run`. |
| | `new-project` | The end-to-end bootstrap: provision → scaffold → vision → PRD → plan → build. |
| Quality | `audits/` (23) | The forensic arsenal: code, sec, perf, a11y, uiux, flow, api, copy, data, debug, dep, dx, feature, i18n, logic, motion, observability, privacy, refont, release, retention, seo, automation. |
| | `acceptance` | Autonomous browser-acceptance + self-heal gate: sweep every route, fix, re-run until green. |
| Decision | `llm-council` | Four Claude models answer independently, peer-review anonymously, a president synthesizes with dissent. No API keys. |
| Marketing | `market-research` | Market size, trends, competitors, decision-makers (gooseworks API). |
| | `marketing-strategist` | GTM/product-marketing/demand-gen strategy lens. |
| | `product-marketing-context` | The positioning/ICP/messaging doc every other marketing skill reads — run first. |
| | `content-strategy` | Topic clusters, pillars, editorial calendar. |
| | `social-content` | Organic posts, threads, carousels, short-form scripts. |
| | `cold-email` | B2B outbound emails + follow-up sequences. |
| | `ad-creative` | Paid-ad copy at scale, iterated on performance data. |
| | `launch-strategy` | Launch/GTM plan: Product Hunt, waitlist, channels. |
| Browser | `browser-use` | Agentic cloud browser for tasks scripted Playwright can't express. |
| Ops | `cleanup` | Disk/session/cache hygiene for the VPS and projects. |
| | `project-tidy` | De-sprawl a repo polluted by agent output (docs/ + agentic/ convention). |
| | `ramflush` | Kernel cache purge + before/after perf report. |
| | `linear` / `linear-setup` | Resolve Linear feedback tickets end to end / one-time widget+labels wizard. |
| Media | `pdfgen` | The branded PDF engine behind `omega pdf` (whitepaper/audit/marketing/doc). |
| | `higgsfield-soul-id` / `higgsfield-generate` | Train a face-faithful Soul character / generate brand-quality images, video, ads. |
| Design | `design` | Tier-1 design-taste engine (inspiration canon) used by the UI pipeline. |

Your installed `~/.omega/skills/` may hold more (user-added libraries); `ls
~/.omega/skills` is the live list, and the Telegram bot's `/skills` shows it
too.

## 7. How work is verified

Nothing in OmegaOS is "done because the agent said so":

1. **done.json** — a worker must write a typed completion signal; the gate
   checks every cited artifact (commit SHA on the remote, file paths, command
   exit codes, URL probes) against reality before `done_clean` is accepted.
2. **Gate (structural)** — in `omega plan-run`, a step cannot be skipped: the
   engine only advances the DAG when the step's `done.json` lands and its
   invariants hold. Skip-prone plans are rejected before the run starts.
3. **Guardian (independent)** — every plan step carries a real
   `verify_command`; the engine runs it itself. A worker's self-report alone
   never completes a step (trivial verify commands like `true` are refused).
4. **Adversarial 2-of-3 consensus** — claims go to independent grader lenses
   (rubric, Popper falsification, regression); a claim survives only if a
   majority agree. The grader is never the writer (R-VERIFY).
5. **Audits at the gate** — the oracle auto-selects the Quality Arsenal audits
   that fit what changed (`omega audit select`).
6. **The mission PDF** — the final report, with a linked "steps to verify"
   section, lands in the project's Telegram topic so the human can re-check
   the proof, not just read the summary.

## 8. Day-2 operations

- `omega doctor` — first command when anything feels off; names the broken
  piece. `omega doctor --fix` repairs the mechanical ones (a self-heal cron
  also runs it every 3 h).
- `omega patrol` — the watchdog (cron, every minute): restarts the rmux
  daemon, resurrects crashed oracles, reaps stale workers.
- `omega resurrect` — bring dead oracles back from persisted state after a
  crash or daemon restart.
- `omega timeline <oracle>` — replay a mission dispatch-by-dispatch when
  something looks stuck.
- `omega usage` — token-budget status; Telegram alerts fire at 80/90%.
- `omega backup` — one `.tgz` of the irreproducible state (`~/.omega` +
  crontab) to `scp` off the box before a reset. Restore procedure:
  [docs/RESET-RECOVERY.md](docs/RESET-RECOVERY.md).
- `omega cleanup` / `omega kill-all` / `omega clean-junk` — session and state
  hygiene, safest first.
- **Logs** live in `~/.omega/logs/` (session logs, bot logs); runtime state in
  `~/.omega/state/`; audit results in `~/.omega/audit/`.

## 9. For LLM agents operating OmegaOS

If you are an agent reading this on an OmegaOS box, this is your contract:

- **What's injected into you.** `~/.omega/OMEGA.md` (universal instructions —
  synced into every LLM CLI's config by `omega sync`) and your role-scoped
  slice of the Laws + Rules via `rules::agent_context_block(scope)`. The full
  registry: `omega rules list`; editable copies in `~/.omega/rules/`.
- **Where state lives.** `~/.omega/state/` — session metadata, scope locks,
  done signals, progress files. Project plans live in the project's
  `.planner/tracker.json`.
- **How to signal done.** Run
  `omega done <session> <status> "<summary>" [--commit <sha>]` with status
  `done_clean | pending | failed | blocked`. This writes
  `~/.omega/state/worker-<session>.done.json`. The schema
  (`crates/omega-core/src/done.rs::DoneSignal`) carries: `session`, `status`,
  `summary`, optional `commit`, `finished_at`, `todos_total` /
  `todos_completed`, `pending_actions[]`, and the ground-truth fields —
  `not_done[]` (REQUIRED honesty: what is not done, even on success),
  `scope.confirms[]` / `scope.does_not_confirm[]`, `corroboration[]`
  (independent signals: `git_remote`, `ci_exit_code`, `prod_healthcheck`,
  `filesystem_check`, `independent_auditor`, … — `worker_self_report` alone is
  never sufficient), `failure_mode` (fabrication / instruction_following /
  cheap_verification_skipped / ignored_correction), `retry_thrash_count`, and
  `artifacts[]` (citations the gate verifies: `git_sha`, `git_branch`,
  `file_path`, `command` + exit code, `url` + expected status, `note`).
- **How to report progress.** Oracles:
  `omega progress <session> --plan "task1|task2|task3"` to set the plan, then
  `omega progress <session> --task "task1" --status done|fail|doing|todo` per
  task. Writes `~/.omega/state/oracle-<key>.progress.json`; the Telegram bot
  renders it as the live card.
- **How to signal blocked.** `omega done <session> blocked "<what blocks you +
  the fallback you already started>"`. Never idle waiting for a human (L3);
  finish every file-disjoint safe task first (L4).
- **The scope-claim protocol.** Before you (or a worker you spawn) write a
  file, the file set must be claimed: `omega spawn-worker <task> "<prompt>"
  --files a.rs,b.rs` claims them with real advisory locks; `omega scope`
  checks for conflicts. Overlapping scope → serialize, or isolate with
  `--worktree` and merge back with `omega-git-merge`. Never two writers on one
  file (R-SCOPE).

## 10. Sharing it / installing on a new box

```bash
npx omega-os          # guided install (Matrix screen, Telegram wizard)
# or
git clone https://github.com/agentik-os/OmegaOS && cd OmegaOS && ./install.sh
```

**What a fresh install reproduces** (Law 0 — install parity): the `rmux` +
`omega` binaries (prebuilt when a release exists, else built from source), the
doctrine, the agents, all shipped skills, the Telegram bot services, the
self-healing crons, shell integration, and the hardened terminal config.

### Updating an existing box

```bash
omega update           # fetch + fast-forward + rebuild + reinstall
omega update --check   # report only — changes nothing
omega update --dir /path/to/OmegaOS   # when the checkout isn't where it looks
```

`omega update` is the supported way to move an install forward. It always
rebuilds from source, so you get exactly the commit you pulled — `main` runs
ahead of the latest release tag, and the prebuilt artifact from that tag would
be OLDER than the code you just fetched.

Your `~/.omega` state survives: `config.toml`, `telegram.toml`, secrets and
`projects.json` are all guarded by the installer and never clobbered.

The update **refuses rather than clobbers**. Local changes or unpushed commits
in the checkout stop it with the exact git command to resolve them — it will
never stash, reset or discard your work. (`npx omega-os` also updates an
existing clone, but it dies on a dirty checkout with a raw git error and can
clone a second copy if run from a different directory — prefer `omega update`.)

An update that pulls nothing is not the same as an install that is current. If
you commit to the checkout yourself, git has nothing to fetch, yet the binary
still predates your commit. `omega update` now owes an install whenever HEAD is
not the commit that was actually installed, and `omega doctor` carries a
**binary provenance** line that goes red with the command that fixes it:

```
[x] binary provenance   installed binary is from a54e09e, checkout HEAD is 3e84acd
                        — run: cd ~/Station/SideBusiness/OmegaOS && ./install.sh
```

### Reconciling after an update

A new binary is not a coherent install. The doctrine exported on disk is still
the previous binary's, finished workers may still hold their panes, and every
session running right now was briefed by the binary you just replaced.

```bash
omega reconcile                # fix what is mechanical, report the rest
omega reconcile --report-only  # change nothing, just tell me
```

It repairs only what is deterministic: re-exports the doctrine to match the
registry this binary carries, records which commit is installed, sweeps workers
that already wrote a terminal done signal. Everything needing judgement is
**named and left alone** — a project registered at a path that no longer
exists, a project with no `CLAUDE.md` / `AGENTS.md` / `OMEGA.md` for agents to
read, a live session still running on the previous doctrine. It will not
rewrite your project docs and will not restart your sessions: a reconciler
doing that unattended is a worse problem than the drift it set out to fix.
It exits non-zero while anything is left for you, so a cron can alert on it.

You rarely run it by hand. `install.sh` runs it inline at the end of every
install, and a successful nightly update launches it in its own session:

```bash
omega stream omega-reconcile   # read what the nightly update found
```

### Staying current on its own

Every install schedules a **daily check at 03:30** that installs what it finds:

```bash
omega config set auto_update apply   # check daily and install  (the default)
omega config set auto_update check   # check daily, alert me, install nothing
omega config set auto_update off     # do nothing
omega config get auto_update         # what is it set to right now?
```

Be clear about what `apply` means: whoever controls the OmegaOS repo controls
this machine's OmegaOS, renewed every night rather than once at install time.
That is the same trust you extended to the installer — if you would rather
approve each change yourself, `check` gives you the alert without the install.

The nightly run is deliberately timid, and each refusal is logged with its
reason in `~/.omega/logs/omega-auto-update.log`:

- **nothing to pull** — it exits in under a second (the normal night)
- **local changes or unpushed commits** — never touched, the update is skipped
- **an agent is mid-turn** — deferred to the next night rather than rebuilding
  the binary under a running mission
- **the same commit failed 3 times** — it stops retrying and alerts you instead
  of thrashing nightly on a broken build
- **an update already running** — one at a time, always (a lock older than 6h
  is treated as a crashed run and cleared)

You get a Telegram alert when an update is installed, when one is available
under `check`, and when something needs you. Silence means it had nothing to do.

`omega update --auto` runs that exact path by hand if you want to watch it.

**What is local secret state** — recreated per machine, never in any repo:
`~/.omega/credentials/` (OAuth tokens, API keys), `telegram.toml`,
`deposit.toml`, `agent-bots.json` (per-project bot tokens),
`provisioning/services.env`, and companion-bot secrets (e.g.
`nova-secrets.env`). `omega backup` is how they survive a box reset.

**Opt-in extras** at install time: `OMEGA_WITH_BROWSER=1 ./install.sh` adds
the Playwright/browser stack (skippable later via the printed one-liner);
`OMEGA_WITH_NOVA=1` for the companion-bot stack is rolling out alongside it.
After any install or update: `omega doctor` — every line `[+]`.
