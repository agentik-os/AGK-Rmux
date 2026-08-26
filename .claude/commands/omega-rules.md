---
description: List which OmegaOS operational rules are active for this session
---

Read every `.md` file in `~/.omega/rules/` and produce a compact table:

| Rule ID | Title | Category | Active |
|---------|-------|----------|--------|
| L1 | Code lies, runtime tells truth | Universal | yes |
| ... | ... | ... | ... |

After the table, note any rules that look conflicting OR any active-rule constraints that might apply to the current task (read the user's last message for context).

Do not invent rules. If `~/.omega/rules/` is empty, suggest the user run `omega rules export`.
