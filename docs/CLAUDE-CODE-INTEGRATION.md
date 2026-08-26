# Claude Code CLI → OmegaOS Integration

> **Historical research snapshot.** Paths and feature status below describe
> the May 2026 design, not the current runtime. Today interactive launches live
> in `crates/omega-core/src/agents.rs`, headless chat streaming lives in
> `crates/omega-gateway/src/chat_driver.rs`, and Telegram orchestration lives in
> `telegram-bot/omega-tg-bot.ts`. See
> [PROVIDER-COMPATIBILITY.md](PROVIDER-COMPATIBILITY.md) for the supported
> contract.

**Research date:** 2026-05-28
**Claude Code reference version:** v2.1.144+ (as documented at https://code.claude.com/docs/en)
**Current OmegaOS surfaces:** `crates/omega-core/src/agents.rs`, `crates/omega-gateway/src/chat_driver.rs`, `telegram-bot/omega-tg-bot.ts`

---

## 1. Full CLI feature inventory

Sub-commands ([cli-reference](https://code.claude.com/docs/en/cli-reference#cli-commands)):

| Sub-command | What it does | OmegaOS today | Integration |
| --- | --- | --- | --- |
| `claude` (interactive) | Start TUI session | Used inside rmux panes for workers | already |
| `claude "query"` | Interactive with seed prompt | not used | yes — drop seed prompt into pane on Oracle spawn instead of `SendKeys` |
| `claude -p "query"` | One-shot SDK call | `claude_stream.rs` uses streaming variant | already |
| `claude -c` / `claude -r <id>` | Continue / resume by ID-or-name | not used | yes — replaces our "respawn worker with --continue" pattern with name-resolved resume |
| `claude --bg "task"` | Background agent, returns session id | not used | **YES (high-value)** — could replace many rmux ephemeral worker panes for short tasks. Trade-off in §2. |
| `claude agents` / `claude agents --json` | Background-session inventory | not used | yes — could mirror into OmegaOS session list |
| `claude attach <id>` / `claude logs <id>` / `claude stop` / `claude respawn` / `claude rm` | Manage background sessions | not used | yes — if we adopt `--bg`, these are the management primitives |
| `claude daemon status` | Supervisor health | not used | yes — surface in `omega doctor` |
| `claude auth login --console` / `--sso` / `setup-token` | Auth + long-lived token | install.sh runs `claude auth login` interactively | yes — `setup-token` for CI / scripted oracle bootstraps |
| `claude mcp` | MCP server config | not used (we have no MCP servers) | enables channels (§2) |
| `claude plugin` | Plugin install / list | not used | optional |
| `claude project purge` | Wipe local Claude state per project | not used | nice for `omega project reset` |
| `claude ultrareview` | One-shot PR/branch review, JSON | not used | yes — could power an `omega review` shortcut |
| `claude remote-control` | Server mode driven by claude.ai/app | not used | **likely not desired** — conflicts with our own bridge |
| `claude --teleport` | Resume claude.ai web session locally | not used | low priority |

Flags (selected — all 60+ documented in cli-reference#cli-flags):

| Flag | OmegaOS today | Integration |
| --- | --- | --- |
| `--print` / `--output-format stream-json` / `--input-format stream-json` / `--verbose` | already used in `claude_stream.rs` | already |
| `--append-system-prompt-file <path>` | used for AISB + workers | already |
| `--system-prompt-file` (replaces default) | not used | yes — for non-coding bridges (Telegram router, dispatcher) where Claude's default coding prompt is wrong |
| `--dangerously-skip-permissions` | used everywhere | already |
| `--bare` | not used | **YES** — for `claude_stream.rs` if we ever want faster cold-start ("skip auto-discovery of hooks, skills, plugins, MCP servers, auto memory, CLAUDE.md"). Trade-off: loses everything in `~/.claude/`. |
| `--agents '<json>'` | not used | **YES** — define worker personas inline at dispatch time, no temp file. Replaces our `--append-system-prompt-file` dance for ephemeral workers. |
| `--agent <name>` | not used | yes — pair with `.claude/agents/*.md` checked into the OmegaOS repo |
| `--mcp-config` / `--strict-mcp-config` | not used | yes — required for channels (§2) |
| `--channels plugin:<n>@<m>` | not used | **YES (§2 priority)** |
| `--dangerously-load-development-channels server:<n>` | not used | yes — for dev/testing our own channels before publishing |
| `--allowedTools` / `--disallowedTools` / `--tools` | not used | yes — tighten worker tool surfaces (e.g., audit workers: read-only) |
| `--permission-mode plan\|auto\|acceptEdits\|bypassPermissions` | only bypass used | yes — `plan` mode for `omega plan`, `auto` for low-risk dispatch |
| `--permission-prompt-tool <mcp_tool>` | not used | yes — could route permission relay through Telegram (see §2) |
| `--max-turns N` | not used | yes — bound runaway workers |
| `--max-budget-usd N` | not used | yes — cost cap (rule R-28) |
| `--json-schema '<schema>'` | not used | yes — for structured oracle reports (`.done.json` validation) |
| `--include-hook-events` / `--include-partial-messages` | not used | yes — Telegram bridge could stream partials |
| `--init-only` / `--init` / `--maintenance` | not used | yes — `omega bootstrap` invokes session-start hooks without burning a turn |
| `--from-pr <n>` / `--worktree` / `--tmux` | not used | conflicts with rmux — see §5 |
| `--name` / `--session-id <uuid>` | not used | yes — deterministic worker names instead of our ad-hoc `${PROJECT}-worker-${X}-${Y}` |
| `--fork-session` | not used | yes — for "branch the AISB master" workflows |
| `--setting-sources user,project,local` | not used | yes — lock down which `.claude/settings.json` layers apply |
| `--effort low\|medium\|high\|xhigh\|max` | not used | yes — match to MEDIUM/COMPLEX/EPIC classification |
| `--fallback-model` | not used | yes — survive model deprecations in long missions |
| `--exclude-dynamic-system-prompt-sections` | not used | yes — prompt-cache reuse across project oracles |
| `--no-session-persistence` | not used | yes — for stateless one-shot helpers (intent classifier, etc.) |
| `--replay-user-messages` | not used | yes — Telegram bridge could ack inbound messages reliably |

---

## 2. Channels (priority — the user asked)

**What they are** ([channels-reference](https://code.claude.com/docs/en/channels-reference)): a *channel* is a local MCP server that pushes `notifications/claude/channel` into a Claude Code session over stdio. Each notification surfaces inside the model's context as a `<channel source="..." key="val">body</channel>` tag. Two-way channels expose an MCP `reply` tool that Claude can call; permission relay (`claude/channel/permission`) forwards tool-approval dialogs to the channel. Research preview, requires Claude Code v2.1.80+, runs behind `--dangerously-load-development-channels` until on the official allowlist.

**Why this matters for OmegaOS:** today our Telegram bridge polls Bot API in Rust, then injects text into AISB Master via `session::send_text`. That's an out-of-band path — the bridge talks *to the terminal*, the model reads the terminal. Channels invert this: external events arrive directly in the model's context as tagged events, and Claude can reply via tool calls. We get:

1. **Structured routing instead of text-stuffing.** Today `telegram_bridge.rs` carefully formats Markdown, escapes Telegram special chars, and prepends `<<<TG:>>>` sentinels so the model knows what's user input vs. terminal noise. A channel delivers `<channel source="telegram" chat_id="123" sender_id="42" topic="13036">message text</channel>`. The model parses tags natively; the bridge stops being a string-formatter.

2. **No more send-keys race conditions.** `session.send_text_raw` pastes into a PTY; if the worker is mid-tool-use the input lands at the wrong prompt. Channels deliver into the model's *context window* on the next turn boundary — no PTY timing.

3. **Permission relay → mobile approval.** Claude Code already supports `notifications/claude/channel/permission_request` (v2.1.81+). OmegaOS could let workers run with default permissions while the user holds the kill-switch on Telegram. This is strictly better than `--dangerously-skip-permissions` for ambiguous ops.

4. **Workers report progress via channels.** A worker channel exposes a `progress(percent, note)` tool. AISB master subscribes to that channel and surfaces a unified progress feed — replacing our `worker-mark-done.sh` polling.

**Concrete fits:**

| OmegaOS surface today | Channel-based replacement |
| --- | --- |
| `telegram_bridge.rs` polling + `<<<TG:>>>` sentinels + `send_text_raw` | `omega-telegram-channel` Rust binary speaking MCP-stdio. Bun ref impl exists ([fakechat](https://github.com/anthropics/claude-plugins-official/tree/main/external_plugins/fakechat)); we'd build the Rust twin. |
| Oracle → Worker progress (currently file-watching `~/.aisb/state/*.done.json`) | Workers expose `progress` channel; Oracle's claude session subscribes via `--channels server:omega-progress` |
| Permission gating for risky ops (currently we go `--dangerously-skip-permissions` everywhere) | `claude/channel/permission` relays prompts to AISB master / Telegram |
| `claude_stream.rs` Telegram bridge → AISB master | AISB master launches with `--channels server:omega-telegram` instead of being driven by an out-of-band pipe |

**Caveat (researcher-not-sycophant):** channels are explicitly *research preview*. The MCP SDK is Node/Bun-native; the only documented runtimes are Bun/Node/Deno. Writing the channel server in Rust requires implementing the JSON-RPC + stdio framing of `@modelcontextprotocol/sdk` ourselves. The protocol is small (about 6 message types we'd touch) — feasible — but this is the one place RUST-BUN-DEFAULT *might* legitimately bend to Bun for the channel server while keeping AISB/Oracle/Worker logic in Rust. Decide explicitly.

---

## 3. Commands / Slash commands

Custom commands and skills have merged ([skills doc](https://code.claude.com/docs/en/slash-commands)): `.claude/commands/foo.md` and `.claude/skills/foo/SKILL.md` both create `/foo`. We already publish 120+ skills under `~/.claude/commands/` and `~/.claude/skills/` for the global system. OmegaOS *as a checked-in repo* can ship a `.claude/commands/` and `.claude/agents/` directory that travels with the install.

**Built-in slash commands** ([commands ref](https://code.claude.com/docs/en/commands)): `/help`, `/compact`, `/clear`, `/model`, `/config`, `/resume`, `/rename`, `/add-dir`, `/mcp`, `/plugin`, `/hooks`, `/agents`, `/skill`, `/debug`, `/code-review`.

**OmegaOS-specific slash commands to ship** (in `omega/.claude/commands/`):

- `/omega-status` — runtime: `omega session list --json`, summarize state
- `/omega-dispatch <project> <task>` — wrapper over `omega dispatch` so users in any session can hand work to an Oracle
- `/omega-promote` — promote current worker to Oracle (long-lived)
- `/omega-channel-publish` — push current session's progress to a named channel
- `/omega-rmux` — open the rmux session manager TUI in a child PTY
- `/omega-rules` — list which rules from `~/.claude/rules/` are active in this session
- `/omega-cost` — print `--max-budget-usd` remaining and turn count

These are 30-50 line markdown files with YAML frontmatter; trivial to ship.

---

## 4. Concrete integration plan (top 10, prioritized)

| # | Item | Why | Where | Effort | Deps |
| --- | --- | --- | --- | --- | --- |
| 1 | `--agents '<json>'` for worker dispatch | Replaces our `--append-system-prompt-file <tempfile>` pattern. One arg, no fs juggling. Cuts ~80 LOC in `dispatch-to-session.sh`. | `crates/omega-cli/src/dispatch.rs` (new) + `crates/omega-cli/src/claude_stream.rs::spawn_args` | S | none |
| 2 | `--tools` / `--allowedTools` per agent role | Audit workers run read-only; doc workers run no-Bash; Oracle keeps full. Tightens the blast radius without depending on the permission system. | same as #1, plus `crates/omega-core/src/agent.rs` role enum | S | none |
| 3 | `--max-turns` + `--max-budget-usd` on every worker spawn | Rule R-28 (cost tracking). Currently we have no hard cap — a runaway worker spends silently. | `crates/omega-cli/src/dispatch.rs` | S | none |
| 4 | `--name` / `--session-id` (UUID) for sessions | Deterministic resume by name. Today we rely on rmux session names + `claude --continue`, which is fragile when the cwd changes. | `crates/omega-core/src/session.rs::SessionId` | S | none |
| 5 | OmegaOS channel server (Telegram replacement) | §2 above. Biggest architectural win — kills the `<<<TG:>>>` sentinel hack and the send-keys race. | new crate `crates/omega-channel-telegram/` OR bun script `tools/channels/telegram.ts` | L | Decide Rust-vs-Bun for channel server |
| 6 | OmegaOS channel server for progress / done.json | Workers push progress as channel events instead of file-watching. AISB master subscribes. Eliminates `~/.aisb/state/*.done.json` polling cron. | new `crates/omega-channel-progress/` (or `.ts`) + `crates/omega-cli/src/aisb_master.rs` subscribes | M | #5 unblocks (same protocol impl) |
| 7 | Permission relay through Telegram channel | Stop spraying `--dangerously-skip-permissions` everywhere. Workers run in `--permission-mode auto` with relay back to Telegram for prompts the model can't auto-handle. | Same channel server from #5 declares `claude/channel/permission: {}` | M | #5 |
| 8 | `--bare` for `claude_stream.rs` cold start | Telegram-bridge persistent subprocess doesn't need hooks/skills/plugins/MCP — it just routes intents. Faster startup + smaller context. | `crates/omega-cli/src/claude_stream.rs::spawn_command` | S | none (just verify nothing in claude_stream relies on auto-loaded skills) |
| 9 | `--json-schema` for `.done.json` | Today workers write `.done.json` from prose. With `--json-schema`, Claude is forced to emit the schema directly. Removes our JSON-cleanup post-processing. | `crates/omega-cli/src/worker.rs::mark_done` (replace shell `worker-mark-done.sh` with `claude -p --json-schema`) | M | none |
| 10 | `.claude/commands/omega-*.md` checked into the repo | Ships OmegaOS-specific slash commands with the install — `./install.sh` symlinks them into `~/.claude/commands/`. Users get `/omega-status` etc. in any session, with zero global pollution. | new dir `omega/.claude/commands/` | S | none |

Notes on what I deliberately left off:
- `--worktree` / `--tmux` — Claude Code's own tmux mode conflicts with rmux. Skip.
- `--remote-control` — claude.ai-driven remote control duplicates our Telegram path. Skip.
- `--from-pr` — useful eventually for code review pipelines, but not core OS infra. Defer.

---

## 5. Anti-patterns / things to AVOID

1. **Do not use `claude --tmux` or `claude -w --tmux`.** It spawns a real tmux session next to rmux; the two multiplexers fight for PTY ownership. rmux is our session layer; Claude Code's tmux integration is for users who don't have one.
2. **Do not use `claude remote-control` for the Telegram path.** It binds Claude Code's own server to claude.ai; OmegaOS owns its remote-control surface (Telegram + future web UI). Two control planes = ambiguity.
3. **Do not blanket-replace `send_text` with channels.** Channels are research-preview, allowlist-gated, and currently require Bun/Node for the SDK. Keep the rmux PTY path as the fallback (and for the rmux-native TUI menus that aren't Claude conversations).
4. **Do not write the channel server in Python.** It would be the only Python in the repo (RUST-BUN-DEFAULT). Bun is acceptable per the rule's exception clause (browser/DOM-ish JS ecosystem); a Rust implementation of the small subset of MCP we need is the cleanest answer if we have appetite.
5. **Do not depend on `--include-partial-messages` for AISB Master text streaming.** It's documented as `--print` mode only. Our streaming bridge uses long-lived `-p --input-format stream-json` which works, but watch for behavior drift when partials are enabled.
6. **Do not over-rotate to `--bg` background sessions.** They run under the supervisor daemon, which is *Claude Code's own session manager*. We already have rmux + AISB. Use `--bg` only for genuinely background-only tasks (e.g., long PR-review chewing) where rmux visibility doesn't matter.
7. **Don't store `ANTHROPIC_API_KEY` in `.claude/settings.json` checked into the repo.** Managed settings precedence is highest; a committed key leaks. User-scope only.

---

## 6. Open questions for the human

1. **Channel server runtime: Rust or Bun?** The MCP SDK is Node/Bun-only. Writing the channel server in Rust means re-implementing the JSON-RPC stdio framing for `claude/channel`, `claude/channel/permission`, and the `tools/list`+`tools/call` subset. ~600-1000 LOC, very stable protocol. Bun version is ~200 LOC using the official SDK but adds Bun to the runtime surface. **Pick one and document the exception.**
2. **Channels research-preview risk.** Anthropic explicitly calls this preview + allowlist-gated. Are we OK shipping `--dangerously-load-development-channels` in OmegaOS install for now, or do we apply for official-marketplace listing first?
3. **`--bg` vs rmux for workers.** The `claude --bg` supervisor and `rmux` are two session managers. Right now we use rmux exclusively. Do we want `--bg` as an alternative for headless workloads (no PTY visibility, supervisor-managed), or do we standardize on rmux everywhere?
4. **`.claude/settings.json` location for OmegaOS.** The repo could ship `omega/.claude/settings.json` (project scope) defining hooks, permissions, channel allowlist, etc. But OmegaOS *is the system*, not a project — does it own `~/.claude/settings.json` (user scope) too at install time? Risk: overwriting the user's existing settings.
5. **Effort levels per role.** `--effort low/medium/high/xhigh/max` — what's our default mapping? My guess: workers=medium, oracles=high, AISB Master=high, audits=max. Confirm.
6. **`--json-schema` and existing `.done.json`.** Our `.done.json` schema is defined in `~/.claude/rules/47-oracle-end-of-work.md`. Migrating to `--json-schema` means encoding that schema as JSON Schema. Worth it, but who owns the canonical schema file?
7. **Hooks**: should OmegaOS install register `SessionStart`/`SessionEnd` hooks user-globally so every Claude session on the VPS gets OmegaOS context loaded? Powerful, but invasive.
8. **`claude install` pinning.** Should `./install.sh` run `claude install 2.1.144` (pin) or `claude install stable`? Channels need ≥2.1.80, permission relay ≥2.1.81. Pinning vs. floating.

---

*End of report.*
