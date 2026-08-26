# 7. Workflow Builder

Models the business and user processes a feature runs on. One file per workflow at
`agentic/product/workflows/<slug>.md`.

## What it models
user journey · business process · automation · internal operations · AI agent workflow ·
approval process · data flow · onboarding · sales process · support process.

## Workflow structure (front-matter + body)
```
Workflow
├── Name
├── Type              (journey | process | automation | ai-agent | approval | data-flow | ...)
├── Trigger           (what starts it: event, schedule, user action, condition, webhook)
├── Actors / Roles    (who or what performs each step: user, agent, system, external service)
├── Inputs            (data/context required to start)
├── Steps             (ordered nodes — see below)
├── Conditions        (branch logic between steps: if / else / switch)
├── Actions           (what each step DOES: create, notify, call, transform, wait, approve)
├── Automations       (steps performed with no human: agent/system actions)
├── Outputs           (what the workflow produces / the end state)
├── Exceptions        (error paths, retries, escalation, timeouts)
├── Metrics           (throughput, completion rate, time-in-step, drop-off)
└── Status            (Draft | Modelled | Implemented | Live | Deprecated)
```

## Step (node) shape
```
Step
├── Name
├── Actor            (user | agent | system | external)
├── Action           (the verb it performs)
├── Input -> Output
├── Next             (the next step, or a Condition that routes)
└── On error         (retry | escalate | fail | fallback)
```

## Trigger types
event · schedule (cron) · user action · state condition · webhook · upstream workflow.

## How the agent uses it
- Model the workflow whenever a feature introduces or changes a process, an automation, or an
  AI-agent flow. A feature that changes behaviour without a modelled workflow is under-specified.
- Distinguish **human steps** from **automations** explicitly (mirrors the Vision principle
  "automation without opacity" — every automated step is visible and has an error path).
- The workflow's `Exceptions` and `On error` map directly to the feature's edge cases (ref 4) and
  to worker Done Criteria: an unhandled error path is an acceptance-criteria gap.
- For an **AI agent workflow**, the actors include agents; keep human-in-the-loop and escalation
  steps explicit (R-LOOP: bounded retries, escalate to a human), never an open loop.
```
Trigger -> Step 1 -> [Condition?] -> Step 2a / Step 2b -> Action -> Output
                          |
                      On error -> escalate / retry / fallback
```
