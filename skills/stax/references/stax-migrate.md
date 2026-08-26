# stax-migrate — the official migration engine (PRIMARY path)

As of framework `main` (commit 8091ec2, 2026-07-20) the Stax repo ships **`stax-migrate`**:
a zero-dependency Node CLI that drives a complete, provably-lossless refonte of any legacy
app to the panel grammar — down to the pixel. **Prefer it over the manual pipeline** in
`conversion-playbook.md`: it replaces "by eye" with two file-backed matrices and a hard
gate, so no feature and no pixel is silently lost.

Location: `~/.omega/repos/stax/frameword/packages/stax-migrate/index.mjs` (kept current by
the daily sync). Read its `README.md` + `templates/design-spec.md` for the full contract.

## The guarantee (why it beats the manual pipeline)

- **`feature-matrix.csv`** — every capability + sub-capability is an `F-NNN[.N]` row
  (column sort, CSV export, a keyboard shortcut, an empty state…). *Not in the matrix =
  doesn't exist for the pipeline.*
- **`element-matrix.csv`** — every icon (named + counted), button/card/badge/input/select/
  table/chart/nav variant, the spacing histogram, every color literal, every font-size is
  an `E-NNN[.N]` row. The smallest icon is a gated row.
- **`design-spec.md`** — the pixel contract (see `design-spec.md` here, copied from the
  framework): panel anatomy (body 18/18/16, bar h56, foot 11/14, card 14/16 r12, drills
  gap 8 / drill 12/14 r10 / lead tile 34×34 r9), type/numbers laws (numbers are ALWAYS
  mono tabular, never serif), one-accent color, radius/shadow/motion scales, icon spec,
  the old-element→Stax table, the six mandatory states.
- **`stax-migrate done`** — a hard gate that reads the CSVs (not a summary) and refuses to
  advance while any F or E row is unmigrated, printing the offending ids.

## The 9 phases

1. **Recon** → forensic `inventory.md` (every route, modal, tab, wizard step, shortcut,
   gate, empty state). 2. **Feature matrix** (F rows). 3. **UI inventory** — pixel crawl
   (E rows). 4. **Feature mapping** (grammar rules → mapping+size per F). 5. **Design
   mapping** (every E → `stax_target` + tokens + spacing per the spec). 6. **Scaffold**
   the panel shell *beside* the untouched old app. 7. **Migrate batches** — the loop: ≤5 F
   rows + every E row they touch → real panels at contract spacing, mark `migrated` +
   evidence, commit, repeat. 8. **Coverage gate** — adversarial re-crawl of the OLD app +
   design audit of the NEW app (greps raw hex, px font-sizes, native controls, margin
   drift); findings become new rows → back to 7. 9. **Acceptance** — golden-path sweep,
   laws audit, six states verified on 10 random elements, redirect every old URL, purge
   dead views → `REPORT.md`.

The old app keeps working throughout (shell mounts beside legacy; only phase 9
redirects/purges). One phase per invocation — the human stays in the loop.

## Commands

```sh
M=~/.omega/repos/stax/frameword/packages/stax-migrate/index.mjs
node "$M" init   /path/to/legacy-app     # create stax-migration/ (9 briefs, design-spec, matrices, state)
cd /path/to/legacy-app
node "$M" status                          # phase + BOTH coverage bars + first unmigrated ids
node "$M" next                            # current phase brief + how to run it
node "$M" run . --agent claude [--phase n]  # drive ONE phase via `claude -p … --permission-mode acceptEdits`
node "$M" done                            # gate check → advance, or refuse with the F-/E- ids
node "$M" prompt <n> .                     # print phase n's brief (paste into any agent)
```

Stack is sniffed from the target's `package.json` (Next.js / Angular / Vue / Svelte /
React / Express / Rails-ish). For a monorepo, `init` the app package, not the repo root.

## When to still use the manual pipeline

Only for a quick, non-guaranteed prototype, a teaching walk-through, or when a Node CLI
can't run. For any real conversion the operator cares about, use `stax-migrate` — the
coverage + design gates are the whole point of "lose nothing, provably".

## Update — v0.3.1 (framework main dac8d6e, 2026-07-21)

The engine gained the **integration contract** (adoption *levels*, and the data layer as
law) and **anti-gaming gates**: `done` now requires real `evidence` on every row marked
`migrated` (a bare `status=migrated` no longer passes), keeps **deletion memory** (a row
removed once can't quietly reappear), enforces **starter scope**, and writes a **phase-8
artifact**. The 9 phases and commands above are unchanged — always read the live
`README.md` + `index.mjs` under `~/.omega/repos/stax/frameword/packages/stax-migrate/`, and
run `node "$M" status` to see the current gate state. License is now **MIT**.

## Update — framework main ede3952 (2026-07-23)

- **Design:** panel bar height **56 → 44**, foot padding **11/14 → 7/14**, the per-panel
  **search moved into the foot** (far-left ⌕), and the foot enforces ONE primary CTA +
  secondary red-on-hover. `design-spec.md`, `assets/styles-reference.css`, `assets/stax-ui.css`
  refreshed accordingly. tokens.css unchanged.
- **New:** `create-stax-app` (`frameword/packages/create-stax-app`) — an official
  scaffolder for a fresh Stax app; the workspace is now an **API** (registry/bridge/layouts/
  verify gate), plus a storage adapter, workspace undo, and an a11y baseline. Prefer
  `create-stax-app` for a brand-new app; keep `stax-migrate` for converting a legacy one.
