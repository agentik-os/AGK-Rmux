# Worker brief — fix1-misroute (F-1 CRITICAL, uiux+flow convergent, reproduced x3)

Mission: in `crates/omega-tui/src/app.rs::refresh()`, re-anchor the selected session by NAME across the rebuild; if the selected session vanished while chat-focused, drop to `SessionFocus::List` with a status notice. ONE commit, explicit path, NO push.

## Defect
`refresh()` (app.rs:1601) rebuilds `self.sessions` every ~2s; `self.selected` is only length-clamped (app.rs:1702-1704). Any session create/kill by another actor shifts indices → chat focus + the forwarded keystream silently retarget to whatever lands on the old index. The uiux audit reproduced this 3x live, including a garbled prompt submitted into a live bypass-permissions Claude session. Evidence: `audits/.uiuxaudit/evidence/l1-fresh-uiux-tui-misroute.txt`; verdicts `audits/.uiuxaudit/verdict.md` (F-1), `audits/.flowaudit/verdict.md` (F-02/FIX-002).

## Exact change (both halves already exist in-tree, unwired)
1. At the TOP of `refresh()`, next to the protection-flag snapshot (app.rs:1604-1609), snapshot the selected name:
   `let selected_name: Option<String> = self.sessions.get(self.selected).map(|e| e.session.name.clone());`
2. AFTER the rebuild + protection restore (after app.rs:1700), replace the bare clamp (app.rs:1702-1704) with:
   - re-anchor: if `selected_name` is Some and `self.select_by_name(&name)` (app.rs:1165) returns true → done.
   - vanished (or None): if `matches!(self.session_focus, SessionFocus::Chat | SessionFocus::ChatFullscreen)` → `self.session_focus = SessionFocus::List;` and `self.status_message = Some(format!("{name} ended — back to list"))` (note: status_message is currently suppressed on the Sessions tab — known F-7, OUT OF SCOPE — set it anyway; the FOCUS DROP is the behavioral fix). Then keep the existing length clamp as the index fallback.
3. Comment style: match the surrounding cause+why comments; cite F-1.
Surgical: app.rs ONLY. No other findings (F-3 enter_chat_focus, F-7 status bar, Esc behavior = separate missions). Esc handling must be byte-identical in this commit.

## Runtime verification protocol (MANDATORY — stty-style rmux captures)
SAFETY HARD RULES:
- Interact ONLY with sessions YOU create, all prefixed `a-fx1-` / `fx1-`.
- NEVER select, type into, kill, or resize any other session (oracle-*, claude-*, *-worker-*, master, uiux-*/flowwf-* leftovers).
- Before EVERY keystroke typed while chat-focused, `rmux capture-pane -p -t fx1-tui` and verify the `▶` row is an `a-fx1-*` session. If not — press Tab (back to list), stop, investigate.
- Markers are inert: every target runs `cat -v`, so typed bytes only echo; Enter only flushes a line; nothing executes.

Setup:
- `BIN_OLD=$HOME/.local/bin/omega` (pre-fix). Edit code, `cargo build --release`, `BIN_NEW=$PWD/target/release/omega`.
- Three targets (the `a-` prefix sorts them to the top rows of the Home section, adjacent alphabetically):
  `rmux new-session -d -s a-fx1-alpha` then `rmux send-keys -t a-fx1-alpha 'cat -v' Enter` (same for `a-fx1-bravo`, `a-fx1-charlie`).
- TUI under test runs INSIDE its own rmux session `fx1-tui` sized 220x55 (same harness as the audits — check `rmux new-session --help` for size flags or resize after create). Drive keys via `rmux send-keys -t fx1-tui …`, read via `rmux capture-pane -p -t fx1-tui`.

BEFORE leg ($BIN_OLD — harness-validity proof, the defect MUST reproduce):
1. Navigate selection to `a-fx1-bravo` (capture-verify `▶ ⌂ a-fx1-bravo`), Enter → title contains `CHAT`.
2. Type `B4` then Enter → capture `a-fx1-bravo`: `B4` present (routing correct pre-mutation).
3. From outside: `rmux kill-session -t a-fx1-alpha`. Wait 4s (refresh ~2s).
4. Capture fx1-tui → EXPECT defect: selection silently moved to `a-fx1-charlie`.
5. Type `MISROUTE1` then Enter → capture both targets: `MISROUTE1` PRESENT in charlie (wrong pane), ABSENT from bravo. Defect reproduced. Save captures.
6. Quit the TUI (Tab to list, then q). Recreate `a-fx1-alpha` + its `cat -v` for the AFTER leg.

AFTER leg ($BIN_NEW — fix proof):
7. Repeat steps 1-3 (fresh TUI on $BIN_NEW).
8. Capture fx1-tui → selection MUST still be `▶ ⌂ a-fx1-bravo` (re-anchored by name, row moved up one).
9. Type `ROUTED_OK` then Enter → present in BRAVO, absent from charlie.
10. Vanish case: while chat-focused on bravo, `rmux kill-session -t a-fx1-bravo`; wait 4s; capture fx1-tui → title shows `LIST  x:kill  .:lock  r:rename` (focus dropped to list — NOT silently retargeted). Press Down, capture → `▶` moved in the list (keys act locally).
11. Cleanup: kill all `a-fx1-*` + `fx1-tui`.

Save ALL captures timestamped to `audits/.fixes/evidence-fix1/` + `NOTES.md` mapping files to steps.

## Done Criteria (each must hold)
- `cargo build --release`: 0 errors, no NEW warnings.
- BEFORE leg reproduces the misroute (step 5) on $BIN_OLD.
- AFTER leg: steps 8, 9, 10 all pass on $BIN_NEW.
- Exactly ONE commit staging ONLY `crates/omega-tui/src/app.rs` (`git add crates/omega-tui/src/app.rs` — NEVER `-A`/`-u`; other sessions co-commit this repo):
  `fix(tui): re-anchor session selection by NAME across refresh — chat keystream never silently retargets (F-1 critical)`
  Body: cite the app.rs anchors + evidence dir. DO NOT push.
- Evidence pack complete.

## Verify Command (run before reporting)
`cd /home/vibe/Station/SideBusiness/OmegaOS && cargo build --release 2>&1 | tail -3 && git log --oneline -1 && git show --stat HEAD | tail -5 && ls audits/.fixes/evidence-fix1/`

## Report
`omega done <your-session-name> done_clean "<file:lines changed + BEFORE repro line + AFTER pass lines + commit sha>"` — use `pending`/`failed` honestly if any criterion is unmet.
