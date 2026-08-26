# Omega App — native macOS + iOS client for OmegaOS

**Date:** 2026-08-10
**Status:** Design approved by operator (sections 1-5 validated in session)
**Next step:** implementation plan (writing-plans), then phased build

## Goal

Turn OmegaOS from a terminal-and-Telegram system into a real product app: a native
macOS + iOS client with the UX of Claude Desktop / ChatGPT, where every user chats
with agents running on their OWN machines (VPS or local box), and manages the whole
OmegaOS world from the app: oracles and missions, sessions, rules, skills, MCP
servers, knowledge memory, and multi-account Claude/Codex logins. The app replaces
Telegram as the default control surface (Telegram stays as an optional channel).

Product-for-everyone from day 1: any `npx omega-os` user pairs the app with their
box. Identity via Clerk (Google connect); box connectivity via Tailscale first,
SSH tunnel fallback.

## Non-goals (v1-v3)

- No Android (revisit after iOS ships).
- No cloud-hosted agent execution: agents only run on user-owned boxes.
- No storage of session content, files, or agent credentials in the Agentik cloud.
- No replacement of the terminal workflow for power users; the app is additive.

## Architecture: three tiers

```
┌─────────────────────────────┐
│  APPS (Omega App)           │  Electron macOS + Expo iOS
│  shared React UI            │  chat, missions, skills, settings
└──────────┬──────────────────┘
           │ 1. identity → Clerk (Google connect)
           │ 2. box data → direct WebSocket
┌──────────▼──────────────────┐
│  AGENTIK CLOUD (thin)       │  Clerk + Convex
│  - user accounts            │  - registry: which boxes belong to whom
│  - push relay (APNs)        │  - stores NO session content, ever
└──────────┬──────────────────┘
           │ outbound registration from each box
┌──────────▼──────────────────┐
│  EACH BOX (VPS or Mac)      │  omega-gateway (Rust daemon, OmegaOS crate)
│  rmux sessions, oracles,    │  authenticated WebSocket/HTTP API
│  rules, skills, MCP, creds  │  reached directly via Tailscale / SSH tunnel
└─────────────────────────────┘
```

Principle: **the cloud is a directory and a postman, never a warehouse.** The app
authenticates with Clerk, reads its box registry from Convex, then talks directly
to each box's `omega-gateway`. Conversations, files, and credentials never transit
the cloud. The cloud only maps Clerk user → paired boxes/devices and relays
minimal, encrypted push events to APNs.

## Component 1: `omega-gateway` (Rust daemon on every box)

New crate `crates/omega-gateway` in the OmegaOS workspace. Installed by
`install.sh`, runs as a systemd service (launchd on macOS boxes). Binds
tailnet/loopback only by default. Every request carries a per-device bearer token
issued at pairing.

### Capabilities (WebSocket typed protocol + REST)

| Domain | What the gateway exposes |
|---|---|
| Chat | A conversation in the app = a headless Claude/Codex session on the box; turns streamed live. This is the "my own Claude/ChatGPT" experience: same UX, but the agent runs on the user's box with their rules, skills, and memory. |
| Sessions | List rmux sessions; live mirror via rendered-pane snapshots (the proven `omega stream` mechanism); full terminal attach (PTY over WebSocket); spawn/kill. |
| Missions | Dispatch to oracles; live progress ledger (`oracle-*.progress.json`), done/blocked signals; R-DESTRUCT escalations surfaced as in-app approvals ("agent wants to drop table X: Approve / Deny") with signed EscalationRecord resolution. |
| Files | Scoped project-tree browse, read/write, upload (supersedes the DEPOSIT drop box). |
| Skills | `omega-skills` catalog + RAG search + invocation. |
| MCP | Configured servers per project, status. |
| Rules | Compiled doctrine (Laws/Rules) read access. |
| Accounts | Claude/Codex credential profiles: list metadata, add via relayed device-auth flow, assign defaults (see Component 3). |
| Memory | `omega-mem` queries; graphify knowledge graphs. |
| Push | Outbound minimal encrypted events to the cloud relay (mission done, question pending, alert) for APNs delivery. |

### Pairing

`omega app pair` prints a QR code. The app scans it, receives a device token, and
the box registers itself (outbound) in the Convex registry bound to the user's
Clerk id. Revocable per box and per device. Multiple boxes per user is first-class
(several VPS + a local machine).

## Component 2: the app (Electron macOS + Expo iOS, shared React core)

Same stack family as Claude Desktop / ChatGPT desktop (operator's explicit
choice). Monorepo `agentik-os/omega-app`:

```
packages/core        # shared React UI + state (the app itself)
packages/protocol    # TS types GENERATED from the gateway's Rust schemas (single source of truth)
apps/desktop         # Electron shell (macOS first)
apps/mobile          # Expo / React Native (iOS first, EAS cloud builds from Linux)
```

### `packages/core/src/` layout

```
app/          # shell, navigation, panel registry, DI
chat/         # Claude/ChatGPT-style conversations (the heart)
fleet/        # user's boxes: QR pairing, health (omega doctor), usage
missions/     # live oracles, plans, progress, destructive-op approvals
sessions/     # session list + live mirror + attach
terminal/     # full terminal (xterm.js), mobile extra-keys bar
files/        # file tree, transfers
editor/       # CodeMirror, syntax detection
md-preview/   # markdown rendering (reports, specs)
skills/       # catalog + RAG + invoke
mcp/          # MCP servers per project
rules/        # doctrine viewer (Laws/Rules)
agents/       # multi-account Claude/Codex manager, quotas, defaults
memory/       # omega-mem + knowledge graph views
inbox/        # alerts + pending questions (replaces Telegram topics)
ssh/          # SSH tunnel fallback, key management (Secure Enclave/Keychain)
workspace/    # multi-box context switching
gestures/     # pinch zoom, panel swipes (mobile)
themes/       # dark/light, single signature accent (Monogram taste)
policies/     # DI-based feature gating (free/pro later), no hard limits in core
plugins/      # panel/tab extension API
lib/          # shared constants, styles, utils
```

Navigation: macOS uses the Stax grammar (panels opening right, one action zone,
one way back); iOS uses tabs (Chat / Missions / Inbox / Fleet / More).

## Component 3: multi-account Claude/Codex + security

- **Agent credentials never leave the box.** Each profile lives in
  `~/.omega/accounts/<profile>/`; the gateway launches sessions with
  `CLAUDE_CONFIG_DIR` / `CODEX_HOME` pointed at the right profile. The app sees
  metadata only (email, plan, quota state).
- **Adding an account from the app:** the gateway runs `claude /login` or
  `codex login --device-auth` and relays the device-auth flow; the user completes
  it in their phone browser.
- **Assignment:** default account per box, overridable per project and per
  conversation (chat menu: agent, account, model).
- **App identity:** Clerk with Google connect, gating the cloud registry and push.
- **Network:** Tailscale preferred; otherwise an SSH tunnel with a key generated
  in the Secure Enclave (never exported). Cloud never sees content or credentials.
- **Approvals:** destructive operations (R-DESTRUCT) require an explicit in-app
  approval signed with the device identity; unattended graphs produce
  EscalationRecords delivered as push + inbox items.

## Phasing

- **V1 — Telegram parity (the unlock):** gateway (chat, sessions, dispatch,
  progress, alerts, pairing) + app with Chat, Missions, Inbox, Fleet, APNs push.
  When V1 runs, Telegram becomes optional.
- **V2 — full cockpit:** terminal, files/editor/md-preview, skills, MCP, rules,
  multi-account manager.
- **V3 — public product:** memory/graph views, plugins, policies (free/pro),
  App Store release, onboarding `npx omega-os` → scan QR.

## Build, test, distribution

- iOS built via EAS cloud builds (works from the Linux VPS, no Mac required);
  TestFlight internal first. Electron shipped as a notarized DMG.
- Gateway: cargo unit + integration tests; protocol schema snapshot tests keep
  `packages/protocol` in lockstep. Desktop: Playwright. Development follows the
  normal OmegaOS pipeline (oracles, workers, tracked plans).
- The gateway ships inside OmegaOS (install.sh + verify-install), so a fresh
  clone reproduces it (L0).

## Key decisions and rejected alternatives

1. **Stack B (Electron + Expo) over Tauri 2 universal** — operator chose the
   Claude Desktop/ChatGPT stack; most mature mobile path, EAS builds from Linux,
   first-class Clerk + push. Cost: protocol reimplemented in TS (mitigated by
   generating types from Rust schemas). Tauri (Rust reuse) and pure SwiftUI
   (Linux fleet cannot build it) rejected.
2. **Hybrid connectivity over pure-direct or pure-relay** — direct
   Tailscale/SSH for data (privacy, zero cloud cost), thin cloud only for
   identity, box registry, and APNs push (iOS cannot do reliable push without it).
3. **Credentials stay on the box** — the multi-account feature is implemented
   server-side as profile directories, not by syncing tokens to the app or cloud.
4. **One gateway per box, many boxes per user** — matches the real fleet
   (Station VPS, gareth, local Macs) instead of assuming a single server.
