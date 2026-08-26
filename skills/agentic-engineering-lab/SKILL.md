---
name: agentic-engineering-lab
description: >
  AGK Agentic Engineering Lab loop that Omega oracles actually run, not a blog
  post. Walk Understand → Explain → Design → Build → Debug → Test → Evaluate →
  Secure → Deploy → Observe → Improve. Required coding-agent dimensions: repo
  context, editing, shell, tests, git, sandbox, verification, human-in-the-loop,
  finish reports. Writers cannot self-approve. Fake-done is forbidden. Use when
  the user says "/lab", "/omg-lab", "agentic engineering lab", "run the lab loop",
  or when an Omega oracle is dispatched.
---

# AGK Agentic Engineering Lab

This skill is operational doctrine. Persist the plan, walk the steps, spawn
writers, verify, and report. Do not narrate the loop as a substitute for running it.

## Loop (in order)

`Understand | Explain | Design | Build | Debug | Test | Evaluate | Secure | Deploy | Observe | Improve`

1. Persist first: `omega progress <oracle> --plan "Understand|Explain|Design|Build|Debug|Test|Evaluate|Secure|Deploy|Observe|Improve"`
2. Keep exactly one task `doing`.
3. Spawn writers only: `claude | codex | glm`. Hermes is Home (`omega new --agent hermes`), never dispatch and never a worker.
4. Every worker brief must include both fields or `omega spawn-worker` refuses:

```
omega spawn-worker <task> "<brief>
Done Criteria: <measurable>
Verify Command: <runtime check>" --dir <dir> --files a,b
```

5. Finish reports are mandatory: `done_clean | failed | blocked` plus evidence in `omega status --json` and the oracle inbox.
6. `omega done` is a candidate. The operator alone `omega gate --accept`. Fake-done is forbidden.

## Coding-agent dimensions (every mission)

repo context · editing · shell · tests · git · sandbox · verification · human-in-the-loop · finish reports

## Three backends, one orchestration API

| Backend | Role |
|---|---|
| Codex | Mac/VPS writer and default oracle |
| Hermes | Home pane only (`omega new --agent hermes`) |
| Cloud | Cursor Cloud Agent — writer for OmegaOS itself. Not `omega dispatch`. |
