---
name: stax
description: >
  Convert ANY project fully to Stax — the panels-inside-panels dashboard grammar (Miller
  columns, evolved): one mechanic (open-right), one action zone (the footer), one way back
  (close / breadcrumb). No pages, no modals, no tabs — the whole UI derives from a single
  serializable panel-workspace state (URL-synced, pinnable references, per-type sizing). Use
  when the user says "/stax", "/omg-stax", "convert to stax", "panelize this app", "make this
  a panel stack", "stax layout", "miller columns", "panels inside panels", "dashboard vision
  structure", or in French "convertis en stax", "passe en panneaux", "structure de dashboard".
  This is the OmegaOS dashboard vision structure — tracks github.com/agentik-os/stax main.
triggers: ["stax","omg-stax","convert to stax","panelize","panelize this app","make this a panel stack","stax layout","miller columns","panels inside panels","dashboard vision structure","convertis en stax","passe en panneaux","structure de dashboard"]
allowed-tools: ["Bash","Read","Write","Edit","Grep","Glob"]
domain: frontend
read_only: false
argument-hint: "<path to the project to convert>  (defaults to the cwd project)"
source: github.com/agentik-os/stax (agentik-os SSOT — tracks main, never a frozen pin)
license: MIT (Agentik OS)
---

> **OmegaOS skill — the dashboard vision structure.** Converts any app to the Stax
> panel grammar. Triggers `/stax` (and `/omg-stax`). The canonical framework lives at
> `~/.omega/repos/stax` (a live checkout of `github.com/agentik-os/stax` main, kept
> current by the daily `OMEGA-CRON-STAX-SYNC-v1` cron). **Always read the live source
> under `~/.omega/repos/stax` — it is the source of truth, this skill is the pilot.**


## L'OUTILLAGE — lance-le, ne juge jamais à l'œil

Stax livre un CLI complet, `stax-migrate`, et **il faut le lancer**. Une session
entière a été perdue à corriger le design au jugement alors que deux commandes
donnaient la réponse en trente secondes.

```bash
STAX=~/.omega/repos/stax/frameword/packages/stax-migrate/index.mjs
```

### Les deux gates — à lancer AVANT de dire que c'est bon

```bash
# LE gate de design : pilote l'app qui tourne et asserte les lois
# (marges intérieures, rythme, pied 44px, champs tokenisés, luminance en sombre)
PLAYWRIGHT_BROWSERS_PATH=$HOME/.cache/ms-playwright \
  node $STAX verify --url http://localhost:3000 --themes light,dark

# L'audit complet : cette app obéit-elle à ses propres lois, sur toutes ses
# surfaces, dans les deux thèmes
node $STAX audit stax --url http://localhost:3000
```

`verify` sort en 1 sur la moindre violation. Il exige `playwright` **dans le
projet** (`npm i -D playwright`) et le navigateur correspondant.

### Les deux commandes qui évitent de dessiner à la main

```bash
node $STAX shapes    # LE ROUTEUR DE FORMES : la mise en page vient de ce que
                     # la donnée EST, jamais de l'habitude
node $STAX patterns  # les écrans que le framework PROUVE déjà, avec leur
                     # grammaire Stax et un panneau de référence live à copier
```

**`shapes` est le plus sous-estimé.** Il répond à « quelle forme pour cette
donnée » avant qu'on ouvre un éditeur :

| La donnée est | La forme | Le piège |
|---|---|---|
| Des lignes que l'utilisateur ÉDITE | `grid` (datatable) | N'y aller qu'en dernier : si la question est « que s'est-il passé », c'est un stream |
| Des événements dans le temps | `stream` | Pas un grid : on lit de haut en bas, on ne trie pas par colonne |
| Des entités comparées par MAGNITUDE | `ladder` | Ne pas ajouter de camembert : la barre EST la comparaison |

**`patterns` couvre les écrans de console déjà résolus** : clés d'API avec
révélation unique, équipe et sièges avec invitations, projets, facturation en
hub (sous-pages en DRILLS, jamais en onglets), usage avec segments au pied. Les
recopier plutôt que les réinventer.

### Le reste

```bash
node $STAX upgrade [dir]        # projet DÉJÀ Stax : les mises à jour layout+design
                                # appliquées et en attente
node $STAX data scan [dir]      # extraction backend programmatique (Convex, Supabase,
                                # Prisma, REST, tRPC) avec les sites d'appel, file:line
node $STAX data check [dir]     # le gate mécanique : chaque ligne non interne est liée
```

> **La règle qui découle de tout ça :** un panneau n'est pas « bon » parce qu'il
> ressemble à la référence. Il est bon quand `verify` passe dans les deux thèmes.
> Le gate voit ce que l'œil ne voit pas, et il ne se fatigue pas.

## What Stax is (the one mechanic)

The entire interface is a **horizontal rail of in-page panels**. Click anything with
depth and a panel opens to its right; the parent stays visible. That panel can open
another, to unbounded logical depth. Navigation is **not routing** — the whole UI is
*derived* from one serializable state object. There are **NO pages, NO modals, NO tabs**:

- **One mechanic** — open-right (`openDetail`); sections switch the thread (`openSpace`).
- **One action zone** — the panel **footer**. Never floating buttons.
- **One way back** — close, `Esc`, or a breadcrumb crumb (derived from the state).

Everything else — breadcrumb, URL, persistence, agent context, the compact phone
back-stack — *derives* from the state. Change the accent token and the whole system
follows. Read `references/canonical-context.md` for the mental model and the 5 laws.

## When NOT to use Stax (fit gate — do this first)

Stax fits **relational, data-dense operational software** where conventional route
changes destroy context (CRMs, admin consoles, dashboards, ops tools, knowledge apps).
It is **not** a universal replacement for pages/tabs/dialogs. Reject panelization where:

- data is shallow, sequential, or unrelated (a linear checkout, a marketing site);
- a blocking yes/no decision belongs in a **dialog**, not a panel;
- alternate representations of ONE record belong in **tabs inside one panel**, not siblings;
- a cross-context tool belongs in the **UtilityDrawer** (an overlay — never on the stack).

Run **Phase 0** and end with `GO` / `ADAPT` / `DO NOT USE` before touching code (L2).

## PRIMARY PATH — `stax-migrate` (the official engine)

The framework now ships **`stax-migrate`** — a zero-dep Node CLI that drives a
provably-lossless refonte with two file-backed matrices (feature + pixel-level element)
and a hard `done` gate that refuses to advance while any row is unmigrated. **Prefer it
over the manual pipeline below** for any real conversion — it replaces "restyled by eye"
with the canonical `design-spec.md` contract and greps the new app for drift. Full guide:
`references/stax-migrate.md`; the pixel contract: `references/design-spec.md`.

```sh
M=~/.omega/repos/stax/frameword/packages/stax-migrate/index.mjs
node "$M" init /path/to/legacy-app     # 9 phase briefs + design-spec + matrices
cd /path/to/legacy-app && node "$M" next   # then: run . --agent claude → done → repeat
```

The manual pipeline below remains valid for a quick prototype, a teaching walk-through, or
where a Node CLI can't run — but it carries no coverage guarantee.

## The manual conversion pipeline (fallback)

Full detail + the paste-ready prompts are in `references/conversion-playbook.md`. The
exact API is in `references/api-reference.md`. Summary:

| Phase | Goal | Output |
|---|---|---|
| **0 · FIT** | Is this app relational + context-losing? Spaces, entities, golden ContextPaths, what must NOT panelize. | `GO`/`ADAPT`/`NO` + Space & entity map |
| **1 · BLUEPRINT** | Every panel type: `panelType`, `resourceKey`, `size`, parent/drill, title/breadcrumb source, footer action, preview-vs-pin, URL, all states. | the `PanelRegistry` map + drill graph |
| **2 · SCAFFOLD** | Vendor the engine + shell + tokens into the target (mechanical). | working empty Stax workspace |
| **3 · PORT** | Convert every page/route/modal/tab into panel types + intent calls. | the app, on the panel grammar |
| **4 · THEME** | Adopt the WhitePaper design system (tokens + stax-ui chrome + serif/mono type); swap only `--accent` for the brand; per-panel container queries. | Stax-looking, token-driven UI |
| **5 · VERIFY** | Laws test-kit + `validate()`==[] + runtime (URL restore, back/forward, focus, compact, states). | a conformance pass (evidence, not vibes) |

### Phase 2 — scaffold (mechanical, do this to unblock the port)

```sh
bash ~/.omega/skills/stax/scripts/stax-scaffold.sh <target-project-dir>
```

It vendors `panels-core` + `panels-react` (rewriting the `@frameword/*` import to a
local path — zero deps beyond React), drops in a `stax/` shell (Sidebar / breadcrumb
Topbar / `ColumnHost` + `PushHost` Stage / `Panel` header-body-footer), **the WhitePaper
design system** (`tokens.css` = palette + Inter/Newsreader/Geist Mono fonts, `stax-ui.css`
= the shell/panel chrome: dot-grid stage, 14px card panels, mono eyebrows, serif titles,
accent footers), and a `registry.tsx` stub. React 18/19 + TS. After it runs, the app both
behaves AND looks like Stax; Phase 3 fills the registry with the real domain.

**The design IS part of Stax, not just the mechanic.** A conversion adopts the WhitePaper
look by default (see `references/design-system.md`); you only swap the `--accent` token for
the brand — the whole system follows (Law 5). Do NOT ship a bare/generic shell.

### Phase 3 — the porting map (the heart of "convert any project")

| You currently have | Becomes in Stax |
|---|---|
| A sidebar/nav **section** | an `openSpace(spaceId, target)` trigger (the only thing that changes lineage) |
| A **page / route** for a record | a registered `panelType` renderer; its content moves into `<PanelBody>` |
| `router.push('/thing/:id')` on a link | `openDetail(parentInstanceId, { panelType, resourceKey, params })` — opens right |
| **Master → detail** route pair | master is a Space root; detail is `openDetail` from it (parent stays) |
| A **modal to view/edit** a record | a **DetailPanel** (opens right, keeps context). Delete the modal. |
| A **modal for a blocking yes/no** | stays a **dialog** — Stax does not swallow real modals |
| **Tabs** = alternate views of one record | tabs **inside** that panel's body |
| **Tabs** = different records | **separate** panels (drill/pin) |
| A persistent **Save / Create** button | the **PanelFooter** — the one action zone |
| A **row action / inline edit** | stays near the object (not the footer) |
| A **breadcrumb** | derived from `getContextPath(state)`; each crumb = `navigateTo(id)` |
| A **"compare two records"** need | **pin** the first (`pinPanel`) then open the second |
| **Deep link / shareable URL** | the `ContextPath` auto-encodes to `location.hash` (URL sync is built in) |
| **Phone** layout | free — `useIsCompact()` swaps `ColumnHost`→`PushHost` (iOS back-stack), same state |

Every `params` value must be **JSON-only** — never store JSX, callbacks, or fetched
rows in navigation state (a hard engine invariant; `validate()` will catch it).

## Verification gate (L1 / L4 — do not skip)

A conversion is "done" only when, at runtime:
1. `validate(workspaceState)` returns `[]` (run the panels-core laws test-kit too);
2. opening the same target from the same parent **reveals**, never duplicates;
3. the breadcrumb, URL hash, and on-screen rail always agree;
4. a shared URL restores the exact ContextPath (refresh + paste-in-new-tab);
5. browser Back/Forward reconcile (distinct from `navigateUp`);
6. focus transfers into the new panel on open, restores on close; `Esc` closes;
7. compact viewport collapses to one focused column (`PushHost`);
8. loading / empty / error / not-found / permission / deleted / dirty-edit states render;
9. no floating action buttons — every commit action lives in a footer.

Cite evidence for each (R-CITE). Documented-but-absent behavior is a FAIL, not a pass.

## Scaffolding a brand-new Stax project (not a conversion)

```sh
# the specimen is the reference implementation to copy/learn from:
cp -r ~/.omega/repos/stax/frameword <your-new-project>/     # or start from crm-specimen
```
For a fresh OmegaOS-stack build (Next.js App Router + Tailwind), keep exactly one route
(`/`) and make everything else panel state — see the Next.js adapter prompt (Prompt 6)
in the playbook. Data/auth/payments (Convex + Clerk + Stripe) are deferred until the
mechanic is locked.

## Keeping current

The framework auto-syncs daily from `github.com/agentik-os/stax` main
(`OMEGA-CRON-STAX-SYNC-v1` → `~/.omega/logs/stax-sync.cron.log`, Telegram-pings on
change). To pull on demand: `bash ~/.omega/skills/stax/scripts/stax-sync.sh`. Because the
checkout tracks **latest main** (not a frozen pin), always re-read the live source before
a conversion — the API in `references/` is a snapshot for orientation, the repo wins.
