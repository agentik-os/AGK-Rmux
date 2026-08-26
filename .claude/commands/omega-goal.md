---
description: Set a /goal-style success condition for the current session and loop until met
argument-hint: <verifiable success condition>
---

This is a OmegaOS-flavored wrapper around the native Claude Code `/goal` command. It adds two behaviors on top — perform BOTH yourself, they are not automatic:

1. **Record the goal**: write the condition verbatim to `~/.omega/state/session-<session-name>.goal` (plain text, one line) so the operator and patrol tooling can inspect what this session is driving toward. This is an inspection record — if the session restarts, re-run `/omega-goal` to re-arm it.
2. **Report completion** by writing a `.done.json` to `~/.omega/state/oracle-<session>.done.json` so the AISB patrol sees the result via the normal pipeline.

Then invoke the real:
```
/goal $ARGUMENTS
```

After Claude completes the goal, append the result to the done.json with status=done_clean and the verification evidence (build output, test results, screenshots — whatever proves the condition is met), and delete the `.goal` record file.
