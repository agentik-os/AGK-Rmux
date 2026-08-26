# OmegaOS Marketing Machine — SSOT & Two-Tier Delivery

> **What this is.** The single, authoritative record of how OmegaOS's marketing skills are
> sourced, vendored, mirrored, and routed across the **two sources of truth** (R-SKILLPUB).
> It closes the "marketing machine" by making the architecture auditable and durable, so a
> fresh `git clone && ./install.sh` reproduces exactly the stated state (L0).
>
> Companion: [`marketing-mastery-alignment.md`](./marketing-mastery-alignment.md) (doctrine ↔
> tactics). Governing rules: **R-MARKETING**, **R-VISUAL-ID**, **R-BROWSER**, **R-SKILLPUB**.

---

## The two sources of truth (R-SKILLPUB)

- **SSOT-1 — `github.com/agentik-os/Agentik-Skills`** (private): the operator's full skill
  *library* (187 `SKILL.md`), organized by category folder (`Marketing/`, `Ads/`, `SEO/`,
  `CAIO/`, `Design/`, `Motion/`, …). The shareable mirror.
- **SSOT-2 — OmegaOS itself**: `skills/<name>/` in the repo + an `install.sh` copy/wire step +
  the installed `~/.omega/skills/<name>/`. **OmegaOS is the SSOT**; the library is the mirror.

A skill that is *part of OmegaOS* must live in **both**. The library may additionally hold
skills that are **not** part of OmegaOS — the library is a superset.

## Two delivery tiers — deliberate, not a gap

OmegaOS marketing skills ship in two tiers. This is an explicit design decision, recorded in
three places: `NOTICE-marketing-visual-id.md` (Agentik-Skills), the `install.sh` comments at
the Marketing-Mastery loop (*"vendored here, so it ships even when the private Agentik-Skills
clone is unavailable"*), and `docs/marketing-mastery-alignment.md` (doctrine vs execution).

### Tier 1 — VENDORED CANON (ships to everyone, no private repo needed)
Copied into the OmegaOS repo under `skills/` and wired by a dedicated `install.sh` loop. These
reproduce on **any** fresh install, with or without Agentik-Skills access.

| Group | Skills | `install.sh` loop | Routing |
|---|---|---|---|
| GTM suite (R-MARKETING) | launch-strategy · product-marketing-context · cold-email · social-content · ad-creative · content-strategy · market-research · marketing-strategist | `GTMK` loop | `/omg-<name>` |
| Visual-identity pair (R-VISUAL-ID) | higgsfield-soul-id · higgsfield-generate | `GTMK` loop | `/omg-<name>` |
| Marketing-Mastery doctrine | marketing-master + mm-01 … mm-12 (13) | `MMK` loop → `skills/marketing-mastery/<name>/` | `/omg-<name>` + `/<name>` |
| Video-analysis primitive (R-MARKETING) | watch | dedicated `WATCH_SRC` block | `/omg-watch` + `/watch` |

**24 canon skills — present in BOTH SSOTs, install.sh-wired, `/omg-*` routable.**

### Tier 2 — MIRRORED LIBRARY (operator-only; auth-gated)
Live only in SSOT-1 (Agentik-Skills) under their category folders. At install time
**Phase 6.95** clones the private library and *mirrors* every `SKILL.md` into
`~/.omega/skills/<name>/`, **skipping any name OmegaOS already vendors**
(`[[ -d "$OMEGA_SRC/skills/$sk_name" ]] && continue`). They are **not** vendored into the
OmegaOS repo and get **no** `/omg-*` slash stub — they are reachable via skill auto-discovery.
Without private-repo auth, a fresh box does **not** receive them (install.sh says so honestly).

| Family | Count | SSOT-1 | OmegaOS repo | Delivery | `/omg-*` |
|---|---|---|---|---|---|
| `mk-*` (Corey Haines marketing pack) | 26 | ✓ | — (by design) | Phase 6.95 mirror | — (skill-discovery) |
| `ads-*` / `ads_*` / `meta_ads_*` / `ad_designer` / `campaign_planner` | 21 | ✓ | — (by design) | Phase 6.95 mirror | — (skill-discovery) |
| `market` / `market-*` (AI Marketing Suite) | 14 | ✓ | — (by design) | Phase 6.95 mirror | — (skill-discovery) |
| `ag-seo-*` | 5 | ✓ | — (by design) | Phase 6.95 mirror | — (skill-discovery) |

**66 library skills — present in SSOT-1, mirror-delivered, intentionally NOT vendored.** The
Marketing-Mastery doctrine (Tier 1) *routes to* these as its execution layer; on a box without
library access, the doctrine degrades to its own embedded guidance.

> **Why not vendor them too?** (1) It would contradict the committed Phase 6.95 mirror
> mechanism and the deliberate "vendor only the canon" decision. (2) Several library packs are
> "rights reserved / mirrored for operator use only" (per `NOTICE`), and OmegaOS is open-source —
> vendoring them would be a licensing problem. (3) R-SKILLPUB requires OmegaOS→library mirroring,
> not library→OmegaOS. The library is correctly a superset.

## External opt-in runtime dependencies (R-MARKETING / R-VISUAL-ID / R-BROWSER)

These are **runtime opt-ins**, never auto-installed by `install.sh`; a live run is not
runtime-verifiable without the operator's own credentials:

- **market-research** → gooseworks paid data API + `~/.gooseworks/credentials.json`
  (`npx gooseworks login`). Documented in `skills/market-research/SKILL.md` + R-MARKETING.
- **higgsfield-soul-id / higgsfield-generate** → higgsfield CLI (curl|sh) + paid plan.
- **browser-use** (agentic) → `BROWSER_USE_API_KEY` + pip venv at `~/.omega/skills/browser-use/.venv`.
- **meta_ads_*** (library) → Meta Ad Library / Marketing API access.
- **watch** (Tier-1 vendored) → ffmpeg + yt-dlp on PATH (apt/pipx, runtime install, never auto-installed); optional Whisper fallback via GROQ_API_KEY or OPENAI_API_KEY (env, then ~/.config/watch/.env, then ~/.omega/secrets/integrations.env), runtime opt-in; the captions path needs no key. /watch is the canonical video-analysis primitive: competitor hook analysis feeding ads-hooks, ads-video, ad-creative, scriptwriter, social-content, YoutubeContent, art-director-content-engine.

## `.agents/product-marketing.md`

Not a repo file. `product-marketing-context` **writes** `.agents/product-marketing.md` *in the
active client project* (SKILL.md), where the other marketing skills read it. It is a per-project
runtime artifact — **N/A** for the OmegaOS-internal repo, which runs no GTM on itself.

## Closure status (L0/L1)

- `cargo build --release` → 0 errors (R-MARKETING/R-VISUAL-ID/R-BROWSER compile, `rules.rs`).
- `scripts/verify-install.sh` → exit 0, **"INSTALL PARITY OK"** (asserts the Agentik-Skills
  mirror step is present in `install.sh`).
- OmegaOS tree clean, in sync with `origin/main`. SSOT-1 marketing families committed + pushed.
