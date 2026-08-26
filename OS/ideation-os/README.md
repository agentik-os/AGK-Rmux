# Ideation OS

AgentikOS build chain **#01** - **integrated** (Brainstorm {OS} v3 ULTIMATE).

The most-upstream system of the build chain: an imagination ecosystem and
Council of independent minds that turns a raw intuition into a population of
challenged, evolved, recombined, decision-ready concepts — independent
chambers, Founder DNA, frame fission, idea genomes, collisions, counterfactual
worlds, structural mutation, incubation, adaptive specialists, adversarial
debate, evidence routing, quality gates, concept lineage, experiments, the
Surface Lab, and downstream handoffs. Payload source:
`Brainstorm_OS_v3_ULTIMATE_COMPLETE.zip` (Deposit, 2026-08-10).

Chain: `01 Ideation -> 02 Researcher -> 03 Blueprint -> 04 Designer ->
05 Stepper -> 06 Builder`. Downstream, a frozen concept feeds Market Research
(Researcher OS) or Blueprint OS.

## Layout

| Path | What |
|---|---|
| `pack/` | The pack verbatim: SKILL.md, 12 references (system prompt, operating contract, council + debate, methods + lenses, specialist councils, imagination + evolution, research + evidence, surface lab, quality + evals, agent prompts, output + handoffs, omega integration), assets (session schema, council/surface profiles, manifests, icon), 3 scripts incl. a 7-test suite, agents/openai.yaml |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-ideation` | The OmegaOS CLI — the pack's deterministic session engine (stdlib Python, no venv): init / frame / dna / add / surface / evolve / portfolio / checkpoint / audit / freeze / export / handoff / summary / validate / migrate |
| `commands/codex-ideation-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/ideation-os.md`) |

The Claude command is the `brainstorm-os` skill (the pack's skill folder
verbatim at `skills/brainstorm-os/`), installed as `/brainstorm`,
`/ideation-os` and `/omg-ideation-os`.

## Run it

```bash
omega-ideation init session.json --title "My idea" --project-id my-app --depth council
omega-ideation validate session.json      # structural validity
omega-ideation audit    session.json      # structural quality gates
omega-ideation summary  session.json      # compact session summary
omega-ideation freeze   session.json ...  # version + freeze the selected concept
omega-ideation handoff  session.json ...  # structured downstream handoff
```

The imagination runs in an agent: `/brainstorm` (or `/ideation-os`) in Claude,
the Codex prompt, or the OS master agent (TUI OS tab -> Enter, Telegram bot
via `T`). Depths: spark / imagination / council / deep / red-team / converge /
audit.

## Hard rules

- Real dissent, not agreement theatre: independent chambers, red teams,
  premortems, blind spots.
- Outputs are hypotheses + decisions, never market truth — evidence-dependent
  claims route to Researcher OS.
- Strict project boundaries; never import another project's model/users/brand.
- Concept LINEAGE is preserved across challenge/continue/evolve/go-deeper.

## v1 scope vs pack spec (honest divergences)

Same posture as the other chain OSes: the single-runtime profile. The
multi-agent council runs sequentially in one agent (or via the OmegaOS
Workflow primitive for a real fan-out); `assets/*.json` (council/surface
profiles, session schema, omega-extension manifest) are honored as the
contract kept via the CLI, not a typed dispatch server.
