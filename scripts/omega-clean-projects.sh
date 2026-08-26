#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — project hygiene cleaner
# ───────────────────────────────────────────────────────────────────────────
# Agents leave audit/test working dirs committed in project repos (.audit,
# .orchestrator, playwright-report, test-results, .playwright-mcp). Once an audit
# is done + fixed, those artifacts are dead weight. This untracks them, gitignores
# them (so they never come back), removes them locally, and makes ONE local commit
# per repo. It does NOT push (each client repo has its own auth — push when ready).
# Code, .md docs, .env/secrets are NEVER touched.
#
#   omega-clean-projects.sh            # DRY-RUN: list what would be cleaned
#   omega-clean-projects.sh --apply    # untrack + gitignore + rm + local commit
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
APPLY=0; [ "${1:-}" = "--apply" ] && APPLY=1
ROOT="${OMEGA_PROJECTS_ROOT:-$HOME/Station}"
# Artifact dirs to purge (agent/test working dirs — never source).
JUNK=(.audit .orchestrator playwright-report test-results .playwright-mcp .planner-cache)
GITIGNORE_BLOCK="# OmegaOS hygiene — agent/test artifacts (auto-added)
.audit/
.orchestrator/
playwright-report/
test-results/
.playwright-mcp/
.planner-cache/"

total_freed=0
mapfile -t repos < <(find "$ROOT" -maxdepth 3 -name .git -type d 2>/dev/null | sed 's#/.git$##' | sort)
for repo in "${repos[@]}"; do
    name="${repo#$ROOT/}"
    found=()
    for j in "${JUNK[@]}"; do
        # top-level only (avoid nested node_modules false-hits)
        [ -d "$repo/$j" ] && found+=("$j")
    done
    [ ${#found[@]} -eq 0 ] && continue
    sz=$(du -sc "${found[@]/#/$repo/}" 2>/dev/null | tail -1 | cut -f1)
    total_freed=$((total_freed + sz))
    echo "── $name  (${sz}K)  →  ${found[*]}"
    if [ "$APPLY" = "1" ]; then
        ( cd "$repo" || exit 0
          git rm -r --cached --quiet "${found[@]}" 2>/dev/null || true
          # ensure gitignore patterns present (idempotent)
          touch .gitignore
          grep -qF "# OmegaOS hygiene" .gitignore 2>/dev/null || printf '\n%s\n' "$GITIGNORE_BLOCK" >> .gitignore
          rm -rf "${found[@]}"
          git add .gitignore 2>/dev/null || true
          if ! git diff --cached --quiet 2>/dev/null; then
              git commit -q -m "chore: clean agent/test artifacts (.audit/.orchestrator/test-results) + gitignore" 2>/dev/null \
                  && echo "    ✓ committed (local — push when ready)" || echo "    ⚠ commit skipped"
          fi
        )
    fi
done

echo "──"
if [ "$APPLY" = "1" ]; then
    echo "[DONE] cleaned ~$((total_freed/1024))M across repos. Local commits made (NOT pushed)."
else
    echo "[DRY-RUN] ~$((total_freed/1024))M reclaimable. Run with --apply to clean + commit (no push)."
fi
