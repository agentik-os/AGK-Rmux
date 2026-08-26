#!/usr/bin/env python3
"""plan_build.py — compiler un blueprint en PLAN D'EXÉCUTION.

Le blueprint dit ce qu'il faut construire. Il ne dit pas dans quel ordre, ni quand
une étape est finie, ni ce qu'il ne faut surtout pas toucher en la faisant. Tant que
ça reste du markdown, l'agent relit tout, décide de l'ordre à chaque fois, et se
déclare fini sur une impression.

Ce script transforme le blueprint en étapes typées. Chaque étape porte les QUATRE
BLOCS de la doctrine Stax, et le troisième est le seul qui compte vraiment :

  1. objectif        ce que ça doit permettre
  2. contraintes     stack, dépendances, invariants
  3. definitionOfDone  MÉCANIQUEMENT VÉRIFIABLE : une commande, pas une opinion
  4. doNotTouch      les fichiers hors périmètre, ce que tout le monde oublie
                     et ce qui évite les diffs de 900 lignes

Le bloc 3 décide aussi de la voie d'exécution : vérifiable par une machine, l'étape
part en lot autonome ; sinon un agent à la fois, avec un humain devant.

Un noeud dont les quatre blocs ne sont pas remplis est ROUGE, et le rouge est
bloquant. C'est ce qui empêche de lancer un agent sur du flou et de récupérer
900 lignes inutilisables.

Usage:
    plan_build.py <dossier-blueprint> [--write]
"""
from __future__ import annotations
import json, re, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "lib"))
from schema_parse import parse_schema, Schema  # noqa: E402

VERSIONS = ["v0", "v1", "v2", "v3", "v4", "v5"]


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.exists() else ""


def _first_md(d: Path) -> Path | None:
    if not d.is_dir():
        return None
    for f in sorted(d.glob("*.md")):
        if f.name != "README.md":
            return f
    return None


def parse_features(bp: Path) -> list[dict]:
    """Les lignes du tableau input → action → output de la phase 06.

    Format attendu : | Feature | Input | Action | Output | Signal | Couche | v |
    Une ligne qu'on ne sait pas lire est signalée, jamais devinée.
    """
    f = _first_md(bp / "06-features")
    if not f:
        return []
    out, unreadable = [], 0
    for line in _read(f).splitlines():
        if not line.startswith("|") or line.startswith("|---"):
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if len(cells) < 7:
            continue
        name, inp, act, outp, signal, layer, ver = cells[:7]
        if name.lower() in ("feature", "") or "---" in name:
            continue
        vm = re.search(r"v[0-9]", ver)
        if not vm:
            unreadable += 1
            continue
        lay = ("differenciant" if "🟣" in layer else
               "parite" if "🟠" in layer else
               "socle" if "🔵" in layer else "refuse" if "❌" in layer else None)
        if lay in (None, "refuse"):
            continue
        out.append({
            "name": re.sub(r"\*\*|`", "", name),
            "input": inp, "action": act, "output": outp,
            "signal": re.sub(r"`", "", signal),
            "layer": lay, "version": vm.group(0),
        })
    if unreadable:
        print(f"  ! {unreadable} ligne(s) de features illisibles (version absente) — ignorées",
              file=sys.stderr)
    return out


def parse_agents(bp: Path) -> list[dict]:
    """Les agents de la phase 08b : un bloc ``` par system prompt rédigé."""
    f = bp / "08-ia" / "system-prompts.md"
    if not f.exists():
        f = _first_md(bp / "08-ia")
    if not f or not f.exists():
        return []
    src = _read(f)
    agents = []
    for m in re.finditer(r"^##\s*Agent\s*\d+\s*[—-]\s*(.+?)\s*(?:\(([^)]*)\))?\s*$",
                         src, re.M):
        title = m.group(1).strip()
        meta = (m.group(2) or "")
        lvl = re.search(r"niveau\s*(\d)", meta)
        model = re.search(r"(Opus|Sonnet|Haiku|Fable)[^,)]*", meta)
        body = src[m.end():]
        nxt = re.search(r"^##\s", body, re.M)
        if nxt:
            body = body[:nxt.start()]
        has_prompt = "```" in body
        agents.append({
            "name": title,
            "level": int(lvl.group(1)) if lvl else None,
            "model": model.group(0).strip() if model else None,
            "emptyOutputAllowed": bool(re.search(r"SORTIE VIDE AUTORIS", body)),
            "citesSources": bool(re.search(r"TU CITES", body)),
            "neverDoes": bool(re.search(r"CE QUE TU NE FAIS JAMAIS", body)),
            "promptWritten": has_prompt,
        })
    return agents


def parse_automations(bp: Path) -> list[dict]:
    f = _first_md(bp / "07-automatisations")
    if not f:
        return []
    out = []
    for line in _read(f).splitlines():
        if not line.startswith("| A") or "---" in line:
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if len(cells) < 6:
            continue
        out.append({"id": cells[0], "name": re.sub(r"\*\*", "", cells[1]),
                    "trigger": cells[2], "decision": re.sub(r"\*\*", "", cells[-1])})
    return out


def step(sid, title, stype, version, objective, constraints, dod, dnt, deps=None,
         files=None, source=None, notes=None) -> dict:
    """Un noeud du plan. Les 4 blocs, sinon il est rouge."""
    blocks_filled = all([objective, constraints, dod, dnt])
    return {
        "id": sid, "title": title, "type": stype, "version": version,
        "status": "todo" if blocks_filled else "incomplete",
        "objective": objective,
        "constraints": constraints,
        "definitionOfDone": dod,          # {"check": "...", "machine": bool}
        "doNotTouch": dnt,
        "dependsOn": deps or [],
        "files": files or [],
        "source": source,
        "notes": notes,
        "lane": "autonomous" if (dod or {}).get("machine") else "supervised",
    }


def build(bp: Path) -> dict:
    schema_path = bp / "09-data" / "schema.ts"
    schema = parse_schema(schema_path) if schema_path.exists() else Schema()
    panels_path = bp / "10-stax" / "panels.json"
    panels = json.loads(_read(panels_path)) if panels_path.exists() else None
    manifest = json.loads(_read(bp / "blueprint.json") or "{}")

    prim = schema.primitive.name if schema.primitive else None
    tenant = schema.primitive.tenant_field if schema.primitive else "tenantId"
    steps: list[dict] = []

    # ── 1. Le socle technique ────────────────────────────────────────────────
    steps.append(step(
        "base-scaffold", "Scaffolder l'app sur la stack canonique", "scaffold", "v0",
        "Une app Next.js + Convex + Clerk + Stax qui démarre, avec Stax vendoré et tracé.",
        ["/stack fait le travail, ne pas scaffolder à la main",
         "Stax est pull avant, le commit va dans stax.lock.json"],
        # Le typecheck ignore convex/ : tant que `npx convex dev` n'a pas tourné une
        # fois, convex/_generated n'existe pas, tout y est `any` et les erreurs sont
        # attendues. Les compter ferait échouer un scaffold parfaitement sain.
        {"check": "test -f stax.lock.json && test -d src/stax && "
                  "! ( npx tsc --noEmit 2>&1 | grep -v '^convex/' | grep -q 'error TS' )",
         "machine": True,
         "note": "convex/ est exclu du typecheck jusqu'au premier `npx convex dev`"},
        ["tout le reste : cette étape ne crée que le squelette"],
        files=["stax.lock.json", "src/stax/", "convex/schema.ts"],
        source="phase 09 + /stack",
    ))
    steps.append(step(
        "base-schema", f"Poser le schéma Convex ({len(schema.tables)} tables)", "schema", "v0",
        f"Le modèle de données du blueprint, primitive `{prim}` en première table.",
        [f"`{tenant}` sur chaque table", "`entries` et `syntheses` jamais mélangées",
         "un index par requête réelle, aucun index spéculatif",
         "jamais d'index sur un champ tableau : il se construit et ne filtre pas"],
        {"check": "bash $OMEGA_DIR/skills/blueprint-os/scripts/convex-validate.sh convex/schema.ts",
         "machine": True},
        ["src/app/ : le schéma ne touche pas l'interface"],
        deps=["base-scaffold"], files=["convex/schema.ts"], source="phase 09",
    ))
    steps.append(step(
        "base-auth", "Brancher Clerk sur Convex", "auth", "v0",
        "Une identité vérifiée côté Convex, et le tenant dérivé d'elle, jamais du client.",
        ["template JWT nommé exactement `convex` dans le dashboard Clerk",
         f"`{tenant}` vient TOUJOURS de l'identité, jamais d'un argument client",
         "CLERK_JWT_ISSUER_DOMAIN posé aussi sur le déploiement Convex"],
        {"check": "grep -q 'getUserIdentity' convex/*.ts && grep -rq 'ConvexProviderWithClerk' src/",
         "machine": True},
        ["convex/schema.ts"],
        deps=["base-schema"], files=["convex/auth.config.ts", "src/middleware.ts",
                                     "src/app/providers.tsx"],
        source="phase 09 + references/clerk.md",
        notes="Le template JWT est l'étape que tout le monde oublie et qui coûte une demi-journée.",
    ))

    # ── 2. Le shell Stax ─────────────────────────────────────────────────────
    if panels:
        steps.append(step(
            "stax-shell", "Monter le shell et le workspace", "stax", "v0",
            "Un rail horizontal de panneaux dérivé d'un seul état sérialisable, URL comprise.",
            ["un seul mécanisme : open-right", "la zone d'action est le footer",
             "NI pages, NI modales, NI onglets",
             "tout dérive de l'état : fil d'ariane, URL, persistance"],
            {"check": "grep -rq 'openDetail' src/stax && grep -rq 'openSpace' src/stax",
             "machine": True},
            ["convex/ : le shell ne touche pas la donnée"],
            deps=["base-scaffold"], files=["src/stax/"], source="phase 10",
        ))
        for sp in panels["spaces"]:
            steps.append(step(
                f"space-{sp['spaceId']}", f"Espace `{sp['spaceId']}`", "space", "v1",
                f"Le fil de premier niveau qui liste les `{sp['rootPanel']}`.",
                [f"ordonné par l'index `{sp['orderedBy']}`",
                 "openSpace change le fil, il n'empile pas"],
                {"check": f"grep -rq '{sp['spaceId']}' src/stax/registry.tsx", "machine": True},
                ["les autres espaces"],
                deps=["stax-shell", "base-schema"], source="phase 10",
            ))
        for p in panels["panels"]:
            deps = ["stax-shell", "base-schema"]
            dnt = ["convex/schema.ts", "les autres panneaux"]
            steps.append(step(
                f"panel-{p['panelType']}", f"Panneau `{p['panelType']}`", "panel",
                "v1" if p["isPrimitive"] else "v2",
                f"Ouvrir une ligne de `{p['table']}` à droite, le parent restant visible.",
                [f"resourceKey = {p['resourceKey']}",
                 f"libellé = `{p['label']}`",
                 f"{len(p['actions'])} action(s) open-right",
                 "le footer porte les actions, jamais un bouton flottant"],
                {"check": f"grep -q '\"{p['panelType']}\"' src/stax/registry.tsx", "machine": True},
                dnt, deps=deps, source="phase 10 (dérivé du schéma)",
                notes=(f"détail de `{p['ownedBy']}` : vit dans son inspecteur, pas dans un espace"
                       if p.get("ownedBy") else None),
            ))

    # ── 3. Les features, par version et par couche ───────────────────────────
    feats = parse_features(bp)
    for i, f in enumerate(feats):
        sid = f"feat-{re.sub(r'[^a-z0-9]+', '-', f['name'].lower()).strip('-')[:38]}-{i}"
        machine = bool(f["signal"] and f["signal"] != "—")
        steps.append(step(
            sid, f["name"], "feature", f["version"],
            f"{f['input']} → {f['action']} → {f['output']}",
            [f"couche : {f['layer']}",
             f"émet le signal `{f['signal']}` dans `entries`" if machine
             else "AUCUN signal émis : vérifier que ce n'est pas du poids mort"],
            {"check": (f"grep -rq '{f['signal']}' convex/" if machine
                       else "revue humaine : la feature n'émet aucun signal"),
             "machine": machine},
            ["le schéma", "les features des autres versions"],
            deps=["base-schema"] + (["base-auth"] if f["layer"] != "socle" else []),
            source="phase 06",
        ))

    # ── 4. Les agents IA ─────────────────────────────────────────────────────
    for a in parse_agents(bp):
        sid = "agent-" + re.sub(r"[^a-z0-9]+", "-", a["name"].lower()).strip("-")[:40]
        ready = a["promptWritten"] and a["emptyOutputAllowed"] and a["citesSources"] and a["neverDoes"]
        steps.append(step(
            sid, f"Agent : {a['name']}", "agent",
            "v2" if (a["level"] or 2) >= 2 else "v1",
            f"Un agent de niveau {a['level'] or '?'} appelé depuis une action Convex.",
            ["le modèle est appelé depuis une ACTION Convex, jamais du client",
             "la sortie est structurée et validée AVANT écriture",
             "elle est écrite dans `syntheses`, jamais dans les tables métier",
             "chaque affirmation porte ses citations",
             "sortie vide = cas nominal, pas une erreur"]
            + (["niveau 3 : confirmation humaine SYSTÉMATIQUE avant tout effet"]
               if a["level"] == 3 else []),
            {"check": f"grep -rq 'syntheses' convex/ai/ && grep -rq 'citations' convex/ai/",
             "machine": True} if ready else
            {"check": "system prompt incomplet : rédiger rôle, sortie vide, citations, interdits",
             "machine": False},
            ["les tables métier : un agent écrit dans syntheses",
             "les autres agents"],
            deps=["base-schema", "base-auth"], source="phase 08b",
            notes=None if ready else "PROMPT INCOMPLET : il manque un des blocs obligatoires",
        ))

    # ── 5. Les automatisations ───────────────────────────────────────────────
    for au in parse_automations(bp):
        sid = "auto-" + au["id"].lower()
        steps.append(step(
            sid, f"{au['id']} · {au['name']}", "automation", "v2",
            f"Déclencheur : {au['trigger']}. Décision touchée : {au['decision']}",
            ["une automatisation qui ne touche aucune décision n'en est pas une",
             "niveau 2 propose et ne touche à rien ; niveau 3 demande confirmation"],
            {"check": f"grep -rq '{au['id'].lower()}' convex/", "machine": True},
            ["les autres automatisations"],
            deps=["base-schema"], source="phase 07",
        ))

    # ── 6. La sortie ─────────────────────────────────────────────────────────
    by_version: dict[str, int] = {}
    for s in steps:
        by_version[s["version"]] = by_version.get(s["version"], 0) + 1

    return {
        "blueprint": manifest.get("name") or bp.name,
        "primitive": manifest.get("primitive") or prim,
        "generatedFrom": str(bp),
        "gates": manifest.get("gates", {}),
        "counts": {
            "total": len(steps),
            "byVersion": {v: by_version.get(v, 0) for v in VERSIONS if by_version.get(v)},
            "byType": {t: sum(1 for s in steps if s["type"] == t)
                       for t in sorted({s["type"] for s in steps})},
            "incomplete": sum(1 for s in steps if s["status"] == "incomplete"),
            "autonomous": sum(1 for s in steps if s["lane"] == "autonomous"),
            "supervised": sum(1 for s in steps if s["lane"] == "supervised"),
        },
        "steps": steps,
    }


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__); return 2
    bp = Path(sys.argv[1])
    if not bp.is_dir():
        print(f"blueprint introuvable : {bp}", file=sys.stderr); return 2

    plan = build(bp)
    if "--write" in sys.argv:
        out = bp / "plan"
        out.mkdir(exist_ok=True)
        (out / "plan.json").write_text(json.dumps(plan, indent=2, ensure_ascii=False),
                                       encoding="utf-8")
        print(f"écrit : {out}/plan.json")

    c = plan["counts"]
    print(f"\n{c['total']} étapes · " +
          " · ".join(f"{v}:{n}" for v, n in c["byVersion"].items()))
    print("types : " + " · ".join(f"{t}:{n}" for t, n in c["byType"].items()))
    print(f"voies : {c['autonomous']} autonomes · {c['supervised']} supervisées")
    if c["incomplete"]:
        print(f"\n\033[31m{c['incomplete']} étape(s) ROUGE\033[0m — "
              "les 4 blocs ne sont pas remplis, elles sont bloquantes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
