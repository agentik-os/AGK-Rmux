#!/usr/bin/env bash
# omega-prompt-scan.sh — UserPromptSubmit hook: catch, at the moment the prompt
# arrives, the two shapes that are unrecoverable later.
#
#   BRANCH 1 — MULTI-ASK: several asks in one prompt, whose tail tasks get
#              silently dropped (L6 ENUMERATE, R-PLAN, R-ORCH fan-out).
#   BRANCH 2 — HIGH BLAST RADIUS: an ask to IMPLEMENT something touching auth,
#              money, migrations/schema, deletion, or a whole new surface —
#              where guessing costs a rewrite (R-PREFLIGHT).
#
# The two branches are INDEPENDENT: either can fire alone, both can fire together,
# and when both fire the injected note covers both without repeating itself.
#
# WHY HERE AND NOT AT STOP TIME: refusing a stop because "you should have fanned
# out" (or "you should have asked first") is useless — the work is already done,
# the turn is already spent. The only moment those decisions can still be made is
# when the prompt lands. So this hook never blocks; it injects context exactly
# when it can still change what happens.
#
# Deliberately quiet: it only speaks when the prompt genuinely looks like several
# asks, or genuinely looks like a high-stakes implementation. An ordinary prompt
# gets nothing, because a nudge that fires every turn is a nudge that gets
# ignored. Proportionality is enforced on branch 2 too: an obviously-tiny ask
# (typo, rename, one-liner) is skipped outright.
#
# Output: {"hookSpecificOutput":{"hookEventName":"UserPromptSubmit",
#          "additionalContext":"…"}} — or nothing at all.

set -uo pipefail

[[ "${OMEGA_PROMPT_SCAN:-on}" == "off" ]] && exit 0

IN=""
[ ! -t 0 ] && IN=$(cat 2>/dev/null)
[ -z "$IN" ] && exit 0

command -v python3 >/dev/null 2>&1 || exit 0

OMEGA_HOOK_INPUT="$IN" python3 - <<'PY'
import json, os, re, sys, unicodedata

try:
    payload = json.loads(os.environ.get("OMEGA_HOOK_INPUT") or "{}")
except Exception:
    sys.exit(0)
if not isinstance(payload, dict):
    sys.exit(0)

prompt = payload.get("prompt") or payload.get("user_prompt") or ""
if not isinstance(prompt, str) or len(prompt) < 60:
    sys.exit(0)          # too short to be a multi-part mission

# Accent-insensitive: the operator types fast and drops diacritics, so "mets a
# jour" must match the same rule as "mets à jour".
low = unicodedata.normalize("NFD", prompt.lower())
low = "".join(c for c in low if unicodedata.category(c) != "Mn")

# ---------------------------------------------------------------------------
# BRANCH 1 — MULTI-ASK
# ---------------------------------------------------------------------------

signals = 0
detail = []

# 1. An explicit enumeration of 3+ items. On its own this IS a multi-ask prompt,
#    so it scores the full threshold — no second signal needed.
#    Counted both as list lines AND inline ("do 1. x 2. y 3. z"), because a fast
#    operator types the whole list on one line.
enumerated = max(
    len(re.findall(r"(?m)^\s*(?:[-*•]|\d+[.)])\s+\S", prompt)),
    len(re.findall(r"(?:(?<=\s)|^)\d+[.)]\s+\S", prompt)),
)
if enumerated >= 3:
    signals += 2
    detail.append("%d enumerated items" % enumerated)

# 2. Additive connectors — the "et aussi" pattern that quietly doubles a mission.
#    FR and EN, because the operator writes in both.
connectors = re.findall(
    r"\b(?:et aussi|puis|ensuite|egalement|en plus|par ailleurs|aussi stp|"
    r"and also|then also|after that|additionally|as well as)\b", low)
if connectors:
    signals += 1
    detail.append("%d additive connector(s)" % len(connectors))

# 3. Several distinct imperatives — separate deliverables, not one elaborated ask.
verbs = set(re.findall(
    r"\b(?:fix|build|create|add|update|refactor|migrate|deploy|test|audit|review|"
    r"verify|document|analyze|analyse|implement|remove|rename|publish|ship|check|"
    r"corrige|construis|cree|ajoute|mets? a jour|met a jour|refactorise|migre|"
    r"deploie|teste|audite|verifie|documente|implemente|supprime|renomme|publie|"
    r"fais|lance|regarde|ameliore|nettoie|installe|configure)\b", low))
if len(verbs) >= 3:
    signals += 2
    detail.append("%d distinct action verbs" % len(verbs))
elif len(verbs) == 2:
    signals += 1
    detail.append("2 distinct action verbs")

# 4. Sheer length. A 300+ character prompt is far more often several asks than
#    one, but it is a weak signal on its own — it only ever confirms another.
if len(prompt) >= 300:
    signals += 1
    detail.append("%d chars" % len(prompt))

multi_ask = signals >= 2

# ---------------------------------------------------------------------------
# BRANCH 2 — HIGH BLAST RADIUS (R-PREFLIGHT)
#
# Independent of branch 1. Fires only on an ask to IMPLEMENT (an action verb is
# present — a question about auth is not an auth change), only when the prompt is
# substantive, and never on an ask that announces its own smallness.
# ---------------------------------------------------------------------------

BLAST = (
    ("auth/identity", r"\b(?:auth|authentication|authentification|login|log in|logout|"
                      r"session|sessions|token|tokens|jwt|oauth|sso|password|passwords|"
                      r"mot de passe|mots de passe|permission|permissions|rbac|role|roles|"
                      r"credential|credentials|identifiants|droits|acl)\b"),
    ("money",         r"\b(?:payment|payments|billing|stripe|checkout|price|prices|pricing|"
                      r"invoice|invoices|subscription|subscriptions|refund|refunds|payout|"
                      r"paiement|paiements|facturation|facture|factures|abonnement|"
                      r"abonnements|prix|tarif|tarifs|remboursement|remboursements)\b"),
    ("migration/schema", r"\b(?:migration|migrations|migrate|migrer|schema|schemas|"
                      r"alter table|drop table|drop column|truncate|index|indexes|indices|"
                      r"prisma|convex schema|supabase migration|base de donnees|"
                      r"database|db schema)\b"),
    ("deletion/irreversible", r"(?:\b(?:delete|deletes|deleting|remove|removes|removing|"
                      r"purge|wipe|drop|reset|revert|rollback|supprimer|supprime|effacer|"
                      r"efface|ecraser|ecrase|purger|vider)\b|rm\s+-rf|force[ -]push|"
                      r"push\s+--force)"),
    ("new surface",   r"(?:\bnew (?:module|service|package|crate|app|api|subsystem)\b|"
                      r"\bnouveau (?:module|service|paquet)\b|\bnouvelle (?:app|api)\b|"
                      r"\brewrite\b|\breecrire\b|\brefonte\b|"
                      r"\brefactor (?:the )?(?:whole|entire|all)\b|"
                      r"\bfrom scratch\b|\bde zero\b|\bfrom the ground up\b)"),
)

# Explicit smallness markers — the operator saying "this is tiny" is the most
# reliable proportionality signal there is, so it wins outright.
SMALL = re.compile(
    r"\b(?:typo|typos|coquille|faute de frappe|one[- ]line|one[- ]liner|single line|"
    r"une ligne|une seule ligne|juste renomme|juste un renommage|petit fix|petite "
    r"correction|small fix|tiny|trivial|quick fix|just rename|rename this|rename the|"
    r"renomme)\b")

# MACHINE-GENERATED PROMPTS ARE NOT ASKS. Measured on 147 real prompts from this
# box: the first cut of this branch fired on 36% of them, and the top offenders
# were not asks at all — an oracle control message (`## Mission …`), a skill
# preamble (`Base directory for this skill: …`), a pasted PRD. A nudge that
# fires on one prompt in three is wallpaper, and wallpaper is how an operator
# learns to ignore every alert (R-MONITOR: a false positive is the only fatal
# defect an alerting surface has).
MACHINE = re.compile(
    r"^\s*(?:##\s*Mission\b|#\s*Mission\b)|"
    r"Base directory for this skill:|"
    r"<(?:system-reminder|command-name|local-command)|"
    r"^\s*\[(?:INFO|OK|WARN|ERROR)\]", re.M)

# Pasted material is CONTEXT, not the ask: a fenced block, a quoted block, or a
# document body under its own headings. The operator's own words are what gets
# scanned; the paste only ever gets to inform, never to trigger.
FENCE = re.compile(r"```.*?```|~~~.*?~~~", re.S)
QUOTED = re.compile(r"(?m)^\s*>.*$")
DOC_BODY = re.compile(r"(?m)^\s{0,3}#{1,6}\s+\S")


def operator_words(text):
    """The prompt with pasted material removed."""
    text = FENCE.sub(" ", text)
    text = QUOTED.sub(" ", text)
    # A document starts at its first heading — everything from there down is the
    # pasted artifact (PRD, spec, report), not the instruction that carries it.
    head = DOC_BODY.search(text)
    if head:
        text = text[: head.start()]
    return text


# Sentence-level co-occurrence. "delete the stale docs and by the way the login
# page is fine" is not an auth change: the category and the verb have to land in
# the SAME sentence before this fires.
SENTENCE_SPLIT = re.compile(r"[.!?\n;]+")
VERB_RE = re.compile(
    r"\b(?:fix|build|create|add|update|refactor|migrate|deploy|implement|remove|"
    r"delete|drop|rename|publish|ship|write|wire|change|switch|move|rewrite|"
    r"corrige|construis|cree|ajoute|mets? a jour|met a jour|refactorise|migre|"
    r"deploie|implemente|supprime|renomme|publie|ecris|branche|change|passe|"
    r"remplace|reecris)\b")

blast_hits = []
scan_text = operator_words(low)
if (
    len(prompt) >= 60
    and len(scan_text.split()) >= 8
    and not SMALL.search(low)
    and not MACHINE.search(prompt)
):
    for sentence in SENTENCE_SPLIT.split(scan_text):
        if not VERB_RE.search(sentence):
            continue
        for name, pat in BLAST:
            if name not in blast_hits and re.search(pat, sentence):
                blast_hits.append(name)

high_blast = bool(blast_hits)

if not multi_ask and not high_blast:
    sys.exit(0)

parts = []

if multi_ask:
    parts.append(
        "## OmegaOS — this prompt looks like SEVERAL asks (%s)\n\n"
        "Before touching anything: ENUMERATE them out loud, one line each, in the operator's "
        "own order (L6 step 1). The asks that get silently dropped are always the last ones.\n"
        "Then TaskCreate one task per ask (R-PLAN) — the finish-guard will refuse to end this "
        "session while any of them is still open, so the plan is what protects you.\n"
        "Then check disjointness: if 3+ of those tasks touch different files, DISPATCH them in "
        "parallel now (R-ORCH) — Workflow, Agent, or `omega spawn-worker` per file scope. "
        "Grinding them one at a time until the turn runs out is the failure L6 names.\n\n"
        "If it turns out to be a single ask, ignore this."
        % ", ".join(detail))

if high_blast:
    parts.append(
        "## OmegaOS — HIGH BLAST RADIUS (%s) — R-PREFLIGHT applies\n\n"
        "1. RESEARCH THE REPO FIRST. Never ask what a minute of searching answers: test "
        "framework, language/runtime version, lint rules, directory layout, existing "
        "abstractions and prior art for this exact concern. Read before you write.\n"
        "2. Then hand the operator a PREFLIGHT and STOP:\n"
        "   - **Goal** — the ask restated in your own words + the acceptance criteria you "
        "will hold yourself to.\n"
        "   - **Blocking questions** — 0 to 3, only where a wrong answer means throwing work "
        "away; each carries your recommended default so \"yes to all\" is a valid reply.\n"
        "   - **Assumptions** — numbered, specific, falsifiable: data shape/volume/trust, "
        "failure behaviour, boundaries and backwards-compat, state/concurrency/idempotency, "
        "environment, what is deliberately out of scope, what will and will not be tested.\n"
        "   - **Plan** — files created or modified, key function/type signatures, the order "
        "of work, and for each real fork the alternative rejected and why (one clause).\n"
        "3. Wait for approval before implementing.\n\n"
        "TWO CARVE-OUTS, so this never deadlocks anything:\n"
        "- PROPORTIONALITY — under ~20 lines with one obvious correct form, just do it. The "
        "full treatment is for a new module, a schema change, or anything touching auth, "
        "money, migrations or deletion.\n"
        "- DISPATCHED SESSIONS (a spawned oracle/worker, nobody watching) NEVER idle at a "
        "prompt: write the preflight into your report/plan, state your assumptions, and "
        "PROCEED (L3). Stop only for a genuinely unsafe unknown — then it is a block-file "
        "plus an escalation, never a silent wait."
        % ", ".join(blast_hits))

if multi_ask and high_blast:
    parts.append(
        "The enumeration above IS step 1 of the preflight **Goal** — write it once, in the "
        "preflight, not twice.")

print(json.dumps({"hookSpecificOutput": {
    "hookEventName": "UserPromptSubmit",
    "additionalContext": "\n\n".join(parts),
}}))
PY
exit 0
