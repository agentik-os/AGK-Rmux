# Surface Lab — Mobile, Web, Desktop, Multi-Surface, and Beyond

## Contents

1. Decision law
2. Surface candidates
3. Comparison protocol
4. Multi-surface architecture
5. Anti-patterns
6. Output and handoff contract

## 1. Decision law

Choose the product embodiment from the user's moment and causal mechanism, not from habit or founder excitement. A responsive website is not automatically a mobile product; four clients with duplicated features are not a multi-surface strategy.

The primary surface should own the highest-leverage value event, not automatically the most frequent interaction. A companion surface may own capture, notification, review, or administration without becoming the conceptual center.

Run Surface Lab when the concept involves an interface, repeated workflow, service touchpoint, device capability, collaboration, ambient context, or a user asks whether it should be mobile, desktop, web, or several.

Always permit `no-interface`, `service`, `physical`, or `chat` to win. If surface choice does not materially affect the idea, mark it `not-applicable` rather than inventing an app.

## 2. Surface candidates

| Surface | Native strengths | Best-fit moments | Structural costs / failure signals |
| --- | --- | --- | --- |
| Mobile app | personal, always carried, camera/sensors, location, notifications, quick capture, offline moments | frequent short actions, context-aware help, private identity, field use | install friction, interruption, small canvas, notification abuse, store dependence |
| Web app | instant access, links, search/discovery, broad compatibility, collaboration, fast iteration | onboarding, occasional tasks, shared views, public or cross-company workflows | weak background behavior, browser limits, tab abandonment, lower device integration |
| Desktop app | keyboard, large canvas, local files, sustained focus, local compute, OS integration | deep work, creation, analysis, complex workflows, privacy/local-first | installation/update burden, device lock-in, weak in-the-moment capture |
| Multi-surface | each surface owns a distinct moment in one continuous job | capture on mobile, deep work on desktop, sharing/admin on web | duplicated feature sets, sync conflict, fragmented identity, excessive scope and maintenance |
| Chat / conversational | low learning cost, flexible input, progressive clarification | ambiguous requests, coaching, orchestration, exception handling | hidden state, poor scanability, verbosity, weak precision, dependency risk |
| API / agent layer | composability, automation, embedded distribution | machine-to-machine jobs, partner ecosystems, repeatable operations | observability, permissions, reliability, misuse, integration burden |
| Ambient / wearable | hands-free, contextual, low-friction presence | timely cues, accessibility, safety, passive sensing | surveillance, false triggers, consent, battery, social acceptability |
| Physical / spatial | trust, ritual, embodiment, location, sensory meaning | hospitality, health context, learning, community, premium experiences | logistics, capital, maintenance, accessibility, limited scale |
| Human service | empathy, judgment, recovery, high-context exceptions | trust-critical, premium, early concierge validation | labor intensity, variance, training, margins, founder dependency |
| No interface | value occurs automatically or through an existing channel | infrastructure, invisible coordination, default change | opacity, lack of control, hard-to-build trust or explain value |

## 3. Comparison protocol

### Step A — Identify moments

Map the job across: discovery, commitment, setup, capture, core value event, deep work, collaboration, recovery, review, administration, and exit. Do not assume one surface owns all moments.

### Step B — Score fit

Use 1–5 scores with confidence and one evidence sentence for:

- native affordance advantage;
- moment/context fit;
- frequency and session shape;
- input/output complexity;
- device, sensor, file, or compute need;
- collaboration and shareability;
- trust, privacy, safety, and control;
- distribution and activation friction;
- offline/degraded-mode need;
- accessibility and inclusion;
- build, sync, QA, support, and maintenance burden;
- speed to decisive validation.

Weights come from the concept, not a generic template. A surface cannot win solely because it has the lowest build cost if it breaks the core mechanism.

### Step C — Require a surface thesis

For every survivor state:

> This surface wins because [native affordance] at [user moment] improves [causal value mechanism] enough to justify [specific burden].

If that sentence is generic, the surface choice is not resolved.

### Step D — Prototype the risky moment

Test the smallest interaction that distinguishes surfaces. Do not build complete clients. Examples: notification-to-action loop, drag-and-drop deep-work flow, link-based collaboration, offline field capture, or cross-device handoff.

## 4. Multi-surface architecture

Choose `multi-surface` only when at least two surfaces have non-redundant jobs and the continuity benefit exceeds sync and maintenance costs.

Required role map:

| Question | Required answer |
| --- | --- |
| Surface role | What unique moment and value event does each surface own? |
| Canonical state | Where is authoritative state and how is conflict resolved? |
| Handoff | What object/action moves between surfaces and why? |
| Identity | How do permission, consent, and user context travel? |
| Degraded mode | What remains useful when another surface or connectivity is unavailable? |
| Release strategy | Which surface proves the mechanism first? What evidence unlocks the next? |
| Scope firewall | Which features must never be duplicated without a causal reason? |

Recommended default: one primary surface, one companion surface, and an explicit trigger before adding more. Multi-surface is a sequencing decision as much as an architecture decision.

Validate a companion role through an existing channel, shortcut, share action, lightweight web view, or concierge workflow before funding another native client. Require evidence that native affordances materially improve completion, context, trust, or quality.

## 5. Anti-patterns

- **Platform maximalism:** “mobile + web + desktop” treated as completeness.
- **Responsive camouflage:** one web UI claimed as distinct surface strategy.
- **Feature mirroring:** every client receives the same navigation and features.
- **Notification business model:** mobile value depends on repeated interruption.
- **Desktop by complexity:** using a native shell when browser capabilities are sufficient.
- **App before service:** building software before learning the recurring human workflow.
- **Chat everywhere:** using conversation for precise, repeatable, inspectable tasks.
- **Surface lock-in:** concept value disappears outside one vendor/device ecosystem without strategic justification.
- **Premature stack:** choosing frameworks, APIs, or stores during Brainstorm instead of defining the surface thesis.

## 6. Output and handoff contract

Return:

1. relevant user moments;
2. candidate surface matrix;
3. recommended primary surface and confidence;
4. strongest alternative;
5. multi-surface role map when selected;
6. surfaces rejected and why;
7. riskiest surface hypothesis;
8. cheapest discriminating prototype;
9. trigger for adding the next surface;
10. concept-level experience principles for Blueprint {OS}.

Brainstorm {OS} owns the embodiment decision and surface thesis. Blueprint {OS} owns journeys, screens, information architecture, requirements, and technical architecture. Builder {OS} owns implementation.
