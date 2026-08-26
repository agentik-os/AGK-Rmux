# Evidence — fix1-misroute (F-1 CRITICAL)

Harness: 3 `cat -v` targets (`a-fx1-alpha/bravo/charlie`), TUI under test inside rmux session `fx1-tui` (220x55). Date: 2026-06-05 ~15:48–15:51.

## BEFORE leg — BIN_OLD = ~/.local/bin/omega (built 15:32, pre-fix) — defect MUST reproduce
| File | Step | Shows |
|---|---|---|
| `*-before-step1-selected-bravo.txt` | 1 | `▶ ⌂ a-fx1-bravo` selected in LIST |
| `*-before-step2-B4-in-bravo.txt` | 2 | `B4` echoed in bravo — routing correct pre-mutation |
| `*-before-step4-defect-selection-jumped-charlie.txt` | 3–4 | after `kill-session a-fx1-alpha` + 4s: title still CHAT, ▶ silently jumped to `a-fx1-charlie` |
| `*-before-step5-MISROUTE1-in-charlie.txt` | 5 | `MISROUTE1` PRESENT in charlie (wrong pane) |
| `*-before-step5-bravo-clean-no-misroute.txt` | 5 | `MISROUTE1` ABSENT from bravo (only `B4`) — **defect reproduced** |

## AFTER leg — BIN_NEW = target/release/omega (built 15:47, with fix)
| File | Step | Shows |
|---|---|---|
| `*-after-step8-reanchored-bravo.txt` | 7–8 | alpha killed, sessions 14→13, ▶ STILL `a-fx1-bravo` (re-anchored by name) — **PASS** |
| `*-after-step9-ROUTED_OK-in-bravo.txt` | 9 | `ROUTED_OK` present in bravo — **PASS** |
| `*-after-step9-charlie-clean.txt` | 9 | charlie has only the old `MISROUTE1`, no `ROUTED_OK` — **PASS** |
| `*-after-step10-vanish-dropped-to-LIST.txt` | 10 | bravo killed while chat-focused → title `LIST  x:kill  .:lock  r:rename` (focus dropped, NOT retargeted) — **PASS** |
| `*-after-step10b-down-moves-list-locally.txt` | 10 | Down moves ▶ in the list; charlie pane unchanged (keys local) — **PASS** |

Note: the `status_message` ("bravo ended — back to list") is set by the fix but not rendered on the Sessions tab — known F-7, out of scope per brief; the behavioral fix is the FOCUS DROP, which is proven above.

Fix: `crates/omega-tui/src/app.rs::refresh()` — name snapshot at top (next to the protection snapshot), re-anchor via `select_by_name()` after rebuild, vanish-while-chat-focused → `SessionFocus::List` + status message, length clamp kept as index fallback.
