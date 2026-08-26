#!/usr/bin/env bash
# audit-runner.sh — Canonical hybrid audit orchestrator.
#
# DESIGN (2026-05-08):
#   Old: LLM reads 30-200 files, runs hand-grep, judges everything → 100-300k tokens
#   New: programmatic gather (lighthouse, axe, semgrep, eslint, etc.) → JSON
#        LLM Phase 2: reads JSON + 3-5 critical files → DEEPER synthesis
#                     (more Popper falsification, more cross-audit XR, more
#                      user-need verification, edge case hunting)
#        LLM Phase 3: contextual fix plan
#
# Token reduction: 70-85% on gather. Quality ↑↑ because:
#   - Tools catch ALL violations deterministically (no LLM omission)
#   - Freed token budget is REINVESTED in synthesis depth
#   - LLM can spend its context on "is this a real bug for THIS user" instead
#     of "what's the bug in this 500-line file"
#
# USAGE:
#   audit-runner.sh <audit-name> <project-path> \
#     --user-need="..." --hinge="..." [--ticket=ID] [--files="..."]
#   audit-runner.sh <audit-name> <project-path> \
#     --user-need="..." --hinge="..." --finalize [--threshold=70]
#
# OUTPUT (in $project-path/audits/.<canonical-audit-id>/):
#   raw/                  — programmatic tool outputs (JSON)
#   raw-history/          — prior raw/ generations, preserved across reruns
#   evidence-summary.json — structured data the LLM consumes
#   verdict.json          — LLM's score + findings + fix plan
#
# EXIT:
#   Gather mode: 0 = evidence is valid and ready for the LLM
#   Finalize mode: 0 = verdict score meets threshold, 1 = below threshold
#   Any mode: 2 = invalid contract, gather failure, or invalid/missing evidence

set -uo pipefail
umask 077

AUDIT="${1:?Usage: audit-runner.sh <audit-name> <project-path> [args...]}"
PROJECT_PATH="${2:?Missing project path}"
shift 2

# Parse optional args
TICKET=""
FILES=""
USER_NEED=""
URL=""
HINGE=""
SCOPE=""
FOCUS=""
SELECTOR=""
NO_FIX=0
FINALIZE=0
THRESHOLD="70"
for arg in "$@"; do
    case "$arg" in
        --ticket=*) TICKET="${arg#--ticket=}" ;;
        --files=*) FILES="${arg#--files=}" ;;
        --user-need=*) USER_NEED="${arg#--user-need=}" ;;
        --url=*) URL="${arg#--url=}" ;;
        --hinge=*) HINGE="${arg#--hinge=}" ;;
        --scope=*) SCOPE="${arg#--scope=}" ;;
        --focus=*) FOCUS="${arg#--focus=}" ;;
        --selector=*) SELECTOR="${arg#--selector=}" ;;
        --no-fix) NO_FIX=1 ;;
        --threshold=*) THRESHOLD="${arg#--threshold=}" ;;
        --finalize) FINALIZE=1 ;;
        --quick|--streamlined|--lightweight|--light|--fast|--custom)
            echo "AUDIT-RUNNER: forbidden reduced-depth mode: $arg" >&2
            exit 2
            ;;
        *)
            echo "AUDIT-RUNNER: unknown argument: $arg" >&2
            exit 2
            ;;
    esac
done

if [ ! -d "$PROJECT_PATH" ]; then
    echo "AUDIT-RUNNER: project path is not a directory: $PROJECT_PATH" >&2
    exit 2
fi
if [ -z "$USER_NEED" ]; then
    echo "AUDIT-RUNNER: --user-need is required" >&2
    exit 2
fi
if [ -z "$HINGE" ]; then
    echo "AUDIT-RUNNER: --hinge is required" >&2
    exit 2
fi
if [ ${#USER_NEED} -gt 8192 ] || [ ${#HINGE} -gt 8192 ] || \
   [ ${#FILES} -gt 65536 ] || [ ${#URL} -gt 8192 ] || \
   [ ${#SCOPE} -gt 8192 ] || [ ${#FOCUS} -gt 8192 ] || \
   [ ${#SELECTOR} -gt 8192 ]; then
    echo "AUDIT-RUNNER: argument exceeds the audit contract size limit" >&2
    exit 2
fi
if [ -n "$TICKET" ]; then
    if [[ ! "$TICKET" =~ ^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$ ]]; then
        echo "AUDIT-RUNNER: --ticket contains unsafe characters" >&2
        exit 2
    fi
    if [ -z "$URL" ]; then
        echo "AUDIT-RUNNER: --ticket requires --url" >&2
        exit 2
    fi
fi
if ! python3 - "$THRESHOLD" <<'PY'
import math
import sys

try:
    value = float(sys.argv[1])
except ValueError:
    raise SystemExit(1)
raise SystemExit(0 if math.isfinite(value) and 0 <= value <= 100 else 1)
PY
then
    echo "AUDIT-RUNNER: --threshold must be a number from 0 to 100" >&2
    exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OMEGA_ROOT="${OMEGA_DIR:-$HOME/.omega}"
REGISTRY="$OMEGA_ROOT/skills/audits/registry.toml"
if [ ! -f "$REGISTRY" ]; then
    REGISTRY="$SCRIPT_DIR/../registry.toml"
fi
if [ ! -f "$REGISTRY" ]; then
    echo "AUDIT-RUNNER: canonical audit registry not found" >&2
    exit 2
fi

CANONICAL_AUDIT="$(
    python3 - "$REGISTRY" "$AUDIT" <<'PY'
import sys
import tomllib

with open(sys.argv[1], "rb") as handle:
    registry = tomllib.load(handle)
if not isinstance(registry, dict) or not isinstance(registry.get("audits"), list):
    raise SystemExit(1)
entries = registry["audits"]
if registry.get("meta", {}).get("total_audits") != len(entries):
    raise SystemExit(1)
ids = []
for entry in entries:
    if not isinstance(entry, dict):
        raise SystemExit(1)
    audit_id = entry.get("id")
    if (not isinstance(audit_id, str) or
            not __import__("re").fullmatch(r"[a-z0-9-]+audit", audit_id)):
        raise SystemExit(1)
    ids.append(audit_id)
if len(ids) != len(set(ids)):
    raise SystemExit(1)
requested = sys.argv[2].strip().lower()
if not __import__("re").fullmatch(r"[a-z0-9-]+", requested):
    raise SystemExit(1)
candidates = (requested, f"{requested}audit")
for candidate in candidates:
    if candidate in ids:
        print(candidate)
        raise SystemExit(0)
raise SystemExit(1)
PY
)" || {
    echo "AUDIT-RUNNER: unknown audit '$AUDIT' in $REGISTRY" >&2
    exit 2
}

PROJECT_PATH="$(cd -- "$PROJECT_PATH" && pwd -P)" || {
    echo "AUDIT-RUNNER: cd $PROJECT_PATH failed" >&2
    exit 2
}
cd "$PROJECT_PATH" || exit 2

# Machine evidence has one canonical root. The registry id, rather than an
# invocation alias (`code`, `sec`, ...), prevents two names for the same audit
# from creating divergent histories.
ARTIFACTS="audits/.${CANONICAL_AUDIT}"
[ -n "$TICKET" ] && ARTIFACTS="audits/.linear-fix/${TICKET}/.${CANONICAL_AUDIT}"
if ! python3 - "$PROJECT_PATH" "$ARTIFACTS" <<'PY'
from pathlib import Path
import sys

project = Path(sys.argv[1])
target = project / sys.argv[2]
audit_root = project / "audits"
try:
    target.relative_to(audit_root)
except ValueError:
    raise SystemExit(1)
current = audit_root
relative = target.relative_to(audit_root)
for part in (Path(), *[Path(*relative.parts[:index]) for index in range(1, len(relative.parts) + 1)]):
    candidate = audit_root / part
    if candidate.is_symlink():
        raise SystemExit(1)
PY
then
    echo "AUDIT-RUNNER: audit artifact path is unsafe or symlinked" >&2
    exit 2
fi
mkdir -p "$ARTIFACTS" || {
    echo "AUDIT-RUNNER: cannot create audit artifact directory" >&2
    exit 2
}

for protected_path in "$ARTIFACTS" "$ARTIFACTS/raw" "$ARTIFACTS/raw-history" \
    "$ARTIFACTS/runner.log" "$ARTIFACTS/evidence-summary.json" \
    "$ARTIFACTS/verdict.json" "$ARTIFACTS/.runner.lock"; do
    if [ -L "$protected_path" ]; then
        echo "AUDIT-RUNNER: refusing symlinked audit artifact: $protected_path" >&2
        exit 2
    fi
done
if ! command -v flock >/dev/null 2>&1; then
    echo "AUDIT-RUNNER: flock is required for fail-closed audit concurrency" >&2
    exit 2
fi
exec {RUNNER_LOCK_FD}>"$ARTIFACTS/.runner.lock" || exit 2
if ! flock -n "$RUNNER_LOCK_FD"; then
    echo "AUDIT-RUNNER: another $CANONICAL_AUDIT runner owns $ARTIFACTS/.runner.lock" >&2
    exit 2
fi

LOG="$ARTIFACTS/runner.log"
echo "[$(date -Iseconds)] audit-runner $CANONICAL_AUDIT on $PROJECT_PATH" >> "$LOG"
RAW_TMP=""
SUMMARY_TMP=""
# shellcheck disable=SC2329  # Invoked by the EXIT trap.
cleanup_staging() {
    if [ -n "${RAW_TMP:-}" ] && [ -d "$RAW_TMP" ]; then
        rm -rf -- "$RAW_TMP"
    fi
    [ -n "${SUMMARY_TMP:-}" ] && rm -f -- "$SUMMARY_TMP"
}
trap cleanup_staging EXIT

if [ "$FINALIZE" -eq 1 ]; then
    SUMMARY_JSON="$ARTIFACTS/evidence-summary.json"
    if [ ! -f "$SUMMARY_JSON" ] || [ -L "$SUMMARY_JSON" ]; then
        echo "AUDIT-RUNNER: missing or unsafe gather evidence: $PROJECT_PATH/$SUMMARY_JSON" >&2
        exit 2
    fi
    if ! python3 - "$SUMMARY_JSON" "$CANONICAL_AUDIT" "$USER_NEED" "$HINGE" \
        "$TICKET" "$FILES" "$URL" "$SCOPE" "$FOCUS" "$SELECTOR" "$NO_FIX" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    summary = json.load(handle)
invocation = summary.get("invocation") if isinstance(summary, dict) else None
if (not isinstance(summary, dict) or summary.get("schema_version") != 1 or
        summary.get("audit") != sys.argv[2] or
        summary.get("user_need") != sys.argv[3] or
        summary.get("hinge") != sys.argv[4] or
        not isinstance(summary.get("evidence"), dict) or
        not isinstance(invocation, dict) or
        invocation != {
            "ticket": sys.argv[5],
            "files": sys.argv[6],
            "url": sys.argv[7],
            "scope": sys.argv[8],
            "focus": sys.argv[9],
            "selector": sys.argv[10],
            "no_fix": sys.argv[11] == "1",
        }):
    raise SystemExit(1)
PY
    then
        echo "AUDIT-RUNNER: gather evidence does not match this finalization" >&2
        exit 2
    fi
    VERDICT_JSON="$ARTIFACTS/verdict.json"
    if [ ! -f "$VERDICT_JSON" ] || [ -L "$VERDICT_JSON" ]; then
        echo "AUDIT-RUNNER: missing final verdict: $PROJECT_PATH/$VERDICT_JSON" >&2
        exit 2
    fi
    FINAL_SCORE="$(
        python3 - "$VERDICT_JSON" "$CANONICAL_AUDIT" "$USER_NEED" "$HINGE" <<'PY'
import json
import math
import sys
from datetime import datetime

def reject_constant(value):
    raise ValueError(f"non-finite JSON number: {value}")

with open(sys.argv[1], encoding="utf-8") as handle:
    verdict = json.load(handle, parse_constant=reject_constant)
if not isinstance(verdict, dict):
    raise SystemExit(1)
if verdict.get("skill_used") != sys.argv[2]:
    raise SystemExit(1)
user_need = verdict.get("user_need_match")
if not isinstance(user_need, dict) or user_need.get("quote") != sys.argv[3]:
    raise SystemExit(1)
if user_need.get("addressed") is not True:
    raise SystemExit(1)
if not isinstance(user_need.get("evidence"), str) or not user_need["evidence"].strip():
    raise SystemExit(1)
edge_cases = user_need.get("edge_cases_covered")
if (not isinstance(edge_cases, list) or
        any(not isinstance(edge_case, str) or not edge_case.strip()
            for edge_case in edge_cases)):
    raise SystemExit(1)
if verdict.get("confidence") != "high":
    raise SystemExit(1)
tests = verdict.get("falsifiable_tests")
required_test_strings = ("name", "hypothesis", "command", "expected", "actual")
if (not isinstance(tests, list) or len(tests) < 3 or any(
        not isinstance(test, dict)
        or any(not isinstance(test.get(field), str) or not test[field].strip()
               for field in required_test_strings)
        or test.get("passed") is not True
        for test in tests)):
    raise SystemExit(1)
hinge_findings = verdict.get("hinge_findings")
if not isinstance(hinge_findings, list) or not hinge_findings:
    raise SystemExit(1)
required_hinge_strings = ("location", "concern", "verified_safe_by")
if any(not isinstance(finding, dict) or any(
        not isinstance(finding.get(field), str) or not finding[field].strip()
        for field in required_hinge_strings) for finding in hinge_findings):
    raise SystemExit(1)
issues = verdict.get("issues_found_and_fixed")
if not isinstance(issues, list):
    raise SystemExit(1)
required_issue_strings = ("severity", "location", "issue", "fix_applied")
if any(not isinstance(issue, dict) or any(
        not isinstance(issue.get(field), str) or not issue[field].strip()
        for field in required_issue_strings)
        or issue["severity"].lower() not in {"critical", "high", "medium", "low"}
        for issue in issues):
    raise SystemExit(1)
if not isinstance(verdict.get("confidence_basis"), str) or not verdict["confidence_basis"].strip():
    raise SystemExit(1)
finished_at = verdict.get("finished_at")
if not isinstance(finished_at, str) or not finished_at.strip():
    raise SystemExit(1)
try:
    finished_time = datetime.fromisoformat(finished_at.replace("Z", "+00:00"))
except ValueError:
    raise SystemExit(1)
if finished_time.tzinfo is None:
    raise SystemExit(1)
hinge = sys.argv[4]
if not any(
    isinstance(finding, dict)
    and isinstance(finding.get("location"), str)
    and (
        hinge in finding["location"]
        or finding["location"] in hinge
    )
    for finding in hinge_findings
):
    raise SystemExit(1)
score = verdict.get("score")
if isinstance(score, bool) or not isinstance(score, (int, float)):
    raise SystemExit(1)
score = float(score)
if not math.isfinite(score) or not 0 <= score <= 100:
    raise SystemExit(1)
serialized = json.dumps(verdict, ensure_ascii=False).casefold()
for phrase in ("looks correct", "appears to work", "should be fine", "no obvious issues"):
    if phrase in serialized:
        raise SystemExit(1)
print(score)
PY
    )" || {
        echo "AUDIT-RUNNER: verdict.json is invalid or does not match this audit contract" >&2
        exit 2
    }
    echo "[$(date -Iseconds)] FINALIZE score=$FINAL_SCORE threshold=$THRESHOLD" >> "$LOG"
    if python3 - "$FINAL_SCORE" "$THRESHOLD" <<'PY'
import sys
raise SystemExit(0 if float(sys.argv[1]) >= float(sys.argv[2]) else 1)
PY
    then
        echo "AUDIT-RUNNER $CANONICAL_AUDIT: PASS score=$FINAL_SCORE threshold=$THRESHOLD"
        exit 0
    fi
    echo "AUDIT-RUNNER $CANONICAL_AUDIT: NEEDS WORK score=$FINAL_SCORE threshold=$THRESHOLD"
    exit 1
fi

# ─────────────────────────────────────────────────────────────────
# Phase 1 — Programmatic gather
# ─────────────────────────────────────────────────────────────────
RAW_TMP="$(mktemp -d "$ARTIFACTS/.raw.XXXXXX")" || {
    echo "AUDIT-RUNNER: cannot create private raw-evidence staging directory" >&2
    exit 2
}
GATHER_SCRIPT="$OMEGA_ROOT/lib/audit-gather/${AUDIT}.sh"
if [ ! -x "$GATHER_SCRIPT" ]; then
    GATHER_SCRIPT="$OMEGA_ROOT/lib/audit-gather/${CANONICAL_AUDIT}.sh"
fi
if [ -x "$GATHER_SCRIPT" ]; then
    echo "[$(date -Iseconds)] PHASE 1 gather: $GATHER_SCRIPT" >> "$LOG"
    "$GATHER_SCRIPT" "$RAW_TMP" "$PROJECT_PATH" "$FILES" "$URL" 2>&1 | tee -a "$LOG"
    GATHER_RC="${PIPESTATUS[0]}"
    echo "[$(date -Iseconds)] PHASE 1 done rc=$GATHER_RC" >> "$LOG"
    if [ "$GATHER_RC" -ne 0 ]; then
        echo "AUDIT-RUNNER: gather failed with exit code $GATHER_RC" >&2
        exit 2
    fi
    GATHER_MODE="programmatic"
else
    echo "[$(date -Iseconds)] PHASE 1 LLM-only: no gather script" >> "$LOG"
    GATHER_MODE="llm-only"
fi

# ─────────────────────────────────────────────────────────────────
# Phase 2 — Build evidence-summary.json from raw/
# ─────────────────────────────────────────────────────────────────
SUMMARY_SCRIPT="$OMEGA_ROOT/lib/audit-gather/${AUDIT}-summarize.py"
if [ ! -x "$SUMMARY_SCRIPT" ]; then
    SUMMARY_SCRIPT="$OMEGA_ROOT/lib/audit-gather/${CANONICAL_AUDIT}-summarize.py"
fi
SUMMARY_JSON="$ARTIFACTS/evidence-summary.json"
SUMMARY_TMP="$(mktemp "$ARTIFACTS/.evidence-summary.XXXXXX")" || {
    echo "AUDIT-RUNNER: cannot create private evidence-summary staging file" >&2
    exit 2
}
if [ -x "$SUMMARY_SCRIPT" ]; then
    if ! "$SUMMARY_SCRIPT" "$RAW_TMP" > "$SUMMARY_TMP" 2>>"$LOG"; then
        echo "[$(date -Iseconds)] PHASE 2 summarizer failed" >> "$LOG"
        echo "AUDIT-RUNNER: evidence summarizer failed" >&2
        exit 2
    fi
else
    if ! python3 - "$RAW_TMP" "$SUMMARY_TMP" <<'PY' 2>>"$LOG"
import json
import os
import stat
import sys

raw_dir, output = sys.argv[1:3]
tool_outputs = []
for name in sorted(os.listdir(raw_dir)):
    path = os.path.join(raw_dir, name)
    metadata = os.lstat(path)
    if stat.S_ISLNK(metadata.st_mode):
        raise SystemExit(1)
    if stat.S_ISREG(metadata.st_mode):
        tool_outputs.append({"file": name, "size": metadata.st_size})
with open(output, "w", encoding="utf-8") as handle:
    json.dump({"tool_outputs": tool_outputs}, handle, indent=2)
    handle.write("\n")
PY
    then
        echo "AUDIT-RUNNER: evidence summary fallback failed" >&2
        exit 2
    fi
fi

if ! python3 - "$SUMMARY_TMP" "$CANONICAL_AUDIT" "$USER_NEED" "$HINGE" \
    "$GATHER_MODE" "$REGISTRY" "$TICKET" "$FILES" "$URL" "$SCOPE" \
    "$FOCUS" "$SELECTOR" "$NO_FIX" <<'PY' 2>>"$LOG"
import json
import os
import sys

path, audit, user_need, hinge, mode, registry = sys.argv[1:7]
ticket, files, url, scope, focus, selector, no_fix = sys.argv[7:14]
with open(path, encoding="utf-8") as handle:
    evidence = json.load(handle)
if not isinstance(evidence, dict):
    raise SystemExit(1)
document = {
    "schema_version": 1,
    "audit": audit,
    "user_need": user_need,
    "hinge": hinge,
    "invocation": {
        "ticket": ticket,
        "files": files,
        "url": url,
        "scope": scope,
        "focus": focus,
        "selector": selector,
        "no_fix": no_fix == "1",
    },
    "provenance": {
        "runner": "omega-audit-runner",
        "gather_mode": mode,
        "registry": os.path.abspath(registry),
    },
    "evidence": evidence,
}
with open(path, "w", encoding="utf-8") as handle:
    json.dump(document, handle, indent=2)
    handle.write("\n")
PY
then
    echo "AUDIT-RUNNER: evidence-summary.json is invalid" >&2
    exit 2
fi
if [ -e "$ARTIFACTS/raw" ]; then
    RAW_HISTORY="$ARTIFACTS/raw-history"
    mkdir -p "$RAW_HISTORY" || exit 2
    RAW_ARCHIVE="$RAW_HISTORY/$(date -u +%Y%m%dT%H%M%S)-$$"
    if ! mv -- "$ARTIFACTS/raw" "$RAW_ARCHIVE"; then
        echo "AUDIT-RUNNER: cannot preserve the previous raw evidence" >&2
        exit 2
    fi
fi
if ! mv -- "$RAW_TMP" "$ARTIFACTS/raw"; then
    echo "AUDIT-RUNNER: cannot publish raw evidence" >&2
    exit 2
fi
RAW_TMP=""
if ! mv -f -- "$SUMMARY_TMP" "$SUMMARY_JSON"; then
    echo "AUDIT-RUNNER: cannot publish evidence-summary.json atomically" >&2
    exit 2
fi
SUMMARY_TMP=""

# ─────────────────────────────────────────────────────────────────
# Phase 3 — LLM is invoked by the audit skill itself.
# audit-runner.sh just guarantees the evidence is ready before the LLM
# starts reasoning. The skill .md template now reads $SUMMARY_JSON instead
# of doing manual greps.
# ─────────────────────────────────────────────────────────────────

echo ""
echo "─────────────────────────────────────────────────────"
echo " AUDIT-RUNNER ${CANONICAL_AUDIT}: evidence ready"
echo " Evidence: $PROJECT_PATH/$SUMMARY_JSON"
echo " Raw outputs: $PROJECT_PATH/$ARTIFACTS/raw/"
echo " LLM phase: produce verdict.json, then rerun with --finalize."
echo "─────────────────────────────────────────────────────"

exit 0
