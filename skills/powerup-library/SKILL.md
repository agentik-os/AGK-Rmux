---
name: powerup-library
description: >
  Router into the private Power-Up skill library (907 specialist Claude skills) that is NOT loaded
  into the active namespace. Use this the moment a request matches one of its domains, so the right
  specialist skill is applied instead of a generic answer. Covers: 30 days of content, cold outreach /
  cold email, sell on Etsy, launch a digital product, launch a podcast, grow on YouTube, faceless
  channel, start and grow a newsletter, LinkedIn personal brand, build an email funnel, get more leads,
  write a sales page, write better ads, write like a human, master AI prompting, make AI Instagram
  videos, create AI product photos, build brand identity, nail pricing, productize a service, find your
  niche, run webinars, sell with social proof, build a paid community, write a book, course creator,
  business analyst, automate your business with AI, vibe-code an app; plus bundle categories: Ads & Paid
  Media, Analytics & Data, Branding & Design, Client & Consulting, Content & Copywriting, Courses &
  Education, E-commerce & Products, Email Marketing & Automation, Events & Speaking, Finance & Pricing,
  HR & Team, Legal & Compliance, Operations & Systems, Sales & Funnels, SEO & Search, Social Media.
  Triggers (EN): "help me with a cold email/sequence", "sell on etsy", "launch my product/podcast/course",
  "grow my newsletter/youtube/linkedin", "write my sales page/ads/book", "pricing strategy", "product
  photos", "brand identity", "webinar", "paid community". Triggers (FR): "aide-moi a vendre sur etsy",
  "sequence de cold email", "lancer mon produit/podcast/formation", "faire grossir ma newsletter",
  "ecrire ma page de vente/mes pubs/mon livre", "strategie de prix", "identite de marque". Do NOT use it
  when a NATIVE OmegaOS skill already fits the task better (audits, higgsfield, design, marketing suite).
allowed-tools: ["Bash", "Read", "Glob"]
metadata:
  source: youraipowerup
  version: "1.0"
---

# Power-Up Library Router

The system carries a purchased library of **907 specialist Claude skills** at
`~/.omega/skills-library/youraipowerup/`. It is deliberately kept OUT of the active
skill namespace (loading ~900 descriptions would bloat every session). This router is
the single always-on entry point: it finds the best-matching library skill for the
user's request and applies it.

## When this fires

A user prompt matches one of the domains in the description above and no native
OmegaOS skill is a clearly better fit. (Native audits, higgsfield generation, the
design pack, and the marketing suite win when they apply — check those first.)

## Procedure

1. **Derive 1-3 search terms** from the request (e.g. "cold email sequence" -> `cold email`;
   "sell my prints on Etsy" -> `etsy`; "launch my course" -> `course launch`).

2. **Search by MEANING first** (semantic RAG over the whole corpus, native + library):
   ```bash
   omega-skills --rag "<the user's need in plain words>"
   ```
   This is the source of truth, never guess a skill name. It ranks the best matches
   (embeddings when available, BM25 otherwise) and mixes native + library skills.
   Use `omega-skills --powerups "<term>"` for a pure keyword search of the library only.
   Pick the best 1-2 matches.

3. **Open the matched skill** and read its full protocol:
   ```bash
   # find its path
   grep -l "name: <skill-name>" ~/.omega/skills-library/youraipowerup/**/SKILL.md 2>/dev/null
   # or query the manifest
   python3 -c "import json;m=json.load(open('$HOME/.omega/skills-library/youraipowerup/MANIFEST.json'));print([s['path'] for s in m['skills'] if s['name']=='<skill-name>'])"
   ```
   Then `Read` that `SKILL.md` and **follow it exactly**, the same way you would run any skill.

4. **Prefer applying it inline.** If the user wants it permanently available as its own
   `/command`, activate it: copy its folder into `~/.claude/skills/`, or upload the
   matching `.plugin` from `~/.omega/skills-library/youraipowerup/plugins-installable/`
   via Claude > Customize > Plugins. Tell the user which you did.

5. If nothing in the library fits after searching, say so and fall back to a native skill
   or a direct answer. Do not force a poor match.

## Notes

- The full browsable catalog: `omega-skills --html` (served tailnet page, two tabs).
- Assets per power-up (prompt CSVs, tutorials, docx templates) live under
  `~/.omega/skills-library/youraipowerup/assets/<slug>/`.
- This library is paid third-party IP: read/apply freely on this machine; never publish
  its contents to a public repo (see rule R-SKILL-ATLAS).
