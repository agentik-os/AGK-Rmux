#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# omega-git-merge.sh — merge all per-worker omega/* branches back into the base
# ───────────────────────────────────────────────────────────────────────────
# The oracle runs this AFTER all workers are terminal + ground-truth verified.
# Merges every local omega/* branch into <base> (default: current branch), one by
# one. On a conflict it ABORTS that merge and reports it (a conflict is a real code
# issue for the oracle to resolve), leaving the tree clean and that branch+worktree
# untouched for resolution. On a clean merge it removes the worker's worktree and
# deletes the branch (tidy). Never force-pushes, never pushes at all — the ship step
# owns the remote.
#
#   omega-git-merge [base-branch] [dir]
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
dir="${2:-.}"; cd "$dir" || { echo "[ERR] bad dir: $dir" >&2; exit 1; }
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "[ERR] not a git repo" >&2; exit 1; }
base="${1:-$(git rev-parse --abbrev-ref HEAD)}"

# Refuse to merge onto a dirty tree (would entangle uncommitted work).
if [ -n "$(git status --porcelain)" ]; then
  echo "[ABORT] working tree is dirty — commit/stash before merging." >&2; exit 2
fi
git checkout "$base" >/dev/null 2>&1 || { echo "[ERR] cannot checkout base $base" >&2; exit 1; }

branches="$(git for-each-ref --format='%(refname:short)' refs/heads/omega/ 2>/dev/null)"
[ -n "$branches" ] || { echo "[OK] no omega/* worker branches to merge."; exit 0; }

# branch → worktree path (empty if the branch has no worktree).
# substr($0,10) = everything after "worktree " — $2 truncated paths at the
# first space, so cleanup could rm the wrong path.
wt_for() { git worktree list --porcelain 2>/dev/null | awk -v b="refs/heads/$1" 'BEGIN{p=""}/^worktree /{p=substr($0,10)}/^branch /{if($2==b){print p;exit}}'; }

merged=0; conflicts=""
while IFS= read -r b; do
  [ -n "$b" ] || continue
  wt="$(wt_for "$b")"
  if git merge --no-ff --no-edit "$b" >/dev/null 2>&1; then
    merged=$((merged+1)); echo "[OK] merged $b"
    # Tidy: drop the worker's worktree + branch now that it's integrated.
    [ -n "$wt" ] && git worktree remove --force "$wt" >/dev/null 2>&1 || true
    git branch -D "$b" >/dev/null 2>&1 || true
  else
    git merge --abort >/dev/null 2>&1 || true
    conflicts="${conflicts}${b}\n"
    echo "[CONFLICT] $b — needs manual resolution (branch + worktree${wt:+ $wt} left intact)"
  fi
done <<< "$branches"

git worktree prune >/dev/null 2>&1 || true
echo "──"
echo "[SUMMARY] merged=$merged onto $base"
if [ -n "$conflicts" ]; then
  echo -e "[CONFLICTS] resolve these branches then re-run:\n$conflicts" >&2
  exit 3
fi
echo "[DONE] all worker branches merged cleanly into $base (not pushed — ship step owns the remote)."
