# Discord — first-class interactive control surface

Discord is a lightweight mobile Mission Control over the same canonical
backend as Desktop, Web, and the TUI.

Do **not** design: user → slash command → wall of text.
Target: user → interactive Discord UI → Agentik Core → Hermes → agents.

## Four bots, not more

| Bot | Profile | Job |
|---|---|---|
| @Operator | operator | Operate the Machine |
| @Agentik | agentik | Operate My Organization |
| @Mission | mission | Operate Other Organizations |
| @Private | private | Operate Myself |

**BOT ≠ AGENT.** Never one bot per agent, client, project, session, or
worker. Internal specialists stay behind Hermes. Discord **displays**
them.

Dev twins (`@Operator Dev` …) talk to a staging backend. Do not test
against production tokens. Tokens live in
`~/.omega/scopes/<profile>/secrets` — never in git.

## Same action, every surface

Natural language, slash command, button, desktop click, and web click
resolve to one canonical action after:

Authentication → Context Resolver → Policy Engine → Action → Event → UI refresh

Component IDs are opaque (`agk1.<action>.<profile_id>.…`). No secrets,
no filesystem paths. Discord button state is **not** authority.

Threads are a **view** of a canonical Hermes session. Desktop/Web/TUI
can open the same `session_id`.

## Do not spam

Raw logs stay in runtime storage. Discord gets human-meaningful events
only (`mission.created`, `approval.required`, `run.failed`,
`run.completed`, `agent.waiting`, …).

Approvals: Approve / Reject buttons → server-side validation → resume run.

Reusable cards (ClientCard, MissionCard, ApprovalCard, …) share one
design language. Slash commands open the same panels.
