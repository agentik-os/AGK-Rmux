---
name: stitch-design-taste
description: Semantic Design System Skill for Google Stitch. Generates agent-friendly DESIGN.md files that enforce premium, anti-generic UI standards — strict typography, calibrated color, asymmetric layouts, perpetual micro-motion, and hardware-accelerated performance.
---

# Stitch Design Taste — Semantic Design System Skill

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

## Overview
This skill generates `DESIGN.md` files optimized for Google Stitch screen generation. It translates the battle-tested anti-slop frontend engineering directives into Stitch's native semantic design language — descriptive, natural-language rules paired with precise values that Stitch's AI agent can interpret to produce premium, non-generic interfaces.

The generated `DESIGN.md` serves as the **single source of truth** for prompting Stitch to generate new screens that align with a curated, high-agency design language. Stitch interprets design through **"Visual Descriptions"** supported by specific color values, typography specs, and component behaviors.

## Prerequisites
- Access to Google Stitch via [labs.google.com/stitch](https://labs.google.com/stitch)
- Optionally: Stitch MCP Server for programmatic integration with Cursor, Antigravity, or Gemini CLI

## The Goal
Generate a `DESIGN.md` file that encodes:
1. **Visual atmosphere** — the mood, density, and design philosophy
2. **Color calibration** — neutrals, accents, and banned patterns with hex codes
3. **Typographic architecture** — font stacks, scale hierarchy, and anti-patterns
4. **Component behaviors** — buttons, cards, inputs with interaction states
5. **Layout principles** — grid systems, spacing philosophy, responsive strategy
6. **Motion philosophy** — animation engine specs, spring physics, perpetual micro-interactions
7. **Anti-patterns** — explicit list of banned AI design clichés

## Analysis & Synthesis Instructions

### 1. Define the Atmosphere
Evaluate the target project's intent. Use evocative adjectives from the taste spectrum:
- **Density:** "Art Gallery Airy" (1–3) → "Daily App Balanced" (4–7) → "Cockpit Dense" (8–10)
- **Variance:** "Predictable Symmetric" (1–3) → "Offset Asymmetric" (4–7) → "Artsy Chaotic" (8–10)
- **Motion:** "Static Restrained" (1–3) → "Fluid CSS" (4–7) → "Cinematic Choreography" (8–10)

Default baseline: Variance 8, Motion 6, Density 4. Adapt dynamically based on user's vibe description.

### 2. Map the Color Palette
For each color provide: **Descriptive Name** + **Hex Code** + **Functional Role**.

**Mandatory constraints:**
- Maximum 1 accent color. Saturation below 80%
- The "AI Purple/Blue Neon" aesthetic is strictly BANNED — no purple button glows, no neon gradients
- Use absolute neutral bases (Zinc/Slate) with high-contrast singular accents
- Stick to one palette for the entire output — no warm/cool gray fluctuation
- Never use pure black (`#000000`) — use Off-Black, Zinc-950, or Charcoal

### 3. Establish Typography Rules
- **Display/Headlines:** Track-tight, controlled scale. Not screaming. Hierarchy through weight and color, not just massive size
- **Body:** Relaxed leading, max 65 characters per line
- **Font Selection:** `Inter` is BANNED for premium/creative contexts. Force unique character: `Geist`, `Outfit`, `Cabinet Grotesk`, or `Satoshi`
- **Serif Ban:** Generic serif fonts (`Times New Roman`, `Georgia`, `Garamond`, `Palatino`) are BANNED. If serif is needed for editorial/creative contexts, use only distinctive modern serifs: `Fraunces`, `Gambarino`, `Editorial New`, or `Instrument Serif`. Serif is always BANNED in dashboards or software UIs
- **Dashboard Constraint:** Use Sans-Serif pairings exclusively (`Geist` + `Geist Mono` or `Satoshi` + `JetBrains Mono`)
- **High-Density Override:** When density exceeds 7, all numbers must use Monospace

### 4. Define the Hero Section
The Hero is the first impression and must be creative, striking, and never generic:
- **Inline Image Typography:** Embed small, contextual photos or visuals directly between words or letters in the headline. Images sit inline at type-height, rounded, acting as visual punctuation. This is the signature creative technique
- **No Overlapping:** Text must never overlap images or other text. Every element occupies its own clean spatial zone
- **No Filler Text:** "Scroll to explore", "Swipe down", scroll arrow icons, bouncing chevrons are BANNED. The content should pull users in naturally
- **Asymmetric Structure:** Centered Hero layouts BANNED when variance exceeds 4
- **CTA Restraint:** Maximum one primary CTA. No secondary "Learn more" links

### 5. Describe Component Stylings
For each component type, describe shape, color, shadow depth, and interaction behavior:
- **Buttons:** Tactile push feedback on active state. No neon outer glows. No custom mouse cursors
- **Cards:** Use ONLY when elevation communicates hierarchy. Tint shadows to background hue. For high-density layouts, replace cards with border-top dividers or negative space
- **Inputs/Forms:** Label above input, helper text optional, error text below. Standard gap spacing
- **Loading States:** Skeletal loaders matching layout dimensions — no generic circular spinners
- **Empty States:** Composed compositions indicating how to populate data
- **Error States:** Clear, inline error reporting

### 6. Define Layout Principles
- No overlapping elements — every element occupies its own clear spatial zone. No absolute-positioned content stacking
- Centered Hero sections are BANNED when variance exceeds 4 — force Split Screen, Left-Aligned, or Asymmetric Whitespace
- The generic "3 equal cards horizontally" feature row is BANNED — use 2-column Zig-Zag, asymmetric grid, or horizontal scroll
- CSS Grid over Flexbox math — never use `calc()` percentage hacks
- Contain layouts using max-width constraints (e.g., 1400px centered)
- Full-height sections must use `min-h-[100dvh]` — never `h-screen` (iOS Safari catastrophic jump)

### 7. Define Responsive Rules
Every design must work across all viewports:
- **Mobile-First Collapse (< 768px):** All multi-column layouts collapse to single column. No exceptions
- **No Horizontal Scroll:** Horizontal overflow on mobile is a critical failure
- **Typography Scaling:** Headlines scale via `clamp()`. Body text minimum `1rem`/`14px`
- **Touch Targets:** All interactive elements minimum `44px` tap target
- **Image Behavior:** Inline typography images (photos between words) stack below headline on mobile
- **Navigation:** Desktop horizontal nav collapses to clean mobile menu
- **Spacing:** Vertical section gaps reduce proportionally (`clamp(3rem, 8vw, 6rem)`)

### 8. Encode Motion Philosophy
- **Spring Physics default:** `stiffness: 100, damping: 20` — premium, weighty feel. No linear easing
- **Perpetual Micro-Interactions:** Every active component should have an infinite loop state (Pulse, Typewriter, Float, Shimmer)
- **Staggered Orchestration:** Never mount lists instantly — use cascade delays for waterfall reveals
- **Performance:** Animate exclusively via `transform` and `opacity`. Never animate `top`, `left`, `width`, `height`. Grain/noise filters on fixed pseudo-elements only

### 9. List Anti-Patterns (AI Tells)
Encode these as explicit "NEVER DO" rules in the DESIGN.md:
- No emojis anywhere
- No `Inter` font
- No generic serif fonts (`Times New Roman`, `Georgia`, `Garamond`) — distinctive modern serifs only if needed
- No pure black (`#000000`)
- No neon/outer glow shadows
- No oversaturated accents
- No excessive gradient text on large headers
- No custom mouse cursors
- No overlapping elements — clean spatial separation always
- No 3-column equal card layouts
- No generic names ("John Doe", "Acme", "Nexus")
- No fake round numbers (`99.99%`, `50%`)
- No AI copywriting clichés ("Elevate", "Seamless", "Unleash", "Next-Gen")
- No filler UI text: "Scroll to explore", "Swipe down", scroll arrows, bouncing chevrons
- No broken Unsplash links — use `picsum.photos` or SVG avatars
- No centered Hero sections (for high-variance projects)

## Output Format (DESIGN.md Structure)

```markdown
# Design System: [Project Title]

## 1. Visual Theme & Atmosphere
(Evocative description of the mood, density, variance, and motion intensity.
Example: "A restrained, gallery-airy interface with confident asymmetric layouts
and fluid spring-physics motion. The atmosphere is clinical yet warm — like a
well-lit architecture studio.")

## 2. Color Palette & Roles
- **Canvas White** (#F9FAFB) — Primary background surface
- **Pure Surface** (#FFFFFF) — Card and container fill
- **Charcoal Ink** (#18181B) — Primary text, Zinc-950 depth
- **Muted Steel** (#71717A) — Secondary text, descriptions, metadata
- **Whisper Border** (rgba(226,232,240,0.5)) — Card borders, 1px structural lines
- **[Accent Name]** (#XXXXXX) — Single accent for CTAs, active states, focus rings
(Max 1 accent. Saturation < 80%. No purple/neon.)

## 3. Typography Rules
- **Display:** [Font Name] — Track-tight, controlled scale, weight-driven hierarchy
- **Body:** [Font Name] — Relaxed leading, 65ch max-width, neutral secondary color
- **Mono:** [Font Name] — For code, metadata, timestamps, high-density numbers
- **Banned:** Inter, generic system fonts for premium contexts. Serif fonts banned in dashboards.

## 4. Component Stylings
* **Buttons:** Flat, no outer glow. Tactile -1px translate on active. Accent fill for primary, ghost/outline for secondary.
* **Cards:** Generously rounded corners (2.5rem). Diffused whisper shadow. Used only when elevation serves hierarchy. High-density: replace with border-top dividers.
* **Inputs:** Label above, error below. Focus ring in accent color. No floating labels.
* **Loaders:** Skeletal shimmer matching exact layout dimensions. No circular spinners.
* **Empty States:** Composed, illustrated compositions — not just "No data" text.

## 5. Layout Principles
(Grid-first responsive architecture. Asymmetric splits for Hero sections.
Strict single-column collapse below 768px. Max-width containment.
No flexbox percentage math. Generous internal padding.)

## 6. Motion & Interaction
(Spring physics for all interactive elements. Staggered cascade reveals.
Perpetual micro-loops on active dashboard components. Hardware-accelerated
transforms only. Isolated Client Components for CPU-heavy animations.)

## 7. Anti-Patterns (Banned)
(Explicit list of forbidden patterns: no emojis, no Inter, no pure black,
no neon glows, no 3-column equal grids, no AI copywriting clichés,
no generic placeholder names, no broken image links.)
```

## Best Practices
- **Be Descriptive:** "Deep Charcoal Ink (#18181B)" — not just "dark text"
- **Be Functional:** Explain what each element is used for
- **Be Consistent:** Same terminology throughout the document
- **Be Precise:** Include exact hex codes, rem values, pixel values in parentheses
- **Be Opinionated:** This is not a neutral template — it enforces a specific, premium aesthetic

## Tips for Success
1. Start with the atmosphere — understand the vibe before detailing tokens
2. Look for patterns — identify consistent spacing, sizing, and styling
3. Think semantically — name colors by purpose, not just appearance
4. Consider hierarchy — document how visual weight communicates importance
5. Encode the bans — anti-patterns are as important as the rules themselves

## Common Pitfalls to Avoid
- Using technical jargon without translation ("rounded-xl" instead of "generously rounded corners")
- Omitting hex codes or using only descriptive names
- Forgetting functional roles of design elements
- Being too vague in atmosphere descriptions
- Ignoring the anti-pattern list — these are what make the output premium
- Defaulting to generic "safe" designs instead of enforcing the curated aesthetic
