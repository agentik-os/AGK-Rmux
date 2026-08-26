# design-intelligence — vendored skill pack

A flattened bundle of UX-process, interaction-design, AI-product-design,
prompt-architecture, AI-alignment, agent-orchestration and AI-evaluation
skills. It complements OmegaOS's existing **visual-generation** skills
(high-end-visual-design, motion, gsap, animejs, theme-factory, higgsfield-*)
and **forensic audits** (uiuxaudit, a11yaudit, …). The router that decides
which skill to reach for is the doctrine rule **R-DESIGN**.

Each subfolder is one skill (`<name>/SKILL.md`), invoked by the Skill tool
via a `~/.claude/skills/<name>` symlink installed by `install.sh`.

## Sources (pinned)

| Upstream | Commit | License |
|---|---|---|
| github.com/Owl-Listener/ai-design-skills | `f41b650` | MIT (`LICENSE.ai-design-skills`) |
| github.com/Owl-Listener/designer-skills  | `acc3e57` | MIT (`LICENSE.designer-skills`) |
| github.com/anthropics/skills (frontend-design only) | `9d2f1ae` | Anthropic (per repo) |

Vendored 2026-07-10.

## Excluded (functional duplicates of existing OmegaOS skills)

`content-strategy` (→ content-strategy) · `data-visualization` (→ dataviz) ·
`accessibility-audit` (→ a11yaudit) · `motion-system` / `animation-principles`
(→ motion, animejs) · `design-token` / `theming-system` (→ theme-factory,
design-system). These are handled by the R-DESIGN router, which points design
requests at the existing skill instead.
