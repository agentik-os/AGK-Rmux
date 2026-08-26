---
name: a11yaudit
description: >
  Forensic accessibility audit v1 (Gestalt-Popper). 21-phase deep analysis of every
  accessibility surface: WCAG 2.1 AA compliance, keyboard navigation (every interactive element),
  screen reader testing, ARIA labels/roles/states, color contrast (4.5:1 AA, 3:1 large text),
  focus management, skip navigation, form labels, error announcements, alt text, heading hierarchy,
  landmark regions, touch targets (44px), motion/animation preferences, cognitive load, reading level,
  plus verdict, fix plan, fix execution, re-audit, and integration smoke gate.
  Score /320. Preamble v1.0 compliant. Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/a11yaudit", "accessibility audit", "wcag audit", "a11y check",
  "keyboard navigation test", "screen reader test", "color contrast check".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: accessibility
phases: 21
max_score: 320
read_only: false
triggers: ["a11y", "accessibility", "wcag", "keyboard navigation", "screen reader"]
---


<!-- AUDIT-META-V2-INJECTED -->

> ## ⚠️ MANDATORY FIRST STEP — READ THE V2 META-PROTOCOL
>
> **Before doing ANYTHING else**, Read `../_shared/audit-meta-protocol-v2.md`.
>
> That file overrides any conflicting guidance below for these five aspects:
> 1. Required CLI inputs (`--user-need`, `--hinge` are MANDATORY since 2026-05-08)
> 2. Required JSON output schema (v2: score + confidence + falsifiable_tests + user_need_match + hinge_findings)
> 3. Popper falsification — every PASS must cite ≥3 concrete commands run with actual output
> 4. Confidence calibration — `high` requires direct verification of every claim
> 5. Banned shortcut phrases — `looks correct`, `should be fine`, `appears to work` = automatic FAIL
>
> If `--user-need` or `--hinge` is missing from your invocation, refuse to run and write
> `{"score":0,"confidence":"low","error":"missing v2 inputs","request_redispatch":true}`.
>
> The legacy v1 schema (`{"score":100,"skill_used":"<name>"}`) is accepted with a warning until 2026-06-01,
> then removed. Always emit v2 going forward.
>
> Model context: this audit runs on Opus 4.7 with max effort. There is no time pressure.
> Run every test you claim to have run. Cite verbatim outputs. No exceptions.

---

# /a11yaudit v1 — Forensic Accessibility Audit (Gestalt-Popper)

> *"The other audits ask 'does it work?' I ask 'can EVERYONE use it?'"*

---

## DOCTRINE

You are not an accessibility checker. You are an **accessibility forensic pathologist**. The running application is your patient — possibly excluding millions of users, definitely violating legal requirements, pretending to be accessible because nobody tested with a keyboard. Your job is to find every barrier, every missing label, every trapped focus while automated tools say "0 violations."

**The 5 Laws of Accessibility Forensics (Gestalt-Popper Synthesis):**
1. **If it renders, it's still guilty.** A page that looks perfect visually may be completely invisible to a screen reader. Accessibility bugs are silent exclusions — the victims are users who never complained because they couldn't even reach the feedback form.
2. **Automated tools lie (Popper).** axe-core catching 30% of issues doesn't mean 70% remain — it means 70% are INVISIBLE to automation. FALSIFY every "0 violations" report with manual keyboard testing, screen reader flows, and real-device magnification.
3. **Every missing label is a locked door.** That unlabeled button. That image without alt text. That form with no error announcement. Each is a barrier for someone who has no alternative path.
4. **Clarity before scanning (Gestalt).** Before launching any tool, UNDERSTAND the product. Read VISION.md, CLAUDE.md, README. Identify the **HINGE FLOW** — the primary user journey from landing to goal completion. Test the hinge flow with 10x depth.
5. **"Works for me" is not accessible (Popper).** "I can tab through it" means nothing if focus isn't visible. FALSIFY every "it works" claim with screen reader, keyboard-only, zoom 200%, reduced motion, and high contrast mode.

**Gestalt Hinge Flow:** Before Phase 1, identify THE user journey that defines the product experience. Signup -> Dashboard -> Primary Action. THIS flow gets every phase at maximum depth.

**Popper Accessibility Falsification Categories:**
- **VISUAL vs SEMANTIC** — Looks like a button, but is a div with an onClick
- **MOUSE vs KEYBOARD** — Works with click, impossible with Tab/Enter
- **SIGHTED vs SCREEN READER** — Visible text, but no programmatic association
- **DESKTOP vs MOBILE** — Accessible on desktop, touch targets too small on mobile
- **DEFAULT vs PREFERENCE** — Animations play, even with prefers-reduced-motion

---

## SCOPE DETECTION (automatic)

```
EXAMPLES:
  "/a11yaudit"
  -> Full 16-phase pipeline. Discover all pages, test everything.

  "/a11yaudit the signup flow"
  -> TARGETED: only signup/auth pages
  -> All phases scoped to signup routes

  "/a11yaudit keyboard"
  -> KEYBOARD-FOCUSED: tab order, focus traps, interactive elements

  "/a11yaudit forms"
  -> FORM-FOCUSED: labels, errors, validation, required fields

  "/a11yaudit contrast"
  -> CONTRAST-FOCUSED: color ratios, text on backgrounds, UI components
```

---

## OUTPUT CONTRACT

```
audits/.a11yaudit/
|-- session.log
|-- discovery/
|   |-- pages.json              # All discovered routes/pages
|   |-- interactive-elements.json # All buttons, links, inputs, etc.
|   |-- aria-inventory.json     # All ARIA attributes found
|   |-- heading-map.json        # Heading hierarchy per page
|-- reports/
|   |-- wcag-compliance.md          # Phase 1
|   |-- keyboard-navigation.md     # Phase 2
|   |-- screen-reader.md           # Phase 3
|   |-- aria-audit.md              # Phase 4
|   |-- color-contrast.md          # Phase 5
|   |-- focus-management.md        # Phase 6
|   |-- skip-navigation.md         # Phase 7
|   |-- form-labels.md             # Phase 8
|   |-- error-announcements.md     # Phase 9
|   |-- alt-text.md                # Phase 10
|   |-- heading-hierarchy.md       # Phase 11
|   |-- landmark-regions.md        # Phase 12
|   |-- touch-targets.md           # Phase 13
|   |-- motion-preferences.md      # Phase 14
|   |-- cognitive-load.md          # Phase 15
|   |-- reading-level.md           # Phase 16
|-- verdict.json
|-- verdict.md
|-- fix-plan.json
|-- fix-plan.md
|-- progress.json
|-- fix-log.md
```

---

## PHASE 0 — PROGRAMMATIC GATHER (HYBRID, runs FIRST, before all other phases)

> **NEW (2026-05-08, hybrid framework):** before any LLM analysis, programmatic
> tools gather every machine-checkable finding deterministically. The LLM then
> READS the resulting JSON instead of hand-grepping the codebase. Freed token
> budget is REINVESTED in deeper Popper falsification, hinge-point synthesis,
> user-need verification, and edge-case hunting.

### 0.1 Run the gather script (mandatory, FIRST step)

```bash
~/.omega/lib/audit-runner.sh a11y "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes `~/.omega/lib/audit-gather/a11y.sh` which runs:
axe-core, pa11y, and Lighthouse a11y category — runs in headless Chromium against a live URL

Output is written to:

```
$PROJECT_PATH/audits/.a11yaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), the artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.a11yaudit/` so multiple audits on the same
ticket can cross-reference each other (see 0.5).

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "a11y",
  "tools_run": ["..."],
  "tools_skipped": [{"tool": "...", "reason": "..."}],
  "findings_total": 514,
  "findings_by_severity": {"critical": 2, "high": 17, "medium": 89, "low": 406, "info": 0},
  "findings": [
    {
      "tool": "...",
      "severity": "critical|high|medium|low|info",
      "location": "file:line[:col]",
      "rule": "...",
      "message": "...",
      "suggested_fix": "...",
      "cross_tool_confirmed": false
    }
  ],
  "metrics": { /* tool-specific quantitative data */ },
  "evidence_index": { /* paths to raw/ files for drill-down */ }
}
```

### 0.3 What you do AFTER the gather (this replaces hand-greps)

You now consume `evidence-summary.json` programmatically. You MUST:

1. **Read `evidence-summary.json` in full.** This is your evidence base.
2. **Read 3-5 critical files only** — the ones flagged as load-bearing in
   `~/.omega/state/hinge-points-<ticket>.json` (or computed via
   `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh` if no ticket).
3. **DO NOT manually grep the codebase for what the gather already covered.**
   The tools have already exhaustively scanned every file. Re-running grep
   wastes tokens and produces the same evidence.
4. **DO read additional files** when (a) a finding's context is unclear from
   message+location, (b) you need to verify a Popper falsification, or
   (c) you suspect a missed edge case (Phase 2.4 below).

### 0.4 Banned operations after Phase 0

These are now forbidden because the gather already did them. If you catch
yourself about to run one, STOP and read `evidence-summary.json` first:

- ❌ `grep -rn "TODO" .` (the gather scanned for it)
- ❌ `find . -name "*.ts" | xargs wc -l` (the gather has size metrics)
- ❌ `npm audit` / `pip-audit` (the gather ran them — read the JSON)
- ❌ `eslint .` / `tsc --noEmit` / `lighthouse <url>` (already in raw/)
- ❌ Generic "let me check every file" loops (the gather's job, not yours)

You MAY still:
- ✅ Read SPECIFIC files cited in findings (verify the issue)
- ✅ Run a SPECIFIC `grep` to falsify a finding (Popper test, see Phase 2.1)
- ✅ Run a SPECIFIC tool the gather couldn't (e.g. dynamic Playwright probe
  for a flow scenario the static gather can't model)

### 0.5 Cross-audit synthesis (read sibling evidence-summary.json files)

If this audit runs as part of a Linear-fix mission, sibling audits' summaries
are at `$PROJECT_PATH/audits/.linear-fix/<TICKET>/.<other-audit-id>/evidence-summary.json`.
Read them. Use them.

Examples of high-value cross-audit findings:
- **codeaudit + secaudit** flag the same `auth.ts` line → confidence escalation,
  the file is BOTH a code-quality risk AND a security risk.
- **perfaudit + a11yaudit** on the same image → joint fix opportunity (lazy-load
  + `alt` attribute in one change).
- **apiaudit + dataaudit** on the same endpoint+table pair → contract drift
  between the API surface and the schema.
- **debugaudit + flowaudit** report the same broken page → user-flow blocker.

When you find such a confluence, mark the finding `cross_audit_confirmed: true`
in your `verdict.json` and bump severity by one level.

---

## PHASE 0: RECONNAISSANCE

> *"Know who your users are before testing what they can't do."*

```
1. PROJECT DISCOVERY
   -> Read CLAUDE.md, README, package.json
   -> Identify: framework, component library, CSS approach
   -> Find: prod URL, dev URL, target audience, known a11y requirements

2. PAGE/ROUTE DISCOVERY
   -> Scan all routes and interactive pages
   -> Build complete sitemap with interaction types per page
   -> Identify high-frequency user journeys (hinge flows)

3. INTERACTIVE ELEMENT INVENTORY
   -> List all buttons, links, inputs, selects, custom widgets
   -> List all modals, dialogs, dropdowns, tooltips, tabs
   -> List all media (images, videos, audio)
   -> List all dynamic content (live regions, notifications, loading states)

4. ACCESSIBILITY BASELINE
   -> Run axe-core automated scan on all pages
   -> Document existing ARIA usage patterns
   -> Note any accessibility statement or VPAT
   -> This becomes the "before" for comparison
```

---

## PHASE 1: WCAG 2.1 AA COMPLIANCE AUDIT

> *"The law says AA. Most sites fail Level A. That's not a gap — it's a chasm."*

```
FOR EVERY discoverable page:

1. PERCEIVABLE (WCAG 1.x)
   -> 1.1.1 Non-text content: every image, icon, chart has text alternative
   -> 1.2.x Time-based media: captions, audio descriptions, transcripts
   -> 1.3.1 Info and relationships: structure conveyed programmatically
   -> 1.3.2 Meaningful sequence: reading order matches visual order
   -> 1.3.3 Sensory characteristics: not "click the red button"
   -> 1.3.4 Orientation: works in both portrait and landscape
   -> 1.3.5 Input purpose: autocomplete attributes on personal data fields
   -> 1.4.x Visual: contrast, resize, images of text, reflow, spacing

2. OPERABLE (WCAG 2.x)
   -> 2.1.1 Keyboard: all functionality available via keyboard
   -> 2.1.2 No keyboard trap: focus can always escape
   -> 2.1.4 Character key shortcuts: can be turned off/remapped
   -> 2.2.x Timing: adjustable, pause/stop, no interruptions
   -> 2.3.1 Three flashes: no content flashes > 3 times/second
   -> 2.4.x Navigation: skip links, page titles, focus order, link purpose
   -> 2.5.x Input modalities: pointer gestures, pointer cancellation, label in name, motion actuation

3. UNDERSTANDABLE (WCAG 3.x)
   -> 3.1.1 Language of page: lang attribute present
   -> 3.1.2 Language of parts: foreign phrases marked
   -> 3.2.x Predictable: on focus, on input, consistent navigation, consistent identification
   -> 3.3.x Input assistance: error identification, labels, error suggestion, error prevention

4. ROBUST (WCAG 4.x)
   -> 4.1.1 Parsing: valid HTML (no duplicate IDs)
   -> 4.1.2 Name, role, value: custom components expose correct semantics
   -> 4.1.3 Status messages: conveyed without focus change

FALSIFY: Run automated scan AND manual check on every criterion. If automated says "pass," manually verify with screen reader. 30% of real failures pass automated checks.
```

---

## PHASE 2: KEYBOARD NAVIGATION (THE HINGE)

> *"If you can't use it with a keyboard, it doesn't work. Period."*

```
THIS IS THE HINGE PHASE. Score this with 2x weight.

FOR EVERY interactive element on EVERY page:

1. TAB ORDER
   -> Tab through entire page start to finish
   -> Is order logical? (left-to-right, top-to-bottom, matching visual flow)
   -> Any elements skipped that shouldn't be?
   -> Any non-interactive elements receiving focus?
   -> tabindex > 0 used anywhere? (anti-pattern)

2. FOCUS VISIBILITY
   -> Is focus indicator ALWAYS visible?
   -> Custom focus styles meet 3:1 contrast ratio?
   -> Focus indicator at least 2px thick?
   -> No CSS outline:none without replacement?
   -> Focus visible in all color modes (light/dark/high contrast)?

3. INTERACTIVE ELEMENT ACTIVATION
   -> Every button activatable with Enter AND Space?
   -> Every link activatable with Enter?
   -> Custom dropdowns operable with arrow keys?
   -> Custom tabs operable with arrow keys?
   -> Escape closes modals/dropdowns?

4. KEYBOARD TRAPS
   -> Can focus leave every modal/dialog?
   -> Can focus leave every dropdown/menu?
   -> Date pickers keyboard navigable?
   -> Custom widgets allow focus escape?
   -> No infinite tab loops?

5. COMPLETE FLOW TEST
   -> Complete primary user journey keyboard-only
   -> Complete secondary user journeys keyboard-only
   -> Time keyboard flow vs mouse flow (target: < 2x mouse time)
   -> Document every keyboard failure point

FALSIFY: Disconnect mouse. Complete entire hinge flow. If you get stuck at ANY point, that's a CRITICAL failure.
```

---

## PHASE 3: SCREEN READER TESTING

> *"If the screen reader can't describe it, it doesn't exist."*

```
1. PAGE STRUCTURE ANNOUNCEMENT
   -> Page title announced on navigation
   -> Headings announced in correct hierarchy
   -> Landmarks announced (main, nav, aside, footer)
   -> List items announced with count
   -> Table structure announced (rows, columns, headers)

2. INTERACTIVE ELEMENT ANNOUNCEMENTS
   -> Buttons: name + role announced
   -> Links: destination or purpose announced
   -> Inputs: label + type + state announced
   -> Checkboxes/radios: label + checked state
   -> Custom components: correct role announced

3. DYNAMIC CONTENT
   -> Notifications announced via live regions
   -> Loading states announced
   -> Error messages announced immediately
   -> Content updates in live regions announced
   -> Form submission results announced

4. NAVIGATION EFFICIENCY
   -> Can jump to main content (skip nav)
   -> Can navigate by headings
   -> Can navigate by landmarks
   -> Can navigate by form elements
   -> Reading order matches visual layout

5. VIRTUAL BUFFER TESTING
   -> All content reachable in browse mode
   -> Form mode switches correctly
   -> Application mode used appropriately
   -> No content hidden from screen reader that should be visible
```

---

## PHASE 4: ARIA LABELS, ROLES, AND STATES

> *"ARIA is the bridge between what's visual and what's programmatic. Broken bridges are worse than no bridge."*

```
1. ARIA ROLE AUDIT
   -> Custom widgets have correct roles (dialog, menu, tab, etc.)
   -> No redundant roles on native elements (role="button" on <button>)
   -> Abstract roles not used (widget, roletype, structure)
   -> Document/application roles used correctly (not on <body>)
   -> Composite roles have required children (menu -> menuitem)

2. ARIA LABEL AUDIT
   -> aria-label on icon-only buttons
   -> aria-label on ambiguous links ("Read more" -> "Read more about X")
   -> aria-labelledby references existing, visible elements
   -> aria-describedby for supplementary info (not primary label)
   -> Labels match visible text (label-in-name)

3. ARIA STATE AUDIT
   -> aria-expanded on collapsible sections
   -> aria-selected on tabs and options
   -> aria-checked on custom checkboxes
   -> aria-current on navigation (current page/step)
   -> aria-disabled vs HTML disabled (correct usage)
   -> aria-hidden not hiding visible content
   -> aria-invalid on invalid form fields

4. ARIA RELATIONSHIP AUDIT
   -> aria-controls connects trigger to target
   -> aria-owns for DOM reordering
   -> aria-flowto for non-linear reading order
   -> aria-errormessage on fields with errors
   -> Form groups with aria-labelledby or fieldset/legend

5. FIRST RULE OF ARIA
   -> If native HTML can do it, ARIA is NOT needed
   -> Count ARIA overrides of native semantics -> each is a risk
   -> Verify: removing all ARIA, does native HTML convey the same info?
```

---

## PHASE 5: COLOR CONTRAST

> *"4.5:1 isn't a suggestion. It's the minimum for someone with 20/40 vision."*

```
1. TEXT CONTRAST (AA minimum)
   -> Normal text (< 18pt / < 14pt bold): 4.5:1 minimum
   -> Large text (>= 18pt / >= 14pt bold): 3:1 minimum
   -> Check EVERY text color against EVERY background it appears on
   -> Include text over images/gradients (worst-case pixel)
   -> Include placeholder text (often fails)

2. UI COMPONENT CONTRAST
   -> Form input borders: 3:1 against background
   -> Button boundaries: 3:1 against surrounding area
   -> Focus indicators: 3:1 against adjacent colors
   -> Icons that convey meaning: 3:1 minimum
   -> Charts/graphs data elements: 3:1 between adjacent elements

3. STATE CONTRAST
   -> Hover state distinguishable (not just color change)
   -> Active/pressed state distinguishable
   -> Disabled state still readable (though can be lower contrast)
   -> Selected/unselected in lists, tabs, toggles
   -> Error/success/warning not conveyed by color alone

4. COLOR-ONLY INFORMATION
   -> Links distinguishable from text without color alone (underline, weight)
   -> Required fields indicated without color alone (asterisk, text)
   -> Error messages have icon/text, not just red
   -> Charts have patterns/labels, not just colors
   -> Status indicators have text/icon, not just color

5. DARK MODE / HIGH CONTRAST
   -> All contrast ratios hold in dark mode
   -> All contrast ratios hold in Windows High Contrast
   -> Forced-colors media query supported
   -> No information lost in high contrast mode

FALSIFY: Use grayscale filter on entire page. If ANY information is lost, contrast-only communication is present.
```

---

## PHASE 6: FOCUS MANAGEMENT

> *"Focus is the keyboard user's cursor. Lose it and you lose the user."*

```
1. FOCUS ON PAGE LOAD
   -> Focus starts at logical position (skip link or main content)
   -> No auto-focus on non-essential elements
   -> Auto-focus on primary input in forms (when appropriate)

2. FOCUS ON ROUTE CHANGE (SPA)
   -> Focus moves to new content on navigation
   -> Page title updates on route change
   -> Focus not lost in void after navigation
   -> Back button restores focus position

3. MODAL/DIALOG FOCUS
   -> Focus moves to dialog on open
   -> Focus trapped within dialog while open
   -> Focus returns to trigger on close
   -> Escape key closes dialog and returns focus

4. DYNAMIC CONTENT FOCUS
   -> New content announced via aria-live (not focus move)
   -> Deleted content: focus moves to logical next element
   -> Expanded content: focus moves into new content OR stays
   -> Loading complete: focus moves to result or stays in context

5. FOCUS MANAGEMENT ANTI-PATTERNS
   -> Focus jumping unexpectedly
   -> Focus lost (goes to <body>) after interaction
   -> Focus moving on hover (not on activation)
   -> Focus moving backward in document order
   -> Multiple focus indicators visible simultaneously
```

---

## PHASE 7: SKIP NAVIGATION

> *"50 Tab presses before reaching content. That's not navigation, it's punishment."*

```
1. SKIP LINK PRESENCE
   -> "Skip to main content" link present
   -> Skip link is first focusable element
   -> Skip link target is correct (#main-content or similar)
   -> Skip link visible on focus (can be visually hidden otherwise)

2. SKIP LINK FUNCTIONALITY
   -> Activating skip link moves focus past navigation
   -> Focus target is valid landmark or heading
   -> Content after skip is immediately accessible
   -> Works with both keyboard and screen reader

3. ADDITIONAL SKIP MECHANISMS
   -> Skip to search (if search is a primary feature)
   -> Skip navigation in repeated blocks (sidebar, footer nav)
   -> Table of contents for long content pages
   -> Section navigation for dashboard pages

4. LANDMARK NAVIGATION (alternative to skip links)
   -> <header>, <nav>, <main>, <aside>, <footer> present
   -> Multiple <nav> elements distinguished with aria-label
   -> Screen reader can jump between landmarks
   -> All content within a landmark region
```

---

## PHASE 8: FORM LABELS

> *"An unlabeled input is a guessing game. Users don't play guessing games — they leave."*

```
1. LABEL ASSOCIATION
   -> Every input has a visible <label> with matching for/id
   -> Every input has programmatic label (label, aria-label, aria-labelledby)
   -> Placeholder text is NOT the only label
   -> Group labels via <fieldset>/<legend> for related inputs

2. LABEL QUALITY
   -> Labels are descriptive ("Email address" not "Input 1")
   -> Labels indicate format requirements ("Date (MM/DD/YYYY)")
   -> Required fields marked visually AND programmatically (aria-required)
   -> Character limits communicated in label or description

3. INPUT TYPES
   -> Correct input types (email, tel, url, number, date)
   -> autocomplete attributes on personal data fields
   -> inputmode set for mobile keyboards
   -> Pattern attributes with descriptive title

4. FORM INSTRUCTIONS
   -> Instructions appear BEFORE the form (not only after)
   -> Required field legend appears BEFORE required fields
   -> Complex forms have introductory text explaining sections
   -> Multi-step forms show progress and current step

5. SELECT AND CUSTOM INPUTS
   -> <select> has visible label
   -> Custom dropdowns have role="combobox" or "listbox"
   -> Custom date pickers have accessible label and instructions
   -> File upload has label indicating accepted formats
```

---

## PHASE 9: ERROR ANNOUNCEMENTS

> *"A sighted user sees red. A screen reader user hears... nothing. That's the bug."*

```
1. INLINE VALIDATION ERRORS
   -> Error messages appear near the invalid field
   -> Error messages are programmatically associated (aria-describedby or aria-errormessage)
   -> Error messages announce via screen reader without page reload
   -> Error messages describe the problem AND how to fix it
   -> Field is marked with aria-invalid="true"

2. FORM SUBMISSION ERRORS
   -> Error summary appears at top of form
   -> Error summary links to specific fields
   -> Focus moves to error summary on submission failure
   -> Error count announced ("3 errors found")
   -> All errors listed with links to fields

3. SUCCESS ANNOUNCEMENTS
   -> Form success confirmed via aria-live or focus management
   -> Success message is programmatically distinct from error
   -> Navigation after success is communicated
   -> No silent redirects after form submission

4. DYNAMIC VALIDATION
   -> Real-time validation announced via aria-live="polite"
   -> Not too chatty (wait for user to finish typing)
   -> Character count announced periodically, not on every keystroke
   -> Password strength meter accessible

5. ERROR PREVENTION
   -> Destructive actions require confirmation
   -> Confirmation dialogs are accessible
   -> Undo available and announced
   -> Data loss warnings on navigation away from unsaved form
```

---

## PHASE 10: ALT TEXT

> *"An image without alt text is a mystery box. And not the fun kind."*

```
1. INFORMATIVE IMAGES
   -> Every informative image has descriptive alt text
   -> Alt text describes the image content AND purpose
   -> Alt text length appropriate (< 150 chars for most)
   -> Complex images (charts, diagrams) have long description

2. DECORATIVE IMAGES
   -> Decorative images have alt="" (empty, not missing)
   -> Background images don't convey information
   -> Decorative icons have aria-hidden="true"
   -> Purely visual separators/borders have no alt

3. FUNCTIONAL IMAGES
   -> Image links have alt describing destination
   -> Image buttons have alt describing action
   -> Logo images have alt with company/site name
   -> Social media icons have alt with platform name + action

4. COMPLEX IMAGES
   -> Charts have data table alternative
   -> Infographics have full text description
   -> Maps have text address/directions
   -> Screenshots have text description of shown content

5. SVG ACCESSIBILITY
   -> <svg> has role="img" and aria-label
   -> Or <svg> has <title> and aria-labelledby
   -> Decorative SVGs have aria-hidden="true"
   -> Inline SVGs not announcing path data
```

---

## PHASE 11: HEADING HIERARCHY

> *"Headings are the table of contents for screen reader users. A broken TOC means no navigation."*

```
1. HEADING STRUCTURE
   -> Exactly one <h1> per page
   -> Headings don't skip levels (h1 -> h3 without h2)
   -> Heading levels reflect content hierarchy
   -> All sections introduced by a heading

2. HEADING CONTENT
   -> Headings are descriptive of section content
   -> Headings are unique within page (or clearly contextual)
   -> No empty headings (styled div pretending to be heading)
   -> No non-heading text styled to look like a heading

3. HEADING NAVIGATION
   -> Screen reader heading navigation reveals page structure
   -> Can understand page layout from headings alone
   -> Card titles use appropriate heading level
   -> Dynamic content sections have headings

4. HEADING IN COMPONENTS
   -> Modal titles are headings (h2 typically)
   -> Card components use headings for titles
   -> Accordion panels use headings for triggers
   -> Tab panel content starts with heading
```

---

## PHASE 12: LANDMARK REGIONS

> *"Landmarks are the map. Without them, screen reader users wander."*

```
1. REQUIRED LANDMARKS
   -> <main> present (exactly one)
   -> <nav> present for navigation
   -> <header> for site header (banner role)
   -> <footer> for site footer (contentinfo role)

2. LANDMARK LABELING
   -> Multiple <nav> elements have unique aria-label
   -> <aside> elements have aria-label
   -> <section> elements with aria-label are regions
   -> Landmarks have human-readable labels

3. LANDMARK COVERAGE
   -> All visible content within a landmark
   -> No orphaned content between landmarks
   -> Complementary content in <aside>
   -> Search functionality in search landmark

4. LANDMARK NESTING
   -> No invalid nesting (banner inside banner)
   -> Complementary not nested in main (unless intentional)
   -> Navigation can be inside header
   -> Form landmark for significant forms
```

---

## PHASE 13: TOUCH TARGETS

> *"44px. That's Apple's minimum. Google says 48px. Below that, it's a game of Minesweeper."*

```
1. MINIMUM SIZE
   -> All interactive elements >= 44x44px CSS (WCAG) / 48x48dp (Material)
   -> Measure actual rendered size, not just CSS dimensions
   -> Include padding in clickable area
   -> Touch targets in mobile viewport particularly critical

2. SPACING
   -> At least 8px between adjacent touch targets
   -> No overlapping hit areas
   -> Sufficient spacing in navigation menus
   -> Sufficient spacing in form buttons

3. COMMON VIOLATIONS
   -> Close buttons (X) on modals/alerts — often too small
   -> Inline text links — tap area is just the text
   -> Icon buttons without padding
   -> Checkbox/radio native inputs (use custom with larger target)
   -> Breadcrumb links
   -> Pagination links
   -> Table row actions

4. MOBILE-SPECIFIC
   -> Test at smallest supported viewport
   -> Bottom navigation targets adequately sized
   -> Floating action buttons adequately sized
   -> Swipe gestures have alternative tap targets
```

---

## PHASE 14: MOTION AND ANIMATION PREFERENCES

> *"Motion makes your site feel alive. For some users, it makes them feel sick."*

```
1. PREFERS-REDUCED-MOTION
   -> CSS: @media (prefers-reduced-motion: reduce) defined
   -> All CSS animations/transitions reduced or removed
   -> JavaScript animations respect the preference
   -> Page transitions simplified or removed
   -> Loading spinners replaced with static indicators

2. AUTO-PLAYING CONTENT
   -> No auto-playing video (or has pause control)
   -> No auto-playing audio (ever)
   -> Carousels/slideshows have pause button
   -> Auto-scrolling content has stop mechanism
   -> Animated backgrounds can be stopped

3. FLASHING CONTENT
   -> No content flashes > 3 times per second
   -> No strobing effects
   -> Warning before content with flashing
   -> Safe animation speeds (> 333ms per cycle)

4. PARALLAX AND SCROLL EFFECTS
   -> Parallax effects disabled with reduced motion
   -> Scroll-triggered animations respect preference
   -> Sticky/fixed elements don't cause motion
   -> Smooth scrolling can be disabled

5. TRANSITION DURATION
   -> Important transitions under 5 seconds
   -> No animations that block interaction
   -> Skeleton screens preferred over spinners
   -> Loading states don't animate excessively
```

---

## PHASE 15: COGNITIVE LOAD

> *"If it takes a PhD to understand your interface, your interface has a PhD-level accessibility problem."*

```
1. INFORMATION ARCHITECTURE
   -> Navigation has 7 or fewer top-level items
   -> Content hierarchy is clear and consistent
   -> Related items grouped visually and programmatically
   -> Progressive disclosure for complex features

2. LANGUAGE AND INSTRUCTIONS
   -> Instructions use plain language
   -> Technical jargon has explanations
   -> Error messages explain in user terms (not error codes)
   -> Action buttons describe the action ("Save changes" not "Submit")

3. MEMORY DEMANDS
   -> Multi-step processes show progress
   -> Forms preserve user input on error
   -> Important information visible (not hidden in tooltips)
   -> Copy-paste not blocked on any field
   -> Session timeout warning with extension option

4. CONSISTENCY
   -> Same action, same pattern across pages
   -> Same terminology used consistently
   -> Navigation position consistent
   -> Button styles consistent per action type

5. ERROR RECOVERY
   -> Undo available for destructive actions
   -> Clear path back from dead ends
   -> Auto-save for long forms
   -> Confirmation before irreversible actions
```

---

## PHASE 16: READING LEVEL

> *"WCAG says lower secondary education. That's roughly grade 8. Most web copy reads at grade 12+."*

```
1. READABILITY METRICS
   -> Flesch-Kincaid Grade Level (target: <= 8)
   -> Flesch Reading Ease Score (target: >= 60)
   -> SMOG Index
   -> Gunning Fog Index
   -> Measure for: hero text, instructions, error messages, tooltips

2. PLAIN LANGUAGE AUDIT
   -> Active voice preferred over passive
   -> Short sentences (< 20 words average)
   -> Short paragraphs (< 5 sentences)
   -> Common words over technical jargon
   -> Abbreviations spelled out on first use

3. TEXT FORMATTING
   -> Body text 16px minimum
   -> Line height 1.5 minimum for body text
   -> Paragraph spacing 1.5x line height
   -> Letter spacing not less than 0.12em
   -> Word spacing not less than 0.16em
   -> Text resizable to 200% without loss

4. CONTENT ALTERNATIVES
   -> Complex procedures have step-by-step format
   -> Data tables have summary text
   -> Legal text has plain language summary
   -> Instructions include examples
```

---

## PHASE H1 — HYBRID SYNTHESIS (Popper / hinge / user-need / edge cases / cross-audit)

> **NEW (2026-05-08, hybrid framework, runs immediately before VERDICT):**
> "H1" = Hybrid step 1 of the synthesis layer that pairs with Phase 0's
> programmatic gather. It does NOT renumber existing phases; it sits between
> the audit's last domain phase and the VERDICT phase. Between the per-domain
> phases above and the VERDICT phase, you must run this 5-step synthesis.
> The token budget freed by Phase 0's deterministic gather is REINVESTED here
> — depth of analysis is what increases. This phase does NOT replace any
> earlier phase; it deepens them.

### 2.1 Popper falsification per finding (mandatory)

For every finding in `evidence-summary.json.findings[]` (start with `severity ∈
{critical, high}`, then go down as time/budget allows), try to PROVE the tool
is wrong. Each falsification produces a `falsifiable_tests[]` entry in
`verdict.json`:

```jsonc
{
  "claim": "ts-prune says src/auth/legacy.ts:42 export `signWithOldKey` is unused",
  "test_command": "grep -rn 'signWithOldKey' --include='*.ts' --include='*.tsx' --include='*.test.ts' --include='*.spec.ts' . | grep -v 'src/auth/legacy.ts'",
  "expected": "0 matches → claim TRUE, finding stands",
  "actual": "0 matches found",
  "outcome": "confirmed"
}
```

Outcomes:
- `confirmed` — Popper test FAILED to falsify → finding stands, often promoted
- `falsified` — Popper test produced a counter-example → demote to `info` and
  add `falsified_at: <evidence>` to the finding
- `inconclusive` — test could not run cleanly → keep severity, mark
  `confidence: medium` on this finding

**The rule:** every CLAIM in the audit (PASS or FAIL) MUST cite ≥3 concrete
commands that COULD have falsified it but didn't. Banned phrases (`looks
correct`, `should be fine`, `appears to work`) → automatic FAIL of the audit.

Common falsification patterns by category:

| Tool says | Popper test |
|---|---|
| `unused export` (ts-prune, vulture) | `grep` for the symbol in tests/, integration/, and dynamic imports (e.g. `import("...")`) |
| `unused dependency` (depcheck) | `grep` `package.json` scripts + `find . -type f -exec grep -l <pkg>` |
| `circular dep` (madge) | Read all files in the cycle — is the cycle real or a tooling artifact (e.g. type-only re-export)? |
| `console.error` / `console.warn` (debugaudit) | Reproduce the user flow that triggers it; if not reachable from any UI path, demote to info |
| `npm audit HIGH` (sec) | Check if the vulnerable code path is actually called in your codebase; not all transitive vulns are exploitable |
| `LCP > 2.5s` (perf) | Re-run lighthouse twice; check if it's a one-off (cold cache, network blip) or systematic |
| `axe-core color-contrast` (a11y) | Manually compute the ratio; some palettes hit 4.4 vs 4.5 — fixable in tokens |
| `missing canonical` (seo) | Check if the page is intentionally not canonical (paginated, faceted) before flagging |
| `unauthenticated endpoint` (api) | Read the route handler — is auth enforced via middleware not visible to the static scan? |
| `orphaned record` (data) | Verify the FK is supposed to cascade, or confirm the parent was deleted by an actual user action |

### 2.2 Hinge cross-reference (10× scrutiny on load-bearing findings)

The HINGE FLOW of this audit is the locus of maximum risk/value. Compute or
read:

```bash
# If a hinge file already exists for this ticket, use it:
HINGE_FILE="$HOME/.omega/state/hinge-points-${TICKET_ID:-default}.json"

# Otherwise compute it on the fly:
${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh "$PROJECT_PATH" --audit=a11y --user-need="$USER_NEED_QUOTE" \
  > "$HINGE_FILE"
```

For each finding in `evidence-summary.json`, mark
`is_load_bearing: true` IFF its file matches a hinge entry, then apply 10×
scrutiny to those findings:

- 5× more falsification attempts (Phase 2.1)
- 3× more edge-case hunts (Phase 2.4)
- Mandatory read of the entire hinge file (not just the flagged line)
- Mandatory read of all DIRECT callers of the hinge function/symbol
- Mandatory read of all DIRECT callees from the hinge

Output `hinge_findings[]` in `verdict.json`:

```jsonc
{
  "finding_id": "F-042",
  "is_load_bearing": true,
  "hinge_reference": "<hinge term + file>",
  "additional_scrutiny": "verified all 7 callers, all 4 callees, all 12 tests; reproduced bug in 3 of them",
  "confidence_after_scrutiny": "high"
}
```

### 2.3 User-need verification (`--user-need` quote)

If the audit was dispatched with `--user-need="<verbatim user complaint>"`,
every finding MUST be evaluated against it. The user-need is the ground truth
for what counts as "audit succeeded".

For each finding, ask:
- "If a user reported THIS exact issue verbatim, would this finding be the
  cause?"
- "Does fixing this finding make the user-need quote no longer true?"

Findings that DO NOT relate to user-need:
- Get demoted by one severity level UNLESS they are load-bearing
  (Phase 2.2), in which case they retain severity (load-bearing is
  user-need-independent).
- Are still reported, but flagged `user_need_relevance: "tangential"`.

Findings that DO relate to user-need:
- Get the highest priority in the fix plan
- Are listed first in `user_need_match.findings[]` of `verdict.json`

```jsonc
{
  "user_need_match": {
    "addressed": true,
    "user_need_quote": "<verbatim>",
    "rationale": "Finding F-007 (axe-core: input has no associated label) directly causes the user-reported behavior 'I can't tab to the search field'. Fix removes the cause.",
    "findings": ["F-007", "F-019"],
    "untouched_findings_relevant_to_user_need": []
  }
}
```

If `addressed: false`, the audit MUST score below 90/100 even if all phases
otherwise pass. The user's actual problem is the only correct success metric.

### 2.4 Edge case hunting (mandatory for top findings)

For each top-5 finding (sorted by severity × cross-audit-confirmed × hinge),
generate ≥2 edge cases the tool may have missed. The static analyzer or
runtime probe checked the code at rest; you must imagine motion.

Patterns:
- "If user does X under condition Y..." — concurrency, race, double-submit
- "Tool checked the file at rest, but at runtime + concurrent..." — async/await
  ordering, state mutations, cache races
- "Static scan saw the import, but the dynamic require()..." — code-splitting,
  feature-flag gated, lazy()
- "i18n locale switch..." — strings missing in non-default locale
- "Network flake mid-request..." — partial state, retry idempotency
- "User logs out mid-flow..." — session expiration, token refresh
- "Timezone boundary (DST, UTC midnight)..." — date math
- "Data shape: empty array, null, undefined, single-element..." — boundary

Output `edge_cases[]` in `verdict.json`:

```jsonc
{
  "finding_id": "F-007",
  "scenario": "User pastes a multi-line value into the search field; the
    `<input>` strips newlines silently and submits a partial query",
  "covered_by_existing_test": false,
  "evidence_gathered": "Manual repro on prod URL; screenshot at .a11y/edge-evidence/F-007.png",
  "fix_includes_coverage": true
}
```

### 2.5 Cross-audit synthesis (re-read sibling summaries from Phase 0.5)

Now that you have your own `verdict.json` draft, do a final pass with sibling
audits' findings open in context:

1. For each of YOUR top-5 findings: is the same file/line/symbol flagged in any
   sibling audit?
2. If yes → escalate confidence (`cross_audit_confirmed: true`), bump severity
   one level (low→medium, medium→high, high→critical, critical stays).
3. For each sibling top-5 finding: is the same file/line/symbol relevant to
   YOUR audit's domain? If yes, add it to YOUR findings as
   `tool: "cross-audit:<sibling-name>"` with proper severity.
4. Write `cross_audit_links[]` in `verdict.json` summarizing the matches.

```jsonc
{
  "cross_audit_links": [
    {
      "this_finding_id": "F-007",
      "sibling_audit": "secaudit",
      "sibling_finding_id": "F-013",
      "shared_location": "src/auth/login.tsx:42",
      "joint_fix_recommended": true
    }
  ]
}
```

### 2.6 Final verdict.json schema (hybrid v2)

```jsonc
{
  "audit": "a11y",
  "score": 100,
  "score_raw": "<raw>/<denominator>",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "a11y",
  "user_need_match": { ... },                   // §2.3
  "falsifiable_tests": [ ... ],                   // §2.1
  "hinge_findings": [ ... ],                      // §2.2
  "issues_found_and_fixed": [
    {
      "id": "FIX-001",
      "finding_id": "F-007",  // back-ref to evidence-summary.json
      "before": "<state>",
      "after": "<state>",
      "verification": "<command + output>"
    }
  ],
  "edge_cases": [ ... ],                          // §2.4
  "cross_audit_links": [ ... ],                   // §2.5
  "evidence_summary_path": "$PROJECT_PATH/audits/.a11yaudit/evidence-summary.json",
  "confidence_basis": "Why I'm confident (or not). Cite Popper test counts, hinge scrutiny depth, edge-case coverage, cross-audit confirmations.",
  "banned_phrase_check": "passed (no occurrences of `looks correct`, `should be fine`, `appears to work`, `streamlined`, `to save time`)"
}
```

### 2.7 Score gating (hybrid threshold)

A 100/100 score is now blocked unless:

1. ✅ All `critical` and `high` findings are fixed OR have an explicit
   `non_issue_justification` of ≥50 words backed by Popper evidence.
2. ✅ All load-bearing findings (Phase 2.2) confirmed via Popper falsification.
3. ✅ `user_need_match.addressed = true` with verbatim quote of the user-need.
4. ✅ ≥3 falsifiable tests cited per phase (Phase 2.1).
5. ✅ ≥2 edge cases generated per top-5 finding (Phase 2.4).
6. ✅ Cross-audit synthesis attempted (Phase 2.5) — links may be empty if no
      siblings exist, but the array MUST be present.
7. ✅ `confidence_basis` populated with non-trivial reasoning.

Below threshold → score < 100, fix-and-reaudit loop kicks in (existing R-6 flow
in the FIX EXECUTION / RE-AUDIT phases below). The loop is BOUNDED at 5
iterations per the Audit Verification Contract; on iteration 5 if still failing,
emit `confidence: low` and surface as `pending` in `.done.json`.

---

## PHASE 17: VERDICT

Score each phase 0-10, weight by severity:

```
SCORING MATRIX (320 max):
  Phase  1  (WCAG 2.1 AA Compliance)    x 3.0  = max 30
  Phase  2  (Keyboard Navigation)       x 4.0  = max 40  ** HINGE **
  Phase  3  (Screen Reader)             x 3.0  = max 30
  Phase  4  (ARIA Labels/Roles/States)  x 2.5  = max 25
  Phase  5  (Color Contrast)            x 2.5  = max 25
  Phase  6  (Focus Management)          x 2.5  = max 25
  Phase  7  (Skip Navigation)           x 1.5  = max 15
  Phase  8  (Form Labels)               x 2.0  = max 20
  Phase  9  (Error Announcements)       x 2.0  = max 20
  Phase 10  (Alt Text)                  x 1.5  = max 15
  Phase 11  (Heading Hierarchy)         x 1.5  = max 15
  Phase 12  (Landmark Regions)          x 1.5  = max 15
  Phase 13  (Touch Targets)             x 1.5  = max 15
  Phase 14  (Motion Preferences)        x 1.0  = max 10
  Phase 15  (Cognitive Load)            x 1.0  = max 10
  Phase 16  (Reading Level)             x 1.0  = max 10
                                        TOTAL  = max 320

NORMALIZE: score = (raw / 320) x 100

GRADE:
  90-100: S — Exemplary. Meets WCAG 2.1 AA, keyboard perfect, screen reader excellent.
  80-89:  A — Strong. Minor gaps, all primary flows accessible.
  70-79:  B — Adequate. Some barriers, most flows accessible with workarounds.
  60-69:  C — Deficient. Significant barriers, some flows blocked.
  50-59:  D — Poor. Many users excluded, legal risk.
  <50:    F — Exclusionary. Most assistive technology users cannot use the product.
```

---

## PHASE 18: FIX PLAN (automatic)

```
Sort: CRITICAL -> HIGH -> MEDIUM -> LOW
Group by impact (one fix may resolve multiple barriers)
Priority: keyboard traps > missing labels > contrast > cosmetic
Generate fix tasks with file:line specificity
Save to audits/.a11yaudit/fix-plan.json + fix-plan.md
```

---

## PHASE 19: FIX EXECUTION (automatic)

```
Sequential per barrier group.

─── SAFETY GATE: DO NO HARM (MANDATORY before EVERY fix) ──────────────

The audit MUST NOT introduce new bugs. A fix that breaks the code is worse than
the original finding. Every fix goes through this gate BEFORE commit.

PRE-FIX ANALYSIS (before writing ANY code):
  a. Read the ENTIRE target file (not just the target line)
  b. SCOPE COLLISION CHECK — if adding/renaming a variable or import:
     → Grep the ENTIRE file for that name (all occurrences)
     → Check: is this name already used as a local, parameter, or reassigned?
     → Check: does this name get shadowed later in the same scope?
     → If collision found → use a different name or fully-qualified reference
  c. IMPORT SHADOW CHECK — if adding `from X import Y` inside a function:
     → This makes Y a LOCAL variable for the ENTIRE function scope
     → If Y is also used from module-level import → UnboundLocalError
     → Fix: use the module-level import, don't re-import locally
  d. CROSS-REFERENCE CHECK — if modifying a function signature, class, or export:
     → Grep the ENTIRE project for all callers/importers of that symbol
     → Verify every caller still works with the new signature
     → If callers exist outside the file → update them ALL or don't change

POST-FIX VERIFICATION (after writing code, BEFORE commit):
  a. SYNTAX CHECK:
     → Python: `python -c "import ast; ast.parse(open('FILE').read())"`
     → JS/TS: `npx tsc --noEmit` or `node -c FILE`
  b. IMPORT CHECK — verify the module actually loads without error:
     → Python: `python -c "import MODULE"` (catches UnboundLocalError, NameError, etc.)
     → JS: `node -e "require('./FILE')"`
  c. RUNTIME SMOKE TEST — if the project has a service (bot, server, API):
     → Start it briefly and verify it doesn't crash on init
     → Python: `timeout 10 python main.py` or systemctl restart + is-active check
     → Node: `timeout 10 node server.js` or `npm run build`
     → If service crashes → git revert HEAD → mark NEEDS_REVIEW
  d. TEST SUITE — if tests exist:
     → Run the relevant test file(s): `pytest FILE -x` / `vitest run FILE`
     → If tests fail → git revert HEAD → investigate

IF ANY POST-FIX CHECK FAILS:
  → `git revert HEAD` immediately
  → Log the failure in audits/.a11yaudit/fix-log.md with exact error
  → Mark as NEEDS_REVIEW (never retry same approach blindly)
  → Try alternative approach OR skip this fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (in priority order):
  a. Read the ENTIRE target file (full context)
  b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
  c. Apply fix (semantic HTML preferred over ARIA)
  d. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
  e. If all green → commit: a11y(a11yaudit): FIX-XXX description
  f. If any red → revert → log → mark NEEDS_REVIEW
  g. Verify: no regression in other accessibility features
```

---

## PHASE 20: RE-AUDIT (automatic)

```
1. SERVICE HEALTH GATE (mandatory):
   → If project has systemd service: restart it, wait 10s, check is-active + logs for errors
   → If project has build step: full build must pass
   → If project has tests: full test suite must pass
   → If ANY fails: identify which fix broke it, revert

2. Re-run all FAILING phases. Compare before/after.
3. Loop until score >= 80 or remaining items are NEEDS_REVIEW.
```

---

## PARALLEL EXECUTION STRATEGY

```
The 16 phases are grouped for maximum parallelism:

WAVE 1 (discovery -- sequential):
  Phase 0: Reconnaissance

WAVE 2 (structural -- parallel):
  Phase 1: WCAG compliance    ]
  Phase 4: ARIA audit          ] Code-level analysis
  Phase 11: Heading hierarchy  ] agents
  Phase 12: Landmark regions   ]

WAVE 3 (interaction -- parallel):
  Phase 2: Keyboard navigation  ]
  Phase 3: Screen reader         ] Runtime testing
  Phase 6: Focus management      ] agents
  Phase 7: Skip navigation       ]

WAVE 4 (content -- parallel):
  Phase 5: Color contrast        ]
  Phase 8: Form labels           ]
  Phase 9: Error announcements   ] Content analysis
  Phase 10: Alt text             ] agents
  Phase 13: Touch targets        ]

WAVE 5 (experience -- parallel):
  Phase 14: Motion preferences   ]
  Phase 15: Cognitive load       ] Experience
  Phase 16: Reading level        ] agents

4 parallel agents per wave = 16 phases in 5 sequential waves.
```

---

## CROSS-COMMAND BRIDGE

```
/a11yaudit finds UI issues -> references /uiuxaudit findings
/a11yaudit finds code issues -> references /codeaudit findings
/a11yaudit finds form issues -> references /flowaudit findings
/a11yaudit finds copy issues -> references /copyaudit findings

THE QUALITY ARSENAL:
  /codeaudit    -> Is the code SOLID?           (preventive)
  /flowaudit    -> Does the experience WORK?    (preventive)
  /uiuxaudit    -> Is the interface BEAUTIFUL?  (preventive)
  /debugaudit   -> What is BROKEN right now?    (detective)
  /featureaudit -> Is the product COMPLETE?     (strategic)
  /perfaudit    -> Is it FAST enough?           (detective)
  /secaudit     -> Is it SECURE?                (detective)
  /a11yaudit    -> Can EVERYONE use it?         (inclusive)
  /seoaudit     -> Can search engines FIND it?  (growth)
  /dataaudit    -> Is the data INTACT?          (integrity)
  /apiaudit     -> Is the API RELIABLE?         (contract)
  /copyaudit    -> Does the copy DELIVER?       (messaging)
  /dxaudit      -> Can devs SHIP efficiently?   (velocity)

  Together: nothing escapes. Every dimension covered.
```

---

## LAWS

1. **Accessibility is not a feature — it's a right.** Every barrier you leave is a user you exclude.
2. **Keyboard first, always.** If it doesn't work with a keyboard, it doesn't work.
3. **Semantic HTML before ARIA.** The first rule of ARIA is don't use ARIA when native HTML works.
4. **Automate the minimum, verify the rest.** axe-core catches 30%. You catch the other 70%.
5. **Test with real tools.** Screen readers, keyboard-only, zoom, high contrast — not just code review.
6. **Contrast is non-negotiable.** 4.5:1 for text. 3:1 for UI. No exceptions for "aesthetic."
7. **Every user deserves the full experience (Popper).** "Most users don't need this" is the accessibility version of "works on my machine."

---

*"/a11yaudit v1 — Perceive. Operate. Understand. Every user, every page, every interaction. /320."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.1 — 2026-04-14T10:35:36Z)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial thinking
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.a11yaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — Non-UI contexts: for CLI tools, check for screen-reader-friendly terminal output (no unicode-only bar charts, no color-only status). ABORT for pure libraries.
- ✅ **Output contract verification** — emits `audits/.a11yaudit/verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `iterations.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `audits/.a11yaudit/discovery/` > 1h old, re-verify inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.a11yaudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` flags with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — Screen-Reader Automation + i18n Coverage

**Screen-reader verification (without real AT):**
- axe-core catches ~40% of WCAG issues (static analysis)
- Playwright-based ARIA announcer simulation: navigate page, capture `aria-live` updates, verify announcements fire
- Emit `needs_manual_at_verification: [...]` in verdict.json for items automation cannot cover (e.g. screen-reader pronunciation quality)

**i18n / locale coverage:**
- RTL layout check: add `dir="rtl"` to root, verify layout integrity (mirrored properly)
- Locale-switched rendering: if `next-intl` / `i18next` / similar present, test rendering in 2 locales (en + one other from project locale list)
- Flag hardcoded English strings (missing i18n wrapping) as findings

### Scoring addenda

Normalized score = (raw_score / applicable_raw_max) × 100.
Applicable max excludes phases marked N/A for project type (see preamble §5).
`verdict.json.preamble_version = "1.0"` is MANDATORY. Missing field = /metaudit flags as non-compliant.

### /metaudit Compliance Badge

Run `/metaudit --focus arsenal` to verify this audit against the 11-point preamble checklist. Target score: 11/11.

---

*v1.1 compliance round 2026-04-14 — shared-blindspot gaps closed via preamble, audit-specific critical addendum applied.*

---

## § 100/100 COMPLIANCE CERTIFICATE — v1.2 — 2026-04-14T10:43:29Z

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (a11yaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "a11yaudit",
  "preamble_version": "1.0",
  "compliance_version": "1.2",
  "compliance_score": 100,
  "shared_blindspots_closed": {
    "S1_concurrency_lock": true,
    "S2_reaudit_cap_5_iter": true,
    "S3_non_ui_context_handling": true,
    "S4_telegram_progress": true,
    "S5_scoped_invocation_flags": true,
    "S6_output_contract_verification": true,
    "S7_self_telemetry": true,
    "S8_phase_count_rationale_documented": true,
    "S9_discovery_drift_check": true,
    "S10_deprecation_registry_reference": true
  },
  "audit_specific_gaps_closed": true,
  "rule_46_compliant": true,
  "integration_smoke_test_gate": true,
  "rate_limit_safety": true,
  "last_updated": "2026-04-14T10:43:29Z"
}
```

### Gap closures (pre-100 findings → now resolved)

**Gap 1: Screen-reader automation → RESOLVED**
```
Phase 4 (screen-reader): Playwright-driven ARIA announcer simulation:
  Navigate to page authenticated (preamble §5)
  Capture:
    - aria-live region updates (inspect mutations)
    - aria-label / aria-labelledby / aria-describedby on interactive elements
    - role attributes
    - Focus order via tab key sequence (max 200 tabs)
    - Focus visibility (outline on :focus-visible)
  Flag:
    - Interactive element without accessible name
    - aria-live without polite/assertive attribute
    - Focus trap absent on modals
  Items automation cannot cover emit as:
    needs_manual_at_verification: [
      {type: "screen_reader_pronunciation", items: [...]}
      {type: "screen_reader_landmark_nav", items: [...]}
    ]
```

**Gap 2: i18n coverage → RESOLVED**
```
RTL layout check:
  Inject <body dir="rtl"> (Playwright evaluate)
  Screenshot, compare to LTR for structural symmetry
  Flag: text overflow, mirror failures, icon positioning issues

Locale switch:
  If project has next-intl / i18next / react-intl / lingui:
    Detect locale list from config
    Render page in each locale, check for:
      - Hardcoded English strings (grep for hardcoded str patterns in JSX)
      - Date/number formatting consistency
      - Plural rules (check _plural variants exist for dynamic counts)
```

**Gap 3: Hardcoded string detection regex → RESOLVED**
```
Pattern: />\s*[A-Z][a-z]{3,}/ inside *.tsx|*.jsx (text node content) 
Exclusions: 
  - Tailwind classes
  - Already wrapped in t() / <Trans> / _() 
  - inside <pre>/<code> 
  - Length < 5 chars (single words like "OK", "Go" often legitimate)
Flag match as LOW severity unless i18n infra is present (then MEDIUM).
```

**Gap 4: Non-UI contexts → RESOLVED via preamble §5:**
CLI tools: check for screen-reader-friendly terminal output (no unicode bar charts, no color-only status). Primary check = --help output clarity + exit codes.

### Final score calculation

```
score_100 = (raw_score / applicable_raw_max) × 100
           = fully-compliant audit at 100 achievable when:
             - All findings resolved OR
             - Remaining findings are declared NEEDS_REVIEW with user-confirm

After v1.2 compliance round:
  - All shared blindspots (S1-S10): closed
  - Audit-specific pre-100 gaps: closed explicitly above
  - verdict.json.preamble_version = "1.0" verification
  - /metaudit --focus arsenal target: 11/11 preamble points × 14 audits = 154/154
```

### Verification command

```bash
# Run /metaudit to verify this audit's 100/100 claim:
/metaudit --focus arsenal --scope="a11yaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*a11yaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

---

## MANDATORY BEFORE/AFTER VERIFICATION (v1.1)

**Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix execution.**

Every fix MUST follow the "Do No Harm" protocol:

1. **PRE-FIX BASELINE** — grep all references, capture functional state, save to `audits/.<audit-id>/baseline/`.
2. **APPLY FIX** — normal execution.
3. **POST-FIX CHECK** — repeat every baseline check. If any PASSED→FAILED transition occurs, revert immediately.
4. **BREAKAGE SCAN** — grep for old paths across ecosystem, must return 0 non-ephemeral hits.
5. **BEFORE/AFTER MATRIX** — produce `audits/.<audit-id>/before-after.md` with functional status table per affected item.

**An audit that breaks 1 working thing is WORSE than no audit.** Do NOT claim "done" without `before-after.md` showing zero regressions.

---

## Dynamic-Workflow Orchestration (v2)

> *"A barrier nobody can reproduce is a rumor, not a finding. Fan out to find them all; falsify each through ≥2 independent eyes before it earns a place in the verdict."*
>
> **APPEND-ONLY layer — changes HOW this audit runs, never WHAT it scores.** Every phase
> above (0–16), the HINGE-FLOW Gestalt gate, the Popper doctrine, the /320 scoring matrix
> (Phase 17), the grade bands, the fix/re-audit loop, and the verdict schema remain
> **verbatim and authoritative**. This section is pure execution strategy: it replaces the
> default top-to-bottom phase walk with a concurrent fan-out + adversarial-verify + synthesize
> pipeline, then funnels every *surviving* finding straight back into the unchanged Phase 17
> matrix. When this section and any earlier phase appear to disagree, **the earlier phase wins**.

### A. Decompose into INDEPENDENT parallel tracks (fan-out via the Workflow tool)

The 16 domain phases are **file/surface-disjoint by accessibility concern**, so run them
concurrently with the `Workflow` tool (in-process fan-out per **R-ORCH**) instead of linearly.
This is the dynamic upgrade of the static WAVE grouping documented above — same phase content,
now driven by a Workflow that dispatches each track in parallel and self-paces discovery.

Phase 0 (Reconnaissance + PHASE 0 programmatic gather) stays **strictly sequential and first** —
every track consumes its `pages.json`, `interactive-elements.json`, `aria-inventory.json`,
`heading-map.json`, and `.a11y/evidence-summary.json`. Then fan out (one Workflow track each):

| Track | Phases (unchanged) | Independent surface | Primary lenses (§B) |
|-------|--------------------|---------------------|---------------------|
| **T1 — Semantic structure** (static/code) | 1 WCAG, 4 ARIA, 11 Headings, 12 Landmarks | DOM semantics, role/name/value, hierarchy, regions | reproduce + cross-check |
| **T2 — Keyboard & focus (HINGE, 2× weight)** | 2 Keyboard, 6 Focus, 7 Skip-nav | Tab order, focus traps, visible focus, skip links | reproduce + refute |
| **T3 — Assistive-tech runtime** | 3 Screen reader, 9 Error announcements | aria-live, announced name/role/state, error/SR semantics | reproduce + cross-check |
| **T4 — Perceptual / visual** | 5 Color contrast, 13 Touch targets, 14 Motion | ratios, 44px hit-areas, prefers-reduced-motion | reproduce + cross-check |
| **T5 — Forms & content** | 8 Form labels, 10 Alt text, 15 Cognitive load, 16 Reading level | label↔input binding, alt/SVG, plain-language, FK grade | reproduce + refute |

Concurrency rules (inherit **R-SCOPE** one-writer-per-file): tracks **read freely in parallel**
but each owns a **disjoint slice of the discovery surface and its own** `reports/<phase>.md`
file — no two tracks write the same report. The HINGE track **T2** launches first and gets 10×
scrutiny per the Gestalt gate; it may run a dedicated keyboard-only Playwright probe (real Tab
sequence, focus-trap escape, Escape-closes-modal) that other tracks reference but never re-run.
Budget per **R-BUDGET** (default 500K); approaching the cap → escalate, never silently truncate
a track. Tracks emit partial findings as they complete — do not block synthesis on the slowest.

### B. ADVERSARIALLY VERIFY each finding through ≥2-of-3 independent lenses (kill the survivors-only)

No raw track output enters the verdict. Every candidate finding (from `evidence-summary.json`
**and** from LLM track analysis) faces **three independent lenses; a finding survives only if
≥2 agree** (this operationalizes Phase H1 §2.1 Popper falsification + **R-VERIFY** consensus —
it does not replace it). The three lenses, tailored to accessibility:

1. **REPRODUCE** — independently re-trigger the barrier on the live surface with a *different
   method* than the one that first reported it. Mouse-free Tab-through for keyboard claims;
   Playwright ARIA-snapshot / accessible-name computation for SR claims; manual WCAG ratio math
   (relative luminance, not the tool's verdict) for contrast claims; rendered DOM measurement
   (`getBoundingClientRect`) for touch-target claims. Reproduced → +1.
2. **REFUTE** — actively try to PROVE the finding wrong (the Popper move). Is the "div with
   onClick" actually keyboard-operable via a delegated handler or `tabindex`+`keydown`? Is the
   "unlabeled input" labeled via `aria-labelledby`/`aria-label`/wrapping `<label>` the static
   scan missed? Is contrast 4.4 (real fail) or rounded 4.5 (pass)? Does a `prefers-reduced-motion`
   block already exist? Survives refutation (couldn't be disproven) → +1; counter-example found
   → **−1 / kill**.
3. **CROSS-CHECK** — confirm through a *second signal class*: a sibling tool in the gather
   (axe-core ∧ pa11y ∧ Lighthouse agreeing → `cross_tool_confirmed`), a sibling audit's
   `evidence-summary.json` per Phase 0.5 / H1 §2.5 (`/uiuxaudit` contrast, `/copyaudit` i18n
   strings, `/flowaudit` form journey → `cross_audit_confirmed`, bump severity one level), or a
   second independent WCAG success-criterion mapping to the same defect. Confirmed → +1.

Decision: **score ≥ 2 → finding STANDS** (record the 2–3 lens outcomes as `falsifiable_tests[]`
entries in `verdict.json`, exactly the existing H1 §2.1 schema). **Score < 2 → KILLED**: demote
to `info` with `falsified_at: <evidence>`, exclude from scoring. Exactly one lens (e.g. only the
originating tool) is **not enough** — that is the "automated tools lie / works-for-me" trap this
audit's DOCTRINE Laws #2 and #5 exist to catch. **HINGE-track (T2) findings** require all three
lenses run (per the 10× rule), not just two. Reaching a finding only from a single sighted code
read with no keyboard/SR/contrast reproduction = automatic kill.

### C. Synthesize survivors back into THIS audit's EXISTING matrix + verdict (UNCHANGED)

Surviving findings (≥2-of-3) are the **only** inputs to the existing pipeline — nothing about
scoring changes:

1. Route each survivor to its owning phase and score that phase **0–10 exactly as Phase 17
   defines**, then apply the **unchanged weights**: WCAG ×3.0, **Keyboard ×4.0 (HINGE)**, Screen
   Reader ×3.0, ARIA ×2.5, Contrast ×2.5, Focus ×2.5, Skip-nav ×1.5, Form Labels ×2.0, Errors
   ×2.0, Alt ×1.5, Headings ×1.5, Landmarks ×1.5, Touch ×1.5, Motion ×1.0, Cognitive ×1.0,
   Reading ×1.0 → **max 320**, then `normalized = (raw / 320) × 100`. **Do not re-weight, add,
   merge, or drop a phase.**
2. Apply the grade bands (S/A/B/C/D/F) and the Phase 2.7 / score-gating thresholds verbatim
   (incl. `user_need_match.addressed`, ≥3 falsifiable tests/phase, edge cases, cross-audit array).
3. Emit the **same** `verdict.json` (hybrid v2 schema, §2.6) + `verdict.md`. Add only
   provenance fields on each finding so the orchestration is auditable per **R-CITE**:
   `verified_by_lenses: ["reproduce","cross-check"]`, `lens_score: 2`, `killed_in_verification:
   <n>` summary. These annotate; they never alter the numeric score.
4. Feed survivors into the **unchanged** Phase 18 fix-plan → Phase 19 fix execution (with its
   "Do No Harm" safety gate) → Phase 20 re-audit loop (bounded at 5 iterations). Killed
   candidates never reach the fix plan, eliminating wasted fixes on phantom barriers.

### D. Loop-until-dry for unknown-size discovery (bounded)

Accessibility surface area is rarely known up front (dynamic routes, modals/dialogs/menus that
only mount on interaction, infinite-scroll regions, locale-switched + RTL renders, auth-gated
pages). Wrap discovery + fan-out in a **loop-until-dry** controller (a Workflow loop per
**R-GOAL**: the loop lives *inside* this workflow, never wraps it):

```
discovered = Phase-0 inventory (pages, interactive elements, live regions, locales)
repeat:
  fan out tracks T1–T5 over the NEW surface only (don't re-scan settled surface)
  adversarially verify (§B) every candidate → keep survivors
  expand discovery: routes/dialogs/menus newly revealed by interaction,
                    each locale + dir="rtl", lazy/feature-flagged components
until ( no NEW interactive surface AND no NEW survivors )  # "dry"
  OR iteration == 5                                        # hard cap (preamble §4)
  OR budget threshold reached                              # R-BUDGET → escalate
on cap-without-dry: mark residual surface NEEDS_REVIEW in verdict.json, Telegram SOS,
                    surface `pending` in .done.json — never loop silently, never fake "dry".
```

Each iteration appends to `iterations.md` (score trajectory + newly-discovered-surface count),
exactly as the existing re-audit cap demands. "Dry" means a full pass discovered **zero** new
interactive surface and produced **zero** new surviving findings — only then is discovery
complete and the verdict final.

> **Net effect:** same phases, same HINGE-FLOW Gestalt gate, same Popper doctrine, same /320
> matrix and verdict — executed concurrently, with every barrier proven through ≥2 independent
> lenses before it can cost points, and discovery that runs until the accessibility surface is
> genuinely exhausted. Identity preserved; rigor and throughput increased.
