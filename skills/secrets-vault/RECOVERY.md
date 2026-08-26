# RECOVERY RUNBOOK — "I lost my computer, get all my keys back"

This is the disaster-recovery ceremony for the OmegaOS secret vault. Follow it top to bottom on a
clean machine. It assumes only that you can install two static binaries and reach GitHub.

## The recovery kit (store OFFLINE, independent of the laptop) — DO THIS BEFORE YOU NEED IT

The vault is necessary but **not sufficient**. The real single point of failure is not the vault —
it is getting back into **GitHub** and your **password manager** after losing the device that may
hold your only 2FA seed. Keep all of the following OFFLINE (password manager + a paper/steel/USB
copy in a safe), never only on the laptop:

1. **The master age key** — the contents of `~/.omega/secrets/age/master.txt` (starts `AGE-SECRET-KEY-1...`).
   Without it, the vault is inert noise. **This is the one thing to guard.**
2. **GitHub 2FA recovery codes** — you cannot `git clone` the private vault if you are locked out of GitHub.
3. **Password-manager recovery kit** — so you can open the manager on a new device.
4. **A bootstrap clone token** (a GitHub PAT with `repo` scope) — because the token that clones
   `omega-vault` must NOT live only inside `omega-vault` (chicken-and-egg).
5. Optional insurance: an **off-GitHub `git bundle`** of omega-vault (`git bundle create omega-vault.bundle --all`)
   on the same USB, in case the GitHub account itself is locked/suspended.

## Recovery steps (clean machine)

```bash
# 1. Install the two static binaries (no node/bun/python needed).
#    sops: https://github.com/getsops/sops/releases   age: https://github.com/FiloSottile/age/releases
#    Verify the checksum, drop both on PATH (~/.local/bin).

# 2. Restore the master key from your offline backup.
mkdir -p ~/.omega/secrets/age && chmod 700 ~/.omega/secrets/age
#   paste the AGE-SECRET-KEY-1... line into:
$EDITOR ~/.omega/secrets/age/master.txt
chmod 600 ~/.omega/secrets/age/master.txt
export SOPS_AGE_KEY_FILE=~/.omega/secrets/age/master.txt

# 3. Clone the SSOT (using the bootstrap token from your kit).
git clone https://github.com/agentik-os/omega-vault.git
cd omega-vault

# 4. DRY-RUN first, then restore every project + the core store.
for d in projects/*/vault core/vault; do
  [ -d "$d" ] || continue
  bin/vault-restore.sh --dry-run "$d"     # shows what lands where
done
# looks right? drop --dry-run:
for d in projects/*/vault core/vault; do
  [ -d "$d" ] || continue
  bin/vault-restore.sh "$d"
done
```

Every secret is now back at its real path (chmod 600). Per-project repos with an in-repo `vault/`
can also self-restore: `git clone <project> && cd <project> && <path-to>/vault-restore.sh vault`.

## Verify the recovery worked (L1 — runtime is the only truth)

Do not trust "it ran". Prove a real credential authenticates:

```bash
# e.g. a restored OpenAI/Stripe/Resend key actually works:
#   curl -s https://api.example.com/v1/me -H "Authorization: Bearer $KEY" | head
```

A key that restores but does not authenticate was **rotated after it was vaulted** — the vault went
stale. Re-vault from the new live value (see the incident/staleness note in SKILL.md).

## If the master key is ever exposed

1. **Rotate the underlying credentials** (the API keys themselves) — this is the real fix. Master-key
   rotation does NOT un-disclose anything already in git history; the old key decrypts old ciphertext
   forever.
2. Generate a new master key, `sops updatekeys` every vault file to the new recipient, commit.
3. Record the incident. If un-rotatable high-value secrets were exposed, treat as a serious incident.

## Test this before you rely on it

Run the ceremony on a throwaway VM/container at least once. An untested backup is not a backup.
