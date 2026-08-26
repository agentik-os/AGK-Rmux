# omega-marketing-machine

> **Avant de promettre quoi que ce soit : lis [`NEEDS-OPERATOR.md`](NEEDS-OPERATOR.md).**
> Des rails externes (Zernio, Higgsfield, Apify) sont bloqués côté facturation et
> `omega marketing doctor` les affiche verts parce qu'il ne teste que la présence des
> clés. L'état réel se lit avec `reels doctor` et `higgsfield account status`.

Per-project **marketing machine** scaffolder. Gives every OmegaOS/Partner project an
**identical `marketing/` folder** (the parent folder of all GTM artifacts), then the
marketing skills fill it. Closes the loop from strategy → copy → visual identity (DA +
Higgsfield) → autonomous publishing via **zernio** (`tools/zernio/`).

Governing rules: **R-MARKETING** (GTM suite + dependency order), **R-VISUAL-ID**
(Higgsfield pair), **R-SKILLPUB** (this is the SSOT), **R-CITE**, **L0/L1**.

## The canonical `marketing/` tree

```
marketing/
  README.md                       # status board + pipeline for this project
  00-context/                     # product-marketing-context, market-research, competitors, personas
    swipe/                        # outlier engine: channels.txt (tracked) + swipe.json (scored)
  01-strategy/                    # gtm, content-strategy, launch-strategy
  02-copy/                        # copywriting, ad-creative, social-content, cold-email
  03-visual-identity/             # DA.md (direction artistique)
    higgsfield/                   # system-prompt, preprompt, avatar-soul, shotlist
  04-publishing/                  # zernio.md (platforms + connect cmds) + calendar.json (queue)
```

`00→04` mirrors the marketing-machine dependency order (R-MARKETING). The two layers
beyond the classic GTM suite are **03-visual-identity** (DA → Higgsfield system/preprompt/
avatar so generation is on-brand and identity-faithful) and **04-publishing** (the zernio
queue that turns validated content into scheduled posts).

## Research first (the outlier engine)

Before writing a single short-form video, find the format that is **already** winning. The
outlier score is `video views / that channel's median views`: >=10x is a signal, >=30x is a
format to reskin this week. Free and keyless (`yt-dlp` on the `/shorts` tab).

```bash
bin/outlier doctor                                        # check the data sources
bin/outlier discover "POV you are the last human"         # find channels to track
bin/outlier scan --channels marketing/00-context/swipe/channels.txt
bin/outlier score                                         # what to copy, and how badly
```

Method, prompts, format library and the publish-and-learn loop: **`growth/OUTLIER-ENGINE.md`**.
Scope: YouTube Shorts only. For Instagram, use `bin/reels` below.

## The Instagram reel loop (`bin/reels`)

The same research idea on Instagram, plus the half `outlier` cannot do: measuring your
**own** account and feeding the result back into what you write next.

```bash
bin/reels doctor                                    # which rails are live today
bin/reels scan  --accounts marketing/00-context/swipe/accounts.txt
bin/reels score --min 10                            # lift = metric / that account's median
bin/reels hooks --top 30                            # hook library + niche pattern leaderboard
bin/reels mine                                      # your reels + insights (Graph API, free)
bin/reels ledger                                    # your lift per pattern vs the niche's
```

Hooks are classified in the **same P1-P11 taxonomy** that `/reel-script` writes in and
`/reel-lint` grades against, so a measured pattern is directly actionable. `reels doctor`
diffs the two tables and shouts on drift. Three pluggable sources (Graph API free,
Apify and ScrapeCreators paid); doctor reports each one live and never fakes a green.

Full method, the 4 bricks and the current blockers: **`growth/IG-REEL-LOOP.md`**.

## Scaffold

```bash
tools/marketing-machine/scaffold.sh                       # all projects in projects.tsv
tools/marketing-machine/scaffold.sh Partners/foo Foo foo  # one ad-hoc project (dir name slug)
```

Idempotent + **non-destructive**: only writes a file if it does not already exist, so it
never clobbers content the marketing skills (or you) have written. Add a project by
appending one TAB-separated line to `projects.tsv`.

## Fill the machine

Per project, run the marketing skills in dependency order (they read `00-context/` first),
or fan out one strong agent per project that embodies the method. Each file carries
frontmatter naming the skill that produces it. French products (DentistryGPT, Gluten-Libre,
Verba, …) → outward copy in French (R-STYLE).

## Publish (zernio — the autonomous half)

One zernio profile per project (`profiles slug` = the project slug). Accounts connect via
OAuth (operator step — `omega-zernio connect <slug> <platform>`), never auto-connected.
After content is validated:

```bash
omega-zernio post <slug> --text "…" --platforms instagram,tiktok --media ./a.png --dry-run
omega-zernio post <slug> --text "…" --platforms instagram,tiktok --schedule 2026-07-01T09:00:00Z
```

The per-project `04-publishing/calendar.json` is the queue feeding these.

## Boundaries (external opt-in, R-VISUAL-ID / R-ENV)

- **Higgsfield** image/video generation = curl|sh CLI + paid web plan; API credits ≠ web
  subscription. The setup phase produces prompts/DA/avatar **specs only** — no live
  generation, not runtime-verifiable without operator credentials.
- **zernio** publishing requires the operator's OAuth-connected accounts. Key lives in
  `~/.omega/secrets/integrations.env` (`ZERNIO_API_KEY`), never in the repo.

## Growth kit

`growth/GROWTH-KIT.md` distills the growth mechanics (X teardown of @tibo_maker + @antinertia,
and the IG/YouTube/TikTok/LinkedIn multi-platform analysis) into a runnable, project-agnostic
playbook: the per-platform hook engine, the carousel system (IG + LinkedIn), the comment-to-DM
demo-and-capture loop, the build-in-public engine, named series formats, and per-platform
protocols. Every content skill and daily engine reads it before producing. No em-dash (R-NODASH).
