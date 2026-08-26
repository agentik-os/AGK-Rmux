# R-SKILL-ATLAS — Discover skills via the Atlas; the Power-Up library exists

**Kind:** Rule
**Category:** Orchestration
**Added:** 2026-07-29

## Rule

The system exposes a single SKILL ATLAS so no agent, oracle, or session ever re-derives what skills exist or how to run them. Two surfaces, one source of truth (`~/.omega/skills-atlas.json`, rebuilt by the atlas generator):

1. **CLI** — `omega-skills` lists every native OmegaOS skill with its command; `omega-skills <term>` searches name/description/command; `omega-skills --powerups <term>` searches the Power-Up library; `omega-skills --all <term>` searches both; **`omega-skills --rag "<need in plain words>"` retrieves by MEANING across the whole corpus (native + library)** via a semantic index (OpenAI embeddings when a key is present, BM25 lexical fallback otherwise) — this is the session-facing RAG: use it when the right skill is not obvious from a keyword; `omega-skills --html` prints the served catalog URL. Reach for it BEFORE guessing a skill name or paraphrasing a protocol as prose (complements R-AUDIT / R-DESIGN — the router picks the RIGHT skill; the atlas proves it exists and gives the exact command).
2. **Catalog** — a searchable HTML atlas served tailnet-only at `https://station.tail64d114.ts.net:8443/omega-skill-atlas.html` (surface per R-ARTIFACT). Two tabs: OmegaOS native (~361 skills + audits, each with its `/command` and `/omg-<name>` alias) and the Power-Up library.

**The Power-Up library** (`~/.omega/skills-library/youraipowerup/`, 907 Claude skills — 501-skill bundle + 406 from 30 power-up plugins) is a purchased, PAID third-party corpus ([[youraipowerup-skill-library]]). It is DELIBERATELY NOT loaded into the active `~/.claude/skills/` namespace (adding ~900 skill descriptions would bloat every session's context and collide on generic names), and it is NOT committed to the PUBLIC OmegaOS repo (that redistributes paid IP). Agents may READ from it and, on demand, ACTIVATE a specific skill by copying its folder into `~/.claude/skills/` or uploading the matching `.plugin` (in `plugins-installable/`). Treat it as an on-demand arsenal, not always-on doctrine. When a task matches one of its skills better than a native one, name it (via `omega-skills --powerups`) and offer to activate it rather than silently ignoring it.

Rebuild the atlas after adding/removing skills so the index stays true (L1); a stale atlas that lists a skill that no longer exists is a defect.

## Origin

The operator purchased "Your AI Power Up" (907 Claude skills) and asked that everything be integrated into OmegaOS, known to every agent/oracle/session, and that the system SHOW all skills and how to run them with their commands. Without a written pointer, agents would neither know the paid library exists (it is intentionally off the active namespace to protect context) nor have a canonical way to enumerate the ~361 native skills and their exact invocation. R-SKILL-ATLAS makes skill discovery a first-class, injected capability (CLI + served catalog + this doctrine), and pins the paid-library boundary: readable and activatable on demand, never auto-loaded, never pushed to the public repo. Complements R-DESIGN / R-AUDIT (routing), R-ARTIFACT (the catalog surface), and R-SKILLPUB (which governs skills WE author, not this purchased corpus).
