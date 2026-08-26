#!/usr/bin/env python3
"""Account Switcher — Multi Claude Max Account Management.

Supports:
- /account              → Status + switch buttons for all accounts
- /account code XXX     → Exchange OAuth code for fresh tokens
- /account check        → Check token validity
- /account refresh      → Try refresh current token
- /account switch NAME  → Switch to a saved account profile
- /account save         → Save current token to active profile
"""

import asyncio
import json
import os
import shutil
import subprocess
import time
from pathlib import Path

# FIX-CODEAUDIT-2026-05-08: removed unused `import httpx` (genuine dead import).
from aisb.config import (
    # FIX-CODEAUDIT-2026-05-08: removed unused BOT_DIR.
    OAUTH_SCRIPT, logger, Update, ContextTypes,
    InlineKeyboardButton, InlineKeyboardMarkup,
)
import aisb.state as _state
from aisb.formatting import escape_html
from aisb.auth import auth_required


# Helper for non-blocking subprocess in async context
async def _arun(*args, **kwargs):
    """Run subprocess.run in a thread to avoid blocking the event loop."""
    return await asyncio.to_thread(subprocess.run, *args, **kwargs)


# ============================================================
# Paths
# ============================================================

CREDENTIALS = Path.home() / ".claude" / ".credentials.json"
ACCOUNTS_DIR = Path.home() / ".claude" / "accounts"
ACCOUNTS_META = ACCOUNTS_DIR / "accounts-meta.json"


# ============================================================
# Account Profile Helpers
# ============================================================

def _load_meta() -> dict:
    """Load accounts metadata."""
    try:
        return json.loads(ACCOUNTS_META.read_text())
    except (OSError, json.JSONDecodeError) as e:
        # FIX-CODEAUDIT-F006 (2026-05-08): log corrupt/missing meta file so
        # missing accounts in the picker are diagnosable.
        logger.debug(f"_load_meta: returning empty meta ({e})")
        return {"accounts": {}, "active": None}


def _save_meta(meta: dict):
    """Save accounts metadata."""
    ACCOUNTS_META.write_text(json.dumps(meta, indent=2))


def _get_current_email() -> str:
    """Get email from claude auth status.

    FIX-NONETYPE-03 (2026-04-17): data.get('email', '?') returns None when
    the key IS present but the value is null (common after reconnect before
    Claude refreshes the profile). Coerce all falsy values to '?'.
    """
    try:
        result = subprocess.run(
            ["claude", "auth", "status"],
            capture_output=True, text=True, timeout=10,
        )
        data = json.loads(result.stdout.strip())
        email = data.get("email")
        return email if email else "?"
    except Exception:
        return "?"


def _get_token_info() -> dict:
    """Read current token info from credentials file."""
    try:
        creds = json.loads(CREDENTIALS.read_text())
        oauth = creds.get("claudeAiOauth", {})
        exp = oauth.get("expiresAt", 0)
        now = int(time.time() * 1000)
        remaining_min = (exp - now) // 60000
        return {
            "valid": now < exp,
            "remaining_min": remaining_min,
            "warning": 0 < remaining_min < 30,
            "tier": oauth.get("rateLimitTier", "?"),
            "sub": oauth.get("subscriptionType", "?"),
        }
    except (OSError, json.JSONDecodeError, KeyError, TypeError) as e:
        # FIX-CODEAUDIT-F006 (2026-05-08): credentials missing/corrupt → log so
        # auth troubleshooting has a trail.
        logger.debug(f"_get_token_info: returning invalid ({e})")
        return {"valid": False, "remaining_min": 0, "warning": False, "tier": "?", "sub": "?"}


def _detect_active_account() -> str | None:
    """Detect which saved account matches current credentials."""
    try:
        current = json.loads(CREDENTIALS.read_text())
        current_token = current.get("claudeAiOauth", {}).get("refreshToken", "")
        if not current_token:
            return None

        meta = _load_meta()
        for name, info in meta.get("accounts", {}).items():
            profile_path = ACCOUNTS_DIR / info.get("credential_file", "")
            if profile_path.exists():
                profile = json.loads(profile_path.read_text())
                if profile.get("claudeAiOauth", {}).get("refreshToken", "") == current_token:
                    return name
        return None
    except (OSError, json.JSONDecodeError, KeyError) as e:
        # FIX-CODEAUDIT-F006 (2026-05-08): explain "why account-X disappeared
        # from picker" cases instead of silent None.
        logger.debug(f"_detect_active_account: returning None ({e})")
        return None


def _save_current_to_profile(account_name: str):
    """Save current credentials to an account profile."""
    meta = _load_meta()
    acc = meta.get("accounts", {}).get(account_name)
    if not acc:
        return False

    profile_path = ACCOUNTS_DIR / acc["credential_file"]
    try:
        previous = meta.get("active") or "?"
        shutil.copy2(str(CREDENTIALS), str(profile_path))
        meta["active"] = account_name
        _save_meta(meta)
        # FIX-2026-05-12: log switch event when active account changes
        if previous != account_name:
            try:
                subprocess.run(
                    [str(Path.home() / ".aisb/lib/account-switch-log.sh"),
                     account_name, previous, "save_or_oauth"],
                    capture_output=True, timeout=5,
                )
            except Exception:
                pass
        return True
    except Exception as e:
        logger.error(f"Failed to save profile {account_name}: {e}")
        return False


def _switch_to_profile(account_name: str) -> dict:
    """Switch to a saved account profile.

    1. Copy profile credentials to active credentials
    2. Try to refresh the token
    3. Return result with status

    Returns: {"ok": bool, "method": "refresh"|"reauth_needed", ...}
    """
    meta = _load_meta()
    acc = meta.get("accounts", {}).get(account_name)
    if not acc:
        return {"ok": False, "error": f"Account '{account_name}' not found"}

    profile_path = ACCOUNTS_DIR / acc["credential_file"]
    if not profile_path.exists():
        return {"ok": False, "error": f"Credential file not found: {acc['credential_file']}"}

    # Backup current credentials
    if CREDENTIALS.exists():
        backup = CREDENTIALS.with_suffix(".json.previous")
        shutil.copy2(str(CREDENTIALS), str(backup))

    # Copy profile to active
    shutil.copy2(str(profile_path), str(CREDENTIALS))

    # Try refresh
    result = subprocess.run(
        [OAUTH_SCRIPT, "try-refresh"],
        capture_output=True, text=True, timeout=30,
    )
    try:
        data = json.loads(result.stdout.strip())
        if data.get("ok"):
            # Refresh worked! Save fresh token back to profile
            shutil.copy2(str(CREDENTIALS), str(profile_path))
            previous = meta.get("active") or "?"
            meta["active"] = account_name
            _save_meta(meta)
            # FIX-2026-05-12: log switch event for per-account usage tracking
            try:
                subprocess.run(
                    [str(Path.home() / ".aisb/lib/account-switch-log.sh"),
                     account_name, previous, "manual"],
                    capture_output=True, timeout=5,
                )
            except Exception:
                pass
            return {
                "ok": True,
                "method": "refresh",
                "expires_min": data.get("expires_min", "?"),
                "email": acc.get("email", "?"),
                "label": acc.get("label", account_name),
            }
    except Exception as e:
        logger.warning(f"account operation failed: {e}")

    # Refresh failed — need reauth for this account
    # Restore previous credentials so we don't break current session
    backup = CREDENTIALS.with_suffix(".json.previous")
    if backup.exists():
        shutil.copy2(str(backup), str(CREDENTIALS))

    return {
        "ok": False,
        "method": "reauth_needed",
        "email": acc.get("email", "?"),
        "label": acc.get("label", account_name),
        "error": "Refresh token expired — need full reauth",
    }


# ============================================================
# Reauth Request
# ============================================================

async def _request_reauth(app, reason: str = "Token expired", target_account: str = None):
    """Reauth using native Claude Code /login in a tmux session.

    Flow:
    1. Create tmux session 'aisb-reauth'
    2. Launch claude, run /login, select option 1
    3. Capture the auth URL from tmux pane
    4. Send URL to the operator on Telegram
    5. the operator clicks, gets code, pastes in Telegram DM
    6. Bot pastes code into the tmux session
    7. Auth done — kill session
    """

    # Cooldown — 30s between attempts (just prevent double-tap, not lockout)
    if time.time() - _state._reauth_cooldown < 30:
        logger.debug("Reauth request skipped (cooldown 30s)")
        return
    # FIX-CODEAUDIT-EC-1 (2026-05-08): TTL self-clear before checking the flag.
    # If _pending_reauth was set >5min ago and still True, a path silently failed
    # and never reset it. Treat as stale and clear so this attempt can proceed.
    if _state._pending_reauth and _state._pending_reauth_set_at > 0:
        _age = time.time() - _state._pending_reauth_set_at
        if _age > _state.PENDING_REAUTH_TTL_SEC:
            logger.warning(f"reauth: stale _pending_reauth flag ({_age:.0f}s old > {_state.PENDING_REAUTH_TTL_SEC}s TTL) — auto-clearing")
            _state._pending_reauth = False
            _state._pending_reauth_set_at = 0
    if _state._pending_reauth:
        logger.debug("Reauth request skipped (already pending)")
        return

    # FIX-2026-05-08: previous version aborted silently when an aisb-reauth
    # session already existed — but monitor.py:1199 + claude_runner.py:288 BOTH
    # auto-spawn this session on 401 errors, so user-initiated /account login
    # was perpetually blocked by stale auto-triggered sessions. Replace the
    # silent abort with: kill any existing session + reset stale pending state,
    # then proceed. The auto-trigger session was either successful (cleaned up
    # already) or stuck (and the user wants to take over anyway).
    _check_existing = await _arun(
        ["tmux", "has-session", "-t", "aisb-reauth"], capture_output=True, timeout=5,
    )
    if _check_existing.returncode == 0:
        logger.warning("reauth: pre-existing aisb-reauth session found — killing + taking over (user-initiated)")
        await _arun(["tmux", "kill-session", "-t", "aisb-reauth"], capture_output=True, timeout=10)
        await asyncio.sleep(0.5)
        # Also reset the in-memory pending flag so we don't trip the next guard
        _state._pending_reauth = False

    _state._pending_reauth = True
    _state._pending_reauth_set_at = time.time()  # FIX-CODEAUDIT-EC-1: TTL anchor
    _state._reauth_cooldown = time.time()
    if target_account:
        _state._reauth_target_account = target_account

    # FIX-013: persist pending state to disk
    try:
        import json as _json
        with open("/tmp/aisb-pending-reauth.json", "w") as f:
            _json.dump({"pending": True, "ts": time.time(), "target": target_account or ""}, f)
    except Exception as e:
        logger.warning(f"FIX-013 persist pending reauth failed: {e}")

    def _reset_pending():
        """Reset pending state on disk + memory — called on ANY failure."""
        _state._pending_reauth = False
        _state._pending_reauth_set_at = 0  # FIX-CODEAUDIT-EC-1: clear TTL anchor
        try:
            import json as _jj
            with open("/tmp/aisb-pending-reauth.json", "w") as _ff:
                _jj.dump({"pending": False}, _ff)
        except Exception:
            pass

    try:
        # 1. Create reauth tmux session
        await _arun(["tmux", "kill-session", "-t", "aisb-reauth"], capture_output=True, timeout=10)
        await asyncio.sleep(0.5)
        await _arun(["tmux", "new-session", "-d", "-s", "aisb-reauth", "-c", "/tmp"], capture_output=True, timeout=10)
        await asyncio.sleep(1)
        await _arun(["tmux", "send-keys", "-t", "aisb-reauth", "claude --dangerously-skip-permissions", "Enter"], capture_output=True, timeout=10)
        await asyncio.sleep(8)

        # 2. Run /login
        await _arun(["tmux", "send-keys", "-t", "aisb-reauth", "/login", "Enter"], capture_output=True, timeout=10)
        await asyncio.sleep(2)
        # Select option 1 (Claude account with subscription)
        await _arun(["tmux", "send-keys", "-t", "aisb-reauth", "Enter"], capture_output=True, timeout=10)
        await asyncio.sleep(5)

        # 3. Capture the auth URL from tmux pane
        capture = await _arun(
            ["tmux", "capture-pane", "-t", "aisb-reauth", "-p", "-S", "-25"],
            capture_output=True, text=True, timeout=5
        )
        output = capture.stdout if capture.stdout else ""

        # Extract URL
        import re
        lines = output.split("\n")
        url_parts = []
        in_url = False
        for line in lines:
            stripped = line.strip()
            if not in_url:
                if "https://claude.com/cai/oauth/authorize" in stripped:
                    in_url = True
                    url_parts.append(stripped)
            else:
                if not stripped or " " in stripped or stripped.startswith(("Paste", "Esc", "❯", "Browser")):
                    break
                url_parts.append(stripped)
        candidate = "".join(url_parts)
        url_match = re.match(r'(https://claude\.com/cai/oauth/authorize[A-Za-z0-9._~:/?#\[\]@!$&\'()*+,;=%-]+)', candidate)
        auth_url = url_match.group(1) if url_match else ""

        if not auth_url:
            logger.error(f"Failed to extract auth URL from /login output. Pane content: {output[-200:]}")
            _reset_pending()
            await _arun(["tmux", "kill-session", "-t", "aisb-reauth"], capture_output=True, timeout=10)
            # Tell user it failed
            from aisb.config import CHAT_ID
            if CHAT_ID and app and hasattr(app, 'bot'):
                try:
                    await app.bot.send_message(chat_id=CHAT_ID, text="❌ Auth link failed — URL not captured. Retry /billing or /account.", parse_mode="HTML")
                except Exception:
                    pass
            return

        # 4. Send URL to the operator on Telegram
        from aisb.config import CHAT_ID
        reauth_text = (
            f"<blockquote>"
            f"<b>🔐 Auth Required</b>  ·  <code>REAUTH</code>\n"
            f"━━━━━━━━━━━━━━━━━━━━\n\n"
            f"<b>Reason:</b> {escape_html(reason)}\n\n"
            f'1. <a href="{auth_url}">Clique pour autoriser</a>\n'
            f"2. Copie le code de la page\n"
            f"3. Colle-le ici (auto-detecte)\n\n"
            f"<i>⏰ Session reauth en attente</i>"
            f"</blockquote>"
        )

        if CHAT_ID and app and hasattr(app, 'bot'):
            await app.bot.send_message(
                chat_id=CHAT_ID,
                text=reauth_text,
                parse_mode="HTML",
                disable_web_page_preview=True,
            )
            logger.info(f"Reauth request sent (reason: {reason}, target: {target_account})")
        else:
            logger.warning("Cannot send reauth request: no app.bot available")
            _reset_pending()

    except Exception as e:
        logger.error(f"_request_reauth crashed: {e}")
        _reset_pending()
        await _arun(["tmux", "kill-session", "-t", "aisb-reauth"], capture_output=True, timeout=10)
        # Tell user
        from aisb.config import CHAT_ID
        if CHAT_ID and app and hasattr(app, 'bot'):
            try:
                await app.bot.send_message(chat_id=CHAT_ID, text=f"❌ Reauth error: {escape_html(str(e)[:200])}", parse_mode="HTML")
            except Exception:
                pass


# ============================================================
# Session Reconnection
# ============================================================

SESSION_RESUME = Path.home() / ".aisb/lib/session-resume.sh"
PROTECTED_SESSIONS = ("Home", "Home-2", "Home-3", "aisb-reauth", "aisb-usage-monitor")


STUCK_MARKERS = (
    "rate_limit_error",
    "Rate limit reached",
    "rate-limit",
    "Token expired",
    "401",
    "Unauthorized",
    "Please run /login",
    "authentication failed",
    "Invalid bearer token",
)


async def _session_is_stuck(sess: str) -> tuple[bool, str]:
    """Check if a session is stuck on 401/rate-limit. Reads last 30 lines of pane.

    Returns (is_stuck, reason). Active workers doing real work return (False, "active").
    """
    try:
        capture = await _arun(
            ["tmux", "capture-pane", "-t", sess, "-p", "-S", "-30"],
            capture_output=True, text=True, timeout=3,
        )
        output = capture.stdout or ""
        for marker in STUCK_MARKERS:
            if marker.lower() in output.lower():
                return True, f"marker:{marker}"
        # Dead pane = also stuck (claude exited from error)
        dead = await _arun(
            ["tmux", "list-panes", "-t", sess, "-F", "#{pane_dead}"],
            capture_output=True, text=True, timeout=3,
        )
        if (dead.stdout or "").strip() == "1":
            return True, "pane_dead"
        return False, "active"
    except Exception as e:
        return False, f"check_failed:{e}"


async def _reconnect_one_session(sess: str, force: bool = False) -> str:
    """Reconnect a tmux session ONLY if it's stuck on 401/rate-limit.

    Active workers doing real work are LEFT ALONE — Claude Code re-reads credentials
    on each API call, so they pick up new tokens naturally.

    Returns 'oracle'/'work' if reconnected, 'skipped:active' if left alone,
    '' if protected.
    """
    if sess in PROTECTED_SESSIONS or sess.startswith("c-"):
        return ""
    try:
        # SAFETY GATE: only reconnect if session is actually stuck.
        # Brute-force reconnecting active workers kills their in-progress work
        # (tool call results lost, builds interrupted, audits restart from scratch).
        if not force:
            is_stuck, reason = await _session_is_stuck(sess)
            if not is_stuck:
                logger.info(f"reconnect: SKIP {sess} (active, {reason})")
                return "skipped:active"
            logger.info(f"reconnect: {sess} is stuck ({reason}) — reconnecting")

        pane = await _arun(
            ["tmux", "list-panes", "-t", sess, "-F", "#{pane_current_command}"],
            capture_output=True, text=True, timeout=3,
        )
        pane_cmd = pane.stdout.strip().split("\n")[0] if pane.stdout else ""
        workdir_q = await _arun(
            ["tmux", "display-message", "-t", sess, "-p", "#{pane_current_path}"],
            capture_output=True, text=True, timeout=3,
        )
        workdir = (workdir_q.stdout or "/home/hacker/VibeCoding").strip() or "/home/hacker/VibeCoding"

        if pane_cmd == "claude":
            await _arun(
                ["bash", "-c", f"{SESSION_RESUME} save {sess}"],
                capture_output=True, timeout=10,
            )
            await _arun(["tmux", "set-option", "-t", sess, "remain-on-exit", "on"], capture_output=True, timeout=3)
            await _arun(["tmux", "send-keys", "-t", sess, "C-c", ""], capture_output=True, timeout=3)
            await asyncio.sleep(1)
            await _arun(["tmux", "send-keys", "-t", sess, "/exit", "Enter"], capture_output=True, timeout=3)
            await asyncio.sleep(2)
            await _arun(["tmux", "respawn-pane", "-k", "-t", sess, "-c", workdir], capture_output=True, timeout=3)
            await asyncio.sleep(1)
            await _arun(
                ["bash", "-c", f"{SESSION_RESUME} launch {sess} {workdir}"],
                capture_output=True, timeout=15,
            )
        else:
            # Shell prompt / dead pane / unknown — respawn fresh
            await _arun(["tmux", "set-option", "-t", sess, "remain-on-exit", "on"], capture_output=True, timeout=3)
            await _arun(["tmux", "respawn-pane", "-k", "-t", sess, "-c", workdir], capture_output=True, timeout=5)
            await asyncio.sleep(1)
            await _arun(
                ["bash", "-c", f"{SESSION_RESUME} launch {sess} {workdir}"],
                capture_output=True, timeout=15,
            )

        return "oracle" if sess.startswith("oracle-") else "work"
    except Exception as e:
        logger.warning(f"reconnect: session {sess} failed: {e}")
        return ""


async def _reconnect_all_sessions(force: bool = False):
    """Reconnect tmux Claude sessions stuck on 401/rate-limit, in PARALLEL.

    FIX-2026-05-12: only reconnect STUCK sessions (401/rate-limit/dead pane).
    Active workers doing real work are LEFT ALONE — Claude Code re-reads
    credentials on each API call, so they naturally pick up new tokens without
    interruption. Killing active workers loses in-progress tool results.

    Args:
        force: if True, reconnect ALL sessions regardless of state (for explicit
               "Reconnect All" button after major auth changes).

    Returns: (oracle_count, work_count, skipped_count) of sessions touched.
    """
    try:
        result = await _arun(
            ["tmux", "list-sessions", "-F", "#{session_name}"],
            capture_output=True, text=True, timeout=5,
        )
        sessions = [s.strip() for s in result.stdout.strip().split("\n") if s.strip()]
    except Exception as e:
        logger.warning(f"reconnect: tmux list-sessions failed: {e}")
        return 0, 0, 0

    # Fan out reconnects in parallel — return_exceptions=True so one failure doesn't abort all
    results = await asyncio.gather(
        *(_reconnect_one_session(s, force=force) for s in sessions),
        return_exceptions=True,
    )

    oracle_count = sum(1 for r in results if r == "oracle")
    work_count = sum(1 for r in results if r == "work")
    skipped = sum(1 for r in results if isinstance(r, str) and r.startswith("skipped:"))
    logger.info(f"Auth change: reconnected {oracle_count} oracles + {work_count} workers, skipped {skipped} active (force={force})")
    return oracle_count, work_count, skipped


def _count_active_sessions():
    """Sync wrapper kept for callers that don't await. Counts only — does NOT reconnect.

    Real reconnection logic is in _reconnect_all_sessions (async).
    """
    oracle_count = 0
    work_count = 0
    try:
        result = subprocess.run(
            ["tmux", "list-sessions", "-F", "#{session_name}"],
            capture_output=True, text=True, timeout=5,
        )
        for sess in result.stdout.strip().split("\n"):
            sess = sess.strip()
            if not sess or sess in PROTECTED_SESSIONS:
                continue
            if sess.startswith("oracle-"):
                oracle_count += 1
            else:
                work_count += 1
    except Exception as e:
        logger.warning(f"_count_active_sessions failed: {e}")
    return oracle_count, work_count


# ============================================================
# Main /account Command
# ============================================================

@auth_required
async def cmd_account(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Multi-account management from Telegram.

    /account              → Status + switch buttons
    /account code XXX     → Exchange OAuth code
    /account check        → Token validity
    /account refresh      → Try refresh
    /account switch NAME  → Switch to saved profile
    /account save         → Save current to active profile
    """

    args = context.args or []
    subcmd = args[0].lower() if args else ""

    # ── /account code <code> ──
    if subcmd == "code":
        await _handle_code(update, context, args)
        return

    # ── /account check ──
    if subcmd == "check":
        await _handle_check(update)
        return

    # ── /account refresh ──
    if subcmd == "refresh":
        await _handle_refresh(update, context)
        return

    # ── /account switch <name> ──
    if subcmd == "switch":
        name = args[1] if len(args) > 1 else ""
        if not name:
            await update.message.reply_text("Usage: /account switch <account_name>")
            return
        await _handle_switch(update, context, name)
        return

    # ── /account save ──
    if subcmd == "save":
        await _handle_save(update)
        return

    # ── Default: show status + all accounts ──
    await _show_account_status(update)


# ============================================================
# Sub-handlers
# ============================================================

async def _handle_code(update: Update, context: ContextTypes.DEFAULT_TYPE, args: list):
    """Paste auth code into the waiting aisb-reauth tmux session."""
    code = args[1] if len(args) > 1 else ""
    if not code:
        await update.message.reply_text("Usage: /account code <paste_the_code_here>")
        return

    msg = await update.message.reply_text("⏳ Pasting code into Claude Code session...")

    try:
        # Check if reauth session exists
        check = await _arun(["tmux", "has-session", "-t", "aisb-reauth"], capture_output=True, timeout=10)
        if check.returncode != 0:
            await msg.edit_text("❌ No reauth session active. Use /account reauth first.")
            _state._pending_reauth = False
            return

        # Snapshot credentials BEFORE pasting code — we'll check if it actually changed
        creds_path = Path.home() / ".claude" / ".credentials.json"
        before_mtime = creds_path.stat().st_mtime if creds_path.exists() else 0
        before_token = ""
        if creds_path.exists():
            try:
                before_token = json.loads(creds_path.read_text()).get("claudeAiOauth", {}).get("refreshToken", "")
            except Exception:
                pass

        # LOG: exactly what code we're about to paste
        logger.info(f"AUTH DEBUG: code length={len(code)}, first10={code[:10]}, last10={code[-10:]}, has_hash={'#' in code}")

        # Capture pane BEFORE paste to verify Claude is waiting for code
        pre_cap = await _arun(
            ["tmux", "capture-pane", "-t", "aisb-reauth", "-p", "-S", "-5"],
            capture_output=True, text=True, timeout=5
        )
        pre_text = pre_cap.stdout if pre_cap.stdout else ""
        logger.info(f"AUTH DEBUG: pane before paste (last 300 chars): {pre_text[-300:]}")
        if "Paste code here" not in pre_text:
            logger.error(f"AUTH DEBUG: Claude NOT waiting for code! Pane content: {pre_text}")

        # Paste code via tmux buffer + bracketed paste
        proc = await asyncio.create_subprocess_exec(
            "tmux", "load-buffer", "-",
            stdin=asyncio.subprocess.PIPE,
        )
        await proc.communicate(code.encode())

        # Verify the buffer actually has the code
        buf_check = await _arun(["tmux", "show-buffer"], capture_output=True, text=True, timeout=5)
        buf_content = buf_check.stdout if buf_check.stdout else ""
        logger.info(f"AUTH DEBUG: tmux buffer length={len(buf_content)}, matches_code={buf_content.strip() == code.strip()}")

        # Paste WITHOUT -p (bracketed paste mode).
        # Claude Code's /login input field does NOT handle bracketed paste
        # escape sequences correctly — the code goes into the buffer but is
        # never displayed/accepted. Plain paste works.
        await _arun(["tmux", "paste-buffer", "-t", "aisb-reauth"], capture_output=True, timeout=10)
        await asyncio.sleep(1)

        # Capture what's in the pane after paste (before Enter)
        mid_cap = await _arun(
            ["tmux", "capture-pane", "-t", "aisb-reauth", "-p", "-S", "-5"],
            capture_output=True, text=True, timeout=5
        )
        mid_text = mid_cap.stdout if mid_cap.stdout else ""
        logger.info(f"AUTH DEBUG: pane after paste (before Enter): {mid_text[-400:]}")

        await _arun(["tmux", "send-keys", "-t", "aisb-reauth", "Enter"], capture_output=True, timeout=10)

        # Wait for credentials to change (poll every 1s, max 20s)
        # This replaces the fixed sleep(6) that was unreliable
        for _wait in range(20):
            await asyncio.sleep(1)
            _cur_mt = creds_path.stat().st_mtime if creds_path.exists() else 0
            if _cur_mt > before_mtime:
                await asyncio.sleep(1)  # Let Claude finish writing
                break

        # Check if auth succeeded — credentials file MUST have changed
        after_mtime = creds_path.stat().st_mtime if creds_path.exists() else 0
        after_token = ""
        if creds_path.exists():
            try:
                after_token = json.loads(creds_path.read_text()).get("claudeAiOauth", {}).get("refreshToken", "")
            except Exception:
                pass

        capture = await _arun(
            ["tmux", "capture-pane", "-t", "aisb-reauth", "-p", "-S", "-10"],
            capture_output=True, text=True, timeout=5
        )
        output = capture.stdout if capture.stdout else ""

        # Real success: credentials file changed AND new token differs from old
        success = (after_mtime > before_mtime) and (after_token != before_token) and bool(after_token)
        if not success:
            logger.warning(f"Auth code paste did NOT update credentials. before_mtime={before_mtime} after_mtime={after_mtime} token_changed={after_token != before_token}")

        if success:
            _state._pending_reauth = False
            email = _get_current_email()
            token_info = _get_token_info()
            expires_min = token_info.get("remaining_min", "?")

            # Auto-save to target account
            target = getattr(_state, "_reauth_target_account", None)
            saved_to = None
            if target:
                _save_current_to_profile(target)
                saved_to = target
                _state._reauth_target_account = None

            # FIX-2026-05-12 (user request): NEVER close sessions on re-login.
            # Just update credentials. Active sessions pick up new token naturally
            # on next API call. Stuck-on-rate-limit sessions are user's call to
            # manually relaunch via the dedicated button.
            logger.info(f"auth-code: success — credentials updated, sessions LEFT UNTOUCHED")

            keyboard = InlineKeyboardMarkup([
                [InlineKeyboardButton("🚀 Relaunch stuck sessions (manual)", callback_data="acc:reconnect_stuck")],
            ])
            result_text = (
                f"<blockquote>"
                f"<b>✅ Authenticated!</b>\n"
                f"━━━━━━━━━━━━━━━━━━━━\n\n"
                f"<b>Email:</b> {escape_html(email)}\n"
                f"<b>Expires:</b> {expires_min} min\n\n"
                f"<i>Credentials updated. All active sessions continue as-is and "
                f"pick up the new token on their next API call. If some sessions "
                f"hit 401/rate-limit before this re-login, click the button below "
                f"to relaunch ONLY those (active workers stay untouched).</i>"
                f"</blockquote>"
            )
            try:
                await msg.edit_text(result_text, parse_mode="HTML", reply_markup=keyboard)
            except Exception as edit_err:
                logger.warning(f"auth-code: edit_text failed ({edit_err}) — sending new message")
                try:
                    await msg.reply_text(result_text, parse_mode="HTML", reply_markup=keyboard)
                except Exception as reply_err:
                    logger.error(f"auth-code: reply_text fallback also failed: {reply_err}")
        else:
            await msg.edit_text(
                f"❌ Auth may have failed.\n\n<code>{escape_html(output[-300:])}</code>",
                parse_mode="HTML",
            )
            _state._pending_reauth = False
    except Exception as e:
        await msg.edit_text(f"❌ Error: {escape_html(str(e))}")
        _state._pending_reauth = False
    finally:
        # ALWAYS cleanup — both tmux session AND pending state
        _state._pending_reauth = False
        try:
            import json as _json_cleanup
            with open("/tmp/aisb-pending-reauth.json", "w") as _f:
                _json_cleanup.dump({"pending": False}, _f)
        except Exception:
            pass
        await _arun(["tmux", "kill-session", "-t", "aisb-reauth"], capture_output=True, timeout=10)


async def _handle_check(update: Update):
    """Check token validity."""
    info = _get_token_info()
    active = _detect_active_account()
    meta = _load_meta()
    label = ""
    if active and active in meta.get("accounts", {}):
        label = f" ({meta['accounts'][active].get('label', active)})"

    icon = "✅" if info["valid"] and not info["warning"] else "⚠️" if info["warning"] else "❌"
    await update.message.reply_text(
        f"{icon} Token {'valid' if info['valid'] else 'EXPIRED'}{label}\n"
        f"Remaining: {info['remaining_min']} min\n"
        f"Tier: {info['tier']}",
    )


async def _handle_refresh(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Try to refresh current token."""
    msg = await update.message.reply_text("🔄 Attempting token refresh...")
    result = await _arun(
        [OAUTH_SCRIPT, "try-refresh"], capture_output=True, text=True, timeout=30
    )
    try:
        data = json.loads(result.stdout.strip())
        if data.get("ok"):
            # Save refreshed token to active profile
            active = _detect_active_account()
            if active:
                _save_current_to_profile(active)

            await msg.edit_text(
                f"✅ Token refreshed! Expires in {data.get('expires_min', '?')} min"
            )
        else:
            await msg.edit_text("❌ Refresh failed — generating reauth link...")
            await _request_reauth(context.application, "Refresh token expired")
    except Exception as e:
        # FIX-CODEAUDIT-F006 (2026-05-08): log parse/edit failure on refresh callback.
        logger.warning(f"refresh callback parse failed: {e}")
        await msg.edit_text(f"❌ {result.stdout[:300]}")


async def _handle_switch(update: Update, context: ContextTypes.DEFAULT_TYPE, account_name: str):
    """Switch to a saved account profile."""
    meta = _load_meta()
    if account_name not in meta.get("accounts", {}):
        names = ", ".join(meta.get("accounts", {}).keys())
        await update.message.reply_text(
            f"❌ Account '{account_name}' not found.\n"
            f"Available: {names}"
        )
        return

    acc = meta["accounts"][account_name]
    msg = await update.message.reply_text(
        f"🔄 Switching to {acc.get('icon', '')} <b>{escape_html(acc.get('label', account_name))}</b>...",
        parse_mode="HTML",
    )

    result = _switch_to_profile(account_name)

    if result["ok"]:
        # FIX-2026-05-12: NEVER touch existing sessions on account switch.
        # Credentials updated → active sessions pick up new token naturally.
        keyboard = InlineKeyboardMarkup([
            [InlineKeyboardButton("🚀 Relaunch stuck sessions (manual)", callback_data="acc:reconnect_stuck")],
        ])
        await msg.edit_text(
            f"<blockquote>"
            f"<b>✅ Switched!</b>\n"
            f"━━━━━━━━━━━━━━━━━━━━\n\n"
            f"<b>Account:</b> {acc.get('icon', '')} {escape_html(result.get('label', account_name))}\n"
            f"<b>Email:</b> {escape_html(result.get('email', '?'))}\n"
            f"<b>Expires:</b> {result.get('expires_min', '?')} min\n"
            f"<b>Method:</b> Token refresh ✅\n\n"
            f"<i>Active sessions continue uninterrupted. Click below ONLY if you "
            f"want to relaunch sessions stuck on 401/rate-limit.</i>"
            f"</blockquote>",
            parse_mode="HTML",
            reply_markup=keyboard,
        )
    else:
        # Refresh failed — need reauth
        keyboard = InlineKeyboardMarkup([
            [InlineKeyboardButton(
                f"🔐 Reauth {acc.get('label', account_name)}",
                callback_data=f"acc:reauth:{account_name}",
            )],
        ])
        await msg.edit_text(
            f"<blockquote>"
            f"<b>⚠️ Refresh Failed</b>\n"
            f"━━━━━━━━━━━━━━━━━━━━\n\n"
            f"<b>Account:</b> {acc.get('icon', '')} {escape_html(acc.get('label', account_name))}\n"
            f"<b>Email:</b> {escape_html(acc.get('email', '?'))}\n\n"
            f"Refresh token expired. Need full OAuth reauth.\n"
            f"<i>Current session still running on previous account.</i>"
            f"</blockquote>",
            parse_mode="HTML",
            reply_markup=keyboard,
        )


async def _handle_save(update: Update):
    """Save current credentials to active profile."""
    active = _detect_active_account()
    email = _get_current_email()

    if active:
        _save_current_to_profile(active)
        meta = _load_meta()
        label = meta["accounts"][active].get("label", active)
        await update.message.reply_text(f"✅ Saved current token to profile: {label}")
    else:
        # Try to match by email
        meta = _load_meta()
        for name, info in meta.get("accounts", {}).items():
            if info.get("email", "").lower() == email.lower():
                _save_current_to_profile(name)
                await update.message.reply_text(f"✅ Saved to profile: {info.get('label', name)}")
                return
        await update.message.reply_text(
            f"❌ No matching profile for email: {email}\n"
            f"Available: {', '.join(meta.get('accounts', {}).keys())}"
        )


async def _show_account_status(update: Update):
    """Show account status — simple: one active account, login/logout."""
    token_info = _get_token_info()
    email = _get_current_email()

    # Status
    if token_info["valid"] and not token_info["warning"]:
        status_icon = "✅"
        status_text_line = f"{token_info['remaining_min']} min remaining"
    elif token_info["warning"]:
        status_icon = "⚠️"
        status_text_line = f"{token_info['remaining_min']} min — switch soon"
    else:
        status_icon = "❌"
        status_text_line = "expired"

    # Sessions count
    tmux_result = await _arun(
        ["tmux", "list-sessions", "-F", "#{session_name}"],
        capture_output=True, text=True, timeout=5,
    )
    sessions = [s for s in tmux_result.stdout.strip().split("\n") if s]
    oracle_count = sum(1 for s in sessions if s.startswith("oracle-"))
    work_count = sum(1 for s in sessions if not s.startswith("oracle-") and s not in ("Home",))

    status_text = (
        f"<blockquote>"
        f"<b>🔐 Claude Code</b>\n"
        f"━━━━━━━━━━━━━━━━━━━━\n\n"
        f"<b>Email:</b> {escape_html(email)}\n"
        f"<b>Token:</b> {status_icon} {status_text_line}\n"
        f"<b>Tier:</b> {escape_html(token_info.get('tier', '?'))}\n"
        f"<b>Sessions:</b> {oracle_count} oracles · {work_count} workers\n"
    )

    if _state._pending_reauth:
        status_text += "\n<b>Status:</b> ⏳ Reauth en attente\n"

    status_text += "</blockquote>"

    # Simple buttons: Login (new account) / Logout / Reconnect
    buttons = [
        [
            InlineKeyboardButton("🔐 Login (new account)", callback_data="acc:reauth"),
            InlineKeyboardButton("🔌 Reconnect", callback_data="acc:reconnect"),
        ],
        [
            InlineKeyboardButton("🚪 Logout", callback_data="acc:logout"),
        ],
    ]

    keyboard = InlineKeyboardMarkup(buttons)

    # FIX-NONETYPE-04 (2026-04-17): update.message is None when this is called
    # from a callback path (acc:reconnect → _show_account_status). Use the
    # effective chat/message resolver so we always have a valid reply target.
    target = update.message or (update.callback_query and update.callback_query.message)
    if target is None:
        logger.warning("_show_account_status: no message target available")
        return
    await target.reply_text(
        status_text,
        parse_mode="HTML",
        reply_markup=keyboard,
    )


# ============================================================
# Callback Handlers (Inline Buttons)
# ============================================================

async def handle_account_callback(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle inline keyboard button presses for account management."""
    query = update.callback_query
    await query.answer()

    data = query.data
    if not data.startswith("acc:"):
        return

    parts = data.split(":", 3)
    action = parts[1] if len(parts) > 1 else ""

    if action == "stop_sdk":
        # Hard stop all running SDK sessions
        from aisb.state import active_runs
        stopped = 0
        for uid in list(active_runs.keys()):
            run = active_runs.pop(uid, None)
            if run:
                client = run.get("client")
                try:
                    if client:
                        await client.disconnect()
                except Exception as e:
                    logger.warning(f"account operation failed: {e}")
                stopped += 1
        await _arun(["pkill", "-f", "claude --output-format"], capture_output=True, timeout=10)
        await _arun(["pkill", "-f", "claude --print"], capture_output=True, timeout=10)
        await query.edit_message_text(f"⏹ Stopped ({stopped} session{'s' if stopped != 1 else ''})")
        return

    if action == "reconnect_stuck":
        # FIX-2026-05-12: new dedicated button — relaunch ONLY sessions stuck on
        # 401/rate-limit. Active workers stay untouched. Replaces the old auto-
        # reconnect that killed in-progress work.
        import time as _t
        t0 = _t.time()
        logger.info(f"reconnect-stuck: button clicked — relaunching stuck sessions only")
        await query.edit_message_text("🔌 Relaunching stuck sessions (active workers untouched)...")
        try:
            restarted, work_restarted, skipped_active = await _reconnect_all_sessions(force=False)
            elapsed = _t.time() - t0
            logger.info(f"reconnect-stuck: DONE in {elapsed:.1f}s — relaunched {restarted + work_restarted}, left {skipped_active} active")
            result_text = (
                f"<blockquote>"
                f"<b>🔌 Stuck sessions relaunched</b>  ·  <code>{elapsed:.1f}s</code>\n"
                f"━━━━━━━━━━━━━━━━━━━━\n"
                f"<b>Relaunched (was stuck):</b> {restarted + work_restarted}\n"
                f"<b>Active sessions left alone:</b> {skipped_active}\n\n"
                f"<i>Active workers continue their ongoing work uninterrupted.</i>"
                f"</blockquote>"
            )
            try:
                await query.edit_message_text(result_text, parse_mode="HTML")
            except Exception as edit_err:
                logger.warning(f"reconnect-stuck: edit failed ({edit_err}) — using reply_text")
                try:
                    await query.message.reply_text(result_text, parse_mode="HTML")
                except Exception as reply_err:
                    logger.error(f"reconnect-stuck: reply_text fallback failed: {reply_err}")
        except Exception as e:
            logger.error(f"reconnect-stuck: failed: {e}")
            try:
                await query.message.reply_text(f"❌ Relaunch failed: {e}")
            except Exception:
                pass
        return

    if action == "reconnect":
        import time as _t
        t0 = _t.time()
        logger.info(f"reconnect: button clicked by user — starting parallel reconnect")
        await query.edit_message_text("🔌 Reconnecting all sessions with current token...")
        try:
            restarted, work_restarted, skipped_active = await _reconnect_all_sessions()
            elapsed = _t.time() - t0
            logger.info(f"reconnect: DONE in {elapsed:.1f}s — oracles={restarted} workers={work_restarted}")
            token_info = _get_token_info()
            active = _detect_active_account()
            meta = _load_meta()
            label = "?"
            if active and active in meta.get("accounts", {}):
                label = meta["accounts"][active].get("label", active)

            keyboard = InlineKeyboardMarkup([
                [InlineKeyboardButton("↩️ Back to accounts", callback_data="acc:back")],
            ])
            result_text = (
                f"<blockquote>"
                f"<b>🔌 Reconnect</b>  ·  <code>{elapsed:.1f}s</code>\n"
                f"━━━━━━━━━━━━━━━━━━━━\n\n"
                f"<b>Account:</b> {escape_html(label)}\n"
                f"<b>Token:</b> {'✅' if token_info['valid'] else '❌'} {token_info['remaining_min']} min\n"
                f"<b>Oracles reconnected:</b> {restarted}\n"
                f"<b>Workers reconnected:</b> {work_restarted}  <i>(stuck only)</i>\n"
                f"<b>Active workers left alone:</b> {skipped_active}\n\n"
                f"<i>Active workers keep working — Claude reads credentials on each API call.</i>"
                f"</blockquote>"
            )
            # FIX-2026-05-12: edit_message_text fails when callback is older than ~15s.
            # Fall back to reply_text (new message) so user ALWAYS sees the result.
            try:
                await query.edit_message_text(result_text, parse_mode="HTML", reply_markup=keyboard)
                logger.info(f"reconnect: edit_message_text OK")
            except Exception as edit_err:
                logger.warning(f"reconnect: edit_message_text failed ({edit_err}) — falling back to reply_text")
                try:
                    await query.message.reply_text(result_text, parse_mode="HTML", reply_markup=keyboard)
                except Exception as reply_err:
                    logger.error(f"reconnect: reply_text fallback also failed: {reply_err}")
        except Exception as e:
            logger.error(f"reconnect: failed with {e}")
            try:
                await query.edit_message_text(f"❌ Reconnect failed: {e}")
            except Exception:
                try:
                    await query.message.reply_text(f"❌ Reconnect failed: {e}")
                except Exception:
                    pass

    elif action == "logout":
        await query.edit_message_text("🚪 Logging out...")
        try:
            # Kill all Claude sessions
            tmux_result = await _arun(
                ["tmux", "list-sessions", "-F", "#{session_name}"],
                capture_output=True, text=True, timeout=5
            )
            killed = 0
            PROTECTED = ("Home", "c-")
            for sess in tmux_result.stdout.strip().split("\n"):
                sess = sess.strip()
                if sess and not any(sess.startswith(p) for p in PROTECTED):
                    await _arun(["tmux", "kill-session", "-t", sess], capture_output=True, timeout=10)
                    killed += 1

            # Run claude auth logout
            await _arun(["claude", "auth", "logout"], capture_output=True, timeout=10)

            # Clear saved accounts (tokens are expired anyway)
            meta = _load_meta()
            meta["accounts"] = {}
            meta["active"] = None
            _save_meta(meta)

            # Clear sessions
            from aisb.sessions import user_sessions, save_sessions
            user_sessions.clear()
            save_sessions({})

            await query.edit_message_text(
                f"<blockquote>"
                f"<b>🚪 Logged Out</b>\n"
                f"━━━━━━━━━━━━━━━━━━━━\n\n"
                f"Sessions killed: {killed}\n"
                f"Accounts cleared\n\n"
                f"Use /account → Login to connect a new account."
                f"</blockquote>",
                parse_mode="HTML",
            )
        except Exception as e:
            await query.edit_message_text(f"❌ Logout failed: {e}")

    elif action == "refresh":
        await query.edit_message_text("🔄 Attempting token refresh...")
        result = await _arun(
            [OAUTH_SCRIPT, "try-refresh"], capture_output=True, text=True, timeout=30
        )
        try:
            data = json.loads(result.stdout.strip())
            if data.get("ok"):
                active = _detect_active_account()
                if active:
                    _save_current_to_profile(active)
                await query.edit_message_text(
                    f"✅ Token refreshed! Expires in {data.get('expires_min', '?')} min\n\n"
                    f"Use /account to see full status.",
                )
            else:
                await query.edit_message_text("❌ Refresh failed — sending reauth link...")
                await _request_reauth(context.application, "Refresh token expired")
        except Exception as e:
            # FIX-CODEAUDIT-F006 (2026-05-08): log parse/edit failure on refresh callback.
            logger.warning(f"refresh callback (inline) parse failed: {e}")
            await query.edit_message_text(f"❌ {result.stdout[:300]}")

    elif action == "reauth":
        # REVERTED FIX-REAUTH-SILENT-01 (2026-04-17): silent refresh was
        # blocking the login link generation when user wanted to connect a
        # NEW account. User needs the /login URL every time to paste the
        # code. Silent refresh is now a SEPARATE button (acc:refresh) —
        # reauth ALWAYS generates the full auth link.
        target_account = parts[2] if len(parts) > 2 else None
        await query.edit_message_text("🔐 Generating auth link...")
        reason = "Manual reauth requested"
        if target_account:
            meta = _load_meta()
            acc = meta.get("accounts", {}).get(target_account, {})
            reason = f"Switch to {acc.get('label', target_account)}"
        await _request_reauth(context.application, reason, target_account=target_account)

    elif action == "switch":
        account_name = parts[2] if len(parts) > 2 else ""
        if not account_name:
            await query.edit_message_text("❌ No account specified")
            return

        meta = _load_meta()
        acc = meta.get("accounts", {}).get(account_name, {})
        await query.edit_message_text(
            f"🔄 Switching to {acc.get('icon', '')} <b>{escape_html(acc.get('label', account_name))}</b>...",
            parse_mode="HTML",
        )

        result = _switch_to_profile(account_name)

        if result["ok"]:
            # FIX-2026-05-12: NEVER touch existing sessions on account switch.
            keyboard = InlineKeyboardMarkup([
                [InlineKeyboardButton("🚀 Relaunch stuck sessions (manual)", callback_data="acc:reconnect_stuck")],
            ])
            await query.edit_message_text(
                f"<blockquote>"
                f"<b>✅ Switched!</b>\n"
                f"━━━━━━━━━━━━━━━━━━━━\n\n"
                f"<b>Account:</b> {acc.get('icon', '')} {escape_html(result.get('label', account_name))}\n"
                f"<b>Email:</b> {escape_html(result.get('email', '?'))}\n"
                f"<b>Expires:</b> {result.get('expires_min', '?')} min\n\n"
                f"<i>Active sessions continue uninterrupted — they pick up the new "
                f"token on next API call. Click below ONLY if you want to relaunch "
                f"sessions stuck on 401/rate-limit.</i>"
                f"</blockquote>",
                parse_mode="HTML",
                reply_markup=keyboard,
            )
        else:
            keyboard = InlineKeyboardMarkup([
                [InlineKeyboardButton(
                    f"🔐 Reauth {acc.get('label', account_name)}",
                    callback_data=f"acc:reauth:{account_name}",
                )],
                [InlineKeyboardButton("↩️ Back to accounts", callback_data="acc:back")],
            ])
            await query.edit_message_text(
                f"<blockquote>"
                f"<b>⚠️ Refresh Failed</b>\n"
                f"━━━━━━━━━━━━━━━━━━━━\n\n"
                f"<b>Account:</b> {acc.get('icon', '')} {escape_html(acc.get('label', account_name))}\n\n"
                f"Refresh token expired.\nNeed full OAuth reauth for this account.\n"
                f"<i>Current session unchanged.</i>"
                f"</blockquote>",
                parse_mode="HTML",
                reply_markup=keyboard,
            )

    elif action == "back":
        # Re-render the full account status inline
        try:
            meta = _load_meta()
            token_info = _get_token_info()
            email = _get_current_email()
            active = _detect_active_account() or meta.get("active")

            if token_info["valid"] and not token_info["warning"]:
                status_icon = "✅"
            elif token_info["warning"]:
                status_icon = "⚠️"
            else:
                status_icon = "❌"

            tmux_result = await _arun(
                ["tmux", "list-sessions", "-F", "#{session_name}"],
                capture_output=True, text=True, timeout=5,
            )
            sessions = [s for s in tmux_result.stdout.strip().split("\n") if s]
            oracle_count = sum(1 for s in sessions if s.startswith("oracle-"))
            work_count = sum(1 for s in sessions if not s.startswith("oracle-") and s not in ("Home",))

            active_label = "unknown"
            active_icon = ""
            if active and active in meta.get("accounts", {}):
                acc = meta["accounts"][active]
                active_label = acc.get("label", active)
                active_icon = acc.get("icon", "")

            status_text = (
                f"<blockquote>"
                f"<b>🔐 Claude Max Accounts</b>\n"
                f"━━━━━━━━━━━━━━━━━━━━\n\n"
                f"<b>Active:</b> {active_icon} {escape_html(active_label)}\n"
                f"<b>Email:</b> {escape_html(email)}\n"
                f"<b>Sub:</b> {escape_html(token_info['sub'])} · {escape_html(token_info['tier'])}\n"
                f"<b>Token:</b> {status_icon} {token_info['remaining_min']} min remaining\n"
                f"<b>Sessions:</b> {oracle_count} oracles · {work_count} workers\n"
            )
            if _state._pending_reauth:
                status_text += "<b>Status:</b> ⏳ Reauth pending\n"
            status_text += "\n<b>📋 Available Accounts:</b>\n"
            for name, info in meta.get("accounts", {}).items():
                icon = info.get("icon", "")
                label = info.get("label", name)
                is_active = "◀️" if name == active else ""
                status_text += f"  {icon} {escape_html(label)} {is_active}\n"
            status_text += "</blockquote>"

            buttons = []
            row = []
            for name, info in meta.get("accounts", {}).items():
                if name == active:
                    continue
                icon = info.get("icon", "")
                label = info.get("label", name)
                row.append(InlineKeyboardButton(
                    f"{icon} {label}", callback_data=f"acc:switch:{name}"
                ))
                if len(row) == 2:
                    buttons.append(row)
                    row = []
            if row:
                buttons.append(row)
            buttons.append([
                InlineKeyboardButton("🔌 Reconnect All", callback_data="acc:reconnect"),
                InlineKeyboardButton("🔄 Refresh Token", callback_data="acc:refresh"),
            ])
            buttons.append([
                InlineKeyboardButton("🔐 Reauth (new link)", callback_data="acc:reauth"),
            ])

            await query.edit_message_text(
                status_text,
                parse_mode="HTML",
                reply_markup=InlineKeyboardMarkup(buttons),
            )
        except Exception as e:
            # FIX-CODEAUDIT-F006 (2026-05-08): log status-render failure for picker.
            logger.warning(f"account status render failed: {e}")
            await query.edit_message_text("Use /account to see all accounts.")


# ============================================================
# Dispatch & Usage Callbacks (kept from original)
# ============================================================

async def handle_dispatch_callback(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle project selection buttons for dispatch."""
    query = update.callback_query
    await query.answer()

    data = query.data
    if not data.startswith("dispatch:"):
        return

    parts = data.split(":", 2)
    project_name = parts[1] if len(parts) > 1 else ""
    prompt = context.user_data.get("_pending_dispatch_prompt", "")

    if project_name == "chat" or not prompt:
        await query.edit_message_text("💬 OK, on discute.")
        if prompt:
            context.user_data.pop("_pending_dispatch_prompt", None)
        return

    from aisb.config import PROJECTS_CONFIG
    proj_conf = None
    for tid, pc in PROJECTS_CONFIG.items():
        if pc.get("name") == project_name:
            proj_conf = pc
            break

    if not proj_conf:
        await query.edit_message_text(f"Projet {project_name} non trouvé")
        return

    icon = proj_conf.get("icon", "")
    await query.edit_message_text(
        f"{icon} <b>{project_name}</b> — c'est parti.\n\n"
        f"L'oracle analyse et dispatche. Je te fais le debrief quand c'est termine.",
        parse_mode="HTML",
    )

    from aisb.oracle_commands import _oracle_direct_dispatch

    class FakeContext:
        def __init__(self):
            self.args = prompt.split()
            self.user_data = context.user_data
    fake_ctx = FakeContext()

    class FakeUpdate:
        def __init__(self):
            self.message = query.message
            self.effective_user = update.effective_user
    fake_update = FakeUpdate()

    await _oracle_direct_dispatch(fake_update, fake_ctx, project_name)
    context.user_data.pop("_pending_dispatch_prompt", None)


async def handle_usage_callback(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle usage monitor inline keyboard buttons.

    FIX-FLOWAUDIT-F14 (2026-05-08): verified — `usage:refresh` and `usage:dismiss`
    both have explicit branches below; `acc:reauth` is handled in
    handle_account_callback. Buttons in /billing card are not silent dead buttons.
    """
    query = update.callback_query
    await query.answer()

    data = query.data
    if not data.startswith("usage:"):
        return

    action = data.split(":", 1)[1]

    if action == "refresh":
        # FIX-USAGE-CB-01 (2026-04-17): usage-monitor.sh takes 12s (spawns tmux
        # + /usage TUI poll). That blows past Telegram's callback_query timeout
        # (~15s server-side + query validity window) → "Query is too old" errors.
        # Strategy: read the cached JSON first (cron refreshes every 10 min).
        # Only re-run the script if cache is >15 min stale, and do it with a
        # reduced timeout so we fail fast rather than hang.
        USAGE_F = "/tmp/aisb-usage.json"
        cache_age = None
        try:
            import os as _os
            import time as _t
            cache_age = _t.time() - _os.path.getmtime(USAGE_F)
        except Exception:
            cache_age = None

        if cache_age is None or cache_age > 900:
            try:
                await _arun(
                    [os.path.expanduser("~/.aisb/lib/usage-monitor.sh")],
                    capture_output=True, text=True, timeout=14,
                )
            except Exception as e:
                logger.warning(f"usage-monitor.sh refresh failed (cache age={cache_age}s): {e}")

        try:
            usage = json.loads(open("/tmp/aisb-usage.json").read())
            pct = usage.get("session_pct", 0)
            week = usage.get("week_pct", 0)
            sonnet = usage.get("sonnet_pct", 0)
            extra = usage.get("extra_pct", 0)
            spent = usage.get("spent", "?")

            filled = int(pct) // 5 if isinstance(pct, (int, float)) else 0
            bar = "█" * filled + "░" * (20 - filled)

            if int(pct) >= 90:
                header = f"🔴  <b>USAGE</b>  ·  <code>{pct}%</code>"
            elif int(pct) >= 70:
                header = f"🟡  <b>USAGE</b>  ·  <code>{pct}%</code>"
            else:
                header = f"🟢  <b>USAGE</b>  ·  <code>{pct}%</code>"

            keyboard = InlineKeyboardMarkup([
                [
                    InlineKeyboardButton("🔄 Switch compte", callback_data="acc:reauth"),
                    InlineKeyboardButton("📊 Refresh", callback_data="usage:refresh"),
                ],
            ])

            await query.edit_message_text(
                f"<blockquote>"
                f"{header}\n"
                f"━━━━━━━━━━━━━━━━━━━━\n\n"
                f"<code>{bar}</code>  {pct}%\n\n"
                f"<b>Daily:</b> {pct}%\n"
                f"<b>Week:</b> {week}%\n"
                f"<b>Sonnet:</b> {sonnet}%\n"
                f"<b>Extra:</b> {extra}%  ·  {spent}\n\n"
                f"━━━━━━━━━━━━━━━━━━━━\n"
                f"◉  <i>{usage.get('ts', '')[:19]}</i>"
                f"</blockquote>",
                parse_mode="HTML",
                reply_markup=keyboard,
            )
        except Exception as e:
            await query.edit_message_text(f"❌ Refresh failed: {e}")

    elif action == "dismiss":
        await query.edit_message_text("✓ <i>Dismissed</i>", parse_mode="HTML")

    elif action == "show_tmux":
        # User asked to disable autonomous mode + show full tmux activity again
        import os as _os
        flag1 = _os.path.expanduser("~/.aisb/state/autonomous-mode.flag")
        flag2 = _os.path.expanduser("~/.aisb/state/hide-tmux-view.flag")
        for f in (flag1, flag2):
            try:
                _os.remove(f)
            except FileNotFoundError:
                pass
            except Exception as _e:
                logger.warning(f"show_tmux: removing {f} failed: {_e}")
        await query.edit_message_text(
            "<blockquote>👁 <b>Vue normale rétablie</b>  ·  <i>autonomous OFF</i></blockquote>",
            parse_mode="HTML",
        )
