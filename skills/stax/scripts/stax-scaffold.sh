#!/usr/bin/env bash
# stax-scaffold.sh — vendor the Stax engine + a generic shell + tokens into a target
# project, so any app can be ported to the panel grammar (Phase 2 of the conversion
# playbook). Idempotent: copies/refreshes the framework files, and writes the shell /
# registry / entry / css stubs ONLY if they do not already exist (never clobbers your
# domain wiring). Zero deps beyond React. Works for Vite or Next (client) apps.
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
REPO="$OMEGA_DIR/repos/stax"
CORE="$REPO/frameword/packages/panels-core/src/index.ts"
REACT="$REPO/frameword/packages/panels-react/src/index.tsx"
TOKENS="$REPO/frameword/apps/crm-specimen/src/tokens.css"

c_info(){ printf '\033[36m[stax]\033[0m %s\n' "$*"; }
c_ok(){   printf '\033[32m[stax]\033[0m %s\n' "$*"; }
c_warn(){ printf '\033[33m[stax]\033[0m %s\n' "$*"; }
c_die(){  printf '\033[31m[stax]\033[0m %s\n' "$*"; exit 1; }

TARGET="${1:-}"
[[ -n "$TARGET" ]] || c_die "usage: stax-scaffold.sh <target-project-dir>"
[[ -d "$TARGET" ]] || c_die "target dir not found: $TARGET"

# Ensure the framework checkout exists (clone if the installer never ran).
if [[ ! -f "$CORE" || ! -f "$REACT" ]]; then
  c_warn "framework not found at $REPO — cloning agentik-os/stax main"
  mkdir -p "$OMEGA_DIR/repos"
  git clone https://github.com/agentik-os/stax "$REPO" >/dev/null 2>&1 \
    || c_die "clone failed — check network / gh auth"
fi
[[ -f "$CORE" && -f "$REACT" ]] || c_die "engine source still missing under $REPO"

# Choose src/stax or stax at the project root.
if [[ -d "$TARGET/src" ]]; then DST="$TARGET/src/stax"; else DST="$TARGET/stax"; fi
mkdir -p "$DST"
c_info "scaffolding into $DST"

# 1) Vendor the engine (import rewritten to the local file — no npm package needed)
#    + the WhitePaper design system (tokens.css + stax-ui.css) so the app WEARS the
#    Stax look, not just its mechanic.
cp -f "$CORE" "$DST/panels-core.ts"
sed 's#@frameword/panels-core#./panels-core#g' "$REACT" > "$DST/panels-react.tsx"
[[ -f "$TOKENS" ]] && cp -f "$TOKENS" "$DST/tokens.css"
SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -f "$SKILL_DIR/assets/stax-ui.css" ]]; then cp -f "$SKILL_DIR/assets/stax-ui.css" "$DST/stax-ui.css"; fi
# keep a fresh tokens snapshot from the skill assets if the framework copy was missing
[[ ! -f "$DST/tokens.css" && -f "$SKILL_DIR/assets/tokens.css" ]] && cp -f "$SKILL_DIR/assets/tokens.css" "$DST/tokens.css"
c_ok "engine + design system vendored: panels-core.ts, panels-react.tsx, tokens.css, stax-ui.css"

# Le gate de design, ecrit DANS le projet. Une app Stax n'est pas "bonne" parce
# qu'elle ressemble a la reference : elle l'est quand `stax verify` passe dans les
# deux themes. Sans ce script sous les yeux, personne ne pense a le lancer, et on
# corrige au jugement pendant des heures ce qu'une commande tranche en trente
# secondes.
mkdir -p "$TARGET/scripts"
cat > "$TARGET/scripts/stax-gate.sh" <<'GATE'
#!/usr/bin/env bash
# Le gate de design Stax. A lancer AVANT de dire que l'interface est bonne.
#
#   ./scripts/stax-gate.sh [url]
#
# Il pilote l'app qui tourne et asserte les lois du design system : marges
# interieures, rythme vertical, pied a 44px, champs tokenises, luminance du
# canvas en sombre. Il sort en 1 a la moindre violation.
set -uo pipefail
URL="${1:-http://localhost:3000}"
STAX="${STAX_MIGRATE:-$HOME/.omega/repos/stax/frameword/packages/stax-migrate/index.mjs}"

[[ -f "$STAX" ]] || { echo "stax-migrate introuvable: $STAX" >&2; exit 2; }
[[ -d node_modules/playwright ]] || { echo "playwright manquant: npm i -D playwright" >&2; exit 2; }
curl -sf -o /dev/null "$URL" || { echo "rien ne repond sur $URL — lancer l'app d'abord" >&2; exit 2; }

export PLAYWRIGHT_BROWSERS_PATH="${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"
node "$STAX" verify --url "$URL" --themes light,dark
GATE
chmod +x "$TARGET/scripts/stax-gate.sh"
c_ok "gate de design ecrit: scripts/stax-gate.sh (le lancer avant de juger a l'oeil)"

# 2) Generic shell (only if absent).
write_if_absent(){ # $1 path ; stdin = content
  if [[ -e "$1" ]]; then c_warn "kept existing $(basename "$1")"; cat >/dev/null; else cat > "$1"; c_ok "wrote $(basename "$1")"; fi
}

write_if_absent "$DST/shell.tsx" <<'EOF'
"use client";
import { useEffect, type ReactNode } from "react";
import { useWorkspace, useIsCompact, panelWidth, type WorkspaceApi } from "./panels-react";
import type { PanelInstance } from "./panels-core";

export type Renderers = Record<string, (panel: PanelInstance, ws: WorkspaceApi) => ReactNode>;
export interface SpaceDef {
  spaceId: string;
  label: string;
  target: { panelType: string; resourceKey: string };
}

function label(ws: WorkspaceApi, p: PanelInstance) {
  const def = ws.registry[p.target.panelType];
  return def?.label ? def.label(p.target.resourceKey) : p.target.resourceKey;
}

export function Shell({ spaces, renderers }: { spaces: SpaceDef[]; renderers: Renderers }) {
  const ws = useWorkspace();
  const compact = useIsCompact();
  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if (e.key === "Escape" && ws.state.focusedPanelId) ws.closePanel(ws.state.focusedPanelId);
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, [ws]);
  return (
    <div className="stax-root">
      <aside className="stax-sidebar">
        {spaces.map((s) => (
          <button key={s.spaceId} className="stax-space-btn"
            data-active={ws.state.spaceId === s.spaceId}
            onClick={() => ws.openSpace(s.spaceId, s.target)}>{s.label}</button>
        ))}
      </aside>
      <div className="stax-main">
        <header className="stax-topbar"><Breadcrumb /></header>
        {compact ? <PushHost renderers={renderers} /> : <ColumnHost renderers={renderers} />}
      </div>
    </div>
  );
}

function Breadcrumb() {
  const ws = useWorkspace();
  return (
    <nav className="stax-breadcrumb" aria-label="Breadcrumb">
      {ws.path.map((id, i) => {
        const p = ws.state.panelsById[id];
        return (
          <span key={id}>
            {i > 0 && <span className="stax-sep">/</span>}
            <button className="stax-crumb" onClick={() => ws.navigateTo(id)}>{label(ws, p)}</button>
          </span>
        );
      })}
    </nav>
  );
}

function ColumnHost({ renderers }: { renderers: Renderers }) {
  const ws = useWorkspace();
  if (!ws.state.rootInstanceId) return <Empty />;
  const context = ws.path.map((id) => ws.state.panelsById[id]);
  const refs = ws.state.referenceRailOrder.map((id) => ws.state.panelsById[id]);
  return (
    <div className="stax-stage stax-rail">
      {context.map((p) => <Panel key={p.instanceId} panel={p} renderers={renderers} />)}
      {refs.map((p) => <Panel key={p.instanceId} panel={p} renderers={renderers} isReference />)}
    </div>
  );
}

function PushHost({ renderers }: { renderers: Renderers }) {
  const ws = useWorkspace();
  const leafId = ws.state.contextLeafId;
  if (!ws.state.rootInstanceId || !leafId) return <Empty />;
  return (
    <div className="stax-stage stax-rail stax-push">
      <Panel panel={ws.state.panelsById[leafId]} renderers={renderers} />
    </div>
  );
}

function Panel({ panel, renderers, isReference }: { panel: PanelInstance; renderers: Renderers; isReference?: boolean }) {
  const ws = useWorkspace();
  const width = panelWidth(ws.registry, panel);
  const isRoot = panel.role === "root";
  const body = renderers[panel.target.panelType];
  const pinned = panel.retention === "retained";
  return (
    <section className="stax-panel" data-role={panel.role} data-reference={isReference || undefined}
      data-focused={ws.state.focusedPanelId === panel.instanceId}
      style={{ width }} onMouseDown={() => ws.focusPanel(panel.instanceId)}>
      <header className="stax-panel-head">
        <span className="stax-panel-title">{label(ws, panel)}</span>
        <span className="stax-panel-ctrls">
          {!isRoot && !isReference && (
            <button className="stax-ico" title={pinned ? "Unpin" : "Pin"}
              onClick={() => (pinned ? ws.unpinPanel(panel.instanceId) : ws.pinPanel(panel.instanceId))}>
              {pinned ? "★" : "☆"}
            </button>
          )}
          <button className="stax-ico" title="Close"
            onClick={() => (isReference ? ws.unpinPanel(panel.instanceId) : ws.closePanel(panel.instanceId))}>✕</button>
        </span>
      </header>
      <div className="stax-panel-body">
        {body ? body(panel, ws) : <div className="stax-missing">No renderer for &quot;{panel.target.panelType}&quot;. Add one in registry.tsx.</div>}
      </div>
      {/* The footer is the ONE action zone — a renderer that needs actions returns its
          own <footer className="stax-panel-foot">…</footer> as the last body element. */}
    </section>
  );
}

function Empty() {
  return <div className="stax-empty">Pick a space to begin.</div>;
}
EOF

write_if_absent "$DST/registry.tsx" <<'EOF'
import type { PanelRegistry } from "./panels-react";
import type { SpaceDef, Renderers } from "./shell";

// width belongs to the KIND, not the user (S 380 / M 480 / L 640 / XL 800).
export const REGISTRY: PanelRegistry = {
  home: { size: "M", label: () => "Home" },
  // contact: { size: "M", label: (k) => `Contact ${k}` },
  // deal:    { size: "L", label: (k) => `Deal ${k}` },
};

// The sidebar sections — each opens a Space (replaces the active thread).
export const SPACES: SpaceDef[] = [
  { spaceId: "home", label: "Home", target: { panelType: "home", resourceKey: "home" } },
];

// One renderer per panelType. Fetch data from panel.target.resourceKey / params.
// Drill to the right with ws.openDetail(panel.instanceId, { panelType, resourceKey }).
export const renderers: Renderers = {
  home: (_panel, ws) => (
    <div style={{ padding: 16 }}>
      <p className="body">Empty Stax workspace. Register panel types and renderers here,</p>
      <p className="body-sm">then drill with <code>ws.openDetail(panel.instanceId, target)</code>.</p>
    </div>
  ),
};
EOF

write_if_absent "$DST/Stax.tsx" <<'EOF'
"use client";
import { WorkspaceProvider } from "./panels-react";
import { Shell } from "./shell";
import { REGISTRY, SPACES, renderers } from "./registry";
import "./stax.css";

// Mount <Stax /> as your single app surface (one route in Next; the app root in Vite).
export function Stax() {
  return (
    <WorkspaceProvider registry={REGISTRY} urlSync storageKey="stax-workspace">
      <Shell spaces={SPACES} renderers={renderers} />
    </WorkspaceProvider>
  );
}
EOF

write_if_absent "$DST/stax.css" <<'EOF'
/* The Stax WhitePaper design system. tokens.css = the palette + fonts (swap the
   accent and the whole system follows); stax-ui.css = the shell/panel chrome
   (dot-grid stage, card panels, mono eyebrows, serif titles, accent footers).
   Add your app-specific panel-body styles below the imports. */
@import "./tokens.css";
@import "./stax-ui.css";
EOF

cat <<NEXT

$(c_ok "scaffold complete → $DST")
Next steps (Phase 3 — PORT):
  1. Mount <Stax /> as your single app surface:
       Vite  → import { Stax } from "./stax/Stax"; render <Stax /> in main.tsx
       Next  → a client component at app/page.tsx that returns <Stax />
  2. Fill $DST/registry.tsx — one panelType + renderer per screen, and the SPACES sidebar.
  3. Replace navigations: router.push / <Link> → ws.openDetail(parentId, target) (drill)
     or ws.openSpace(spaceId, target) (section). Delete view/edit modals; keep dialogs.
  4. Verify: ws.violations must stay [] ; a shared URL restores the ContextPath.
See ~/.omega/skills/stax/references/conversion-playbook.md for the full pipeline.
NEXT
