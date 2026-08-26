# Skill Atlas — discover every skill and how to run it

**Rule:** R-SKILL-ATLAS · **Installer:** `scripts/install-skill-atlas.sh` (install.sh Phase 6.915) · **CLI:** `omega-skills`

## What it is

A single discovery surface so no agent, oracle, or session ever re-derives what
skills exist or how to invoke them.

- **CLI** — `omega-skills` (symlinked onto PATH):
  - `omega-skills` — list every native OmegaOS skill with its `/command`
  - `omega-skills <term>` — search native skills (name / description / command)
  - `omega-skills --powerups <term>` — search the private Power-Up library
  - `omega-skills --all <term>` — search both
  - `omega-skills --rag "<need>"` — SEMANTIC retrieval by meaning (native + library), embeddings or BM25
  - `omega-skills --html` — print the served catalog URL
- **Catalog** — a searchable HTML atlas at `~/.omega/artifacts/omega-skill-atlas.html`,
  served tailnet-only per R-ARTIFACT. Two tabs: OmegaOS native (each skill with
  `/name` + `/omg-name`) and the Power-Up library.
- **Index** — `~/.omega/skills-atlas.json`, rebuilt by `omega-skills-atlas.py`.

## Rebuild

`omega-skills` lazy-builds the atlas if it is missing. Force a rebuild after
adding/removing skills: `python3 ~/.omega/bin/omega-skills-atlas.py`. A stale
atlas that lists a removed skill is a defect (L1).

## The Power-Up library (paid, private)

`~/.omega/skills-library/youraipowerup/` holds a PURCHASED third-party corpus
(907 Claude skills: a 501-skill bundle + 406 from 30 power-up plugins). It is:

- **NOT** loaded into the active `~/.claude/skills/` namespace (≈900 extra skill
  descriptions would bloat every session's context and collide on generic names).
- **NOT** committed to this PUBLIC repo (it is paid third-party IP; publishing it
  here would redistribute someone's paid product).

For the operator's own future installs it is restored, **auth-gated**, from the
PRIVATE `agentik-os/Agentik-Skills` repo (`youraipowerup/` folder). A public or
unauthenticated `git clone OmegaOS && ./install.sh` cannot reach that repo and
simply skips the pull — the atlas then shows only the native skills, which is
correct. Activate one library skill on demand by copying its folder into
`~/.claude/skills/`, or upload its `.plugin` (in `plugins-installable/`) via
Claude › Customize › Plugins.

Opt-out: `OMEGA_SKIP_ATLAS=1` (whole step) · `OMEGA_SKIP_SKILL_LIBRARY=1` (only
the private-library pull).
