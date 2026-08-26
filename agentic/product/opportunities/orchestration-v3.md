---
type: opportunity
title: Adaptive and provider-neutral orchestration
status: Validated
user_segment: OmegaOS operator and downstream installers
frequency: Every mission
severity: High
confidence: 0.91
related:
  - pillar:governance-kernel
  - pillar:mission-runtime
  - pillar:adaptive-orchestration
updated: 2026-07-30
---

# Opportunity: Adaptive and provider-neutral orchestration

## User problem

OmegaOS currently combines universal safety invariants, operator preferences, project-specific procedures, provider-specific commands, and infrastructure runbooks in one doctrine surface. The result is excessive context, contradictory absolutes, outdated model references, and orchestration decisions driven by fixed ceremony rather than measured value.

## Context

The problem appears at session start, mission dispatch, skill routing, worker spawning, quality gating, recovery, and reporting. OpenAI Codex now supplies native plans, subagents, hooks, skills, plugins, connectors, sandboxing, and permissions, while OmegaOS still carries policies designed around older Claude-only primitives.

## Existing solution

A typed Rust rules registry, generated doctrine files, rmux-backed Oracle and Worker sessions, skill directories, Quality Arsenal audits, and shell hooks.

## Current frustration

- Provider-specific mechanics leak into universal laws and rules.
- Fixed thresholds can over-orchestrate small work and under-specify complex work.
- Quality is described as 100 percent without a calibrated confidence model.
- Skills lack one explicit, provider-neutral lifecycle contract.
- Detailed infrastructure runbooks consume universal context.
- Several sources of truth must remain synchronized manually.

## Business opportunity

Make OmegaOS the reliable provider-neutral control plane for current agent runtimes, with lower context cost, better completion rates, clearer authority, and easier extension.

## Evidence

- Direct operator request on 2026-07-30 to redesign laws, rules, skills, oracle sessions, and orchestration to current best practice.
- Existing 47-rule doctrine includes provider-specific commands and model identifiers.
- Current repository contains typed mission, oracle, worker, gate, scope, loop, rules, and skill-registry modules, making an incremental migration feasible.
- Official current Codex documentation confirms native subagents, hierarchical AGENTS.md, hooks, skills, plugins, connectors, sandboxing, and approval policies.

## Potential solutions

1. Continue adding rules to the universal prompt.
2. Replace OmegaOS with one provider's native runtime.
3. Keep a small constitutional kernel and move procedures into conditionally loaded policy packs, skills, provider adapters, and runtime state machines.

Option 3 is the candidate feature because it preserves OmegaOS differentiation while removing duplicated platform mechanics.
