# Open Design — the self-hosted design engine

Open Design (vendored from [nexu-io/open-design](https://github.com/nexu-io/open-design), Apache-2.0,
pinned in [`tools/open-design/README.md`](../tools/open-design/README.md)) turns a coding agent into
a design engine: prototypes, dashboards, landing pages, decks, images and video from a brand
contract (`DESIGN.md`), 150+ built-in design systems. It runs as a Docker daemon and is served
tailnet-only so you SEE the build in a browser.

## Quick access

```
omega-design open        # a card: the view link + how-to + detected agents
omega-design url         # just the link
omega-design status      # container + serve + health
omega-design projects    # OmegaOS projects you can open as a working directory
omega-design agents      # which local CLIs Open Design detected
```

**View link:** Tailscale up → `https://<tailnet-host>:7456`; no Tailscale → `http://localhost:7456`.
(This box: `https://station.tail64d114.ts.net:7456` — needs Tailscale on your device.)

## Updates & existing installs

A **fresh** OmegaOS install does NOT set up the daemon (heavy Docker dep — opt in once with the
installer below). But **once it is installed**, every `omega update` re-runs the idempotent
installer (install.sh Phase 6.916) so the deployment tracks `main`: latest override (host-net,
auth/git/projects mounts), MCP re-wired for claude/codex, projects re-imported, tailnet re-served.
Opt out with `OMEGA_SKIP_OD_REFRESH=1`. So on an existing install: `omega update` and it self-heals;
on a new one: run the installer once, then updates keep it current.

## Getting the link (Tailscale, or SSH when there is none)

```
omega-design url     # the view URL
omega-design open    # a card: URL + how-to + agents
omega-design ssh     # the SSH tunnel command when there is no Tailscale
```

- **Tailscale present** → `https://<tailnet-host>:7456` (open it directly; needs Tailscale on your device).
- **No Tailscale** → the daemon is loopback-only, so tunnel in with the SSH key you already use:
  `ssh -N -L 7456:127.0.0.1:7456 [-p <port>] <user>@<host>` then open `http://localhost:7456`.
  `omega-design ssh` prints the exact command (auto-filled from `$SSH_CONNECTION`; override with
  `OMEGA_SSH_HOST` / `OMEGA_SSH_PORT` / `OMEGA_SSH_USER`).

## Install (opt-in — heavy Docker dependency, not auto-run by install.sh)

```
bash tools/open-design/install-open-design.sh
```

It clones (pinned), builds a CLI-baked image, wires auth + git + projects, starts the daemon,
serves it tailnet-only, and installs `omega-design`. Opt-outs: `OMEGA_SKIP_OD_LOCALCLI=1` (use the
UI's BYOK instead of the local CLIs), `OMEGA_SKIP_TS_SERVE=1`, `OMEGA_PROJECTS_ROOT=<dir>`.

## Local CLI mode = your subscription (not BYOK)

Open Design delegates the agent loop to a coding-agent CLI on the **daemon's** PATH. The daemon is a
Docker container, so it can't see the host's `claude`/`codex` — out of the box it finds none and errors
`vela binary not found`. OmegaOS fixes this by:

- baking `claude`+`codex` into an extension image `od-omega` (`deploy/Dockerfile.omega-agents`),
- running the container as the **host uid** so it reads the operator's auth and writes files with
  correct ownership,
- mounting a resolved copy of the operator's auth at `~/.omega/open-design-agent-home`
  (never the host `~/.claude` symlink), via `deploy/docker-compose.omega.yml`.

Result: the "Local CLI" picker detects **claude** and **codex** and runs on the operator's
subscription. `omega-design agents` confirms it.

## Redesign an existing project (refonte)

The served web UI cannot browse **server** folders (a browser folder-picker only sees the operator's
own device, not the daemon's filesystem), so you register a project from the CLI instead:

```
omega-design projects            # list your OmegaOS project paths (~/Station)
omega-design import <name|path>  # register it into Open Design -> it appears in the UI project list
```

Then open the link, pick the project, and claude/codex (your subscription) read/write the real
project files at `~/Station/...` (mounted read-write). Git is wired
(`~/.omega/secrets/agentik-os.git-credentials`) so it can push.

**There is no "connect GitHub" step** — Open Design has no clone-from-GitHub feature (the `github`
you see in the UI is a *design system* named github). Your projects are already local git repos;
the mount + git creds are what let a refonte push.


## Design from a chat (MCP)

The daemon's MCP server is wired into **Claude and Codex** (Claude: user-scoped; Codex:
`~/.codex/config.toml` — it does not read Claude's config). Reached via `docker exec` into the
container. So from any OmegaOS chat you can: `list_projects`, `create_project`, `list_skills`
(design research over 162 skills / 151 design systems — see
[references/od-catalog.md](../skills/open-design/references/od-catalog.md)), and `start_run` to
commission a design, then `get_run`. Re-run the installer or `omega-design` to re-wire; opt out with
`OMEGA_SKIP_OD_MCP=1`.

## Connectors & keys (Composio, BYOK) — loopback-only

Open Design gates sensitive settings (Composio API key, connector/provider keys) to **loopback
only** — a deliberate security boundary: only the local machine may change them, never a remote
tailnet client. So the **tailnet UI cannot save a key** (it returns 403 by design). Set them
**server-side** instead:

```
omega-design composio <COMPOSIO_API_KEY>   # clears with: omega-design composio -
```

(OmegaOS runs the daemon with host networking so its own loopback works behind Docker — without it,
even a localhost request was seen as non-loopback through the bridge NAT and every management write
failed.)

## Marketing machine

Build the visual system and viewable creatives (landing pages, decks, ad mocks) here; the operator
reviews them at the link. Publishing still goes through Zernio (R-ZERNIO). Open Design BUILDS + shows;
Zernio DISTRIBUTES.

## Security posture

Container bound to `127.0.0.1:7456`; exposure is `tailscale serve` (tailnet-only, no Funnel). The
tailnet is the trusted auth layer, so daemon token-auth is disabled behind it. The API token still
lives in `~/.omega/secrets/open-design.env`. Running as the host uid with `~/Station` mounted
read-write is deliberate (so a refonte can write) — keep it tailnet-only; never add Funnel without
re-enabling `OD_API_TOKEN` and dropping the projects mount.
