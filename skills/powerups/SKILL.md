---
name: powerups
description: Router into the Power-Ups corpus — 907 purchased business/marketing Claude skills (30 Power-Up packs of 406 skills + the 501-skill Ultimate Bundle) plus 272 companion assets (prompt libraries, swipe files, templates, tutorials) installed at ~/.omega/powerups. Use whenever a request matches a business, marketing, content, sales, launch, email, SEO, social, ads, pricing, course, community, podcast, newsletter, YouTube, LinkedIn, Etsy, outreach, branding, ops, HR, legal, finance, nonprofit or client-consulting task and no first-class OmegaOS skill already owns it. Also use when the user says "/powerups", "power up", "power-ups", "the skills I bought", "the 501 bundle", "youraipowerup", or in French "les skills que j'ai achetés", "le pack de 501 skills", "mes power-ups".
---

# Power-Ups router

907 third-party Claude skills live at `~/.omega/powerups/`. They are NOT registered
with the Skill tool on purpose: 907 entries would add roughly 30k tokens to every
session. You reach them through search, then read the one SKILL.md you need and
follow it exactly as if it had been invoked as a skill.

## The one command

```bash
powerup-find <query>                    # rank skills by relevance (default 12 results)
powerup-find -n 25 <query>              # more results
powerup-find --pack <slug>              # every skill in one pack, plus its assets
powerup-find --list-packs               # the 30 packs and the 20 bundle categories
powerup-find --prompts <query>          # search the 1500-prompt library
powerup-find --prompts --full <query>   # …and print the whole prompt bodies
powerup-find --assets <query>           # swipe files, templates, tutorials, connector setup
powerup-find --json <query>             # machine-readable
```

Also at `~/.omega/powerups/bin/powerup-find` if `~/.local/bin` is not on PATH.

## Protocol

1. **Search first.** Run `powerup-find` with the user's own words. Do not guess a
   path and do not read `INDEX.md` (435 KB).
2. **Pick by description, not by name.** Every skill's `description` states the
   exact triggers it was written for. A close name with a wrong description is
   the wrong skill.
3. **Read the whole SKILL.md** at the path the search printed, then follow it
   literally. These skills are phase-gated (Brief, Design, Build, Polish) and
   several stop at an explicit GATE to confirm with the user. Honour the gates.
   Paraphrasing the protocol instead of running it is the failure R-AUDIT names.
4. **Pull the referenced material.** Many skills ship `references/` and
   `examples/` beside their SKILL.md, and packs ship `resources/` (swipe files
   and templates, converted to Markdown), `prompts/` (50-prompt CSV libraries)
   and `tutorials/` (HTML walkthroughs). Read them when the skill points at them.
5. **A pack has an orchestrator.** Most of the 30 packs contain an `*-orchestrator`
   or `*-architect` skill that drives the rest, and a `*-setup` skill that writes
   a `config.json` brand profile the other skills in that pack read. For a
   multi-step job in one domain, run the setup skill once, then the orchestrator,
   rather than cherry-picking leaf skills.
6. **Prompts are a separate, blunter layer.** The 1500-prompt library (50 per
   pack, `--prompts`) is for a one-shot ask: "give me a hook", "pressure-test
   this idea". A skill is for a real job with gates and deliverables. When both
   fit, run the skill; reach for a prompt when the user wants something
   paste-ready or when no skill covers the angle.

## Precedence — OmegaOS canon wins

These are bought third-party skills, not OmegaOS doctrine. When a first-class
OmegaOS skill covers the request, it wins and the Power-Up is at most a
supplement:

| Request | Use the OmegaOS skill, not a Power-Up |
|---|---|
| Any named audit (ux, code, sec, a11y, seo, perf, …) | the real `/…audit` skill (R-AUDIT) |
| Go-to-market, launch, content strategy, cold email | the `/omg-*` marketing suite (R-MARKETING) |
| Publishing a post or running ads on a project account | Zernio (R-ZERNIO) |
| Design generation or a design process artifact | the Design Router (R-DESIGN) |
| Brand identity for an OmegaOS project | `/omg-brand-identity` |
| Report delivery | the report router (R-ARTIFACT / R-HTML / R-PDF) |

Power-Ups are strongest where OmegaOS is thin: Etsy, podcasts, paid communities,
webinars, faceless channels, course creation, HR, legal and compliance,
nonprofit, industry-specific playbooks, and the long tail of the 501 bundle.

Three Power-Up skill names collide with installed OmegaOS skills (`case-study`,
`growth-engine`, `market-research`). The OmegaOS ones are canon and keep the
name; the Power-Up twins are reachable only through this router.

## What is installed

```
~/.omega/powerups/
  MANIFEST.json          907 skills + 302 assets, each with description and path
  PROMPTS.json           1500 prompts, parsed from the 30 CSV libraries
  INDEX.md               full human-readable index (grep it, do not read it)
  bin/powerup-find       the search tool
  scripts/               rebuild tooling (docx2md, build_index, build_assets)
  packs/<slug>/          30 Power-Up packs
    .claude-plugin/plugin.json
    skills/<name>/SKILL.md (+ references/, examples/)
    resources/  prompts/  tutorials/  connectors/
  bundle-501/<Category>/<name>/SKILL.md    501 skills across 20 categories
  .claude-plugin/marketplace.json          local marketplace listing the 30 packs
  marketplace/           the 30 original .plugin files
  _source-zips/          the 31 pristine archives, for a clean rebuild
```

Every `.docx` template and `.html` tutorial has a Markdown twin written beside
it, so you never have to open a binary or parse HTML. Search returns the twin.

To make one pack always-on in a project (native Claude Code plugin, so its 12 to
15 skills register directly with the Skill tool), add the local marketplace once
with `/plugin marketplace add ~/.omega/powerups`, then install the pack you want.
Do that for a pack actively in use, never for all 30 at once: each pack adds its
whole skill list to every prompt in that project. The 501-skill bundle is
deliberately NOT packaged as a plugin and is reached only through this router.

## Provenance

Purchased from youraipowerup.com by the operator (order `cs_live_b1Up…`,
account `x@agentik-os.com`), installed 2026-07-28. Paid third-party content:
it lives under `~/.omega/` and must NOT be committed to the public OmegaOS
repository. See `~/.omega/powerups/PROVENANCE.md`.
