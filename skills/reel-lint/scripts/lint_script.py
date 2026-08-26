#!/usr/bin/env python3
"""
Linter de viralité pour scripts de reels — score /100 AVANT tournage.

Règles calibrées sur des patterns de reels mesurés à +100K vues
(hooks, rétention, format téléprompteur, CTA, outil branché).

Usage :
  python3 lint_script.py <fichier.md>          # extrait la section SCRIPT FACE CAM
  python3 lint_script.py --text "…script…"     # texte brut
  python3 lint_script.py <f1.md> <f2.md>       # comparer plusieurs variantes

Gratuit, instantané, zéro dépendance (stdlib uniquement).
"""

import re
import sys
import unicodedata

# ── Constantes calibrées ────────────────────────────────────────────────────
WORDS_PER_SEC = 2.9          # débit téléprompteur FR mesuré (~175 mots ≈ 60s)
MAX_WORDS = 175              # format 50-60s
HOOK_WORDS = 30              # ~ les 10 premières secondes parlées

# Mots impactants ET simples — matching mot entier.
# NB : « le plus fou » du cliffhanger est scoré à part (CLIFFHANGERS), pas ici.
POWER_WORDS = [
    "littéralement", "fou", "folle", "fous", "dingue", "multitude", "énorme",
    "retourné le cerveau", "inconnu", "puissance", "gigantesque",
    "cheat code", "triche pure", "méconnu", "méconnus", "brutal",
    "personne ne", "le truc que personne", "explosé", "record",
]

CLIFFHANGERS = [
    "mais attends", "attends attends", "ce n'est même pas le plus fou",
    "c'est même pas le plus fou", "et ce n'est pas tout",
    "voici ce qu'il se passe", "le plus fou arrive",
    "le pire, c'est", "le pire c'est",           
]

ROLE_METAPHORS = [
    "employé", "associé", "équipe", "agence", "salarié", "stagiaire",
    "consultant", "studio", "usine",
]

# Patterns P1-P11 de la bibliothèque (heuristiques de détection sur le hook)
PATTERNS = [
    ("P1  STOP + douleur (607K)",            r"^(arrête|stop)\b"),
    ("P2  contraste financier ⭐ filon n°1",  r"(dollars?|euros?|€|\$|paient?|paie|facture|prix|gratuit)"),
    ("P3  contrarian (427K)",                r"(la plupart des gens|tout le monde (pense|croit|fait))"),
    ("P4  « tout le monde parle » (1,49M)",  r"tout le monde parle"),
    ("P5  menace métier actu (1,09M)",       r"vient de (sortir|lancer).*(ia|agent)|se faire remplacer"),
    ("P6  combo cheat code (8,7M 🥇)",       r"(\+ *claude|claude *\+|branché|fusionné)"),
    ("P7  gros chiffre choc (724K)",         r"\d{3}[  ]?\d{3}|\d+ ?(k|m|millions?|milliers?) ?(de )?vues"),
    ("P8  autorité + open source (167K)",    r"(ceo|open.?sourc|vient de rendre public)"),
    ("P9  série numérotée (8,7M)",           r"jour \d+"),
    ("P10 listicle + prix", r"\d+ (réglages|astuces|commandes|skills|façons).*(méconnu|dollars?|€)"),
    ("P11 KILL — le KILL (260K/153K)",  r"(vient de tuer|j'ai tué|a tué)"),
]
TOP_PATTERNS = {"P2", "P11", "P6", "P4"}   # filons prioritaires mesurés


def strip_accents(s: str) -> str:
    return "".join(c for c in unicodedata.normalize("NFD", s) if unicodedata.category(c) != "Mn")


def extract_script(path: str) -> str:
    """Extrait la section SCRIPT FACE CAM d'un fichier .md de reel, sinon tout le texte."""
    text = open(path, encoding="utf-8").read()
    m = re.search(r"^##.*SCRIPT FACE CAM.*$", text, re.MULTILINE)
    if not m:
        return text
    body = text[m.end():]
    end = re.search(r"^(---|## )", body, re.MULTILINE)
    if end:
        body = body[: end.start()]
    lines = [l for l in body.splitlines() if l.strip() and not l.strip().startswith(">")]
    return "\n".join(lines)


def lint(script: str, label: str) -> int:
    lines = [l.strip() for l in script.splitlines() if l.strip()]
    if len(lines) <= 1:  # texte brut sans retours à la ligne → découper en phrases
        lines = [s.strip() for s in re.split(r"[.!?]+", script) if s.strip()]
    words = script.split()
    n_words = len(words)
    duration = n_words / WORDS_PER_SEC
    low = script.lower()
    hook = " ".join(words[:HOOK_WORDS]).lower()

    checks = []   # (points_obtenus, points_max, label, détail)

    # ── HOOK /35 ──
    m = re.search(r"\d[\d  ]*", hook)
    pts = 10 if m and hook.index(m.group()) < len(" ".join(words[:12])) else (5 if m else 0)
    checks.append((pts, 10, "Chiffre concret dès la 1ʳᵉ phrase",
                   f"« {m.group().strip()} »" if m else "aucun chiffre dans le hook"))

    detected = [(name, rx) for name, rx in PATTERNS if re.search(rx, hook)]
    if detected:
        is_top = any(name.split()[0] in TOP_PATTERNS for name, _ in detected)
        pts = 10 if is_top else 7
        detail = " · ".join(name for name, _ in detected)
    else:
        pts, detail = 0, "aucun pattern P1-P11 reconnu dans le hook"
    checks.append((pts, 10, "Pattern de hook mesuré (P1-P11)", detail))

    open_loop = any(k in low for k in ["voici ce qu'il se passe", "je vais te montrer", "voici comment"])
    checks.append((5 if open_loop else 0, 5, "Boucle ouverte / promesse",
                   "présente" if open_loop else "manque « je vais te montrer… » / « voici ce qu'il se passe »"))

    fp = re.search(r"j'ai (testé|reconstruit|branché|cloné|remplacé|construit)|je tape|j'ai tué", low)
    checks.append((5 if fp else 0, 5, "First-person testé",
                   f"« {fp.group()} »" if fp else "manque le « j'ai testé/reconstruit… »"))

    money = re.search(r"(dollars?|euros?|€|\$)", hook)
    checks.append((5 if money else 0, 5, "Argent/désir dans le hook",
                   "présent" if money else "pas de montant dans les 30 premiers mots"))

    # ── RÉTENTION /25 ──
    cliff = [c for c in CLIFFHANGERS if c in low]
    checks.append((10 if cliff else 0, 10, "Cliffhanger de rétention",
                   f"« {cliff[0]} »" if cliff else "manque « mais attends… le plus fou » entre les étapes"))

    promised = re.search(r"en (\d+) étapes", low)
    n_steps = len(re.findall(r"étape \d+", low))
    if promised and n_steps == int(promised.group(1)):
        pts, detail = 10, f"promesse « en {promised.group(1)} étapes » tenue ({n_steps} étapes)"
    elif n_steps:
        pts, detail = 6, f"{n_steps} étapes trouvées mais promesse « en N étapes » " + \
                         (f"= {promised.group(1)} (incohérent)" if promised else "absente du hook")
    else:
        pts, detail = 0, "pas d'étapes numérotées"
    checks.append((pts, 10, "Étapes numérotées annoncées et tenues", detail))

    low_no_cliff = low
    for c in CLIFFHANGERS:                       # le cliffhanger est déjà scoré à part
        low_no_cliff = low_no_cliff.replace(c, " ")
    pw_rx = "|".join(re.escape(w) for w in sorted(POWER_WORDS, key=len, reverse=True))
    pw = re.findall(rf"\b(?:{pw_rx})\b", low_no_cliff)
    pts = 5 if 1 <= len(pw) <= 3 else (3 if len(pw) > 3 else 0)
    checks.append((pts, 5, "Power words simples et impactants (1-3×)",
                   " · ".join(pw) if pw else "aucun (littéralement, FOU, multitude…)"))

    # ── FORMAT /20 ──
    if n_words <= MAX_WORDS:
        pts, detail = 10, f"{n_words} mots ≈ {duration:.0f}s ✓ (format 50-60s)"
    elif n_words <= MAX_WORDS * 1.2:
        pts, detail = 5, f"{n_words} mots ≈ {duration:.0f}s — un peu long, viser ≤ {MAX_WORDS}"
    else:
        pts, detail = 0, f"{n_words} mots ≈ {int(duration//60)}min{duration%60:02.0f}s — TROP LONG (max {MAX_WORDS} mots)"
    checks.append((pts, 10, "Durée", detail))

    avg = n_words / max(len(lines), 1)
    checks.append((5 if avg <= 9 else (3 if avg <= 12 else 0), 5, "Phrases courtes (téléprompteur)",
                   f"{avg:.1f} mots/ligne en moyenne (cible ≤ 9)"))

    has_accents = script != strip_accents(script)
    checks.append((5 if has_accents else 0, 5, "Accents français",
                   "présents" if has_accents else "AUCUN accent — règle absolue violée"))

    # ── CTA /10 ──
    cta_ok = "commente" in low and "abonne" in low
    checks.append((5 if cta_ok else 0, 5, "CTA commente + abonne-toi",
                   "présent" if cta_ok else "manque « commente » + « abonne-toi »"))

    kw = re.search(r"[Cc]ommente\s+(?:le mot\s+)?([A-ZÀ-Ü]{3,})\b", script)
    checks.append((0 if kw else 5, 5, "Pas de mot-clé imposé dans le CTA",
                   f"« commente {kw.group(1)} » = ancienne règle, remplacer par « commente ce que tu veux »" if kw else "conforme"))

    # ── VOIX /10 ──
    tu = re.search(r"\b(tu|toi|ton|tes)\b", low)
    checks.append((5 if tu else 0, 5, "Tutoiement", "présent" if tu else "absent"))

    role = [r for r in ROLE_METAPHORS if re.search(rf"\b{r}\b", low)]
    checks.append((5 if role else 0, 5, "Métaphore IA = rôle humain",
                   " · ".join(role) if role else "manque « employé / associé / studio… »"))

    # ── PUISSANCE /5 — Claude Code branché à un outil concret ──
    builtin_cmds = {"/model", "/clear", "/compact", "/agents", "/plugin"}
    tools = re.findall(r"\b[a-z0-9_.-]*[a-z][a-z0-9_.-]*/[a-z0-9_.-]*[a-z][a-z0-9_.-]*\b", low)   # repo slug
    tools += [s for s in re.findall(r"(?<![\w/])/[a-z][a-z0-9-]{2,}\b", low) if s not in builtin_cmds]  # skills
    tools += re.findall(r"\bmcp\b|\bplugins?\b|\bgithub\b|open.?source", low)
    tools = sorted(set(tools))
    checks.append((5 if tools else 0, 5, "Outil branché (repo GitHub / MCP / plugin / skill)",
                   " · ".join(tools[:6]) if tools else
                   "aucun outil concret connecté à Claude Code — nommer un repo, un MCP, un plugin ou un skill"))

    # ── Rapport ──
    total = sum(p for p, _, _, _ in checks)
    total_max = sum(mx for _, mx, _, _ in checks)
    score = round(100 * total / total_max)

    print(f"\n{'═'*74}\n🧪 LINTER DE VIRALITÉ — {label}\n{'═'*74}")
    print(f"   {n_words} mots · ~{duration:.0f}s au téléprompteur · {len(lines)} lignes\n")
    sections = [("HOOK", 0, 5), ("RÉTENTION", 5, 8), ("FORMAT", 8, 11), ("CTA", 11, 13),
                ("VOIX", 13, 15), ("PUISSANCE", 15, 16)]
    for name, a, b in sections:
        sub = checks[a:b]
        got, mx = sum(p for p, *_ in sub), sum(m for _, m, *_ in sub)
        print(f"  {name} — {got}/{mx}")
        for p, m, lbl, detail in sub:
            icon = "✅" if p == m else ("⚠️ " if p > 0 else "❌")
            print(f"    {icon} {lbl} ({p}/{m}) — {detail}")
    blocking = []
    if n_words > MAX_WORDS * 1.2:
        blocking.append(f"durée ({n_words} mots — il faut COUPER, pas ajuster)")
    if not has_accents:
        blocking.append("accents absents (règle absolue)")

    print(f"\n  SCORE GLOBAL : {score}/100")
    if blocking:
        print(f"  🔴 VERDICT : RÉÉCRIRE — règle bloquante violée : {' · '.join(blocking)}.")
    elif score >= 85:
        print("  🟢 VERDICT : GO TOURNAGE — le script coche tous les patterns mesurés.")
    elif score >= 70:
        print("  🟡 VERDICT : corrections mineures — corriger les ❌ ci-dessus avant tournage.")
    else:
        print("  🔴 VERDICT : réécrire — le script ne coche pas les fondamentaux mesurés.")
    return score


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        sys.exit(1)
    if args[0] == "--text":
        lint(" ".join(args[1:]), "texte brut")
        return
    for path in args:
        lint(extract_script(path), path.split("/")[-1])


if __name__ == "__main__":
    main()
