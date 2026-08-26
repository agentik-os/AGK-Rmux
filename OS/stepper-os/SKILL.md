---
name: stepper-os
description: >
  Drive a project through Stepper OS (AgentikOS suite) - the execution operating
  system around coding agents. Compiles a Blueprint into a dependency-aware graph
  of modules/epics/slices/steps, then executes it step by step through the
  `omega-stepper` CLI: plan -> start -> implement -> verify -> done, where DONE is
  gated by a deterministic verifier (never self-report). Use when the user says
  "/stepper-os", "/omg-stepper-os", "stepper", "execute the blueprint",
  "run the steps", "next step", "step plan", or in French "execute le blueprint",
  "etape par etape", "prochaine etape", "lance le stepper". NOT for designing the
  blueprint itself (that is /blueprint-os or /omg-blueprint-os) and NOT a generic
  todo list (that is the harness task tool).
---

# Stepper OS - execute a Blueprint step by step, verification-gated

Stepper OS is NOT the coding agent. It is the execution operating system around
you: it owns the sequence (planner), the truth (tracker), and the definition of
done (verifier). You implement; Stepper decides what is next and what is proven.

The full doctrine lives in the pack: `OS/stepper-os/pack/` in the OmegaOS repo
(installed at `~/.omega/os/stepper-os/pack/`). Load `12_AGENT_OPERATING_PROTOCOL.md`
as your operating rules for the whole session. The CLI is `omega-stepper`
(auto-installs its venv on first run).

## The loop you follow

1. **Locate or create the project.**
   - Existing Stepper project: a `stepper.yaml` at the project root.
   - New project: `omega-stepper init --name <name>` then compile the Blueprint
     (see "Compiling a Blueprint" below).
2. **Recover exact state first** (never trust conversation memory):
   ```bash
   omega-stepper resume     # reconcile interrupted attempts
   omega-stepper status     # weighted + raw progress
   omega-stepper plan       # ranked READY candidates + safe wave
   ```
3. **Claim the top wave step:** `omega-stepper start <STEP-ID>` - it prints the
   full agent brief (contract, blueprint refs, invariants, forbidden changes,
   commands, acceptance checks). READ every context file it names before editing.
4. **Implement the contract.** One step, one contract - never widen scope.
   Include the required tests, errors, security, docs the brief names.
5. **Close through the verifier:** `omega-stepper done <STEP-ID>`.
   - PASS -> step is DONE, go back to `plan` and keep going (protocol rule 13:
     do not stop at arbitrary milestones).
   - FAIL -> repair against the printed evidence and run `done` again. The
     attempt ceiling (`max_fix_attempts`) is a hard stop: when `start` refuses
     with "escalate", stop and hand the failure to the operator.
6. **Review gates:** steps with `review_roles` need a recorded review:
   `omega-stepper review <STEP-ID> <role> PASS --by <name>`. Never record a
   review you did not actually perform; ask the operator when a human gate
   (security, architecture) is required.
7. **Finish = release gate, not vibes:** `omega-stepper release-check` must PASS.
   Report progress with `omega-stepper report`.

## Compiling a Blueprint into steps

When the project has a Blueprint (e.g. from /blueprint-os) but no steps yet:

1. Read the Blueprint documents (`blueprint/`) AND, when the project went
   through Design OS, the Design Handoff (`design/`). Declare both in
   `stepper.yaml` under `sources:` (`blueprint.root`/`handoff` and
   `design.root`/`handoff`) so every step's references resolve to real files.
2. Derive modules -> epics -> vertical slices -> atomic steps per
   `pack/00_MASTER_SPEC.md` and the schemas in `pack/03_STEP_CONTRACT_SPEC.md`
   (full example: `pack/10_EXAMPLE_STEP.yaml`).
3. Write one YAML file per spec under `stepper/modules/`, `stepper/epics/`,
   `stepper/slices/`, `stepper/steps/`. **Each step names the exact upstream
   docs that govern it** — `blueprint_references` (product truth: WHAT/WHY,
   requirement/decision/capability ids) AND `design_references` (UX/UI truth:
   flow / screen / surface / state ids from the Design Handoff). A UI-touching
   step with no `design_references` is almost certainly missing its design docs.
4. Step granularity: one focused agent cycle (~15 min to ~2 h human-equivalent).
   Every step must be independently executable: contract, context files,
   acceptance checks, definition of done. A vague step is invalid - refine it.
5. `omega-stepper validate` must pass — it checks schema + the acyclic graph AND
   audits that every step's Blueprint/Design references resolve to real docs
   under their source roots (and warns when a UI step cites no design ref).
   The agent brief (`omega-stepper start <id>`) then loads BOTH the Blueprint
   and Design references so the coding agent reads the right source of truth.

## Hard rules (from the pack, enforced by the engine)

- Tracker owns progress; planner owns sequence. Never freelance the order.
- DONE only through `omega-stepper done` - the verifier has final say.
- Dependencies are authoritative; `start` refuses steps that are not READY.
- Repair against evidence; never pivot to unrelated work after a failure.
- Preserve existing user work; never reset the repo to simplify your task.
- Blueprint drift is forbidden: a canonical decision change is a decision
  request to the operator, not a silent redesign.
