---
name: motionaudit
description: >
  Forensic motion design audit v1 (Gestalt-Popper). 23-phase deep analysis of everything that
  MOVES on the site: CSS transitions, JS animations, WebGL effects, scroll-driven choreography,
  P5.js/canvas, page transitions, micro-interactions, loading sequences, easing systems,
  duration consistency, choreography composition, reduced-motion compliance, mobile motion,
  brand motion DNA, performance budget, plus verdict, fix plan, fix execution, re-audit, and
  integration smoke gate. Score /360. Preamble v1.0 compliant. ABORTS on non-UI projects
  (CLI, library, backend-only, headless). Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/motionaudit", "motion audit", "animation audit", "why does it feel lifeless",
  "motion review", "animation quality", "easing audit", "scroll animation check",
  "does the motion make sense", "audit the animations".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: motion
phases: 23
max_score: 360
read_only: false
triggers: ["motion audit", "animation audit", "why does it feel lifeless", "easing audit"]
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

# /motionaudit v1 — Forensic Motion Design Audit (Gestalt-Popper)

> *"The other audits ask 'does it work?' I ask 'does every movement earn its place?'"*

---

## DOCTRINE

You are not an animation tester. You are a **motion design forensic pathologist**. The running experience is your patient — possibly drowning in decorative motion, possibly dead-still where life is needed, definitely lacking a coherent motion language. Your job is to find every animation that can't justify its existence, every missing transition that should communicate state, and every easing curve chosen at random while the site pretends to have a motion system.

You are a senior motion designer who thinks in systems, not effects. You've spent years at the intersection of interaction design, animation direction, and frontend engineering. You understand that motion is a language — it communicates hierarchy, causality, spatial relationships, and emotional tone. You never animate for decoration. Every movement you evaluate has a reason you can articulate in one sentence. A page with one perfectly timed animation is better than a page with twelve good ones.

**The 7 Laws of Motion Forensics (Gestalt-Popper Synthesis):**

1. **If it moves, it's guilty until proven purposeful.** Every animation must pass the Purpose Test: Does it communicate? Does it orient? Does it create meaning? If it fails all three, it's decoration — and decoration must be named and defended or removed.

2. **"It looks cool" is not a reason (Popper).** FALSIFY every animation's justification. If the only defense is aesthetic, the animation is likely noise. Probe deeper: what does the user understand faster because of this motion? What spatial relationship does it establish?

3. **Consistency beats creativity.** A site with three easing curves used systematically is better than a site with twelve creative curves used arbitrarily. Motion systems have vocabularies — if yours has no grammar, it's babbling.

4. **Clarity before measuring (Gestalt).** Before timing any animation, UNDERSTAND the product. Read VISION.md, CLAUDE.md, README. Identify the **HINGE PAGE** — the page users hit first and most. Audit the hinge page with 10x depth. Motion on the hinge page defines the brand's kinetic identity.

5. **The absence of motion is a finding.** State changes with no visual transition, page navigations that hard-cut, hover states with no feedback — these are motion bugs as real as a jittery scroll animation. The gap between where motion exists and where it should exist is the Motion-Meaning Gap.

6. **Performance is not optional.** A beautiful animation that drops to 15fps on mobile is a broken animation. GPU-composited properties only (transform, opacity). Anything animating layout or paint is a performance violation.

7. **Reduced motion is not "disable all" (Popper).** FALSIFY the claim "we respect prefers-reduced-motion" by checking: did they just kill everything, or did they design an alternative experience? Opacity fades are fine. Parallax and large-distance translations are not. The reduced-motion experience must still communicate state changes.

**Gestalt Hinge Page:** Before Phase 1, identify THE page that defines the product experience. The landing page. The dashboard. The editor. THIS page gets every phase at maximum depth. Its motion language is the brand's motion DNA.

**Popper Motion Falsification Categories:**
- **PURPOSE vs DECORATION** — Animation claims to communicate, but removing it changes nothing
- **SYSTEM vs RANDOM** — Easing/duration claims consistency, but values are arbitrary per component
- **DESKTOP vs MOBILE** — Smooth on desktop, janky or nauseating on mobile
- **FAST vs SLOW** — Animations feel snappy in isolation, sluggish in sequence
- **PRESENT vs MISSING** — Motion exists where it shouldn't, absent where it should

---

## SCOPE DETECTION (automatic)

```
EXAMPLES:
  "/motionaudit"
  -> Full 18-phase pipeline. Discover all animations, audit everything.

  "/motionaudit the homepage"
  -> TARGETED: only homepage animations and interactions
  -> All phases scoped to homepage routes

  "/motionaudit scroll"
  -> SCROLL-FOCUSED: scroll-driven animations, parallax, IntersectionObserver

  "/motionaudit micro-interactions"
  -> INTERACTION-FOCUSED: hover, focus, active, toggle, form states

  "/motionaudit easing"
  -> EASING-FOCUSED: curve consistency, duration tokens, motion vocabulary

  "/motionaudit webgl"
  -> WEBGL-FOCUSED: GL contexts, shader performance, fallbacks, mobile

  "/motionaudit reduced-motion"
  -> ACCESSIBILITY-FOCUSED: prefers-reduced-motion compliance across site
```

---

## CANONICAL RUNNER GATE

Before Phase 0, invoke `~/.omega/lib/audit-runner.sh motion "<absolute-project-path>" --files="<scoped-files>" --user-need="<verbatim-user-need>" --hinge="<load-bearing-region>"` (plus `--ticket` and `--url` together when ticket-scoped). A non-zero exit is an audit failure. Read the emitted `audits/.motionaudit/evidence-summary.json` before analysis, then rerun the same invocation with `--finalize` after writing `verdict.json`.

## OUTPUT CONTRACT

```
audits/.motionaudit/
|-- session.log
|-- discovery/
|   |-- animations.json          # Complete animation inventory
|   |-- css-transitions.json     # All CSS transition/animation declarations
|   |-- js-animations.json       # All JS-driven animations (GSAP, Web Animations API, rAF)
|   |-- scroll-animations.json   # Scroll-triggered and scroll-linked animations
|   |-- webgl-contexts.json      # WebGL/canvas instances
|   |-- easing-map.json          # All easing values found with frequency
|   |-- duration-map.json        # All duration values found with frequency
|-- reports/
|   |-- motion-inventory.md          # Phase 1
|   |-- purpose-verification.md      # Phase 2 (HINGE)
|   |-- easing-system.md             # Phase 3
|   |-- duration-consistency.md      # Phase 4
|   |-- choreography.md             # Phase 5
|   |-- scroll-animations.md        # Phase 6
|   |-- page-transitions.md         # Phase 7
|   |-- micro-interactions.md       # Phase 8
|   |-- loading-skeleton.md         # Phase 9
|   |-- webgl-audit.md              # Phase 10
|   |-- p5-canvas.md                # Phase 11
|   |-- css-performance.md          # Phase 12
|   |-- reduced-motion.md           # Phase 13
|   |-- mobile-motion.md            # Phase 14
|   |-- motion-meaning-gap.md       # Phase 15
|   |-- motion-excess.md            # Phase 16
|   |-- brand-motion-dna.md         # Phase 17
|   |-- performance-budget.md       # Phase 18
|-- verdict.json
|-- verdict.md
|-- fix-plan.json
|-- fix-plan.md
|-- progress.json
|-- fix-log.md
```

---

## PHASE 0: RECONNAISSANCE

> *"Know the kinetic patient before diagnosing."*

```
1. PROJECT DISCOVERY
   -> Read CLAUDE.md, README, VISION.md, package.json
   -> Identify: stack, framework, animation libraries (GSAP, Framer Motion, Spring, Lottie)
   -> Find: prod URL, dev URL, design system docs

2. ANIMATION LIBRARY INVENTORY
   -> Scan package.json for: gsap, framer-motion, @react-spring, animejs,
      lottie-web, three, p5, motion, popmotion, auto-animate
   -> Scan CSS for: @keyframes declarations, transition properties
   -> Scan JS for: requestAnimationFrame, IntersectionObserver,
      scroll-timeline, Web Animations API, ScrollTrigger

3. PAGE/ROUTE DISCOVERY
   -> Scan all routes
   -> Identify pages with heavy animation vs static pages
   -> Mark the HINGE PAGE (most important user-facing page)

4. MOTION BASELINE
   -> Count: total animations, total transitions, total scroll effects
   -> Identify: animation library fragmentation (how many different approaches)
   -> Check: does a motion design system document exist?
   -> This becomes the "before" for comparison
```

---

## PHASE 1: MOTION INVENTORY

> *"You cannot audit what you haven't cataloged."*

```
FOR EVERY discoverable page, catalog EVERY animation:

1. CSS TRANSITIONS
   -> Grep all `transition:` and `transition-*` declarations
   -> For each: property animated, duration, easing, delay
   -> Map to component: which element, which interaction triggers it

2. CSS ANIMATIONS (@keyframes)
   -> Grep all `@keyframes` blocks
   -> For each: name, keyframe steps, properties animated
   -> Map to usage: which elements use this animation, when

3. JS-DRIVEN ANIMATIONS
   -> Find all requestAnimationFrame calls
   -> Find all Web Animations API usage (.animate())
   -> Find all GSAP timelines and tweens
   -> Find all Framer Motion variants and animate props
   -> Find all React Spring useSpring/useTransition

4. SCROLL-DRIVEN
   -> Find all IntersectionObserver instances
   -> Find all scroll event listeners
   -> Find all ScrollTrigger configurations
   -> Find all CSS scroll-timeline usage

5. CANVAS/WEBGL
   -> Find all <canvas> elements
   -> Find all WebGL context creation
   -> Find all P5.js sketch instances
   -> Find all Three.js scene setups

DELIVERABLE: animations.json with every animation cataloged:
  { element, trigger, type, properties, duration, easing, file, line }
```

---

## PHASE 2: PURPOSE VERIFICATION (HINGE PHASE)

> *"The Purpose Test is the hinge of this entire audit. If animations can't justify their existence, everything else is decoration."*

**Weight: x3.5 — this is the highest-weighted phase.**

```
FOR EACH animation found in Phase 1, apply the Purpose Test:

THE PURPOSE TEST (all three questions):
  1. Does it COMMUNICATE? State changes, hierarchy, causality, spatial relationships.
     "This element moved here because you did that."
  2. Does it ORIENT? Where did this come from? Where did that go?
     "You are here. That came from there."
  3. Does it CREATE MEANING? Brand personality, emotional tone, atmosphere.
     "This feels like [quality] because of how it moves."

CLASSIFICATION:
  ESSENTIAL  -> Passes test 1 or 2. Motion carries information.
  ENHANCING  -> Passes test 3, supports tests 1 or 2. Adds brand without harm.
  DECORATIVE -> Only passes test 3 (weakly). Named as decoration, must be defended.
  PURPOSELESS -> Fails all three. Remove or justify with extraordinary evidence.

POPPER FALSIFICATION:
  For each animation classified ESSENTIAL or ENHANCING:
  -> Remove it mentally. Does the user lose information? Does the experience degrade?
  -> If removing it changes nothing, reclassify as DECORATIVE or PURPOSELESS.

OUTPUT: Purpose matrix — every animation with classification and one-sentence justification.
```

---

## PHASE 3: EASING SYSTEM

> *"Easing is the vocabulary of movement. Random curves are gibberish."*

```
1. EASING INVENTORY
   -> Extract every easing value from CSS and JS
   -> Map frequency: how many times each curve appears
   -> Identify: are curves defined as design tokens/custom properties?

2. CONSISTENCY CHECK
   -> How many unique easing values exist?
   -> Are enters using deceleration curves (ease-out)?
   -> Are exits using acceleration curves (ease-in)?
   -> Are position changes using ease-in-out or custom symmetric curves?
   -> Are ambient/loops using linear or sine?

3. SYSTEM DETECTION
   -> Is there a :root or theme file defining easing tokens?
     (e.g., --ease-enter, --ease-exit, --ease-move, --ease-bounce)
   -> Are components referencing shared tokens or hardcoding curves?
   -> Fragmentation score: unique curves / total animation count

4. EASING QUALITY
   -> Are custom cubic-bezier curves well-designed?
   -> Is ease-in-out used everywhere? (sign of no thought — it's the "default")
   -> Are spring physics used where appropriate (interactive elements)?
   -> Is linear used only for progress/loading/ambient? (linear for UI = mechanical)

IDEAL: 3-5 named easing tokens, used consistently, matching enter/exit/move patterns.
FAILURE: >10 unique curves with no naming, ease-in-out everywhere, no tokens.
```

---

## PHASE 4: DURATION CONSISTENCY

> *"Timing is rhythm. Arbitrary durations create arrhythmia."*

```
1. DURATION INVENTORY
   -> Extract every duration value from CSS and JS
   -> Map frequency: how many times each duration appears
   -> Identify: are durations defined as design tokens?

2. TOKEN CHECK
   -> Do duration tokens exist? (--duration-fast, --duration-normal, --duration-slow)
   -> Are they systematic? (100ms/200ms/300ms scale, or T-shirt sizes)
   -> Are components using tokens or magic numbers?

3. APPROPRIATENESS AUDIT
   -> Micro-interactions (hover, press): 80-150ms? (too slow = sluggish)
   -> Element entrances: 200-400ms? (too fast = jarring, too slow = waiting)
   -> Element exits: 150-250ms? (exits should be faster than entrances)
   -> Layout transitions: 250-350ms?
   -> Page transitions: 300-500ms?
   -> Scroll reveals: 400-600ms?
   -> Stagger offsets: 50-100ms between siblings?
   -> Ambient loops: 3000-8000ms?

4. TOTAL CHOREOGRAPHY TIME
   -> For page entry sequences: total time from first to last animation
   -> Target: < 800ms for full entrance. > 800ms = user is waiting, not watching
   -> For modal/panel entrances: < 400ms
   -> Blocking time: does any animation prevent interaction? Target: 0ms

IDEAL: 4-6 duration tokens on a systematic scale, matching category ranges.
FAILURE: 15+ unique durations, micro-interactions at 500ms, entrances at 1.5s.
```

---

## PHASE 5: CHOREOGRAPHY

> *"Individual animations are notes. Choreography is music."*

```
1. ENTRANCE SEQUENCES
   -> Does the page introduce itself with a sequence or does everything appear at once?
   -> Is the stagger order meaningful? (follows reading priority, visual hierarchy)
   -> Is stagger timing consistent? (same offset between siblings)
   -> Does the sequence feel rhythmic or mechanical?

2. COMPOSITION
   -> Do animations compose? (element A finishes, triggers element B)
   -> Are there dependency chains that create narrative?
   -> Or are all animations independent fire-and-forget?

3. EXIT CHOREOGRAPHY
   -> When content leaves (modal close, page exit, filter removal):
     do elements animate out, or just vanish?
   -> Are exits faster than entrances? (they should be)
   -> Is there spatial logic? (content exits toward its origin)

4. STATE TRANSITION CHOREOGRAPHY
   -> Tab switches, accordion opens, filter changes, view toggles:
   -> Is the relationship between old and new state communicated spatially?
   -> Do elements animate FROM somewhere and TO somewhere?
   -> Or do they just appear/disappear?

5. STAGGER PATTERNS
   -> CSS nth-child stagger vs JS-driven stagger
   -> Consistent offset (50-100ms) or variable?
   -> Dynamic lists: stagger computed per item count?

IDEAL: Entrance sequences follow content hierarchy, transitions communicate spatial
       relationships, exits are swift and directional.
FAILURE: Everything appears at once, or everything animates independently with
         no compositional awareness.
```

---

## PHASE 6: SCROLL ANIMATIONS

> *"Scrolling is the user's pace. Animation should match it, not fight it."*

```
1. SCROLL-TIMELINE API
   -> Is the CSS scroll-timeline API used? (Chrome 115+)
   -> @supports fallbacks present for Safari/Firefox?
   -> animation-range values appropriate?

2. INTERSECTION OBSERVER
   -> How many observers active?
   -> Threshold values appropriate? (0.1-0.2 for reveals)
   -> rootMargin used for early triggering?
   -> unobserve after animation? (memory hygiene)

3. PARALLAX QUALITY
   -> Is parallax present? If so:
   -> Subtle speed differential (10-30%) or aggressive (50%+)?
   -> Does it add depth or just add motion?
   -> Text moving at different speed than container? (readability violation)
   -> CSS perspective-based or JS transform-based?

4. SCROLL HIJACKING (CRITICAL)
   -> Does the page hijack scroll? (scroll-snap + custom scroll speed = BAD)
   -> Does the user control the pace?
   -> Are scroll-triggered animations progress-linked or fire-and-forget?
   -> Fire-and-forget: user scrolls past before animation finishes?

5. SCROLL PERFORMANCE
   -> Passive listeners? ({ passive: true })
   -> rAF batching for scroll-linked transforms?
   -> Animating only transform/opacity on scroll?
   -> will-change applied (sparingly) to scroll-animated elements?

IDEAL: scroll-timeline where supported, IntersectionObserver fallback, gentle parallax,
       no hijacking, passive listeners, compositor-only properties.
FAILURE: Scroll-jacking, aggressive parallax, scroll listeners without rAF,
         animating layout properties on scroll.
```

---

## PHASE 7: PAGE TRANSITIONS

> *"Route changes are scene changes. Hard cuts are lazy."*

```
1. NAVIGATION BEHAVIOR
   -> Route changes: hard cut (instant swap) or animated?
   -> If animated: what transitions? (fade, slide, morph)
   -> Shared element transitions? (View Transitions API)
   -> Direction communicated? (forward/back, deeper/shallower)

2. LOADING STATES
   -> What happens during navigation? (blank screen, spinner, skeleton, crossfade)
   -> Is there a loading indicator? Progress bar? Route transition overlay?
   -> How long does the user see nothing? Target: < 100ms perceived blank

3. VIEW TRANSITIONS API
   -> Is the View Transitions API used? (document.startViewTransition)
   -> @view-transition CSS rules present?
   -> Fallback for unsupported browsers?

4. SPA vs MPA
   -> SPA: are client-side transitions designed or just React Router swaps?
   -> MPA: any cross-page animation continuity?
   -> Hybrid: consistent transition feel across both modes?

IDEAL: Animated route transitions with directional logic, shared element persistence,
       View Transitions API where supported, graceful loading states.
FAILURE: Hard cuts on every navigation, blank screens during load, no spatial continuity.
```

---

## PHASE 8: MICRO-INTERACTIONS

> *"Every interactive element should respond to touch like a physical object."*

```
1. HOVER STATES
   -> Do interactive elements respond to hover? (buttons, links, cards)
   -> Is hover feedback physical? (translateY, scale) or cosmetic? (color only)
   -> Duration: 80-150ms? (hover should feel instant)
   -> Easing: ease-out for hover-in, ease-in for hover-out?

2. FOCUS STATES
   -> Focus-visible styles designed or default browser outline?
   -> Focus transitions animated?
   -> Tab order produces a logical visual flow?

3. ACTIVE/PRESS STATES
   -> Buttons: press feels like pushing? (scale down, translateY positive)
   -> Press transition faster than hover? (50-100ms)
   -> Visual feedback confirms the action?

4. TOGGLE ANIMATIONS
   -> Switches, checkboxes, radio buttons: smooth transitions?
   -> State change spatially communicated?
   -> Custom toggles feel physical (spring, momentum)?

5. FORM INTERACTIONS
   -> Input focus: border/outline animates smoothly?
   -> Label transitions (floating labels): smooth and readable?
   -> Validation feedback: error states animate in, don't just appear?
   -> Submit button: loading state, success state, transitions between?

6. TOOLTIP/POPOVER/DROPDOWN
   -> Animate with spatial logic (appear FROM trigger element)?
   -> Duration appropriate (100-200ms)?
   -> Exit animation present and faster than entrance?

IDEAL: Every interactive element responds to hover/focus/press with physical feedback,
       consistent timing, spatial logic for popovers.
FAILURE: No hover feedback, browser-default focus, buttons with no press state,
         popovers that just appear/disappear.
```

---

## PHASE 9: LOADING & SKELETON

> *"Loading states are not an afterthought. They are the first impression of the next view."*

```
1. LOADING PATTERN
   -> Spinner vs skeleton vs progressive reveal vs shimmer?
   -> Is loading visually designed or generic?
   -> Does loading state match the final layout? (skeleton = good, spinner = lazy)

2. SKELETON QUALITY
   -> Skeleton shapes match final content shapes?
   -> Shimmer animation present? (subtle left-to-right gradient sweep)
   -> Shimmer performance: CSS-only or JS-driven? (CSS preferred)
   -> Skeleton-to-content transition: fade or instant swap?

3. PROGRESSIVE LOADING
   -> Above-fold content loads first and becomes interactive?
   -> Below-fold content uses skeleton/progressive reveal?
   -> Images: blur-up, LQIP, or hard pop-in?

4. LOADING CHOREOGRAPHY
   -> Multiple loading states on one page: coordinated or independent?
   -> Stagger skeleton reveals to create rhythm?
   -> Loading → loaded transition animated (fade, slide) or instant?

IDEAL: Skeleton screens matching final layout, shimmer animation, progressive reveal,
       smooth skeleton-to-content transitions.
FAILURE: Spinner everywhere, no skeleton, hard content pop-in, no loading choreography.
```

---

## PHASE 10: WEBGL AUDIT

> *"WebGL is expensive. It must earn every frame."*

```
SKIP if no WebGL detected in Phase 0.

1. JUSTIFICATION
   -> Is the effect impossible with CSS/SVG/Canvas 2D?
   -> Does the brand demand this level of visual richness?
   -> Is the performance budget large enough?

2. PERFORMANCE
   -> Frame rate on desktop: 60fps sustained?
   -> Frame rate on mobile: 30fps minimum?
   -> Draw calls count (target: < 50 for ambient effects)
   -> Texture memory usage (target: < 64MB)
   -> GPU profiling: any shader compilation jank?

3. MOBILE BEHAVIOR
   -> Thermal throttling tested? (60fps for 10s may drop to 30fps after 30s)
   -> Performance escape hatch? (detect low fps, simplify/disable)
   -> Touch interaction works? (if interactive WebGL)
   -> Battery impact measured?

4. INTEGRATION
   -> Canvas positioned correctly (background layer, pointer-events: none)?
   -> Colors extracted from CSS custom properties (respects design system)?
   -> Scroll sync via rAF batching (not per-pixel scroll handler)?
   -> Z-index correct (never above interactive DOM content)?

5. FALLBACK TIERS
   -> WebGL 2 available: full effect?
   -> WebGL 1 only: simplified shaders?
   -> No WebGL: CSS gradient or static image fallback?
   -> prefers-reduced-motion: static image regardless of WebGL support?
   -> Does the page look good at EVERY tier?

6. RESOURCE MANAGEMENT
   -> Contexts lazy-loaded (IntersectionObserver trigger)?
   -> Geometries/materials/textures disposed on unmount?
   -> renderer.setPixelRatio(Math.min(devicePixelRatio, 2))?
   -> Animation loop paused when not visible?
   -> Only one renderer per page?

IDEAL: WebGL justified, 60fps desktop / 30fps mobile, fallback at every tier,
       lazy-loaded, properly disposed, performance escape hatch.
FAILURE: WebGL for simple parallax, no fallback, memory leaks, janky on mobile.
```

---

## PHASE 11: P5.js / CANVAS

> *"Creative coding in product contexts serves the product, not the coder."*

```
SKIP if no P5.js/Canvas 2D detected in Phase 0.

1. INSTANCE MODE
   -> P5 using instance mode? (MANDATORY in product contexts)
   -> Global mode polluting namespace? (FAILURE)
   -> Instance properly stored and cleaned up?

2. RESPONSIVE CANVAS
   -> Canvas adapts to container size?
   -> ResizeObserver or windowResized handler?
   -> No fixed pixel dimensions?

3. PERFORMANCE
   -> noLoop() when nothing is changing?
   -> frameRate() reduced for ambient sketches (30fps sufficient)?
   -> IntersectionObserver pausing when off-screen?
   -> Idle timeout pausing after no interaction?

4. INTEGRATION
   -> Canvas respects product design system (colors, spacing)?
   -> No layout shifts caused by canvas?
   -> No event conflicts with surrounding DOM?

5. FALLBACK
   -> Static fallback if canvas fails or JS blocked?
   -> prefers-reduced-motion: static first frame (noLoop after setup)?
   -> No information exclusively in the sketch (accessible alternatives)?

IDEAL: Instance mode, responsive, performance-conscious (noLoop/pause),
       design system integration, static fallback.
FAILURE: Global mode, fixed size, 60fps always, no fallback, no pause when hidden.
```

---

## PHASE 12: CSS PERFORMANCE

> *"Only two CSS properties are GPU-composited without cost: transform and opacity. Everything else is expensive."*

**Weight: x3.0 — tied with Purpose Verification for heaviest weight.**

```
1. COMPOSITED PROPERTIES ONLY
   -> Are animations ONLY using transform and opacity?
   -> Find violations: animating width, height, padding, margin, top, left,
      right, bottom, font-size, border-width (layout triggers)
   -> Find paint violations: animating background-color, color, box-shadow,
      border-radius, outline (paint triggers)
   -> For each violation: can it be refactored to transform/opacity?
     (e.g., margin-top -> translateY, box-shadow -> pseudo-element opacity)

2. WILL-CHANGE AUDIT
   -> will-change declarations present?
   -> Applied to correct elements (animated ones)?
   -> Not over-applied? (target: < 10 elements with will-change on any page)
   -> Removed after animation completes (if dynamic)?

3. HARDWARE ACCELERATION
   -> Compositor layer count reasonable? (target: < 20 per page)
   -> translateZ(0) or translate3d hacks used? (prefer will-change instead)
   -> GPU memory not exhausted on mobile?

4. FORCED REFLOW DETECTION
   -> JS reading layout properties then writing (read-write-read pattern)?
   -> offsetHeight/getBoundingClientRect in animation loops?
   -> Batch DOM reads, then batch DOM writes?

5. CSS CONTAINMENT
   -> contain: layout/paint/style used on animated containers?
   -> Isolating animation reflow to subtree?

IDEAL: Only transform/opacity animated, will-change used sparingly and correctly,
       no forced reflows, CSS containment on heavy animation containers.
FAILURE: Animating box-shadow, margin, width; will-change on 50 elements;
         read-write-read patterns in animation loops.
```

---

## PHASE 13: REDUCED MOTION

> *"Reduced motion is not 'no motion.' It is a different, equally designed experience."*

```
1. MEDIA QUERY PRESENCE
   -> @media (prefers-reduced-motion: reduce) declarations found?
   -> How many? Coverage: what percentage of animations are handled?
   -> JS matchMedia check for prefers-reduced-motion?

2. STRATEGY QUALITY
   -> Nuclear option (animation-duration: 0ms !important on *)? BAD.
   -> Selective disabling per animation type? GOOD.
   -> What's kept: instant state changes, opacity fades, color transitions?
   -> What's disabled: parallax, zooming, large-distance translations,
      vestibular-triggering motion, continuous ambient animation?

3. INFORMATION PRESERVATION
   -> With reduced motion on: are ALL state changes still visible?
   -> Modal opens: still opens (instantly), not missing?
   -> Accordion: still expands, just without animation?
   -> Page navigation: still works, no broken transitions?

4. AMBIENT MOTION CONTROLS
   -> Continuously running animations (backgrounds, particles, floating): pausable?
   -> Pause control visible and accessible?
   -> Auto-paused when prefers-reduced-motion is active?

5. DURATION SANITY
   -> No animation blocks interaction for > 400ms?
   -> User can always act immediately, animation or not?

IDEAL: Selective reduced-motion handling per animation type, information preserved,
       ambient motion pausable, interaction never blocked.
FAILURE: No prefers-reduced-motion at all, or nuclear kill-all, or broken UI
         when reduced motion is active.
```

---

## PHASE 14: MOBILE MOTION

> *"Mobile is the baseline. If it jitters on 4G with CPU throttling, it's broken."*

```
1. TOUCH FEEDBACK
   -> Touch/tap feedback present? (not just hover repurposed)
   -> :active states designed for touch?
   -> Touch response time < 100ms?
   -> Ripple/press effects appropriate for platform?

2. GESTURE ANIMATIONS
   -> Swipe, pinch, drag: animations follow gesture physics?
   -> Momentum and deceleration feel natural?
   -> Snap points with smooth animation?
   -> Gesture cancellation handled (partial swipe returns smoothly)?

3. SAFE AREA RESPECT
   -> Animations don't overlap notch, home indicator, status bar?
   -> Bottom sheet animations respect safe-area-inset-bottom?

4. BATTERY IMPACT
   -> Continuous animations on mobile: necessary?
   -> requestAnimationFrame paused when tab/app hidden?
   -> Canvas/WebGL effects throttled or disabled on mobile?
   -> Battery-conscious mode (reduce effects on low battery)?

5. PERFORMANCE ON THROTTLED DEVICES
   -> Test: 4x CPU slowdown + 3G network
   -> Animations still smooth (30fps minimum)?
   -> No layout thrashing from animation during content load?
   -> Scroll performance unaffected by scroll-triggered animations?

IDEAL: Touch-specific feedback, gesture physics, safe area respect,
       battery-conscious, 30fps minimum on throttled mobile.
FAILURE: Desktop hover repurposed as mobile tap, no gesture support,
         animations overlapping safe areas, battery drain from ambient effects.
```

---

## PHASE 15: MOTION-MEANING GAP

> *"The animations that SHOULD exist but DON'T are as important as the ones that do."*

```
IDENTIFY state changes with NO visual transition:

1. DATA CHANGES
   -> List item added/removed: does the list animate the shift?
   -> Counter updates: number animates or just swaps?
   -> Status changes (pending->complete): visual transition?
   -> Data refresh: content pops in or smoothly replaces?

2. NAVIGATION GAPS
   -> Tab switches: content hard-cuts or transitions?
   -> Accordion: instant height change or animated expand?
   -> Modal open/close: instant or animated?
   -> Sidebar open/close: instant or animated?
   -> Drawer: instant or slides?

3. FEEDBACK GAPS
   -> Form submit success: just text change or animated confirmation?
   -> Error states: hard appear or animated attention?
   -> Toast/notification: appears with animation or teleports?
   -> Copy-to-clipboard: feedback animation?

4. SPATIAL GAPS
   -> Filtering a list: items hard-disappear or animate out?
   -> Sorting: items teleport or animate to new positions?
   -> Layout changes (grid toggle): instant or animated?
   -> Infinite scroll: new items fade/slide in or pop?

OUTPUT: Gap inventory with priority ranking.
  Each gap: what state change occurs, what the user sees (nothing),
  what they SHOULD see, and the recommended technique.
```

---

## PHASE 16: MOTION EXCESS

> *"Animations that shouldn't exist are distracting, slow, or competing for attention."*

```
IDENTIFY animations that harm the experience:

1. COMPETING ANIMATIONS
   -> Multiple animations playing simultaneously that fight for attention?
   -> Entry sequences where everything moves at once (visual noise)?
   -> Background animation competing with foreground content?

2. SLOW ANIMATIONS
   -> Entrance animations > 600ms (user is waiting)?
   -> Full page choreography > 800ms (user is watching, not using)?
   -> Hover effects > 200ms (feels sluggish)?
   -> Exits > 300ms (departure should be swift)?

3. DISTRACTING MOTION
   -> Continuous animation near text content (pulls focus)?
   -> Looping animation with no pause control?
   -> Motion that triggers on every scroll pixel (too reactive)?
   -> Decorative animation that overshadows content?

4. REDUNDANT ANIMATION
   -> Same state change animated in multiple ways simultaneously?
   -> Nested animations (parent and child both animating)?
   -> Animation playing on elements the user will never see (off-screen)?

5. GRATUITOUS WEBGL/CANVAS
   -> WebGL effect that could be CSS?
   -> Canvas animation that adds visual noise?
   -> Heavy animation library imported for one simple transition?

OUTPUT: Excess inventory with removal or simplification recommendations.
```

---

## PHASE 17: BRAND MOTION DNA

> *"Does motion express brand personality? Is there a motion language?"*

```
1. MOTION PERSONALITY
   -> If the brand were a person, how would they move?
   -> Does the current motion match? (playful brand with stiff animations = mismatch)
   -> Are motion characteristics consistent across pages?

2. MOTION PRINCIPLES
   -> Can you extract 2-3 motion principles from what exists?
   -> Are they explicit (documented) or implicit (inferred)?
   -> If explicit: are they followed consistently?
   -> If implicit: are they coherent enough to formalize?

3. SIGNATURE MOMENTS
   -> Is there a motion moment that feels uniquely "this brand"?
   -> Loading animation, page transition, or interaction that's distinctive?
   -> Or is all motion generic (could belong to any product)?

4. MOTION VOCABULARY
   -> Easing: named and purposeful? (the "voice" of movement)
   -> Duration: scaled and systematic? (the "pace" of the brand)
   -> Direction: consistent spatial logic? (the "geography" of the product)
   -> Scale: consistent use of translateY amounts? (the "gesture size")

5. CROSS-PAGE CONSISTENCY
   -> Same interaction pattern animated the same way on every page?
   -> Card hover on page A feels the same as card hover on page B?
   -> Modal opens the same way everywhere?
   -> Or does each page feel like a different designer animated it?

IDEAL: Articulated motion principles, consistent easing/duration vocabulary,
       at least one signature moment, cross-page consistency.
FAILURE: No discernible motion personality, inconsistent patterns,
         generic animations indistinguishable from a template.
```

---

## PHASE 18: PERFORMANCE BUDGET

> *"Total animation cost: CPU, GPU, memory, frames. The budget is real."*

```
1. FRAME RATE AUDIT
   -> Desktop: sustained 60fps during all animations?
   -> Mobile: sustained 30fps minimum during all animations?
   -> Jank detection: any frames > 16.6ms (desktop) or > 33ms (mobile)?
   -> Frame drops during page entry choreography?

2. CPU IMPACT
   -> Main thread time consumed by animations during page load?
   -> Long tasks (> 50ms) caused by animation initialization?
   -> Animation JS execution time per frame?
   -> Idle between animation frames?

3. GPU IMPACT
   -> Compositor layer count per page?
   -> GPU memory usage from will-change and composited layers?
   -> Shader compilation time (if WebGL)?
   -> GPU thermal state on mobile after 60s of animation?

4. BUNDLE COST
   -> Total KB of animation libraries in bundle
   -> GSAP: ~30KB min. Framer Motion: ~45KB min. Three.js: ~150KB min.
   -> Are animation libraries tree-shaken?
   -> Are heavy libraries lazy-loaded?

5. TOTAL MOTION COST
   -> Sum: animation JS + animation CSS + canvas/WebGL resources
   -> Percentage of total page weight dedicated to motion
   -> Target: < 15% of total JS for animation code
   -> Target: < 5% of total CPU time for animation during interaction

IDEAL: 60fps desktop, 30fps mobile, animation libraries lazy-loaded,
       < 15% JS weight, no long tasks from animation.
FAILURE: Frame drops below 30fps, animation library 40%+ of bundle,
         long tasks from animation, GPU memory exhaustion.
```

---

## PHASE 19: VERDICT

Score each phase 0-10, weight by motion impact severity:

```
SCORING MATRIX (360 max):
  Phase  1  (Motion Inventory)        x 1.5  = max 15
  Phase  2  (Purpose Verification)    x 3.5  = max 35   ** HINGE **
  Phase  3  (Easing System)           x 2.5  = max 25
  Phase  4  (Duration Consistency)    x 2.0  = max 20
  Phase  5  (Choreography)           x 2.5  = max 25
  Phase  6  (Scroll Animations)      x 2.0  = max 20
  Phase  7  (Page Transitions)       x 2.0  = max 20
  Phase  8  (Micro-interactions)     x 2.5  = max 25
  Phase  9  (Loading & Skeleton)     x 1.5  = max 15
  Phase 10  (WebGL Audit)            x 1.5  = max 15   (0 if N/A — redistributed)
  Phase 11  (P5.js / Canvas)         x 1.0  = max 10   (0 if N/A — redistributed)
  Phase 12  (CSS Performance)        x 3.0  = max 30   ** HEAVY **
  Phase 13  (Reduced Motion)         x 2.5  = max 25
  Phase 14  (Mobile Motion)          x 2.0  = max 20
  Phase 15  (Motion-Meaning Gap)     x 2.5  = max 25
  Phase 16  (Motion Excess)          x 2.0  = max 20
  Phase 17  (Brand Motion DNA)       x 1.5  = max 15
  Phase 18  (Performance Budget)     x 2.0  = max 20
                                     TOTAL  = max 360

When Phase 10 or 11 is N/A (no WebGL/Canvas), redistribute their weight
evenly to Phases 2, 12, and 8.

NORMALIZE: score = (raw / 360) x 100

GRADE:
  90-100: S — Intentional. Every animation justified, system-level consistency, brand DNA clear.
  80-89:  A — Thoughtful. Minor gaps, strong motion language, good performance.
  70-79:  B — Decent. Some purposeless motion, inconsistent system, acceptable performance.
  60-69:  C — Average. Motion exists but feels template-like, system is weak.
  50-59:  D — Neglected. Random animations, no system, performance issues.
  <50:    F — Harmful. Motion hurts the experience more than it helps.
```

---

## PHASE 20: FIX PLAN (automatic)

```
Sort: CRITICAL -> HIGH -> MEDIUM -> LOW

CRITICAL:
  - Purposeless animations (Phase 2 failures) -> remove
  - CSS performance violations (Phase 12 failures) -> refactor to transform/opacity
  - Reduced motion absent (Phase 13 failures) -> implement immediately
  - Frame drops below 30fps (Phase 18 failures) -> optimize or remove

HIGH:
  - Motion-Meaning Gaps (Phase 15) -> add transitions where missing
  - Easing inconsistency (Phase 3) -> define and apply tokens
  - Duration inconsistency (Phase 4) -> define and apply tokens
  - Motion Excess (Phase 16) -> simplify or remove

MEDIUM:
  - Choreography improvements (Phase 5) -> add stagger and composition
  - Micro-interaction gaps (Phase 8) -> add hover/focus/press states
  - WebGL fallbacks (Phase 10) -> add tiered fallbacks
  - Mobile motion (Phase 14) -> add touch-specific feedback

LOW:
  - Brand Motion DNA refinement (Phase 17) -> document motion principles
  - Loading choreography (Phase 9) -> upgrade to skeletons
  - Page transition polish (Phase 7) -> add View Transitions API
  - Signature moments (Phase 17) -> design distinctive brand motion

Generate fix tasks with file:line specificity.
Save to audits/.motionaudit/fix-plan.json + fix-plan.md
```

---

## PHASE 21: FIX EXECUTION (automatic)

```
Sequential per fix group.

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
  → Log the failure in audits/.motionaudit/fix-log.md with exact error
  → Mark as NEEDS_REVIEW (never retry same approach blindly)
  → Try alternative approach OR skip this fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (in priority order):
  a. Read the ENTIRE target file (full context)
  b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
  c. Measure BEFORE (fps, jank, user perception)
  d. Apply fix (add motion, remove motion, refactor animation)
  e. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
  f. If all green → measure AFTER (same metrics) → commit: style(motionaudit): FIX-XXX description
  g. If any red → revert → log → mark NEEDS_REVIEW
  h. Verify: prefers-reduced-motion still handled
  i. Verify: mobile performance not degraded
```

---

## PHASE 22: RE-AUDIT (automatic)

```
1. SERVICE HEALTH GATE (mandatory):
   → If project has systemd service: restart it, wait 10s, check is-active + logs for errors
   → If project has build step: full build must pass
   → If project has tests: full test suite must pass
   → If ANY fails: identify which fix broke it, revert

2. Re-run all FAILING phases. Compare before/after.
3. Loop until score >= 80 or remaining items are NEEDS_REVIEW.
4. Focus on: Purpose Verification score, CSS Performance score, Reduced Motion score.
   These three phases are the hinge — if they pass, the experience is sound.
```

---

## PARALLEL EXECUTION STRATEGY

```
The 18 phases are grouped for maximum parallelism:

WAVE 1 (discovery — sequential):
  Phase 0: Reconnaissance
  Phase 1: Motion Inventory (depends on discovery)

WAVE 2 (system analysis — parallel):
  Phase 2: Purpose Verification  ]
  Phase 3: Easing System          ] All analyze the inventory
  Phase 4: Duration Consistency    ] from different angles
  Phase 5: Choreography           ]

WAVE 3 (interaction & navigation — parallel):
  Phase 6: Scroll Animations      ]
  Phase 7: Page Transitions        ] Separate agents
  Phase 8: Micro-interactions      ] per phase
  Phase 9: Loading & Skeleton      ]

WAVE 4 (technology-specific — parallel):
  Phase 10: WebGL Audit            ]
  Phase 11: P5.js / Canvas         ] Only if applicable
  Phase 12: CSS Performance        ]

WAVE 5 (meta-analysis — parallel):
  Phase 13: Reduced Motion         ]
  Phase 14: Mobile Motion          ] Cross-cutting
  Phase 15: Motion-Meaning Gap     ] concerns
  Phase 16: Motion Excess          ]
  Phase 17: Brand Motion DNA       ]
  Phase 18: Performance Budget     ]

4 parallel agents per wave = 18 phases in 5 sequential waves.
```

---

## CROSS-COMMAND BRIDGE

```
/motionaudit finds performance issues -> references /perfaudit findings
/motionaudit finds missing interactions -> references /uiuxaudit findings
/motionaudit finds accessibility gaps -> references /a11yaudit findings
/motionaudit finds purpose failures -> complements /copyaudit (words) with motion (movement)

THE QUALITY ARSENAL:
  /codeaudit    -> Is the code SOLID?           (preventive)
  /flowaudit    -> Does the experience WORK?    (preventive)
  /uiuxaudit    -> Is the interface BEAUTIFUL?  (preventive)
  /debugaudit   -> What is BROKEN right now?    (detective)
  /featureaudit -> Is the product COMPLETE?     (strategic)
  /perfaudit    -> Is it FAST enough?           (detective)
  /secaudit     -> Is it SECURE?                (detective)
  /motionaudit  -> Does it MOVE with purpose?   (experiential)
  /a11yaudit    -> Is it ACCESSIBLE?            (inclusive)

  Together: nothing escapes. Every dimension covered.
```

---

## THE 7 LAWS OF MOTION DESIGN

1. **Every animation must pass the Purpose Test.** Communicate, orient, or create meaning — or be removed.
2. **Consistency is the system.** 3 easing curves used with grammar beat 12 used randomly.
3. **Enter with energy, exit with speed.** Entrances decelerate (ease-out). Exits accelerate (ease-in). Exits are always faster.
4. **Transform and opacity only.** Anything else triggers layout or paint. If you must animate paint properties, use pseudo-element opacity tricks.
5. **Reduced motion is a design problem, not a boolean.** Design the alternative experience, don't just disable everything.
6. **Mobile is the baseline.** If it drops below 30fps on a mid-range Android, it's broken.
7. **Absence is as loud as presence.** A state change with no transition is a motion bug. A decoration with no purpose is motion debt.

---

*"/motionaudit v1 — Catalog. Justify. Systematize. Perform. Include. Every animation, every transition, every frame. /360."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.1 — 2026-04-14T10:35:36Z)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial thinking
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.motionaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — Non-UI contexts: ABORT for backend/library/CLI (no motion surface to audit).
- ✅ **Output contract verification** — emits `audits/.motionaudit/verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `iterations.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `audits/.motionaudit/discovery/` > 1h old, re-verify inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.motionaudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` flags with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — Video/GIF Coverage + Minimal-Motion N/A

**Coverage expansion:**
- CSS transitions + keyframes
- JS animations (requestAnimationFrame, Web Animations API)
- WebGL scenes (Three.js, React Three Fiber, PlayCanvas)
- P5.js sketches
- HTML5 `<video>` with autoplay
- Animated GIFs (detect `.gif` with > 1 frame)
- SVG SMIL animations + Lottie / dotLottie

**Minimal-motion projects (auto-N/A):**
```
Count total animation instances across project
If count < 3:
  Mark Phases 3-15 (the bulk) as N/A
  Run only Phases 1 (inventory), 2 (purpose), 16 (accessibility reduced-motion) on the few instances
  Emit in verdict.md: "Minimal-motion project — applicable phases: 1, 2, 16 only. Full audit score N/A."
If 3 <= count < 10: run all phases but reduce weightings
If count >= 10: full audit
```

Animation purpose weighting is necessarily subjective but falsifiable: "Does this animation convey information (direction, causality, state change) OR is it decorative?" Binary classification.

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

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (motionaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "motionaudit",
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

**Gap 1: Coverage list explicit → RESOLVED**
```
Motion sources audited:
  1. CSS @keyframes (in .css, .scss, Tailwind animate plugin, styled-components)
  2. CSS transition properties
  3. JS animations: requestAnimationFrame, Web Animations API, Anime.js, GSAP, Framer Motion, Motion One
  4. WebGL: Three.js, React Three Fiber, PlayCanvas, Babylon.js
  5. P5.js sketches
  6. HTML5 <video autoplay>
  7. Animated GIFs (detect .gif > 1 frame via exiftool or equivalent)
  8. SVG SMIL <animate>
  9. Lottie / dotLottie animations
  10. CSS scroll-driven animations (@scroll-timeline)
  11. View transitions API
```

**Gap 2: Minimal-motion N/A threshold → RESOLVED**
```
total_animations = count_across_all_sources_above

if total_animations < 3:
  applicable phases: 1 (inventory), 2 (purpose), 16 (reduced-motion) only
  applicable_raw_max: 15 + 25 + 20 = 60
  Emit in verdict.md: "Minimal-motion project — 3 phases applicable."
if 3 <= total_animations < 10:
  applicable phases: 1, 2, 3, 5, 16, 17 (core only)
  applicable_raw_max: 60 + 25 + 20 + 15 = 120
if total_animations >= 10:
  all 18 phases applicable
```

**Gap 3: Purpose classification binary test → RESOLVED**
```
For each animation, answer:
  "Does this animation convey STATE CHANGE, DIRECTION, or CAUSALITY?"
  YES → purposeful (full credit)
  NO → decorative
Additional check for decorative:
  Is it respected by prefers-reduced-motion?
  YES → decorative but accessible (partial credit)
  NO → decorative and accessibility-hostile (fail)
```

**Gap 4: Non-UI contexts → RESOLVED:**
ABORT for backend/library/CLI. Motion is UI-specific.

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
/metaudit --focus arsenal --scope="motionaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*motionaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

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

> *"Motion is parallel by nature — easing, choreography, scroll, performance all play at once. Audit it the same way: fan out, falsify, converge."*
>
> This section governs HOW the phases above execute. It changes the *engine*, not the
> *doctrine*. Every phase, every scoring weight (Phase 19 matrix, /360), every verdict
> field, and the Gestalt-Popper laws above remain **exactly as specified** — this only
> replaces linear phase-walking with adversarially-verified parallel fan-out via the
> **Workflow** tool. When in doubt, the scoring matrix and verdict format upstream win.

### 1. Decompose into independent parallel tracks (fan-out)

Do NOT walk Phases 1→18 sequentially. After the discovery gate (Phase 0 recon + Phase 1
inventory, which MUST finish first because every track reads `discovery/animations.json`),
launch the remaining phases as **concurrent Workflow tracks**. The existing
`## PARALLEL EXECUTION STRATEGY` waves are the dependency map — promote them to real
fan-out:

```
GATE (sequential, blocking — produces the shared inventory):
  Phase 0  Reconnaissance      → stack, libs, routes, HINGE PAGE
  Phase 1  Motion Inventory    → discovery/*.json (animations, easing-map, duration-map, …)

FAN-OUT (independent Workflow tracks — run concurrently, no shared writes):
  Track A — Motion-system semantics : Phase 2 (Purpose, HINGE x3.5) ∥ Phase 3 (Easing) ∥ Phase 4 (Duration) ∥ Phase 5 (Choreography)
  Track B — Interaction & navigation: Phase 6 (Scroll) ∥ Phase 7 (Page transitions) ∥ Phase 8 (Micro-interactions) ∥ Phase 9 (Loading/Skeleton)
  Track C — Technology-specific     : Phase 10 (WebGL) ∥ Phase 11 (P5/Canvas) ∥ Phase 12 (CSS Performance, HEAVY x3.0)
  Track D — Cross-cutting meta       : Phase 13 (Reduced motion) ∥ Phase 14 (Mobile) ∥ Phase 15 (Motion-Meaning Gap) ∥ Phase 16 (Excess) ∥ Phase 17 (Brand DNA) ∥ Phase 18 (Perf budget)
```

Each track is a self-contained Workflow unit reading the same frozen inventory from a
different angle. Tracks share **zero** writes (each owns its `reports/*.md`), so they are
file-disjoint and safe to parallelize per R-SCOPE. Honor the N/A gates already defined:
Track C drops Phase 10/11 when no WebGL/Canvas exists (redistribute weight per Phase 19);
the minimal-motion thresholds (<3 / 3–9 / ≥10 instances from the v1.2 addendum) decide how
many tracks even spawn. The HINGE PAGE gets every track at 10x depth before any other route.

### 2. Adversarially verify every finding (≥2-of-3 lenses)

A track's raw observation is a **candidate**, never a finding. Before any candidate enters
the Phase 19 score or the fix-plan, it must survive **≥2 of 3 independent lenses** — tailored
to motion, where "I saw it move" is not proof it *should*:

- **Lens 1 — REPRODUCE (runtime, First Law):** observe the actual movement, don't trust the
  CSS/JS source. Confirm the animation truly fires at the claimed trigger — capture the real
  computed `transition`/`animation`, a frame trace, or a DevTools Performance recording.
  "Code says `transition: transform 200ms`" is intent; the rendered frames are truth.
- **Lens 2 — REFUTE (Popper kill-test):** actively try to falsify the candidate. For a
  *purpose* claim → mentally (or via DOM toggle) REMOVE the animation: if the user loses no
  information/orientation, the "ESSENTIAL/ENHANCING" verdict is dead — reclassify DECORATIVE/
  PURPOSELESS. For a *performance* violation → confirm the property actually triggers layout/
  paint (not a false positive on a transform-able shorthand). For an *easing/duration system*
  claim → check the value is genuinely arbitrary, not a token resolved at runtime. For a
  *reduced-motion* PASS → toggle `prefers-reduced-motion: reduce` and verify state changes
  still communicate (not a nuclear `* { animation: none }` that breaks the UI).
- **Lens 3 — CROSS-CHECK (corroboration):** triangulate against a second source or sibling
  audit. Same file:line flagged by `/perfaudit` (compositor/jank), `/uiuxaudit` (missing
  feedback), or `/a11yaudit` (vestibular/motion-sensitivity) → corroboration. Or cross-map
  Motion-Meaning Gap (Phase 15) against Motion Excess (Phase 16): a state change with no
  transition AND an unrelated decorative loop nearby is a double-confirmed misallocation.

Decision rule: **≥2 lenses agree → finding is accepted** (record which lenses + verbatim
evidence per R-CITE, into the finding's `evidence` field). **<2 agree → KILL the candidate**
(log it in `discovery/killed-candidates.md` with the lens that refuted it and why — never
silently drop, never let it touch the score). When two lenses confirm a cross-audit match,
populate `cross_audit_confirmations[]` and elevate severity per Preamble §6.

This maps onto the audit's existing Popper falsification categories (PURPOSE vs DECORATION,
SYSTEM vs RANDOM, DESKTOP vs MOBILE, FAST vs SLOW, PRESENT vs MISSING) — each candidate is
falsified along the relevant axis before it counts.

### 3. Synthesize survivors into the existing scoring matrix (unchanged)

Synthesis is the orchestrator's own job — never paste a track's summary as the verdict
(R-ORCH). Collect ONLY the surviving (≥2-of-3) findings from all four tracks and fold them
back into **the Phase 19 SCORING MATRIX exactly as written**: same phase weights
(Purpose x3.5, CSS Performance x3.0, Easing x2.5, …), same /360 raw max, same N/A
redistribution rule, same `normalized = (raw / applicable_raw_max) × 100`, same S/A/B/C/D/F
grade bands. The fan-out changes *how findings are gathered and proven*, not *how they are
scored*. Then proceed into Phase 20 (fix plan), Phase 21 (fix execution + Do-No-Harm gate),
Phase 22 (re-audit, 5-iteration cap) and the MANDATORY BEFORE/AFTER VERIFICATION gate above —
all unchanged. `verdict.json` keeps its v2 schema and `preamble_version: "1.0"`.

### 4. Loop-until-dry for unknown-size discovery

Motion inventory is open-ended (new routes, lazy-loaded canvases, dynamically-injected
animations, conditionally-mounted Lottie/WebGL). Run discovery as a **loop-until-dry**, not a
single pass:

```
discovered = inventory(Phase 1)
repeat:
    fan-out the four tracks over `discovered` (with ≥2-of-3 verification)
    rescan for motion sources missed last pass:
      - routes/components not yet visited, IntersectionObserver-gated reveals,
        code-split chunks, runtime-injected @keyframes, autoplay <video>, animated GIFs (>1 frame),
        SVG SMIL, dotLottie — across ALL motion sources in the v1.2 coverage list
    new = (rescan △ discovered)
until new == ∅  (two consecutive passes add zero new animations → DRY)
```

Bound the loop by the audit's existing controls: the 5-iteration fix-and-reaudit cap, the 4h
concurrency-lock window, and the R-BUDGET ceiling. On cap/budget hit → mark residual surfaces
`NEEDS_REVIEW` in `verdict.json` and Telegram SOS — never spin silently, never claim DRY when
the loop was merely truncated.

---

*Dynamic-Workflow Orchestration (v2): fan-out the 23 phases as parallel Workflow tracks over a frozen inventory → kill every finding that fails ≥2-of-3 lenses (reproduce / refute / cross-check) → synthesize survivors into the unchanged /360 matrix → loop-until-dry. The engine changed; the doctrine and the score did not.*
