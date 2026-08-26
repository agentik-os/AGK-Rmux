"""Shared transcript analyzer for the OmegaOS finish hooks.

ONE parser, two consumers — the Stop finish-guard (stop-verify-hook.sh) and the
SessionStart contract (omega-session-contract.sh). They ask the same question
("what does this session's tracked plan look like right now?") and a second
implementation would drift from the first within a week (R-KARPATHY: no parallel
re-implementations of an existing pattern).

Consumed as a module (`import omega_plan_state`) with OMEGA_HOOKS_DIR on
sys.path, and never executed directly.
"""

import json
import os
import re
import unicodedata

# A verification is a pair: an admissible verification command followed by its
# successful tool result. Merely opening a shell, or writing "tests passed" in
# assistant prose, is never evidence.
VERIFY_COMMAND_RE = re.compile(
    r"(?:^|[\s;&|])(?:"
    r"cargo\s+(?:build|test|check|clippy|fmt\s+--check)\b|"
    r"(?:npm|pnpm|bun|yarn)\s+(?:run\s+)?(?:build|test|check|lint|typecheck)\b|"
    r"(?:python(?:3)?\s+-m\s+)?(?:pytest|unittest)\b|"
    r"go\s+(?:build|test|vet)\b|"
    r"(?:make|just)\s+(?:test|check|verify|build)\b|"
    r"(?:bash\s+)?(?:\./)?[\w./-]*verify[\w.-]*\.sh\b|"
    r"curl\s+[^\n]*(?:--fail|-f)\b"
    r")",
    re.I,
)

VERIFY_SUCCESS_RE = re.compile(
    r"Process exited with code 0|Command exited with code 0|"
    r"\bexit(?:ed)?(?: with)?(?: code)?[=: ]+0\b|"
    r"\"exit_code\"\s*:\s*0\b|test result:\s*ok|"
    r"\b\d+\s+passed\b|\bOK\b|Finished\s+\S+\s+profile|"
    r"HTTP/[\d.]+\s+2\d\d|\b2\d\d OK\b|passed;\s*0 failed",
    re.I,
)

VERIFY_FAILURE_RE = re.compile(
    r"Process exited with code [1-9]\d*|Command exited with code [1-9]\d*|"
    r"\bexit(?:ed)?(?: with)?(?: code)?[=: ]+[1-9]\d*\b|"
    r'"exit_code"\s*:\s*[1-9]\d*\b|test result:\s*failed|'
    r"\b[1-9]\d*\s+failed\b|"
    r"Traceback \(most recent call last\)",
    re.I,
)

# ── R-PREFLIGHT ──────────────────────────────────────────────────────────────
# A preflight is the structured hand-off an INTERACTIVE session gives the
# operator BEFORE touching anything with a real blast radius: Goal, Blocking
# questions, Assumptions, Plan — then it stops and waits for approval. The
# finish guard has to recognise it, otherwise a thorough (read-only) preflight
# trips the planless-work check and gets refused, which is a false positive —
# the one defect a guard cannot afford.
#
# The marker must OPEN a line, may be wrapped in markdown heading / bold /
# emphasis markup, and is either the whole line or is followed by a ':' or '.'
# delimiter. That last condition is what keeps ordinary prose ("Plan the
# migration carefully…") from scoring a family.
FINAL_MESSAGE_MAX = 4000
PREFLIGHT_MIN_FAMILIES = 3
TEXT_BLOCK_TYPES = ("text", "output_text", "input_text", "response_text")


def _marker_re(alternatives):
    return re.compile(
        r"^[ \t]*(?:[#>]{1,6}[ \t]*)*(?:[*_~`]{1,3})?[ \t]*"
        r"(?:" + alternatives + r")\b"
        r"[ \t]*(?:[*_~`]{1,3})?[ \t]*"
        r"(?:[.:]|[ \t\r]*$)",
        re.M,
    )


# Accent- and case-insensitive: matched against text already normalized by
# _fold(), so "Hypothèses" and "hypotheses" are the same marker.
PREFLIGHT_MARKERS = (
    _marker_re(r"goal|objectif"),
    _marker_re(r"blocking questions?|questions? bloquantes?"),
    _marker_re(r"assumptions?|hypotheses?"),
    _marker_re(r"plan"),
)


def _fold(text):
    """Lowercase + strip diacritics (NFD, drop combining marks).

    Same normalization as scripts/hooks/omega-prompt-scan.sh, so FR and EN
    sessions are read by one set of markers. Line structure is preserved.
    """
    folded = unicodedata.normalize("NFD", text.lower())
    return "".join(c for c in folded if unicodedata.category(c) != "Mn")


def is_preflight(text):
    """True when `text` is a structured preflight awaiting approval.

    Pure function of the text so it is testable on its own — analyze() only
    supplies the last assistant message.
    """
    if not text or not isinstance(text, str):
        return False
    folded = _fold(text)
    hits = sum(1 for marker in PREFLIGHT_MARKERS if marker.search(folded))
    return hits >= PREFLIGHT_MIN_FAMILIES


# R-PLAN wants a RECORDED enumeration proving no ask was dropped. A plan tool is
# the usual way to record one, but not every harness exposes TaskCreate /
# TodoWrite / update_plan — and in a session that has none, the planless-work
# check could never be satisfied, so it burned its whole block budget refusing a
# stop the agent had no way to earn. Measured on this box before the fix: 305
# sessions blocked, 62 of them driven to the 3-block ceiling.
#
# So the enumeration itself is accepted, wherever it is recorded. The final
# message qualifies when it lists several distinct items AND marks their state
# — a checklist, a status table, or done/verified markers. Ordinary prose does
# not accidentally satisfy both conditions, so this cannot silently switch the
# guard off: the agent still has to produce the enumeration R-PLAN is asking for.
ENUM_MIN_ITEMS = 3
ENUM_ITEM_RES = (
    re.compile(r"^[ \t]*[-*+][ \t]*\[[ xX~/-]\]", re.M),      # - [x] checklist
    re.compile(r"^[ \t]*\|?[ \t]*\[[ xX~/-]\][ \t]", re.M),  # [x] in a table cell
    re.compile(r"^[ \t]*(?:[0-9]+[.)]|[-*+])[ \t].*"
               r"(?:\bdone\b|\bshipped\b|\bverified\b|\bfait\b|\bverifie)", re.M),
    re.compile(r"^[ \t]*\|.*\|.*(?:\bdone\b|\bverified\b|\bshipped\b"
               r"|\bfait\b|\bverifie)", re.M),                 # | ask | done |
)


def is_enumeration(text):
    """True when `text` records a per-item enumeration with completion state.

    Pure function of the text, like is_preflight(), so it is testable alone.
    """
    if not text or not isinstance(text, str):
        return False
    folded = _fold(text)
    for regex in ENUM_ITEM_RES:
        if len(regex.findall(folded)) >= ENUM_MIN_ITEMS:
            return True
    return False


# Evidence that this harness has NO plan tool at all: the agent asked the tool
# registry for one by name and the registry said there is none. Real transcript
# evidence, never a guess — absent this string the guard assumes plan tools ARE
# available and keeps its full strictness.
NO_PLAN_TOOL_RE = re.compile(
    r"no matching deferred tools found", re.I)
PLAN_TOOL_NAMES = ("taskcreate", "todowrite", "update_plan", "taskupdate")

SHELL_TOOLS = (
    "Bash",
    "shell",
    "exec",
    "exec_command",
    "run_terminal_cmd",
    "local_shell",
)
MUTATION_TOOLS = ("Write", "Edit", "NotebookEdit", "apply_patch")

# A shell command can write a file just as surely as the Write tool, and some
# harnesses actively instruct the agent to prefer Bash heredocs over Write/Edit.
# Counting only the typed tools therefore reported "0 file mutations" for a
# session that had rewritten twenty files — cosmetic in the planless-work
# message, but NOT cosmetic at stop-verify-hook.sh's preflight branch, which is
# gated on `mutations == 0`: a session that had already written files via Bash
# could still claim the legal preflight pause that R-PREFLIGHT grants only
# BEFORE the first write. This closes that hole, so the fix makes the guard
# stricter, never looser.
#
# WRITES only, deliberately — the typed MUTATION_TOOLS are all writes, and
# deletion has its own doctrine (R-DESTRUCT). `/dev/*` and fd dups (`2>&1`)
# are excluded so ordinary output silencing never scores.
SHELL_WRITE_RES = (
    # > file  /  >> file   (not /dev/…, not >&2, not a pipe or another verb)
    re.compile(r">>?\s*(?!/dev/|&|\s*\|)[\w./~$@{}-]*[\w./~}-]"),
    re.compile(r"\btee\b(?!\s+/dev/)"),
    re.compile(r"\bsed\b[^|;&]*\s-i\b"),
    re.compile(r"\b(?:cp|mv|install|truncate)\s+-?"),
    re.compile(r"\b(?:patch|git\s+apply)\b"),
)


def _is_shell_write(command):
    """True when a shell command plausibly writes a file to disk."""
    if not command or not isinstance(command, str):
        return False
    return any(r.search(command) for r in SHELL_WRITE_RES)
DONE_STATES = ("completed", "done", "deleted", "cancelled")
CUSTOM_CMD_RE = re.compile(r'\bcmd\s*:\s*("(?:\\.|[^"\\])*")', re.S)


def _decode_tool_input(obj):
    raw = obj.get("input")
    if raw is None:
        raw = obj.get("arguments")
    if isinstance(raw, str):
        try:
            raw = json.loads(raw)
        except Exception:
            if obj.get("name") == "exec":
                match = CUSTOM_CMD_RE.search(raw)
                if match:
                    try:
                        return {"cmd": json.loads(match.group(1))}
                    except Exception:
                        pass
            return {}
    return raw if isinstance(raw, dict) else {}


def _walk_tool_uses(obj):
    if isinstance(obj, dict):
        if obj.get("type") in ("tool_use", "function_call", "custom_tool_call") and obj.get(
            "name"
        ):
            yield {
                "name": obj.get("name"),
                "input": _decode_tool_input(obj),
                "call_id": obj.get("id") or obj.get("call_id"),
            }
        for v in obj.values():
            yield from _walk_tool_uses(v)
    elif isinstance(obj, list):
        for v in obj:
            yield from _walk_tool_uses(v)


def _walk_tool_results(obj):
    if isinstance(obj, dict):
        result_type = obj.get("type")
        if result_type in (
            "tool_result",
            "function_call_output",
            "custom_tool_call_output",
        ):
            yield {
                "call_id": obj.get("tool_use_id") or obj.get("call_id"),
                "content": obj.get("content")
                if result_type == "tool_result"
                else obj.get("output"),
                "is_error": bool(obj.get("is_error")),
            }
        for value in obj.values():
            yield from _walk_tool_results(value)
    elif isinstance(obj, list):
        for value in obj:
            yield from _walk_tool_results(value)


def _is_assistant_record(rec):
    """Defensive: providers disagree on where the role lives."""
    if not isinstance(rec, dict):
        return False
    if rec.get("type") == "assistant" or rec.get("role") == "assistant":
        return True
    for key in ("message", "payload"):
        nested = rec.get(key)
        if isinstance(nested, dict) and (
            nested.get("role") == "assistant" or nested.get("type") == "assistant"
        ):
            return True
    return False


def _walk_text_blocks(obj):
    if isinstance(obj, dict):
        if obj.get("type") in TEXT_BLOCK_TYPES:
            text = obj.get("text")
            if isinstance(text, str) and text.strip():
                yield text
            return
        # A tool_result carries the WORLD's words, never the assistant's.
        if obj.get("type") in ("tool_result", "function_call_output", "custom_tool_call_output"):
            return
        for value in obj.values():
            yield from _walk_text_blocks(value)
    elif isinstance(obj, list):
        for value in obj:
            yield from _walk_text_blocks(value)


def _assistant_text(rec):
    try:
        return "\n".join(_walk_text_blocks(rec)).strip()
    except Exception:
        return ""


def _command_from_input(inp):
    return str(inp.get("command") or inp.get("cmd") or "")


def _is_verification_command(command):
    return bool(command and VERIFY_COMMAND_RE.search(command))


def _result_succeeded(result):
    if result.get("is_error"):
        return False
    text = json.dumps(result.get("content") or "", ensure_ascii=False)
    if VERIFY_FAILURE_RE.search(text):
        return False
    return bool(VERIFY_SUCCESS_RE.search(text))


def analyze(transcript_path):
    """Reconstruct the session's plan + work signals from its transcript.

    Returns None when there is nothing observable (missing/unreadable file), so
    callers can fail open — a hook must never break a session.

    Keys: open_items, total_tasks, plan_ever, edited, verified, mutations,
    tool_calls, final_message, preflight, enumeration, plan_tools_missing.
    """
    if not transcript_path or not os.path.isfile(transcript_path):
        return None

    created, status_of, todos = [], {}, None
    edited = verified = plan_ever = False
    mutations = tool_calls = 0
    plan_tools_missing = False
    saw_plan_tool_lookup = False
    pending_verifications = {}
    final_message = ""

    try:
        with open(transcript_path, "r", errors="replace") as fh:
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                try:
                    rec = json.loads(line)
                except Exception:
                    continue
                # A sub-agent's own plan is ITS business, not this session's.
                if isinstance(rec, dict) and rec.get("isSidechain"):
                    continue
                # The LAST assistant text wins: a preflight is the final word of
                # the turn, and any later prose replaces it.
                # A tool_result saying the registry has no plan tool, on a
                # lookup that named one, is evidence this harness cannot record
                # a plan the typed way (see NO_PLAN_TOOL_RE).
                # The lookup and its answer arrive in SEPARATE records (the
                # query names the tools, the tool_result carries the verdict),
                # so the two halves are correlated across the stream rather
                # than required to coincide.
                if not plan_tools_missing:
                    blob = json.dumps(rec).lower()
                    if any(n in blob for n in PLAN_TOOL_NAMES):
                        saw_plan_tool_lookup = True
                    if saw_plan_tool_lookup and NO_PLAN_TOOL_RE.search(blob):
                        plan_tools_missing = True
                if _is_assistant_record(rec):
                    text = _assistant_text(rec)
                    if text:
                        final_message = text[-FINAL_MESSAGE_MAX:]
                for tu in _walk_tool_uses(rec):
                    name = tu.get("name") or ""
                    inp = tu.get("input") or {}
                    if not isinstance(inp, dict):
                        inp = {}
                    if name == "TaskCreate":
                        created.append(inp.get("subject") or "(untitled task)")
                        plan_ever = True
                    elif name == "TaskUpdate":
                        tid = str(inp.get("taskId") or "")
                        st = inp.get("status")
                        if tid and st:
                            status_of[tid] = st
                        plan_ever = True
                    elif name == "TodoWrite":
                        t = inp.get("todos")
                        if isinstance(t, list):
                            todos = t
                        plan_ever = True
                    elif name == "update_plan":       # Codex
                        t = inp.get("plan") or inp.get("steps")
                        if isinstance(t, list):
                            todos = t
                        plan_ever = True
                    else:
                        tool_calls += 1
                        if name in SHELL_TOOLS:
                            cmd = inp.get("command") or inp.get("cmd") or ""
                            if isinstance(cmd, (list, tuple)):
                                cmd = " ".join(str(x) for x in cmd)
                            if _is_shell_write(cmd):
                                mutations += 1
                                edited = True
                        if name in MUTATION_TOOLS:
                            edited = True
                            mutations += 1
                            # Evidence is only evidence for the code that existed
                            # when it was gathered. A new edit invalidates it.
                            verified = False
                            pending_verifications.clear()
                        elif name in SHELL_TOOLS:
                            command = _command_from_input(inp)
                            if _is_verification_command(command):
                                call_id = tu.get("call_id")
                                if call_id:
                                    pending_verifications[str(call_id)] = command
                for result in _walk_tool_results(rec):
                    call_id = result.get("call_id")
                    if call_id is None:
                        continue
                    command = pending_verifications.pop(str(call_id), None)
                    if command is not None:
                        verified = _result_succeeded(result)
    except Exception:
        return None

    open_items = []
    total = 0
    if todos is not None:
        total = len(todos)
        for t in todos:
            if not isinstance(t, dict):
                continue
            if (t.get("status") or "").lower() not in DONE_STATES:
                open_items.append(
                    t.get("content") or t.get("step") or t.get("subject") or "(task)"
                )
    elif created:
        total = len(created)
        # Task ids are handed out sequentially as "1", "2", … in creation order.
        for i, subject in enumerate(created, start=1):
            if (status_of.get(str(i)) or "pending").lower() not in DONE_STATES:
                open_items.append(subject)

    return {
        "open_items": open_items,
        "total_tasks": total,
        "plan_ever": plan_ever,
        "edited": edited,
        "verified": verified,
        "mutations": mutations,
        "tool_calls": tool_calls,
        "final_message": final_message,
        "preflight": is_preflight(final_message),
        "enumeration": is_enumeration(final_message),
        "plan_tools_missing": plan_tools_missing,
    }


def int_env(key, default):
    try:
        return int(os.environ.get(key) or default)
    except ValueError:
        return default
