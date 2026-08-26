#!/usr/bin/env python3
"""stax_emit.py — écrire le CODE de l'interface Stax, pas seulement sa carte.

`stax_derive.py` déduit du schéma quels panneaux existent. Ce script écrit les
fichiers React qui les implémentent, dans la grammaire Stax : un mécanisme
(ouvrir à droite), une zone d'action (le pied), un retour (fermer).

Ce qu'il génère est du code de DÉPART, pas du code fini. Il pose la structure que
personne n'a envie de retaper vingt fois (le registre, les types, l'anatomie des
panneaux, les drills dérivés des références du schéma) et laisse le contenu de
chaque inspecteur au concepteur. Générer le contenu serait deviner ce que le
produit veut montrer, et une devinette est plus chère qu'un trou.

Ce qui reste à la main, volontairement :
  - ce qu'un inspecteur met en avant, et dans quel ordre
  - le sens des couleurs et des statuts
  - les actions du pied, qui sont des décisions produit

Usage:
    stax_emit.py <dossier-blueprint> --out <dossier-app/src/stax-ui> [--force]
"""
from __future__ import annotations
import json, sys
from pathlib import Path

HEADER = """/**
 * GÉNÉRÉ par stax_emit.py depuis le schéma du blueprint. Point de départ, pas
 * point d'arrivée : la structure est posée, le contenu des inspecteurs est à
 * vous. Régénérer écrase ce fichier, sauf s'il a été modifié (voir --force).
 *
 * La grammaire, non négociable :
 *   un mécanisme   ouvrir à droite (openDetail). Les sections changent le fil.
 *   une zone d'action   le pied du panneau, jamais un bouton flottant
 *   un retour      fermer, Échap, ou une miette du fil d'Ariane
 *   interdits      pages, modales, onglets
 */
"""


def emit_types(d: dict) -> str:
    kinds = " | ".join(f'"{p["panelType"]}"' for p in d["panels"])
    sizes = "\n".join(
        f'  {p["panelType"]}: "{"L" if p["isPrimitive"] else ("M" if p["proposedSpace"] else "S")}",'
        for p in d["panels"]
    )
    titles = "\n".join(f'  {p["panelType"]}: "{p["panelType"]}",' for p in d["panels"])
    spaces = "\n".join(
        f'  {{ spaceId: "{s["spaceId"]}", rootPanel: "{s["rootPanel"]}" }},'
        for s in d["spaces"]
    )
    return f"""{HEADER}
export type PanelKind = {kinds};

export interface PanelTarget {{
  kind: PanelKind;
  /** L'_id de la ligne. La cible est (type, ressource), toujours. */
  resourceKey: string;
}}

/** La taille appartient au GENRE du panneau, jamais à son contenu. */
export const PANEL_SIZE: Record<PanelKind, "S" | "M" | "L"> = {{
{sizes}
}};

/** À remplacer par les libellés du métier : ce sont les noms de table pour l'instant. */
export const PANEL_TITLE: Record<PanelKind, string> = {{
{titles}
}};

/** Les espaces de premier niveau. Proposés par la dérivation, à trancher. */
export const SPACES = [
{spaces}
] as const;
"""


def emit_registry(d: dict) -> str:
    imports = "\n".join(
        f'import {{ {p["panelType"].capitalize()}Panel }} from "./panels/{p["panelType"]}";'
        for p in d["panels"]
    )
    entries = "\n".join(
        f'  {p["panelType"]}: {p["panelType"].capitalize()}Panel,' for p in d["panels"]
    )
    actions = "\n".join(
        f'  {{ action: "{a["action"]}", params: {json.dumps(a["params"])}, opens: "{a["opens"]}" }},'
        for a in d["actionRegistry"]
    )
    return f"""{HEADER}
import type {{ PanelKind }} from "./types";
{imports}

export const PANEL_COMPONENT: Record<PanelKind, React.ComponentType<{{ resourceKey: string }}>> = {{
{entries}
}};

/**
 * Le registre d'actions ouvrables par la couche IA.
 *
 * C'est ce qui permet à un agent de MONTRER une corrélation au lieu de la
 * raconter : il ouvre deux inspecteurs côte à côte et laisse voir.
 */
export const ACTION_REGISTRY = [
{actions}
] as const;
"""


def emit_panel(p: dict, d: dict) -> str:
    name = p["panelType"].capitalize()
    fields = p["inspector"]["fields"]
    lists = p["inspector"]["lists"]
    actions = p["actions"]

    field_rows = "\n".join(
        f'      <div className="kv"><span className="k">{f}</span>'
        f'<span className="v">{{String(row.{f} ?? "—")}}</span></div>'
        for f in fields[:8]
    )
    drill_rows = "\n".join(
        f"""      {{/* {a["why"]} */}}
      {{row.{a["via"]} && (
        <button className="drill" onClick={{() => open({{ kind: "{a["to"]}", resourceKey: String(row.{a["via"]}) }})}}>
          <span className="bd"><span className="tt">{a["to"]}</span></span>
          <span className="arr">→</span>
        </button>
      )}}"""
        for a in actions if not a["via"].endswith("s")  # les tableaux se listent, ils ne drillent pas
    )
    list_blocks = "\n".join(
        f"""      <div className="section-label">{l["of"]}</div>
      {{/* {l["why"]} — chaque ligne est elle-même drillable */}}
      <p className="empty">Brancher la requête : {l["ofTable"]} where {l["via"]} = cette ligne.</p>"""
        for l in lists[:4]
    )
    status = p.get("status")
    status_note = (
        f'\n      {{/* statuts : {", ".join(status["values"])} — leurs couleurs sont une décision produit */}}'
        if status else ""
    )

    return f"""{HEADER}
"use client";

import type {{ PanelTarget }} from "../types";

/**
 * Panneau `{p["panelType"]}` — table `{p["table"]}`.
 *
 * Libellé : `{p["label"]}`{" (détail de " + p["ownedBy"] + ")" if p.get("ownedBy") else ""}
 * {len(actions)} action(s) open-right · {len(lists)} liste(s) drillable(s)
 *
 * À FAIRE : remplacer `row` par la vraie requête, et choisir ce que l'inspecteur
 * met en avant. L'ordre des champs ci-dessous est celui du schéma, pas celui du
 * regard.
 */
export function {name}Panel({{ resourceKey }}: {{ resourceKey: string }}) {{
  // const row = useQuery(api.{p["table"]}.get, {{ id: resourceKey }});
  const row = {{}} as Record<string, unknown>;
  const open = (_t: PanelTarget) => {{ /* branché par le workspace */ }};

  return (
    <>
      <div className="panel-body">{status_note}
{field_rows}

{drill_rows}

{list_blocks}
      </div>
      <div className="panel-foot">
        {{/* LA zone d'action. Jamais un bouton flottant ailleurs. */}}
        <span className="foot-note">{p["table"]} · {resource_note(p)}</span>
      </div>
    </>
  );
}}
"""


def resource_note(p: dict) -> str:
    if p.get("ownedBy"):
        return f"vit dans l'inspecteur de {p['ownedBy']}"
    if p["proposedSpace"]:
        return "espace de premier niveau"
    return "panneau de détail"


def main() -> int:
    if len(sys.argv) < 2 or "--out" not in sys.argv:
        print(__doc__)
        return 2
    bp = Path(sys.argv[1])
    out = Path(sys.argv[sys.argv.index("--out") + 1])
    force = "--force" in sys.argv

    panels_json = bp / "10-stax" / "panels.json"
    if not panels_json.exists():
        print(f"carte des panneaux absente : {panels_json}", file=sys.stderr)
        print("Lancer d'abord : stax_derive.py <blueprint> --write", file=sys.stderr)
        return 2

    d = json.loads(panels_json.read_text(encoding="utf-8"))
    (out / "panels").mkdir(parents=True, exist_ok=True)

    written, skipped = [], []

    def write(path: Path, content: str):
        # Ne jamais écraser un fichier que quelqu'un a édité : le générateur pose
        # un départ, il ne reprend pas la main sur du travail humain.
        if path.exists() and not force:
            skipped.append(path.name)
            return
        path.write_text(content, encoding="utf-8")
        written.append(path.name)

    write(out / "types.ts", emit_types(d))
    write(out / "registry.ts", emit_registry(d))
    for p in d["panels"]:
        write(out / "panels" / f"{p['panelType']}.tsx", emit_panel(p, d))

    print(f"écrits  : {len(written)} fichiers dans {out}")
    if skipped:
        print(f"gardés  : {len(skipped)} déjà présents (relancer avec --force pour écraser)")
        print(f"          {', '.join(skipped[:6])}{'…' if len(skipped) > 6 else ''}")
    print(f"\n{len(d['panels'])} panneaux · {len(d['spaces'])} espaces · "
          f"{len(d['actionRegistry'])} actions ouvrables par l'IA")
    print("\nCe qui reste à vous : ce que chaque inspecteur met en avant, le sens")
    print("des couleurs, et les actions du pied. Le reste est structurel.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
