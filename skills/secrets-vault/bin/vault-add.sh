#!/usr/bin/env bash
# vault-add.sh — encrypt a plaintext secret file INTO a repo vault, append to MANIFEST,
# and prove the roundtrip (L1). Never prints a secret value. Idempotent per (vault_file).
#
#   Usage: vault-add.sh <SRC_PLAINTEXT> <VAULT_DIR> <RESTORE_PATH> [VAULT_NAME]
#     SRC_PLAINTEXT  the plaintext file to protect (stays where it is; NOT copied into repo)
#     VAULT_DIR      the repo's vault/ dir (e.g. ./vault or omega-vault/projects/foo/vault)
#     RESTORE_PATH   where restore should write it back (repo-relative or absolute ~/...)
#     VAULT_NAME     optional ciphertext filename (default derived from RESTORE_PATH)
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=vault-lib.sh
source "$HERE/vault-lib.sh"

SRC="${1:?src plaintext file}"; VAULT_DIR="${2:?vault dir}"; RPATH="${3:?restore path}"
[[ -f "$SRC" ]] || vault_die "source not found: $SRC"
KEYS=$(grep -cE '^[A-Za-z_][A-Za-z0-9_]*=.' "$SRC" 2>/dev/null); KEYS=${KEYS:-0}

# ciphertext filename: given, or restore-path slugged (/ -> __), .env preserved for dotenv
NAME="${4:-}"
if [[ -z "$NAME" ]]; then NAME="$(echo "$RPATH" | sed 's#^[~/]*##; s#/#__#g')"; fi
DST="$VAULT_DIR/$NAME"

mkdir -p "$VAULT_DIR"
MODE="$(vault_encrypt "$SRC" "$DST")"

# Encryption gate — the file must be provably ciphertext.
vault_is_encrypted "$DST" "$MODE" || { rm -f "$DST"; vault_die "filestatus says NOT encrypted: $DST (aborted)"; }

# Roundtrip proof: parsed KV map for dotenv/json, byte cmp for binary.
RT=$(mktemp); trap 'rm -f "$RT"' EXIT   # mktemp = unpredictable name, mode 0600 (no symlink race on plaintext)
vault_decrypt_to "$DST" "$RT" "$MODE" >/dev/null 2>&1 || vault_die "decrypt failed for $DST"
if [[ "$MODE" == "binary" ]]; then
  cmp -s "$SRC" "$RT" && PROOF="BYTE-OK" || vault_die "binary roundtrip mismatch: $DST"
else
  if diff <(vault_kv "$SRC") <(vault_kv "$RT") >/dev/null; then PROOF="KV-OK";
  else vault_die "KV roundtrip mismatch: $DST"; fi
fi

# Append to MANIFEST.tsv (vault_file<TAB>restore_path<TAB>mode), dedup by vault_file.
MAN="$VAULT_DIR/MANIFEST.tsv"
touch "$MAN"
grep -vP "^${NAME}\t" "$MAN" > "$MAN.tmp" 2>/dev/null || true
mv "$MAN.tmp" "$MAN"
printf '%s\t%s\t%s\n' "$NAME" "$RPATH" "$MODE" >> "$MAN"
sort -o "$MAN" "$MAN"

echo "ADDED $NAME  mode=$MODE keys=$KEYS proof=$PROOF -> restores to $RPATH"
