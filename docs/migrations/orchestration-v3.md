# Orchestration V3 migration

Date: 2026-07-30

## Compatibility policy

The mission ledger is authoritative for V3 missions. Existing JSON done files,
progress files, timelines, and Telegram cards remain readable compatibility
projections. Each V3 projection carries the mission ID, source sequence, schema
version, and `projection_source=mission_engine`, so it cannot be re-ingested as
a new event.

## Provider default

Fresh or structurally missing configuration selects Codex for `agent_command`
and `aisb_agent`. An existing explicit value is operator state and is preserved,
including an explicit Claude selection. To opt in on an existing install:

```toml
agent_command = "codex"
aisb_agent = "codex"
```

Then run:

```bash
codex login status
omega sync
omega doctor
```

## Mission state

The first V3 dispatch creates `~/.omega/state/mission-engine-v3.sqlite3` in WAL
mode. Schema initialization and additive column migrations are transactional.
No legacy state is deleted. A mission chooses its engine at creation and never
switches authority halfway through execution.

Rollback changes readers back to legacy projections. It does not rewrite or
delete ledger history.

## Skills

`omega skills compile` writes `~/.omega/skill-catalog-v1.json`. Atlas, RAG,
Claude activation, and Codex activation derive from that catalog. The installer
recompiles after the private library mirror so late-arriving skills cannot leave
provider indexes stale.

Codex skill links live under `~/.agents/skills`. Locally owned, unrecognized
entries are never removed by catalog reconciliation.

## Audits

`skills/audits/registry.toml` is the only audit catalogue. The runner requires
`--user-need` and `--hinge`. Gather mode returns success only when a valid
evidence envelope exists. Final status requires a matching `verdict.json`:

```bash
~/.omega/lib/audit-runner.sh code /path/to/project \
  --user-need="reliable mission acceptance" \
  --hinge="candidate completion to accepted result"

~/.omega/lib/audit-runner.sh code /path/to/project \
  --user-need="reliable mission acceptance" \
  --hinge="candidate completion to accepted result" \
  --finalize --threshold=70
```

Missing contracts, failed gatherers, failed summarizers, malformed evidence,
and malformed verdicts exit with status 2. A valid verdict below the threshold
exits with status 1.

## Verification

Run before accepting the migration:

```bash
cargo test --workspace --no-fail-fast
scripts/tests/test_audit_runner.sh
python3 scripts/tests/test_skill_catalog_scripts.py
./scripts/verify-install.sh
git diff --check
```
