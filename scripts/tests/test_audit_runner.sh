#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT/skills/audits/_shared/audit-runner.sh"
NOTIFY="$ROOT/skills/audits/_shared/audit-notify.sh"
TEST_ROOT="$(mktemp -d)"
export OMEGA_DIR="$TEST_ROOT/omega"
PROJECT="$TEST_ROOT/project"
mkdir -p "$OMEGA_DIR/skills/audits" "$OMEGA_DIR/lib/audit-gather" "$PROJECT"
cp "$ROOT/skills/audits/registry.toml" "$OMEGA_DIR/skills/audits/registry.toml"

python3 - "$ROOT/skills/audits/registry.toml" \
    "$ROOT/skills/audits/audit-orchestrator/SKILL.md" \
    "$ROOT/skills/audits/README.md" "$ROOT/skills/audits/WHEN-TO-USE.md" \
    "$ROOT/skills/audits/_shared/QUALITY-ARSENAL-PREAMBLE.md" \
    "$ROOT/skills/audits" <<'PY'
from pathlib import Path
import re
import sys
import tomllib

with open(sys.argv[1], "rb") as handle:
    registry = tomllib.load(handle)
ids = [entry["id"] for entry in registry["audits"]]
assert registry["meta"]["total_audits"] == len(ids) == 23
assert len(ids) == len(set(ids))
assert {entry["category"] for entry in registry["audits"]} == {"preventive", "detective"}
assert sum(entry["category"] == "preventive" for entry in registry["audits"]) == 16
assert sum(entry["category"] == "detective" for entry in registry["audits"]) == 7
text = open(sys.argv[2], encoding="utf-8").read()
full_mode = text.split("## Full Audit Mode", 1)[1].split("## State Tracking", 1)[0]
for audit_id in ids:
    assert len(re.findall(rf"(?<![a-z]){re.escape(audit_id)}(?![a-z])", full_mode)) == 1, audit_id
assert "Skip Plan + Fix" not in text
readme = open(sys.argv[3], encoding="utf-8").read()
when_to_use = open(sys.argv[4], encoding="utf-8").read()
preamble = open(sys.argv[5], encoding="utf-8").read()
registry_table = preamble.split("## 15. AUDIT REGISTRY", 1)[1].split(
    "## 16. PROJECT SIGNAL DETECTION", 1)[0]
audit_root = Path(sys.argv[6])
skill_ids = {
    path.parent.name for path in audit_root.glob("*/SKILL.md")
    if path.parent.name not in {"audit-orchestrator", "audit-tracker"}
}
assert skill_ids == set(ids), (sorted(set(ids) - skill_ids), sorted(skill_ids - set(ids)))
for audit_id in ids:
    skill_text = (audit_root / audit_id / "SKILL.md").read_text(encoding="utf-8")
    match = re.search(r"^name:\s*([^\n]+)", skill_text, re.MULTILINE)
    assert match and match.group(1).strip() == audit_id, audit_id
    assert "audit-runner.sh" in skill_text, audit_id
    assert "--user-need=" in skill_text and "--hinge=" in skill_text, audit_id
    assert f"`/{audit_id}`" in readme, audit_id
    assert f"`{audit_id}`" in when_to_use, audit_id
    table_row = re.search(
        rf"^\| /{re.escape(audit_id)} \| (\d+) \| (\d+) \|", registry_table,
        re.MULTILINE,
    )
    assert table_row, audit_id
    assert (int(table_row.group(1)), int(table_row.group(2))) == (
        next(entry["max_score"] for entry in registry["audits"] if entry["id"] == audit_id),
        next(entry["phases"] for entry in registry["audits"] if entry["id"] == audit_id),
    ), audit_id
PY

cleanup() {
    rm -rf "$TEST_ROOT"
}
trap cleanup EXIT

fail() {
    echo "test_audit_runner: $*" >&2
    exit 1
}

assert_exit() {
    local expected="$1"
    shift
    set +e
    "$@" >/dev/null 2>&1
    local actual=$?
    set -e
    [ "$actual" -eq "$expected" ] || fail "expected exit $expected, got $actual: $*"
}

set -e

assert_exit 2 "$RUNNER" code "$PROJECT" --hinge="runtime behavior"
assert_exit 2 "$RUNNER" unknown "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior"
assert_exit 2 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" --threshold=101
assert_exit 2 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" --ticket=../escape \
    --url=https://example.test
[ ! -e "$TEST_ROOT/escape" ] || fail "invalid ticket escaped the audit artifact root"
assert_exit 2 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" --ticket=SAFE-1

SYMLINK_PROJECT="$TEST_ROOT/symlink-project"
OUTSIDE="$TEST_ROOT/outside"
mkdir -p "$SYMLINK_PROJECT" "$OUTSIDE"
ln -s "$OUTSIDE" "$SYMLINK_PROJECT/audits"
assert_exit 2 "$RUNNER" code "$SYMLINK_PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior"
[ ! -e "$OUTSIDE/.codeaudit" ] || fail "runner followed a symlinked audit root"

HISTORY_PROJECT="$TEST_ROOT/history-project"
mkdir -p "$HISTORY_PROJECT/audits/.codeaudit"
ln -s "$OUTSIDE" "$HISTORY_PROJECT/audits/.codeaudit/raw-history"
assert_exit 2 "$RUNNER" code "$HISTORY_PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior"

cat > "$OMEGA_DIR/lib/audit-gather/code.sh" <<'SH'
#!/usr/bin/env bash
exit 7
SH
chmod +x "$OMEGA_DIR/lib/audit-gather/code.sh"
assert_exit 2 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior"

cat > "$OMEGA_DIR/lib/audit-gather/code.sh" <<'SH'
#!/usr/bin/env bash
set -eu
mkdir -p "$1"
printf '{"violations":[]}\n' > "$1/results.json"
SH
chmod +x "$OMEGA_DIR/lib/audit-gather/code.sh"
assert_exit 0 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior"

[ -d "$PROJECT/audits/.codeaudit/raw" ] || \
    fail "runner did not use the canonical registry-id output root"
[ ! -e "$PROJECT/.code" ] || fail "legacy root-level audit directory was created"

python3 - "$PROJECT/audits/.codeaudit/evidence-summary.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    summary = json.load(handle)
assert summary["schema_version"] == 1
assert summary["audit"] == "codeaudit"
assert summary["user_need"] == "reliable audits"
assert summary["hinge"] == "runtime behavior"
assert summary["provenance"]["gather_mode"] == "programmatic"
assert summary["evidence"]["tool_outputs"][0]["file"] == "results.json"
PY

chmod -x "$OMEGA_DIR/lib/audit-gather/code.sh"
assert_exit 0 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior"
python3 - "$PROJECT/audits/.codeaudit" <<'PY'
import json
from pathlib import Path
import sys

root = Path(sys.argv[1])
summary = json.loads((root / "evidence-summary.json").read_text(encoding="utf-8"))
assert summary["provenance"]["gather_mode"] == "llm-only"
assert summary["evidence"]["tool_outputs"] == []
assert any((path / "results.json").is_file() for path in (root / "raw-history").iterdir())
PY

cat > "$OMEGA_DIR/lib/audit-gather/code.sh" <<'SH'
#!/usr/bin/env bash
set -eu
sleep 1
mkdir -p "$1"
printf '{"violations":[]}\n' > "$1/results.json"
SH
chmod +x "$OMEGA_DIR/lib/audit-gather/code.sh"
"$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" >/dev/null 2>&1 &
FIRST_RUNNER_PID=$!
for _ in $(seq 1 50); do
    set +e
    flock -n "$PROJECT/audits/.codeaudit/.runner.lock" -c ':' >/dev/null 2>&1
    LOCK_PROBE_RC=$?
    set -e
    [ "$LOCK_PROBE_RC" -ne 0 ] && break
    sleep 0.02
done
[ "$LOCK_PROBE_RC" -ne 0 ] || fail "the first runner never acquired its lock"
assert_exit 2 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior"
wait "$FIRST_RUNNER_PID" || fail "the lock-owning runner did not finish successfully"

assert_exit 2 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" --finalize

python3 - "$PROJECT/audits/.codeaudit/verdict.json" <<'PY'
import json
import sys

verdict = {
    "score": 69,
    "confidence": "high",
    "skill_used": "codeaudit",
    "user_need_match": {
        "quote": "reliable audits",
        "addressed": True,
        "evidence": "scripts/tests/test_audit_runner.sh:1",
        "edge_cases_covered": [],
    },
    "falsifiable_tests": [{"name": "runner", "passed": True}],
    "hinge_findings": [
        {
            "location": "runtime behavior",
            "concern": "fail-open result",
            "verified_safe_by": "runner",
        }
    ],
    "issues_found_and_fixed": [],
    "confidence_basis": "Direct runner execution.",
    "finished_at": "2026-08-11T12:00:00Z",
}
with open(sys.argv[1], "w", encoding="utf-8") as handle:
    json.dump(verdict, handle)
PY
assert_exit 2 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" --finalize

python3 - "$PROJECT/audits/.codeaudit/verdict.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    verdict = json.load(handle)
verdict["user_need_match"]["evidence"] = "scripts/tests/test_audit_runner.sh:1"
verdict["user_need_match"]["edge_cases_covered"] = ["concurrent invocation"]
verdict["falsifiable_tests"] = [
    {
        "name": f"runner-{index}",
        "hypothesis": "the runner would return a non-zero status",
        "command": "bash scripts/tests/test_audit_runner.sh",
        "expected": "exit 0",
        "actual": "exit 0",
        "passed": True,
    }
    for index in range(3)
]
with open(sys.argv[1], "w", encoding="utf-8") as handle:
    json.dump(verdict, handle)
PY
assert_exit 1 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" --finalize

python3 - "$PROJECT/audits/.codeaudit/verdict.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    verdict = json.load(handle)
verdict["score"] = 70
with open(sys.argv[1], "w", encoding="utf-8") as handle:
    json.dump(verdict, handle)
PY
assert_exit 2 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" \
    --scope="different invocation" --finalize
assert_exit 0 "$RUNNER" code "$PROJECT" \
    --user-need="reliable audits" --hinge="runtime behavior" --finalize

cat > "$OMEGA_DIR/lib/audit-gather/code.sh" <<'SH'
#!/usr/bin/env bash
set -eu
mkdir -p "$1"
printf '{"violations":[]}\n' > "$1/results.json"
SH
chmod +x "$OMEGA_DIR/lib/audit-gather/code.sh"
while IFS= read -r audit_id; do
    assert_exit 0 "$RUNNER" "$audit_id" "$PROJECT" \
        --user-need="registry-wide runner coverage" --hinge="runtime behavior"
done < <(python3 - "$ROOT/skills/audits/registry.toml" <<'PY'
import sys
import tomllib
with open(sys.argv[1], "rb") as handle:
    for entry in tomllib.load(handle)["audits"]:
        print(entry["id"])
PY
)
python3 - "$ROOT/skills/audits/registry.toml" "$PROJECT/audits" <<'PY'
import json
from pathlib import Path
import sys
import tomllib

with open(sys.argv[1], "rb") as handle:
    ids = [entry["id"] for entry in tomllib.load(handle)["audits"]]
root = Path(sys.argv[2])
for audit_id in ids:
    summary = json.loads(
        (root / f".{audit_id}" / "evidence-summary.json").read_text(encoding="utf-8"))
    assert summary["audit"] == audit_id
    assert summary["user_need"] == "registry-wide runner coverage"
PY

mkdir -p "$TEST_ROOT/home/.local/bin" "$PROJECT/audits/.privacyaudit"
cat > "$TEST_ROOT/home/.local/bin/telegram" <<'SH'
#!/usr/bin/env bash
exit 0
SH
chmod +x "$TEST_ROOT/home/.local/bin/telegram"
# shellcheck disable=SC2016  # Positional parameters expand inside bash -c.
assert_exit 2 env HOME="$TEST_ROOT/home" OMEGA_DIR="$OMEGA_DIR" \
    OMEGA_TELEGRAM_USER_ID=1 bash -c 'cd "$1" && exec "$2" "$3" verdict unsafe' \
    _ "$PROJECT" "$NOTIFY" ../../escape
# shellcheck disable=SC2016  # Positional parameters expand inside bash -c.
assert_exit 0 env HOME="$TEST_ROOT/home" OMEGA_DIR="$OMEGA_DIR" \
    OMEGA_TELEGRAM_USER_ID=1 bash -c 'cd "$1" && exec "$2" privacy verdict safe' \
    _ "$PROJECT" "$NOTIFY"
[ -s "$PROJECT/audits/.privacyaudit/notifications.log" ] || \
    fail "audit notification did not resolve the canonical registry id"

echo "test_audit_runner: PASS"
