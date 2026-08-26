#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# omega-git-branch.sh — per-worker git branch / worktree lifecycle (OmegaOS)
# ───────────────────────────────────────────────────────────────────────────
# Branch-per-worker isolation so 10–100 workers can edit overlapping files in
# parallel; the oracle merges them back with omega-git-merge.sh. Safe by design:
# never force-anything, never touches a remote, only local branch ops.
#
#   omega-git-branch create   <worker-name> [base-branch] [dir]
#       → creates + checks out omega/<worker-name>-<shortid> from base (default: current).
#         SHARED working dir — only safe for SEQUENTIAL workers (HEAD is global to the dir).
#   omega-git-branch worktree <worker-name> [base-branch] [repo-dir]
#       → creates omega/<worker-name>-<shortid> AND a dedicated git WORKTREE for it under
#         ~/.omega/worktrees/<repo>/<branch>, then prints that worktree's absolute path.
#         TRUE parallel isolation: each worker gets its own HEAD + working tree, so concurrent
#         workers never race. node_modules + .env* are symlinked in so builds/tests work.
#   omega-git-branch list   [dir]            → list local omega/* worker branches
#   omega-git-branch delete <branch> [dir]   → delete a local omega/* branch (+ its worktree)
# ═══════════════════════════════════════════════════════════════════════════
set -euo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
action="${1:-}"; [ -n "$action" ] || { echo "usage: omega-git-branch {create|worktree|list|delete} …" >&2; exit 1; }

# Derive a safe omega/<slug>-<sha> branch name, unique against existing refs.
_mk_branch() { # $1=worker-name
  local slug short branch n
  slug="$(printf '%s' "$1" | tr '[:upper:] ' '[:lower:]-' | tr -cd 'a-z0-9-' | sed 's/^-*//;s/-*$//')"
  short="$(git rev-parse --short=8 HEAD 2>/dev/null || echo 00000000)"
  branch="omega/${slug:-worker}-${short}"
  n=1; while git rev-parse --verify "$branch" >/dev/null 2>&1; do branch="omega/${slug:-worker}-${short}-${n}"; n=$((n+1)); done
  printf '%s' "$branch"
}
# Symlink the heavy untracked deps from the main checkout into a fresh worktree so
# builds/tests run there. node_modules is shared (read-mostly); a worker that mutates
# deps should not share — that's the rare exception. target/ is NOT shared (concurrent
# cargo builds would corrupt it) — Rust worktrees rebuild independently.
_link_deps() { # $1=main_toplevel  $2=worktree_dir
  [ -d "$1/node_modules" ] && [ ! -e "$2/node_modules" ] && ln -s "$1/node_modules" "$2/node_modules" 2>/dev/null || true
  local f
  for f in "$1"/.env "$1"/.env.* "$1"/.env.local; do
    [ -e "$f" ] && ln -sf "$f" "$2/$(basename "$f")" 2>/dev/null || true
  done
}

case "$action" in
  create)
    worker="${2:?worker-name required}"
    dir="${4:-.}"; cd "$dir"
    git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "[ERR] not a git repo: $dir" >&2; exit 1; }
    base="${3:-$(git rev-parse --abbrev-ref HEAD 2>/dev/null)}"
    branch="$(_mk_branch "$worker")"
    git checkout -b "$branch" "$base" >/dev/null 2>&1 || { echo "[ERR] could not branch from $base" >&2; exit 1; }
    echo "$branch"
    ;;
  worktree)
    worker="${2:?worker-name required}"
    repodir="${4:-.}"; cd "$repodir"
    git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "[ERR] not a git repo: $repodir" >&2; exit 1; }
    base="${3:-$(git rev-parse --abbrev-ref HEAD 2>/dev/null)}"
    main_top="$(git rev-parse --show-toplevel)"
    branch="$(_mk_branch "$worker")"
    wtroot="$OMEGA_DIR/worktrees/$(basename "$main_top")"
    mkdir -p "$wtroot"
    wt="$wtroot/${branch#omega/}"
    # Path must be free (branch is already unique; mirror that for the dir).
    n=1; while [ -e "$wt" ]; do wt="$wtroot/${branch#omega/}-${n}"; n=$((n+1)); done
    git worktree add -b "$branch" "$wt" "$base" >/dev/null 2>&1 \
      || { echo "[ERR] worktree add failed (branch=$branch base=$base)" >&2; exit 1; }
    _link_deps "$main_top" "$wt"
    echo "$wt"   # caller cd's here and runs the worker in full isolation
    ;;
  list)
    dir="${2:-.}"; cd "$dir"
    git for-each-ref --format='%(refname:short)' refs/heads/omega/ 2>/dev/null || true
    ;;
  delete)
    branch="${2:?branch required}"; dir="${3:-.}"; cd "$dir"
    case "$branch" in
      omega/*)
        # Remove its worktree first (if any), then the branch.
        wt="$(git worktree list --porcelain 2>/dev/null | awk -v b="refs/heads/$branch" 'BEGIN{p=""}/^worktree /{p=$2}/^branch /{if($2==b){print p;exit}}')"
        [ -n "$wt" ] && git worktree remove --force "$wt" >/dev/null 2>&1 || true
        git worktree prune >/dev/null 2>&1 || true
        git branch -D "$branch" >/dev/null 2>&1 && echo "[OK] deleted $branch${wt:+ (+ worktree)}" ;;
      *) echo "[ERR] refusing to delete non-omega branch: $branch" >&2; exit 1 ;;
    esac
    ;;
  *) echo "[ERR] unknown action: $action" >&2; exit 1 ;;
esac
