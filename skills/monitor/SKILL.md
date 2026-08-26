---
name: monitor
description: >
  Point a monitor at a running rmux session (on this box or on any ssh host) and get an ANSWER
  instead of a screen. A cheap 60s watcher classifies the session as QUESTION, STALLED, BLOCKED or
  WORKING and answers each one differently: a question goes to a human because it needs judgement, a
  stall gets a mechanical nudge, a block is NEVER nudged because that is manufactured thrash, and
  working stays silent. A deep audit team of parallel read-only sub-agents runs on a slower cadence,
  with dimensions derived from the WATCHED project's own rules. Use when the user says "/monitor",
  "/omg-monitor", "monitor this session", "watch this build", "keep an eye on the oracle", "babysit
  that session", "is it stuck", "did it stall", "why did it stop", "audit that session continuously",
  or in French "surveille cette session", "surveille ce build", "garde un oeil sur l'oracle",
  "est-ce qu'il est bloque", "il s'est arrete", "pourquoi il ne bouge plus", "audite la session en
  continu". NOT for MIRRORING a session so a human can read it (that is `omega stream`, R-STREAM),
  NOT the Claude billing and account view (that is bare `omega monitor` with no target), and NOT a
  one-shot scored audit of a repo (that is /codeaudit, /secaudit and friends, R-AUDIT).
argument-hint: "@<session> | <host>:<session> | list   [--work-probe <cmd>] [--progress-probe <cmd>] [--nudge-budget N] [--interval S]"
allowed-tools: ["Bash", "Read", "Grep", "Glob", "Task"]
domain: orchestration
read_only: false
triggers: ["monitor", "omg-monitor", "monitor this session", "watch this build", "keep an eye on", "babysit the session", "is it stuck", "did it stall", "why did it stop", "audit the session continuously", "surveille cette session", "surveille ce build", "garde un oeil", "est-ce qu'il est bloque", "il s'est arrete", "pourquoi il ne bouge plus", "audite la session en continu"]
source: OmegaOS, built on the omega stream substrate (R-STREAM)
---

# /monitor, a watcher that answers instead of a screen you read

`omega stream` renders a running session so a HUMAN can read it. `/monitor` decides what that
same rendered screen MEANS and acts on it without a human in the inner loop: it classifies the
session every 60 seconds, nudges the stops that are mechanical, escalates the stops that need
judgement, stays silent while work is happening, and periodically hands the watched project to a
team of adversarial read-only auditors.

The whole value is credibility. A watcher that cries wolf trains the operator to ignore it, and an
ignored alert is worth exactly nothing. Every constraint below was paid for by a real false positive
on a hand-built watcher. Do not re-derive them, and do not hand-roll a second watcher beside this
one.

---

## Do not confuse it with the billing view: `omega monitor` was already taken

`omega monitor` with NO target is a pre-existing, unrelated feature: the read-only view of Claude
Code billing, accounts and AISB bot status (`crates/omega-core/src/monitor.rs`, its own TUI Monitor
tab). It still does exactly that, unchanged.

**A TARGET is what selects the session monitor.** Bare command, billing. Target, session monitor.
The session-monitor logic lives in `crates/omega-core/src/session_monitor.rs`, never in `monitor.rs`.

---

## Invocation

| Form | What it does |
|---|---|
| `/monitor @<session>` | watch a session on THIS box (the `@` is the skill-level sigil for a session name, stripped before it reaches the CLI) |
| `/monitor <host>:<session>` | watch a session on `<host>`, an ALIAS from `~/.ssh/config` |
| `/monitor list` | every session watchable here and on every ssh alias |
| `/monitor` (no target) | list first, then ask which one. Never guess a session |

Underneath, one canonical command:

```bash
omega monitor <session>                 # local
omega monitor <host>:<session>          # remote, host = ssh alias
omega monitor list                      # what is watchable, everywhere
omega monitor                           # UNCHANGED: the billing view

# tuning (all optional, all on a targeted invocation)
omega monitor <target> \
  --work-probe     '<shell command printing an integer>' \
  --progress-probe '<shell command printing a monotonic integer>' \
  --nudge-budget   5 \
  --interval       60
```

The judgement itself is a pure Rust function over a captured pane, `session_monitor::classify(pane,
work)`, exposed for the poll loop as:

```bash
omega monitor classify --work <n> < pane.txt      # prints: QUESTION | STALLED | BLOCKED | WORKING
```

Pure means unit-testable with zero ssh and zero rmux, which is how all four states are proven against
real captured panes instead of being asserted. The shell loop (`~/.omega/bin/omega-monitor.sh`,
shipped from `scripts/`) does the poll, the ssh, the sleep and the send; it delegates every judgement
to that function. The reference watcher put all of this in bash greps that could never be tested, and
that is the one thing this design deliberately improves on.

The watcher runs as its own detached rmux session (documented form `monitor-<session>`, or
`monitor-<host>-<session>`, with the same injective fingerprint fallback `omega stream` uses when a
plain name cannot encode the coordinate faithfully) so it survives your turn ending. Stop it with
`omega kill monitor-<session>`.

---

## The four states, because two is one too few

This is the single biggest lesson: **a stop has more than one shape, and the shapes need opposite
answers.** A watcher with only "moving" and "not moving" will nudge a session that is waiting for a
human, and will nudge a session with nothing left to run.

| State | What it means | The correct answer |
|---|---|---|
| `QUESTION` | The agent is asking and will not move until answered. | Needs JUDGEMENT, so it goes to a human (or the orchestrating agent). NEVER auto-answer. |
| `STALLED` | The turn finished with work still available. | MECHANICAL, so answer it mechanically: one nudge. |
| `BLOCKED` | Nothing is runnable. | NEVER nudge. A nudge here is not persistence, it is manufactured thrash. Escalate instead. |
| `WORKING` | Busy. | Say NOTHING. Silence is the correct output most of the time. |

### QUESTION: match the STRUCTURE, on ONE line

The question UI prints a single hint line carrying BOTH `Enter to select` AND `to navigate`. From
`GOLDEN-question-real`, a genuine capture of a live question in the classifier's golden corpus:

```
Enter to select · ↑/↓ to navigate · Esc to cancel
```

Match both markers **on the same line**. Matching either one alone fires on ordinary prose: a review
that said "there is nothing to navigate to while it is open" was read as a pending question, and the
operator got paged for a sentence.

The same-line rule is necessary and not sufficient. A pane where an agent is DISCUSSING the matcher
puts both markers on one line for real, because our own command text echoes into the pane:

```
q=$(grep -cE 'Enter to select.*to navigate' "panes/$s.txt")
```

That is `GOLDEN-self-echo-false-question`, the hardest adversarial case in the corpus, and it is a
real pane from a session that was building this very feature. It is killed by defect 1 below (exclude
what we sent, by content, in short slices) plus the precedence rule in the next section, never by the
regex.

### WORKING: read the live chrome, never the scrollback

The activity indicator is `· … tokens` on the spinner line, or `esc to interrupt` in the status bar:

```
✻ Designing /monitor on the stream substrate… (4m 30s · ↓ 19.4k tokens)
  ⏵⏵ bypass permissions on (shift+tab to cycle) · esc to interrupt · ctrl+t to hide tasks
```

When the turn ends, both vanish. `GOLDEN-stalled-real` is the same session after the answer
landed, and its status bar is `⏵⏵ bypass permissions on (shift+tab to cycle) · ← 2 agents`, with an
empty prompt box above it. Neither marker, so not WORKING.

**A pane rendering an activity indicator is WORKING even if something upstream in the scrollback
looks like a question**, because a turn that is actively rendering is not parked waiting for an
answer. That precedence is the second line of defense on the self-echo pane above (which genuinely
was working). It only holds if the activity markers are read from the LIVE chrome at the tail, never
from anywhere in the scrollback, which is what defects 5 and 6 are about.

### STALLED vs BLOCKED: the work probe decides

Not-working and not-question is ambiguous, and no amount of pane-reading resolves it. A **work
probe** does: a shell command supplied per target that prints an integer, the amount of work
currently available in the watched build.

- `> 0` → `STALLED`. Nudge it.
- `== 0` → `BLOCKED`. Never nudge, escalate.
- unreadable, non-numeric, probe missing → **assume WORK**, so `STALLED`. Never stall silently.

The fail-open direction is deliberate and asymmetric: a wrong nudge costs one turn and is bounded by
the nudge budget, while a wrong silence costs the whole build and nothing bounds it. With no
`--work-probe` supplied, every non-question stop is therefore STALLED, and BLOCKED can never be
distinguished. Supply one whenever the build has any queryable notion of remaining work.

**A work probe counts work in ANY form, not just buildable work.** This is defect 7: the first
watcher counted only steps marked *runnable* and reported BLOCKED while steps sat awaiting a
sign-off. Awaiting judgement IS work. A probe that undercounts manufactures a blocked verdict on a
healthy build, and BLOCKED is the one state that stops the machine.

---

## Two layers, two cadences

| Layer | Cadence | Cost | What it answers |
|---|---|---|---|
| The watcher | 60s | one capture plus one probe | Is it moving, and if not, which kind of stopped? |
| The deep audit team | slow (default: every 30 watcher cycles, and on every escalation) | a parallel fan-out | Is what it is building actually any good? |

The watcher is deliberately cheap because it runs forever. It captures, classifies, acts, sleeps.
The audit is deliberately expensive because it runs rarely, and it is a Workflow fan-out you own
(R-ORCH), not something the poll loop does inline: a 60s loop that spawns sub-agents is a loop that
spends the budget on watching instead of on building.

Unattended runs pace the audit layer with the native `/loop` (R-LOOP): the watcher session already
survives on its own, so only poll state the harness cannot see, and keep the R-LOOP ceilings.

---

## The deep audit team: dimensions come from the WATCHED project

One sub-agent per dimension, run IN PARALLEL, each READ-ONLY, each reporting `file:line` evidence
and RANKED findings (R-CITE: uncited assertions are rejected).

**Derive the dimensions from the watched project's own rules.** Discover them first: its `CLAUDE.md`,
its `RULES.md`, its rules file, its stated invariants. A generic checklist finds generic nothing, and
the operator learns to skip the report. If, and only if, the project states no rules of its own, fall
back to these four:

- design and token rules
- access control and isolation
- test reality (does the green actually mean anything)
- corpus and documentation integrity

**An audit that returns "all clean" every time is an audit nobody reads.** So instruct every auditor
to be ADVERSARIAL (try to falsify the project's claim about itself, R-VERIFY), and to state plainly
IN ONE LINE when a rule genuinely holds and why it holds. A clean result written as one specific line
("the token ratchet covers px in components, the exempt list is the token file only") is information;
"no issues found" is noise. Never let an auditor pad a finding to look useful, and never accept a
delegate's summary as the verdict: synthesize it yourself.

---

## The nudge bound: it bounds the ABSENCE OF PROGRESS, not the work

A flat cap of N nudges stops a healthy long run for no reason. Measured: a cap of 25 ran out around
step 40 of a 157-step build, and the build had been advancing the entire time.

So the counter is bound to progress, not to attempts:

- `--progress-probe` prints a **monotonic integer**, the build's own progress metric (steps done,
  tickets closed, tests green, whatever that build actually counts).
- Every time it ADVANCES, reset the nudge counter to zero and say the budget reset, in one line.
- Stop only after `--nudge-budget` nudges that produced NOTHING.
- On exhaustion, stop nudging, set `escalate_to_human`, and say plainly that this needs a human and
  why, through the alert funnel (`~/.omega/bin/omega-alert-send.sh "<html>"`, R-TGDELIVER + R-TGSEC).

That is what R-LOOP actually asks for: bounded retries on the SAME failure, never a ceiling on
healthy work. Re-nudging a fourth time into the same wall is thrash, not persistence.

The nudge itself is one short instruction sent to the session (locally, `omega send <session>
"<text>"`; remotely, the loop does the ssh and the rmux `send-keys` where **Enter is a SEPARATE
call**). Record every nudge in the echo ledger AT SEND TIME, before it can come back at you as pane
text: see defect 1.

---

## Why the matching is so specific: the seven defects

Each one is an invariant plus the false positive that taught it. Each cost credibility, which is the
only thing an alerting system has.

1. **It matched its OWN messages.** Text sent into a session ECHOES in the pane. Record what you send
   at SEND TIME and exclude it BY CONTENT, in SHORT SLICES, because the pane hard-wraps and a
   whole-line match silently fails. Sentinel required: `grep -f` against an EMPTY pattern file
   matches nothing and blanks the entire stream, so the ledger is seeded with one impossible line and
   is never empty.
2. **Two signals shared one state variable and ping-ponged**, re-firing forever. ONE VARIABLE PER
   SIGNAL. (The failure branch wrote it, the idle branch overwrote it, and on the next poll the
   unchanged failure text no longer matched the variable so it fired again, forever.)
3. **Clearing the failure latch on resume made frozen scrollback re-fire.** Do NOT clear it. The pane
   keeps its scrollback; genuinely new failure text differs from the old and gets through on its own.
4. **It matched WORDS where STRUCTURES were meant.** Bare `MISSING` fired on the prose "missing
   redirect" describing an already-fixed defect. `is RED` fired on "is REDundant". Match the EXACT
   strings a runner emits, WITH their punctuation: `MISSING ROW:`, `is RED.`, `exit code [1-9]`,
   `Traceback`.
5. **Dropping the tail of the capture did not reliably cut the input box**, because a long pending
   message is longer than the drop. Cut it STRUCTURALLY, to the prompt marker (the rule line plus the
   `❯` box), never by a fixed line count.
6. **Completed task lines re-render constantly and drowned the in-progress transitions**, which are
   the only lines that say what is happening NOW. Drop completed items before diffing.
7. **It counted only runnable steps and reported BLOCKED while steps awaited a sign-off.** Awaiting
   judgement is work.

---

## Coordinates and substrate: reuse, never re-implement

`crates/omega-core/src/stream.rs` already solved the coordinate layer and is battle-tested (R-STREAM).
The session monitor IMPORTS it and reimplements none of it: `parse_target`, `is_safe_coordinate`,
`read_ssh_config` / `ssh_hosts`, `probe_target`, `rmux_bin()`, `session_exists`.

What that buys you, and what you must not undo:

- **Host coordinates come from `~/.ssh/config`, never from a literal.** Pass the ALIAS and let ssh
  resolve HostName, Port, User and IdentityFile. One box on this tailnet answers on port 42820, and a
  probe against 22 times out in a way that reads exactly like a firewall block.
- **rmux is not tmux.** Always the absolute `~/.local/bin/rmux` (it is not on the non-interactive
  PATH); it exports `RMUX` and `RMUX_PANE`, not `$TMUX` (testing `$TMUX` reports "not in a
  multiplexer" from inside one); `send-keys` needs its Enter as a separate call; and rmux does not
  REJECT a bad session name, it silently REWRITES `:` and `.` to `_`, which is why coordinates go
  through `is_safe_coordinate` up front.
- **Quoting kills this silently.** A `$VAR` inside a double-quoted remote ssh command expands
  LOCALLY: the remote rmux path must reach the REMOTE shell unexpanded, and a `#S` format stays
  quoted or the remote shell reads `#` as a comment.
- **The loop MUST NEVER EXIT ON ERROR.** If it exits, the watcher session dies and the operator sees
  nothing at all, which is strictly worse than an error on screen. No `set -e`. Errors are RENDERED,
  never fatal.
- **Every capture runs under its OWN wall clock.** `ConnectTimeout` bounds only the CONNECT: a box
  that answers TCP and then never replies leaves the capture blocking forever and the watcher freezes
  while still claiming to watch. `timeout` is GNU coreutils and does not exist on stock macOS
  (`gtimeout` at best), so it is resolved once with a bash watchdog fallback that signals the
  capture's PROCESS GROUP. Signalling only the direct child leaves a grandchild holding the stdout
  pipe open, and the loop stays frozen even though the clock fired.

---

## What comes back, and what you do with it

The watcher prints one line per poll, and stays quiet otherwise:

```
MONITOR · matrix:MoonBaseCapital · WORKING · work=7 · progress=41 · nudges 0/5 · 17:42:03
MONITOR · matrix:MoonBaseCapital · STALLED · work=7 · progress=41 · nudged (1/5)
MONITOR · matrix:MoonBaseCapital · WORKING · work=6 · progress=42 · nudge budget reset (progress 41 → 42)
MONITOR · matrix:MoonBaseCapital · QUESTION · escalated to the operator, NOT answered
MONITOR · matrix:MoonBaseCapital · BLOCKED  · work=0 · not nudged, escalated
```

Your side of the contract as the agent that pointed it there:

- **QUESTION goes to a human, verbatim.** Relay the question and the options; never answer on the
  operator's behalf, and never let the relay editorialize the choice. Judgement is the reason this
  state exists.
- **BLOCKED goes to a human too, with the work probe's number and the last transition line.** It is
  not a retry candidate.
- **STALLED needs nothing from you** until the budget runs out. Then it becomes an escalation with
  `escalate_to_human` set (R-LOOP).
- **WORKING needs nothing from you, ever.** Do not summarize a healthy build every minute.
- Keep the monitoring task open in your plan until you have verified the outcome yourself (R-PLAN,
  R-VERIFY): a watcher's own line is an input, never the verdict.
- Deliverables (an audit report, a link) also go to Telegram in the same turn (R-TGDELIVER).

---

## Pairs with

- **`omega stream` (R-STREAM)**: the substrate, and the human-facing twin. Stream RENDERS a session
  for a person to read, monitor DECIDES what it means without one. Same coordinates, same rmux
  constraints, opposite consumer.
- **`/dynamic` (R-ORCH)**: the fan-out primitive the deep audit team runs on, one read-only sub-agent
  per project-derived dimension, in parallel, then you synthesize.
- **`/codeaudit`, `/secaudit`, `/uiuxaudit` and friends (R-AUDIT)**: when the watched project needs a
  real scored forensic audit rather than a continuous adversarial sweep. Invoke the real skill, never
  a paraphrase of it.
- **`/goal` and `omega spawn-worker`**: the things you usually end up monitoring. Point the watcher at
  the worker, not at your own session.
