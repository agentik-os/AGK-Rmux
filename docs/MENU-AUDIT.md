# OmegaOS TUI menu audit — 2026-05-28

> **⚠️ HISTORICAL SNAPSHOT — frozen as of 2026-05-28, annotated 2026-06-10.**
> This is a dated agent audit report, kept for traceability. It does **not**
> describe the current TUI:
>
> - **Tab topology changed**: the TUI now has **5 tabs**
>   (`Sessions / Menu / Agentic / Settings / Help` — `app.rs` `Tab` enum).
>   The former *Monitor* tab is the Monitor group inside **Settings**; the
>   former *Projects* tab is the Projects group inside **Agentic**.
> - **The CRIT/HIGH findings below are FIXED** in the current tree, including:
>   protection-flag persistence across refresh (snapshot/restore in
>   `App::refresh`), the per-keystroke `providers.toml` reload
>   (`App::providers_cache`), the dead Projects actions (`d`/`p`/`Enter` are
>   wired), `refresh_projects()` call sites, the unreachable
>   `render_settings_detail` fallback, and Help scroll wiring.
> - The one finding that survived until June — the `column >= 30` mouse
>   hit-test heuristic (Sessions, LOW below) — was fixed on 2026-06-10:
>   `handle_mouse` now hit-tests against the rendered layout rects that
>   `draw_sessions` records each frame (the Menu tab's geometry-cache
>   pattern).
>
> Treat everything below as the state of 2026-05-28, not as an open bug list.

## Summary

The 7-tab TUI (`Sessions / Menu / Monitor / Projects / Settings / Agentic / Help`) is structurally
sound: a clean `Tab` enum, a coherent two-column "list ↔ detail" model with shared `Tab` /
`Tab-Tab` semantics, layered `Esc`, and consistent icon language (`★◆●⌂⚙§`). The two large
defects are (a) **persistence loss** — protection flags and registry caches are rebuilt from
scratch every refresh, so the lock state advertised by `.` is silently lost; and (b) **dead-end
UX** — `Projects` advertises three actions (`[d] [p] [Enter]`) but only the focus toggle works,
`Action::SendToSession` is defined but never produced, and an entire 175-line fallback render
branch in `render_settings_detail` is unreachable. Several smaller issues compound: `q` quits
even while typing into chat (no — verified, chat-focus routes first, OK), Settings reloads
`providers.toml` from disk on every keystroke, and `ui.rs` is at 2,387 lines (past the 1,500
refactor signal). Net assessment: well-architected skeleton, ~20 hours of polish away from a
shippable v1.

## Per-tab findings

### Sessions
| Severity | Issue | Fix |
|---|---|---|
| HIGH | `is_protected` is reset to `false` on every `refresh()` (app.rs:1001, 1072). The `.` key (input.rs:828) and `MenuAction::ToggleProtection` (input.rs:1107) flip a flag that the next 5-second refresh erases. The "session is protected" status message lies. | Persist protection in `~/.omega/state/protected.json` (HashSet<String>) or read `is_protected` from a stable store inside `flush_group_rows`. |
| HIGH | `§ Locked` indicator (ui.rs:524) therefore never appears after a refresh — observable bug, since refresh runs every 5s in the run loop. | Same fix as above. |
| MED | Comment at app.rs:996-1001 explicitly says "Master is NOT protected anymore — killing it triggers an auto-respawn." Yet `x` on Master will send the kill signal mid-respawn — race. | Either re-protect Master in the list OR confirm-modal before killing it. |
| MED | `Action::SendToSession` (input.rs:21) is defined and handled in main.rs:922 but **never produced** anywhere — `handle_key_chat` only emits `ForwardCharToSession` / `ForwardKeyToSession` / `SendTextRawToSession`. Pure dead code. | Delete the variant + its main.rs handler, OR wire it to a buffered "type a line then Enter to send" mode if intended. |
| MED | `enter_chat_focus()` (app.rs:739) is called once from main.rs:509 after session creation, but the legacy `toggle_session_focus()` (app.rs:776) is never called. | Remove `toggle_session_focus()` (orphan). |
| MED | `Enter` on a session header row (`SessionRow::Header`) does nothing visible — selection still points at a `SessionEntry` because `select_next/prev` ignore non-entries — but the user can't visually tell the header is unselectable. | Skip headers explicitly during `↑/↓` navigation (already implicit), and dim the header when cursor lands on the entry just after. |
| LOW | Title shows `Sessions (N)` (ui.rs:396) where N excludes the master… actually it includes master (it's pushed into `self.sessions`). Just confirm the count matches what the user sees. | Verify or document. |
| LOW | Mouse heuristic `column >= 30` (input.rs:90, 108) breaks if the terminal is narrower than ~120 cols (25% of 80 = 20, so click at col 25 falls in the wrong panel). | Pass actual `area` width via a shared layout cache, or store last-rendered split widths in `App`. |
| LOW | `Esc` from `Sessions::ChatFullscreen` returns to `List`, not to the prior `Chat` split state — minor friction for the user who just wanted to glance at the list. | Optional: remember previous focus and restore it. |

### Menu
| Severity | Issue | Fix |
|---|---|---|
| MED | `MenuAction::ToggleProtection` ("Toggle protection on selected") cycles a flag erased every refresh — same bug as Sessions. The menu entry feels broken because the indicator vanishes. | Same fix: persist protection store. |
| MED | The shortcut keys (`c C g p h G t d r . x q`) are listed in the menu (ui.rs:585) but **also work from every other tab** because `handle_key_normal` matches them globally (input.rs:754-810). This is great UX but undocumented. Status line in Sessions doesn't say "press c for Claude". | Document in help and in the Sessions bottom-bar. |
| LOW | `MenuAction::Refresh` (`r`) collides with `r/R = Rename selected session` (input.rs:813) — `r` from the Menu tab opens the rename modal on the currently selected Session row, not refresh. The menu label is misleading. | Make `r` context-aware: in Menu tab → `Refresh`; elsewhere → `Rename`. OR change Menu shortcut to `F5` only. |
| LOW | Menu uses grouped section headers but no border-style change for the focused list, making it hard to tell whether the cursor is on a header or an action. | Already-baked highlight covers this; verify visually. |

### Monitor
| Severity | Issue | Fix |
|---|---|---|
| HIGH | The tab claims at the bottom "This tab refreshes every 5s" (ui.rs:873) but **nothing in the TUI loop forces a Monitor-tab redraw at 5s** — the read is cheap because it just reads cached files, but the user only sees fresh data when *they* press a key. | Either re-render on the 5s refresh tick (already exists for sessions), OR rephrase the claim. |
| MED | `MonitorAction::RefreshBilling` (input.rs:751, 919) dispatches `Action::RefreshBilling` which is handled in main.rs:749 — verify it actually triggers `usage-monitor.sh` and updates `/tmp/aisb-usage.json` before the tab re-reads. If sync, OK; if async, user sees stale data for one cycle. | Add a "refreshing…" status_message until the script completes. |
| MED | `[T] Telegram setup` action on this tab opens the inline 3-step wizard (good). But if Telegram is **already configured**, this action still launches the wizard from scratch with no warning. | Detect existing config and either show a "reconfigure?" prompt or hide the action when configured (it's also redundant with Settings → Telegram). |
| LOW | Long content scrolls (auto-scroll logic at ui.rs:877-886 handles this), but Home/End jumps mid-scroll then have no obvious effect because the auto-scroll snaps back to keep the action visible. | Disable auto-scroll when user explicitly Home/End. |
| LOW | Hard-coded path `/tmp/aisb-usage.json` (ui.rs:693) is fragile if cron disabled or path changed. | Make the path a config field. |

### Projects
| Severity | Issue | Fix |
|---|---|---|
| CRIT | Detail panel claims `[d] Dispatch oracle    [p] Run planner    [Enter] Open in terminal` (ui.rs:1271) — **none of these are wired up on the Projects tab.** `Enter` only focuses the detail panel (input.rs:722-731). `d` invokes the global Dispatch (input.rs:791) but does NOT pre-fill the selected project name. `p` does nothing. | Wire context-aware Enter: when a project is selected, prompt to attach/open it. Wire `d` to pre-fill `DispatchProject` with `selected_project().name`. Wire `p` to run `omega planner` for the selected project. |
| HIGH | `App::refresh_projects()` (app.rs:696) exists but is **never called** anywhere. The registry is loaded once at TUI startup (app.rs:692). Any project added via `omega project add` in another shell will not appear until the TUI is restarted. | Call `refresh_projects()` whenever the user switches *to* the Projects tab, and on `F5`. |
| MED | When `project_registry.projects` is empty, the left list shows `(no projects registered)` but `projects_selected` is still 0 — and Enter still tries to focus the detail; detail shows the help blurb. Mostly OK, but feels unfinished — no "Press X to scan now" CTA. | Add an inline `[s] Scan ~/VibeCoding` shortcut when empty. |
| LOW | Tracker / Bootstrap data is re-read from disk on every keystroke / scroll (`PlanTracker::load`, `BootstrapState::load` in `render_project_detail`). Fine perf-wise but not cached. | Memoize for 1-2s if it shows in a profile. |
| LOW | Icon column uses `📁` emoji fallback (ui.rs:1036). Conflicts with the otherwise pure-ASCII icon convention (`★◆●⌂⚙`). | Pick one: emoji everywhere or ASCII everywhere. ASCII is safer for VT. |

### Settings
| Severity | Issue | Fix |
|---|---|---|
| CRIT | `render_settings_detail` (ui.rs:1389-1704) has a 184-line **unreachable fallback** after `return (lines, selected_line);` at line 1520 (the `#[allow(unreachable_code)]` is the smoking gun). This is dead code from an earlier refactor. | Delete lines 1521-1703 outright. |
| HIGH | `ProvidersConfig::load()` is called **on every keystroke** in Settings (input.rs:491, 564, 605, 681, 696 + ui.rs:1279). That's disk I/O + TOML parse per key. Will get visibly laggy if `~/.omega/providers.toml` lives on slow storage or is large. | Cache the loaded providers in `App`, invalidate only after `CommitSettingsEdit` / `ToggleSettingsBool`. |
| HIGH | `__INTERNAL_TELEGRAM_SETUP__` (app.rs:479, input.rs:706) — a "command" string that's actually a control signal. Brittle: if a real shell command ever matches this string it would mis-route. | Add a dedicated `SettingsField::Wizard(WizardKind)` variant instead of overloading `Action`. |
| MED | `Toggle` fields show `✓ on / ○ off` but the badge color and icon don't agree (Color::Red for the off circle is harsh). | Use `○` Gray for off, `●` Green for on — matches Sessions icon language. |
| MED | "Re-spawn Master AISB now" and "Kill Master AISB" (app.rs:445-455) have `confirm_first: true` but the field handler at input.rs:706 ignores the `confirm_first` flag — it dispatches immediately. The flag is dead. | Either honor it (extra Enter to confirm) or remove the flag. |
| MED | Settings tab has **no "save status" feedback** after `CommitSettingsEdit`. The user types a key, presses Enter, the modal closes — there's no "saved to providers.toml ✓" indicator. | Set `status_message` from main.rs:881 handler. |
| LOW | EditText field for API keys (`masked: true`) shows `(not set)` if empty — fine — but after editing, the redraw still shows the masked string from disk, not the value just typed. Minor confusion. | Refresh the cache immediately after commit. |
| LOW | The General section has Info-only fields for "Default AISB agent" and "Default model" but **no edit field** for them — user has to drop to CLI. | Add `EditText` for `general.aisb_agent`, `general.default_model`. |
| LOW | Section list and detail use `25/75` split, but Projects uses `30/70` and Info uses `25/75`. Slight inconsistency. | Standardize on 25/75 everywhere. |

### Agentic (Info)
| Severity | Issue | Fix |
|---|---|---|
| MED | The `[` and `]` keys for sub-section nav (input.rs:645-651) are documented nowhere — Help screen doesn't mention them, status bar doesn't either. | Add to Help. Or remove (since ↑/↓ already navigates sub-sections when list-focused). |
| MED | `select_info_next()` (app.rs:1110-1117) has a misleading `let _ = n;` discard and a comment about "virtually navigates sub-entries" that the function does NOT do — it just advances the section. | Remove the dead n calculation; comment is misleading. |
| MED | When detail-focused on AISB Agents, `↑/↓` navigates the 13-agent list (good). On other sections (Oracle/Workers/Rules) `↑/↓` scrolls the paragraph — fine, but the UX shift is silent. | Mention "↑/↓ to navigate agents" in the title when on AisbAgents. |
| LOW | The static `render_info_oracle/workers/rules` content (ui.rs:1930-2073) is hard-coded. Drifts from `~/.aisb/rules/`. | Load these from `omega-core` constants OR from disk at compile-time include_str!. |
| LOW | Tab label is "Agentic" but the enum variant is `Tab::Info` and InfoSection — semantic drift. | Rename enum to `Tab::Agentic` to match the UI. |

### Help
| Severity | Issue | Fix |
|---|---|---|
| MED | Help screen lists `r / R   Rename selected session` (ui.rs:2125) and separately `r   Refresh sessions` in the Menu section (ui.rs:2152). Same key, two meanings — confusing. | Reconcile: keep `F5` for Refresh in menu, document `r/R` as rename everywhere. |
| MED | Help omits: `[` `]` for Info sub-section nav, `Alt+↑/↓`, `Ctrl+W` / `Alt+Bksp` / `M-<` / `M->` chat shortcuts, mouse scroll. | Add a "Chat" sub-section. |
| MED | Help is static — no scrollbar/scroll handling (input.rs:172-176 sends scroll to `detail_scroll` but ui.rs:2225 passes `scroll((0, 0))`). On short terminals lines get clipped. | Wire `scroll((app.detail_scroll, 0))`. |
| LOW | "OmegaOS — Agentic Terminal Operating System" branding line is great; add the version number (`env!("CARGO_PKG_VERSION")`). | One-line addition. |

## Cross-cutting findings

| Severity | Issue | Where |
|---|---|---|
| HIGH | Status-bar prompt strip when in `InputMode != Normal` (ui.rs:2239) **hides system stats**. Fine, but the prompt is 1 line and never wraps — long inputs will visually overflow into the right column. | ui.rs:2290-2298 |
| HIGH | Icon usage IS consistent across tabs (✓): `★ Master`, `◆ Oracle`, `● Worker`, `⌂ Home`, `⚙ System`. But `📁` for Projects breaks the convention. | ui.rs:1036 |
| MED | `Action::SendToSession` is the lone orphan Action — declared in input.rs:21 + handled in main.rs:922 + never produced. | Already noted. |
| MED | App has orphan state-field cousins: `info_agent_selected` is only meaningful for AisbAgents but is reset on every section change (app.rs:1116). Fine. `chat_input: String` (app.rs:642) is **never read** anywhere — chat now forwards keys live, the buffer is dead. | Remove `chat_input` field + `App::handle_paste`'s `input_buffer` push (line 199 of input.rs is for *modals*, OK). |
| MED | Async refresh: `refresh_preview()` reads via `SessionManager::connect_cached()` (app.rs:952) — good (cached daemon socket). But `refresh()` (app.rs:970) calls `SessionManager::connect()` (uncached) every 5s. Inconsistent. | Use `connect_cached()` for both. |
| MED | Help line in the bottom bar is **not** dynamically updated per tab — it shows the last `status_message`, which is whatever the user last triggered. New users land on Sessions with an empty bar — no inline hint. | Compute a per-tab default `status_message` when entering a tab. |
| MED | Focused-panel high-contrast indication relies entirely on border color (Cyan vs Gray vs Yellow). Color-blind / monochrome accessibility is poor. | Add a `▶ FOCUSED` text marker in the focused panel's title. |
| LOW | Tab cycling on `prev_tab/next_tab` does NOT call `reset_2col_focus` from the BackTab arm in chat-focused mode (input.rs:471) — minor: when Shift+Tab from Sessions chat, focus state remains. | Add reset call. |
| LOW | The `centered_rect` helper produces popups with `Box::leak`-allocated string (ui.rs:104-108) — leaks one allocation per modal redraw on the Telegram step 3 case. | Pre-format into a Cow or store in `App`. |
| LOW | `ui.rs` is 2,387 lines — past the 1,500 refactor signal (rule omega-FILE-SIZE-LIMIT). Natural seams: `draw_settings`/`draw_info`/`draw_help`/modal helpers each ~400 lines. | Split into `ui/sessions.rs`, `ui/settings.rs`, `ui/info.rs`, `ui/help.rs`. |
| LOW | `main.rs` is also 2,638 lines — same rule. | Out of scope but flag. |

## Top 10 fixes (prioritized)

| Rank | Issue | Effort | Impact | File |
|---|---|---|---|---|
| 1 | Persist `is_protected` across refresh (Sessions, Menu lock toggle) | S | high | app.rs:1001, 1072 |
| 2 | Delete unreachable fallback in `render_settings_detail` (184 lines) | S | med | ui.rs:1521-1703 |
| 3 | Wire Projects tab actions: `d` pre-fills selected project, `Enter` attaches, `p` runs planner | M | high | input.rs:722, ui.rs:1271 |
| 4 | Call `refresh_projects()` on tab switch + F5 | S | high | input.rs (tab nav), app.rs:696 |
| 5 | Cache `ProvidersConfig` in `App`, invalidate on commit (kill per-keystroke disk I/O) | S | high | input.rs:491, 564, 605, 681, 696; ui.rs:1279 |
| 6 | Remove orphan `Action::SendToSession`, dead `toggle_session_focus`, dead `chat_input` | S | med | input.rs:21, app.rs:642, 776 |
| 7 | Reconcile `r` keybinding: F5 for Refresh, `r/R` for Rename only | S | med | input.rs:813, ui.rs:2152 |
| 8 | Use `connect_cached()` in `refresh()` too | S | med | app.rs:970 |
| 9 | Add per-tab default `status_message` and document `[` `]` in Help; mention chat shortcuts | S | med | input.rs (tab nav), ui.rs:2092-2235 |
| 10 | Split `ui.rs` into per-tab modules (>1500-line rule) | L | low (code health) | ui.rs whole |

## Quick wins (< 30 min each)

- Delete dead fallback in `render_settings_detail` (ui.rs:1521-1703).
- Delete `Action::SendToSession` variant + its main.rs handler.
- Delete `App::toggle_session_focus` (app.rs:776) and `chat_input` field (app.rs:642).
- Add `let mut providers_cache: Option<...>` to App, replace 6 `ProvidersConfig::load()` call sites.
- Switch `📁` to a `▣` ASCII icon in `render_project_detail`.
- Add `Help` scroll wiring: `scroll((app.detail_scroll, 0))` in ui.rs:2226.
- Add version line `concat!("v", env!("CARGO_PKG_VERSION"))` to Help.
- Add a `status_message` set in main.rs:881 (`CommitSettingsEdit`) so the user sees "Saved ✓".
- Add `setting_field.confirm_first` honouring OR remove the flag (currently dead).
- In `draw_tabs`, append a tiny tab counter (e.g. "3/7") for orientation.

## Open questions for the user

1. **Protection persistence** — should `is_protected` survive across rmux daemon restarts too (i.e. write to disk) or only across in-process refreshes (in-memory HashSet)?
2. **Projects actions** — confirm the intended semantics: should `Enter` *attach* to the project's Oracle session if alive, else just open a shell in the project dir? Should `p` open the planner TUI or just dispatch `omega planner`?
3. **AISB Master "kill triggers respawn"** (app.rs:998-1001) — is this still true post-2026-05-27 changes? If yes, document inline; if not, re-protect Master.
4. **Static Info text** (ui.rs:1930-2073) — should this come from `~/.aisb/rules/` at runtime, or stay compiled-in for offline correctness?
5. **`[` `]` keys** for Info sub-section nav — keep them (and document) or remove (since ↑/↓ already covers it when list-focused)?
6. **Monitor tab refresh cadence** — is "5s" the truth, or do we want push-on-cache-update? If the cron runs every minute, "5s" is misleading.
7. **`Tab::Info` vs label "Agentic"** — rename the enum to match the UI label?
