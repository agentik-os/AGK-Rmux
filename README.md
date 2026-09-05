<p align="center"><a href="https://discord.gg/agentik-os"><img src="https://img.shields.io/badge/Discord-Agentik_OS-c5f277?style=flat-square&amp;labelColor=10161c" alt="Agentik OS community"></a></p>

<p align="center"><img src="docs/assets/readme/ecosystem-hero.svg" width="880" alt="AGK-Rmux · A control plane for durable agent work."></p>

# AGK-Rmux

[Install](#guide-1) · [Updating](#guide-2) · [First 5 minutes](#guide-3) · [What you can do](#guide-4)

> Follow the installation and limits sections, using your own machine and provider accounts. Historical OmegaOS material below documents the shared lineage; it does not grant access to an existing fleet.

<p align="center"><img src="docs/assets/readme/ecosystem-flow.svg" width="880" alt="Intent → Orchestration → Runtime → Evidence. System overview. Authenticate your own providers after installation."></p>

**Agentik OS (AGK)** — control plane on [rmux](https://github.com/agentik-os/rmux).

This is the clean product home. The tree starts from [OmegaOS](https://github.com/agentik-os/OmegaOS) so we can rebrand without locking that line. Same monorepo, same installer, topology-agnostic profiles (Operator / Agentik / Mission / Private).

We ship an opinionated default, not a locked architecture.

- Topology: [docs/TOPOLOGY.md](docs/TOPOLOGY.md)
- Four profiles: [docs/SCOPE-TENANCY.md](docs/SCOPE-TENANCY.md)
- Discord surface: [docs/DISCORD-SURFACE.md](docs/DISCORD-SURFACE.md)
- Rebrand notes: [REBRAND.md](REBRAND.md)

---

# OmegaOS

A terminal control plane for running a fleet of AI coding agents in parallel, where every agent obeys the same typed rulebook.

[English](README.md) | [Français](README.fr.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [Español](README.es-ES.md)

[![CI](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml/badge.svg)](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml) ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg) ![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)

OmegaOS is not a library you import. You install it on a Linux box and you get the `omega` command, a TUI for watching and killing sessions, an orchestration layer that hands work to agents, and a Telegram bridge for phone control. New sessions use OpenAI Codex by default. Claude Code, Google Antigravity, enterprise/API-key Gemini CLI, OpenRouter/Pi, Hermes, GLM, and Kimi remain explicit choices. Every agent receives a compact, typed, role-scoped policy context compiled from the same doctrine.

Current version: see [CHANGELOG.md](CHANGELOG.md) (`omega -V` on an installed box). I run it daily; expect rough edges.

<a id="guide-1"></a>
## Install

One command on a Linux box (macOS mostly works):

```
npx omega-os
```

It clones the repo and runs the installer behind an interactive Matrix-rain progress screen (type to inject glyphs, `space` to pulse; `npx omega-os --plain` for a plain bar). Prefer to do it by hand:

```
git clone https://github.com/agentik-os/OmegaOS
cd OmegaOS
./install.sh
```

The installer downloads prebuilt `rmux` + `omega` + `omega-gatewayd` binaries for your platform when a release is published (verified by checksum), and falls back to building from source otherwise — so a fresh clone always reproduces the system, just faster when a binary exists. Force a source build with `OMEGA_FROM_SOURCE=1 ./install.sh`.

<a id="guide-2"></a>
## Updating

```
omega update           # fetch + fast-forward + reinstall
omega update --check   # what would change? (touches nothing)
omega reconcile        # after an update: fix what is mechanical, report the rest
```

It updates the checkout it finds (`$OMEGA_SRC`, the current directory, then
`~/Station/SideBusiness/OmegaOS`, `~/Station/OmegaOS`, `~/OmegaOS` — or pass
`--dir`), rebuilds from source, and re-runs the installer. Your `~/.omega`
state is preserved: secrets, projects, Telegram config and `config.toml` are
never overwritten.

If you have local changes or unpushed commits in the checkout, the update
**stops and tells you** rather than touching your work — commit, stash or push,
then re-run it.

It also keeps itself current: every install checks daily at 03:30 and installs
what it finds, skipping any night where your checkout has local work, an agent
is mid-turn, or the same commit has already failed three times. Auto-installing
means trusting the repo nightly, so the switch is one command:

```
omega config set auto_update check   # alert me instead of installing
omega config set auto_update off     # do nothing at all
```

<a id="guide-3"></a>
## First 5 minutes

The stack installs itself; only the personal pieces are left. **`omega guide`
prints the full step-by-step** (also saved at `~/.omega/GETTING-STARTED.md`,
and shown at the end of the install). In short:

1. **Connect Codex** *(required for the default runtime)*: run `omega codex-login`, then check with `omega codex-login-status`. Claude remains optional through `claude auth login`.
2. **Telegram remote** *(recommended)* — token from [@BotFather](https://t.me/BotFather), your id from [@userinfobot](https://t.me/userinfobot), then `OMEGA_TG_TOKEN=<TOKEN> omega telegram setup <ID> --user-id <ID>` (the env form keeps the token out of the process list). For one-topic-per-project: group + Topics on + bot admin → `/setupgroup` → `/sync`.
3. **Service keys** *(optional)* — `~/.omega/provisioning/services.env` (Vercel / GitHub / Convex / Stripe / OpenAI-for-voice) powers auto-provisioning of new apps.
4. **Add a project** — `omega` → **[N] New Project**, Telegram → *Import from GitHub*, or just drop a repo under `~/Station/<Category>/`.
5. **Verify** — `omega doctor`: investigate every `[x]`; treat `[!]` as an
   explicit warning rather than a pass.

Representative invariant lines from the source-built runtime are below.
Session counts, paths, installed provenance, authentication, and service health
are machine-specific and must come from your own run:

```
OmegaOS doctor

  [+] binary           omega 0.1.14
  [+] doctrine         7 Laws + 52 Rules
  ... machine-specific checks follow ...
```

`[!]` lines are warnings with the repair command inline; `omega doctor --fix` repairs the mechanical ones.

<a id="guide-4"></a>
## What you can do

- **Dispatch missions.** `omega dispatch <Project> "<mission>"` hands work to that project's oracle, which plans, spawns workers, and gates the result. `omega orchestrate` runs the full classify → plan → dispatch → monitor → gate pipeline in one command.
- **Run typed plans.** `/omg-planner` decomposes a build into a typed DAG (`.planner/tracker.json`); `omega plan-run` executes it with structural can't-skip enforcement (Gate) and independent verify-command proof (Guardian).
- **Bootstrap whole apps.** `/omg-new-project` provisions Vercel/Convex/GitHub/Clerk/Stripe from your keys, scaffolds the stack, then runs vision → PRD → plan → build.
- **Parallelize safely.** Workers claim their files with real advisory locks (`fs2`), and `omega spawn-worker --worktree` gives each parallel worker its own git worktree with a clean merge at the end. A done signal creates a candidate result. Only an independent verifier and the mission acceptance gate can close it.
- **Audit everything.** A Quality Arsenal of 23 forensic Gestalt-Popper audits (`codeaudit`, `secaudit`, `perfaudit`, `a11yaudit`, …) auto-selected for what changed, plus `/omg-acceptance` — an autonomous browser-acceptance gate that sweeps every route and fixes what it finds.
- **Convene a council.** `/omg-llm-council` puts one question to four different Claude models in parallel, has them peer-review each other anonymously, and synthesizes a verdict with the dissent intact — no API keys, it runs inside your existing session.
- **Browse agentically.** `/omg-browser-use` drives a cloud browser for tasks scripted Playwright can't express.
- **Do the go-to-market too.** A vendored marketing pack (market research, positioning, content strategy, social, cold email, ad creative, launch strategy) plus the Higgsfield visual-identity pair.
- **Get reports on your phone.** With Telegram configured, completed mission delivery can include a branded PDF report in the project's topic while a live progress card updates in place. A deposit bot gives agents a private inbox for files you send from your phone.
- **Operate it.** `omega doctor` (whole-stack health), `patrol` (session watchdog), `usage` (token budget + Telegram alerts), `backup` (irreproducible `~/.omega` state → one tgz), `cleanup` / `kill-all`, `timeline` (replay a mission), `resurrect` (revive a crashed oracle), `provision` (per-client credential groups).
- **Resolve Linear tickets end to end.** `/omg-linear` fixes, captures evidence, audits to 100/100, comments, and moves the ticket to review — never to Done; a human does that. See [Linear integration](#linear-integration).
- **Learn from any book — the Librarian.** ALEXANDRIA OS (`/alexandria`) is a personal librarian, learning coach and intellectual sparring partner: it X-rays a book chapter by chapter, distills an idea across many books, puts authors in combat, challenges your plan in a 10-round sparring, builds flashcards and spaced repetition, and finds the 50 best books worldwide on a topic with 50 actionable tips (`/best`). It runs as a Skill in any session AND as its own **Telegram persona bot** you connect in one tap — **Menu → 🤖 Agents → 📚 Link your librarian (Alexandria)**, paste a @BotFather token, done. It replies in **English by default** (`/language fr` to switch), transcribes voice notes and audio, reads PDF/EPUB files, and starts with `/setup` to adapt to how *you* learn. Same wiring links your companion (Nova) and security agent (Trinity).

Three ways in: the `ratatui` TUI (7 tabs: Sessions, Projects, OS, Menu, System, Help, Settings — opening a project offers Coding, Marketing, or the Oracle, each under any installed LLM agent), the `omega` CLI (`omega --help` is the live command inventory), and the Telegram hub. An RPC mode (JSONL over stdin/stdout) drives it from other tools. Underneath, it all runs on [rmux](https://github.com/agentik-os/rmux), a Rust terminal multiplexer — no tmux dependency.

<a id="guide-5"></a>
## The OS suite

OmegaOS registers 24 canonical operative-system products under [`OS/`](OS/),
grouped as Personal, Build, Growth, and Systems. Five older slugs remain as
compatibility aliases. The TUI's **OS** tab derives readiness labels from
filesystem evidence; those labels are not claims that an OS has passed live
runtime verification. See the canonical registry and alias table in
[`docs/OS-SUITE.md`](docs/OS-SUITE.md).

<a id="guide-6"></a>
## The doctrine

There's a typed registry of 7 Laws and 52 named operational Rules. `omega rules list` prints the current set. The compiler lives in `crates/omega-core/src/rules.rs`; it emits a deterministic, provider-aware context with a hard 24 KB OmegaOS budget.

**Laws are inviolable.** They bind every agent and override every rule and task. There are seven:

- **L0 — Ship the truth.** A change isn't done until a clean rebuild reproduces it and it's pushed. Anything less is a draft.
- **L1 — Runtime is the only truth.** Code and comments state intent. Only running it reveals reality. When they disagree, runtime wins.
- **L2 — Researcher, not sycophant.** Challenge a flawed premise with reasoning before you act. No fake confidence. "This should work" without evidence is a lie.
- **L3 — Decide and proceed.** A dispatched agent is autonomous. It never stops to ask "should I continue?" It decides, executes, and reports after.
- **L4 — Done means 100%, verified.** 92% is not done. Enumerate the tasks, finish each, verify each against runtime.
- **L5 — Quality over speed.** No streamlined, lightweight, or quick variant of a real protocol. A 403 or a 401 is an abort, not a pass.
- **L6 — Finish the mission.** Enumerate, execute, verify, and report every requested deliverable. A plan or partial phase is not a legal stopping point.

**Rules are operational.** Named (R-SCOPE, R-VERIFY, R-CITE, …) and sorted into Universal, QualityGate, Orchestration, Reporting, and Safety. Each Rule is scoped to the roles it binds: Master, Oracle, Worker. A worker doesn't get burdened with orchestration rules it can't act on, and an oracle doesn't carry the worker's file-locking discipline. Same registry, different slices.

### The funnel

This is the mechanism. `rules::compile_rule_context_for_provider` combines the compact law kernel, role contract, mission-relevant rules, provider mechanics, and skill references. It rejects output above the context budget instead of truncating silently. Every compiled context has a deterministic digest for drift detection.

A worker three levels down the tree carries the same seven Laws as the Master. Operational procedures are loaded only when role, mission, risk, and provider require them. This keeps the invariants universal without injecting every runbook into every turn.

See the whole thing:

```
omega rules list
```

![omega rules list — the Laws and Rules, printed by OmegaOS](assets/omega-rules.svg)

<a id="guide-7"></a>
## Architecture

Four levels, top to bottom:

```
┌─────────────────────────────────────────────────────────────────┐
│  Level 1 — Human Interface                                      │
│  TUI (7 tabs) · CLI (`omega --help`) · Telegram hub             │
│                      ↓ intent                                   │
├─────────────────────────────────────────────────────────────────┤
│  Level 2 — Atlas Telegram orchestration service                 │
│  15 typed Matrix agent templates, classify → dispatch           │
│                      ↓ dispatch                                 │
├─────────────────────────────────────────────────────────────────┤
│  Level 3 — Oracle (1 per project)                               │
│  Classify → Plan → Dispatch workers → Quality gate              │
│                      ↓ decompose                                │
├─────────────────────────────────────────────────────────────────┤
│  Level 4 — Workers (ephemeral, parallel, file-lock scoped)      │
│  Execute → Verify → done.json → Oracle acks → close             │
└─────────────────────────────────────────────────────────────────┘
```

**Level 2 — Atlas.** A persistent Telegram service receives operator messages,
resolves the project or topic, and dispatches work to an oracle. The typed
registry contains 15 Matrix-named agent templates, including Trinity. The
legacy `aisb-master` session is a read-only conversation viewer, not the
routing authority.

**Level 3 — Oracle.** One per project. It classifies the request, plans, dispatches workers, and runs the quality gate at the end. An oracle orchestrates. It does not edit project code itself, so the grader and the writer are never the same agent.

**Level 4 — Workers.** Ephemeral. They run in parallel, each scoped to its own files by a file-lock claim (advisory locks via `fs2`) — and optionally to its own git worktree. A worker signals completion by writing a `done.json` with status `done_clean`, `pending`, `failed`, or `blocked`; without that status it isn't done. Writing the signal is also what closes it: see [how a worker closes](#how-a-worker-closes).

### How a mission runs

A request enters via the TUI, CLI, or Telegram. Direct dispatch resolves the
project oracle; Telegram messages first pass through Atlas for topic and
project routing. The oracle plans the mission, splits it into tasks, and
dispatches workers. A worker's `done.json` is only a candidate projection;
independent verification and the mission acceptance gate decide closure.

A worker doesn't have to chew through its subtasks one at a time. It can run a workflow in-process: spawn parallel sub-agents, check their outputs, and combine them into one answer. Code review uses this, as do research, audits, and design work.

Verification is deliberately adversarial: a worker reporting "done" doesn't end the check; its claim goes to independent agents, and it only survives if a majority (two of three) agree. The Quality Arsenal audits plug in right here, at the gate.

This depends on the doctrine funnel above: every agent, at every level, gets its role-scoped Laws and Rules injected the moment it's dispatched.

This README section is itself an example. A workflow produced it. One agent wrote the draft, independent readers went through it hunting for AI-generated prose, another agent revised against what they flagged, and native speakers handled the translation. So no part of this text came from a single unreviewed pass.

### How a worker closes

A worker's `done.json` is not just a report, it is the trigger for its own teardown. When a terminal status lands (`done_clean`, `failed`, or `blocked`), `omega done` closes that session's pane, releases its file-scope claim so the next worker can claim those paths, and unregisters its git worktree when the worktree holds nothing unsaved. `pending` is deliberately not a stop, and a worker with no signal at all is still working and is never touched.

`omega reap` is the same reconciliation run by hand, for the stragglers:

```
omega reap --dry-run     # print what would close, change nothing
omega reap               # close every worker holding a terminal signal
omega reap <session>     # just this one
```

It is idempotent. Reaping twice does what reaping once does, and an already-closed session is a quiet exit 0.

Two properties matter more than they look. The scope claim is released *before* the pane dies, because a leaked claim outlives the worker and rejects the next `spawn-worker` on those files with an owner nobody can find. And a worktree carrying uncommitted or unmerged work is kept, always: losing a worker's commits is worse than leaving a directory behind, and the leftover directory is recoverable while the commits are not.

The `patrol` watchdog runs the same logic on a timer for workers whose oracle died mid-mission. That sweep is bounded at both ends: it waits out a grace period so a re-dispatch can't be raced, and it ignores any completion signal too old to have plausibly governed a live worker.

### When a mission is a graph, not a list

Most missions are a list of tasks with dependencies, and `omega dispatch` handles those. Some are not. When a mission needs a branch taken on a classification, a stage that repeats until it stops finding anything, or a step that must not run unattended, `omega-core` carries a typed, persisted mission **graph** instead.

The vocabulary is small. Nodes are units of work; edges are the dependencies between them. A **router** resolves a classification string through an exact table lookup and then a default, so the same classification lands on the same node on every machine and in every replay. A model may produce the classification, but it never decides the branch. **Fan-out** is the set of nodes the executor reports as runnable right now, given the edges and the budgets.

**Convergence** is enforced structurally rather than hoped for. Every back edge carries a finite iteration bound, and graph validation refuses a graph that is still cyclic once the bounded edges are cut. At runtime three counters only ever move forward (attempts per node, traversals per edge, lifecycle transitions), so no sequence of steps can spin forever. Bounded retries live on the node itself, and an attempt is counted *before* the ceiling is tested, so a limit of three runs a node exactly three times. Re-entering a loop does not refund the budget. When a node fails terminally with no live fallback, everything reachable only through it is reported as cancelled rather than left queued, and "nothing will ever be ready" is a distinct answer from "nothing is ready yet".

In front of execution sits the **risk gate**, which is R-DESTRUCT expressed as a type. The executor decides what *may* run from edges and budgets alone, which means a node that drops a schema is runnable by exactly the same test as one that prints a file. The gate is the check that stands in between. A node with no risk classification defaults to elevated, never safe: the absence of a classification is not evidence that something is harmless. Running unattended, an escalated node produces a durable escalation record naming the step and what would be lost, rather than a prompt nobody is there to answer. Approvals and denials are attributed and refuse an empty approver, so an agent cannot write its own permission slip.

```
omega risk-gate show    <graph.json> <node>
omega risk-gate approve <graph.json> <node> --approver "<who>"
omega risk-gate deny    <graph.json> <node> --approver "<who>"
```

`--approver` is a required flag on both verdicts, not a courtesy: the attribution is the point.

The graph itself is a library primitive in `omega-core` (`graph.rs` for the vocabulary, `graph_executor.rs` for the decision core, `graph_risk.rs` for the gate), not a CLI surface: `omega risk-gate` is the operator-facing half. The core is deliberately pure, with no process spawn, no network, no filesystem and no clock anywhere in it, which is the only reason a mission replays from persisted state and reaches the same decisions on another machine. `docs/GRAPH-EXECUTION-LAYER.md` is the full contract.

<a id="guide-8"></a>
## Stack

It's a Rust-first workspace with four members:

- `omega-core` — orchestration, the rules registry, doctor, timeline, cleanup, patrol, file-scope locking.
- `omega-cli` — the `omega` binary, built on `clap`.
- `omega-tui` — the session manager, built on `ratatui`.
- `omega-gateway` — the optional HTTP gateway surface.

Underneath, it runs on [rmux](https://github.com/agentik-os/rmux), a Rust terminal multiplexer: a daemon, a typed SDK, and PTY handling. rmux is a typed Rust library, so OmegaOS calls it directly instead of shelling out to tmux and parsing text. There is no tmux dependency anywhere.

Bun and TypeScript power the Telegram bots and supporting scripts. Shell is
used for bootstrap and operational wrappers; Rust remains the primary runtime.

<a id="guide-9"></a>
## Connecting remotely

The rmux daemon owns every session, so your agents keep running after you disconnect. To get back to them, **attach** — reconnect your terminal to a session that's already running:

```
rmux attach              # re-attach to the last session
rmux attach -t claude-1  # attach to a specific one
rmux list-sessions       # see what's live
```

Detach again with `Ctrl-b d` — the session and its agents keep running without you.

`omega` wraps the entrypoints you actually reach for:

```
omega                       # open the TUI session manager (browse / launch / monitor)
omega attach -t claude-1    # drop straight into one session to work in it
omega aisb-view             # open the read-only Telegram conversation viewer
omega list                  # list every live session
```

Use the menu (`omega`) to manage and launch; use a direct attach (`omega attach -t …`, or `rmux attach -t …`) when you want to type heads-down in a single session — the menu's preview *mirrors* the pane, while a direct attach is the lowest-latency path.

Over SSH from a laptop, plain SSH waits a full network round-trip before echoing each keystroke, so on a distant box typing feels laggy and agent output arrives in chunks — no matter how fast the box is, because it's latency, not CPU. `install.sh` installs [`mosh`](https://mosh.org) for this: it echoes your keystrokes locally and ships screen diffs over UDP, so typing is instant and streaming is smooth at any latency. Connect straight into a session with:

```
mosh user@host -- omega attach -t claude-1
```

In a client like **Termius**: set the host IP + port, turn the **mosh** toggle on, and add a startup snippet — `omega` for the menu, or `omega attach -t <session>` to land directly in a session.

(Use rmux's `Alt+Up/Down` for scrollback, not mosh's PageUp.) The installer also wires `/etc/rmux.conf` and a UTF-8 locale system-wide, so every account — root and future users — gets the same hardened session (mouse scroll, drag-select to the local clipboard over SSH, snappy keys, truecolor) with no per-user setup.

<a id="guide-10"></a>
## Linear integration

If you track user feedback in [Linear](https://linear.app), OmegaOS resolves the tickets end to end. Two commands.

`/omg-linear-setup` is a one-time wizard, run inside your own app. It installs an in-app feedback widget (it captures a screenshot, the page URL, the clicked element, and the browser console at report time), the Linear labels the pipeline keys off, and the API route that turns a widget report into a Linear issue. It detects your stack, auth provider, and UI library first, so it writes code that fits the project rather than a generic template.

`/omg-linear` does the work. It reads the open tickets, and for each one it fixes the code, captures before/after evidence, then runs the Quality Arsenal audits that fit the change. A ticket only advances if those audits hit 100/100. Then it posts a fix-verification comment on the ticket and moves it to a review state — `In Review` if your team has one, otherwise a neutral `Omega Review` it creates. It never marks a ticket Done; a human does that after checking. The v2 engine runs this through a Workflow: it triages the open tickets, fans the per-ticket fix-and-audit out in parallel, and verifies each resolution adversarially before commenting.

It's trigger-guarded. OmegaOS only touches Linear when you ask for it by name (`/omg-linear`, `fix linear`, a ticket id like `KOM-7`, or a `linear.app` link). The bare word "feedback" never sets it off, and it won't mention Linear unless you do.

```
omega_dir=~/.omega          # the protocol ships to ~/.omega/skills/linear/
/omg-linear-setup           # once per app — installs the widget + labels + route
/omg-linear                 # resolve open tickets: fix -> audit -> comment -> In Review
```

<a id="guide-11"></a>
## Limits

I'd rather you know these going in.

- **Linux-first.** Developed on a headless VPS. No Windows. macOS gets real fixes (launchd services, Homebrew path) but is less exercised.
- The TUI assumes a 256-color terminal. On a 16-color terminal it'll be ugly.
- The default agent runtime is OpenAI Codex, so the `codex` CLI must be logged in. Claude Code, Antigravity, enterprise/API-key Gemini CLI, OpenRouter/Pi, Hermes, GLM, and Kimi are supported alternatives.
- **Single machine.** The rmux daemon is local. There's no multi-host orchestration.
- It's 0.1.x. I use it daily, but you'll find rough edges I haven't hit yet.

<a id="guide-12"></a>
## Read GUIDE.md next

**[GUIDE.md](GUIDE.md)** is the operator manual: the vocabulary (mission, oracle, worker, goal, plan, Atlas), the three cockpits, your first missions, the skill catalog, and how work gets verified. Then go deeper:

- [docs/FEATURES.md](docs/FEATURES.md) — **the complete feature catalog** (every subsystem + how to reach it).
- [docs/README.md](docs/README.md) — the documentation index.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — the full-system reference.
- [docs/MAP.md](docs/MAP.md) — where everything lives on disk.
- [docs/THEMES.md](docs/THEMES.md) — the TUI palette gallery.
- [docs/RESET-RECOVERY.md](docs/RESET-RECOVERY.md) — backup and rebuild a box.
- [CHANGELOG.md](CHANGELOG.md) — what shipped, release by release.

<a id="guide-13"></a>
## Credits

OmegaOS builds on a lot of other people's work:

The largest debt is [rmux](https://github.com/agentik-os/rmux), the Rust terminal multiplexer everything here runs on.

The rest of the Rust stack:

- [ratatui](https://github.com/ratatui/ratatui) and [crossterm](https://github.com/crossterm-rs/crossterm) — the TUI.
- [tokio](https://github.com/tokio-rs/tokio) — the async runtime.
- [clap](https://github.com/clap-rs/clap) and `clap_complete` — the CLI and shell completions.
- [serde](https://github.com/serde-rs/serde) with `serde_json`, `serde_yaml`, and `toml` — config and state.
- [anyhow](https://github.com/dtolnay/anyhow) and [thiserror](https://github.com/dtolnay/thiserror) — error handling.
- `chrono` (timestamps), `dirs` (paths), `fs2` (the advisory file locks behind scope claims), `regex`, `tempfile`, `tracing` with `tracing-subscriber` (logging), and `reqwest` (Telegram and PDF HTTP).

[Claude Code](https://www.anthropic.com) by Anthropic is the agent runtime.

<a id="guide-14"></a>
## License

Dual licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Standard Rust convention. Pick whichever you prefer.

---

**Community and licensing** · [Discord](https://discord.gg/agentik-os) · [LICENSE-APACHE](LICENSE-APACHE) · [LICENSE-MIT](LICENSE-MIT)

Illustrative visuals, not live telemetry. Service access and secrets belong to each installation.
