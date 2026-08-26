#!/usr/bin/env bash
# vault-lib.sh — shared helpers for the OmegaOS secrets-vault (SOPS + age).
# Pure bash + sops + age. NO node/bun/python on the recovery path (R-STACK bootstrap
# exception, council-mandated: the disaster-recovery path must run on a bare box with
# only two static binaries). R-SECRETS-VAULT is the doctrine this implements.
set -euo pipefail

VAULT_KEY_FILE="${SOPS_AGE_KEY_FILE:-$HOME/.omega/secrets/age/master.txt}"

vault_die() { echo "vault: $*" >&2; exit 1; }

# Resolve the age recipient (public key) from the master key file, unless one is passed.
vault_recipient() {
  if [[ -n "${VAULT_RECIPIENT:-}" ]]; then echo "$VAULT_RECIPIENT"; return; fi
  [[ -f "$VAULT_KEY_FILE" ]] || vault_die "master key not found at $VAULT_KEY_FILE (set SOPS_AGE_KEY_FILE)"
  age-keygen -y "$VAULT_KEY_FILE" 2>/dev/null || grep -oE 'age1[a-z0-9]+' "$VAULT_KEY_FILE" | head -1
}

# Detect the SOPS store mode for a source file. Council-verified routing:
#   multiline value (PEM/JWT/inline JSON) HARD-FAILS dotenv  -> binary
#   .json -> json ; .toml/*.git-credentials/npmrc/.sh/.pem   -> binary
#   plain KEY=VALUE .env                                     -> dotenv
vault_mode() {
  local f="$1" base; base="$(basename "$f")"
  case "$base" in
    *.json) echo json; return;;
    *.toml|*.git-credentials|npmrc|.npmrc|*.pem|*.sh|*.key) echo binary; return;;
  esac
  # An unbalanced number of double-quotes on a KEY="... line signals a multiline value.
  if grep -qE '^[A-Za-z_][A-Za-z0-9_]*="[^"]*$' "$f" 2>/dev/null; then echo binary; return; fi
  # A value containing a literal backslash-n or an inline PEM header -> binary.
  if grep -qE 'BEGIN [A-Z ]*PRIVATE KEY|BEGIN CERTIFICATE' "$f" 2>/dev/null; then echo binary; return; fi
  # Any non-comment/non-blank content line that is NOT KEY=VALUE (e.g. a bare
  # credential URL, an INI [section], a PEM body) breaks the dotenv parser -> binary.
  if grep -vE '^[[:space:]]*(#|$)' "$f" 2>/dev/null | grep -qvE '^[A-Za-z_][A-Za-z0-9_]*='; then echo binary; return; fi
  echo dotenv
}

# Encrypt SRC -> DST(vault ciphertext) with the right store for the format.
vault_encrypt() {
  local src="$1" dst="$2" recip mode
  recip="$(vault_recipient)"
  mode="$(vault_mode "$src")"
  mkdir -p "$(dirname "$dst")"
  # --config /dev/null: the wrapper is the sole authority on the recipient (--age).
  # Without it, a repo .sops.yaml whose creation_rules match on the INPUT path (the
  # plaintext file, not vault/) makes `sops -e` fail with "no matching creation rules".
  case "$mode" in
    dotenv) sops -e --config /dev/null --age "$recip" --input-type dotenv --output-type dotenv "$src" > "$dst";;
    json)   sops -e --config /dev/null --age "$recip" --input-type json   --output-type json   "$src" > "$dst";;
    binary) sops -e --config /dev/null --age "$recip" --input-type binary --output-type binary "$src" > "$dst";;
  esac
  echo "$mode"
}

# Decrypt a vault file to stdout (mode from MANIFEST or re-detected by extension).
vault_decrypt_to() {
  local vf="$1" out="$2" mode="${3:-}"
  [[ -f "$VAULT_KEY_FILE" ]] || vault_die "master key not found at $VAULT_KEY_FILE"
  if [[ -z "$mode" ]]; then
    case "$vf" in *.json) mode=json;; *.env) mode=dotenv;; *) mode=binary;; esac
  fi
  mkdir -p "$(dirname "$out")"
  SOPS_AGE_KEY_FILE="$VAULT_KEY_FILE" sops -d --config /dev/null --input-type "$mode" --output-type "$mode" "$vf" > "$out"
  chmod 600 "$out"   # council: sops -d inherits umask (0644); creds must be 0600
}

# Encryption gate (council-verified): gitleaks scans ciphertext CLEAN, so filestatus is
# the REAL proof a vault file is encrypted. filestatus guesses the store from the file
# extension, so a mode MUST be passed for dotenv/binary vault files (they have no telltale
# extension) or it parses them as json and errors. Returns 0 iff encrypted:true.
vault_is_encrypted() {
  local vf="$1" mode="${2:-}"
  if [[ -z "$mode" ]]; then case "$vf" in *.json) mode=json;; *.env) mode=dotenv;; *) mode=binary;; esac; fi
  sops filestatus --input-type "$mode" "$vf" 2>/dev/null | grep -q '"encrypted":true'
}

# Parsed KEY=VALUE map of a dotenv-ish file (sorted) — the correct roundtrip metric,
# because a dotenv roundtrip is NOT byte-identical (comments/blank lines shift).
vault_kv() { grep -E '^[A-Za-z_][A-Za-z0-9_]*=' "$1" 2>/dev/null | sort; }
