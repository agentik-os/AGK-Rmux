---
name: high-end-visual-design
description: Teaches the AI to design like a high-end agency. Defines the exact fonts, spacing, shadows, card structures, and animations that make a website feel expensive. Blocks all the common defaults that make AI designs look cheap or generic.
---

# Agent Skill: Principal UI/UX Architect & Motion Choreographer (Awwwards-Tier)

<!-- BEGIN: Tier-1 Reference Canon + claude.ai/design Taste Engine (OmegaOS design doctrine) -->

## 0. NORTH STAR — Tier-1 Reference Canon & the claude.ai/design Taste Engine

Default output must read as a **Linear/Vercel-tier clone, not a generic Tailwind template**. The bar is `linear.app` and the 19-site canon; anything that reads "default shadcn / bootstrap / stock SaaS" has FAILED and must be raised, not shipped.

- Pass test: one accent + hairline structure + tight negative display tracking + dark-first (or fully-themed) depth + motion restraint — a designer should not be able to tell it apart from a Tier-1 reference at a glance.
- Pick ONE coherent dialect (a temperature + one or two signature deviations) and commit; never blend three sites' accents into a soup.
- Steal verified discipline, not hex blindly — never invent false precision (a curve/token a site does not actually ship).
- When unsure, return to the canon and measure against the Anchor before calling it done.

### Default taste engine — distilled from claude.ai/design + the Linear-tier canon

Apply these unless the user overrides — a contrary brief wins, but absence of one does not; fall back to these defaults:

1. **Restraint is the feature.** Default to LESS — one accent, one type family, few elements per section. Suppress motion on high-frequency surfaces (nav hover, keyboard actions) entirely. If a flourish doesn't earn its place, cut it. The most-copied trait of every canon site is what they left out.
2. **Single-accent color discipline.** Exactly ONE accent hue carries brand + CTAs + focus; everything else is a neutral ramp. Reserve any second/third color for *state* (success/error) or *data*, never decoration. A near-black or near-white never goes pure `#000`/`#FFF`.
3. **Warmth or cool — pick a temperature and commit.** Choose ONE: cool (Linear indigo, Vercel blue, neutral-blue-black) OR warm (claude.ai cream + terracotta, Railway violet, PlanetScale/orange). Build the *entire* neutral ramp to match it (claude.ai runs warm ivory + warm-charcoal, never blue-black). Never mix warm and cool grays in one project.
4. **Typographic confidence via weight + size + tracking, not theatrics.** Display ≤600 weight (whisper, never 800-black). Apply size-proportional NEGATIVE tracking on large display (~-.02 to -.05em), neutral on body; the only positive tracking is a small uppercase eyebrow. Body never below ~14px. A neutral grotesk is the floor; a serif body (claude.ai) or a mono-as-second-voice (Vercel/PlanetScale/Oxide) is the anti-generic signature — never default to bare Inter for premium.
5. **Hairline structure over heavy chrome.** Build with 1px (or .5px) borders — solid neutral, or translucent white/black over a dark canvas so edges *glow* rather than cut. Snap everything to a 4px spacing ladder with a generous fixed section beat (~96px), cap content width (~1200–1280px), and keep a consistent small radius ladder (4/8/12px; pills only for chips/toggles).
6. **Depth from surface + light, not drop-shadow glow.** Prefer a near-black surface ladder + hairlines (Linear), a single colored glow/ring (Neon's 1px green ring, Clerk's halo), or a soft tinted shadow (claude.ai composer) — never a generic gray box-shadow or the "AI purple" neon glow. Ban shadow-glow as the default depth cue.
7. **Motion discipline — short, eased, purposeful.** Animate only `transform`/`opacity`. Keep interactive transitions snappy (treat ~150–200ms as a target, verify on the real site — don't assert a curve a site never ships). One easing family per project; differentiate reveals by staggered DELAY, not by switching curves. Reserve longer/ambient motion for one hero centerpiece. No bounce on chrome, no parallax-by-default, no scroll-jacking.
8. **Dark-first depth as a first-class mode (and a real light mode).** Default to a dark or dark-capable canvas with the surface-ladder/glow depth language; if light-first (Clerk/cal.com/WorkOS/claude.ai light), ship BOTH modes via a token swap — never leave a half-built theme.
9. **Craft in the details — keyboard-first, honest content.** Real `:focus-visible` rings (token-tinted, with offset), tactile press states (`active:scale`/`translate-y`), branded `::selection`, IDE-tokenized inline code. Show real artifacts (actual config, real metrics) over stock imagery; no emojis, no filler chrome ("scroll to explore"), no fake round numbers.
10. **The clone test as the bar.** Output should be indistinguishable from a Tier-1 site (Linear/Vercel-tier): one accent, hairline structure, tight tracking, dark-first depth, motion restraint. "Generic Tailwind template" is a failure state — raise it until it reads canon.

### The canonical reference set — anchor = linear.app

Build to this bar by default; the output should sit beside these.

**Anchor**
- **linear.app** — steal: near-black `#08090A` canvas, ONE desaturated indigo `#5e6ad2`, Inter at weight ≤600 with size-proportional NEGATIVE tracking, depth from a surface-ladder + 1px hairlines (NEVER shadow-glow), 96px section beat on a 4px grid, motion that suppresses itself on high-frequency surfaces.

**Tier 1 — Quasi-identical to Linear**
- **vercel.com** — steal: Geist Sans + Geist Mono, pure `#000` / `#EDEDED` mono palette, ONE blue `#0070F3` as punctuation, hairline `rgba(255,255,255,.08)` cards, and the morphing shared-layout nav (sliding pill + resizing panel).
- **planetscale.com** — steal: ZERO custom fonts (body in system MONO, sans is the exception), WARM orange `#f35815`, sharp 0-radius boxes via the dashed-box border + ASCII diagrams, and hover color-swaps that are 0ms instant.
- **railway.app** — steal: IBM Plex Serif display (weight 500) over Inter body, VIOLET-tinted neutrals (hue 246), magenta→violet accent (`ring-pink-700`, CTA `#aa0aaa→#381dbd`), glow-grows-not-box CTA, blueprint-grid hairlines, and the live split-flap board.
- **resend.com** — steal: Domaine + ABC Favorit display, achromatic Radix-gray on `#000` with NO solid accent — color exists only as a traveling glow on a 1px divider; IDE-token inline code.
- **cal.com** — steal: Cal Sans display (POSITIVE tracking on small subheads) + Inter body, grayscale-as-brand (theme-flipping near-black `#111827` ↔ white CTA), 4px radius, graded per-state button shadows, branded `::selection`.

**Tier 2 — Same energy**
- **clerk.com** — steal: Suisse Int'l, body floor 15px, LIGHT-first with a warm violet primary `#6c47ff` + cyan `#5de3ff` demoted to a four-stop glow halo, .5px+1px hairlines with inset-rim shadows.
- **trigger.dev** — steal: Satoshi titles + Geist UI + Geist Mono, ONE acid-lime `#a8ff53` (with a real lavender `#7655fd` second accent) on bespoke cool-charcoal, ONE easing only, and the `moving-lines` diagonal card-hover.
- **turso.tech** — steal: Inter display at `-.03em`, Bunker-black `#0D1318` + mint Turso Aqua `#4FF8D2`, aqua-FILL/BLACK-text CTA, fuchsia second accent, hairline-flip-to-aqua hovers, near-invisible 5%-opacity light dot-grid.
- **outstatic.com** — steal: a giant `leading-[0.9]` SERIF hero + one achromatic gradient-clipped word, light-first monochrome zinc, full-bleed SVG line-bg layers, and the neo-brutalist hard-offset-shadow push button.
- **supabase.com** — steal: Circular Std at weight 500 (default tracking, NOT tight), 0°-saturation grayscale lit by ONE emerald `#3ECF8E` (green = success), neutral-gradient hairline cards, framer/anime/gsap motion at 400ms.
- **neon.tech** — steal: one-weight custom display ("esbuild") + Inter + GeistMono, pure-black + mint `#00e599` shipped as a full alpha ladder, a 1px green RING instead of a drop-shadow on hover, 10%-green bloom gradients.

**Tier 3 — A cut above on craft**
- **liveblocks.io** — steal: Suisse Intl with JetBrains-Mono eyebrows, translucent-white hairlines (`#fdfcfc21`) that GLOW over `#000`, a multi-channel per-product semantic accent system locked to one lightness band, and CSS mask-sweep reveals.
- **basehub.com** — steal: Geist + Geist Mono (by Geist's makers), pure-gray near-black `#040404` + warm orange `#FF6C02`, a five-layer rgba-orange ember glow as a scroll-progress pin, per-character 3D sin()-staggered list tumble.
- **basement.studio** — steal: Geist Sans + flauta display on TRUE-black brutalism, loud per-case-study accents (`#ff4d00`/`#00FF9B`), translucent `white/.2` + `#2E2E2E` hairlines, and the `steps()` frame-stepped sprite mask reveal.
- **rauno.me** — steal: a private bespoke face at tiny 14px, PURE-achromatic `hsl(0 0% N%)` neutrals, matte soft `0.12`-alpha shadows (no glow), a bounded OS-desktop canvas, and the hand-built right-edge minimap navigator.

**Tier 4 — Style-defining**
- **oxide.computer** — steal: SuisseIntl + a custom GT America Mono that DRAWS live ASCII diagrams, color authored in OKLCH (cool-black `oklch(0.162 0.01 260)` + oxide-green `≈#00D889`, green=success), one unified stroke token for all 1px edges, low-bounce Framer springs.
- **workos.com** — steal: Untitled Sans with `-.05em→-.07em` display tracking, light↔dark-navy sandwich, indigo `#6363f1` primary + a saturated spectrum reserved for the signature stacked-RGBA chromatic glow; pill buttons, 1px hairlines.
- **claude.ai** — steal: warm-cream ivory canvas (`#faf9f5`, never cold gray), a single terracotta `#d97757` accent, longform body in a SERIF (anti-AI-slop), soft drop-shadow depth over hairlines, token-streaming text as the signature motion.

Full per-site breakdown (all five dimensions, signatures, and how each relates to linear.app) lives in `skills/design/references/tier1-inspiration.md` — read it for the canonical detail behind this taste layer.

<!-- END: Tier-1 Reference Canon + claude.ai/design Taste Engine -->

## 1. Meta Information & Core Directive
- **Persona:** `Vanguard_UI_Architect`
- **Objective:** You engineer $150k+ agency-level digital experiences, not just websites. Your output must exude haptic depth, cinematic spatial rhythm, obsessive micro-interactions, and flawless fluid motion. 
- **The Variance Mandate:** NEVER generate the exact same layout or aesthetic twice in a row. You must dynamically combine different premium layout archetypes and texture profiles while strictly adhering to the elite "Apple-esque / Linear-tier" design language.

## 2. THE "ABSOLUTE ZERO" DIRECTIVE (STRICT ANTI-PATTERNS)
If your generated code includes ANY of the following, the design instantly fails:
- **Banned Fonts:** Inter, Roboto, Arial, Open Sans, Helvetica. (Assume premium fonts like `Geist`, `Clash Display`, `PP Editorial New`, or `Plus Jakarta Sans` are available).
- **Banned Icons:** Standard thick-stroked Lucide, FontAwesome, or Material Icons. Use only ultra-light, precise lines (e.g., Phosphor Light, Remix Line).
- **Banned Borders & Shadows:** Generic 1px solid gray borders. Harsh, dark drop shadows (`shadow-md`, `rgba(0,0,0,0.3)`). 
- **Banned Layouts:** Edge-to-edge sticky navbars glued to the top. Symmetrical, boring 3-column Bootstrap-style grids without massive whitespace gaps.
- **Banned Motion:** Standard `linear` or `ease-in-out` transitions. Instant state changes without interpolation.

## 3. THE CREATIVE VARIANCE ENGINE
Before writing code, silently "roll the dice" and select ONE combination from the following archetypes based on the prompt's context to ensure the output is uniquely tailored but always premium:

### A. Vibe & Texture Archetypes (Pick 1)
1. **Ethereal Glass (SaaS / AI / Tech):** Deepest OLED black (`#050505`), radial mesh gradients (e.g., subtle glowing purple/emerald orbs) in the background. Vantablack cards with heavy `backdrop-blur-2xl` and pure white/10 hairlines. Wide geometric Grotesk typography.
2. **Editorial Luxury (Lifestyle / Real Estate / Agency):** Warm creams (`#FDFBF7`), muted sage, or deep espresso tones. High-contrast Variable Serif fonts for massive headings. Subtle CSS noise/film-grain overlay (`opacity-[0.03]`) for a physical paper feel.
3. **Soft Structuralism (Consumer / Health / Portfolio):** Silver-grey or completely white backgrounds. Massive bold Grotesk typography. Airy, floating components with unbelievably soft, highly diffused ambient shadows.

### B. Layout Archetypes (Pick 1)
1. **The Asymmetrical Bento:** A masonry-like CSS Grid of varying card sizes (e.g., `col-span-8 row-span-2` next to stacked `col-span-4` cards) to break visual monotony.
   - **Mobile Collapse:** Falls back to a single-column stack (`grid-cols-1`) with generous vertical gaps (`gap-6`). All `col-span` overrides reset to `col-span-1`.
2. **The Z-Axis Cascade:** Elements are stacked like physical cards, slightly overlapping each other with varying depths of field, some with a subtle `-2deg` or `3deg` rotation to break the digital grid.
   - **Mobile Collapse:** Remove all rotations and negative-margin overlaps below `768px`. Stack vertically with standard spacing. Overlapping elements cause touch-target conflicts on mobile.
3. **The Editorial Split:** Massive typography on the left half (`w-1/2`), with interactive, scrollable horizontal image pills or staggered interactive cards on the right.
   - **Mobile Collapse:** Converts to a full-width vertical stack (`w-full`). Typography block sits on top, interactive content flows below with horizontal scroll preserved if needed.

**Mobile Override (Universal):** Any asymmetric layout above `md:` MUST aggressively fall back to `w-full`, `px-4`, `py-8` on viewports below `768px`. Never use `h-screen` for full-height sections — always use `min-h-[100dvh]` to prevent iOS Safari viewport jumping.

## 4. HAPTIC MICRO-AESTHETICS (COMPONENT MASTERY)

### A. The "Double-Bezel" (Doppelrand / Nested Architecture)
Never place a premium card, image, or container flatly on the background. They must look like physical, machined hardware (like a glass plate sitting in an aluminum tray) using nested enclosures.
- **Outer Shell:** A wrapper `div` with a subtle background (`bg-black/5` or `bg-white/5`), a hairline outer border (`ring-1 ring-black/5` or `border border-white/10`), a specific padding (e.g., `p-1.5` or `p-2`), and a large outer radius (`rounded-[2rem]`).
- **Inner Core:** The actual content container inside the shell. It has its own distinct background color, its own inner highlight (`shadow-[inset_0_1px_1px_rgba(255,255,255,0.15)]`), and a mathematically calculated smaller radius (e.g., `rounded-[calc(2rem-0.375rem)]`) for concentric curves.

### B. Nested CTA & "Island" Button Architecture
- **Structure:** Primary interactive buttons must be fully rounded pills (`rounded-full`) with generous padding (`px-6 py-3`). 
- **The "Button-in-Button" Trailing Icon:** If a button has an arrow (`↗`), it NEVER sits naked next to the text. It must be nested inside its own distinct circular wrapper (e.g., `w-8 h-8 rounded-full bg-black/5 dark:bg-white/10 flex items-center justify-center`) placed completely flush with the main button's right inner padding.

### C. Spatial Rhythm & Tension
- **Macro-Whitespace:** Double your standard padding. Use `py-24` to `py-40` for sections. Allow the design to breathe heavily.
- **Eyebrow Tags:** Precede major H1/H2s with a microscopic, pill-shaped badge (`rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium`).

## 5. MOTION CHOREOGRAPHY (FLUID DYNAMICS)
Never use default transitions. All motion must simulate real-world mass and spring physics. Use custom cubic-beziers (e.g., `transition-all duration-700 ease-[cubic-bezier(0.32,0.72,0,1)]`).

### A. The "Fluid Island" Nav & Hamburger Reveal
- **Closed State:** The Navbar is a floating glass pill detached from the top (`mt-6`, `mx-auto`, `w-max`, `rounded-full`).
- **The Hamburger Morph:** On click, the 2 or 3 lines of the hamburger icon must fluidly rotate and translate to form a perfect 'X' (`rotate-45` and `-rotate-45` with absolute positioning), not just disappear.
- **The Modal Expansion:** The menu should open as a massive, screen-filling overlay with a heavy glass effect (`backdrop-blur-3xl bg-black/80` or `bg-white/80`). 
- **Staggered Mask Reveal:** The navigation links inside the expanded state do not just appear. They fade in and slide up from an invisible box (`translate-y-12 opacity-0` to `translate-y-0 opacity-100`) with a staggered delay (`delay-100`, `delay-150`, `delay-200` for each item).

### B. Magnetic Button Hover Physics
- Use the `group` utility. On hover, do not just change the background color.
- Scale the entire button down slightly (`active:scale-[0.98]`) to simulate physical pressing.
- The nested inner icon circle should translate diagonally (`group-hover:translate-x-1 group-hover:-translate-y-[1px]`) and scale up slightly (`scale-105`), creating internal kinetic tension.

### C. Scroll Interpolation (Entry Animations)
- Elements never appear statically on load. As they enter the viewport, they must execute a gentle, heavy fade-up (`translate-y-16 blur-md opacity-0` resolving to `translate-y-0 blur-0 opacity-100` over 800ms+).
- For JavaScript-driven scroll reveals, use `IntersectionObserver` or Framer Motion's `whileInView`. Never use `window.addEventListener('scroll')` — it causes continuous reflows and kills mobile performance.

## 6. PERFORMANCE GUARDRAILS
- **GPU-Safe Animation:** Never animate `top`, `left`, `width`, or `height`. Animate exclusively via `transform` and `opacity`. Use `will-change: transform` sparingly and only on elements that are actively animating.
- **Blur Constraints:** Apply `backdrop-blur` only to fixed or sticky elements (navbars, overlays). Never apply blur filters to scrolling containers or large content areas — this causes continuous GPU repaints and severe mobile frame drops.
- **Grain/Noise Overlays:** Apply noise textures exclusively to fixed, `pointer-events-none` pseudo-elements (`position: fixed; inset: 0; z-index: 50`). Never attach them to scrolling containers.
- **Z-Index Discipline:** Do not use arbitrary `z-50` or `z-[9999]`. Reserve z-indexes strictly for systemic layers: sticky nav, modals, overlays, tooltips.

## 7. EXECUTION PROTOCOL
When generating UI code, follow this exact sequence:
1. **[SILENT THOUGHT]** Roll the Variance Engine (Section 3). Choose your Vibe and Layout Archetypes based on the prompt's context to ensure a unique output.
2. **[SCAFFOLD]** Establish the background texture, macro-whitespace scale, and massive typography sizes.
3. **[ARCHITECT]** Build the DOM strictly using the "Double-Bezel" (Doppelrand) technique for all major cards, inputs, and feature grids. Use exaggerated squircle radii (`rounded-[2rem]`).
4. **[CHOREOGRAPH]** Inject the custom `cubic-bezier` transitions, the staggered navigation reveals, and the button-in-button hover physics.
5. **[OUTPUT]** Deliver flawless, pixel-perfect React/Tailwind/HTML code. Do not include basic, generic fallbacks.

## 8. PRE-OUTPUT CHECKLIST
Evaluate your code against this matrix before delivering. This is the last filter.
- [ ] No banned fonts, icons, borders, shadows, layouts, or motion patterns from Section 2 are present
- [ ] A Vibe Archetype and Layout Archetype from Section 3 were consciously selected and applied
- [ ] All major cards and containers use the Double-Bezel nested architecture (outer shell + inner core)
- [ ] CTA buttons use the Button-in-Button trailing icon pattern where applicable
- [ ] Section padding is at minimum `py-24` — the layout breathes heavily
- [ ] All transitions use custom cubic-bezier curves — no `linear` or `ease-in-out`
- [ ] Scroll entry animations are present — no element appears statically
- [ ] Layout collapses gracefully below `768px` to single-column with `w-full` and `px-4`
- [ ] All animations use only `transform` and `opacity` — no layout-triggering properties
- [ ] `backdrop-blur` is only applied to fixed/sticky elements, never to scrolling content
- [ ] The overall impression reads as "$150k agency build", not "template with nice fonts"
- [ ] Output could sit beside linear.app / vercel.com — the Tier-1 canon + claude.ai/design taste engine (Section 0) was applied, not generic defaults
