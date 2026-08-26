"""Parseur du schema.ts Convex d'un blueprint.

Un seul parseur, partagé par stax_derive, plan_build et diagrams : trois lectures
divergentes du même fichier finiraient par ne plus décrire le même produit.

Il ne cherche pas à comprendre TypeScript. Il extrait ce dont la chaîne a besoin :
les tables, leurs champs, les références entre elles, et les index. Ce qu'il ne sait
pas lire, il le dit au lieu de le deviner.
"""
from __future__ import annotations
import re
from dataclasses import dataclass, field as dc_field
from pathlib import Path

# Les noms de champ qui portent le tenant. L'ordre compte : le premier trouvé gagne.
TENANT_FIELDS = ("tenantId", "clubId", "orgId", "workspaceId", "accountId")

# Les tables de la doctrine, qui ne sont jamais des panneaux : ce sont des couches
# techniques, pas des objets que l'utilisateur ouvre.
DOCTRINE_TABLES = {"entries", "syntheses"}


@dataclass
class Field:
    name: str
    raw: str                    # le validator brut, ex. 'v.optional(v.id("people"))'
    optional: bool = False
    is_array: bool = False
    ref: str | None = None      # table référencée si v.id("x")
    kind: str = "unknown"       # string | number | boolean | id | array | union | object | any

    @property
    def literals(self) -> list[str]:
        """Les valeurs d'un v.union de v.literal, ex. les statuts."""
        return re.findall(r'v\.literal\("([^"]+)"\)', self.raw)


@dataclass
class Table:
    name: str
    fields: list[Field] = dc_field(default_factory=list)
    indexes: list[tuple[str, list[str]]] = dc_field(default_factory=list)
    search_indexes: list[str] = dc_field(default_factory=list)
    order: int = 0

    @property
    def tenant_field(self) -> str | None:
        for t in TENANT_FIELDS:
            if any(f.name == t for f in self.fields):
                return t
        return None

    @property
    def refs(self) -> list[tuple[str, str]]:
        """(nom du champ, table référencée) — ce qui devient une action open-right."""
        return [(f.name, f.ref) for f in self.fields if f.ref]

    @property
    def is_doctrine(self) -> bool:
        return self.name in DOCTRINE_TABLES

    def field(self, name: str) -> Field | None:
        return next((f for f in self.fields if f.name == name), None)


@dataclass
class Schema:
    tables: list[Table] = dc_field(default_factory=list)
    source: str = ""

    @property
    def primitive(self) -> Table | None:
        """La primitive est la PREMIÈRE table. Si on hésite, la phase 2 a échoué."""
        return self.tables[0] if self.tables else None

    def get(self, name: str) -> Table | None:
        return next((t for t in self.tables if t.name == name), None)

    @property
    def panel_tables(self) -> list[Table]:
        """Les tables qui deviennent des panneaux : tout sauf les couches de doctrine."""
        return [t for t in self.tables if not t.is_doctrine]

    def incoming(self, name: str) -> list[tuple[str, str]]:
        """(table source, champ) qui pointent vers `name`."""
        out = []
        for t in self.tables:
            for fname, ref in t.refs:
                if ref == name:
                    out.append((t.name, fname))
        return out


def _split_top_level(body: str) -> list[str]:
    """Découpe un corps d'objet en entrées de premier niveau (profondeur 0)."""
    parts, depth, cur = [], 0, []
    in_str = False
    for ch in body:
        if ch == '"':
            in_str = not in_str
        if not in_str:
            if ch in "({[":
                depth += 1
            elif ch in ")}]":
                depth -= 1
            elif ch == "," and depth == 0:
                parts.append("".join(cur)); cur = []; continue
        cur.append(ch)
    if "".join(cur).strip():
        parts.append("".join(cur))
    return parts


def _strip_comments(src: str) -> str:
    src = re.sub(r"/\*.*?\*/", "", src, flags=re.S)
    return re.sub(r"//[^\n]*", "", src)


def _match_balanced(src: str, start: int, open_ch: str = "{", close_ch: str = "}") -> tuple[int, int] | None:
    """Trouve le bloc équilibré qui commence au premier open_ch après `start`."""
    i = src.find(open_ch, start)
    if i < 0:
        return None
    depth, j, in_str = 0, i, False
    while j < len(src):
        ch = src[j]
        if ch == '"':
            in_str = not in_str
        elif not in_str:
            if ch == open_ch:
                depth += 1
            elif ch == close_ch:
                depth -= 1
                if depth == 0:
                    return (i + 1, j)
        j += 1
    return None


def _kind_of(raw: str) -> str:
    r = raw.strip()
    if ".id(" in r:      return "id"
    if ".array(" in r:   return "array"
    if ".union(" in r:   return "union"
    if ".object(" in r:  return "object"
    if ".string(" in r:  return "string"
    if ".number(" in r or ".float64(" in r or ".int64(" in r: return "number"
    if ".boolean(" in r: return "boolean"
    if ".any(" in r:     return "any"
    return "unknown"


def parse_schema(path: str | Path) -> Schema:
    src = _strip_comments(Path(path).read_text(encoding="utf-8"))
    schema = Schema(source=str(path))

    for order, m in enumerate(re.finditer(r"^\s{2}([A-Za-z_][A-Za-z0-9_]*)\s*:\s*defineTable\s*\(", src, re.M)):
        tname = m.group(1)
        span = _match_balanced(src, m.end() - 1)
        if not span:
            continue
        body = src[span[0]:span[1]]
        table = Table(name=tname, order=order)

        for entry in _split_top_level(body):
            e = entry.strip()
            fm = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(.+)$", e, re.S)
            if not fm:
                continue
            fname, raw = fm.group(1), fm.group(2).strip()
            ref = None
            idm = re.search(r'v\.id\("([^"]+)"\)', raw)
            if idm:
                ref = idm.group(1)
            table.fields.append(Field(
                name=fname, raw=raw,
                optional=raw.startswith("v.optional("),
                is_array=raw.startswith("v.array("),
                ref=ref, kind=_kind_of(raw),
            ))

        # Les index vivent APRÈS le bloc, chaînés jusqu'à la table suivante.
        tail = src[span[1]:span[1] + 1200]
        cut = re.search(r"^\s{2}[A-Za-z_][A-Za-z0-9_]*\s*:\s*defineTable", tail, re.M)
        if cut:
            tail = tail[:cut.start()]
        for im in re.finditer(r'\.index\(\s*"([^"]+)"\s*,\s*\[([^\]]*)\]', tail):
            fields = [f.strip().strip('"') for f in im.group(2).split(",") if f.strip()]
            table.indexes.append((im.group(1), fields))
        for sm in re.finditer(r'\.searchIndex\(\s*"([^"]+)"', tail):
            table.search_indexes.append(sm.group(1))

        schema.tables.append(table)

    return schema
