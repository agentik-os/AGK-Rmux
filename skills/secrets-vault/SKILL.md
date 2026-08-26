---
name: secrets-vault
description: >
  Encrypted-in-repo secret vault (SOPS + age) so a lost laptop never means lost keys: every
  project's secrets are committed to git as age CIPHERTEXT and a fresh `git clone` + ONE master
  key restores everything (`sops -d`). A repo leak yields only ciphertext. Implements the
  R-SECRETS-VAULT doctrine. Use when the user says "vault", "encrypt my secrets", "secret backup",
  "recover my keys", "commit my API keys", "sops", "age", "never lose a key", or in French
  "coffre-fort", "chiffrer mes secrets", "sauvegarder mes cles", "recuperer mes cles". NOT for
  live secret INJECTION into a running app (that stays a plaintext .env in ~/.omega); NOT for
  public repos (they NEVER carry ciphertext).
argument-hint: "init <repo> | add <src> <vault> <restore> | verify <vault> | restore [vault]"
allowed-tools: ["Bash", "Read", "Write"]
domain: safety
read_only: false
triggers: ["vault", "encrypt my secrets", "secret backup", "recover my keys", "commit my api keys", "sops", "age", "never lose a key", "coffre-fort", "chiffrer mes secrets", "recuperer mes cles"]
---

# secrets-vault — encrypted-in-repo secret recovery (SOPS + age)

The operator's hard goal: **lose the laptop, clone the GitHub repo, recover ALL API keys.** This
skill delivers exactly that, safely, under **R-SECRETS-VAULT** — a *narrowing* of L0/R-ENV, never
an exception:

- Plaintext secret values + the master private key **never** touch a repo. `~/.omega` stays the
  plaintext source of truth; the vault is its **encrypted recovery mirror**.
- Only **age ciphertext** whose plaintext is machine-verified absent is committable, and **only to
  a PRIVATE repo**. Public repos (OmegaOS, rmux) carry ciphertext **never** (harvest-now-decrypt-
  later + readable key names are a permanent leak).
- One master age keypair at `~/.omega/secrets/age/master.txt` (chmod 600). Its **public** recipient
  is committed in `.sops.yaml`.

## The two stores

| Store | What | Why |
|---|---|---|
| `agentik-os/omega-vault` (PRIVATE) | SSOT superset: `projects/<name>/vault/` + `core/vault/` | ONE clone recovers EVERYTHING. Holds partner-owned / no-remote / orphan / central `~/.omega` secrets. |
| in-repo `vault/` (operator-owned PRIVATE repos) | that project's own secrets, mirrored | a project clone is self-sufficient |

## Tools (`bin/`, pure bash + sops + age — no node/bun/python on the recovery path)

- `vault-add.sh <src> <vault_dir> <restore_path> [name]` — encrypt one file in, append MANIFEST,
  prove the roundtrip (KV map for dotenv/json, byte for binary), gate on `sops filestatus`.
- `vault-verify.sh <vault_dir>` — adversarial re-check: every file encrypted + decrypts + no leak.
- `vault-restore.sh [--dry-run] [vault_dir]` — **the recovery button.** Reads `MANIFEST.tsv`,
  `sops -d` every entry back to its real path, chmod 600.
- `vault-lib.sh` — shared helpers (format routing, filestatus gate, KV metric).

## Format routing (council-verified at runtime — do NOT assume dotenv everywhere)

- plain `KEY=VALUE` `.env` → **sops dotenv**
- multiline / PEM / JWT / inline-JSON value → **binary** (sops dotenv HARD-FAILS on multiline:
  `invalid dotenv input line`)
- `.json` → **json** (note: encrypts ALL string values, incl. non-secret names)
- `.toml` / `*.git-credentials` / `npmrc` / `.sh` / `.pem` → **binary** (no native TOML/INI type)

## Proof is runtime, not vibes (L1)

- A dotenv roundtrip is **NOT byte-identical** (comments become encrypted blobs, blank lines drop,
  empty values stay plaintext) — verify the **parsed KEY=VALUE map**, never `cmp`/sha the file.
- `gitleaks` scans SOPS ciphertext **CLEAN** — it is NOT the encryption proof. The proof is
  `sops filestatus <f>` == `{"encrypted":true}`.
- `sops -d` inherits umask (0644) — restore **chmods 0600** (git-credentials, npmrc especially).

## Custody (the one thing to guard)

The single master key honors the "one key" goal but is a total-compromise single point. Its backup
is **MANDATORY and TEST-verified**: operator **password manager** + an **OFFLINE** copy (paper/steel/
USB in a safe), stored beside the **GitHub 2FA recovery codes** and a **bootstrap clone token that
lives OUTSIDE omega-vault** (else recovery cannot bootstrap). **NEVER** put the master key in GitHub
Actions (that recouples key + ciphertext) — mint a scoped CI recipient if CI ever needs decrypt,
never the master. See `RECOVERY.md`.

## Incident policy (a plaintext secret already pushed)

Rotation is the **default** remediation: rotate the live credential → untrack + placeholder-scrub →
history-rewrite **only on operator sign-off** (force-push is R-COUNCIL). Ciphertext-in-history and
pushed-plaintext-in-history are both readable forever; master-key rotation does **not** remediate a
leaked value — only rotating the value does. Keep the vault in sync on every rotation (a stale vault
silently restores a dead key).

## v2 hardening (recorded, council dissent)

Passphrase-wrap (`age -p`) or yubikey-back the at-rest identity; split a recovery key (PM/offline
only) from per-project operational recipients for real cryptographic R-PROJ blast-radius containment.
`vault-lib.sh` already supports multiple recipients via `VAULT_RECIPIENT`.
