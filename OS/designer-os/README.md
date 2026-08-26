# Designer OS (UX/UI)

AgentikOS build chain **#04** - **integrated** (Design {OS} v1.0).

A product-design compiler and adversarial user-flow challenger: it turns the
approved product Blueprint into a challenged, coherent, modern UX/UI definition
and a machine-readable Design Handoff for Stepper — flows, information
architecture, screen/surface contracts, interaction + state coverage, a visual
system, responsive + accessibility states, mapped to shadcn/ui + STAX (the
OmegaOS stack). Payload source: `Design-OS-v1.0-full.zip` (Deposit, 2026-08-10).

Chain: `01 Ideation -> 02 Researcher -> 03 Blueprint -> 04 Designer ->
05 Stepper -> 06 Builder`. Designer sits between Blueprint (WHAT/WHY) and
Stepper (the implementation DAG).

## Layout

| Path | What |
|---|---|
| `pack/` | The pack verbatim: SKILL.md, 12 references (master system prompt, flow challenge, interaction system, visual system, responsive + accessibility, stax + shadcn, AI intelligence, workflow gates, output contract, validation evals, and the intake + handoff JSON schemas), assets (icon), 3 scripts (blueprint-intake validator, design-handoff validator, self-test), agents/openai.yaml |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-designer` | The OmegaOS CLI — the pack's deterministic validators (stdlib Python, no venv): intake / handoff / self-test. NB: distinct from `omega-design` (the Open Design workspace) |
| `commands/codex-designer-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/designer-os.md`) |

The Claude command is the `design-os` skill (the pack skill folder verbatim at
`skills/design-os/`), installed as `/design-os`, `/designer-os` and
`/omg-designer-os`.

## Run it

```bash
omega-designer self-test                       # validator self-test
omega-designer intake  blueprint-intake.json   # validate the Blueprint intake
omega-designer handoff design-handoff.json     # validate the Design Handoff for Stepper
```

The design runs in an agent: `/design-os` (or `/designer-os`) in Claude, the
Codex prompt, or the OS master agent (TUI OS tab -> Enter, Telegram bot via
`T`). It challenges the flow, defines IA / screens / states, maps to
shadcn/ui + STAX, and emits the validated Design Handoff.

## Hard rules

- Preserve product intent; challenge the interface. Start from user goals +
  data, never a gallery of screens.
- Every async state gets a named persistent rendering; reversible actions +
  local undo over confirmation dialogs.
- Consume the Blueprint handoff; never redefine product scope or create the
  implementation DAG. Product conflicts escalate upstream, never a silent
  redesign.

## v1 scope vs pack spec (honest divergences)

Single-runtime profile, like the chain OSes: the engine is the pack's
deterministic validators over the two JSON schemas (intake + handoff); the
design reasoning runs in the agent (skill / bot) or via the OmegaOS Workflow
primitive, and the visual generation stays on the existing OmegaOS generators
(R-DESIGN: high-end-visual-design, the design-intelligence pack, Open Design)
which Design OS orchestrates, never forks.
