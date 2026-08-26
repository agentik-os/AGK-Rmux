# Marketing Machine — BACKLOG (deferred from v1)

v1 of the marketing-machine program shipped the anti-forgetting registry
(`capabilities.toml`) + the `omega marketing {capabilities,status,next,doctor}`
subcommands + the scaffolder fix (P0-1). The following are intentionally NOT
done in v1 and remain open. Source of truth for scope: `AUDIT.md` §3.

## Deferred — daily-engine unification (P0-2 / P0-4)

- **P0-2 — Unify the calendar schema + the daily engine.** One canonical
  `calendar-90d.json` (superset `days[].posts[].{slot,platform,pillar,mode,text,
  hashtags,visualPrompt,cta}`) and one `sent-log.json` state model. Port Site's
  `caio-daily.ts` off `queue.json`. The scaffolder now writes the canonical
  `calendar-90d.json` + a `daily-engine/sent-log.json` in this shape, so the data
  contract is in place — the runner is not.
  *Verify:* one engine binary runs both Verba and Site from `calendar-90d.json`.

- **P0-4 — Extract a portable `marketing-machine` runner** wired to
  `omega marketing run <project> [--publish|--dry-run]` (capabilities.toml `O7`,
  status `missing`). Today VD12 (`verba-daily.ts`) and VD13 (`caio-daily.ts`) are
  two per-project forks. Collapse them into config + a shared engine reading
  `calendar-90d.json` + `06-branding/tokens.json`.
  *Verify:* Verba's bespoke renderer becomes a config + brand assets, not a fork.

The `omega marketing run` command is deliberately NOT added to the CLI in v1
(only `capabilities`, `status`, `next`, `doctor`). Add it when the portable
engine exists — otherwise it would be a command with no engine behind it.

## Other open items (see AUDIT.md §3)

- P1-2 — the six `<Still>` archetypes as code (scaffolder writes the skeleton +
  `templates/stills/` today; the actual HTML/Remotion code is not built).
- P1-3 — the validation gate as code (safe-zone, WCAG contrast, token
  conformance, kill-list scan, hook/beat-grid). Only R-NODASH is coded today.
- P2-1 — portable voice/audio helper (ElevenLabs TTS + music ducking) as a lib.
- P2-2 — HeyGen avatar runner (capabilities.toml `U2` = missing).
- P2-3 — decide the legacy outreach engine (`~/Station/Marketing/engine/`).
- ~~P2-4 — `pattern-ledger.md` metrics loop.~~ **DONE 2026-07-26** — `bin/reels ledger`
  (capabilities.toml `G17`) joins your measured reels against the niche leaderboard and
  emits a verdict per hook pattern. Method: `growth/IG-REEL-LOOP.md`.
- P3-1 — Reddit-native module in social-content.
- G7 — comment-to-DM funnel runner (capabilities.toml `G7` = missing).
