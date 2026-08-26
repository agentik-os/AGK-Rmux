# Task

Here is an implementation plan for OmegaOS. Challenge it before any code is
written.

Inspect the real repository and the existing off-repo Graphiti runtime under
`~/.omega/memory-db` when useful. Identify architectural mistakes, missing
files, unsafe assumptions, install/update gaps, test gaps, and a narrower or
more reliable implementation path.

Pay special attention to:

- one-poller Telegram semantics and cross-machine installation
- durable idempotency and restart recovery
- prompt injection and SSRF boundaries
- the mandatory `/duo` contract
- Graphiti 0.29.2 and FalkorDB persistence
- Claude and Codex skill/rule parity for Oracle, Worker, and default sessions
- preserving user config, secrets, current memory data, and active rmux sessions
- whether the scope is realistically implementable and testable in one change

Do not edit any file. Respond with concrete objections and file citations.

---

# Plan under review

See `agentic/duo/link-research-graphiti/plan.md`. Read that file completely,
then critique it.
