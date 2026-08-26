# User-flow challenge protocol

## Contents

1. Flow unit
2. Challenge lenses
3. Friction score
4. Step-by-step mutation protocol
5. Recovery and trust
6. Flow report template
7. Anti-patterns

## 1. Flow unit

Treat a flow as an outcome-seeking graph:

```text
trigger -> understand -> decide -> act -> wait -> verify -> continue/recover
```

A screen is only one possible node. Include system transitions, external tools, email/SMS, permission checks, async processing, and return visits.

For every flow, record:

- actor and desired outcome;
- trigger and motivation strength;
- preconditions and permissions;
- information the user already has versus must discover;
- irreversible/consequential points;
- success evidence visible to the user;
- system/business success event;
- likely interruption and resume behavior.

## 2. Challenge lenses

Run independent passes. Do not let visual polish protect a weak flow.

### Outcome lens

- Does the flow solve a real job or expose an internal feature?
- Can success be stated without mentioning the interface?
- Is this the shortest safe path to that outcome?
- Does the first screen answer “why am I here” and “what can I do now”?

### Comprehension lens

- Are labels written in the user's language rather than the data model?
- Is the primary action obvious before scrolling?
- Are plan, role, permission, cost, and consequence visible before commitment?
- Is progressive disclosure hiding complexity or hiding necessary truth?

### Effort lens

- Which data can the system derive, remember, prefill, or defer?
- Which steps repeat a choice already made?
- Can adjacent steps merge without increasing cognitive load?
- Can a power user complete the path from keyboard/composer/command palette?

### Context lens

- Does navigation destroy useful parent, selection, comparison, or source context?
- Should a detail open inline, in a panel, split view, drawer, overlay, or route?
- Does Back mean browser history, semantic parent, or undo? Make these distinct.
- Can the user leave and resume without reconstruction?

### Trust lens

- Can the user see what the system knows, assumes, sends, changes, and stores?
- Are async progress and failures specific rather than generic?
- Is an AI action inspectable, stoppable, retryable, and attributable?
- Are external writes summarized immediately before execution?

### Reversibility lens

- Can delete/archive/move/edit be undone locally?
- If not reversible, is explicit confirmation proportional to the consequence?
- Does retry duplicate an effect? Require idempotency when it could.
- Can a user branch or restore instead of overwriting generated work?

### Accessibility lens

- Is the path complete by keyboard and touch?
- Does focus land and return predictably?
- Are names, roles, values, errors, and progress announced?
- At 200% zoom or narrow width, is the task preserved?

### Business-abuse lens

- Can roles, invitation, payments, referrals, curation, or access rules be bypassed?
- Does convenience weaken trust boundaries?
- Could dark patterns improve a metric while harming the member/user?
- What abuse, spam, fraud, scraping, or accidental disclosure path exists?

### Failure lens

- What if the network dies after submit?
- What if permission changes mid-flow?
- What if source data is stale, deleted, conflicted, or partial?
- What if only some batch items succeed?
- What happens after session expiry, rate limit, provider error, or tool timeout?

## 3. Friction score

Count observable burden in the proposed critical path:

| Metric | Symbol | Guidance |
| --- | --- | --- |
| Physical/keyboard actions | `A` | Click, tap, submit, shortcut, drag |
| Decisions | `D` | Meaningful choices, not automatic transitions |
| New information fields | `I` | Values the user must recall or locate |
| Context switches | `C` | Route, app, device, email, file picker |
| Wait states | `W` | Async boundaries requiring attention |
| Recovery risk | `R` | 0–3 based on consequence and ambiguity |

Use:

`friction = A + 2D + 2I + 3C + 2W + 3R`

Compare before and after for the same outcome. Do not optimize the number by hiding decisions, feedback, safety, or consent. Add a note when lower friction would reduce trust.

Also track:

- time to first meaningful value;
- completion rate;
- abandon point;
- retry/regeneration rate;
- correction/edit rate;
- resume success;
- accidental-action/undo rate;
- support demand caused by uncertainty.

## 4. Step-by-step mutation protocol

For each step:

1. Name the user's question.
2. Name the information required to answer it.
3. Identify the surface/state currently carrying it.
4. Apply one of these mutations:
   - `DELETE`: no outcome or trust value;
   - `DERIVE`: system can compute it;
   - `REMEMBER`: safely reuse a prior choice;
   - `PREFILL`: known but editable;
   - `MERGE`: adjacent decisions share context;
   - `DEFER`: not needed for current outcome;
   - `INLINE`: preserve source context;
   - `PROMOTE`: hidden information is required before commitment;
   - `PARALLELIZE`: independent waits can run together;
   - `BRANCH`: preserve alternatives rather than overwrite;
   - `UNDO`: replace confirmation with recovery;
   - `ESCALATE`: consequence requires explicit approval;
   - `AUTOMATE`: deterministic rule is safe and observable.
5. Re-run failure and accessibility lenses.
6. Record tradeoff and affected IDs.

Do not remove a step merely because it is slow. Some steps establish consent, comprehension, or safety.

## 5. Recovery and trust

Every P0 flow must answer:

- Can the user cancel before commitment?
- What persists after cancellation?
- How is partial success rendered?
- Is retry safe and idempotent?
- Where is the raw error available without overwhelming the default view?
- Can the user return later and resume?
- Who owns a blocked state?
- What is the escape hatch to manual completion/support?

Use optimistic UI only when rollback is clear and the false-success interval is acceptable. Never optimistically render payment, publication, deletion, access grant, or external send as final unless the backend contract guarantees it.

## 6. Flow report template

```text
FLOW-### — Name
Priority: P0 | P1 | P2
Actor / job / outcome:
Source requirements:
Trigger and preconditions:
Success evidence:

Current/proposed graph:
Happy path:
Alternate paths:
Failure/recovery paths:
Permission and plan gates:
Async/resume behavior:
Keyboard/touch behavior:

Friction before: A=?, D=?, I=?, C=?, W=?, R=? -> total ?
Friction after:  A=?, D=?, I=?, C=?, W=?, R=? -> total ?
Mutations applied:
Trust tradeoffs:
Analytics and guardrails:
Affected SURF/STATE/INT/COMP/EVAL IDs:
Open unknowns:
Verdict: keep | simplify | merge | defer | reject | upstream decision
```

When presenting a before/after comparison, use exact step tables rather than vague claims such as “more intuitive.”

## 7. Anti-patterns

- Starting from a dashboard because the product has data.
- Turning every noun into a top-level navigation item.
- Copying the backend entity hierarchy into the sidebar.
- Asking for data already available from account, project, device, or prior step.
- Hiding permission/plan failure until the final submit.
- Using a modal for every secondary task.
- Treating browser Back, semantic parent, cancel, and undo as one action.
- Destroying a compare/selection context to open details.
- Forcing chat for precise repetitive operations better served by controls.
- Forcing controls for ambiguous intent better served by conversation.
- Generic empty states, generic spinners, or errors without a next action.
- Auto-scroll that takes control from a user reading earlier content.
- Confirmation fatigue for reversible actions.
- Silent AI model/tool/routing changes that alter cost, latency, or capability.
- “Mobile later” or desktop components merely stacked in one column.
- Optimizing clicks while increasing ambiguity, risk, or cognitive load.

