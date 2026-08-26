# Worker brief — fix2-esc (F-2 HIGH — operator override: fix the BEHAVIOR, not the copy)

Mission: Esc while chat-focused (Chat AND ChatFullscreen) returns to the session list — handled locally in the TUI, NEVER forwarded to the agent. ONE commit, explicit path, NO push.

## Defect
Three surfaces promise "Esc → list" (ui.rs:694 `"CHAT (Esc → list to manage)"`; Help ui.rs:2752 `Esc = Back`; ui.rs:2762-2765 `Esc = back … Same pattern on Sessions…`) but `handle_key_chat` FORWARDS Esc to the live agent (input.rs:1814) → interrupts a running Claude turn. The intended layered-back arm is dead code (input.rs:1524-1527) because the router (input.rs:828-835) short-circuits Sessions+Chat into `handle_key_chat` first. Operator decision ("fix le Esc critique"): make Esc actually return to the list. The existing hint strings already describe the target behavior → NO ui.rs changes needed.

## Exact change (`crates/omega-tui/src/input.rs` ONLY)
1. input.rs:1814 — replace `KeyCode::Esc => Action::ForwardKeyToSession { session, key: "Escape" },` with a local back arm:
   ```rust
   KeyCode::Esc => {
       // Esc = back to the session list — matches the title hint, the Help
       // tab, and the layered-Esc pattern on every other tab (F-2). NOT
       // forwarded: interrupting the agent stays available via Ctrl+C (C-c).
       app.session_focus = SessionFocus::List;
       app.status_message = Some("Focus: session list".to_string());
       Action::None
   }
   ```
   This match serves both Chat and ChatFullscreen; List restores the split layout exactly like Tab-from-fullscreen.
2. Module doc comment (input.rs:1613-1623): remove `Esc` from the forwarded enumeration (line ~1615) and add `Esc → back to session list` to the TUI-local list.
3. Delete the now-redundant dead inner arm input.rs:1524-1527 (the `matches!(… Chat | ChatFullscreen)` branch inside the layered-Esc handler) — keep the `else` quit path for Sessions+List Esc; add a one-line comment that chat Esc is handled in handle_key_chat.
Nothing else: cmd_capture's Esc-cancel (input.rs:1666-1669) stays; F-8 onboarding copy (input.rs:1077) is OUT OF SCOPE; Ctrl+X guard (F-12) OUT OF SCOPE.

## Runtime verification protocol (same harness style + SAFETY RULES as fix1 — only touch `a-fx2-*`/`fx2-*`; capture-verify the `▶` row before every chat keypress; cat -v target so nothing executes)
- BEFORE leg binary: `$HOME/.local/bin/omega` (has fix1, NOT fix2 — Esc still forwarded). AFTER leg: `target/release/omega` post-edit.
1. `rmux new-session -d -s a-fx2-target`; `rmux send-keys -t a-fx2-target 'cat -v' Enter`. TUI in `fx2-tui` (220x55).
2. Select `a-fx2-target` (capture-verify `▶`), Enter → title `CHAT`.
BEFORE leg:
3. Press Esc (rmux send-keys -t fx2-tui Escape), then Enter (flush). Capture `a-fx2-target` → `^[` PRESENT (Esc reached the pane); capture fx2-tui → title STILL `CHAT` (no back-nav). Record the `^[` count. Quit TUI.
AFTER leg (edit + `cargo build --release` first; keep the SAME a-fx2-target alive across legs so the ^[ count comparison is meaningful):
4. Fresh TUI on the new binary → chat focus → press Esc → capture fx2-tui: title `LIST  x:kill  .:lock  r:rename`. Capture `a-fx2-target`: `^[` count UNCHANGED from step 3.
5. Fullscreen variant: re-enter chat, drive to ChatFullscreen (Tab cycling — verify via the fullscreen title/footer marker), press Esc → capture: back to split with `LIST` title; `^[` count still unchanged.
6. Interrupt-path intact: re-enter chat on the target, press Ctrl+C → capture target: `cat -v` killed (prompt back / `^C` visible) → C-c still delivered. (Do this LAST — it kills cat.)
7. Cleanup `a-fx2-*` + `fx2-tui`.
Evidence → `audits/.fixes/evidence-fix2/` + `NOTES.md` mapping captures to steps.

## Done Criteria (each must hold)
- `cargo build --release`: 0 errors, no NEW warnings.
- BEFORE: `^[` delivered + title stayed CHAT. AFTER: Esc → `LIST` title from split AND fullscreen; `^[` count unchanged; Ctrl+C still delivered.
- Exactly ONE commit staging ONLY `crates/omega-tui/src/input.rs` (explicit path, never `-A`):
  `fix(tui): Esc in chat focus returns to the session list — local, never forwarded (F-2)`
  DO NOT push.
- Evidence pack complete.

## Verify Command
`cd /home/vibe/Station/SideBusiness/OmegaOS && cargo build --release 2>&1 | tail -3 && git log --oneline -1 && git show --stat HEAD | tail -5 && ls audits/.fixes/evidence-fix2/`

## Report
`omega done <your-session-name> done_clean "<file:lines + BEFORE/AFTER evidence + sha>"` — `pending`/`failed` honestly otherwise.
