#!/usr/bin/env python3
"""stax_derive.py — dériver le layout Stax DEPUIS le schéma.

Le renversement que ce script encode : la grammaire de panneaux n'est pas une phase
tardive où l'on dessine des écrans, c'est une CONSÉQUENCE du modèle de données. Si
une table existe, elle a un panneau. Si un champ est un `v.id("x")`, c'est une action
open-right vers le panneau de x. Si un champ est un union de literals, ce sont les
statuts, donc les couleurs du board.

Dessiner ces écrans à la main revient à re-décider ce que le schéma a déjà décidé,
et les deux divergent au premier changement.

Ce qui reste humain, et que ce script n'invente jamais : quels panneaux sont des
espaces de premier niveau, ce qu'un inspecteur met en avant, et le sens des couleurs.
Il propose, il marque ses hypothèses, il ne tranche pas à la place du concepteur.

Usage:
    stax_derive.py <dossier-blueprint> [--write]

Sans --write, il imprime ce qu'il produirait. Avec, il écrit dans 10-stax/ :
    panels.json      la carte machine, consommée par plan_build et le runner
    panels.md        la carte lisible
    panels.mmd       le diagramme Mermaid
"""
from __future__ import annotations
import json, re, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "lib"))
from schema_parse import parse_schema, Schema, Table, Field  # noqa: E402

IRREGULAR = {
    "people": "person", "memberships": "membership", "syntheses": "synthesis",
    "entries": "entry", "houseAccounts": "houseAccount", "houseRules": "houseRule",
    "guestPasses": "guestPass", "reciprocity": "reciprocity", "partners": "partner",
}


def singular(name: str) -> str:
    if name in IRREGULAR:
        return IRREGULAR[name]
    if name.endswith("ies"):
        return name[:-3] + "y"
    if name.endswith("sses") or name.endswith("shes") or name.endswith("ches"):
        return name[:-2]
    if name.endswith("s") and not name.endswith("ss"):
        return name[:-1]
    return name


def label_field(t: Table, schema: Schema) -> str:
    """Le champ qui NOMME une ligne dans une liste. C'est ce que l'oeil lit en premier."""
    for cand in ("name", "title", "label", "rule", "body", "guestName", "observation"):
        f = t.field(cand)
        if f and f.kind == "string":
            return cand
    # Une table sans nom propre (une candidature, une appartenance) se nomme par la
    # ligne qu'elle qualifie : « la candidature de X », pas « la candidature #4f2a ».
    prim = schema.primitive
    if prim:
        for fname, ref in t.refs:
            if ref == prim.name and not (t.field(fname) or Field("", "")).optional:
                return f"→{fname}"
    for f in t.fields:
        if f.kind == "string" and f.name not in ("clubId", "tenantId", "userId") \
                and not f.name.endswith("Url") and not f.name.endswith("Id"):
            return f.name
    return "_id"


def owned_by(t: Table, schema: Schema) -> str | None:
    """La table dont celle-ci n'est qu'un détail.

    Si TOUS ses liens obligatoires pointent vers une seule autre table, cette ligne
    n'a pas d'existence propre : une appartenance appartient à une personne, une
    proposition appartient à une candidature. Elle vit dans l'inspecteur de son
    propriétaire, pas dans un espace de premier niveau.
    """
    required = {ref for fname, ref in t.refs
                if not (t.field(fname).optional if t.field(fname) else True)
                and schema.get(ref) and not schema.get(ref).is_doctrine and ref != t.name}
    return next(iter(required)) if len(required) == 1 else None


def status_field(t: Table) -> tuple[str, list[str]] | None:
    """Le champ d'état : un union de literals. Il pilote les couleurs du board."""
    for f in t.fields:
        if f.kind == "union" and f.literals and f.name in (
            "status", "statut", "standing", "decision", "kind", "severity", "category"
        ):
            return (f.name, f.literals)
    for f in t.fields:
        if f.kind == "union" and len(f.literals) >= 2:
            return (f.name, f.literals)
    return None


def derive(schema: Schema) -> dict:
    prim = schema.primitive
    panels, spaces = [], []

    for t in schema.panel_tables:
        ptype = singular(t.name)
        lab = label_field(t, schema)
        st = status_field(t)
        owner = owned_by(t, schema)

        # Les actions open-right : une par référence sortante. C'est le drill-down.
        actions = [{
            "action": "openDetail",
            "via": fname,
            "to": singular(ref),
            "toTable": ref,
            "why": f"le champ {fname} pointe une ligne de {ref}",
        } for fname, ref in t.refs if schema.get(ref) and not schema.get(ref).is_doctrine]

        # Les listes de l'inspecteur : ce qui pointe VERS cette table. Chaque ligne
        # de ces listes est elle-même drillable, ce qui donne la profondeur.
        lists = [{
            "of": singular(src), "ofTable": src, "via": fname,
            "why": f"{src}.{fname} référence cette ligne",
        } for src, fname in schema.incoming(t.name)
            if src != t.name and not schema.get(src).is_doctrine]

        # La timeline : toute table dont une ligne peut être l'acteur d'un signal.
        has_timeline = any(
            src in ("entries",) for src, _ in schema.incoming(t.name)
        )

        # Un espace de premier niveau demande deux choses : être indexé par le tenant,
        # et avoir une IDENTITÉ PROPRE. Une ligne qui ne sait pas se nommer autrement
        # que par celle qu'elle qualifie (label « →personId ») est un détail : une
        # appartenance, une proposition, une arête. Elle vit dans l'inspecteur de son
        # propriétaire. Une demande, un événement, un forum ont un nom à eux.
        # Heuristique assumée : le script PROPOSE, le concepteur tranche. Un OS réel a
        # 5 à 7 espaces, pas un par table.
        indexed_by_tenant = any(ix[1] and ix[1][0] == t.tenant_field for ix in t.indexes)
        has_own_identity = not lab.startswith("→") and lab != "_id"
        is_detail = owner is not None and not has_own_identity
        is_space = indexed_by_tenant and (t.name == prim.name or not is_detail)

        panel = {
            "panelType": ptype,
            "table": t.name,
            "resourceKey": "_id",
            "label": lab,
            "isPrimitive": t.name == prim.name,
            "status": {"field": st[0], "values": st[1]} if st else None,
            "searchable": bool(t.search_indexes),
            "inspector": {
                "fields": [f.name for f in t.fields
                           if f.name not in (t.tenant_field,) and f.kind != "any"][:12],
                "lists": lists,
                "timeline": has_timeline,
            },
            "actions": actions,
            "proposedSpace": is_space,
            "ownedBy": owner if is_detail else None,
        }
        panels.append(panel)
        if is_space:
            spaces.append({"spaceId": t.name, "rootPanel": ptype,
                           "orderedBy": t.indexes[0][0] if t.indexes else None})

    # Le registre d'actions ouvrables par la couche IA. openDetail par type, plus les
    # deux actions qui rendent le produit démonstratif au lieu de narratif.
    registry = [{"action": f"open{p['panelType'][0].upper()}{p['panelType'][1:]}",
                 "params": ["id"], "opens": p["panelType"]} for p in panels]
    registry += [
        {"action": "compare", "params": ["idA", "idB"], "opens": f"{singular(prim.name)} x2",
         "why": "ouvrir deux inspecteurs côte à côte plutôt que raconter une corrélation"},
        {"action": "openPath", "params": ["fromId", "toId"], "opens": "path",
         "why": "montrer le chemin entre deux lignes de la primitive"},
    ]

    return {
        "generatedFrom": schema.source,
        "primitive": prim.name,
        "primitivePanel": singular(prim.name),
        "grammar": {
            "mechanic": "open-right (openDetail). Les sections changent le fil (openSpace).",
            "actionZone": "le footer du panneau, jamais un bouton flottant",
            "back": "close, Esc, ou une miette du fil d'ariane",
            "forbidden": ["pages", "modales", "onglets"],
            "retention": ["preview", "retained"],
            "placement": ["context", "reference"],
        },
        "spaces": spaces,
        "panels": panels,
        "actionRegistry": registry,
        "urlState": {
            "encoded": ["l'espace courant", "la pile de panneaux dans l'ordre",
                        "les références épinglées", "les filtres actifs",
                        "la taille par type de panneau"],
            "notEncoded": ["le contenu (relu depuis Convex)", "les brouillons",
                           "la position de défilement"],
            "warning": "une URL partageable porte des identifiants : l'accès se revalide "
                       "côté serveur à chaque ouverture, l'URL n'est jamais une autorisation",
        },
    }


def to_markdown(d: dict) -> str:
    L = [f"# Layout Stax (dérivé du schéma)\n",
         "*Généré par `stax_derive.py`. Ne pas éditer à la main : régénérer.*\n",
         "> La carte des panneaux n'est pas dessinée, elle est **déduite du modèle de "
         "données**. Une table donne un panneau, un `v.id()` donne une action open-right, "
         "un union de literals donne les statuts et donc les couleurs.\n",
         "\n## La grammaire, non négociable\n",
         f"- **Un mécanisme** : {d['grammar']['mechanic']}",
         f"- **Une zone d'action** : {d['grammar']['actionZone']}",
         f"- **Un retour** : {d['grammar']['back']}",
         f"- **Interdits** : {', '.join(d['grammar']['forbidden'])}\n",
         "\n## Les espaces proposés\n",
         "| Espace | Panneau racine | Ordonné par |", "|---|---|---|"]
    for s in d["spaces"]:
        L.append(f"| `{s['spaceId']}` | `{s['rootPanel']}` | `{s['orderedBy'] or '—'}` |")

    detail_panels = [p for p in d["panels"] if p["ownedBy"]]
    if detail_panels:
        L += ["\nLes panneaux suivants ne sont **pas** des espaces : tous leurs liens "
              "obligatoires pointent une seule table, donc ils n'ont pas d'existence "
              "propre et vivent dans l'inspecteur de leur propriétaire.\n",
              "| Panneau | Détail de |", "|---|---|"]
        for p in detail_panels:
            L.append(f"| `{p['panelType']}` | `{p['ownedBy']}` |")

    L += ["\n## La carte des panneaux\n",
          "| Panneau | Table | Libellé | Statuts | Recherche | Ouvre (via) |",
          "|---|---|---|---|---|---|"]
    for p in d["panels"]:
        st = ", ".join(p["status"]["values"][:4]) + ("…" if p["status"] and len(p["status"]["values"]) > 4 else "") if p["status"] else "—"
        opens = ", ".join(f"`{a['to']}`({a['via']})" for a in p["actions"]) or "—"
        star = " ⭐" if p["isPrimitive"] else ""
        L.append(f"| `{p['panelType']}`{star} | `{p['table']}` | `{p['label']}` | {st} | "
                 f"{'oui' if p['searchable'] else '—'} | {opens} |")

    L += ["\n## Les inspecteurs\n"]
    for p in d["panels"]:
        L.append(f"\n### `{p['panelType']}`")
        L.append(f"\nChamps : {', '.join('`'+f+'`' for f in p['inspector']['fields'])}")
        if p["inspector"]["lists"]:
            L.append("\nListes drillables :\n")
            for l in p["inspector"]["lists"]:
                L.append(f"- **{l['of']}** — {l['why']}")
        if p["inspector"]["timeline"]:
            L.append("\nPorte une **timeline** (des signaux la référencent).")

    L += ["\n## Le registre d'actions ouvrables par l'IA\n",
          "| Action | Paramètres | Ouvre |", "|---|---|---|"]
    for a in d["actionRegistry"]:
        L.append(f"| `{a['action']}` | {', '.join(a['params'])} | {a['opens']} |")

    L += ["\n## L'état d'URL\n", "**Encodé** : " + " · ".join(d["urlState"]["encoded"]),
          "\n**Non encodé** : " + " · ".join(d["urlState"]["notEncoded"]),
          f"\n> ⚠️ {d['urlState']['warning']}\n"]
    return "\n".join(L) + "\n"


def to_mermaid(d: dict) -> str:
    L = ["graph LR"]
    for p in d["panels"]:
        shape = f'(["{p["panelType"]}"])' if p["isPrimitive"] else f'["{p["panelType"]}"]'
        L.append(f'  {p["panelType"]}{shape}')
    seen = set()
    for p in d["panels"]:
        for a in p["actions"]:
            key = (p["panelType"], a["to"])
            if key in seen:
                continue
            seen.add(key)
            L.append(f'  {p["panelType"]} -->|{a["via"]}| {a["to"]}')
    L.append("  classDef prim stroke-width:3px;")
    L.append(f'  class {d["primitivePanel"]} prim;')
    return "\n".join(L) + "\n"


def to_erd(schema: Schema) -> str:
    """L'ERD du schéma. Il ne ment jamais : il est relu du schéma, pas dessiné."""
    L = ["erDiagram"]
    for t in schema.tables:
        L.append(f"  {t.name} {{")
        for f in t.fields[:14]:
            typ = {"id": "id", "string": "string", "number": "number",
                   "boolean": "bool", "array": "array", "union": "enum",
                   "object": "object", "any": "any"}.get(f.kind, "unknown")
            mark = "  PK" if f.name == "_id" else ("  FK" if f.ref else "")
            opt = "_optional" if f.optional else ""
            L.append(f"    {typ}{opt} {f.name}{mark}")
        L.append("  }")
    seen = set()
    for t in schema.tables:
        for fname, ref in t.refs:
            if not schema.get(ref):
                continue
            f = t.field(fname)
            card = "}o--o|" if (f and f.is_array) else ("}o--||" if not (f and f.optional) else "}o--o|")
            key = (t.name, ref, fname)
            if key in seen:
                continue
            seen.add(key)
            L.append(f'  {t.name} {card} {ref} : "{fname}"')
    return "\n".join(L) + "\n"


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__); return 2
    bp = Path(sys.argv[1])
    write = "--write" in sys.argv
    schema_path = bp / "09-data" / "schema.ts"
    if not schema_path.exists():
        print(f"schéma introuvable : {schema_path}", file=sys.stderr); return 2

    schema = parse_schema(schema_path)
    d = derive(schema)
    out = bp / "10-stax"
    out.mkdir(exist_ok=True)

    if write:
        (out / "panels.json").write_text(json.dumps(d, indent=2, ensure_ascii=False), encoding="utf-8")
        (out / "panels.md").write_text(to_markdown(d), encoding="utf-8")
        (out / "panels.mmd").write_text(to_mermaid(d), encoding="utf-8")
        (bp / "09-data" / "schema.mmd").write_text(to_erd(schema), encoding="utf-8")
        print(f"écrit : {out}/panels.json · panels.md · panels.mmd")
        print(f"écrit : {bp}/09-data/schema.mmd (ERD)")
    else:
        print(to_markdown(d))

    print(f"\n{len(d['panels'])} panneaux · {len(d['spaces'])} espaces proposés · "
          f"{len(d['actionRegistry'])} actions · primitive = {d['primitivePanel']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
