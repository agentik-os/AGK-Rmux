# /uiuxaudit · Forensic UI/UX audit: `agentic/reports/omegaos-report-surfaces-rollout.html`

**Verdict: 89/100 · Grade A (Professional tier)**
Worker: `OmegaOS-worker-uiuxaudit-artifact-page` · 2026-07-03 · uiuxaudit v2 (Gestalt-Popper), audit-only mode (fix phases 21-23 excluded by operator brief: READ-ONLY on the page).

---

## 0. Scope, inputs, methodology

- **Target**: one self-contained content-only HTML report page (187 lines, ~14 KB), untracked at audit time (`git status`: `?? agentic/reports/omegaos-report-surfaces-rollout.html`).
- **Standard**: the page's own claims (mission brief) + the design contract in `~/.omega/skills/artifact-design/SKILL.md` section 4.
- **v2 meta-protocol inputs**: the dispatch carried no literal `--user-need`/`--hinge` flags. Per L3 (decide and proceed) the equivalents were taken verbatim from the brief instead of refusing: user-need = "Audit against those claims and the design contract" (claims list quoted in §2); hinge = **the token cascade block, HTML lines 3-54** (the entire dual-theme claim rests on it) + the evidence tables (the page's hinge component: the VÉRIFIÉ proof table, lines 127-134). This adaptation is recorded here so the gate can audit it.
- **Runtime harness** (L1): Playwright (playwright-core 1.60.0, Chromium) on a file:// URL. The raw file is content-only by design (the Artifact publish wrapper adds doctype/head/body), so a TEMP doctype-wrapped copy was rendered at `/tmp/uiuxaudit-artifact/render.html` (original untouched; `document.compatMode = "CSS1Compat"`, standards mode, fair rendering). Both color schemes emulated (`colorScheme: 'light' | 'dark'`), desktop 1440px + mobile 375px, plus `media: print` emulation under BOTH schemes.
- **Evidence bank**: `/tmp/uiuxaudit-artifact/{desktop,mobile,focus,print}-{light,dark}.png` (8 screenshots) + `/tmp/uiuxaudit-artifact/probe-results.json` (computed-style probe, contrast math, overflow, theme-flip, focus, TOC-anchor checks).
- **Not covered** (honest bounds): the real claude.ai publish wrapper (its exact CSS reset, `data-theme` stamping, and `lang` attribute were not reproduced); Safari/Firefox rendering; actual paper print (print EMULATION only); the live artifact URL.

## 1. Claims-vs-runtime matrix (Popper: every claim tested against the rendered pixel)

| # | Claim (brief) | Runtime verdict | Evidence |
|---|---|---|---|
| C1 | Token-level dual themes; `:root[data-theme]` overrides win in BOTH directions | **PASS** | Under dark OS: forcing `data-theme="light"` flips `--bg` to `#f6f7f5`; under light OS: forcing `data-theme="dark"` flips to `#10181c` (probe-results.json `osTokenBg/forcedLightBg/forcedDarkBg`, both schemes). Cascade shape matches contract exactly: HTML lines 3-16 base, 17-30 `@media dark`, 31-54 `data-theme` blocks LAST. |
| C2 | 76ch measure | **PASS with deviation** | `.wrap` max-width 76ch (line 57) computes to 676px; effective running-text measure **71.5ch** (probe `effectiveCh`, padding included). The artifact-design contract §4 says "running text near **65ch**": ~6.5ch over target. See F-4. |
| C3 | tabular-nums on numeric cells | **PASS** | `td.num` computed `font-variant-numeric: tabular-nums` in both themes (probe `numVariant`; rule at line 76). |
| C4 | overflow-x auto table containers | **PASS** | 4 `.tablebox` elements, all computed `overflow-x: auto`; at 375px 2 of 4 actually scroll internally; body `scrollWidth == clientWidth` (375/375 and 1440/1440, both schemes): the page body NEVER scrolls sideways. Screenshots mobile-light.png / mobile-dark.png. |
| C5 | focus-visible outlines | **PASS** | Tab lands on TOC link with `outline: solid 2px rgb(23,109,140)` offset 2px (light) / `rgb(95,182,217)` (dark); focus indicator vs page bg = 5.42:1 light, 7.85:1 dark (≥3:1 WCAG). focus-light.png / focus-dark.png; rule at line 87. |
| C6 | prefers-reduced-motion | **PASS (inert)** | Guard present (line 88) and syntactically correct; the page defines zero animations/transitions, so it protects nothing today. Harmless. F-9 INFO. |
| C7 | @media print | **PARTIAL FAIL** | Correct under light scheme (print-light.png: white page, TOC hidden, #bbb borders). **Broken under a dark-scheme browser**: see F-1 HIGH. |
| C8 | Zero em/en dashes | **PASS** | `grep -P '[\x{2013}\x{2014}\x{2012}\x{2015}]'` on the raw file: zero matches (R-NODASH clean, whole file, not just visible copy). |
| C9 | French copy | **PASS** | All visible copy French (screenshots); code/identifiers English per R-STYLE. But no `lang` attribute anywhere: F-2. |

## 2. Findings (each survived the ≥2-of-3 adversarial lens gate: L1 reproduce / L2 steelman / L3 cross-check)

### F-1 · HIGH · Print from a dark-scheme browser is illegible (dark-token leak into @media print)
- **Where**: `@media print` block, HTML lines 89-93. It resets only `body` (`#fff`/`#111`) and border colors; every token-driven surface keeps its DARK values when the OS/browser scheme is dark.
- **Runtime proof** (print emulation + `colorScheme: 'dark'`): inline `code` = text `rgb(17,17,17)` on background `rgb(12,20,24)` → **contrast 1.05:1**; `td` text `rgb(17,17,17)` on table surface `rgb(22,33,38)` → ~1.2:1. Screenshot `/tmp/uiuxaudit-artifact/print-dark.png`: white page with 4 dark table slabs and black-on-black code chips, entire tables unreadable. Light-scheme print is fine (`print-light.png`).
- **Why it matters**: a report page's print path IS a claimed surface (C7); any dark-mode user hitting Ctrl+P (or print-to-PDF) gets a broken deliverable. Pass A (token census: print block overrides body only) and Pass B (claim "@media print" traced top-down) collide on the same lines → collision finding, escalated per doctrine.
- **Lens verdicts**: L1 reproduce ✅ (computed styles + screenshot) · L2 steelman ✗ (no rationale for dark tables on white paper) · L3 cross-check ✅ (contract §4 requires "a print-friendly @media print block"; light-scheme behavior shows intent). 2.5/3 → survives.
- **Fix (for the follow-up worker, not applied here)**: re-assert the LIGHT token set inside `@media print` on `:root` (9 custom properties), so every component prints light regardless of scheme.

### F-2 · MEDIUM · French content with no `lang` attribute
- **Where**: no `lang=` anywhere in the file (grep: `NO_LANG`); content is entirely French. The publish wrapper controls `<html>` (likely `lang` unset or `en`), but the content COULD carry `<div class="wrap" lang="fr">` (line 96).
- **Impact**: screen readers pick the wrong speech synthesizer for the whole document (WCAG 3.1.1/3.1.2); hyphenation and quotes handling also degrade.
- **Lenses**: L1 ✅ (grep) · L2 ✗ (no design reason to omit; wrapper constraint does not block a `lang` on the content root) · L3 ✅ (contract §6 mandates French artifacts for this operator; a11y follows). Survives.

### F-3 · MEDIUM · Table semantics: no `<thead>`, no `scope`, no `<caption>`
- **Where**: all 4 tables; header rows are bare `<tr><th>…` (e.g. line 128), `scope=` count 0, `<thead>` count 0.
- **Impact**: screen readers can still guess column headers, but `scope="col"` + `<thead>` make the 5-row proof matrix (the page's hinge component) unambiguous; captions would name each table for AT users jumping between 4 tables.
- **Lenses**: L1 ✅ (grep + DOM) · L2 partial (minimal markup is a style choice, but costs nothing to fix) · L3 ✅ (contract: "readable tables"; a11y-as-design phase 8 doctrine). Survives.

### F-4 · MEDIUM-LOW · Measure 71.5ch vs contract target "near 65ch"
- **Where**: `.wrap { max-width: 76ch }` (line 57); measured running-text width 636px = **71.5ch** at 16px system-ui (probe `effectiveCh`, both schemes).
- **Impact**: ~10% wider than the contract's readability target; lines of French prose run long (visible in desktop screenshots). Not a defect of the page's own claim (it honestly says 76ch) but a deviation from the standard the brief audits against.
- **Lenses**: L1 ✅ (measured) · L2 partial (76ch container minus 2.5rem padding was plausibly chosen to land NEAR the target; 71.5 ≠ near 65) · L3 ✅ (contract §4 explicit). Survives as MEDIUM-LOW.

### F-5 · LOW · Four dead CSS rules (speculative styling, R-KARPATHY)
- **Where**: `.note` (line 86, 0 uses), `.pill.warn` (lines 80-82, 0 `class="pill warn"` in markup), `h3` (line 64, 0 `<h3>` elements · the probe crashed on it first pass), `pre`/`pre code` (lines 78-79, 0 `<pre>` elements).
- **Impact**: ~8% of the stylesheet styles nothing; every future clone of this page as a template inherits the dead weight. Harmless at runtime (0 console errors).
- **Lenses**: L1 ✅ (grep counts + null selector at runtime) · L2 partial (a template-ish "component kit" is defensible, but this ships as a single page, not a template) · L3 ✅ (R-KARPATHY: no speculative abstractions). Survives as LOW.

### F-6 · LOW · Inline style bypasses the token/stylesheet system
- **Where**: line 185, `<p class="sub" style="margin-top:2.5rem">`.
- **Impact**: the only inline style in the file; the footer spacing decision lives outside the stylesheet. One-line hygiene fix (a `.footer` rule or utility class).
- **Lenses**: L1 ✅ (grep) · L2 ✗ (no reason a one-off margin must be inline) · L3 ✅ (contract: style through tokens/stylesheet). Survives.

### F-7 · LOW · Sub-12px type at three levels
- **Where**: `.pill` 11.2px (line 80), `th` 11.52px (line 74), `.eyebrow`/`.toc .t` 11.52px (lines 59, 70).
- **Runtime mitigation**: all pass WCAG AA even at small size: weakest pair `pill.ok` on light surface = **4.96:1**; everything else 5.4-14.5:1 (probe `contrast`, both themes). Uppercase + letter-spacing keeps them legible in screenshots.
- **Verdict**: legibility floor is respected; flagged because 11.2px uppercase mono is at the threshold on low-DPI screens. Cosmetic watch item, not a violation.

### F-8 · INFO · `td.num` carries non-numeric content
- **Where**: lines 174-175: `<td class="num">(v2)</td>`; line 173: `14a07b6 / 93f5b26`.
- **Impact**: `.num` (tabular-nums, mono) used as a "mono cell" style for placeholders. Semantic stretch only; rendering is fine. Rename to `.mono` usage (class exists, line 76) for placeholder cells.

### F-9 · INFO · Reduced-motion guard is inert
- **Where**: line 88. The page has zero `animation`/`transition` declarations, so the guard is a no-op. Claim technically satisfied; noted so nobody counts it as exercised protection.

### Killed candidates (for the record, per the ≥2-of-3 gate)
- "System font stack as the page's voice = anti-slop violation" · KILLED: contract §4 explicitly permits system stacks under the artifact CSP (no external fonts); the mono-eyebrow + numbered-key treatment supplies the identity. L2 steelman holds.
- "Margin-stacked document flow violates the flex/grid-with-gap rule" · KILLED: the contract's layout rule targets component layouts; a single-column prose report in document flow with a margin rhythm is the idiomatic and correct choice. L2 holds.
- "Numbers not right-aligned in tables (phase 17 convention)" · KILLED: the only `.num` columns hold commit SHAs (identifiers, not quantities); left alignment is correct for identifiers. L2 holds.

## 3. Phase scores (uiuxaudit weights; N/A phases excluded per preamble §5)

| Phase | Score | Weighted | Notes |
|---|---|---|---|
| 1 Color system | 9/10 | 18/20 | Full token coverage, semantic names, dark palette adapted (not inverted); all 8 measured pairs ≥4.96:1 both themes; 1 dead token path (.pill.warn) |
| 2 Typography | 8/10 | 20/25 | Clear H1>H2, balance, 1.6 line-height, 16px mobile; measure 71.5ch vs 65ch target (F-4), sub-12px labels (F-7), dead h3 rule |
| 3 Spacing & rhythm | 8/10 | 16/20 | Consistent rem scale; single inline-style outlier (F-6) |
| 4 Component anatomy | 8/10 | 24/30 | Pills/tables/TOC coherent; 4 dead component rules (F-5); links rely on default underline (acceptable for prose) |
| 5 Coherence | 9/10 | 27/30 | Single page vs contract: sections, tables, badges uniform; both themes feel like the same designed object (desktop-light/dark.png) |
| 6 Interaction & motion | 8/10 | 16/20 | Static doc; focus states real (C5); reduced-motion inert (F-9) |
| 7 Responsive | 9/10 | 22.5/25 | Zero body overflow at 375 (probe); tables scroll internally; 16px body preserved |
| 8 Accessibility | 7/10 | 17.5/25 | Focus + contrast strong; missing lang (F-2), table semantics (F-3), small uppercase labels (F-7) |
| 9 Design smells | 9/10 | 18/20 | No banned combos (no cream+serif+terracotta, no gradient hero, no emoji markers); distinctive editorial-mono identity |
| 10 Visual hierarchy | 9/10 | 27/30 | TLDR-first, numbered mono keys, evidence tables dominate exactly where they should; single H1 (probe `headings`) |
| 11 Copy & microcopy | 9/10 | 18/20 | Precise French, R-CITE-shaped evidence cells, honest "Limites" section |
| 12 Performance as design | 10/10 | 20/20 | One 14KB file, zero JS, zero requests, system fonts, no CLS surface, 0 console errors |
| 13 Dark mode (+print) | 6/10 | 12/20 | Dark theme itself excellent; print-under-dark token leak is HIGH (F-1) |
| 14 System maturity | 8/10 | 20/25 | Semantic tokens, single source; dead rules + inline style are the gaps |
| 15 Navigation | 9/10 | 22.5/25 | TOC all 7 anchors resolve (probe `tocTargets`), aria-label on nav, print hides TOC |
| 17 Data visualization | 9/10 | 18/20 | tabular-nums verified; identifier alignment correct; `.num` semantic stretch (F-8) |
| 19 Brand expression | 9/10 | 18/20 | Signature detail (Spruce): the mono numbered section keys + pill evidence system; cover the logo and it still reads OmegaOS-forensic |
| 16 Onboarding, 18 Error recovery | N/A | · | Static report page: no first-use flow, no error states |

**Raw: 334.5 / applicable max 375 → normalized 89/100 → Grade A.**
Gestalt whole-over-parts check: the whole is MORE coherent than the part scores suggest (no cap applied). What keeps it from S-tier is one real HIGH defect on a claimed surface (print/dark) plus the a11y semantics layer.

## 4. v2 meta-protocol output

```json
{
  "score": 89,
  "confidence": "high",
  "skill_used": "uiuxaudit",
  "ticket": "uiuxaudit-artifact-page",
  "mode": "audit-only (fixes forbidden by brief; phases 21-23 not run)",
  "derived_inputs_note": "--user-need/--hinge not passed as flags; taken verbatim from the operator brief (recorded in section 0) instead of refusing, per L3 and the brief's Done Criteria",
  "user_need_match": {
    "quote": "It claims: token-level dual themes (@media prefers-color-scheme dark + :root[data-theme] overrides that must win in both directions), 76ch measure, tabular-nums on numeric cells, overflow-x auto table containers, focus-visible outlines, prefers-reduced-motion, @media print, zero em/en dashes, French copy. Audit against those claims and the design contract in ~/.omega/skills/artifact-design/SKILL.md section 4.",
    "addressed": true,
    "evidence": "All 9 claims individually tested against the rendered runtime (section 1 matrix): 7 PASS, 1 PASS-with-deviation (measure), 1 PARTIAL FAIL (@media print under dark scheme, F-1). Contract section 4 checked line by line (killed-candidates record included).",
    "edge_cases_covered": ["data-theme override under OPPOSITE OS scheme, both directions", "print emulation under BOTH color schemes", "375px overflow with wide tables", "keyboard focus in both themes"]
  },
  "falsifiable_tests": [
    {"name": "theme toggle beats OS preference (both directions)", "hypothesis": "if the data-theme blocks did not win, forcing data-theme=light under a dark OS would leave --bg at #10181c", "command": "playwright evaluate: set documentElement data-theme under colorScheme:'dark' and 'light', read --bg", "expected": "forcedLight #f6f7f5 under dark OS; forcedDark #10181c under light OS", "actual": "forcedLightBg=#f6f7f5 (dark OS), forcedDarkBg=#10181c (light OS) · probe-results.json", "passed": true},
    {"name": "print legibility under dark scheme", "hypothesis": "if print tokens were reset, inline code under print+dark would be dark-on-light", "command": "playwright emulateMedia print + colorScheme dark, computed styles on p code / td", "expected": "readable dark-on-light", "actual": "code rgb(17,17,17) on rgb(12,20,24) = 1.05:1; td on rgb(22,33,38) · FAILED, print-dark.png", "passed": false},
    {"name": "body never scrolls sideways at 375px", "hypothesis": "wide tables would push scrollWidth past 375 if .tablebox containment failed", "command": "playwright viewport 375: scrollingElement.scrollWidth vs clientWidth + per-.tablebox scrollable check", "expected": "375/375, tables scroll internally", "actual": "scrollW=375 clientW=375 both themes; 2 of 4 tableboxes internally scrollable, overflow-x:auto on all 4", "passed": true},
    {"name": "R-NODASH kill pass", "hypothesis": "any U+2012-2015 in the file fails the claim", "command": "grep -nP '[\\x{2013}\\x{2014}\\x{2012}\\x{2015}]' agentic/reports/omegaos-report-surfaces-rollout.html", "expected": "no matches", "actual": "NO_DASHES", "passed": true},
    {"name": "focus-visible + contrast floor", "hypothesis": "Tab focus without a visible ≥3:1 outline fails C5", "command": "playwright keyboard.press Tab, read outline computed style; WCAG ratio math on 8 color pairs per theme", "expected": "visible outline ≥3:1; text pairs ≥4.5:1", "actual": "2px solid accent, offset 2px, 5.42:1/7.85:1; weakest text pair 4.96:1 (pill.ok, light)", "passed": true}
  ],
  "hinge_findings": [
    {"location": "agentic/reports/omegaos-report-surfaces-rollout.html:3-54 (token cascade)", "concern": "data-theme overrides could lose to the @media block or only win one direction", "verified_safe_by": "theme toggle test above (both directions, both OS schemes)"},
    {"location": "agentic/reports/omegaos-report-surfaces-rollout.html:89-93 (@media print)", "concern": "print resets body only, not the 9 theme tokens", "verified_safe_by": "NOT safe: falsified by the print-under-dark test (F-1 HIGH)"},
    {"location": "agentic/reports/omegaos-report-surfaces-rollout.html:127-134 (proof table, hinge component)", "concern": "the page's credibility rests on this table being readable everywhere", "verified_safe_by": "desktop/mobile screenshots both themes + contrast table; FAILS only in print+dark (covered by F-1)"}
  ],
  "issues_found_and_fixed": [
    {"severity": "high", "location": "html:89-93", "issue": "F-1 print under dark scheme illegible (dark-token leak)", "fix_applied": "NONE (read-only brief); prescribed: re-assert light tokens on :root inside @media print"},
    {"severity": "medium", "location": "html:96", "issue": "F-2 no lang=\"fr\" on content root", "fix_applied": "NONE; prescribed: lang=\"fr\" on .wrap"},
    {"severity": "medium", "location": "html:128 et al.", "issue": "F-3 tables lack thead/scope/caption", "fix_applied": "NONE; prescribed"},
    {"severity": "low", "location": "html:57", "issue": "F-4 measure 71.5ch vs contract ~65ch", "fix_applied": "NONE; prescribed: max-width ~70ch or contract amendment"},
    {"severity": "low", "location": "html:64,78-82,86", "issue": "F-5 four dead CSS rules", "fix_applied": "NONE; prescribed: prune or use"},
    {"severity": "low", "location": "html:185", "issue": "F-6 inline style", "fix_applied": "NONE; prescribed"},
    {"severity": "low", "location": "html:74,80", "issue": "F-7 sub-12px labels (contrast-mitigated)", "fix_applied": "NONE; watch item"}
  ],
  "confidence_basis": "Every scored claim was verified against computed styles, WCAG math, or screenshots from a real Chromium render in both color schemes, both viewports, and print emulation; the one FAIL was reproduced twice (computed styles + screenshot). Nothing load-bearing was assumed from source reading alone. Bounds: publish-wrapper behavior, non-Chromium engines, and physical print were not exercised (section 0).",
  "finished_at": "2026-07-03T11:58:00Z"
}
```

## 5. Fix priority for the follow-up worker (no fixes applied here)

1. **F-1** print token reset (HIGH, ~10 lines of CSS, fixes the only broken claimed surface).
2. **F-2** `lang="fr"` on `.wrap` (1 attribute).
3. **F-3** `<thead>` + `scope="col"` on 4 tables.
4. **F-5/F-6** prune 4 dead rules, move the inline style into the stylesheet.
5. **F-4** decide: tighten `.wrap` toward ~70ch max-width, or amend the contract's 65ch wording to match the shipped 76ch convention (skill and rule must agree either way).

*Evidence bank retained at `/tmp/uiuxaudit-artifact/` (8 screenshots + probe-results.json + render.html harness). Original page untouched; this file is the audit's single write.*
