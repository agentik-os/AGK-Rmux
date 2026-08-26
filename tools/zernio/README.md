# omega-zernio

Pure **Bun/TS**, **zero-dependency** CLI over the [Zernio](https://zernio.com) REST
API. Social publishing modeled as **one Zernio profile per OmegaOS project** —
the project→profile map is persisted at `~/.omega/zernio-profiles.json`.

- **Base:** `https://zernio.com/api/v1`
- **Auth:** `Authorization: Bearer $ZERNIO_API_KEY`
- **Key:** read from the env, else parsed at runtime from
  `~/.omega/secrets/integrations.env` (`ZERNIO_API_KEY=...`). The key is **never**
  printed, logged, or written to any repo file (R-ENV / L0).

## Install

```bash
bash tools/zernio/install-zernio.sh
# → copies cli.ts into ~/.omega/skills/zernio/ and writes the launcher
#   ~/.local/bin/omega-zernio (exec bun ~/.omega/skills/zernio/cli.ts "$@")
```

Shipped automatically by the OmegaOS `install.sh`. Re-run the installer after each
`cli.ts` edit — the launcher runs the **deployed** copy, not the worktree.

## Commands

```bash
omega-zernio status                            # key present? API reachable? counts
omega-zernio profiles                          # list profiles + mapped projects
omega-zernio connect <project> <platform>      # resolve/create profile → print hosted authUrl
omega-zernio accounts [project]                # list connected accounts (optionally per project)
omega-zernio post <project> --text "…" --platforms a,b,c [--media url|path] [--dry-run] [--schedule ISO]
```

Global flags: `--json` (machine-readable, used by the Telegram bot), `--help`.

### Project → profile resolution

1. Look up `<project>` in `~/.omega/zernio-profiles.json`.
2. Else case-insensitive **normalized** match against `/v1/profiles`
   (`agentik-os` → `agentikos` matches `Agentik OS`).
3. Else: `connect` **creates** the profile; every other command errors with the
   exact `connect` command to run.

### Connecting accounts (per project, per platform)

```bash
omega-zernio connect gta6 instagram
omega-zernio connect gta6 tiktok
omega-zernio connect agentik-os tiktok
omega-zernio connect agentik-os linkedin
```

Open the printed `authUrl` to authorize; the account attaches to that project's
profile.

### Posting

```bash
# Dry run — validates via /v1/tools/validate/post and prints the would-send
# /v1/posts body WITHOUT publishing:
omega-zernio post agentik-os --text "hello" --platforms tiktok --dry-run

# Publish now:
omega-zernio post agentik-os --text "hello" --platforms tiktok,twitter

# With media (public URL, or a local file that is presign-uploaded):
omega-zernio post agentik-os --text "launch" --platforms instagram --media https://x.com/a.jpg
omega-zernio post agentik-os --text "launch" --platforms instagram --media ./shot.png

# Schedule (ISO 8601):
omega-zernio post agentik-os --text "later" --platforms tiktok --schedule 2026-07-01T09:00:00Z
```

> **Validator media blind spot:** `/v1/tools/validate/post` only accepts
> `{content, platforms:[{platform}]}` — it never sees `--media`, so for
> media-required platforms (e.g. TikTok) it reports `media required` even when
> your post will include media. When you pass `--media`, `--dry-run` demotes that
> specific error to an informational **note** and keeps `valid: true`; the
> would-send `/v1/posts` body already carries `mediaItems`. Other real errors
> still block.

Media type is inferred from the extension: `jpg/jpeg/png/webp → image`,
`gif → gif`, `mp4/mov/webm → video`, `pdf → document`. Local files are uploaded
via `/v1/media/presign`; if the presign shape can't be resolved, the CLI degrades
with a clear error telling you to pass a public URL instead.

Platforms: `facebook, instagram, linkedin, twitter, tiktok, youtube, threads,
reddit, pinterest, bluesky, googlebusiness, telegram, snapchat, discord,
whatsapp`.
