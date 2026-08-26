---
description: Dispatch a mission to an OmegaOS Oracle from any Claude session
argument-hint: <project> <mission text>
---

Parse arguments: first token = project name (must match an existing entry in `~/.omega/projects.json`), rest = mission text.

Then run:
```bash
omega dispatch "$PROJECT" "$MISSION"
```

The oracle spawns in a rmux session. Show the user the resulting `oracle_name` (e.g. `Causio-oracle-2`) and the rmux attach command (`rmux attach -t <oracle_name>`) so they can watch live if they want.

If no project name matches, list available projects from `omega projects` and ask which one.
