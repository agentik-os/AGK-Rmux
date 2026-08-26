# Provenance — reel-lint

Vendored from **github.com/smaroc/sophiene-claude-skills**, commit
`aa11daab6b7caca702b5570879ea8a218e69c944` (2026-07-12), public release.

Reviewed before install (R-REPO-INSTALL): `scripts/lint_script.py` (236 lines)
imports nothing beyond the python stdlib, opens no socket, reads no credential,
executes no subprocess. It reads a script file and prints a score.

Local edits vs upstream: `SKILL.md` now invokes the linter by ABSOLUTE path
(`python3 ~/.claude/skills/reel-lint/scripts/lint_script.py`). Upstream ships a
relative `python3 scripts/lint_script.py`, which never resolves when the skill is
invoked from a project directory.

Role in OmegaOS: the pre-filming quality gate of the Instagram reel loop.
A script ships to filming only at 🟢 (>= 85/100). Method:
`tools/marketing-machine/growth/IG-REEL-LOOP.md`.

Runtime-verified 2026-07-26: scored a test script 86/100, exit 0.

To refresh: re-clone upstream, diff, re-apply the absolute-path edit + this note.
