# /codeaudit — Forensic Verdict: `/watch` vendored skill

**Scope:** mission diff `0cf3ab6..be31973` (5 commits, +2888 / −1 across 25 files)
**Target:** `skills/watch/**` (vendored video-analysis skill), `install.sh` watch block, `scripts/verify-install.sh` watch line, marketing insertions in `skills/ad-creative/SKILL.md` + `skills/social-content/SKILL.md` + `docs/marketing-machine-ssot.md`
**User need:** "the vendored `/watch` skill works as the canonical video-analysis primitive and is safely wired into install"
**Date:** 2026-07-03 · **Auditor:** `OmegaOS-worker-watch-codeaudit`
**Method:** Phase-0 programmatic gather + hinge (218 regions) → 6-track forensic Workflow fan-out → ≥1-lens adversarial verify per finding (REFUTE + REPRODUCE) → synthesis. 41 verification agents, 3.1M tokens.

---

## SCORE: 72 / 100 — Grade B (Good; real correctness & security-doc debt, golden path works)

**Verdict summary:** The skill **functions on its golden path** — `watch.py --help` exits 0, the vendored unit suite passes 7/7, `py_compile`/`bash -n` are clean, and a real captions-path E2E is shipped as evidence (`agentic/reports/watch-e2e-2026-07-03/`). Install wiring is **safe and correct** (repo-file copy only, no network/sudo/curl|sh at install time; `verify-install.sh` gates the right files). The vendored code has **no supply-chain red flags** (all URLs are Groq/OpenAI/YouTube; pure-stdlib HTTP; keys never logged).

But the diff ships **2 HIGH** and **~11 MEDIUM** confirmed defects: a wrong-video correctness bug, audio egress that contradicts the skill's own written security guarantees, zero subprocess timeouts, and a cookie-harvest helper that cannot run on a fresh install. None break the golden path, but several silently produce **wrong or misattributed output** — the worst failure mode for an analysis primitive.

- **Findings:** 31 confirmed, 4 falsified (killed by adversarial verify), 0 inconclusive.
- **Falsified (correctly not counted):** T6-01 (frontmatter routing — parser tolerates it), T4-02/T4-04 (playwright resolution — Bun auto-install is the guard), T3-04 (playwright missing — documented, not silent).

| Severity | Count | Weight in score |
|---|---|---|
| HIGH | 2 | dominant |
| MEDIUM | 11 | dominant |
| LOW | 14 | minor |
| INFO | 3 | none |

---

## HIGH — must fix before this is trusted as canonical

### H1 — Failed download silently analyzes the WRONG video
`skills/watch/scripts/download.py:89-94`
`subprocess.run` never gates on `result.returncode`; success is decided solely by `_pick_video()` finding any pre-existing `video.*` in the (reusable) `--out-dir`.
**Reproduce (verified):** stubbed `yt-dlp` returning exit 1 into a dir holding a prior run's artifacts → `download_url("…v=NEW_B", …)` returned `{"video_path":".../video.mp4","info":{"title":"OLD RUN-1 VIDEO"},"downloaded":true}` — success, stale content, no error.
**Scenario:** `/watch URL-A --out-dir ~/w` succeeds; later `/watch URL-B --out-dir ~/w` while URL-B is bot-checked/geo-blocked (the exact throttled-datacenter case SKILL.md documents). yt-dlp downloads nothing; `report.md` is written with `source: URL-B` but video A's frames, transcript, and title. SKILL.md Step 5 tells Claude to *leave* the workdir for follow-ups, making reuse likely.
**Fix:** clear/mkdtemp the download dir before yt-dlp, or require the picked video's mtime to be newer than run-start before applying the "video present = success" heuristic. The `download.py:87-88` comment only justifies ignoring exit codes for *subtitle* 429s, not for a zero-download run.

### H2 — Audio egresses to an *unintended provider* when a forced backend's key is missing
`skills/watch/scripts/hook.py:59` (+ `whisper.py:292` `transcribe_video`, invoked from `watch.py:160-170`)
`--whisper groq|openai` documents "Force a specific Whisper backend," but two paths ignore it: (a) the hook microscope re-runs an **unfiltered** `load_api_key()` when the forced backend's key is absent and uploads the first-10s audio to *whichever key it finds first*; (b) `transcribe_video` pairs a caller-forced backend string with the first-found key regardless of provider, so an **OpenAI key can be sent as a Bearer token to `api.groq.com`** (and vice-versa).
**Why it matters:** the user's explicit backend choice is a data-egress control. Overriding it silently sends their audio (and their API key) to a provider they deliberately excluded. Confirmed by both T2 and T5 tracks on independent reads.
**Fix:** in the hook path, pass the forced backend to `load_api_key(preferred=…)` and abort (not re-detect) if its key is missing; in `transcribe_video`, verify the resolved key's provider matches the forced backend before sending.

---

## MEDIUM — silent-wrong-output / resilience / doc-accuracy

- **M1 — No subprocess timeouts anywhere.** `download.py:89` (yt-dlp, also no `--socket-timeout`/`--retries`), `frames.py:62/172/239`, `hook.py:66`, `whisper.py:108` (ffmpeg) — 6 external-binary calls, `grep -c timeout` = 0 for all but the one `urlopen(timeout=300)`. A stalled/tarpitted YouTube or wedged ffmpeg hangs `/watch` forever, no ceiling (R-LOOP). *(T3-02/T3-03)*
- **M2 — Security section makes a false egress guarantee.** `SKILL.md:265` promises audio goes out "only when native captions are missing," but the hook microscope uploads first-10s audio on **every** captioned video ≥30s when a key exists (`watch.py:160-170` runs *before* the captions check at `:178`). Reproduced with a monkeypatched `analyse_hook` → 81 KB upload on a captioned clip. The same section self-contradicts at `SKILL.md:283`. Doc-only fix, but it is the exact "does NOT do" list an auditor trusts. *(T1-01/T2-02)*
- **M3 — Optional hook stage can destroy the core deliverable.** `hook.py:49` / `watch.py:160`: a failure in the *nice-to-have* hook-microscope frame extraction throws before the transcript and `report.md` are produced — the optional feature aborts the whole pipeline. *(T3-05)*
- **M4 — `IncompleteRead` crashes the pipeline post-download.** `whisper.py:188`: `response.read()` sits under handlers for `HTTPError` and `(URLError, TimeoutError, ConnectionResetError, OSError)`; `http.client.IncompleteRead` subclasses `HTTPException`, **not** `OSError` (verified: `issubclass(IncompleteRead, OSError)` → `False`). Callers catch only `SystemExit`, so a truncated body from Cloudflare-fronted Groq kills the run *after* download+frames succeeded, no `report.md`. The retry machinery built for this exact flake never fires. *(T3-02)*
- **M5 — Work dir never cleaned.** `watch.py:76`: no path deletes the working dir; every run leaves a ≤720p video + `audio.mp3` + `hook_audio.mp3` + ~100 JPEGs under the temp dir. On any mid-pipeline crash even the prose "delete when done" pointer (printed only at the end) never appears. Disk time bomb. *(T3-06)*
- **M6 — Cookie harvester depends on an unpinned auto-installed npm package.** `harvest-youtube-cookies.ts:9` imports `playwright-core` with no `package.json`/lockfile in `skills/watch`; it only runs via Bun's silent, network-fetching, unpinned auto-install. *(T1-02)*
- **M7 — Cookie harvester reports success on total failure.** `harvest-youtube-cookies.ts:38`: all navigation failures are swallowed as a "nav warning"; the script still writes the jar and prints a success line and exits 0 — a fully failed harvest (network down / bot wall / DNS) looks like success. *(T3-03)*
- **M8 — Headline feature silently skipped under 30s.** `watch.py:160`: the "0-10s hook microscope" is skipped for any video <30s — a threshold documented **nowhere** in SKILL.md. Short-form (the primary hook-analysis use case) silently gets no hook pass. *(T2-03)*
- **M9 — Vault-consent claim contradicted by the recipe.** `SKILL.md:270` claims "Does not write to the vault without explicit user consent at the Step 4.5 prompt," but Step 4.4 instructs writing `report.md` + hero frames into the vault and opening Obsidian *before* that prompt. *(T2-04)*
- **M10 — Server recipe runs unpinned third-party code.** `SKILL.md:292`: the "YouTube on servers" stack has an agent `git clone` a third-party repo at floating HEAD, run its `npm install` (arbitrary lifecycle scripts), and `pipx inject` it into yt-dlp — no commit pin, no integrity check. Documented recipe, not auto-run, but it is a supply-chain instruction shipped to every install (R-REPO-INSTALL glance applies to what the skill tells agents to execute). *(T4-01)*
- **M11 — Preflight fails a captions-only box.** `setup.py:212/219`: `--check` returns "ready" only if a Whisper key exists, though the skill contract (and setup.py's own template) declares the key **optional** and the captions path is key-free. A captions-only install fails preflight (exit 3) on every run. *(T2-07/T4-05)*

---

## LOW — cosmetic, doc drift, minor hygiene

- `SKILL.md:144` — "`--fps 3` → 3 fps (90 frames)" is impossible: fps is clamped to `MAX_FPS=2.0` and default `--max-frames`=80. Doc contradicts the code and itself. *(T1-04/T2-05/T5-05)*
- `pacing.py:82` — `motion_scores_from_frames()` is dead code; `watch.py` hardcodes `motion_scores=None`, so the "highest-motion shot" hero heuristic silently degenerates. *(T1-05)*
- `pacing.py:75` — `cuts_per_minute` is actually *shots*-per-minute (off-by-one: a 1-shot static video reports a nonzero cut rate). *(T2-06)*
- `download.py:77` — subtitle langs hardcoded English-only (`en,en-US,en-GB,en-orig`); the "captions-first (free, preferred)" path never fires for non-English video, silently falling through to paid Whisper egress. *(T1-06)*
- `frames.py:281` — unparseable ffmpeg `pts_time` lines are zero-padded, so unmatched frames get labeled `t=00:00` and pacing is corrupted, silently. *(T3-07/T5-06)*
- `setup.py:330` — any arg other than `--check`/`--json` runs the **full installer**; `setup.py --help` scaffolds `~/.config/watch/.env` and (macOS) runs `brew install`. *(T5-03)*
- `setup.py:232` — `SETUP_COMPLETE` is a dead flag: `cmd_check()` never reads it, and it's only written when a key exists. *(T1-03)*
- `watch.py:302` — when the Whisper fallback *ran and failed* (401/429/network), the user-facing diagnostic falsely blames "no API key set, or `--no-whisper` was used." *(T3-07 orig)*
- `report.py:181` — `report.md` written non-atomically (single `write_text`); a kill/OOM mid-write leaves a truncated file that Step 4.5 ingest would consume as complete. Relatedly `whisper.py:150` `MAX_429_RETRIES=2` permits only **one** 429 retry (`>= 2` check at `:199`). *(T3-08)*
- `harvest-youtube-cookies.ts:12` — `CHROME_EXE`/`PROFILE_DIR`/`COOKIES_OUT` dereferenced with non-null assertions, no validation → cryptic playwright `TypeError` instead of a usage message. *(T5-07/T6-03)*
- `harvest-youtube-cookies.ts` — jar written with default perms (0644/0664 here), inconsistent with the 0600 posture `setup.py` uses for its own secrets. *(T4-04-related, low)*
- `SKILL.md:296` — hardcodes maintainer home path `/home/vibe/.config/yt-dlp/youtube-cookies.txt` — the exact leak class `verify-install.sh` already guards for the linear skill; the new watch check doesn't. Confirmed present. *(T4-03/T6)*
- `scripts/verify-install.sh:167` — greps install.sh for `WATCH_SRC="$OMEGA_SRC/skills/watch"` with an unescaped mid-pattern `$` in a BRE; passes under GNU grep, but is a latent false-fail on stricter grep. *(T6-02)*
- `docs/marketing-machine-ssot.md` — one **added** line contains an em-dash (U+2014): `**24 canon skills — present in BOTH SSOTs…**`. R-NODASH applies to marketing-machine copy; verified via `git diff … | grep -P '[\x{2013}\x{2014}]'`. Replace with a comma. *(T6, self-verified)*

---

## INFO — noted, not scored

- `whisper.py:29` — model ids/endpoints hardcoded (`whisper-large-v3`, `whisper-1`); `setup.py` checks yt-dlp presence but never version. Rot-prone (Groq model retirement / stale yt-dlp breaks silently). *(T4-06)*
- `harvest-youtube-cookies.ts:22` — ships bot-detection-evasion (`--disable-blink-features=AutomationControlled` + pre-seeded SOCS/CONSENT cookies). Standard yt-dlp-ecosystem own-use practice, no third-party attack surface, but sits near the R-TRINITY "malicious detection-evasion" boundary — keep documented as own-use only. *(T4-07)*
- `install.sh:1168` — `cp -r "$WATCH_SRC"/*` copies untracked `__pycache__` on dev boxes (install-vs-repo drift noise); fresh clones unaffected (not in git). *(T6-03 orig)*

---

## What was checked and found CLEAN (Popper — falsification failed)

- **Install parity / L0:** `install.sh` watch block is `[[ -d ]]`-guarded, plain `cp -r` of repo-tracked files + heredoc command stubs; **no** network, sudo, curl|sh, or runtime-dep auto-install (boundary comment matches behavior). `bash -n` clean. `verify-install.sh:167` gates `SKILL.md` + `watch.py` + `LICENSE` + both wiring greps. A fresh `git clone && ./install.sh` reproduces a working `/watch` (captions path); `omega sync` prunes the bare stub but the `~/.claude/skills/watch` symlink keeps the trigger live (verified on this box). *(T6/T1)*
- **Supply chain:** all hosts in the changed code are `api.groq.com` / `api.openai.com` / YouTube; whisper.py is pure stdlib (no `pip install groq/openai`); no obfuscation, no credential exfiltration, no curl|sh in the vendored scripts. *(T4)*
- **Error hygiene:** zero bare `except:` and zero `except Exception: pass` in the changed `.py`; the broad handlers all log-and-degrade. Every ffmpeg/ffprobe/yt-dlp call is preceded by a `shutil.which` guard raising an actionable `SystemExit`. *(T3)*
- **`--no-whisper` zero-egress (commit be31973's fix):** `watch.py:163-164` gates `load_api_key`; `hook.py:57` gates key lookup, audio extraction, and the API call on `transcribe`. Claim holds in code. *(T3/T1)* — but note **no regression test pins it** (M-adjacent, see T5-04).
- **Phantom imports / paths / flags:** every import resolves (stdlib or sibling module); all 11 documented CLI flags exist in argparse and none is undocumented; CWD `.env` confirmed **out** of the key-lookup chain (commit 89bcc70 verified). *(T1)*
- **API-key hygiene:** grep of every `print`/stderr write — only backend names and file paths printed, never key values. *(T3/T4)*

---

## Test coverage gap (contributes to score, not a standalone finding)
`skills/watch/scripts/tests/` covers only `frames` scene-detection, `pacing`, and `report` (7 tests, all meaningful assertions). **Zero** coverage on the load-bearing modules: `watch.py` orchestrator, `download.py` (incl. the H1 stale-video path and the `is_url("-…")` injection guard), `whisper.py` fallback/backend selection (H2), and `hook.py`. Notably, **no regression test pins the be31973 `--no-whisper` fix** it was the whole point of the last commit. *(T5-04)*

---

## User-need verdict
> "works as the canonical video-analysis primitive and is safely wired into install"

**Partially met.** *Wired into install* — **yes**, safely and reproducibly (L0 holds). *Works as canonical primitive* — **on the golden captions path, yes** (proven E2E); but H1 (wrong-video), H2 (egress to unintended provider), M2 (false security doc), M3 (optional stage aborts core), and M8 (headline hook silently skipped <30s) mean it can silently produce **wrong or misattributed analysis** and **violates its own stated data-egress guarantees**. Fix the 2 HIGH + M1–M4 before treating its output as trustworthy without a second look.

*Read-only audit — no code edited, committed, or pushed. Findings ranked most-severe first; each carries a file:line citation and (for HIGH/MEDIUM) an adversarially-reproduced or code-quoted evidence chain.*
