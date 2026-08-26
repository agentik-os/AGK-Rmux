#!/usr/bin/env python3
"""runner.py — le pilote que l'agent suit, étape par étape.

Il existe pour supprimer trois gestes que l'agent refait à chaque tour et rate
régulièrement : relire tout le blueprint pour décider quoi faire, choisir un ordre
qui respecte les dépendances, et se déclarer fini sur une impression.

Ici, l'ordre est calculé, la prochaine étape est donnée, et « fini » est une commande
qui s'exécute et qui peut échouer. Une étape ne passe jamais à `done` sans que sa
définition du fini soit VERTE : c'est tout l'intérêt du bloc 3.

    runner.py <blueprint> status              où on en est
    runner.py <blueprint> next [--version v1] la prochaine étape prête, en entier
    runner.py <blueprint> show <id>           une étape en entier
    runner.py <blueprint> verify <id>         exécute sa définition du fini
    runner.py <blueprint> done <id>           la ferme, APRÈS vérification
    runner.py <blueprint> block <id> "raison" la bloque et dit pourquoi
    runner.py <blueprint> reset <id>          la remet à todo
    runner.py <blueprint> diagram             le DAG Mermaid de l'avancement

L'état vit dans plan/state.json, séparé de plan.json qui est régénérable : refaire
le plan ne doit jamais effacer ce qui a été fait.

`verify` s'exécute dans le dossier de l'APP (build.chemin_app du blueprint), pas dans
le blueprint : les commandes parlent de convex/ et src/, qui n'existent que là-bas.
"""
from __future__ import annotations
import json, os, subprocess, sys
from pathlib import Path

C = {"g": "\033[32m", "r": "\033[31m", "y": "\033[33m", "b": "\033[36m",
     "d": "\033[2m", "B": "\033[1m", "x": "\033[0m"}
MARK = {"done": f"{C['g']}✓{C['x']}", "todo": "☐", "doing": f"{C['b']}▸{C['x']}",
        "blocked": f"{C['r']}✗{C['x']}", "incomplete": f"{C['r']}!{C['x']}"}


class Runner:
    def __init__(self, bp: Path):
        self.bp = bp
        self.plan_path = bp / "plan" / "plan.json"
        self.state_path = bp / "plan" / "state.json"
        if not self.plan_path.exists():
            sys.exit(f"plan absent : {self.plan_path}\nLancer d'abord : plan_build.py {bp} --write")
        self.plan = json.loads(self.plan_path.read_text(encoding="utf-8"))
        self.state = (json.loads(self.state_path.read_text(encoding="utf-8"))
                      if self.state_path.exists() else {})
        manifest = json.loads((bp / "blueprint.json").read_text(encoding="utf-8")) \
            if (bp / "blueprint.json").exists() else {}
        self.app = manifest.get("build", {}).get("chemin_app")

    # ── état ────────────────────────────────────────────────────────────────
    def save(self):
        self.state_path.parent.mkdir(exist_ok=True)
        self.state_path.write_text(json.dumps(self.state, indent=2, ensure_ascii=False),
                                   encoding="utf-8")

    def status_of(self, s: dict) -> str:
        if s["status"] == "incomplete":
            return "incomplete"
        return self.state.get(s["id"], {}).get("status", "todo")

    def get(self, sid: str) -> dict | None:
        return next((s for s in self.plan["steps"] if s["id"] == sid), None)

    def ready(self, s: dict) -> bool:
        """Prête = pas finie, pas bloquée, complète, et toutes ses dépendances faites."""
        if self.status_of(s) in ("done", "blocked", "incomplete"):
            return False
        return all(self.status_of(self.get(d)) == "done"
                   for d in s["dependsOn"] if self.get(d))

    def blockers(self, s: dict) -> list[str]:
        return [d for d in s["dependsOn"]
                if self.get(d) and self.status_of(self.get(d)) != "done"]

    # ── commandes ───────────────────────────────────────────────────────────
    def cmd_status(self, version: str | None = None):
        steps = [s for s in self.plan["steps"] if not version or s["version"] == version]
        counts: dict[str, int] = {}
        for s in steps:
            st = self.status_of(s)
            counts[st] = counts.get(st, 0) + 1
        total = len(steps)
        done = counts.get("done", 0)
        pct = int(done * 100 / total) if total else 0

        print(f"{C['B']}═══ {self.plan['blueprint']} ═══{C['x']}")
        print(f"primitive : {self.plan['primitive']}")
        if self.app:
            print(f"app       : {self.app}")
        bar = "█" * (pct // 4) + "░" * (25 - pct // 4)
        print(f"\n{bar}  {done}/{total}  ({pct}%)\n")

        for v in ("v0", "v1", "v2", "v3", "v4", "v5"):
            vs = [s for s in self.plan["steps"] if s["version"] == v]
            if not vs:
                continue
            d = sum(1 for s in vs if self.status_of(s) == "done")
            bl = sum(1 for s in vs if self.status_of(s) == "blocked")
            inc = sum(1 for s in vs if self.status_of(s) == "incomplete")
            line = f"  {v}  {d:>3}/{len(vs):<3}"
            if bl:
                line += f"  {C['r']}{bl} bloquée(s){C['x']}"
            if inc:
                line += f"  {C['r']}{inc} rouge(s){C['x']}"
            print(line)

        rdy = [s for s in steps if self.ready(s)]
        print(f"\n{len(rdy)} étape(s) prête(s) maintenant.")
        blocked = [s for s in steps if self.status_of(s) == "blocked"]
        if blocked:
            print(f"\n{C['r']}Bloquées :{C['x']}")
            for s in blocked:
                why = self.state.get(s['id'], {}).get("reason", "")
                print(f"  ✗ {s['id']} — {why}")
        inc = [s for s in steps if self.status_of(s) == "incomplete"]
        if inc:
            print(f"\n{C['r']}Rouges (les 4 blocs ne sont pas remplis, donc bloquantes) :{C['x']}")
            for s in inc[:8]:
                print(f"  ! {s['id']} — {s.get('notes') or 'bloc manquant'}")

    def cmd_next(self, version: str | None = None):
        cand = [s for s in self.plan["steps"]
                if self.ready(s) and (not version or s["version"] == version)]
        if not cand:
            nxt = [s for s in self.plan["steps"] if self.status_of(s) == "todo"]
            if not nxt:
                print(f"{C['g']}Tout est fait.{C['x']}")
                return
            print(f"{C['y']}Aucune étape prête.{C['x']} Les suivantes attendent :")
            for s in nxt[:5]:
                print(f"  {s['id']} ← {', '.join(self.blockers(s))}")
            return
        order = {v: i for i, v in enumerate(["v0", "v1", "v2", "v3", "v4", "v5"])}
        cand.sort(key=lambda s: (order.get(s["version"], 9), len(s["dependsOn"])))
        self.show(cand[0])
        print(f"{C['d']}({len(cand)} étapes prêtes au total){C['x']}")

    def show(self, s: dict):
        print(f"\n{C['B']}▸ {s['id']}{C['x']}")
        print(f"{C['B']}{s['title']}{C['x']}")
        print(f"{C['d']}{s['type']} · {s['version']} · voie {s['lane']} · "
              f"source : {s['source']}{C['x']}\n")
        print(f"{C['b']}1. OBJECTIF{C['x']}\n   {s['objective']}\n")
        print(f"{C['b']}2. CONTRAINTES{C['x']}")
        for c in s["constraints"]:
            print(f"   - {c}")
        dod = s["definitionOfDone"]
        tag = f"{C['g']}vérifiable par machine{C['x']}" if dod["machine"] else \
              f"{C['y']}revue humaine{C['x']}"
        print(f"\n{C['b']}3. DÉFINITION DU FINI{C['x']}  ({tag})")
        print(f"   $ {dod['check']}")
        if dod.get("note"):
            print(f"   {C['d']}{dod['note']}{C['x']}")
        print(f"\n{C['b']}4. NE PAS TOUCHER{C['x']}")
        for d in s["doNotTouch"]:
            print(f"   - {d}")
        if s["files"]:
            print(f"\n{C['d']}fichiers : {', '.join(s['files'])}{C['x']}")
        if s["dependsOn"]:
            print(f"{C['d']}dépend de : {', '.join(s['dependsOn'])}{C['x']}")
        if s.get("notes"):
            print(f"\n{C['y']}! {s['notes']}{C['x']}")
        print()

    def cmd_show(self, sid: str):
        s = self.get(sid)
        if not s:
            sys.exit(f"étape inconnue : {sid}")
        self.show(s)
        print(f"statut : {self.status_of(s)}")

    def cmd_verify(self, sid: str) -> bool:
        s = self.get(sid)
        if not s:
            sys.exit(f"étape inconnue : {sid}")
        dod = s["definitionOfDone"]
        if not dod["machine"]:
            print(f"{C['y']}Cette étape n'est pas vérifiable par machine.{C['x']}")
            print(f"À faire en revue : {dod['check']}")
            return False
        cwd = self.app if self.app and Path(self.app).is_dir() else str(self.bp)
        print(f"{C['d']}$ cd {cwd}{C['x']}")
        print(f"{C['d']}$ {dod['check']}{C['x']}\n")
        env = dict(os.environ, OMEGA_DIR=os.environ.get("OMEGA_DIR",
                                                        str(Path.home() / ".omega")))
        r = subprocess.run(dod["check"], shell=True, cwd=cwd, env=env,
                           capture_output=True, text=True)
        out = (r.stdout + r.stderr).strip()
        if out:
            print("\n".join(out.splitlines()[:20]))
        ok = r.returncode == 0
        print(f"\n{C['g']}VERT{C['x']}" if ok else f"\n{C['r']}ROUGE (code {r.returncode}){C['x']}")
        return ok

    def cmd_done(self, sid: str, force: bool = False):
        s = self.get(sid)
        if not s:
            sys.exit(f"étape inconnue : {sid}")
        if s["definitionOfDone"]["machine"] and not force:
            if not self.cmd_verify(sid):
                print(f"\n{C['r']}Refus de fermer : la définition du fini est rouge.{C['x']}")
                print("Corriger, ou fermer en connaissance de cause avec --force.")
                sys.exit(1)
        self.state[sid] = {"status": "done"}
        self.save()
        print(f"{C['g']}✓ {sid} fermée.{C['x']}")
        nxt = [x for x in self.plan["steps"] if self.ready(x)]
        if nxt:
            print(f"{len(nxt)} étape(s) prête(s). Suivante : {nxt[0]['id']}")

    def cmd_block(self, sid: str, reason: str):
        if not self.get(sid):
            sys.exit(f"étape inconnue : {sid}")
        self.state[sid] = {"status": "blocked", "reason": reason}
        self.save()
        print(f"{C['r']}✗ {sid} bloquée : {reason}{C['x']}")

    def cmd_reset(self, sid: str):
        self.state.pop(sid, None)
        self.save()
        print(f"{sid} remise à todo.")

    def cmd_diagram(self):
        print("graph TD")
        for s in self.plan["steps"]:
            st = self.status_of(s)
            shape = {"done": '["{}"]', "blocked": '{{"{}"}}',
                     "incomplete": '{{"{}"}}'}.get(st, '("{}")')
            sid = s["id"].replace("-", "_")
            print(f'  {sid}{shape.format(s["title"][:34])}')
        for s in self.plan["steps"]:
            for d in s["dependsOn"]:
                if self.get(d):
                    print(f'  {d.replace("-", "_")} --> {s["id"].replace("-", "_")}')
        print("  classDef done fill:#1f6b4a,color:#fff;")
        print("  classDef blocked fill:#b3261e,color:#fff;")
        for st, cls in (("done", "done"), ("blocked", "blocked"), ("incomplete", "blocked")):
            ids = [s["id"].replace("-", "_") for s in self.plan["steps"]
                   if self.status_of(s) == st]
            if ids:
                print(f'  class {",".join(ids)} {cls};')


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__); return 2
    bp = Path(sys.argv[1])
    if not bp.is_dir():
        sys.exit(f"blueprint introuvable : {bp}")
    cmd, args = sys.argv[2], sys.argv[3:]
    ver = None
    if "--version" in args:
        i = args.index("--version"); ver = args[i + 1]; args = args[:i] + args[i + 2:]
    r = Runner(bp)

    if cmd == "status":    r.cmd_status(ver)
    elif cmd == "next":    r.cmd_next(ver)
    elif cmd == "show":    r.cmd_show(args[0])
    elif cmd == "verify":  return 0 if r.cmd_verify(args[0]) else 1
    elif cmd == "done":    r.cmd_done(args[0], "--force" in args)
    elif cmd == "block":   r.cmd_block(args[0], " ".join(args[1:]) or "sans raison")
    elif cmd == "reset":   r.cmd_reset(args[0])
    elif cmd == "diagram": r.cmd_diagram()
    else:
        print(__doc__); return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
