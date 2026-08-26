---
name: brand-identity
description: >
  OmegaOS-shipped Complete Brand Identity System v2.1. From discovery to a deployed
  interactive brand book: Kapferer Brand Identity Prism, 2-3 switchable variants,
  typography, colors, voice, logo direction, AI prompts, design tokens, component
  previews (Next.js brand book on Vercel). Use when user says "/omg-brand-identity",
  "/brand-identity", "brand book", "brand system", or "visual identity". Step 3 of the
  OmegaOS new-project pipeline: run /omg-vision then /omg-prd first, follow with
  /omg-planner then `omega plan-run`.
triggers: ["omg-brand-identity", "brand-identity", "brand book", "brand system", "visual identity", "brand kit"]
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Task", "ToolSearch", "WebSearch", "WebFetch", "Skill"]
---

# /brand-identity - Complete Brand Identity System v2.1

<brand-identity-banner>
```
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║   ██████╗ ██████╗  █████╗ ███╗   ██╗██████╗                     ║
║   ██╔══██╗██╔══██╗██╔══██╗████╗  ██║██╔══██╗                    ║
║   ██████╔╝██████╔╝███████║██╔██╗ ██║██║  ██║                    ║
║   ██╔══██╗██╔══██╗██╔══██║██║╚██╗██║██║  ██║                    ║
║   ██████╔╝██║  ██║██║  ██║██║ ╚████║██████╔╝                    ║
║   ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝                    ║
║                                                                  ║
║   Brand Identity System v2.1                                     ║
║   "From soul to screen. Every pixel intentional."                ║
║                                                                  ║
║   🔍 Discovery → 🧠 Strategy → 🎨 Design → 🏗️ Build → 🚀 Deploy ║
║                                                                  ║
║   by Dafnck Studio                                               ║
╚══════════════════════════════════════════════════════════════════╝
```
</brand-identity-banner>

**Display the banner above when starting.**

---

## CRITICAL: 100% AUTONOMOUS EXECUTION

**This skill runs FULLY AUTONOMOUSLY from Phase 0 through Phase 7.**

Rules:
- **NEVER ask "should I continue to the next phase?"** — Always auto-proceed.
- **NEVER ask "ready for Phase X?"** — Just do it.
- **NEVER ask permission between phases** — The only acceptable questions are branding/discovery questions (Phase 0-1) that gather actual project information from the user.
- **After Phase 1 (intake) is complete, ALL remaining phases execute without any user interaction.**
- Phase transitions are silent: finish one phase → start the next immediately.
- If something fails (build error, deploy error), fix it yourself and continue.

**Acceptable questions:** "What's the product?" "Who's the audience?" "Color preferences?" (Phase 0-1 intake)
**FORBIDDEN questions:** "Should I proceed?" "Ready for Phase 3?" "Want me to continue?" "Shall I deploy?"

---

## OVERVIEW

This skill creates a **complete, interactive Brand Identity System** deployed as a Next.js website on Vercel. The client receives a URL with their full brand book featuring **2-3 switchable brand variants** they can explore and approve.

### What Gets Created

| Deliverable | Description |
|-------------|-------------|
| **Kapferer Prism** | 6-facet brand identity prism (Physique, Personality, Culture, Relationship, Reflection, Self-Image) |
| **Emotional Core** | Who, how they feel, anti-positioning |
| **Brand Personality** | Persona, adjectives, manifesto, archetypes |
| **2-3 Brand Variants** | Complete identity systems (colors, type, voice, imagery) |
| **Color Palettes** | 5-7 colors per variant with emotional intent + oklch + hex + hsl |
| **Typography System** | 2-3 font pairings per variant with live specimens |
| **Voice & Tone** | Microcopy examples, do/don't, tone spectrum radar chart |
| **Visual Language** | Textures, materials, eras, photography/illustration style |
| **Logo Direction** | 3 logo concepts with rationale (wordmark/icon/combo) |
| **Anti-Pattern List** | 10+ specific things the brand must NEVER do |
| **AI Image Prompts** | 7+ ready-to-use prompts per variant (Midjourney/Flux) |
| **Design Tokens** | CSS custom properties + Tailwind config export |
| **Component Preview** | Live buttons, cards, inputs in each variant |
| **Spacing & Grid** | 4px/8px base grid, spacing scale, layout system |
| **Motion Guidelines** | Easing curves, durations, transition rules |
| **Dark Mode** | Full dark mode treatment per variant |
| **Social Media Kit** | Profile/cover specs, post dimensions |
| **Favicon + OG** | SVG favicon, Open Graph image template |
| **Dev Handoff** | Selected variant → CLAUDE.md brand section for /team |
| **Interactive Brand Book** | Next.js site with variant switcher → Vercel URL |

---

## CRITICAL: PROJECT DIRECTORY CONVENTION

**ALWAYS ASK the user: "Is this a personal project or a client project?"**

Resolve `<projects_dir>` from `~/.omega/config.toml` (`projects_dir`). Never
hardcode `~/VibeCoding` or a maintainer home path.

| Answer | Directory | Example |
|--------|-----------|---------|
| **Personal** | `<projects_dir>/side-business/[ProjectName]/` (fallback: existing `work/`) | `side-business/Atma/` |
| **Client** | `<projects_dir>/customers/[ProjectName]/` (fallback: existing `clients/`) | `customers/Resonant/` |

This question is asked during Phase 1 intake. NEVER assume — always ask.

```bash
# Personal project
$PROJECTS_DIR/side-business/Atma/

# Client project
$PROJECTS_DIR/customers/Resonant/

# The brand book Next.js project goes INSIDE
$PROJECTS_DIR/<category>/[ProjectName]/brand-book/
```

### Directory Structure

```
$PROJECTS_DIR/<category>/[ProjectName]/
├── brand-book/                   # Next.js brand book site (deployed to Vercel)
│   ├── src/
│   ├── public/
│   ├── package.json
│   └── CLAUDE.md
├── docs/                         # All strategy documents
│   ├── DISCOVERY-BRIEF.md
│   ├── STRATEGY.md
│   ├── CREATIVE-DIRECTION.md
│   ├── AI-PROMPTS.md
│   ├── ANTI-PATTERNS.md
│   └── BRAND-SORT.md
├── exports/                      # Dev handoff files
│   ├── design-tokens.css         # CSS custom properties
│   ├── tailwind-brand.ts         # Tailwind config extension
│   ├── brand-claude-section.md   # Ready to paste in project CLAUDE.md
│   └── figma-tokens.json         # Design tokens in Figma format
├── BRAND-VARIANTS.json           # Complete variant data
└── CLAUDE.md                     # Project instructions
```

---

## UNIFIED WORKFLOW

```
/brand-identity
    │
    ▼
╔═══════════════════════════════════════════════════════════════════╗
║  PHASE 0: BRAND NAMING (optional)                                ║
║  ────────────────────────────────                                ║
║  • Only if brand name is NOT decided                             ║
║  • Brainstorm 10+ name candidates                                ║
║  • Check domain availability (WebSearch)                         ║
║  • Present shortlist with rationale                              ║
║  • User selects → proceed                                        ║
╚═══════════════════════════════════════════════════════════════════╝
    │
    ▼
╔═══════════════════════════════════════════════════════════════════╗
║  PHASE 1: INTAKE & DISCOVERY (10-30 min)                         ║
║  ────────────────────────────                                    ║
║  • Auto-check: .prd/DESIGN-SYSTEM.md, TECH-ARCHITECTURE.md,     ║
║    VISION.md → use as constraints/foundation if they exist       ║
║  • Read client docs (PRD, PDF, brief, vision)                    ║
║  • Analyze existing data sufficiency                             ║
║  • Ask targeted questions for missing info                       ║
║  • Research competitor visual identities (WebFetch)              ║
║  • Understand: product, audience, market, competitors            ║
║  • Output: DISCOVERY-BRIEF.md                                    ║
╚═══════════════════════════════════════════════════════════════════╝
    │
    ▼
╔═══════════════════════════════════════════════════════════════════╗
║  PHASE 2: STRATEGIC FOUNDATION (3 parallel agents)               ║
║  ──────────────────────────────────────────────                  ║
║  🤖 Agent 1: Emotional Core + Anti-Positioning                   ║
║  🤖 Agent 2: Brand Personality + Manifesto + Archetypes          ║
║  🤖 Agent 3: Competitor/Market Analysis + Positioning Map        ║
║  • Output: STRATEGY.md                                           ║
╚═══════════════════════════════════════════════════════════════════╝
    │
    ▼
╔═══════════════════════════════════════════════════════════════════╗
║  PHASE 3: CREATIVE DIRECTION (5 parallel agents)                 ║
║  ────────────────────────────────────────────                    ║
║  🤖 Agent 1: Color Systems (3 palettes + dark mode + a11y)       ║
║  🤖 Agent 2: Typography Research (3 type systems + scale)        ║
║  🤖 Agent 3: Visual Language + Moodboard + Motion + Spacing      ║
║  🤖 Agent 4: Voice & Tone + Microcopy + Social Voice             ║
║  🤖 Agent 5: Logo Direction (3 concepts + favicon + OG)          ║
║  • Output: CREATIVE-DIRECTION.md                                 ║
╚═══════════════════════════════════════════════════════════════════╝
    │
    ▼
╔═══════════════════════════════════════════════════════════════════╗
║  PHASE 4: VARIANT ASSEMBLY + DESIGN TOKENS (sequential)          ║
║  ─────────────────────────────────────────────────               ║
║  • Combine Phase 2 + Phase 3 into 2-3 complete brand variants    ║
║  • Each variant = colors + type + voice + imagery + logo          ║
║  • Generate CSS custom properties per variant                    ║
║  • Generate Tailwind config extension per variant                ║
║  • Generate spacing scale + grid system                          ║
║  • Name each variant (e.g., "Eclipse", "Horizon", "Pulse")       ║
║  • Output: BRAND-VARIANTS.json + exports/                        ║
╚═══════════════════════════════════════════════════════════════════╝
    │
    ▼
╔═══════════════════════════════════════════════════════════════════╗
║  PHASE 5: AI PROMPT PACK + ANTI-PATTERNS (2 parallel agents)     ║
║  ──────────────────────────────────────────────────              ║
║  🤖 Agent 1: AI Image Prompting Starter Pack (7+ per variant)    ║
║  🤖 Agent 2: Anti-Pattern List (10+ specific rules)              ║
║  • Output: AI-PROMPTS.md + ANTI-PATTERNS.md                      ║
╚═══════════════════════════════════════════════════════════════════╝
    │
    ▼
╔═══════════════════════════════════════════════════════════════════╗
║  PHASE 6: BUILD INTERACTIVE BRAND BOOK (Next.js)                 ║
║  ──────────────────────────────────────────────                  ║
║  🤖 Agent: Next.js Developer (nextjs-developer)                  ║
║  • Scaffold Next.js project with Tailwind + shadcn/ui            ║
║  • Build brand book pages with variant switcher                  ║
║  • Typography live specimens, color swatches with copy buttons   ║
║  • Component preview (buttons, cards, inputs per variant)        ║
║  • Motion guidelines with live animation demos                   ║
║  • Favicon + OG meta for sharing                                 ║
║  • Dark mode toggle per variant                                  ║
║  • Responsive (mobile + desktop)                                 ║
║  • Footer: "Created by Dafnck Studio"                            ║
║  • Output: Complete Next.js project                              ║
╚═══════════════════════════════════════════════════════════════════╝
    │
    ▼
╔═══════════════════════════════════════════════════════════════════╗
║  PHASE 7: DEPLOY + DEV HANDOFF                                   ║
║  ────────────────────────                                        ║
║  • Build check (0 errors)                                        ║
║  • Deploy to Vercel (Dafnck Studio account)                      ║
║  • Generate dev handoff exports                                  ║
║  • Generate BRAND-SORT.md (quick reference)                      ║
║  • Return shareable URL to user                                  ║
║  • 🎉 Brand book live! Share with client.                        ║
╚═══════════════════════════════════════════════════════════════════╝
```

---

## PHASE 0: BRAND NAMING (Optional)

**Only run if the brand name is NOT decided yet.**

When the user says "the name isn't decided" or provides no name:

1. **Research the space** — Use WebSearch to understand naming conventions in the industry
2. **Generate 10-15 name candidates** across categories:

| Category | Description | Example |
|----------|-------------|---------|
| **Descriptive** | Says what it does | "TaskFlow", "MindMap" |
| **Evocative** | Suggests a feeling | "Atma", "Lume", "Verve" |
| **Abstract** | Made-up or unusual | "Kova", "Zeph", "Nōva" |
| **Metaphorical** | Borrows from nature/culture | "Compass", "Forge", "Atlas" |
| **Acronym/Blend** | Combined words | "OptiVerse", "AgentIQ" |

3. **For each candidate, check:**
   - Domain availability (.com, .io, .app) via WebSearch
   - Social handle availability (quick search)
   - Trademark conflicts (basic search)
   - Pronunciation in multiple languages
   - Memorability score (1-10)

4. **Present shortlist of 5** with rationale
5. **User selects → proceed to Phase 1**

---

## PHASE 1: INTAKE & DISCOVERY

### Step 1.0: PRD & Vision Intake (Automatic)

**Before asking ANY questions, silently check for existing project documentation.**

```bash
# Check for PRD design system, tech architecture, and vision doc
ls -la .prd/DESIGN-SYSTEM.md .prd/TECH-ARCHITECTURE.md VISION.md 2>/dev/null
```

| File | If Found | How It's Used |
|------|----------|---------------|
| `.prd/DESIGN-SYSTEM.md` | **Read it.** Extract color preferences, typography suggestions, component patterns, spacing rules. These become **hard constraints** for Phase 3 (Creative Direction) — not just inspiration, actual requirements. |
| `.prd/TECH-ARCHITECTURE.md` | **Read it.** Identify the tech stack (Tailwind vs vanilla CSS, design token format needs, component library in use). Informs Phase 4 design token export format (e.g., Tailwind config vs CSS custom properties vs both). |
| `VISION.md` | **Read it.** This is the output of `/vision` — emotional positioning, brand soul, target feelings, anti-positioning. Use it as the **foundation** for Phase 2 (Strategic Foundation) instead of rediscovering from scratch. Skip redundant discovery questions already answered in VISION.md. |

**Behavior:**
- If **VISION.md** exists: Pre-fill emotional core, anti-positioning, and brand personality seeds from it. Phase 2 agents refine rather than start from zero. Discovery interview (Step 1.3) skips questions already answered.
- If **DESIGN-SYSTEM.md** exists: Lock in any specified colors, fonts, or spacing as constraints. Creative Direction agents (Phase 3) work within these constraints rather than exploring freely.
- If **TECH-ARCHITECTURE.md** exists: Note the stack for token export format decisions in Phase 4. If Tailwind is used, prioritize `tailwind-brand.ts`. If CSS-only, prioritize `design-tokens.css`.
- If none exist: Proceed normally — no impact on workflow.

Log what was found in the DISCOVERY-BRIEF.md under a `## Prior Documentation` section.

### Step 1.1: Data Assessment

When user provides input (PRD, PDF, brief, vision doc, or description):

1. **Read all provided documents** using Read tool (PDF native) or read-documents.sh (Word)
2. **Extract key information:**

| Data Point | Required? | Source |
|------------|-----------|--------|
| Brand Name | YES | Client doc or Phase 0 |
| Product Description | YES | Client doc or ask |
| Target Audience | YES | Client doc or ask |
| Business Model | YES | Client doc or ask |
| Competitors | YES | Client doc or research |
| Existing Brand Assets | NICE | Client files |
| Industry/Sector | YES | Infer or ask |
| Color Preferences | NICE | Ask |
| Color Avoidances | NICE | Ask |
| Reference Brands | NICE | Ask |
| Aesthetic Era | NICE | Ask |
| Logo Preference | NICE | Ask |

3. **Assess data sufficiency:**
   - **Sufficient (>70% data points)** → Proceed directly to Phase 2 (no confirmation needed)
   - **Partial (40-70%)** → Ask 3-5 targeted questions, then proceed immediately
   - **Insufficient (<40%)** → Run full discovery interview, then proceed immediately

### Step 1.2: Competitor Visual Research

**ALWAYS do this, even with sufficient data.**

Use WebSearch + WebFetch to analyze 3-5 competitor websites:
- Screenshot or describe their visual identity
- Note: color schemes, typography, tone, imagery style
- Identify what they do well and what they do poorly
- Map the visual landscape to find whitespace

### Step 1.3: Discovery Interview (if needed)

Ask these questions in order, grouping 2-3 per message to move fast:

**Block 1 — Identity:**
1. What's the product in one sentence?
2. What problem does it solve? (Emotional, not just functional)
3. Who is this for? Be specific — not "everyone"

**Block 2 — Feeling:**
4. How should someone FEEL when they open the app/site? (3 emotions max)
5. How should they feel when they CLOSE it?
6. What existing product/brand has a feeling you admire? (Even from different industry)

**Block 3 — Positioning:**
7. What is this product absolutely NOT? (3 things it must never become)
8. Who are the direct competitors, and what do they get wrong?
9. If this brand were a person, describe them (age, style, music taste, vibe)

**Block 4 — Visual Direction:**
10. Any color preferences or colors to AVOID?
11. What aesthetic era appeals? (Minimal 2020s? Bold 90s? Retro-futurism? Y2K?)
12. Logo preference: initials/monogram, icon/symbol, wordmark, or combination?

**Block 5 — Business:**
13. Business description in one line (what you sell to whom)
14. Revenue model (subscription, one-time, freemium, etc.)
15. What makes you different from competitors in ONE word?

**Block 6 — Project Setup:**
16. Is this a personal/AgentikOS project or a CLIENT project?
    → Determines directory: `work/` vs `clients/`

### Step 1.4: Output DISCOVERY-BRIEF.md

```markdown
# Discovery Brief — [Brand Name]

## Product
[One paragraph — what it is, what it does, why it exists]

## Target Audience
[Specific emotional description, NOT demographics. Who are they emotionally?]

## Market Position
[Where this sits vs competitors. What's the whitespace?]

## Competitor Visual Landscape
[Summary of 3-5 competitor visual identities. What's overdone? What's missing?]

## Emotional Direction
[3 core emotions / desired feelings. Opening feeling + closing feeling.]

## Anti-Positioning
[What this is NOT. Specific traps to avoid.]

## Visual Signals
[Any expressed preferences: colors, eras, references, avoidances]

## Business Model
[Revenue model + key differentiator]

## Prior Documentation
[List any .prd/DESIGN-SYSTEM.md, .prd/TECH-ARCHITECTURE.md, or VISION.md found.
Summarize key constraints extracted from each. "None found" if none exist.]

## Logo Direction
[Preference: wordmark, monogram, icon, combination, or undecided]
```

Save to `[ProjectDir]/docs/DISCOVERY-BRIEF.md`

---

## KAPFERER BRAND IDENTITY PRISM (MANDATORY)

**The Kapferer Prism is a MANDATORY framework for every brand identity project.** It must be filled during Phase 2 (Strategic Foundation) and displayed as a dedicated page in the brand book.

### The 6 Facets

| Facet | Description | Question to Answer |
|-------|-------------|-------------------|
| **Physique** | The tangible, physical qualities of the brand — visual identity, colors, logo, product look, distinctive visual features | "What does the brand LOOK like? What physical traits make it instantly recognizable?" |
| **Personality** | The brand's character and tone of voice — if the brand were a person, how would they speak, behave, express themselves? | "If this brand walked into a room, what impression would they make? How do they talk?" |
| **Culture** | The values and principles the brand stands for — the beliefs, ideology, and mission that drive every decision | "What does this brand fundamentally believe in? What values would it never compromise on?" |
| **Relationship** | The type of relationship between brand and customer — mentor, friend, guide, partner, confidant, accomplice | "What role does this brand play in the customer's life? How do they interact?" |
| **Reflection** | The outward mirror — how the target customer sees themselves reflected in the brand, the idealized user image the brand projects | "Who does the brand's communication portray as its user? What aspirational image does it project?" |
| **Self-Image** | The inward mirror — how the customer's internal self-perception changes when using the brand | "When someone uses this brand, how do they feel about THEMSELVES? What internal dialogue changes?" |

### Template for Each Facet

When filling the prism during Phase 2, use this structure for each facet:

```markdown
### [Facet Name]

**Core statement:** [One sentence that captures this facet]

**Details:**
- [3-5 specific, vivid descriptions — no generic startup jargon]

**Manifests as:**
- In design: [How this facet shows up visually]
- In copy: [How this facet shows up in writing]
- In UX: [How this facet shows up in interactions]

**Anti-pattern:** [What this facet is explicitly NOT]
```

### Example (Premium Coaching Brand)

```
PHYSIQUE: Rose-gold tones, aviation-inspired typography, warm lighting, tactile textures
PERSONALITY: A sharp-dressed mentor who speaks with authority AND warmth — never cold, never casual
CULTURE: Excellence without arrogance, human connection in a digital world, earned trust
RELATIONSHIP: A copilot — not a teacher above you, not a friend beside you, but a trusted navigator
REFLECTION: A successful dirigeant who has "made it" but still invests in growth — not a struggling beginner
SELF-IMAGE: "I'm the kind of leader who invests in strategic thinking, not just daily firefighting"
```

### Brand Book Page: /prism

The prism MUST be generated as a page in the brand book at route `/prism`. The page should display:
- A visual hexagonal or diamond-shaped prism diagram with the 6 facets
- Each facet as an expandable card with the full details
- Two axes labeled: "Externalization" (Physique, Relationship, Reflection) vs "Internalization" (Personality, Culture, Self-Image)
- Two poles labeled: "Sender/Brand" (Physique, Personality) vs "Receiver/Customer" (Reflection, Self-Image)
- The prism should be variant-aware (colors adapt to active variant)

---

## PHASE 2: STRATEGIC FOUNDATION

**Launch 3 agents IN PARALLEL using Task tool:**

### Agent 1: Emotional Core Strategist

```
subagent_type: "general-purpose"
model: "opus"
prompt: |
  You are a senior brand strategist specializing in emotional positioning.

  Given this discovery brief:
  ---
  [INSERT FULL DISCOVERY-BRIEF.md CONTENT]
  ---

  Create the EMOTIONAL CORE:

  1. **Primary Emotions** (3 max):
     - For each: name, intensity (1-10), description, UI manifestation
     - How this emotion shows up in: color, typography, spacing, motion, copy

  2. **Emotional Journey Map:**
     - First impression (0-3 seconds): what they feel, what they see
     - Exploration phase (first 5 minutes): curiosity triggers, aha moments
     - Regular use (daily/weekly): comfort, reliability, delight moments
     - Advocacy moment (when they tell someone): pride, identity, story they tell

  3. **Anti-Positioning Matrix:**
     - List 5+ specific things this brand is NOT
     - For each: the trap, why it's wrong, what to do instead
     - Reference REAL products/brands by name

  4. **Internal Compass:**
     - One sentence that settles ANY design debate
     - Must be specific, opinionated, and memorable
     - Test: could a designer use this to make a decision alone?

  5. **Emotional Design Principles** (5):
     - Format: "[Principle] over [Counter-Principle]"
     - For each: one-sentence explanation + concrete UI example

  6. **Kapferer Brand Identity Prism** (MANDATORY):
     Fill ALL 6 facets with specific, vivid descriptions:
     - **Physique:** Tangible visual qualities (colors, shapes, textures, distinctive features)
     - **Personality:** Character/tone of voice (how the brand speaks, behaves, expresses itself)
     - **Culture:** Core values and beliefs that drive every brand decision
     - **Relationship:** The type of relationship with the customer (mentor, guide, partner, copilot, etc.)
     - **Reflection:** The outward mirror — how customers see themselves reflected in the brand
     - **Self-Image:** The inward mirror — how customers' self-perception changes when using the brand
     For each facet: core statement + 3-5 specific details + how it manifests in design/copy/UX + anti-pattern

  Format as structured markdown. Be EXTREMELY specific — no startup jargon.
  Reference real brands, real products, real design patterns.
```

### Agent 2: Brand Personality Architect

```
subagent_type: "general-purpose"
model: "opus"
prompt: |
  You are a creative director defining brand personality.

  Given this discovery brief:
  ---
  [INSERT FULL DISCOVERY-BRIEF.md CONTENT]
  ---

  Create the BRAND PERSONALITY:

  1. **Brand as a Person:**
     - Age, gender expression, energy level
     - What music they listen to (name specific artists, albums, playlists)
     - What they wear (specific brands, styles, not "casual")
     - Their apartment/workspace (describe the room: furniture, lighting, objects)
     - Their favorite apps (3-5 specific apps they actually use)
     - Their bookshelf (3-5 specific books)
     - Their Netflix queue (3-5 specific films/shows)
     - How they speak (cadence, vocabulary, humor style, catchphrases)
     - Their morning routine (what do they do first?)

  2. **5 Brand Adjectives** (UNUSUAL ones — NOT modern/clean/simple/innovative):
     - For each: the adjective, what it means in this brand's context,
       how it manifests visually, how it manifests in copy

  3. **3 Cross-Industry References:**
     - Brands/aesthetics from COMPLETELY different industries
     - For each: what specifically resonates, what to borrow, what to reject
     - Be specific: "The material quality of Aesop packaging" not "Aesop"

  4. **Manifesto:**
     - "We believe that..." (one powerful sentence, max 20 words)
     - Extended manifesto (3-5 sentences, for About page)
     - Anti-manifesto: "We reject..." (one sentence)

  5. **Brand Archetypes:**
     - Primary archetype (Creator, Explorer, Sage, Hero, Magician, etc.)
     - Why this archetype fits
     - Shadow archetype (what this brand becomes if it loses its way)
     - Adjacent archetype (secondary influence)

  Format as structured markdown. Be BOLD, specific, opinionated.
  Every description should be vivid enough to cast an actor for the role.
```

### Agent 3: Market Positioning Analyst

```
subagent_type: "general-purpose"
model: "opus"
prompt: |
  You are a market positioning analyst and competitive strategist.

  Given this discovery brief:
  ---
  [INSERT FULL DISCOVERY-BRIEF.md CONTENT]
  ---

  Create the MARKET POSITIONING:

  1. **Competitive Landscape:**
     - Top 5 competitors (direct AND indirect)
     - For each: name, one-line positioning, visual identity summary
       (dominant colors, font style, tone), key weakness we exploit

  2. **Positioning Map:**
     - Define 2 key axes relevant to this market
       (e.g., Premium↔Accessible, Playful↔Serious, Technical↔Emotional)
     - Place this brand AND all 5 competitors on the map
     - Identify the WHITESPACE — the unclaimed position
     - Describe why this position is defensible

  3. **Unique Value Proposition:**
     - One sentence that NO competitor could credibly claim
     - 3 supporting proof points
     - How this UVP manifests in the visual identity

  4. **Category Design:**
     - Is this brand creating a new category or entering existing?
     - If new: name the category, define its rules
     - If existing: what existing assumption does this brand violate?

  5. **Target Audience Deep-Dive:**
     - 3 emotional archetypes (NOT demographics, NOT job titles)
     - For each: name, emotional state, life moment, "finally" trigger,
       top 3 objections, how the brand visually addresses each objection

  6. **Visual Differentiation Strategy:**
     - What the competitors all do visually (the cliché)
     - What THIS brand does differently (the break)
     - The one visual element that will make this brand instantly recognizable

  Format as structured markdown. Use REAL competitor names.
```

**After all 3 agents complete → Read outputs → Synthesize into STRATEGY.md**

---

## PHASE 3: CREATIVE DIRECTION

**Launch 5 agents IN PARALLEL using Task tool:**

### Agent 1: Color Systems Designer

```
subagent_type: "general-purpose"
model: "opus"
prompt: |
  You are a color theory expert and brand designer.

  Given this brand strategy:
  ---
  [INSERT STRATEGY.md CONTENT]
  ---

  Create 3 DISTINCT COLOR PALETTES (each for a different brand variant):

  For EACH palette:

  1. **Palette Name** (evocative name, not "Option A")

  2. **Colors (7 per palette):**

     For each color provide ALL of these:
     - Creative name (e.g., "Midnight Ink", NOT "Dark Blue")
     - Hex code (e.g., #1a1a2e)
     - oklch value (e.g., oklch(0.15 0.02 280))
     - HSL value (e.g., hsl(240, 28%, 14%))
     - RGB value (e.g., rgb(26, 26, 46))
     - Role: Primary / Secondary / Accent / Background / Surface / Muted / Border / Destructive
     - Emotional intent: what this color communicates in one sentence

     Required roles per palette:
     - Primary (main brand color)
     - Secondary (supporting brand color)
     - Accent (attention-grabbing, used sparingly)
     - Background (page background)
     - Surface (card/component background)
     - Muted (subtle backgrounds, disabled states)
     - Border (dividers, outlines)

  3. **Dark Mode Palette:**
     - For EACH of the 7 colors, provide the dark mode equivalent
     - Same format: name, hex, oklch, hsl, rgb, role
     - Dark mode is NOT just "invert" — it's a thoughtful adaptation

  4. **Color Relationships:**
     - Primary text on Background: contrast ratio (must be >= 4.5:1 for WCAG AA)
     - Primary text on Surface: contrast ratio
     - Accent on Background: contrast ratio
     - Which combinations are SAFE for body text
     - Which combinations are ONLY for large text/headings

  5. **Accessibility:**
     - Color-blind safe? (deuteranopia, protanopia, tritanopia)
     - If not fully safe: which pairs to avoid, alternatives
     - High-contrast mode adaptation

  6. **Color Psychology:**
     - Why this palette was chosen
     - Cultural considerations (any color meanings that vary by culture)
     - Industry context (does this break conventions? follow them?)

  7. **Usage Rules:**
     - Accent color: max % of screen surface (e.g., "5-10% max")
     - Primary: where it appears (buttons, links, headers?)
     - Background vs Surface: when to use which
     - Gradient rules: allowed? forbidden? direction constraints?

  REQUIREMENTS:
  - Palettes must feel DISTINCT (not just hue shifts of each other)
  - One palette should be warm, one cool, one neutral/unexpected
  - ALL text combinations must pass WCAG AA (4.5:1 for normal text)
  - Provide oklch values for modern Tailwind v4 compatibility
  - Include an "unexpected" color in each palette
  - For PREMIUM-positioned brands: include a gold/champagne accent color in at least one variant
    (e.g., #C9A96E warm gold, #D4AF37 classic gold) — gold works for borders, icon accents,
    hover states, premium badges, and CTA highlights

  Format as structured markdown with JSON code blocks for machine-readable data.
```

### Agent 2: Typography Researcher

```
subagent_type: "general-purpose"
model: "opus"
prompt: |
  You are a typography expert and brand designer.

  Given this brand strategy:
  ---
  [INSERT STRATEGY.md CONTENT]
  ---

  Research and propose 3 DISTINCT TYPOGRAPHY SYSTEMS:

  For EACH system:

  1. **System Name** (e.g., "Eclipse" — matches color variant name)

  2. **Display/Heading Font:**
     - Font name (MUST be on Google Fonts)
     - Google Fonts URL (exact link)
     - Weights: which weights to load (e.g., 500, 600, 700, 800)
     - Personality: what this font communicates (3 adjectives)
     - Why: specific reason this font was chosen for this brand
     - Character: what makes this font DISTINCTIVE vs generic alternatives
     - Style: serif, sans-serif, slab, display, mono, handwritten?

  3. **Body/Text Font:**
     - Font name (Google Fonts)
     - Google Fonts URL
     - Weights: 400, 500, (600 optional)
     - Readability: x-height, letter spacing, line height notes
     - Pairing rationale: WHY this body + display combination works
       (contrast, harmony, shared geometry, era compatibility)

  4. **Monospace/Accent Font** (required if product has code/technical elements):
     - Font name + Google Fonts URL
     - When to use: code blocks, labels, metadata, timestamps

  5. **Type Scale:**
     Base size: 16px (1rem)
     Scale ratio: (e.g., 1.25 Major Third, 1.333 Perfect Fourth)

     | Name | Size | Line Height | Letter Spacing | Weight | Use Case |
     |------|------|-------------|----------------|--------|----------|
     | xs | ?px | ? | ? | 400 | Captions, metadata |
     | sm | ?px | ? | ? | 400 | Secondary text |
     | base | 16px | 1.6 | 0 | 400 | Body text |
     | lg | ?px | ? | ? | 500 | Lead paragraphs |
     | xl | ?px | ? | ? | 600 | Section headers |
     | 2xl | ?px | ? | ? | 600 | Page sub-headers |
     | 3xl | ?px | ? | ? | 700 | Page titles |
     | 4xl | ?px | ? | ? | 700 | Hero sub-headlines |
     | 5xl | ?px | ? | ? | 800 | Hero headlines |

  6. **Typography Rules:**
     - Max line length: ? characters (optimal reading width)
     - Paragraph spacing: ? (relative to font size)
     - Heading capitalization: Title Case / Sentence case / UPPERCASE / lowercase?
     - Display font: used for headings only? Also buttons? Also nav?
     - Body font: used for body only? Also UI labels? Also inputs?
     - Number style: tabular or proportional? Oldstyle or lining?
     - Ligatures: on or off?
     - Font feature settings: any special OpenType features to enable?

  7. **Google Fonts Import Code:**
     ```html
     <link href="https://fonts.googleapis.com/css2?family=..." rel="stylesheet">
     ```
     AND next/font import:
     ```typescript
     import { FontName } from 'next/font/google'
     const heading = FontName({ subsets: ['latin'], weight: ['600', '700'] })
     ```

  REQUIREMENTS:
  - ALL fonts must be on Google Fonts (free, no licensing issues)
  - BANNED fonts: Inter, Roboto, Arial, Open Sans, Lato, Montserrat, Poppins,
    Nunito, Raleway, Source Sans Pro (too generic, too overused)
  - Each system must feel GENUINELY different (serif vs sans, geometric vs humanist, etc.)
  - Consider character: Fraunces, Instrument Serif, Playfair Display, DM Serif,
    Clash Display, Syne, Epilogue, Urbanist, Plus Jakarta Sans, Instrument Sans,
    Bricolage Grotesque, Outfit, Figtree, Geist, Onest, Lexend, Space Mono,
    JetBrains Mono, Fira Code, IBM Plex Mono, etc.
  - Test: would a designer be excited to use this combination?

  Format as structured markdown with JSON for machine-readable type scale.
```

### Agent 3: Visual Language + Motion + Spacing Architect

```
subagent_type: "general-purpose"
model: "opus"
prompt: |
  You are a visual director, motion designer, and design systems architect.

  Given this brand strategy:
  ---
  [INSERT STRATEGY.md CONTENT]
  ---

  Create the VISUAL LANGUAGE, MOTION SYSTEM, and SPACING SYSTEM for 3 brand variants.

  ## PART A: VISUAL LANGUAGE (per variant)

  For EACH variant:

  1. **Visual Direction Name**

  2. **Photography Style:**
     - Subject matter: people? objects? abstract? architecture? nature?
     - Lighting: natural, studio, dramatic, golden hour, flat, chiaroscuro?
     - Color treatment: vivid, muted, desaturated, monochrome, warm shift, cool shift?
     - Composition: centered, rule of thirds, asymmetric, close-up, wide, overhead?
     - Post-processing: grain? clean? film emulation? high contrast?
     - DO: [3 specific examples of good photography for this brand]
     - DON'T: [3 specific examples of wrong photography]

  3. **Illustration Style:**
     - Approach: geometric, organic, hand-drawn, 3D render, flat vector, isometric?
     - Line weight: thin (1px), medium (1.5px), bold (2px+), variable?
     - Detail level: minimal, moderate, intricate?
     - Color: flat fill, gradients, outlined, monochrome, full palette?
     - Animation: static, micro-animated (hover/scroll), full motion?

  4. **Texture & Materials:**
     - Surface: grain, smooth, paper, glass, concrete, fabric, metallic?
     - Background: solid, gradient, noise, pattern, subtle texture, bold pattern?
     - Border/edge: sharp, slightly rounded (4px), rounded (8px), pill, organic?
     - Shadow: none, subtle (0 1px 2px), medium, dramatic, colored shadow?
     - Opacity/Glass: none, subtle blur, glassmorphism, neumorphism?

  5. **Era & Cultural References:**
     - Time period (be specific: "1972 Swiss poster design" not "retro")
     - Art movements (Bauhaus, Swiss, Memphis, Brutalism, Art Deco, Minimalism...)
     - Cultural references (films, music, architecture, fashion, subcultures)
     - Digital-era references (specific websites, apps, or interfaces to reference)

  6. **Imagery Tone of Voice:**
     - Emotions images should evoke (3 keywords)
     - The story images tell (one sentence)
     - What is NEVER shown (5 specific things)
     - Diversity/representation guidelines

  7. **Moodboard Keywords** (10 per variant):
     - Specific enough to get relevant results on Pinterest/Dribbble/Behance
     - Mix of abstract and concrete terms

  8. **Icon Style:**
     - Weight: 1px, 1.5px, 2px?
     - Style: outline, filled, duotone, custom?
     - Corner: sharp, rounded (2px), fully round?
     - Size grid: 16px, 20px, 24px?
     - Hover animation: none, scale, color shift, stroke draw?

  ## PART B: MOTION SYSTEM (shared across variants, adapted per variant)

  1. **Easing Curves:**
     | Name | CSS Value | Use Case |
     |------|-----------|----------|
     | ease-default | cubic-bezier(0.4, 0, 0.2, 1) | General transitions |
     | ease-in | cubic-bezier(0.4, 0, 1, 1) | Elements leaving |
     | ease-out | cubic-bezier(0, 0, 0.2, 1) | Elements entering |
     | ease-bounce | cubic-bezier(0.34, 1.56, 0.64, 1) | Playful interactions |
     | ease-spring | cubic-bezier(0.175, 0.885, 0.32, 1.275) | Satisfying clicks |

  2. **Duration Scale:**
     | Name | Duration | Use Case |
     |------|----------|----------|
     | instant | 0ms | Immediate feedback |
     | fast | 100ms | Micro-interactions (hover, focus) |
     | normal | 200ms | Standard transitions |
     | slow | 300ms | Page transitions, modals |
     | slower | 500ms | Complex animations |
     | slowest | 800ms | Hero animations, reveals |

  3. **Animation Patterns:**
     - Page load: staggered fade-up? slide-in? instant? scale-in?
     - Scroll reveal: fade-up? slide-from-side? parallax? none?
     - Hover states: scale? color shift? shadow? glow? underline?
     - Click feedback: scale-down? ripple? color flash?
     - Modal entry: fade + scale? slide-up? slide-from-right?
     - Loading states: skeleton? pulse? spinner? progress bar?

  4. **Reduced Motion:**
     - What changes when `prefers-reduced-motion: reduce` is active
     - Which animations are removed vs simplified

  ## PART C: SPACING & GRID SYSTEM (shared)

  1. **Base Unit:** 4px (0.25rem)

  2. **Spacing Scale:**
     | Token | Value | Use Case |
     |-------|-------|----------|
     | space-0 | 0 | None |
     | space-1 | 4px (0.25rem) | Tight inline |
     | space-2 | 8px (0.5rem) | Icon gaps, tight padding |
     | space-3 | 12px (0.75rem) | Small component padding |
     | space-4 | 16px (1rem) | Standard padding |
     | space-5 | 20px (1.25rem) | Section gap |
     | space-6 | 24px (1.5rem) | Card padding |
     | space-8 | 32px (2rem) | Section spacing |
     | space-10 | 40px (2.5rem) | Large section gap |
     | space-12 | 48px (3rem) | Major section spacing |
     | space-16 | 64px (4rem) | Page section breaks |
     | space-20 | 80px (5rem) | Hero spacing |
     | space-24 | 96px (6rem) | Major page divisions |

  3. **Layout Grid:**
     - Container max-width: ?px
     - Column count: 12
     - Gutter width: ?px
     - Margin (mobile): ?px
     - Margin (desktop): ?px
     - Breakpoints: sm (640px), md (768px), lg (1024px), xl (1280px), 2xl (1536px)

  4. **Border Radius Scale:**
     | Token | Value | Use Case |
     |-------|-------|----------|
     | radius-none | 0 | Sharp edges |
     | radius-sm | 4px | Subtle rounding |
     | radius-md | 8px | Standard components |
     | radius-lg | 12px | Cards |
     | radius-xl | 16px | Large cards, modals |
     | radius-2xl | 24px | Pills, badges |
     | radius-full | 9999px | Circles, avatars |

  Format as structured markdown with JSON code blocks for machine-readable data.
```

### Agent 4: Voice & Tone Architect

```
subagent_type: "general-purpose"
model: "opus"
prompt: |
  You are a UX writer, brand voice specialist, and content strategist.

  Given this brand strategy:
  ---
  [INSERT STRATEGY.md CONTENT]
  ---

  Create the VOICE & TONE SYSTEM for 3 brand variants.

  For EACH variant:

  1. **Voice Character:**
     - If this brand were speaking at a dinner party, how would they sound?
     - Vocabulary level: casual / conversational / professional / poetic / technical?
     - Sentence structure: short punchy / flowing / fragmented / mixed?
     - Humor style: none / dry / warm / playful / irreverent / self-deprecating?
     - Formality when things go wrong (errors, downtime): warmer? same? more formal?

  2. **Tone Spectrum** (rate 1-10):
     - Formal ←→ Casual: ?
     - Serious ←→ Playful: ?
     - Technical ←→ Simple: ?
     - Reserved ←→ Enthusiastic: ?
     - Authoritative ←→ Friendly: ?
     - Poetic ←→ Direct: ?

  3. **Microcopy Examples** (for each: ✅ FITS and ❌ DOESN'T FIT):

     | Situation | ✅ Fits This Brand | ❌ Wrong For This Brand |
     |-----------|-------------------|------------------------|
     | Welcome (new user) | "..." | "..." |
     | Welcome (returning) | "..." | "..." |
     | Error (generic) | "..." | "..." |
     | Error (server down) | "..." | "..." |
     | Success (task done) | "..." | "..." |
     | Empty state (no data) | "..." | "..." |
     | CTA (primary) | "..." | "..." |
     | CTA (secondary) | "..." | "..." |
     | Loading | "..." | "..." |
     | 404 page | "..." | "..." |
     | Onboarding step | "..." | "..." |
     | Upgrade prompt | "..." | "..." |
     | Farewell/logout | "..." | "..." |
     | Notification | "..." | "..." |

  4. **Writing Rules:**
     - Max sentence length: ? words
     - FORBIDDEN words/phrases (list 15+):
       e.g., "leverage", "synergy", "game-changer", "cutting-edge", etc.
     - PREFERRED words/phrases (list 10+):
       e.g., use "build" not "leverage", use "simple" not "streamlined"
     - Capitalization: Title Case / Sentence case / lowercase everywhere?
     - Emoji usage: never / sparingly (CTAs only) / freely / specific emojis only
     - Exclamation marks: never / max 1 per page / freely
     - Oxford comma: yes / no
     - Contractions: always ("you're") / never ("you are") / mixed
     - Numbers: spelled out under 10? Always digits?

  5. **Tagline Candidates** (3 per variant):
     - Must be evocative, NOT descriptive
     - Max 6 words each
     - For each: why it works, where to use it (hero, footer, social bio)

  6. **Brand Story Template:**
     - Hero headline (max 8 words)
     - Subheadline (max 20 words)
     - 3-sentence elevator pitch
     - About page opening paragraph (50-80 words)

  7. **Social Media Voice Adaptation:**
     - Twitter/X: character, emoji use, hashtag policy
     - LinkedIn: formality shift, content types
     - Instagram: caption style, story tone
     - How voice changes across platforms (subtle shifts, not identity changes)

  Format as structured markdown with clear variant separation.
```

### Agent 5: Logo System Architect + SVG Generator

**This agent does NOT just describe logos — it PRODUCES actual SVG code for every concept.**

```
subagent_type: "general-purpose"
model: "opus"
prompt: |
  You are a SENIOR LOGO DESIGNER at a world-class branding agency (think Pentagram,
  Wolff Olins, Collins). You have 15+ years of experience crafting iconic marks for
  brands like Stripe, Linear, Vercel, Notion, Arc, Figma, Spotify.

  Your specialty: designing logos directly in SVG code with the precision of a master
  typographer and the eye of an art director. Every curve is intentional. Every
  proportion is considered. Your logos look like they cost $50K — not like a developer
  threw some shapes together.

  Given this brand strategy:
  ---
  [INSERT STRATEGY.md CONTENT]
  ---
  Brand name: [BRAND_NAME]
  Brand colors: [PRIMARY_HEX], [SECONDARY_HEX], [ACCENT_HEX]
  Typography: [PRIMARY_FONT], [SECONDARY_FONT]

  ---

  ## SENIOR DESIGNER MINDSET

  Before you write a single line of SVG, think like a senior designer:

  1. **The mark must have a CONCEPT** — not just "pretty shapes". What idea does it encode?
     Good: Airbnb's "A" that's also a heart + location pin + person
     Bad: Random geometric shapes that "look modern"

  2. **The wordmark must have CRAFT** — not just a font with letter-spacing.
     Good: Custom kerning pairs, optical adjustments, intentional weight choices
     Bad: <text font-family="Inter">BrandName</text> with zero modifications

  3. **The icon must be MEMORABLE** — recognizable at a glance, even tiny.
     Good: Stripe's gradient-shifted "S", Linear's layered squares
     Bad: A generic circle with a letter inside

  4. **NEGATIVE SPACE** — great logos use the space between elements.
     Good: FedEx arrow, Spartan Golf Club, NBC peacock
     Bad: Solid shape with no visual trick or secondary reading

  5. **OPTICAL CORRECTIONS** — mathematically perfect ≠ visually perfect.
     Round shapes must overshoot baseline. Pointed shapes extend past cap height.
     "O" appears smaller than "H" at same size — compensate.

  ---

  ## WHAT AMATEUR SVG LOGOS LOOK LIKE (NEVER DO THIS)

  ```svg
  <!-- AMATEUR — Generic, no craft, no concept -->
  <svg viewBox="0 0 200 50">
    <circle cx="25" cy="25" r="20" fill="#3B82F6"/>
    <text x="55" y="35" font-family="Inter" font-size="28" fill="#111">
      BrandName
    </text>
  </svg>
  ```

  Problems: Basic circle with no meaning. Default font with no kerning. No personality.
  No concept. Looks like a placeholder. Could be ANY brand.

  ## WHAT SENIOR-LEVEL SVG LOGOS LOOK LIKE (THIS IS YOUR BAR)

  ```svg
  <!-- SENIOR — Intentional, crafted, conceptual -->
  <svg viewBox="0 0 240 56" xmlns="http://www.w3.org/2000/svg" aria-label="Meridian Logo">
    <defs>
      <linearGradient id="mark-grad" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color="#6366F1"/>
        <stop offset="100%" stop-color="#8B5CF6"/>
      </linearGradient>
    </defs>
    <!-- Mark: Abstract "M" formed by two intersecting planes — suggests convergence -->
    <g transform="translate(4, 4)">
      <!-- Left plane -->
      <path d="M0 44 L20 0 L24 0 L24 44 Z" fill="url(#mark-grad)" opacity="0.9"/>
      <!-- Right plane — shifted, creates depth -->
      <path d="M16 44 L36 0 L40 0 L40 44 Z" fill="url(#mark-grad)" opacity="0.65"/>
      <!-- Intersection creates lighter stripe — the "meeting point" -->
      <path d="M16 44 L24 24.9 L24 44 Z" fill="#ffffff" opacity="0.3"/>
    </g>
    <!-- Wordmark: Custom-kerned with optical adjustments -->
    <g transform="translate(60, 0)">
      <!-- Each letter positioned manually for optical perfection -->
      <text y="39" font-family="'DM Sans', sans-serif" font-weight="500"
            font-size="30" fill="#0f172a" letter-spacing="-0.01em">
        <tspan>M</tspan><tspan dx="-1">e</tspan><tspan dx="-0.5">r</tspan><tspan>i</tspan><tspan dx="-0.5">d</tspan><tspan>i</tspan><tspan dx="-0.5">a</tspan><tspan>n</tspan>
      </text>
    </g>
  </svg>
  ```

  Why this is better: The mark HAS A CONCEPT (two converging planes = "meridian"/meeting
  point). Uses gradient for depth. Intersection creates a visual "aha". Wordmark uses
  custom letter-by-letter kerning with tspan dx offsets. Font choice is specific and
  intentional (DM Sans 500 — geometric but warm). Every element has a reason.

  ---

  ## YOUR DELIVERABLES

  Produce **3 complete logo concepts**, each with **production-grade SVG code**.

  ---

  ## PART 1: LOGO CONCEPTS (3 required)

  For EACH of the 3 concepts:

  ### A. Concept Story (2-3 sentences)
  - What IDEA does this mark encode?
  - What should someone "get" after looking at it for 3 seconds?
  - How does it connect to the brand's emotional core?

  ### B. Design Rationale
  - **Construction grid:** How is this built? (golden ratio, modular grid, circle-packing, custom)
  - **Where the personality lives:** Which specific element carries the brand's character?
  - **The "aha moment":** What secondary reading or hidden meaning exists?
  - **Why this works at scale:** What makes it recognizable at 16px AND impressive at 400px?

  ### C. Typography Decisions
  - **Font choice:** Specific Google Font name + weight (e.g., "DM Sans 500", "Space Grotesk 600")
  - **Why this font:** What quality of this typeface matches the brand personality?
  - **Custom kerning:** Which letter pairs need dx/dy adjustments? (list specific pairs)
  - **Optical corrections:** Which letters need size/position tweaks? (round letters overshoot baseline?)
  - **Case & spacing:** lowercase/UPPER/Title + letter-spacing value + any custom modifications
  - **If custom lettering:** Describe each letter modification precisely (cut terminals, rounded corners, connected strokes, geometric reconstruction)

  ### D. SVG CODE (THE DELIVERABLE — PRODUCTION QUALITY)

  **For EACH concept, generate these 5 SVGs:**

  **1. Primary Logo** (full mark — icon + wordmark or full wordmark)
  ```
  Requirements:
  - viewBox with exact proportions (not 0 0 100 100 — use real ratios like 0 0 240 56)
  - Use <path d="..."/> for icon/symbol shapes — draw with precision:
    * Use cubic bezier curves (C) for smooth organic curves
    * Use quadratic beziers (Q) for simpler curves
    * Use arcs (A) for perfect circular segments
    * Use lines (L/H/V) for straight edges
    * Close paths with Z
  - Use <text> with <tspan dx="..."> for CUSTOM KERNING on wordmark
    * Adjust dx per letter pair: "AV" needs -1.5, "To" needs -1, "LT" needs -0.5, etc.
    * Round letters (O, C, G, Q) may need dy="-0.5" overshoot
    * Reference exact Google Font: font-family="'Space Grotesk', sans-serif"
  - Use <defs> for:
    * <linearGradient> or <radialGradient> if the design uses gradient
    * <clipPath> for masked elements
    * <filter> ONLY for subtle effects (very light blur for glow — keep it tasteful)
  - NEVER use: drop-shadow, outer-glow, 3D bevel, emboss, or any effect that screams "amateur"
  - Group with <g> and use transform="translate(x,y)" for precise positioning
  - Comment sections: <!-- Mark --> <!-- Wordmark --> <!-- Tagline lockup -->
  ```

  **2. Icon Only** (square, for app icon / avatar)
  ```
  Requirements:
  - viewBox="0 0 64 64" — square format
  - Icon/symbol only, no text
  - Must be recognizable as a standalone mark
  - Center the mark with proper padding (typically 8-12px on each side)
  - Must survive circular crop (for social avatars)
  - At 16px equivalent: only the boldest shapes should remain
  ```

  **3. Monochrome** (single color — black)
  ```
  Requirements:
  - Same geometry as primary
  - ALL fills = "#0a0a0a" (not pure black — slightly warm)
  - ALL strokes = "#0a0a0a"
  - Gradients replaced with flat fills (or very subtle opacity differences)
  - Must maintain full hierarchy and readability
  - This version must work for: single-color print, embossing, engraving
  ```

  **4. Reversed** (white on dark)
  ```
  Requirements:
  - Same geometry as primary
  - ALL fills = "#fafafa" (not pure white — slightly warm)
  - If the mark uses transparency/opacity layers, adjust for dark backgrounds
  - Test mentally: does this still "pop" on #0a0a0a, #1a1a2e, brand-primary backgrounds?
  ```

  **5. Favicon** (maximum simplification)
  ```
  Requirements:
  - viewBox="0 0 32 32"
  - RADICALLY simplified — remove ALL fine detail
  - Only the boldest, most recognizable element survives
  - Thick strokes (min 2px at 32px scale), high contrast
  - Must be recognizable in a browser tab at 16px next to page title
  - Consider: could someone identify this brand from JUST this tiny mark?
  ```

  ### E. Advanced SVG Techniques to Use

  **Gradients (when appropriate):**
  ```svg
  <defs>
    <linearGradient id="brand-grad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#6366F1"/>
      <stop offset="100%" stop-color="#EC4899"/>
    </linearGradient>
  </defs>
  <path d="..." fill="url(#brand-grad)"/>
  ```

  **Layered opacity for depth:**
  ```svg
  <path d="..." fill="#6366F1" opacity="0.9"/>
  <path d="..." fill="#6366F1" opacity="0.5"/>  <!-- Creates depth illusion -->
  ```

  **Custom kerning with tspan:**
  ```svg
  <text font-family="'Space Grotesk', sans-serif" font-weight="600" font-size="28">
    <tspan x="0" y="35">L</tspan>
    <tspan dx="-1.2">i</tspan>    <!-- Li: tighten -->
    <tspan dx="0">n</tspan>
    <tspan dx="-0.4">e</tspan>    <!-- ne: slight tighten -->
    <tspan dx="-0.3">a</tspan>    <!-- ea: slight tighten -->
    <tspan dx="0">r</tspan>
  </text>
  ```

  **Negative space tricks:**
  ```svg
  <!-- Cutout using white fill on colored background -->
  <rect width="48" height="48" rx="8" fill="#0f172a"/>
  <path d="..." fill="#ffffff"/>  <!-- "Cuts" into the dark shape -->
  ```

  **Smooth curves with cubic beziers:**
  ```svg
  <!-- Don't: rough approximation with line segments -->
  <path d="M10 40 L15 30 L20 25 L30 20 L40 10"/>
  <!-- Do: smooth professional curve -->
  <path d="M10 40 C15 35, 20 28, 30 20 S45 8, 50 10"/>
  ```

  ### F. Usage System

  For each concept:

  **Color applications** (describe, with specific hex values):
  - On white (#ffffff)
  - On light gray (#f8fafc)
  - On brand primary
  - On brand secondary
  - On dark (#0a0a0a)
  - On photography (which version + overlay recommendation)
  - Monochrome only (for stamps, embossing, watermarks)

  **Spacing rules:**
  - Clear space = Nx (where N = specific multiplier of icon height)
  - Generate an SVG showing the clear space zone with dotted lines

  **10 Logo DON'Ts** (specific, not generic):
  1. Never stretch or compress — always scale proportionally
  2. Never change the icon-to-wordmark spacing ratio
  3. Never use colors outside the brand palette
  4. Never add shadows, glows, bevels, or 3D effects
  5. Never place on backgrounds with <3:1 contrast ratio
  6. Never rotate, skew, or warp any element
  7. Never outline or stroke the logo (it's designed as fills)
  8. Never rearrange the icon/wordmark relationship
  9. Never animate without approved motion guidelines
  10. Never use the icon at less than 16px or wordmark at less than 80px wide

  ---

  ## PART 2: FAVICON + APP ICON SYSTEM

  For the BEST concept for small sizes:

  - **SVG Favicon** (primary — most modern browsers support this)
  - **ICO fallback description** (32x32 + 16x16 — describe what to render)
  - **Apple Touch Icon (180x180):** Describe treatment — padding, background color, corner radius
  - **Android Icons (192 + 512):** Describe treatment
  - **App Icon variant:** Does the icon get a colored background? Rounded corners? More padding?

  ---

  ## PART 3: OG IMAGE TEMPLATES (actual SVGs)

  Generate 3 SVG templates at viewBox="0 0 1200 630":

  **Homepage OG:**
  - Logo centered or positioned strategically
  - Brand colors as background gradient
  - Tagline text element
  - Clean, bold, impressive when shared on social media

  **Blog Post OG:**
  - Title placeholder area (large, prominent)
  - Author/date area
  - Brand bar (top or bottom with logo)
  - Designed for content sharing

  **Generic Page OG:**
  - Logo + dynamic title area
  - Flexible layout for any page type

  Each as actual SVG code with <text> placeholders.

  ---

  ## PART 4: CONTEXT MOCKUPS (as SVG scenes)

  Generate actual SVG mockup scenes showing each concept in context:

  **1. Browser Tab Mockup** (viewBox="0 0 300 36")
  - Tab shape with favicon + page title
  - Shows how the favicon looks at real browser tab size

  **2. Navbar Mockup** (viewBox="0 0 1200 64")
  - Full-width nav: logo left, nav links right
  - Shows logo at typical navbar size (~120-160px wide)

  **3. App Icon Mockup** (viewBox="0 0 120 120")
  - iOS-style rounded square with icon inside
  - Shows how it looks on a phone home screen

  **4. Social Avatar Mockup** (viewBox="0 0 80 80")
  - Circular crop
  - Shows how it survives the circle treatment

  **5. Dark Mode Sidebar** (viewBox="0 0 240 600")
  - Dark sidebar (#0f172a) with reversed logo at top
  - Nav items below — realistic SaaS sidebar

  ---

  ## SELF-REVIEW CHECKLIST (DO THIS BEFORE SUBMITTING)

  After generating all SVGs, review each one against these criteria:

  | Check | Pass? |
  |-------|-------|
  | Does the mark have a real CONCEPT (not just "shapes")? | |
  | Could I explain the idea in one sentence? | |
  | Is the wordmark custom-kerned (not default spacing)? | |
  | Do the curves use proper beziers (not jagged line segments)? | |
  | Does the icon work at 16px? (mentally shrink it) | |
  | Does it survive monochrome? (remove all color — still recognizable?) | |
  | Is there negative space magic or a secondary reading? | |
  | Would a senior designer at Pentagram approve this? | |
  | Does it look DIFFERENT from the top 5 competitors? | |
  | Would I be proud to put this in my portfolio? | |

  If ANY check fails → revise the SVG before including it.

  ---

  ## OUTPUT FORMAT

  ```markdown
  # Logo System — [BRAND_NAME]

  ## Concept 1: "[Name]" — [Type]
  ### The Idea
  [2-3 sentences: what concept does this mark encode?]
  ### Design Rationale
  [Construction, personality, aha moment, scalability]
  ### Typography
  [Font, weight, kerning decisions, optical corrections]

  ### SVG: Primary Logo (Color)
  ```svg
  [production SVG code]
  ```

  ### SVG: Icon Only (64x64)
  ```svg
  [production SVG code]
  ```

  ### SVG: Monochrome (Black)
  ```svg
  [production SVG code]
  ```

  ### SVG: Reversed (White)
  ```svg
  [production SVG code]
  ```

  ### SVG: Favicon (32x32)
  ```svg
  [production SVG code]
  ```

  ### Usage Rules
  [Color apps, spacing, don'ts]

  ---
  ## Concept 2: "[Name]" — [Type]
  [same full structure]

  ---
  ## Concept 3: "[Name]" — [Type]
  [same full structure]

  ---
  ## Favicon + App Icon System
  [Recommended concept + all sizes]

  ## OG Image Templates
  ### SVG: Homepage OG (1200x630)
  ### SVG: Blog Post OG (1200x630)
  ### SVG: Generic Page OG (1200x630)

  ## Context Mockups
  ### SVG: Browser Tab
  ### SVG: Navbar
  ### SVG: App Icon
  ### SVG: Social Avatar
  ### SVG: Dark Mode Sidebar

  ## Final Recommendation
  [Ranked 1-2-3 with specific reasoning for each]
  [Which concept best embodies the brand strategy?]
  [Which is most versatile across all contexts?]
  [Which has the strongest "aha" moment?]
  ```

  ---

  ## CRITICAL REQUIREMENTS

  - One concept MUST be text-focused (wordmark or lettermark)
  - One concept MUST be icon-focused (symbol or emblem)
  - One concept MUST be a combination mark
  - ALL concepts must include REAL, PRODUCTION-QUALITY SVG CODE
  - ALL SVGs must render correctly in any browser (test mentally)
  - ALL must work at 16px (favicon) AND 400px (hero)
  - ALL must work in monochrome (black and white)
  - ALL wordmarks must use CUSTOM KERNING (tspan dx adjustments)
  - ALL icons must use proper bezier curves (C/S/Q commands, not jagged lines)
  - EVERY mark must have a real concept — not just "geometric shapes"
  - EVERY mark must have something that makes you look twice (negative space, secondary reading, clever construction)
  - Think DIGITAL-FIRST: app icon, social avatar, browser tab, email sig, navbar
  - Quality bar: Would a creative director at Pentagram sign off on this?
```

**After all 5 agents complete → Read outputs → Synthesize into CREATIVE-DIRECTION.md**

---

## PHASE 4: VARIANT ASSEMBLY + DESIGN TOKENS

### Step 4.1: Assemble Variants

Combine all Phase 2 + Phase 3 outputs into 2-3 cohesive variants.

Each variant is a **complete identity system:**

```json
{
  "meta": {
    "brandName": "...",
    "generatedAt": "2026-02-25",
    "generatedBy": "Dafnck Studio Brand Identity System v2.0",
    "variantCount": 3
  },
  "shared": {
    "emotionalCore": {
      "primaryEmotions": [...],
      "journeyMap": {...},
      "internalCompass": "...",
      "designPrinciples": [...],
      "antiPositioning": [...]
    },
    "kapfererPrism": {
      "physique": { "core": "...", "details": [...], "manifests": {...}, "antiPattern": "..." },
      "personality": { "core": "...", "details": [...], "manifests": {...}, "antiPattern": "..." },
      "culture": { "core": "...", "details": [...], "manifests": {...}, "antiPattern": "..." },
      "relationship": { "core": "...", "details": [...], "manifests": {...}, "antiPattern": "..." },
      "reflection": { "core": "...", "details": [...], "manifests": {...}, "antiPattern": "..." },
      "selfImage": { "core": "...", "details": [...], "manifests": {...}, "antiPattern": "..." }
    },
    "brandPersonality": {
      "personDescription": {...},
      "adjectives": [...],
      "crossIndustryRefs": [...],
      "manifesto": "...",
      "extendedManifesto": "...",
      "archetype": {...}
    },
    "positioning": {
      "competitors": [...],
      "positioningMap": {...},
      "uvp": "...",
      "targetArchetypes": [...]
    },
    "spacing": {...},
    "motion": {...},
    "antiPatterns": [...]
  },
  "variants": [
    {
      "id": "variant-1",
      "name": "Eclipse",
      "subtitle": "Bold, confident, premium",
      "colors": {
        "light": {
          "primary": { "name": "...", "hex": "...", "oklch": "...", "hsl": "...", "role": "primary", "intent": "..." },
          "secondary": {...},
          "accent": {...},
          "background": {...},
          "surface": {...},
          "muted": {...},
          "border": {...}
        },
        "dark": {
          "primary": {...},
          ...
        }
      },
      "typography": {
        "heading": { "name": "...", "googleUrl": "...", "weights": [...], "personality": "..." },
        "body": { "name": "...", "googleUrl": "...", "weights": [...], "readability": "..." },
        "mono": { "name": "...", "googleUrl": "..." },
        "scale": {...}
      },
      "voice": {
        "character": "...",
        "toneSpectrum": {...},
        "microcopy": {...},
        "writingRules": {...},
        "taglines": [...]
      },
      "visualLanguage": {
        "photography": {...},
        "illustration": {...},
        "textures": {...},
        "iconStyle": {...},
        "moodboardKeywords": [...]
      },
      "logo": {
        "concept": "...",
        "type": "wordmark|monogram|symbol|combination|emblem",
        "description": "...",
        "svgPrimary": "<svg viewBox='...' ...>...</svg>",
        "svgIcon": "<svg viewBox='0 0 64 64' ...>...</svg>",
        "svgMonochrome": "<svg ...>...</svg>",
        "svgReversed": "<svg ...>...</svg>",
        "svgFavicon": "<svg viewBox='0 0 32 32' ...>...</svg>",
        "usageRules": {...},
        "donts": [...],
        "contextMockups": {...},
        "ogTemplate": {
          "svgHomepage": "<svg viewBox='0 0 1200 630' ...>...</svg>",
          "svgBlogPost": "<svg ...>...</svg>",
          "svgGeneric": "<svg ...>...</svg>"
        }
      },
      "tagline": "...",
      "manifesto": "..."
    }
  ]
}
```

### Step 4.2: Generate Design Tokens

For EACH variant, generate:

**`exports/design-tokens-[variant].css`:**
```css
:root {
  /* Colors - Light Mode */
  --color-primary: oklch(0.55 0.15 250);
  --color-secondary: oklch(0.65 0.10 200);
  --color-accent: oklch(0.70 0.20 330);
  --color-background: oklch(0.98 0.005 250);
  --color-surface: oklch(0.96 0.005 250);
  --color-muted: oklch(0.92 0.005 250);
  --color-border: oklch(0.88 0.01 250);

  /* Typography */
  --font-heading: 'Clash Display', sans-serif;
  --font-body: 'Instrument Sans', sans-serif;
  --font-mono: 'JetBrains Mono', monospace;

  /* Type Scale */
  --text-xs: 0.75rem;
  --text-sm: 0.875rem;
  /* ... */

  /* Spacing */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  /* ... */

  /* Radius */
  --radius-sm: 0.25rem;
  --radius-md: 0.5rem;
  /* ... */

  /* Motion */
  --ease-default: cubic-bezier(0.4, 0, 0.2, 1);
  --duration-fast: 100ms;
  --duration-normal: 200ms;
  /* ... */
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-primary: oklch(0.75 0.12 250);
    /* ... dark mode overrides */
  }
}
```

**`exports/tailwind-brand-[variant].ts`:**
```typescript
// Paste this into your tailwind.config.ts extend section
export const brandTheme = {
  colors: {
    primary: 'oklch(0.55 0.15 250)',
    secondary: 'oklch(0.65 0.10 200)',
    // ...
  },
  fontFamily: {
    heading: ['Clash Display', 'sans-serif'],
    body: ['Instrument Sans', 'sans-serif'],
    mono: ['JetBrains Mono', 'monospace'],
  },
  // ...
}
```

**`exports/brand-claude-section.md`:**
```markdown
## Brand Identity — [Brand Name] ([Variant Name])

### Colors (oklch for Tailwind v4)
| Role | oklch | Hex |
|------|-------|-----|
| Primary | oklch(...) | #... |
| ... |

### Typography
- **Headings:** [Font Name] (Google Fonts)
- **Body:** [Font Name] (Google Fonts)
- **Mono:** [Font Name] (Google Fonts)

### Voice & Tone
- Tone: [spectrum summary]
- Forbidden words: [list]
- CTA style: [example]

### Visual Style
- Photography: [one-line summary]
- Illustration: [one-line summary]
- Icons: [style]

### Anti-Patterns
[Top 5 anti-patterns]
```

Save as `BRAND-VARIANTS.json` + all export files in working directory.

---

## PHASE 5: AI PROMPTS & ANTI-PATTERNS

**Launch 2 agents IN PARALLEL:**

### Agent 1: AI Image Prompting Pack

For EACH variant, generate **7 ready-to-use prompts:**

| # | Prompt Type | Description |
|---|-------------|-------------|
| 1 | **Background Texture** | Subtle texture for sections/cards/hero backgrounds |
| 2 | **Hero Visual** | Key illustration or photography for landing page hero |
| 3 | **App Icon Direction** | Square icon concept (1024x1024 for app stores) |
| 4 | **UI Accent Element** | Decorative element: blob, line art, abstract shape |
| 5 | **Mood Reference Scene** | Environmental/lifestyle shot showing the brand "world" |
| 6 | **Pattern/Motif** | Repeatable brand pattern for backgrounds, merch, etc. |
| 7 | **Social Media Visual** | Instagram/LinkedIn post visual template |

**Each prompt MUST include ALL of these:**
```
Subject: [What the image shows]
Style: [Specific style — reference real artists/techniques/eras]
Medium: [Photography, digital art, 3D render, vector, watercolor, etc.]
Lighting: [Natural, studio, dramatic, golden hour, neon, etc.]
Color palette: [Reference hex codes from the variant palette]
Mood: [3 emotional keywords]
Composition: [Centered, asymmetric, close-up, bird's eye, etc.]
Camera/lens: [If photography: 35mm, 85mm, macro, wide angle, etc.]
Negative prompt: [What to EXCLUDE — be specific, ALWAYS include this]
Aspect ratio: [16:9, 1:1, 4:5, 9:16, etc.]
Recommended model: [Midjourney v6 / Flux Pro 1.1 / DALL-E 3 / Ideogram / Gemini Imagen 4]
Quality tags: [--q 2, --s 750, or equivalent for the model]
```

**CRITICAL PROMPT RULES (learned from production):**
- Prompts MUST describe VISUAL APPEARANCE of the result, NOT the design process
  - WRONG: "a logo design for a coaching brand", "vector illustration of brand identity"
  - RIGHT: "a gold rose emblem embossed on dark textured paper", "elegant typography on cream surface"
- ALWAYS include negative instructions in every prompt:
  - "no ampersand, no text other than [BRAND NAME], no clip art, no watermark, no stock photo feel"
  - "no blurry edges, no digital artifacts, no generic stock imagery"
- For logo/mark prompts: describe what the FINAL MARK looks like as a physical object, not the design process
- For lifestyle mockups: specify the SAME distinctive visual element across all mockups for brand consistency

**Example prompt:**
```
A textured paper surface with subtle warm grain, reminiscent of Japanese washi paper.
Soft cream base color (#FAF7F2) with barely visible fiber strands in muted gold (#C9A96E).
Shot from directly above, flat lay style. Even, soft diffused lighting with no harsh shadows.
Macro photography feel, 100mm lens. Minimal, meditative, tactile.
--no text, logos, objects, patterns, sharp edges, digital artifacts
--ar 16:9 --q 2 --s 250
Model: Midjourney v6
```

### Agent 2: Anti-Pattern Curator

Generate **12+ specific anti-patterns** organized by category:

**Format per anti-pattern:**
```markdown
### Anti-Pattern #N: [Descriptive Name]
**Category:** Visual / Voice / UX / Content / Motion
**The Trap:** [What it looks like — be vivid and specific]
**Why It's Wrong:** [How it betrays the brand soul — connect to emotional core]
**Real-World Example:** [Name a real product/brand that does this]
**Instead:** [What to do — specific alternative]
```

**Minimum coverage:**
- 3+ Visual design anti-patterns (e.g., "never use stock photos of people smiling at laptops")
- 3+ Voice/tone anti-patterns (e.g., "never say 'unlock your potential' or any self-help cliché")
- 2+ UX/interaction anti-patterns (e.g., "never add confetti animations on task completion")
- 2+ Content/imagery anti-patterns (e.g., "never show empty dashboards in marketing")
- 2+ Motion/animation anti-patterns (e.g., "never bounce or overshoot — this brand is calm")

---

## PHASE 6: BUILD INTERACTIVE BRAND BOOK

**Use the `nextjs-developer` agent type to build the brand book site.**

### Project Location

```bash
# ALWAYS in the project's brand-book subdirectory
$PROJECTS_DIR/<category>/[ProjectName]/brand-book/
```

### Tech Stack

- **Next.js 15+** (App Router, static export)
- **Tailwind CSS v4** (oklch colors, CSS custom properties)
- **shadcn/ui** — MANDATORY design system. Use these components:
  - `Button` (primary, secondary, outline, ghost, destructive, link variants)
  - `Card` (for all content sections and swatches)
  - `Tabs` (for variant sections, category filtering)
  - `Badge` (for labels, tags, categories)
  - `Tooltip` (for color codes, copy confirmations)
  - `Sheet` (mobile navigation)
  - `Separator` (section dividers)
  - `ScrollArea` (long content sections)
  - `Toggle` / `ToggleGroup` (variant switcher, dark mode toggle)
  - `Accordion` (collapsible anti-pattern details)
  - `Dialog` (enlarged previews)
  - `Slider` (tone spectrum interactive)
  - `Switch` (dark mode toggle)
  - `Skeleton` (loading states)
  - Initialize with `npx shadcn@latest init` + add needed components
- **next/font** (Google Fonts loading — load ALL variant fonts)
- **React Context** (variant state propagation)
- **Framer Motion** (scroll reveals, variant transitions, hover states)
- **Lucide Icons** (consistent icon set throughout)
- No database, no auth — pure static site

### shadcn/ui Theming

The brand book itself uses a NEUTRAL base theme so it doesn't compete with the brand variants being showcased. The variant colors are applied to PREVIEW sections (component preview, color swatches) while the brand book chrome (nav, headers, footer) stays neutral.

```css
/* Brand book chrome = neutral */
--book-bg: oklch(0.98 0 0);
--book-text: oklch(0.15 0 0);
--book-muted: oklch(0.90 0 0);

/* Preview areas = variant-aware */
--preview-primary: var(--variant-primary);
--preview-secondary: var(--variant-secondary);
/* etc. */
```

### Project Structure

```
brand-book/
├── src/
│   ├── app/
│   │   ├── layout.tsx              # Root layout, font loading, metadata
│   │   ├── page.tsx                # Overview / hero landing
│   │   ├── emotional-core/
│   │   │   └── page.tsx            # Emotional core + journey map
│   │   ├── prism/
│   │   │   └── page.tsx            # 🆕 Kapferer Brand Identity Prism (6 facets)
│   │   ├── personality/
│   │   │   └── page.tsx            # Brand personality + archetypes
│   │   ├── colors/
│   │   │   └── page.tsx            # Color palettes + dark mode + accessibility
│   │   ├── typography/
│   │   │   └── page.tsx            # Live specimens + type scale + article preview
│   │   ├── voice/
│   │   │   └── page.tsx            # Tone spectrum + microcopy + writing rules
│   │   ├── visual-language/
│   │   │   └── page.tsx            # Photography + illustration + moodboard
│   │   ├── components-preview/
│   │   │   └── page.tsx            # 🆕 Live buttons, cards, inputs per variant
│   │   ├── spacing-motion/
│   │   │   └── page.tsx            # 🆕 Grid, spacing scale, motion demos
│   │   ├── logo/
│   │   │   └── page.tsx            # Logo system: inline SVGs, size slider, bg switcher, copy buttons
│   │   ├── anti-patterns/
│   │   │   └── page.tsx            # What NOT to do
│   │   ├── ai-prompts/
│   │   │   └── page.tsx            # AI prompt cards (copyable)
│   │   └── export/
│   │       └── page.tsx            # 🆕 Design tokens, CSS, Tailwind config
│   ├── components/
│   │   ├── variant-switcher.tsx    # 🔥 Floating variant switcher
│   │   ├── brand-nav.tsx           # Side navigation with section links
│   │   ├── color-swatch.tsx        # Color display with copy-to-clipboard
│   │   ├── type-specimen.tsx       # Live typography preview
│   │   ├── tone-radar.tsx          # Radar chart for tone spectrum
│   │   ├── prompt-card.tsx         # AI prompt card with copy button
│   │   ├── anti-pattern-card.tsx   # Anti-pattern display card
│   │   ├── component-preview.tsx   # 🆕 Live UI component demos
│   │   ├── motion-demo.tsx         # 🆕 Animation demo component
│   │   ├── spacing-viz.tsx         # 🆕 Spacing scale visualization
│   │   ├── code-block.tsx          # Code display with copy
│   │   ├── section-header.tsx      # Consistent section headers
│   │   └── footer.tsx              # "Created by Dafnck Studio"
│   ├── lib/
│   │   ├── brand-data.ts           # All variant data (typed)
│   │   ├── variant-context.tsx     # React context for active variant
│   │   └── utils.ts                # Copy-to-clipboard, formatters
│   └── styles/
│       └── globals.css             # Base styles + variant CSS variables
├── public/
│   ├── og-image.png                # Default OG image
│   └── favicon.svg                 # SVG favicon
├── next.config.ts
├── package.json
├── tsconfig.json
└── CLAUDE.md
```

### Key Feature: Variant Switcher

The variant switcher is the **hero interaction:**

```
┌─────────────────────────────┐
│  Choose Your Direction       │
│                              │
│  ┌──────┐ ┌──────┐ ┌──────┐│
│  │Eclipse│ │Horizon│ │Pulse ││
│  │  ●    │ │  ○   │ │  ○   ││
│  └──────┘ └──────┘ └──────┘│
│                              │
│  Each variant changes:       │
│  Colors • Fonts • Voice •    │
│  Imagery • Logo • Everything │
└─────────────────────────────┘
```

- Floating/sticky at top of page (or in sidebar on desktop)
- Shows variant name + color preview dot
- Click → smooth CSS transition of ALL custom properties
- URL updates with `?variant=eclipse` (shareable)
- Stores in localStorage for persistence
- On mobile: compact pill version

### Design Requirements

| Requirement | Implementation |
|-------------|----------------|
| **Premium feel** | This IS the brand — the book itself is a design piece |
| **Responsive** | Full mobile support (client views on phone) |
| **Dark mode** | System-aware toggle + per-variant dark treatment |
| **Animations** | Framer Motion: scroll reveals, hover states, variant transitions |
| **Copy buttons** | Every color code, font name, prompt, code snippet is copyable |
| **Print-friendly** | `@media print` stylesheet for key sections |
| **Fast** | Static generation (SSG), optimized fonts, minimal client JS |
| **Accessible** | WCAG AA, keyboard nav, screen reader labels |
| **Branded** | Footer: "Created by Dafnck Studio" with subtle branding |
| **Shareable** | OG meta tags, favicon, proper page titles per section |

### Pages in Detail

**1. Overview (/)**
- Hero with brand name + selected tagline (variant-aware)
- Variant switcher prominently placed
- Manifesto section with type animation
- Summary cards linking to each section
- "Created for [Client Name] by Dafnck Studio" footer

**2. Emotional Core (/emotional-core)**
- Primary emotions as visual cards with intensity bars
- Emotional journey map (horizontal timeline or flow)
- Internal compass as large highlighted quote
- Anti-positioning matrix (grid of traps)
- Design principles (X over Y cards)

**3. Kapferer Prism (/prism)** 🆕
- Visual hexagonal/diamond prism diagram with all 6 facets
- Each facet as an expandable card with core statement + details + manifestations
- Two labeled axes: Externalization (Physique, Relationship, Reflection) vs Internalization (Personality, Culture, Self-Image)
- Two labeled poles: Sender/Brand (Physique, Personality) vs Receiver/Customer (Reflection, Self-Image)
- Variant-aware colors on the prism diagram
- Animated reveal on scroll (Framer Motion)

**4. Personality (/personality)**
- Brand-as-a-person profile card (age, style, interests)
- 5 adjectives as visual pills/badges with descriptions
- Cross-industry references as side-by-side cards
- Brand archetype wheel/card
- Full manifesto display

**5. Colors (/colors)**
- Large swatches (variant-aware) with ALL codes: hex, oklch, hsl, rgb
- Copy-to-clipboard button on each code
- Light mode / dark mode toggle preview
- Contrast ratio checker (live: pick text + background)
- Color-blind simulation (deuteranopia, protanopia, tritanopia)
- Usage examples: buttons, text, backgrounds, borders
- Gradient combinations (if allowed)

**6. Typography (/typography)**
- Live font specimens: all weights displayed
- Full alphabet + numbers + special characters
- Heading + body in context (fake article preview)
- Type scale visualization (stacked sizes)
- Font loading code snippet (Next.js + CSS) — copyable
- Paragraph example with proper line height and spacing

**7. Voice & Tone (/voice)**
- Tone spectrum radar chart (animated)
- Microcopy examples: DO vs DON'T side by side (green/red)
- Writing rules as checklist
- Forbidden words cloud (red) + preferred words (green)
- Tagline candidates as votable cards
- Brand story (hero headline + pitch)
- Social media voice variations

**8. Visual Language (/visual-language)**
- Photography style: description with mood reference
- Illustration direction: style notes
- Texture/material references
- Icon style guide with sample icons
- Moodboard keyword cloud (interactive, clickable to Pinterest search)
- Era/cultural references

**9. Component Preview (/components-preview)**
- Live buttons: primary, secondary, outline, ghost, destructive
- Cards: standard, with image, horizontal
- Form inputs: text, select, textarea, checkbox, toggle
- Badges/tags
- Alert/notification styles
- ALL rendered in the active variant's colors + fonts
- Dark mode toggle to preview both

**10. Spacing & Motion (/spacing-motion)**
- Spacing scale visualization (boxes growing in size)
- Grid system demo (12-column overlay)
- Border radius showcase
- Motion demos: click buttons to see easing curves in action
- Transition speed demo: fast → slow
- Reduced motion note

**11. Logo System (/logo)**
- 3 logo concepts rendered as ACTUAL INLINE SVGs (not descriptions — real visuals)
- Each concept shows: primary (color), icon-only, monochrome, reversed, favicon
- Variant switcher changes logo colors per brand variant
- Interactive size slider: drag to preview logo from 16px to 400px
- Background switcher: preview on white, dark, brand color, photo overlay
- Clear space diagram rendered as SVG with measurement annotations
- DO / DON'T grid: side-by-side correct vs incorrect usage (SVG examples)
- Context mockups: navbar preview, app icon preview (rounded corners), social avatar (circular crop), email signature, favicon in browser tab
- Copy SVG button for each variation (copies clean SVG code)
- Download section: SVG, PNG @1x @2x @3x for each variation
- OG image template preview (1200x630) with dynamic text simulation

**12. Anti-Patterns (/anti-patterns)**
- Card grid of anti-patterns
- Category tabs (Visual, Voice, UX, Content, Motion)
- Each card: trap description, why it's wrong, what to do instead
- "Bad example" vs "Good alternative" visual comparison

**13. AI Prompts (/ai-prompts)**
- Copyable prompt cards with syntax highlighting
- Organized by type (texture, hero, icon, accent, mood, pattern, social)
- Model recommendation badge
- One-click copy button per prompt
- Negative prompt section highlighted

**14. Export (/export)**
- CSS custom properties (full) — copy button
- Tailwind config extension — copy button
- CLAUDE.md brand section — copy button
- Google Fonts import links — copy button
- Complete BRAND-VARIANTS.json — download button

---

## PHASE 7: DEPLOY + DEV HANDOFF

### Step 7.1: Build Check

```bash
cd "$PROJECTS_DIR/<category>/[ProjectName]/brand-book"
npm run build
# Must have 0 TypeScript errors
# Must have 0 build errors
```

### Step 7.2: Deploy to Vercel

```bash
# Use Dafnck Studio Vercel account
# Token from ~/.claude/config or project .env.local
vercel --prod --yes --token "$VERCEL_TOKEN"
```

### Step 7.3: Generate Dev Handoff

Save to `[ProjectDir]/exports/`:
- `design-tokens-[variant].css` for each variant
- `tailwind-brand-[variant].ts` for each variant
- `brand-claude-section.md` (ready to paste in any project CLAUDE.md)
- `figma-tokens.json` (design token format for Figma)

### Step 7.4: Generate BRAND-SORT.md

Quick reference card saved to `[ProjectDir]/docs/BRAND-SORT.md`:

```markdown
# Brand Sort — [Brand Name]

| Field | Value |
|-------|-------|
| **Brand Name** | [Name] |
| **Description** | [One sentence] |
| **Target Audience** | [Emotional description] |
| **Color Image** | [Dominant color + mood] |
| **Business Description** | [What you sell to whom] |
| **Tagline** | [Best tagline] |
| **Internal Compass** | [Decision filter] |
| **Manifesto** | "We believe that..." |
| **Archetype** | [Primary archetype] |

## Variants Available
| Variant | Vibe | Primary Color |
|---------|------|---------------|
| Eclipse | Bold, confident | #... |
| Horizon | Warm, inviting | #... |
| Pulse | Energetic, fresh | #... |

## Quick Prompts
[Top 3 AI image prompts]

## Color Codes (Selected Variant)
| Role | Hex | oklch |
|------|-----|-------|
| Primary | #... | oklch(...) |
| Secondary | #... | oklch(...) |
| ... |

## Fonts
- **Heading:** [Name] — [Google Fonts URL]
- **Body:** [Name] — [Google Fonts URL]
- **Mono:** [Name] — [Google Fonts URL]
```

### Step 7.5: Deliver

```markdown
🎉 **Brand Identity System Live!**

📎 **URL:** https://brand-[name].vercel.app
📎 **Variants:** [Variant 1] • [Variant 2] • [Variant 3]

**Share this link with your client.**
They click between variants → explore colors, typography, voice, everything.
When they choose → you have design tokens ready for development.

**Project location:** $PROJECTS_DIR/<category>/[ProjectName]/
**Brand book:** /brand-book/ (deployed)
**Strategy docs:** /docs/ (6 files)
**Dev exports:** /exports/ (design tokens, Tailwind config, CLAUDE.md section)

**Next steps:**
1. Client reviews → picks a variant
2. You run: `/team` or `/prd` with the selected variant
3. Design tokens auto-injected into the new project
```

---

## ARGUMENTS

```bash
# Standard (full workflow with all phases)
/brand-identity

# With input file (PDF, Word, markdown)
/brand-identity --input CLIENT-BRIEF.pdf

# With existing PRD
/brand-identity --prd /path/to/PRD.md

# Quick mode (1 variant only, minimal questions)
/brand-identity --quick

# Skip deploy (generate files + build only)
/brand-identity --no-deploy

# Resume from specific phase
/brand-identity --resume phase-3

# Name not decided (triggers Phase 0)
/brand-identity --needs-name

# Specific project name (creates under $PROJECTS_DIR/<category>/[name]/)
/brand-identity --project Atma
```

---

## AGENT ORCHESTRATION RULES

### Parallel Execution Strategy

**ALWAYS launch parallel agents in a SINGLE message with multiple Task tool calls.**

```
Phase 0: 0 agents (manual naming)
Phase 1: 0 agents (orchestrator does intake)
Phase 2: 3 agents in parallel → wait for all → synthesize STRATEGY.md
Phase 3: 5 agents in parallel → wait for all → synthesize CREATIVE-DIRECTION.md
Phase 4: 0 agents (orchestrator assembles variants)
Phase 5: 2 agents in parallel → wait for all → save files
Phase 6: 1 agent (nextjs-developer) → build brand book
Phase 7: 0 agents (orchestrator deploys)

Total: 11 parallel agents across 3 waves
```

### Agent Configuration

| Phase | Agent Count | Agent Type | Model | Max Turns |
|-------|------------|-----------|-------|-----------|
| Phase 2 | 3 | `general-purpose` | `opus` | 15 |
| Phase 3 | 5 | `general-purpose` | `opus` | 20 |
| Phase 5 | 2 | `general-purpose` | `opus` | 15 |
| Phase 6 | 1 | `nextjs-developer` | `opus` | 50 |

### Context Passing Protocol

Each agent receives:
1. **Its specific domain instructions** (detailed in each phase)
2. **The DISCOVERY-BRIEF.md** content (Phase 2 agents)
3. **The STRATEGY.md** content (Phase 3+ agents)
4. **The brand name and variant names** (Phase 5+ agents)
5. **All previous phase docs** when needed

### Quality Gate Between Phases

After each parallel wave, the orchestrator MUST:
1. Read ALL agent outputs
2. Check for consistency (e.g., Agent 1's colors don't clash with Agent 3's visual language)
3. Resolve contradictions (pick the stronger direction)
4. Synthesize into the unified document
5. **AUTO-PROCEED to the next phase immediately** — NEVER ask the user "should I continue?" or "ready for the next phase?"

---

## INTEGRATION WITH OTHER SKILLS

### With /vision (was /branding)
`/vision` is the **quick emotional discovery** (outputs VISION.md only).
`/brand-identity` is the **complete system** including visual design, tokens, and deployed site.

**Recommended:** Use `/brand-identity` for client-facing work. Use `/vision` for internal projects where you just need quick emotional alignment.

### With /build (full pipeline)
```
/build [product]  # Orchestrates: /vision → /prd → /brand-identity → create a plan → execution
```
`/build` automatically runs `/brand-identity` as Phase 3 and ensures all brand outputs (STRATEGY.md, design tokens, CREATIVE-DIRECTION.md) are propagated as `context_files[]` to every downstream implementation step.

### With /team
```
/brand-identity → Client approves variant → /team (reads brand exports from /exports/)
```

### With /prd
```
/brand-identity → Selected variant → /prd (references brand guidelines in STRATEGY.md)
```

### With /frontend-design
When building UI after brand identity:
- Load `exports/design-tokens-[variant].css` into the project
- Load `exports/tailwind-brand-[variant].ts` into Tailwind config
- Reference anti-patterns during design review

---

## LEARNINGS FROM PRODUCTION (RM Project, Feb 2026)

These are battle-tested learnings from the first full production run. ALWAYS apply them.

### Logo Generation with AI

- **Gemini Imagen 4** via Google GenAI SDK is the recommended tool for AI-generated logo explorations and lifestyle mockups
- **AI image prompts MUST describe VISUAL APPEARANCE, not design processes.** Never say "logo design", "vector illustration", "brand identity design", "graphic design process". Instead describe what the RESULT looks like: "a gold rose emblem on dark paper", "embossed lettering on a leather surface"
- **Always add negative instructions** in AI image prompts: "no ampersand, no text other than [BRAND NAME], no clip art, no watermark, no stock photo feel"
- After AI generates logo explorations, the **client selects favorite directions**, then we create variations/mixes of those favorites
- **Lifestyle mockups** (image, 3D, mockup) should ideally use the SAME logo mark in different contexts, but AI generates different interpretations each time -- this is a known limitation. Mention this to the client.
- **Always generate an SVG version** of the final selected logo for download in the brand book export page

### Gold Accent Color

- **Gold is a common premium addition** clients request for luxury/premium brands
- Always include a gold/champagne accent color option in at least one variant for premium-positioned brands
- Example gold values: `#C9A96E` (warm gold), `#D4AF37` (classic gold), `#B8860B` (dark goldenrod)
- Gold works especially well for: borders, icon accents, hover states, premium badges, CTA highlights

### Brand Book Structure (Proven & Locked)

The current brand book page structure is **proven and should be kept as-is** for all future projects:

| Route | Content | Status |
|-------|---------|--------|
| `/` | Overview hub | Proven |
| `/prism` | Kapferer Brand Identity Prism | **NEW — MANDATORY** |
| `/emotional-core` | Brand emotions, compass, principles | Proven |
| `/personality` | Brand-as-person, manifesto | Proven |
| `/colors` | Color palettes, swatches, codes | Proven |
| `/typography` | Font specimens, scale, pairings | Proven |
| `/voice` | Tone spectrum, microcopy, writing rules | Proven |
| `/components-preview` | Live UI component showcase | Proven |
| `/logo` | Logo directions, usage rules | Proven |
| `/anti-patterns` | What NOT to do | Proven |
| `/ai-prompts` | AI image generation prompts | Proven |
| `/export` | Download CSS, Tailwind, fonts, SVG logos | Proven |
| `/site-*` | Website blueprints (per offer/product) | Optional per project |

### Technical Notes

- The brand book is a **static Next.js site** (`output: "export"`) deployed on Vercel
- Variant switching uses CSS custom properties on `data-variant` HTML attribute
- For projects where the client has already chosen a direction, **lock the variant** in `variant-context.tsx` (variant switcher becomes a no-op)
- Font loading: use `next/font/google` in `layout.tsx`, load all variant fonts upfront (3 per variant: heading, body, mono)

---

## QUALITY CHECKLIST

Before delivering:

### Strategy
- [ ] Kapferer Prism: ALL 6 facets filled with specific, vivid descriptions (no generic jargon)
- [ ] Kapferer Prism: Each facet has a core statement + details + design/copy/UX manifestations
- [ ] Internal compass is SPECIFIC (could settle a design debate)
- [ ] Emotional archetypes describe STATES, not demographics
- [ ] Anti-positioning names 5+ REAL products/brands
- [ ] Manifesto is under 20 words and memorable

### Creative
- [ ] 2-3 genuinely DISTINCT variants (not color shifts)
- [ ] All colors have oklch + hex + hsl + rgb
- [ ] All colors pass WCAG AA contrast for body text
- [ ] Dark mode is thoughtful (not just inverted)
- [ ] All fonts are on Google Fonts (free)
- [ ] No banned fonts (Inter, Roboto, Arial, etc.)
- [ ] Type scale includes all sizes with line heights
- [ ] Voice examples are specific and contrasting (fits vs doesn't fit)

### Logo System
- [ ] 3 logo concepts: wordmark + icon/symbol + combination mark
- [ ] ACTUAL SVG CODE generated for every concept (not just descriptions)
- [ ] 5 SVG variations per concept: primary, icon-only, monochrome, reversed, favicon
- [ ] All SVGs are valid, clean, and production-usable
- [ ] All logos work at 16px (favicon) and 400px (hero)
- [ ] All logos work in monochrome (black and white)
- [ ] 10 DON'T rules per concept with specific descriptions
- [ ] Context mockups described: navbar, app icon, social avatar, email sig, browser tab
- [ ] OG image templates as SVG (homepage, blog post, generic)
- [ ] Clear recommendation with ranked 1-2-3 reasoning
- [ ] Brand book shows inline SVGs (real visual rendering, not text)
- [ ] Interactive size slider and background switcher in brand book
- [ ] Copy SVG button for each variation

### AI Prompts
- [ ] AI prompts include negative prompts and model recommendations
- [ ] Anti-patterns are SPECIFIC (reference real things)

### Brand Book
- [ ] Kapferer Prism page (/prism) present with visual diagram and all 6 facets
- [ ] Variant switcher works smoothly with CSS transitions
- [ ] All pages render correctly on mobile (375px)
- [ ] Dark mode toggle works per variant
- [ ] Copy buttons work for all codes/snippets
- [ ] Component preview section shows live styled elements
- [ ] Footer: "Created by Dafnck Studio"
- [ ] OG meta tags present (title, description, image)
- [ ] Favicon present

### Logo & AI Prompts
- [ ] SVG download available for final/selected logo in export page
- [ ] AI image prompts describe VISUAL APPEARANCE (never "logo design", "vector", etc.)
- [ ] AI image prompts include negative instructions ("no text other than...", "no ampersand", etc.)
- [ ] Gold accent color included for premium-positioned brands

### Deployment
- [ ] Build passes with 0 errors
- [ ] Deployed to Vercel (Dafnck Studio account)
- [ ] URL is shareable and loads correctly
- [ ] All export files generated in /exports/
- [ ] BRAND-SORT.md generated
- [ ] Project in correct directory: $PROJECTS_DIR/<category>/[ProjectName]/

---

## Next Steps

After brand identity completion, suggest to the user:

```
Next: `create a plan` for implementation plan | `/deepux` for UX refinement | `/build` to continue pipeline
```

---

**Last Updated:** 2026-02-27
**Version:** 2.1
**Author:** Dafnck Studio
**Changelog v2.1:** Added Kapferer Brand Identity Prism (mandatory), gold accent for premium brands, AI prompt production rules, SVG logo export, RM project learnings
**Output:** Interactive Next.js Brand Book + Strategy Docs + Design Tokens + Vercel URL
**Integration:** /team, /vision, /build, /prd, /frontend-design
