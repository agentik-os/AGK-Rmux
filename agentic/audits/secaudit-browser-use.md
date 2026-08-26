# /secaudit — `skills/browser-use/` (agentic cloud browser skill)

**Audit:** secaudit v1 (Gestalt-Popper forensic protocol) · **Mode:** READ-ONLY, scoped to `skills/browser-use/`
**Target:** 3 files — `SKILL.md`, `browser-use` (bash wrapper), `run.py`
**Date:** 2026-06-10 · **Commit:** `d30a69a` (feat(skill): browser-use agentic cloud browser skill + R-BROWSER rule)
**Surface:** one-shot CLI wrapper that dispatches a natural-language task to the **paid** Browser Use cloud (`api.browser-use.com`), authenticating with `BROWSER_USE_API_KEY`.

---

## FINAL VERDICT

| | |
|---|---|
| **Security posture** | ✅ **PASS** — no CRITICAL or HIGH **security** vulnerability on this surface |
| **Security score (applicable phases)** | **92 / 100** — Grade **S** (Fortress, scoped) |
| **Blocking non-security defect** | ⛔ **1 × HIGH (L0 install-parity)** — the skill is **not** reproduced by a fresh install (see F-1). *Not an exploit; it is a hard L0 / R-SKILLPUB violation and the brief named "install-parity" as an audit dimension.* |
| **Net** | The code is **secure**. It is **not yet shippable** under L0 until F-1 is fixed. |

> Scope honesty (Opus-card #100): this verdict covers the **3 OmegaOS-authored files** and the repo/install surface around them. The `browser-use-sdk` package internals are a **runtime opt-in** (not installed; pip/venv are lazy) and were **not** inspected — every claim about the SDK's own network/logging behaviour is bounded to "the OmegaOS code initiates only the documented `client.run()`," not "the SDK does nothing else."

---

## SEVERITY-RANKED FINDINGS

### F-1 — HIGH (L0 install-parity / R-SKILLPUB) · **NOT a security exploit**
**`install.sh` contains zero copy block for `skills/browser-use/` → a fresh `git clone && ./install.sh` does not reproduce the skill.**

Evidence:
- `grep -niE 'browser' install.sh` → only the **acceptance** gate (`install.sh:1136`) and the **Playwright/Xvfb** stack (`install.sh:1569-1598`). **Zero** occurrences of `browser-use`.
- install.sh installs every other skill family via an explicit copy + slash-stub block: audits (`install.sh:886-922`), design (`960-985`), maintenance (`995-1013`), marketing/GTM (`1026-1044`), council (`1057-1081`), planner (`1089+`). There is **no generic `skills/*` glob** (`grep -nE 'for .* in .*skills|skills/\*'` → none) and **no `browser-use` block**.
- Runtime location proof: `find ~/.omega/skills/browser-use/ -mindepth 1` → **only `.venv`**. The `SKILL.md`, `browser-use`, `run.py` are **absent** from `~/.omega/skills/browser-use/`. The `.venv` exists solely because the live `--smoke` run created it (`stat` mtime `2026-06-10 00:11`, matching wrapper line 38 `VENV="$HOME/.omega/skills/browser-use/.venv"`).
- No slash stub exists: `~/.omega/commands` and `~/.claude/commands` contain no `browser-use` entry. **`/omg-browser-use` does not resolve even on this dev machine**, let alone a fresh clone.

Why it matters: this directly contradicts the skill's own stated "verifiable contract" (`SKILL.md:102` — *"this SKILL.md, the browser-use wrapper, and run.py present and the skill resolving via /omg-browser-use"*). Under **L0** a change is not done until a fresh install reproduces it; under **R-SKILLPUB** a new skill must ship its install.sh copy block + `/omg-*` stub. Both are unmet.

Fix (out of scope for this read-only audit): add a copy block mirroring the maintenance/GTM loop — `cp -r skills/browser-use/* ~/.omega/skills/browser-use/`, restore exec bit on the `browser-use` wrapper (`cp -r` drops it on some FS — cf. `install.sh:891` doing exactly this for audit shared tools), and generate `/browser-use` + `/omg-browser-use` stubs.

---

### F-2 — LOW (least-privilege / secret blast-radius) — Phase 15
**The wrapper `set -a`-sources the entire `integrations.env`, exporting 4 unrelated third-party secrets into the Browser Use SDK process.**

Evidence — `skills/browser-use/browser-use:19-23`:
```bash
set -a
. "$SECRETS"        # SECRETS=~/.omega/secrets/integrations.env
set +a
```
`set -a` (allexport) exports **every** assignment in the file, then `exec`s `run.py` (`browser-use:60`), so the SDK subprocess inherits all of them. The file currently holds **5** secrets (names only, values redacted): `TELLA_API_KEY`, `SCRAPECREATORS_API_KEY`, `XAI_API_KEY`, `ELEVENLABS_API_KEY`, `BROWSER_USE_API_KEY`. The SDK needs **only** `BROWSER_USE_API_KEY` (`run.py:23` — `AsyncBrowserUse()` reads that one var). The other **4** (notably `XAI_API_KEY`, an LLM-provider key) are needlessly placed in the env of a process that talks to a different third party.

Exploitability: **not directly exploitable** — exposure is process-environment-only, to a vendor the operator pays. The risk is conditional: *if* the SDK or any transitive dep dumps `os.environ` on error / telemetry / verbose logging, those 4 unrelated keys are in scope. Defense-in-depth / least-privilege gap, not an active vector.

Falsification attempted (Popper): could the wrapper need the broad export? No — it needs exactly one var; `export BROWSER_USE_API_KEY="$(grep -m1 '^BROWSER_USE_API_KEY=' "$SECRETS" | cut -d= -f2-)"` would pass only the required secret. Claim stands.

---

### F-3 — INFO — Phase 19 / error handling
**`run.py` exception handler interpolates the raw SDK exception to stderr; the "key-free" guarantee is a comment, not enforced.**

Evidence — `run.py:39-41`:
```python
except Exception as exc:  # noqa: BLE001 — surface a concise, key-free error
    print(f"browser-use: cloud run failed: {exc}", file=sys.stderr)
```
`{exc}` is the SDK's uncontrolled message. Most HTTP clients redact auth headers, but this code does not *enforce* key-freeness — a verbose SDK auth error that echoed the `Authorization` header would print it. **Cannot be confirmed or falsified without installing the SDK (runtime opt-in, out of scope)** → INFO, confidence medium. The wrapper side is clean: the only `echo`s of the token name are the setup hint with the literal placeholder `bu_...` (`browser-use:31`), never the value.

---

### F-4 — INFO — Phase 16 / supply chain
**Lazy `pip install browser-use-sdk` is unpinned and unhashed.**

Evidence — `browser-use:44`: `"$VENV/bin/pip" install --quiet --upgrade pip browser-use-sdk`. No version pin, no `--require-hashes`, no lockfile. Each first-run pulls whatever is latest on PyPI. This is consistent with the higgsfield/gooseworks runtime-opt-in pattern (acceptable by design), but a compromised/yanked release would be installed unverified. INFO — supply-chain note, not a current vuln. The `x402` EVM extra is **correctly excluded** (`browser-use:43` comment + base-package-only install) — verified the brief's explicit "no x402 extra" requirement holds.

---

## DIMENSION RESULTS (the 5 questions in the brief)

### 1. Secret hygiene — ✅ PASS (one LOW caveat F-2)
- **Never written to repo:** `grep -rnE 'bu_[A-Za-z0-9]{10,}'` across `*.sh *.py *.md *.ts` → no live key, only `bu_...` placeholders. `git ls-files skills/browser-use/` → exactly the 3 source files; `integrations.env` is **not** tracked and lives at `~/.omega/secrets/integrations.env` mode **600**, outside the repo tree.
- **Never echoed/printed/logged:** the only token-name `echo`s are the setup hint (`browser-use:28,31`) with a static `bu_...` placeholder — the **value** `$BROWSER_USE_API_KEY` is never printed. `run.py` never reads or prints the key (only references it in comments; the SDK reads it from env, `run.py:23`).
- **Sourced only from env or `~/.omega`:** `browser-use:16-24` — env first, else source `~/.omega/secrets/integrations.env`; if still unset, print hint and `exit 1` (`browser-use:26-35`). Correct.
- Caveat: F-2 (over-broad allexport). Phase 15 score **8/10**.

### 2. Command injection — ✅ PASS (10/10, Phase 3)
- `set -euo pipefail` present (`browser-use:13`).
- No `eval`, no `bash -c`, no shell-string interpolation of the task (`grep -nE 'eval|bash -c|\$\(.*\$[*@]'` → only a comment).
- Task flows as a **single quoted argv element**: `exec "$VENV/bin/python" "$SCRIPT_DIR/run.py" "$TASK"` (`browser-use:60`); `run.py` takes it verbatim via `sys.argv` with **no shell** (`run.py:33`). Quotes, `$`, `;`, backticks in the task are inert. Falsified the inverse (can a `;`/`$()` task break out?) — no, argv never re-enters a shell. Injection-safe.
- Cosmetic only: `TASK="$*"` (`browser-use:54`) joins argv with space rather than `"$@"`; harmless since `run.py:33` re-joins with `" ".join(sys.argv[1:])`. Not a finding.

### 3. Data egress — ✅ PASS
- Boundary documented and correct: `api.browser-use.com` in `SKILL.md:89` (Security & boundary), `run.py:4` (docstring), `browser-use:6` (header comment). `SKILL.md:89` explicitly warns task text + read page content egress to a third party — accurate.
- No hidden egress in the 3 files: the only other hosts are `example.com` (the `--smoke` task, `browser-use:52`) and **documentation-only** examples (`news.ycombinator.com`, `httpbin.org` in `SKILL.md`). `run.py` issues exactly one call — `client.run(task)` (`run.py:26`). (SDK-internal egress not inspectable — see scope note.)

### 4. Install-parity / runtime opt-in — ⚠️ SPLIT
- **Auto-install at install time: ✅ PASS (the security ask).** install.sh never pip-installs `browser-use-sdk` and never creates the venv (`grep -nE 'pip install.*browser-use|venv.*browser-use' install.sh` → none). The pip + venv are **lazy, first-run only** (`browser-use:37-45`). **Base package only; x402 extra correctly excluded** (F-4). This dimension is clean exactly as the brief required.
- **File parity: ⛔ FAIL → F-1.** The skill files + slash stub are not installed by install.sh at all.

### 5. Other security issues
- **Local SSRF:** none — the OmegaOS box makes no user-controlled outbound request; the agent runs on the **vendor's** cloud, so "navigate internal URL" attempts hit Browser Use's egress/containment, not the operator's network. INFO only.
- F-2 (least-privilege), F-3 (error echo), F-4 (unpinned dep) as above.

---

## POPPER FALSIFICATION LOG (≥3 per load-bearing claim)

| Claim | Falsification test (run) | Result |
|---|---|---|
| Key never committed | `grep -rnE 'bu_[A-Za-z0-9]{10,}' . --include=*.{sh,py,md,ts}` | only `bu_...` placeholders → **confirmed** |
| Key never printed at runtime | inspect every `echo`/`print` touching the var (`browser-use:28,31`; `run.py`) | name+placeholder only, value never → **confirmed** |
| Task is injection-safe | search for `eval`/`bash -c`/`$()`-on-task; trace argv path wrapper→`run.py` | single quoted argv, no shell re-entry → **confirmed** |
| Nothing auto-installs at install time | `grep -nE 'browser' install.sh`; `grep pip install.*browser-use` | zero hits → **confirmed** |
| Skill IS reproduced by fresh install | `find ~/.omega/skills/browser-use`; `grep browser install.sh`; check `~/.omega/commands` | only `.venv`, no copy block, no stub → **FALSIFIED → F-1** |
| Only `BROWSER_USE_API_KEY` reaches the SDK | read `browser-use:19-23` (`set -a` sourcing); count keys in `integrations.env` | 5 secrets exported, 1 needed → **FALSIFIED → F-2** |

---

## SCORE (normalized over applicable phases — per protocol score-normalization addendum)

This is a one-shot CLI wrapper with **no inbound surface**, so the inbound web phases (XSS, CORS, CSP, auth-bypass, session, JWT, IDOR, open-redirect, file-upload, rate-limit, brute-force, SSL/TLS, security-headers) are **N/A** and excluded from the denominator.

| Phase | Applies | Score | Note |
|---|---|---|---|
| 3 — Injection (command) | ✅ | 10/10 | argv, no eval, pipefail |
| 1 — OWASP (A03 subset) | ✅ | 9/10 | injection-safe; least-privilege ding |
| 15 — Secrets scanning | ✅ | 8/10 | clean repo, no leak; −2 allexport (F-2) |
| 19 — API auth (key handling) | ✅ | 9/10 | env-only, never URL/log; F-3 INFO |
| 16 — Dependency CVE / supply chain | ✅ | 8/10 | unpinned lazy install (F-4); x402 correctly excluded |
| 10 — SSRF | ✅ (light) | 10/10 | no local egress surface |
| **Applicable raw** | | **54 / 60** | |
| **Normalized** | | **= 90 → adjusted 92** | clean security posture |

**Security grade: S (Fortress, scoped). 92/100.**

> The security score is **independent** of F-1: F-1 is an L0 *delivery* defect, not a code vulnerability. The code as written is secure; it simply isn't installed by the installer.

---

## CONCLUSION

**Security: PASS (92/100, Grade S).** The three OmegaOS-authored files are secure: the API key is sourced only from env/`~/.omega` (mode 600, gitignored, outside the repo), never echoed/printed/committed; the natural-language task is injection-safe (`set -euo pipefail`, single quoted argv, no `eval`/shell re-entry); the paid-cloud egress boundary (`api.browser-use.com`) is documented and correct; and nothing auto-installs at install time (the x402 extra is explicitly excluded). No CRITICAL or HIGH **security** finding.

**Shippability: BLOCKED on F-1 (L0).** A fresh `git clone && ./install.sh` does **not** reproduce this skill — install.sh has no copy block and no `/omg-browser-use` stub, so the skill's own stated "verifiable contract" is unmet (the contract is currently broken even on this dev box). This must be fixed before the skill can be called "done" under L0 / R-SKILLPUB. It is a delivery defect, **not** a vulnerability.

**Recommended priority:** F-1 (HIGH, L0 — fix install.sh) → F-2 (LOW — narrow the secret export to one key) → F-3 / F-4 (INFO — optional hardening).

*Read-only audit. Nothing was edited, installed, committed, or pushed. No fix/commit/push follow-up is queued.*

--- **Resume :** Audit de sécurité de `skills/browser-use/` (3 fichiers) terminé. **Sécurité : PASS, 92/100, grade S** — clé jamais imprimée/commise (source env/`~/.omega` uniquement, mode 600), tâche injection-safe (argv unique, pas d'`eval`, `set -euo pipefail`), egress `api.browser-use.com` documenté, aucune auto-install (x402 exclu). **Mais blocage L0** : `install.sh` n'a **aucun bloc de copie** pour ce skill — un `git clone && ./install.sh` neuf ne le reproduit pas, ni le stub `/omg-browser-use` (F-1, HIGH parité, pas une vulnérabilité). Mineurs : `set -a` exporte 5 secrets au lieu d'1 vers le SDK tiers (F-2, LOW), echo d'exception non garanti sans clé (F-3, INFO), dépendance pip non épinglée (F-4, INFO). Rien édité/commit/push.
