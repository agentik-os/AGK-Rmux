# Brainstorm {OS}

## Purpose

Run a rigorous multi-agent imagination, evolution, and decision council that turns a raw idea into a population of challenged, evolved, and decision-ready concepts, protecting founder intent while requiring genuine dissent, evidence discipline, and explicit convergence.

## Position

Core Stack (supporting): the suite's ideation entry point, preceding Market Research OS and Blueprint OS.

`Raw idea -> Brainstorm {OS} -> optional Market Research -> Blueprint {OS} -> Stepper {OS} -> Builder {OS}`

## What this OS contains

- One agent adapter, `agents/openai.yaml`, an interface manifest (display name, description, default prompt, icon) that drives the council roles named in `assets/council-profiles.json`. There are no per-agent `.md` files; the roles (Expansion, Reality, Adversarial cells, plus imagination/evolution chambers and specialist hats) are defined in the profile JSON and in `references/council-and-debate.md`, and are spawned as independent agent passes by the operating contract, not as separate agent files.
- 12 reference docs under `references/`: `operating-contract.md` (laws, stages, gates), `council-and-debate.md` (topology, roles, debate rounds), `imagination-and-evolution.md`, `methods-and-lenses.md`, `omega-os-integration.md`, `output-and-handoffs.md`, `quality-and-evals.md`, `research-and-evidence.md`, `specialist-councils.md`, `surface-lab.md`, `system-prompt.md`, `agent-prompts.md`.
- 3 Python scripts under `scripts/`: `brainstorm_os.py` (session state: init, migrate, record frames/genomes/generations, compare surfaces, incubate, audit, freeze, export, hand off, validate), `install_omega_os.py` (explicit Omega OS installation only), and `test_brainstorm_os.py` (the test suite for the session engine).
- 5 assets under `assets/`: `council-profiles.json` (chamber/cell/specialist definitions), `session.schema.json` (JSON Schema for a session), `surface-profiles.json` (portable embodiment criteria), `omega-extension.json` and `package-manifest.json` (packaging metadata), and `icon.svg`.

## Commands

Root command: `/brainstorm`

No alias commands are registered at the suite level; the interaction commands (`/brainstorm --deep`, `/ideate --wild`, `/frame-fission`, `/evolve`, `/collision`, `/worlds`, `/converge`, `/handoff blueprint`, and the rest) are conversational sub-commands documented in `SKILL.md`, not separate root entries.

## Main handoffs

- Produces `brainstorm.concept.selected`, consumed by Market Research OS (for validation) and Blueprint OS (for a decision-ready concept when validation is explicitly skipped).
- Consumes nothing upstream: Brainstorm {OS} is the suite's ideation entry point.
- Also emits `brainstorm.session.completed` and stages concept lineage into Context & Memory OS.

## Triggers

Use for: `/brainstorm`, Brainstorm {OS}, ideation, challenging or evolving an idea, wilder non-obvious directions, agent councils, blind spots, red teams, premortems, evidence return, debate audits, or convergence. Also for choosing mobile, web, desktop, multi-surface, chat, API, ambient, physical, service, or no-interface embodiment, and for preparing ideas for Market Research {OS}, Blueprint {OS}, decisions, experiments, or creative/project briefs.

## Declencheurs (FR)

- "brainstorm sur cette idee"
- "session de creativite"
- "conseil de personas"
- "genere des concepts"
- "fais evoluer cette idee"
- "challenge mon idee"
- "quelle direction choisir"

## More detail

See `OMEGA_INTEGRATION.md` for the registration record, context injection order, event types, and state classification.
