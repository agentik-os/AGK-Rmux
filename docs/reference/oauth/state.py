#!/usr/bin/env python3
"""Runtime mutable state — shared across all modules.

Separated from config.py so constants stay constant and state stays here.
Import state from here, config from config.py.
"""

import asyncio
import contextvars
import json
import logging
import os

logger = logging.getLogger("aisb")

# SMITH learnings file path and rotation threshold (5 MB)
SMITH_LEARNINGS_PATH = os.path.expanduser("~/.aisb/smith/learnings.jsonl")
_SMITH_MAX_BYTES = 5 * 1024 * 1024  # 5 MB


def atomic_write_result(path: str, content: str) -> bool:
    """Atomically write an oracle result file using .tmp + rename + fcntl.flock.

    All writers to /tmp/aisb-oracle-result-*.md MUST use this helper to prevent
    partial reads by the result watcher (FIX-001).

    Returns True on success, False on failure (logged).
    """
    import fcntl
    tmp_path = path + ".tmp"
    lock_path = path + ".lock"
    try:
        # Per-path advisory lock — blocks concurrent writers to same project file
        with open(lock_path, "w") as lock_fd:
            try:
                fcntl.flock(lock_fd.fileno(), fcntl.LOCK_EX)
            except OSError as e:
                logger.warning(f"atomic_write_result: flock failed on {lock_path}: {e}")
            with open(tmp_path, "w") as f:
                f.write(content)
                f.flush()
                try:
                    os.fsync(f.fileno())
                except OSError:
                    pass
            os.rename(tmp_path, path)
        try:
            os.unlink(lock_path)
        except OSError:
            pass
        return True
    except OSError as e:
        logger.warning(f"atomic_write_result failed for {path}: {e}")
        try:
            if os.path.exists(tmp_path):
                os.unlink(tmp_path)
        except OSError:
            pass
        return False


def smith_log(entry: str) -> None:
    """Append a SMITH learning entry with automatic size-based rotation.

    If learnings.jsonl exceeds 5 MB, renames it to .old and starts fresh.
    """
    try:
        path = SMITH_LEARNINGS_PATH
        if os.path.exists(path) and os.path.getsize(path) > _SMITH_MAX_BYTES:
            old_path = path + ".old"
            try:
                os.replace(path, old_path)
                logger.info(f"SMITH learnings rotated: {path} -> {old_path}")
            except OSError as e:
                logger.warning(f"SMITH rotation failed: {e}")
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "a") as f:
            f.write(entry + "\n")
    except Exception as e:
        logger.debug(f"SMITH log failed: {e}")


# Request correlation — set per-handler for structured log tracing
_request_id: contextvars.ContextVar[str] = contextvars.ContextVar('request_id', default='')

# Per-user rate limiter: max 30 commands per 60 seconds
# FIX-CODEAUDIT-2026-05-08: E402 design-forced — `time` import here is
# scoped to the rate-limiter section to match the audit-comment naming.
import time as _time_rl  # noqa: E402
_RATE_LIMIT_MAX = 30
_RATE_LIMIT_WINDOW = 60  # seconds
_rate_limit_log: dict[int, list[float]] = {}  # user_id -> list of timestamps


def rate_limit_check(user_id: int) -> bool:
    """Check if user is within rate limit. Returns True if allowed, False if exceeded."""
    now = _time_rl.time()
    timestamps = _rate_limit_log.get(user_id, [])
    # Prune timestamps outside window
    timestamps = [t for t in timestamps if now - t < _RATE_LIMIT_WINDOW]
    if len(timestamps) >= _RATE_LIMIT_MAX:
        _rate_limit_log[user_id] = timestamps
        return False
    timestamps.append(now)
    _rate_limit_log[user_id] = timestamps
    return True

# Graceful shutdown flag — checked by all background loops
_shutdown_requested: bool = False

# Track active Claude SDK sessions for /stop command
# Maps user_id -> {"task": asyncio.Task, "client": ClaudeSDKClient, "started": float, "prompt": str}
active_runs: dict[int, dict] = {}
_active_runs_lock = asyncio.Lock()

# Track last known state per oracle session to detect transitions
_oracle_states: dict[str, dict] = {}
_oracle_states_lock = asyncio.Lock()

# Track consecutive restart counts per oracle session
_oracle_restart_counts: dict[str, int] = {}

# Reaction tracking: message_id -> list of emoji strings
message_reactions: dict[int, list[str]] = {}

# Pending reauth state
_pending_reauth: bool = False  # True when waiting for user to send auth code
# FIX-CODEAUDIT-EC-1 (2026-05-08): timestamp when _pending_reauth was last set True.
# 5-min TTL self-clears the flag if a path failed silently and never reset it.
# Without this, a forgotten True locks /account login forever (until bot restart).
_pending_reauth_set_at: float = 0  # Unix timestamp; 0 = not pending
PENDING_REAUTH_TTL_SEC = 300  # 5 minutes
_reauth_cooldown: float = 0  # Timestamp — prevent spam reauth requests
_reauth_target_account: str | None = None  # Which account profile to save after reauth

# Rate limit tracking (legacy — usage-monitor.sh handles this now via /tmp/aisb-usage.json)
_last_rate_limit_utilization: float = 0.0

# Routine scheduler state — tracks last execution time per routine
_routine_last_runs: dict[str, float] = {}

# Conversation memory — last N messages per chat for context continuity
# Format: {chat_id: [{"role": "user"/"assistant", "text": "...", "ts": float}, ...]}
# Persisted to ~/.aisb/status/conversation-memory.json on every write
MEMORY_MAX_MESSAGES = 500
_MEMORY_FILE = os.path.expanduser("~/.aisb/status/conversation-memory.json")
try:
    conversation_memory: dict[int, list] = {int(k): v for k, v in json.loads(open(_MEMORY_FILE).read()).items()} if os.path.exists(_MEMORY_FILE) else {}
except Exception as e:
    # FIX-CODEAUDIT-F006 (2026-05-08): corrupt conversation-memory.json → log so
    # silent loss-of-context is diagnosable on next boot.
    logger.warning(f"conversation_memory: starting empty ({e})")
    conversation_memory: dict[int, list] = {}

def save_conversation_memory():
    """Persist conversation memory to disk (atomic write)."""
    try:
        import tempfile
        data = {str(k): v for k, v in conversation_memory.items()}
        dir_path = os.path.dirname(_MEMORY_FILE)
        fd, tmp = tempfile.mkstemp(dir=dir_path, suffix='.tmp')
        try:
            with os.fdopen(fd, 'w') as f:
                json.dump(data, f)
            os.replace(tmp, _MEMORY_FILE)
        except BaseException:
            # Clean up temp file on failure
            try:
                os.unlink(tmp)
            except OSError:
                pass
            raise
    except Exception as e:
        logger.warning(f"FIX-005 file failed: {e}")

# Token recovery: set True when 401/rate-limit detected, triggers auto-reconnect on fresh token
_needs_reconnect: bool = False

# God Mode: autonomous loop — AISB re-prompts oracles until goal is met
# {project_name: {goal, iteration, max_iterations, started, chat_id, progress_log, vision_file}}
# Persisted to disk so it survives bot restart
_GODMODE_FILE = os.path.expanduser("~/.aisb/status/godmode-sessions.json")
try:
    _godmode_sessions: dict[str, dict] = json.loads(open(_GODMODE_FILE).read()) if os.path.exists(_GODMODE_FILE) else {}
except Exception as e:
    # FIX-CODEAUDIT-F006 (2026-05-08): corrupt godmode-sessions.json → log so
    # silent loss of godmode resume-state is visible.
    logger.warning(f"_godmode_sessions: starting empty ({e})")
    _godmode_sessions: dict[str, dict] = {}

def reap_dead_godmode_sessions() -> int:
    """FIX-015: Remove godmode entries whose oracle tmux session is dead.

    Called at bot start. Returns number of entries reaped.
    """
    import subprocess as _sp
    reaped = 0
    for pname in list(_godmode_sessions.keys()):
        oracle_sess = f"oracle-{pname}"
        try:
            r = _sp.run(
                ["tmux", "has-session", "-t", oracle_sess],
                capture_output=True, timeout=5,
            )
            if r.returncode != 0:
                del _godmode_sessions[pname]
                reaped += 1
                logger.info(f"FIX-015 reaped dead godmode session: {pname}")
        except Exception as e:
            logger.warning(f"FIX-015 reap check failed for {pname}: {e}")
    if reaped:
        try:
            save_godmode()
        except Exception as e:
            logger.warning(f"FIX-015 save after reap failed: {e}")
    return reaped


def save_godmode():
    """Persist godmode state to disk (atomic write)."""
    try:
        import tempfile
        dir_path = os.path.dirname(_GODMODE_FILE)
        fd, tmp = tempfile.mkstemp(dir=dir_path, suffix='.tmp')
        try:
            with os.fdopen(fd, 'w') as f:
                json.dump(_godmode_sessions, f, indent=2)
            os.replace(tmp, _GODMODE_FILE)
        except BaseException:
            try:
                os.unlink(tmp)
            except OSError:
                pass
            raise
    except Exception as e:
        logger.warning(f"FIX-005 file failed: {e}")

# Report reply routing: message_id → project_name
# Persisted to disk so it survives bot restart.
# Max 100 entries to avoid bloat.
_REPLY_MAP_FILE = os.path.expanduser("~/.aisb/status/reply-routing.json")
try:
    _report_message_map: dict[int, str] = {int(k): v for k, v in json.loads(open(_REPLY_MAP_FILE).read()).items()} if os.path.exists(_REPLY_MAP_FILE) else {}
except Exception as e:
    # FIX-CODEAUDIT-F006 (2026-05-08): corrupt reply-routing.json → log so reply
    # routing breakage isn't invisible.
    logger.warning(f"_report_message_map: starting empty ({e})")
    _report_message_map: dict[int, str] = {}

def save_reply_routing():
    """Persist reply routing to disk (atomic write)."""
    try:
        import tempfile
        # Keep only last 100
        items = dict(sorted(_report_message_map.items(), reverse=True)[:100])
        dir_path = os.path.dirname(_REPLY_MAP_FILE)
        fd, tmp = tempfile.mkstemp(dir=dir_path, suffix='.tmp')
        try:
            with os.fdopen(fd, 'w') as f:
                json.dump({str(k): v for k, v in items.items()}, f)
            os.replace(tmp, _REPLY_MAP_FILE)
        except BaseException:
            try:
                os.unlink(tmp)
            except OSError:
                pass
            raise
    except Exception as e:
        logger.warning(f"FIX-005 file failed: {e}")


# Last oracle prompt per project — for "Relancer" action button
_last_oracle_prompts: dict[str, str] = {}  # pname -> last prompt text

# Message batching: collect split messages before processing
# Key: (user_id, chat_id) → {"texts": [str], "updates": [Update], "timer": asyncio.Task|None, "type": "text"|"command"}
_message_batch: dict[tuple[int, int], dict] = {}
# How long to wait for more message parts (seconds)
MESSAGE_BATCH_DELAY = 2.0


# ============================================================
# Message WAL (Write-Ahead Log) — P0 crash recovery
# ============================================================
# Before processing a message, write it to WAL. After success, mark done.
# On startup, replay any unprocessed entries.

_WAL_PATH = os.path.expanduser("~/.aisb/state/message-wal.jsonl")


def wal_write(msg_id: int, chat_id: int, user_id: int, text: str, topic_id: int | None = None) -> None:
    """Append a message to the WAL before processing."""
    import time as _t
    entry = json.dumps({
        "msg_id": msg_id, "chat_id": chat_id, "user_id": user_id,
        "text": text[:2000], "topic_id": topic_id,
        "ts": _t.time(), "status": "pending",
    })
    try:
        os.makedirs(os.path.dirname(_WAL_PATH), exist_ok=True)
        # FIX-CODEAUDIT-F004 (2026-05-08): WAL durability — flush + fsync to mirror
        # atomic_write_result. Without this, a kernel buffer + SIGKILL/OOM loses the
        # entry and Telegram never replays the update on restart.
        with open(_WAL_PATH, "a") as f:
            f.write(entry + "\n")
            f.flush()
            try:
                os.fsync(f.fileno())
            except OSError:
                pass
    except Exception as e:
        logger.warning(f"WAL write failed: {e}")


def wal_mark_done(msg_id: int) -> None:
    """Mark a WAL entry as done by appending a completion record."""
    entry = json.dumps({"msg_id": msg_id, "status": "done"})
    try:
        # FIX-CODEAUDIT-F004 (2026-05-08): mirror durability discipline on done-record.
        with open(_WAL_PATH, "a") as f:
            f.write(entry + "\n")
            f.flush()
            try:
                os.fsync(f.fileno())
            except OSError:
                pass
    except Exception as e:
        logger.warning(f"WAL mark_done failed: {e}")


def wal_get_pending() -> list[dict]:
    """Read WAL and return entries that were never marked done."""
    if not os.path.exists(_WAL_PATH):
        return []
    pending = {}  # msg_id -> entry
    try:
        with open(_WAL_PATH, "r") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    entry = json.loads(line)
                    mid = entry.get("msg_id")
                    if mid is None:
                        continue
                    if entry.get("status") == "done":
                        pending.pop(mid, None)
                    elif entry.get("status") == "pending":
                        pending[mid] = entry
                except json.JSONDecodeError:
                    continue
    except Exception as e:
        logger.warning(f"WAL read failed: {e}")
    return list(pending.values())


def wal_compact() -> None:
    """Compact the WAL file by removing all done entries. Called after replay."""
    pending = wal_get_pending()
    try:
        import tempfile
        dir_path = os.path.dirname(_WAL_PATH)
        os.makedirs(dir_path, exist_ok=True)
        fd, tmp = tempfile.mkstemp(dir=dir_path, suffix='.tmp')
        try:
            with os.fdopen(fd, 'w') as f:
                for entry in pending:
                    f.write(json.dumps(entry) + "\n")
            os.replace(tmp, _WAL_PATH)
        except BaseException:
            try:
                os.unlink(tmp)
            except OSError:
                pass
            raise
    except Exception as e:
        logger.warning(f"WAL compact failed: {e}")


# FIX-CODEAUDIT-EC-2 (2026-05-08): WAL replay deduplication markers.
# When startup replay surfaces a pending message AND Telegram redelivers the
# same update on bot resume (long-poll timeout), the bot would otherwise
# process it twice. We write a marker on replay-notify; process_prompt early-
# returns if a fresh (<5min) marker exists for the incoming message_id.
WAL_REPLAYED_DIR = os.path.expanduser("~/.aisb/state/wal-replayed")
WAL_REPLAYED_TTL_SEC = 300  # 5 minutes


def wal_mark_replayed(msg_id: int) -> None:
    """Stamp a marker that this message_id was surfaced by WAL replay.

    Called when post_init recovers pending entries — defends against Telegram
    redelivering the same update on bot resume.
    """
    try:
        os.makedirs(WAL_REPLAYED_DIR, exist_ok=True)
        marker = os.path.join(WAL_REPLAYED_DIR, f"wal-replayed-{int(msg_id)}.ts")
        with open(marker, "w") as f:
            f.write(str(__import__("time").time()))
    except Exception as e:
        logger.debug(f"wal_mark_replayed failed: {e}")


def wal_is_replayed(msg_id: int) -> bool:
    """Return True if a fresh (<TTL) replay marker exists for this msg_id."""
    try:
        marker = os.path.join(WAL_REPLAYED_DIR, f"wal-replayed-{int(msg_id)}.ts")
        if not os.path.exists(marker):
            return False
        import time as _t
        age = _t.time() - os.path.getmtime(marker)
        return age < WAL_REPLAYED_TTL_SEC
    except Exception as e:
        logger.debug(f"wal_is_replayed failed: {e}")
        return False
