# RESET-RECOVERY — after a full VPS reset

## 0. BACK UP FIRST (before you wipe anything)

A reset is irreversible. `install.sh` reproduces OmegaOS itself, but your
**secrets** (`~/.omega`) and any **unpushed project work** are NOT in the repo —
wipe them and they're gone. Two built-in commands make this safe:

```bash
omega doctor --pre-reset   # read-only: what you'd lose right now
                           #   → secrets present, memory size, crontab,
                           #     and which project repos have uncommitted/unpushed work
omega backup               # writes ~/omega-backup-<timestamp>.tgz  (just ~/.omega + crontab)
```

Then **copy the archive OFF the machine** and **push your projects to your own git**:

```bash
# from your LOCAL machine — pull the backup down:
scp -P <ssh-port> <user>@<host>:~/omega-backup-*.tgz ./
```

> `omega backup` bundles **only OmegaOS-owned state** (`~/.omega`). It never
> touches your project repositories — those live in your own GitHub. It only
> *reports* repos with unpushed work so you push them yourself. Add
> `--include-memory` to also archive the claude-mem store (large).

To restore after reinstall: `tar xzf omega-backup-*.tgz -C ~` (puts `~/.omega`
back), then re-clone your project repos from your git.

---

OmegaOS reinstalls with one command:

```bash
git clone https://github.com/agentik-os/OmegaOS && cd OmegaOS && ./install.sh
```

That **reproduces everything OmegaOS needs to function**: `omega` + `rmux` binaries
(verified release binaries when current, with a locked source-build fallback), agent prompts (`~/.omega/agents`), the Laws+Rules registry
(`omega rules export` → `~/.omega/rules` → `~/.claude/rules` symlinks), slash commands
(`omega-*`, `/dynamic`, audit stubs), skills (`audits`, `pdfgen`), docs, the
**persistent Telegram systemd service**, the **tracking + verify hooks**, a **SOUL.md**
identity baseline, **native billing** (`omega usage --check`), and the `omega patrol`/`usage` crons.

It does **NOT** restore secrets or external plugins (by design — secrets never live in the repo).
Do these **manually after install**:

## 🔑 Secrets to re-add
| What | How |
|---|---|
| Telegram bot | `OMEGA_TG_TOKEN=<BOT_TOKEN> omega telegram setup <CHAT_ID> --user-id <YOUR_USER_ID>` then `systemctl --user start omega-tg-bot.service` |
| Claude / Codex / Gemini auth | run each provider's login; then use `omega doctor --deep` (and `omega codex-reconcile --json` for Codex) to verify topology without printing secrets |
| Vercel tokens | recreate `~/.omega/config/vercel-tokens.json` → `{"teams":{"<orgId>":{"token":"…"}}}` |
| GitHub | `gh auth login` |
| Tella (if used) | `~/.omega/secrets/integrations.env` → `TELLA_API_KEY=…` (chmod 600) |

## 📦 External (reinstall separately — not part of OmegaOS core)
| What | How |
|---|---|
| claude-mem (memory system) | `/plugin marketplace add thedotmack/claude-mem` → `/plugin install claude-mem` |
| superpowers / stripe / frontend-design plugins | `/plugin marketplace add anthropics/claude-plugins-official` → `/plugin install …` |
| Personal global `~/.claude/CLAUDE.md` + agents | optional: re-clone `agentik-os/claude-config` into `~/.claude` |
| Personal `MEMORY.md` (user profile) | optional: re-clone agentik-monitor (kept out of the public repo for privacy) |
| Other projects (agentik-os-site, clients) | re-clone their repos; their crons live in those repos |

## ✅ Verify after install
```bash
omega rules list                       # 7 Laws + operational rules render
systemctl --user status omega-tg-bot.service # command bot service (once configured)
omega usage --check && cat ~/.omega/state/usage.json   # native billing
ls ~/.omega/hooks/ ~/.omega/SOUL.md    # hooks + identity present
./scripts/verify-install.sh            # INSTALL PARITY OK
```

## What is intentionally NOT restored (legacy, retired)
The `~/.aisb` bash/tmux orchestration layer (37 watchdog crons, dispatch-to-session, the
python aisb-bot) is **replaced** by omega/rmux + `omega patrol` + the systemd Telegram
service. A fresh install does not — and should not — recreate it.
