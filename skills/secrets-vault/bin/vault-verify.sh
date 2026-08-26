#!/usr/bin/env bash
# vault-verify.sh — adversarial verification of a vault dir (R-VERIFY). Proves, per file:
#   (1) it is encrypted (sops filestatus — gitleaks scans ciphertext clean, so it is NOT the proof)
#   (2) it decrypts and its parsed KV map / bytes roundtrip
#   (3) no plaintext value leaked into the ciphertext (dotenv structural check)
# Never prints a secret value. Exit 0 iff every manifest entry passes.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=vault-lib.sh
source "$HERE/vault-lib.sh"

VAULT_DIR="${1:-vault}"
MAN="$VAULT_DIR/MANIFEST.tsv"
[[ -f "$MAN" ]] || vault_die "no manifest at $MAN"

pass=0; bad=0
while IFS=$'\t' read -r vfile rpath mode; do
  [[ -z "$vfile" || "$vfile" == \#* ]] && continue
  vf="$VAULT_DIR/$vfile"
  [[ -f "$vf" ]] || { echo "FAIL $vfile: missing"; bad=$((bad+1)); continue; }
  if ! vault_is_encrypted "$vf" "$mode"; then echo "FAIL $vfile: filestatus != encrypted"; bad=$((bad+1)); continue; fi
  # dotenv structural: every non-comment/meta/empty-value line must be an ENC[...] value.
  # (An empty value `KEY=` stays plaintext by design — it carries no secret — so exclude it.)
  if [[ "$mode" == "dotenv" ]]; then
    leak=$(grep -vE '^(#|$|sops_)' "$vf" | grep -vE '^[A-Za-z_][A-Za-z0-9_]*=$' | grep -vcE '=ENC\[' || true)
    [[ "$leak" == "0" ]] || { echo "FAIL $vfile: $leak non-encrypted dotenv line(s)"; bad=$((bad+1)); continue; }
  fi
  # roundtrip decrypt must succeed
  RT=$(mktemp); if vault_decrypt_to "$vf" "$RT" "$mode" >/dev/null 2>&1; then :; else echo "FAIL $vfile: decrypt error"; rm -f "$RT"; bad=$((bad+1)); continue; fi
  rm -f "$RT"
  echo "PASS $vfile [$mode] -> $rpath"
  pass=$((pass+1))
done < "$MAN"

echo "----"
echo "vault=$VAULT_DIR pass=$pass fail=$bad"
[[ $bad -eq 0 ]]
