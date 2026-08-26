# Design validation and evals

## Contents

1. Evaluation philosophy
2. Eval case contract
3. Required suites
4. Prototype and visual validation
5. AI-specific evals
6. Readiness review

## 1. Evaluation philosophy

Treat every design rule as a hypothesis with an oracle. “Feels intuitive” is not an acceptance criterion. Combine deterministic inspection, instrumented prototypes, accessibility tools, visual comparison, and human judgment.

Run gates at three levels:

- contract: IDs, coverage, state/permission/behavior completeness;
- interaction: executable critical paths and failures;
- perception: hierarchy, comprehension, trust, brand, and content quality.

Do not use an LLM judge as the sole oracle for accessibility, pixel geometry, keyboard behavior, data correctness, or irreversible consequence.

## 2. Eval case contract

```text
EVAL-### — Name
Category: flow | state | navigation | component | visual | a11y | ai | trust | performance
Priority: blocking | non-blocking
Refs: upstream + FLOW/SURF/STATE/INT/COMP/A11Y
Preconditions and fixture
Actor/role/plan/device/input mode
Steps/events
Expected visible result
Expected state/data result
Forbidden result
Oracle: deterministic | automated-a11y | visual-diff | human-rubric | mixed
Evidence to retain
```

Every production bug or user-study failure becomes a regression `EVAL-###`.

## 3. Required suites

### Traceability suite

- Critical upstream requirement has complete design coverage.
- Every P0 flow has surface, state, recovery, and eval refs.
- Every component has a flow/system use or is retired.
- No dangling or wrong-prefix refs.

### Cognitive walkthrough

For each P0 flow ask at every step:

1. Will the actor know the correct goal/action?
2. Is the action visible and named in their language?
3. Will they understand the system response?
4. Can they recover from the likely mistake?

Run with novice, returning, and power-user assumptions separately.

### State suite

Test default, hover/focus, loading, empty, error, permission loss, offline, stale/conflict, partial success, cancel, retry, and return/resume where relevant. Use slow and out-of-order events, not only fixtures that resolve instantly.

### Navigation suite

- Deep links open meaningful state.
- Browser Back/Forward reconciles correctly.
- Semantic Navigate Up is distinct from browser Back.
- Refresh/reopen restores promised state.
- Invalid/deleted leaf degrades to a useful ancestor.
- Overlay dismissal precedes panel/route dismissal.
- Focus is placed/restored correctly.

### Action/consequence suite

- Reversible local action exposes working undo.
- Undo restores state and selection/focus.
- External write requires scoped approval.
- Retry is idempotent.
- Partial batch success shows per-item outcome.
- Permission change immediately disables/rejects the write.

### Responsive suite

Test representative containers, not device labels only. Include:

- 320 CSS px/reflow target where applicable;
- 200% zoom;
- medium split view;
- expanded desktop;
- software keyboard open;
- long localization and RTL;
- coarse pointer/no hover;
- orientation change and safe areas.

### Accessibility suite

Use automated checks plus manual keyboard and screen-reader testing. Include accessible names, roles, values, errors, live status, modal behavior, target size, dragging alternative, focus not obscured, forced colors, reduced motion, and content reflow.

### Content/data suite

- Zero/one/many/huge records.
- Long names, large/negative numbers, missing/estimated/stale values.
- Localized dates/currency/decimal formats.
- Empty and permission-redacted content.
- User-generated unsafe/invalid content.
- Derivations use one source and meaning-aware signs.

### Visual-system suite

- No hardcoded values outside allowed exceptions.
- Token contrast passes in every state/theme.
- Component variants match registry.
- Radii, borders, spacing, icon optical size, and typography roles do not drift.
- Skeleton geometry resembles final content.
- Compact/dark/high-contrast states remain legible.

## 4. Prototype and visual validation

Prototype in risk order:

1. ambiguous navigation/state transition;
2. consequential action/recovery;
3. asynchronous or AI execution;
4. compact/mobile transformation;
5. high-density data/content;
6. final visual signature.

Use realistic content and error states. A polished happy-path mock is not evidence.

Retain:

- flow recording or step captures;
- viewport/theme/state matrix;
- keyboard/focus trace;
- usability observations and severity;
- before/after friction metrics;
- visual diffs tied to token/component refs;
- changed design decisions.

Human comprehension rubric (1–5): goal clarity, next-action clarity, system-status clarity, consequence clarity, recovery confidence, information hierarchy, trust, brand fit. Aesthetic preference alone cannot override a critical usability failure.

## 5. AI-specific evals

Include at least:

- manual model selection is honored;
- automatic route is visible after completion;
- context manifest matches what is sent;
- long paste becomes bounded context token;
- tool steps have named statuses and actionable errors;
- independent tools render in parallel without order confusion;
- stop reaches runtime and leaves a coherent partial result;
- reconnect resumes without duplicated events;
- regeneration/edit creates a branch;
- citations open exact source/span;
- no relevant source yields an explicit unsupported result;
- memory can be inspected, corrected, forgotten, and superseded;
- prompt injection inside retrieved content cannot trigger write;
- external write shows target/effect and requires approval;
- auto-scroll stops when the user reads above;
- model/tool failure offers a truthful next action;
- compact host preserves source/tool/artifact access;
- cost/latency-sensitive mode discloses material tradeoff;
- title/tags/follow-ups fail without blocking the answer;
- privacy telemetry excludes sensitive prompt/source content by default.

Before production AI implementation, establish at least 50 versioned runtime eval cases. Design OS may hand off the first 10–20 representative cases; Stepper expands implementation coverage.

## 6. Readiness review

Conduct one explicit red-team review:

- weakest P0 flow;
- most expensive cognitive decision;
- most dangerous action;
- least visible system state;
- narrowest/highest-zoom layout;
- most fragile focus/keyboard path;
- largest/emptiest/slowest data state;
- highest prompt-injection or cross-tenant risk;
- strongest reason not to use selected navigation model;
- strongest reason a user would abandon or distrust the product.

Record failures as risks/unknowns and revise affected IDs. Do not waive a blocking failure by calling it an implementation detail.

