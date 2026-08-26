# Evidence — fix2-esc (F-2 HIGH): Esc in chat focus → session list, TUI-local

Date: 2026-06-05 · Harness: rmux sessions `a-fx2-target` (cat -v) + `fx2-tui` (220x55).
Same target pane kept alive across BOTH legs so the `^[` count is comparable.

## Change (crates/omega-tui/src/input.rs only)
- input.rs:1814-area — Esc arm in `handle_key_chat` forwarded-match replaced by a
  TUI-local back arm: `session_focus = List`, status "Focus: session list", `Action::None`.
  Serves both Chat and ChatFullscreen (same handler via router input.rs:828-835).
- Module doc (handle_key_chat): Esc removed from the forwarded enumeration, added to
  the TUI-local list (`Esc → back to session list (F-2; interrupt agent = Ctrl+C)`).
- Dead inner Esc arm (old input.rs:1524-1527, unreachable behind the router) deleted;
  else-quit path kept; one-line comment points to handle_key_chat.

## BEFORE leg — `~/.local/bin/omega` (fix1, NOT fix2)
| Step | Capture | Result |
|---|---|---|
| Esc in CHAT focus + Enter flush | `01-before-tui-title-after-esc.txt` | Title STILL `CHAT (Esc → list…` — no back-nav |
| Target pane | `02-before-target-esc-delivered.txt` | `^[` PRESENT — Esc forwarded to the agent pane. **Count: 2 lines** (echoed input + cat output) |

## AFTER leg — `target/release/omega` (fix2 build, 0 errors, 59.17s)
| Step | Capture | Result |
|---|---|---|
| Split: CHAT → Esc | `03-after-split-esc-to-list.txt` | Title `Sessions (6) — LIST  x:kill  .:…` — back to list |
| Target pane after split-Esc | `04-after-split-target-count-unchanged.txt` | `^[` count **unchanged: 2** — nothing forwarded |
| Fullscreen: Tab-Tab → `[FULLSCREEN — Tab-Tab to exit]` title → Esc | `05-after-fullscreen-esc-to-list.txt` | Split restored, title `LIST`; `^[` count still 2 |
| Interrupt path: re-enter CHAT → Ctrl+C | `06-after-ctrlc-cat-killed.txt` | `^C` visible, shell prompt returned — C-c still delivered |

Safety: only `a-fx2-*`/`fx2-tui` touched; `▶ ⌂ a-fx2-target` capture-verified before
every chat keypress; markers only into the cat -v pane.
