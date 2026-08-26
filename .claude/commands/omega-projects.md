---
description: List all OmegaOS projects (name, path, last activity) discovered under your projects root
---

Run `omega projects` and present the result.

`omega projects` enumerates every registered/auto-discovered project (it scans
your projects root and the rmux session tree). Show a compact table:

- Project name
- Path
- Active oracle/session (if any)
- Last activity

No prose — just the table. If the user asks to act on one, use `omega dispatch`
or `omega new` against that project's path.
