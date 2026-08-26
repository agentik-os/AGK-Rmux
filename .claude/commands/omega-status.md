---
description: Show OmegaOS runtime status — active oracles, workers, recent done.json events
---

Run `omega list` (live sessions + roles) and `omega status` (system overview), then summarize:

- Active oracles (and which project each owns)
- Active workers and their parent oracle
- AISB master session status (alive / dead / restart count)
- Last 5 `.done.json` events from `~/.omega/state/oracle-*.done.json` (mtime DESC)
- Any worker-blocked files in `~/.omega/state/worker-blocked-*.json`

Output a compact table. No prose.
