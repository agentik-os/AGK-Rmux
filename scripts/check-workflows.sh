#!/usr/bin/env bash
# Syntax gate for saved workflow graphs in .claude/workflows/.
#
# A workflow script is NOT a plain ES module: its body runs inside an async
# function (top-level `await` and top-level `return` are both legal, and the
# orchestration hooks — agent/parallel/pipeline/phase/log/args/budget — are
# injected globals). So we wrap the source exactly the way the runtime does
# before asking node to parse it.
#
# This proves the script PARSES. It does not prove it behaves — that needs a
# real run (L1).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/.claude/workflows"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

if ! command -v node >/dev/null 2>&1; then
  echo "check-workflows: node not found — skipping syntax gate" >&2
  exit 0
fi

fail=0
count=0
for f in "$DIR"/*.js; do
  [ -e "$f" ] || continue
  count=$((count + 1))
  name="$(basename "$f" .js)"
  out="$TMP/$name.mjs"
  {
    echo "async function __wf(agent, parallel, pipeline, log, phase, workflow, args, budget) {"
    # `export const meta` is a module-level export in the real file; inside the
    # wrapper it is just a const.
    sed 's/^export const meta/const meta/' "$f"
    echo "}"
    echo "void __wf;"
  } >"$out"

  if node --check "$out" 2>"$TMP/$name.err"; then
    echo "  ok   $name"
  else
    echo "  FAIL $name"
    sed 's/^/       /' "$TMP/$name.err"
    fail=1
  fi

  # meta must be a literal the runtime can read before executing anything.
  if ! grep -q "^export const meta = {" "$f"; then
    echo "  FAIL $name — must start with 'export const meta = {'"
    fail=1
  fi
  for field in "name:" "description:"; do
    if ! grep -q "  $field" "$f"; then
      echo "  FAIL $name — meta is missing $field"
      fail=1
    fi
  done
  # These throw at runtime inside a workflow script (they would break resume).
  if grep -nE 'Date\.now\(\)|Math\.random\(\)|new Date\(\)' "$f" >/dev/null; then
    echo "  FAIL $name — uses Date.now()/Math.random()/new Date(), which throw in a workflow"
    fail=1
  fi
done

if [ "$count" -eq 0 ]; then
  echo "check-workflows: no scripts in $DIR"
  exit 0
fi

if [ "$fail" -ne 0 ]; then
  echo "check-workflows: FAILED"
  exit 1
fi
echo "check-workflows: $count workflow(s) OK"
