# Open Design — vendored tool (design engine, self-hosted)

Upstream: **github.com/nexu-io/open-design** (Apache-2.0), pinned commit **`7d7c56a`**.
Docker image: `ghcr.io/nexu-io/od:latest` (daemon `apps/daemon` owns `/api/*`).

A local-first design workspace that turns coding agents into design engines: prototypes,
dashboards, decks, images and video from a brand contract (`DESIGN.md`), 150+ design systems.
OmegaOS runs it as a Docker daemon and serves it tailnet-only so the operator SEES the build.

## Boundary (same as ZernFlow / higgsfield / browser-use)

OmegaOS ships only this markdown + `install-open-design.sh` + the `omega-design` CLI + the
`open-design` skill. The 503M clone, the Docker pull, and the run are a **runtime opt-in** —
`install.sh` does NOT clone or start it. A live daemon is not runtime-verifiable without running
the installer.

## Install (opt-in)

```bash
bash tools/open-design/install-open-design.sh
```

It (1) shallow-clones upstream to `~/.omega/repos/open-design`, (2) writes `deploy/.env` with a
generated `OD_API_TOKEN` (mirrored to `~/.omega/secrets/open-design.env`, never the repo),
(3) `docker compose pull && up -d`, (4) `tailscale serve --https=7456` (tailnet-only, no Funnel),
(5) verifies `/api/health`. Opt out of the serve step with `OMEGA_SKIP_TS_SERVE=1`.

## Use

- `omega-design status | url | up | down | serve | systems | skills | templates | api <path>`
- Agent doctrine: the `open-design` skill (`~/.claude/skills/open-design`).
- View: `https://<tailnet-host>:7456`.

## Security posture

Container is `read_only`, `no-new-privileges`, `mem_limit 384m`, `pids_limit 256`, bound to
`127.0.0.1:7456`. Exposure is via `tailscale serve` (tailnet-only); the tailnet is the trusted
auth layer, so daemon token-auth is disabled behind it (`OPEN_DESIGN_DISABLE_API_AUTH=1`). Never
add Funnel (public) without re-enabling `OD_API_TOKEN`.

## Update the pin

Bump the commit here, re-run the installer (it fast-forwards the clone), then `docker compose pull`.

## Local CLI mode (subscription, not BYOK)

The daemon delegates generation to a coding-agent CLI on ITS PATH. In Docker that PATH has none, so the installer bakes `claude`+`codex` into an `od-omega` image (`Dockerfile.omega-agents`) and mounts a resolved copy of the operator's auth (`~/.omega/open-design-agent-home`, chown 1001) via `docker-compose.omega.yml`. Result: Open Design's agent picker detects claude/codex and runs on the operator's subscription. Opt out with `OMEGA_SKIP_OD_LOCALCLI=1` (then use the UI's BYOK). `omega-design agents` shows what's detected.
