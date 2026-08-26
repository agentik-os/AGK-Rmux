#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# hinge-analyzer.sh — Identify the load-bearing 10% of a change.
#
# Created 2026-05-08 (Opus 4.7 + max effort upgrade).
# Vendored into OmegaOS skills/audits/_shared/ for install parity
# (LAW 0): a fresh clone+install ships the analyzer with the audits,
# so no audit SKILL.md depends on a path outside the installed tree.
#
# Concept: in any code change, ~10% of the lines carry ~90% of the risk.
# These "hinge points" are typically:
#   • New control-flow branches (if/switch/early-return)
#   • Boundary conditions (null checks, length checks, bounds)
#   • Async/await additions or removals
#   • Try/catch additions/removals
#   • SQL/Convex/Prisma query mutations
#   • Auth/permission gates
#   • State setters in shared state (Zustand, Context, useReducer)
#   • Public API contract changes (exported function signatures)
#   • Crypto / token / secret handling
#   • Loop bounds and concurrency primitives
#
# This script analyzes `git diff` and emits a JSON list of hinge regions —
# {file, line_start, line_end, kind, why}. The audit-meta-protocol-v2 then
# requires every audit to apply 10x scrutiny to these regions specifically.
#
# Usage:
#   hinge-analyzer.sh [--ref=HEAD~1] [--project-path=.]
#
# Output (stdout): JSON
# {
#   "ref": "HEAD~1",
#   "files_count": 3,
#   "hinges": [
#     {"file":"src/auth/validate.ts", "line_start":42, "line_end":48,
#      "kind":"new_branch", "why":"new if-condition guarding token expiry"},
#     ...
#   ]
# }
#
# Exit codes:
#   0 = ok, JSON on stdout
#   1 = bad args / not in a git repo
#   2 = no hinges detected (unusual — pure docs/whitespace change)
# ═══════════════════════════════════════════════════════════════

set -euo pipefail

REF="HEAD~1"
PROJECT_PATH="."
TICKET=""

while [ $# -gt 0 ]; do
    case "$1" in
        --ref=*) REF="${1#*=}" ;;
        --project-path=*) PROJECT_PATH="${1#*=}" ;;
        --ticket=*) TICKET="${1#*=}" ;;
        --help|-h)
            sed -n '2,46p' "$0" | sed 's/^# *//'
            exit 0
            ;;
        *) echo "unknown arg: $1" >&2; exit 1 ;;
    esac
    shift
done

cd "$PROJECT_PATH" || { echo "ERROR: cannot cd to $PROJECT_PATH" >&2; exit 1; }
[ -d .git ] || { echo "ERROR: not a git repo: $PROJECT_PATH" >&2; exit 1; }

# Sanity-check the ref exists
if ! git rev-parse --verify "$REF" >/dev/null 2>&1; then
    echo "ERROR: ref '$REF' not found in $PROJECT_PATH" >&2
    exit 1
fi

# Run Python analyzer; it shells out to git diff itself (avoids heredoc/stdin
# collision and lets the script invoke git directly without quoting issues).
TICKET="$TICKET" REF="$REF" PROJECT_PATH="$PWD" python3 - <<'PYEOF'
import json, os, re, subprocess, sys

TICKET = os.environ.get("TICKET", "")
REF = os.environ.get("REF", "HEAD~1")
PROJECT_PATH = os.environ.get("PROJECT_PATH", ".")

# Capture the unified diff for code files only.
diff_proc = subprocess.run(
    ["git", "diff", f"{REF}..HEAD", "--unified=0", "--",
     "*.ts", "*.tsx", "*.js", "*.jsx", "*.py", "*.go", "*.rs", "*.sql"],
    cwd=PROJECT_PATH,
    capture_output=True, text=True, timeout=30,
)
DIFF = diff_proc.stdout

# Hinge pattern catalog. Each entry: (regex, kind, why_template).
# Patterns match ADDED lines (lines starting with '+') because hinges in v2
# are about NEW logic, not removed logic.
HINGE_PATTERNS = [
    (r"^\+\s*(?:if|else if|switch|case|while|for|do)\b", "new_branch",
     "New control-flow branch — boundary conditions live here"),
    (r"^\+\s*return\s+(?!.*function)", "new_early_return",
     "Early return changes function exit logic"),
    (r"^\+\s*throw\s+", "new_throw",
     "New throw — error propagation path mutated"),
    (r"^\+\s*try\s*{", "new_try", "Try block added — exception path mutated"),
    (r"^\+\s*catch\s*\(", "new_catch", "Catch added — error swallowing risk"),
    (r"^\+\s*await\s+", "new_await",
     "Await added — async ordering and error propagation changed"),
    (r"^\+\s*async\s+", "new_async", "New async function — caller chain affected"),
    (r"^\+\s*export\s+(?:default\s+)?(?:function|class|const|let|var|interface|type|enum)\b",
     "new_public_api", "New public export — caller surface widened"),
    (r"^\+\s*(?:export\s+)?(?:function|class|const|let|var)\s+(?:[A-Za-z_$][\w$]*)\s*\(",
     "new_function", "Function signature change risks all callers"),
    (r"\b(?:db|prisma|convex|ctx)\.(?:mutation|insert|update|delete|patch|replace)\b",
     "data_mutation", "Database write — data integrity hinge"),
    (r"^\+.*\b(?:JWT|jwt|token|password|secret|crypto|bcrypt|argon2|hash)\b",
     "security_critical", "Security-critical primitive touched"),
    (r"^\+.*\b(?:auth|permission|role|admin|isAdmin|requireAuth|currentUser)\b",
     "auth_gate", "Authorization gate touched"),
    (r"^\+.*\b(?:Math\.random|crypto\.random)\b", "randomness",
     "Randomness/non-determinism introduced"),
    (r"^\+.*\b(?:setTimeout|setInterval|requestAnimationFrame)\b",
     "timing", "Timing primitive — race condition risk"),
    (r"^\+.*\bnew\s+(?:Promise|Worker|Channel|EventEmitter)\b",
     "concurrency", "Concurrency primitive — race condition risk"),
    (r"^\+.*\b(?:useState|useReducer|createStore|atom|signal)\b",
     "shared_state", "Shared-state setter — cross-component effects"),
    (r"^\+.*\bprocess\.env\.[A-Z_]+", "env_var",
     "Env var read — config drift risk"),
    (r"^\+\s*(?:fetch|axios|http\.|https\.|request)\(",
     "network_io", "Network call — failure mode + retry semantics"),
    (r"^\+\s*window\.location\.|^\+\s*router\.push\(|^\+\s*redirect\(",
     "navigation", "Navigation/redirect — flow integrity"),
    (r"^\+.*\bdangerouslySetInnerHTML\b|^\+.*\binnerHTML\b",
     "xss_surface", "innerHTML usage — XSS surface"),
]

hunks = []
current_file = None
hunk_old_start = hunk_new_start = 0
hunk_lines = []
in_hunk = False

def flush_hunk():
    if current_file and hunk_lines:
        hunks.append({
            "file": current_file,
            "new_start": hunk_new_start,
            "lines": list(hunk_lines),
        })

for raw in DIFF.splitlines():
    if raw.startswith("diff --git"):
        flush_hunk()
        current_file = None
        hunk_lines = []
        in_hunk = False
        m = re.match(r"diff --git a/(.+?) b/(.+)", raw)
        if m:
            current_file = m.group(2)
    elif raw.startswith("@@"):
        flush_hunk()
        hunk_lines = []
        m = re.match(r"@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@", raw)
        if m:
            hunk_new_start = int(m.group(1))
        in_hunk = True
    elif in_hunk and current_file:
        hunk_lines.append(raw)

flush_hunk()

# Score each hunk: count hinge-pattern matches, classify the strongest one.
hinges = []
for hunk in hunks:
    line_no = hunk["new_start"]
    detected_in_hunk = []
    for line in hunk["lines"]:
        if line.startswith("+") and not line.startswith("+++"):
            for pattern, kind, why in HINGE_PATTERNS:
                if re.search(pattern, line):
                    detected_in_hunk.append({
                        "file": hunk["file"],
                        "line": line_no,
                        "kind": kind,
                        "why": why,
                        "evidence": line.strip()[:200],
                    })
                    break
            line_no += 1
        elif line.startswith("+"):
            pass  # +++ header
        elif not line.startswith("-"):
            line_no += 1   # context line
        # '-' lines don't advance new line counter

    # Coalesce contiguous lines of same kind into a range
    if detected_in_hunk:
        for d in detected_in_hunk:
            hinges.append(d)

# Compress: same file + same kind on adjacent lines → range
compressed = []
hinges.sort(key=lambda x: (x["file"], x["line"]))
i = 0
while i < len(hinges):
    h = hinges[i]
    line_start = h["line"]
    line_end = h["line"]
    j = i + 1
    while j < len(hinges) and hinges[j]["file"] == h["file"] and hinges[j]["kind"] == h["kind"] and hinges[j]["line"] <= line_end + 5:
        line_end = hinges[j]["line"]
        j += 1
    compressed.append({
        "file": h["file"],
        "line_start": line_start,
        "line_end": line_end,
        "kind": h["kind"],
        "why": h["why"],
        "evidence_first_line": h["evidence"],
    })
    i = j

# Get total file count from diff
files_set = set(h["file"] for h in hunks)

result = {
    "ref": REF,
    "ticket": TICKET,
    "files_analyzed": len(files_set),
    "files_list": sorted(files_set),
    "hinges_count": len(compressed),
    "hinges": compressed,
}

if not compressed:
    # No hinges — likely pure docs/whitespace/comment change. Still emit JSON,
    # but exit code 2 to signal the oracle.
    print(json.dumps(result, indent=2))
    sys.exit(2)

print(json.dumps(result, indent=2))
PYEOF
