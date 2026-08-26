#!/usr/bin/env bash
# OmegaOS Marketing Machine — per-project folder scaffolder (SSOT, reusable).
# Creates an identical marketing/ tree in every project listed in projects.tsv.
# Idempotent + NON-DESTRUCTIVE: writes a file only if it does not already exist.
# Real content is filled by the marketing machine (see README.md).
#
# Usage:
#   tools/marketing-machine/scaffold.sh                 # all projects in projects.tsv
#   tools/marketing-machine/scaffold.sh <dir> <name> <slug>   # one ad-hoc project
#   STATION=/path tools/marketing-machine/scaffold.sh   # override ~/Station root
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STATION="${STATION:-$HOME/Station}"
MANIFEST="${MANIFEST:-$HERE/projects.tsv}"
STAMP="$(date +%Y-%m-%d)"

w() { local path="$1"; shift; mkdir -p "$(dirname "$path")"
  if [ -e "$path" ]; then echo "  skip (exists): ${path#"$ROOT/"}"; return 0; fi
  cat > "$path"; echo "  +            : ${path#"$ROOT/"}"; }

scaffold() {
  ROOT="$1"; NAME="$2"; SLUG="$3"; local M="$ROOT/marketing"
  echo "### $NAME  ($ROOT)"
  [ -d "$ROOT" ] || { echo "  !! project dir missing, skipping"; return 0; }
  mkdir -p "$M"/{00-context/swipe,01-strategy,02-copy,03-visual-identity/higgsfield,04-publishing/daily-engine,05-calendar,06-branding/prompt-library,06-branding/templates/stills}

  # --- the outlier engine's swipe feed (G11) — seed only, never clobbered ---
  w "$M/00-context/swipe/channels.txt" <<EOF
# Swipe feed for the outlier engine (tools/marketing-machine/growth/OUTLIER-ENGINE.md).
# One YouTube @handle per line. Track ~40 channels in ONE lane, not 40 lanes.
# Find lanes you don't track yet:  bin/outlier discover "<query>"
# Then:  bin/outlier scan --channels marketing/00-context/swipe/channels.txt && bin/outlier score
#
# outlier score = video views / that channel's median views.
#   >= 10x  a signal, something in that video worked
#   >= 30x  a format to reskin THIS WEEK (the window is ~2 weeks)
EOF

  # --- the reels engine's IG accounts feed (G14) — seed only, never clobbered ---
  w "$M/00-context/swipe/accounts.txt" <<EOF
# Instagram accounts for the reels engine (tools/marketing-machine/growth/IG-REEL-LOOP.md).
# One @handle per line, all in ONE niche. Business/Creator accounts only: the free
# Graph rail reads public business accounts and cannot see personal ones.
# Then:  bin/reels scan --accounts marketing/00-context/swipe/accounts.txt
#        bin/reels score --min 10 && bin/reels hooks --top 30
#
# Same math as the outlier engine: lift = metric / that account's median.
# Your own side of the loop:  bin/reels mine && bin/reels ledger
EOF

  w "$M/README.md" <<EOF
# Marketing Machine — $NAME

> OmegaOS per-project marketing machine. SSOT for this project's strategy, copy,
> visual identity (DA + Higgsfield), and zernio publishing. Generated $STAMP.
> Governing rules: R-MARKETING · R-VISUAL-ID · R-CITE · L0/L1.

**zernio profile slug:** \`$SLUG\`  ·  **publish CLI:** \`omega-zernio post $SLUG …\`

## Status board
| Layer | File | Status |
|---|---|---|
| Context | 00-context/product-marketing.md | ☐ to fill |
| Context | 00-context/market-research.md | ☐ to fill |
| Context | 00-context/competitors.md | ☐ to fill |
| Context | 00-context/audience-personas.md | ☐ to fill |
| Strategy | 01-strategy/gtm-strategy.md | ☐ to fill |
| Strategy | 01-strategy/content-strategy.md | ☐ to fill |
| Strategy | 01-strategy/launch-strategy.md | ☐ to fill |
| Copy | 02-copy/copywriting.md | ☐ to fill |
| Copy | 02-copy/ad-creative.md | ☐ to fill |
| Copy | 02-copy/social-content.md | ☐ to fill |
| Copy | 02-copy/cold-email.md | ☐ to fill |
| Visual | 03-visual-identity/DA.md | ☐ to fill |
| Visual | 03-visual-identity/higgsfield/system-prompt.md | ☐ to fill |
| Visual | 03-visual-identity/higgsfield/preprompt.md | ☐ to fill |
| Visual | 03-visual-identity/higgsfield/avatar-soul.md | ☐ to fill |
| Visual | 03-visual-identity/higgsfield/shotlist.md | ☐ to fill |
| Publish | 04-publishing/zernio.md | ☐ to fill |
| Publish | 04-publishing/calendar.json | ☐ to fill |

## Pipeline (marketing machine dependency order)
1. product-marketing-context → 00-context/product-marketing.md (positioning, ICP — read by all)
2. market-research / ads-competitors → 00-context/{market-research,competitors,audience-personas}.md
3. marketing-strategist + content-strategy + launch-strategy → 01-strategy/*
4. mk-copywriting / ad-creative / social-content / cold-email → 02-copy/*
5. brand-identity + art-director → 03-visual-identity/DA.md, then higgsfield-soul-id + higgsfield-generate → higgsfield/*
6. content-strategy calendar → 04-publishing/calendar.json → \`omega-zernio post $SLUG\`

## Golden path to autonomous publishing
validate content → \`omega-zernio connect $SLUG <platform>\` (OAuth, operator) → dry-run → \`omega-zernio post $SLUG …\` / bulk-upload.
EOF

  w "$M/00-context/product-marketing.md" <<EOF
---
project: $NAME
layer: context
produced_by: product-marketing-context (/omg-product-marketing-context)
inputs: [VISION.md, PRD*.md, README.md, brand-book/, existing .agents/product-marketing.md]
status: to-fill
---
# Product Marketing Context — $NAME

> SSOT every other marketing file reads (R-MARKETING: run FIRST). Reconcile with any existing
> \`.agents/product-marketing.md\` (do not contradict; this is the canonical mirror).

## One-liner
## Category & positioning
## ICP (ideal customer profile)
## Buyer personas (summary — detail in audience-personas.md)
## Core value proposition
## Differentiators / moat
## Messaging pillars (3–5)
## Proof / social proof
## Pricing & monetization context
## Voice & tone
EOF

  w "$M/00-context/market-research.md" <<EOF
---
project: $NAME
layer: context
produced_by: market-research (/omg-market-research) [gooseworks opt-in] + manual desk research
status: to-fill
---
# Market Research — $NAME
## Market size & trends (cite sources — R-CITE)
## Demand signals
## Channels where the ICP lives
## Pricing landscape
## Risks / headwinds
EOF

  w "$M/00-context/competitors.md" <<EOF
---
project: $NAME
layer: context
produced_by: ads-competitors / market-competitors
status: to-fill
---
# Competitive Landscape — $NAME
## Direct competitors (table: name · positioning · price · channels · weakness)
## Indirect / substitutes
## Positioning gaps WE exploit
## Swipe-worthy angles competitors miss
EOF

  w "$M/00-context/audience-personas.md" <<EOF
---
project: $NAME
layer: context
produced_by: ads-audience
status: to-fill
---
# Audience Personas — $NAME
> 3–7 forensic personas: demographics · psychographics · pains · triggers · objections ·
> platform-specific targeting (Meta/Google/LinkedIn/TikTok) · negative audiences.
EOF

  w "$M/01-strategy/gtm-strategy.md" <<EOF
---
project: $NAME
layer: strategy
produced_by: marketing-strategist (/omg-marketing-strategist)
status: to-fill
---
# GTM Strategy — $NAME
## Strategic objective (12 mo) + North-star metric
## Motion (PLG / sales-led / community / content)
## Channel strategy & priority
## Funnel architecture (TOFU/MOFU/BOFU)
## Budget posture & expected CAC/LTV
## 90-day roadmap
EOF

  w "$M/01-strategy/content-strategy.md" <<EOF
---
project: $NAME
layer: strategy
produced_by: content-strategy (/omg-content-strategy)
status: to-fill
---
# Content Strategy — $NAME
## Content pillars (3–5)
## Topic clusters (searchable + shareable)
## Channel mix & cadence (which zernio platforms, how often)
## Editorial calendar (→ 04-publishing/calendar.json)
## Repurposing engine (1 hero → N derivatives)
## KPIs per pillar
EOF

  w "$M/01-strategy/launch-strategy.md" <<EOF
---
project: $NAME
layer: strategy
produced_by: launch-strategy (/omg-launch-strategy)
status: to-fill
---
# Launch Strategy — $NAME
## Launch moment(s) & timeline
## Channels (Product Hunt / waitlist / email / social)
## Pre-launch → launch-day → post-launch sequence
## Assets checklist
## Success criteria
EOF

  w "$M/02-copy/copywriting.md" <<EOF
---
project: $NAME
layer: copy
produced_by: mk-copywriting
status: to-fill
---
# Core Copy — $NAME
## Taglines (5)
## Hero headline + subhead variants
## Value-prop blocks
## Feature → benefit table
## CTAs
## Objection-handling copy
EOF

  w "$M/02-copy/ad-creative.md" <<EOF
---
project: $NAME
layer: copy
produced_by: ad-creative (/omg-ad-creative) + ads-copy
pairs_with: 03-visual-identity (visual half — R-VISUAL-ID)
status: to-fill
---
# Paid Ad Copy — $NAME
> Per platform (Meta/Google/LinkedIn/TikTok): headlines, primary text, descriptions, RSA variants.
## Angles (PAS / AIDA / BAB / 4Ps)
## Meta
## Google
## LinkedIn
## TikTok
EOF

  w "$M/02-copy/social-content.md" <<EOF
---
project: $NAME
layer: copy
produced_by: social-content (/omg-social-content)
status: to-fill
---
# Organic Social Content — $NAME
> Posts/threads/carousels/short-form scripts + hooks. Each item maps to a calendar.json slot.
## Hooks bank
## LinkedIn
## X/Twitter
## Instagram / TikTok (scripts)
## Reddit / communities
EOF

  w "$M/02-copy/cold-email.md" <<EOF
---
project: $NAME
layer: copy
produced_by: cold-email (/omg-cold-email)
status: to-fill
---
# Cold Email / Outbound — $NAME
## ICP segments
## Sequences (3–5 touches each)
## Subject lines
EOF

  w "$M/03-visual-identity/DA.md" <<EOF
---
project: $NAME
layer: visual-identity
produced_by: brand-identity (/omg-brand-identity) + art-director-content-engine
inputs: [brand-book/, BRAND*.md, design-system, existing visual assets]
status: to-fill
---
# Direction Artistique (DA) — $NAME
> The visual law every Higgsfield generation obeys. Reconcile with existing brand-book if present.

## Visual concept (1 line)
## Mood / emotion
## Color palette (hex + roles)
## Typography (display / body)
## Composition & layout principles
## Photography / illustration style
## Motion language (if video)
## Reference board (links / descriptions)
## DO / DON'T (anti-generic guardrails)
EOF

  w "$M/03-visual-identity/higgsfield/system-prompt.md" <<EOF
---
project: $NAME
layer: visual-identity/higgsfield
produced_by: higgsfield-generate (system layer) — derived from DA.md
status: to-fill
note: External opt-in (R-VISUAL-ID). Higgsfield CLI = curl|sh + paid web plan; API credits != web subscription. NO live generation in setup phase.
---
# Higgsfield SYSTEM PROMPT — $NAME

> Persistent, brand-locked system prompt prepended to EVERY generation for this project.
> Encodes the DA so output is on-brand by default. Keep it stable; vary per-shot via preprompt.md.

## System prompt (brand-locked)
\`\`\`
[Brand visual law from DA.md: palette, type feel, mood, composition, photography style, DO/DON'T.
 State the non-negotiables: aspect ratios, color discipline, what to never render.]
\`\`\`

## Soul-id binding
- Soul reference id: \`<set after higgsfield-soul-id training — see avatar-soul.md>\`
- Usage: \`higgsfield generate --soul-id <id> …\`
EOF

  w "$M/03-visual-identity/higgsfield/preprompt.md" <<EOF
---
project: $NAME
layer: visual-identity/higgsfield
produced_by: higgsfield-generate (per-shot layer)
status: to-fill
---
# Higgsfield PRE-PROMPT template — $NAME

> Per-generation prompt template. Combine SYSTEM (system-prompt.md) + this PRE-PROMPT + shot variables.

## Template
\`\`\`
{SYSTEM}
Subject: {subject}
Scene/context: {scene}
Format: {aspect_ratio} for {platform}
Action/emotion: {action}
Lighting: {lighting}
--- on-brand constraints from DA: {palette}, {style}
\`\`\`

## Variable presets (per content pillar) — fill from content-strategy.md
EOF

  w "$M/03-visual-identity/higgsfield/avatar-soul.md" <<EOF
---
project: $NAME
layer: visual-identity/higgsfield
produced_by: higgsfield-soul-id (trains the identity anchor ONCE)
status: to-fill
note: NOVA_SOUL_ID already exists for the Nova persona; per-project soul trained separately if the brand needs its own face/character.
---
# Avatar / Soul-ID — $NAME

## Does this project need a consistent avatar/character?  [ ] yes  [ ] no
## If yes — soul identity brief
- Who is the avatar (role, vibe, demographic)
- Face/character reference source
- Personality the visuals should radiate
## Soul-id (filled after training)
- soul_id: \`<returned by higgsfield-soul-id>\`
- trained_on: \`<refs>\`
## If no — visual identity is product/scene-led (no recurring face); document why.
EOF

  w "$M/03-visual-identity/higgsfield/shotlist.md" <<EOF
---
project: $NAME
layer: visual-identity/higgsfield
status: to-fill
---
# Shot List — $NAME
> Each planned image/video generation, mapped to a content pillar + calendar.json slot.
| # | Pillar | Platform | Format | Prompt ref | Soul? | Calendar slot |
|---|---|---|---|---|---|---|
EOF

  w "$M/04-publishing/zernio.md" <<EOF
---
project: $NAME
layer: publishing
tool: omega-zernio (tools/zernio/cli.ts) → https://zernio.com/api/v1
profile_slug: $SLUG
status: to-fill
---
# Publishing — $NAME (zernio)

> One zernio profile per project. Accounts connect via OAuth (operator step). NO auto-connect/auto-publish in setup.

## Target platforms (pick from connected/desired)
facebook · instagram · linkedin · twitter · tiktok · youtube · threads · reddit · pinterest · bluesky · telegram · snapchat · discord · whatsapp · googlebusiness

## Connect commands (run by operator — opens hosted authUrl)
\`\`\`bash
omega-zernio connect $SLUG instagram
omega-zernio connect $SLUG tiktok
omega-zernio connect $SLUG linkedin
omega-zernio accounts $SLUG        # verify isActive
\`\`\`

## Publish (after content validated)
\`\`\`bash
omega-zernio post $SLUG --text "…" --platforms instagram,tiktok --media ./asset.png --dry-run
omega-zernio post $SLUG --text "…" --platforms instagram,tiktok --schedule 2026-07-01T09:00:00Z
\`\`\`

## Cadence (from content-strategy.md)
EOF

  w "$M/04-publishing/calendar.json" <<EOF
{
  "project": "$NAME",
  "profileSlug": "$SLUG",
  "timezone": "Europe/Paris",
  "_note": "Publishing queue derived from 01-strategy/content-strategy.md. Each item -> omega-zernio post / bulk-upload. Do NOT publish until content validated + accounts connected.",
  "posts": []
}
EOF

  # ---------- 05-calendar (daily operating plan) ----------
  w "$M/05-calendar/README.md" <<EOF
---
project: $NAME
layer: calendar
status: to-fill
---
# Calendrier prévisionnel — $NAME

> Ton plan opérationnel quotidien. Objectif : savoir **chaque jour quoi faire en 30 min – 2 h**.
> 1 à 3 posts/jour. Chaque item est taggé **AUTO** ou **MANUEL**.

## Les 2 modes
- **AUTO** 🤖 — je génère le texte + le visuel et je poste via \`omega-zernio post $SLUG …\` (aucune action de ta part). C'est le gros du volume.
- **MANUEL** 🙋 — nécessite un humain : vidéo/selfie fondateur, prise de parole perso, réponse à des commentaires/DM, validation avant envoi. C'est là que passe ton temps.

## Ton budget 30 min – 2 h / jour
Tu ne fais QUE les items **MANUEL** du jour + l'engagement. Le reste part tout seul.
Regarde \`calendar-14d.md\` → colonne MODE. Fais les 🙋, ignore les 🤖 (ils se publient).

## Fichiers
- \`calendar-14d.md\` — le plan 14 jours (dates, plateformes, textes complets, mode, temps estimé).
- \`calendar.json\` — version machine (alimente le poster auto / zernio).
- \`daily-human-tasks.md\` — les rituels humains récurrents (engagement, veille, DM).

## Mise en route de l'AUTO
Connecte les comptes (\`omega-zernio connect $SLUG <platform>\`), puis les items AUTO peuvent être postés/programmés. Voir \`04-publishing/zernio.md\`.
EOF

  w "$M/05-calendar/calendar-14d.md" <<EOF
---
project: $NAME
layer: calendar
horizon: 14 days
cadence: 1-3 posts/day
status: to-fill
---
# Plan 14 jours — $NAME

> Légende MODE : 🤖 AUTO (je poste) · 🙋 MANUEL (toi). Chaque post porte son **texte complet** prêt à publier.

<!-- À remplir : pour chaque jour (J1..J14), 1 à 3 posts. Format par post :
### J1 — <jour> · <date relative>
- **Post 1** · <HH:MM> · <plateforme> · pilier <X> · MODE 🤖/🙋 · ~<min>
  - **Texte :** <le post complet, prêt à copier/publier>
  - **Visuel :** <prompt Higgsfield (AUTO) ou "à filmer/photographier" (MANUEL)>
  - **CTA :** <si applicable>
-->
EOF

  w "$M/05-calendar/calendar.json" <<EOF
{
  "project": "$NAME",
  "profileSlug": "$SLUG",
  "timezone": "Europe/Paris",
  "horizonDays": 14,
  "_note": "Plan quotidien. mode = auto|manual. Les 'auto' alimentent le poster; les 'manual' apparaissent dans l'agenda humain.",
  "days": []
}
EOF

  w "$M/05-calendar/daily-human-tasks.md" <<EOF
---
project: $NAME
layer: calendar
status: to-fill
---
# Rituels humains quotidiens — $NAME (hors posts programmés)

> Ce que TOI tu fais chaque jour en plus des posts, dans ton budget 30 min – 2 h.

## Quotidien (~10-20 min)
- [ ] Répondre aux commentaires/DM de la veille (plateformes actives)
- [ ] Engager : commenter 3-5 posts de comptes cibles / de l'ICP
## Hebdo (répartir dans la semaine)
- [ ] <ex : 1 vidéo/selfie fondateur> (MANUEL, alimente les posts vidéo)
- [ ] <ex : 1 prise de parole perso / retour terrain>
EOF

  # ---------- 05-calendar/calendar-90d.json (canonical machine calendar) ----------
  # This is the calendar the Rust core (marketing.rs) prefers and both daily
  # engines read. Superset schema: days[].posts[].{slot,platform,pillar,mode,
  # text,hashtags,visualPrompt,cta}. Empty until content-strategy fills it.
  w "$M/05-calendar/calendar-90d.json" <<EOF
{
  "schema": "marketing-machine/calendar-90d@2026-07",
  "project": "$NAME",
  "profileSlug": "$SLUG",
  "timezone": "Europe/Paris",
  "horizonDays": 90,
  "_note": "Canonical 90-day plan. Each day: {dayOffset, date?, posts:[{slot,platform,pillar,mode,text,hashtags,visualPrompt,cta}]}. mode = auto|manual. Read by omega marketing + the daily engine. Filled by /omg-content-strategy.",
  "days": []
}
EOF

  # ---------- 06-branding (the token SSOT — read before ANY generation) ----------
  w "$M/06-branding/SOCIAL-BRAND-BOOK.md" <<EOF
---
project: $NAME
layer: branding
produced_by: brand-identity (/omg-brand-identity) + art-director-content-engine (compile from 03-visual-identity/DA.md)
schema: social-brand-book@2026-07
status: to-fill
---
# Social Brand Book — $NAME

> The 10-section master brand doc for social/marketing generation. Compiled from
> \`03-visual-identity/DA.md\`. Every image/video/avatar generation obeys this + \`tokens.json\`.

## 1. Canvas & formats (aspect ratios per platform: 9:16 / 4:5 / 1:1 / 16:9)
## 2. Safe zones & caption reserve (where text/UI never overlaps)
## 3. Typography (display / body / caption, sizes, tracking)
## 4. Color roles (bg / ink / accent / signal — hex + usage)
## 5. Motion language (easing, durations, transitions — if video)
## 6. Photography / illustration style
## 7. Template registry (the reusable Still archetypes → templates/)
## 8. Logo usage (clear space, min size, do/don't)
## 9. Avatar / persona (→ avatar-persona.md)
## 10. Governance (kill-list, validation gate, pattern-ledger loop)
EOF

  w "$M/06-branding/tokens.json" <<EOF
{
  "\$schema": "branding-tokens@2026-07",
  "project": "$NAME",
  "_note": "Machine-readable brand tokens. READ BEFORE ANY generation. Fill from 03-visual-identity/DA.md + /omg-brand-identity.",
  "status": "to-fill",
  "color": { "bg": "", "ink": "", "accent": "", "signal": "", "palette": [] },
  "type": { "display": "", "body": "", "caption": "", "scale": [] },
  "motion": { "easing": "", "durationMs": null, "transitions": [] },
  "canvas": { "formats": ["9:16", "4:5", "1:1", "16:9"], "safeZonePct": null, "captionReservePct": null },
  "photo": { "style": "", "lighting": "", "grade": "" },
  "logo": { "src": "", "clearSpace": "", "minSizePx": null },
  "avatar": { "soulId": null, "heygenAvatarId": null, "mode": "scene-led" }
}
EOF

  w "$M/06-branding/design-guidelines.md" <<EOF
---
project: $NAME
layer: branding
status: to-fill
---
# Design Guidelines — $NAME
## Grid & layout
## Safe zones (per format)
## Type scale (display → caption)
## Logo spec (clear space, min size)
## DO / DON'T (anti-generic guardrails)
EOF

  w "$M/06-branding/avatar-persona.md" <<EOF
---
project: $NAME
layer: branding
produced_by: higgsfield-soul-id (image face) + HeyGen (talking-head)
status: to-fill
note: Set ids only if this brand needs a recurring face/character. Scene-led brands leave them null (a valid choice — document why).
---
# Avatar / Persona — $NAME

## Recurring avatar? [ ] yes  [ ] no (scene-led)
## If yes — persona sheet (role, vibe, demographic, personality)
## Soul-ID (Higgsfield image identity)
- soul_id: \`<from higgsfield-soul-id, or null>\`
## HeyGen (talking-head video)
- heygen_avatar_id: \`<from HeyGen, or null>\`
- voice_id: \`<ElevenLabs / HeyGen voice, or null>\`
EOF

  # ----- 06-branding/prompt-library (pre-filled per-brand generation prompts) -----
  w "$M/06-branding/prompt-library/image-higgsfield.md" <<EOF
---
project: $NAME
layer: branding/prompt-library
model: nano_banana_2 (default) / soul_2_0 / flux_2
status: to-fill
---
# Image prompts (Higgsfield) — $NAME
> Pre-filled, brand-locked image prompts. Encode tokens.json (palette, type feel,
> photo style, canvas). One block per content pillar.

## Base system (prepended to every image gen)
\`\`\`
[Brand visual law from tokens.json + SOCIAL-BRAND-BOOK.md: palette, type feel, mood,
 composition, photography style, DO/DON'T, aspect ratio, what to never render.]
\`\`\`

## Per-pillar prompt presets
EOF

  w "$M/06-branding/prompt-library/video-higgsfield.md" <<EOF
---
project: $NAME
layer: branding/prompt-library
model: seedance_2_0 / kling_3_0 / veo_3_1 / marketing-studio
status: to-fill
---
# Video prompts (Higgsfield) — $NAME
> Brand-locked video/clip prompts + keyframe guidance. Hook in frame 1, beat grid <=5s.

## Base system
## Per-format presets (Reel 9:16 / Story / TV-spot 16:9)
EOF

  w "$M/06-branding/prompt-library/avatar-heygen.md" <<EOF
---
project: $NAME
layer: branding/prompt-library
engine: HeyGen (HEYGEN_API_KEY) — talking-head UGC
status: to-fill
note: Runner not yet built (capabilities.toml U2 = missing). This is the spec the runner will consume.
---
# HeyGen avatar prompts — $NAME
> Talking-head UGC clips with word-timing captions. 3-layout alternator (front / lower-third / split).

## Avatar id + voice (from avatar-persona.md)
## Script template (hook → value → CTA), <=60s
## Caption style (word-timed, safe-zone aware)
EOF

  w "$M/06-branding/prompt-library/flux-brand.json" <<EOF
{
  "\$schema": "flux-brand@2026-07",
  "project": "$NAME",
  "_note": "Flux 2 brand params (style refs, LoRA/refs, negative prompt). Fill from tokens.json.",
  "status": "to-fill",
  "styleRefs": [],
  "negativePrompt": "",
  "guidance": null
}
EOF

  # kill-list carries the R-NODASH line (the one coded guard the engines enforce).
  w "$M/06-branding/prompt-library/kill-list.md" <<EOF
---
project: $NAME
layer: branding/prompt-library
status: active
---
# Prompt kill-list — $NAME
> Words/patterns BANNED from generated copy + prompts. Scanned by the validation gate.

## R-NODASH — never emit em-dashes (—) or en-dashes (–) in any generated copy or caption.
## Generic AI tells (ban): "elevate", "unlock", "seamless", "game-changer", "in today's fast-paced world", "revolutionize", "empower", "leverage" (as verb), emoji-stuffing.
## Brand-specific bans (fill): <words/claims this brand must never make>
EOF

  w "$M/06-branding/templates/README.md" <<EOF
# Still templates — $NAME
> The six reusable \`<Still>\` archetypes (Cover / Big-Stat / Step / Quote / Chart / CTA)
> as HyperFrames HTML / Remotion. Consumed by the daily engine + carousel system.
> v1: skeleton only — actual template code is P1-2 (see AUDIT.md / BACKLOG).

- \`design.md\` — the shared design contract (grid, tokens, safe zones).
- \`frame.md\` — per-frame composition rules.
- \`stills/\` — one file per archetype (to be coded).
EOF

  w "$M/06-branding/templates/design.md" <<EOF
---
project: $NAME
layer: branding/templates
status: to-fill
---
# Template design contract — $NAME
> Shared grid + token bindings every Still archetype obeys. Reads 06-branding/tokens.json.
EOF

  w "$M/06-branding/templates/frame.md" <<EOF
---
project: $NAME
layer: branding/templates
status: to-fill
---
# Frame composition rules — $NAME
> Per-frame rules (title zone, caption reserve, logo lockup) for the Still archetypes.
EOF

  w "$M/06-branding/templates/stills/README.md" <<EOF
# Still archetypes — $NAME
> Cover · Big-Stat · Step · Quote · Chart · CTA. Code lands in P1-2.
EOF

  w "$M/06-branding/pattern-ledger.md" <<EOF
---
project: $NAME
layer: branding
schema: pattern-ledger@2026-07
status: empty
---
# Pattern ledger — $NAME
> Win/lose metrics log. Promote winners; retire any pattern that loses twice.
> Empty until real platform metrics exist (L2 — no fabricated data).

| date | asset | pillar | format | platform | metric | verdict (promote/retire/watch) |
|---|---|---|---|---|---|---|
EOF

  w "$M/06-branding/VALIDATED-STYLE.md" <<EOF
---
project: $NAME
layer: branding
status: to-validate
---
# Validated style — $NAME
> Operator-locked style choices. NOT valid until the operator signs below.
> The daily engine checks this file exists before publishing.

## Locked choices
- <fill after operator review>

## Sign-off
- validated_by: \`<operator>\`
- validated_at: \`<date — unsigned until then>\`
EOF

  # ---------- 04-publishing/daily-engine (portable engine stub) ----------
  w "$M/04-publishing/daily-engine/README.md" <<EOF
---
project: $NAME
layer: publishing/daily-engine
status: stub
---
# Daily engine — $NAME

> Renders + publishes this project's day from \`05-calendar/calendar-90d.json\`,
> obeying \`06-branding/tokens.json\` + \`VALIDATED-STYLE.md\`.
>
> v1 = STUB. A per-project engine is a temporary fork; the PORTABLE engine
> (\`omega marketing run <project>\`) is the target that reads calendar-90d.json
> for ANY project — see BACKLOG (P0-2 schema unification, P0-4 portable runner).

## Wire the engine (once built)
\`\`\`bash
omega marketing run $SLUG --dry-run     # render, don't publish
omega marketing run $SLUG --publish     # render + omega-zernio post
\`\`\`

## Inputs
- \`../../05-calendar/calendar-90d.json\` (what to post)
- \`../../06-branding/tokens.json\` (how it looks)
- \`../../06-branding/VALIDATED-STYLE.md\` (must exist + be signed)

## State
- \`sent-log.json\` — { produced: [dayOffset...], published: [dayOffset...] }
EOF

  w "$M/04-publishing/daily-engine/sent-log.json" <<EOF
{
  "schema": "daily-engine/sent-log@2026-07",
  "project": "$NAME",
  "_note": "Engine state. produced = days rendered; published = days sent. Written by the engine, do not hand-edit while it runs.",
  "produced": [],
  "published": []
}
EOF
  echo
}

if [ "$#" -eq 3 ]; then
  scaffold "$STATION/$1" "$2" "$3"
else
  [ -f "$MANIFEST" ] || { echo "manifest not found: $MANIFEST"; exit 1; }
  while IFS=$'\t' read -r dir name slug; do
    case "$dir" in ''|\#*) continue;; esac
    scaffold "$STATION/$dir" "$name" "$slug"
  done < "$MANIFEST"
fi
echo "=== DONE ==="
