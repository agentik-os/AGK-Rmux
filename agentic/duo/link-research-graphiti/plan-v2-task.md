# /duo Plan v2 Synthesis Task

Act as the Claude strategist for the mandatory `/duo` workflow.

Read these files completely:

1. `agentic/duo/link-research-graphiti/plan.md`
2. `/home/vibe/.omega/logs/duo/2026-07-23T19-35-38-506Z-plan.log`
3. `skills/duo/SKILL.md`
4. `crates/omega-core/src/codex_login.rs`
5. The relevant credential, sync, dispatch, Telegram, installer, and Graphiti
   runtime files cited by the critique.

Rewrite the implementation strategy as a realistic Plan v2. Do not edit code.
Return complete Markdown suitable for replacing `plan.md`.

Mandatory decisions:

- Treat the exposed Telegram token as revoked. Never repeat it or use it.
- Make Codex authentication persistence and the `/duo` bridge reliable first.
  The current fresh `~/.codex/auth.json` must be adopted atomically into the
  Omega canonical store; stale backups must never be restored over it.
- Preserve live Codex/rmux sessions. Diagnose old processes but do not kill
  them automatically.
- Ship a dedicated research bot everywhere but disabled/standby by default.
  Exactly one explicitly activated ingress machine owns the rotated token.
  A Telegram 409 disables polling without a service restart storm.
- Use SQLite WAL or an equivalently transactional durable store for Telegram
  offset, intake rows, research jobs, state transitions, leases, and the
  acknowledgement message id. State the unavoidable initial-send ambiguity.
- Separate Telegram update identity from canonical URL/content-version
  research identity. Support explicit force-resubmit.
- Add a constrained fetcher with DNS and per-redirect public-address checks,
  bounded content, safe ports and types, no ambient proxy/credentials, and
  immutable captured evidence. Agents receive the capture, not a URL to refetch.
- Add prompt-injection fixtures. Retrieved or Telegram-authored prose is data,
  never instructions.
- Research starts automatically when a link is shared, but repository mutation
  stops at `awaiting_approval`. Approval must be explicit and authenticated.
- Approved adoption uses a stable request id, a durable dispatch receipt, an
  explicit Claude Oracle, the exact Claude plan -> Codex critique/code ->
  Claude review loop, repository writer locking, bounded retries, audits,
  install parity, commit, and push.
- Package the existing GetZep Graphiti runtime behind `omega-mem`, but never
  mutate the live memory first. Inventory, snapshot, canary restore, pinned
  Graphiti/FalkorDB with AOF, query-level preservation proof, then controlled
  cutover. MCP is optional, not the SSOT.
- Fix `OMEGA_DIR`/`CODEX_HOME` resolution and skill discovery for default
  Claude and Codex sessions. Existing Oracle and Worker rule injection is
  already provider-neutral, so do not redesign it.
- Prefer an explicit repair command over silently turning every already-current
  `omega update` into a rebuild.
- Preserve config, secrets, Graphiti data, user AGENTS.md content, and active
  sessions. Fresh installs without secrets must succeed and remain idle.
- Include exact files, verifiable success criteria, failure injection tests,
  delivery order compatible with `verify-install.sh`, and a phased code plan
  that can be implemented and reviewed without weakening the user's end goal.

The user asked for implementation, not a report-only MVP. Narrow unsafe claims
and stage the delivery, but retain the complete approved-adoption workflow.
