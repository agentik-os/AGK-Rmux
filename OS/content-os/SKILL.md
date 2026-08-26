---
name: content-os
description: >
  Operate the full content lifecycle from positioning and daily capture through story mining, research, writing, visual/audio/video production, platform-native packaging, publishing, community engagement and performance learning. Contains 38 specialist agents, 44 skills, 12 protocols and 10 schemas. Use for content strategy, daily capture, story mining, pillar assets, multi-platform cascades (Instagram, TikTok, YouTube, LinkedIn, X, newsletter, article), visual/video/sound briefs, editorial calendars, or performance review. Trigger words: content, capture, story mine, pillar, cascade, platform package, visual brief, video brief, content calendar; FR: contenu, capture du jour, mine d'histoires, cascade multi-plateforme, brief visuel, brief video, calendrier editorial.
---

# Content {OS}

Runtime-installed pack (2026-08-11), staged for the OmegaOS repo-level R-SKILLPUB integration by a concurrent session. This SKILL.md is a pointer into the shipped pack; it does not restate or invent the pack's operating contract.

## Load before operating

- [README.md](README.md) for purpose, operating loop, commands and main handoffs.
- [system/SYSTEM_PROMPT.md](system/SYSTEM_PROMPT.md) for the full operating contract.
- [system/PRINCIPLES.md](system/PRINCIPLES.md) and [system/BOUNDARIES.md](system/BOUNDARIES.md) for scope and limits.
- [system/ROUTER.md](system/ROUTER.md) for command/intent routing.
- [MANIFEST.json](MANIFEST.json) for the full inventory (agents, skills, protocols, schemas).
- [OMEGA_INTEGRATION.md](OMEGA_INTEGRATION.md) for registration ID, event types and cross-OS handoffs.
- `agents/*.md` for specialist agent definitions, `skills/*.md` for reusable skill procedures, `protocols/*.md` for multi-step operating protocols, `schemas/*.json` for the data model.

## Commands

| Command | Mode | Purpose |
| --- | --- | --- |
| `/content` | strategy | Open Content OS |
| `/content-gps` | strategy | Define positioning and content system |
| `/capture-day` | capture | Ingest the day as source material |
| `/story-mine` | mine | Find stories and insights |
| `/pillar` | create | Create a pillar asset |
| `/cascade` | cascade | Build a multi-platform waterfall |
| `/instagram` | platform | Create Instagram-native package |
| `/tiktok` | platform | Create TikTok-native package |
| `/youtube` | platform | Create YouTube package |
| `/linkedin` | platform | Create LinkedIn package |
| `/x` | platform | Create X package |
| `/newsletter` | platform | Create newsletter |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).
