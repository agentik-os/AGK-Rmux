#!/usr/bin/env bash
# vault-restore.sh — THE RECOVERY COMMAND. Fresh clone + master key -> every plaintext
# secret back on disk at its real path. This is the operator's "lose laptop, clone, recover
# everything" button. Reads vault/MANIFEST.tsv (vault_file<TAB>restore_path<TAB>mode).
#
#   Usage:  SOPS_AGE_KEY_FILE=~/.omega/secrets/age/master.txt vault-restore.sh [--dry-run] [VAULT_DIR]
#   Default VAULT_DIR = ./vault . Recovery from omega-vault: run once per projects/<name>/ + core/.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=vault-lib.sh
source "$HERE/vault-lib.sh"

DRY=0; [[ "${1:-}" == "--dry-run" ]] && { DRY=1; shift; }
VAULT_DIR="${1:-vault}"
MAN="$VAULT_DIR/MANIFEST.tsv"
[[ -f "$MAN" ]] || vault_die "no manifest at $MAN — is this a vault dir?"
[[ -f "$VAULT_KEY_FILE" ]] || vault_die "master key not found at $VAULT_KEY_FILE. Restore it from your password manager / offline backup FIRST (see RECOVERY.md)."

ok=0; fail=0
while IFS=$'\t' read -r vfile rpath mode; do
  [[ -z "$vfile" || "$vfile" == \#* ]] && continue
  src="$VAULT_DIR/$vfile"
  # Expand a leading ~ in the restore path.
  dest="${rpath/#\~/$HOME}"
  if [[ ! -f "$src" ]]; then echo "MISS  $vfile (listed in manifest, absent on disk)"; fail=$((fail+1)); continue; fi
  if [[ $DRY -eq 1 ]]; then echo "PLAN  $vfile -> $dest [$mode]"; ok=$((ok+1)); continue; fi
  if vault_decrypt_to "$src" "$dest" "$mode"; then echo "OK    $dest [$mode]"; ok=$((ok+1));
  else echo "FAIL  $vfile -> $dest"; fail=$((fail+1)); fi
done < "$MAN"

echo "----"
echo "restored=$ok failed=$fail  (dry-run=$DRY)"
[[ $fail -eq 0 ]] || vault_die "$fail file(s) failed to restore"
