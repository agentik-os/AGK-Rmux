---
name: vision
description: >
  Define product identity and emotional positioning through Socratic discovery. Generates
  VISION.md with internal compass, personas, and design principles. Pure discovery, no code.
  Use when user says "/vision", "define brand", "product identity", "emotional positioning",
  or "what is this product about". Follow with /prd for product requirements, /brand-identity
  for full brand book, or /build to run the complete pipeline (vision + PRD + brand + UX +
  code).
---

# /vision - Product Identity & Emotional Positioning

**Command:** `/vision`
**Skill:** `branding.md`
**Mode:** Plan Mode (discovery only, no code)

---

## Command Behavior

When user types `/vision`, invoke the branding skill:

```
Use the branding skill to define the emotional foundation and product identity.
```

This will:
1. Enter Plan Mode
2. Ask deep discovery questions about WHO, HOW, WHAT NOT
3. Guide Socratic dialogue about sensory/emotional associations
4. Generate VISION.md with internal compass and design principles

---

## Arguments

```bash
# Standard invocation
/vision

# With context file
/vision --input PROJECT_BRIEF.md

# For existing project
/vision --project /path/to/project

# Quick mode (fewer questions)
/vision --quick
```

---

## Output

**Primary output:** `VISION.md` in project root or `/docs` folder

**VISION.md contains:**
- Internal Compass (one-sentence decision filter)
- Product Identity (soul statement)
- Emotional Personas (3 archetypes)
- Sensory Description (how it should feel)
- Anti-Positioning (what it's NOT)
- Emotional Design Principles (5 principles)

---

## When to Use

- **New product** - before any code is written
- **Before /prd** - as input for comprehensive PRD generation
- **Before /build** - first phase of the full build pipeline
- **After /team discovery** - to define emotional foundation
- **Rebranding** - when product has lost its identity
- **Design debates** - when team needs alignment

---

## Integration

**Full build pipeline (recommended):**

```
/build [product]    # Orchestrates everything: /vision -> /prd -> /brand-identity -> create a plan -> execution
```

**Manual workflow:**

```
/vision (identity) -> /prd (specs) -> /brand-identity (visual) -> create a plan -> /team
```

Or standalone:

```
/vision -> manual development
```

---

## Example Session

**User:** `/vision`

**Claude enters Plan Mode:**

```
Let's define the emotional foundation of your product.

First, give me a one-sentence description of what this is.
```

**User:** "A voice journal that feels like a cassette recorder"

**Claude:**

```
Perfect. Now let's go deeper.

What pain does this solve? Not functionally -- emotionally.
What are people tired of that makes them need this?
```

[... continues deep discovery ...]

**Final output:** `VISION.md` with Internal Compass, Product Identity, Emotional Personas, etc.

---

## Notes

- **No code is written** - this is pure discovery and positioning
- **Be opinionated** - vague answers get challenged
- **Be sensory** - abstract language gets pushed to concrete
- **Be specific** - generic startup speak gets rejected

---

**See also:**
- `/brand-identity` - Full brand identity system with colors, typography, logo, deployed as interactive site
- `/prd` - Generate full documentation suite from VISION.md
- `/build` - Full autonomous pipeline: /vision + /prd + /brand-identity + create a plan + execution
- `/team` - Complete product development workflow
- Skill file: `/home/hacker/.claude/skills/branding.md`

**When to use which:**
- `/vision` -> Quick internal alignment, outputs VISION.md text file
- `/brand-identity` -> Complete brand system, 2-3 switchable variants, deployed interactive site
- `/build` -> Full pipeline from vision to shipped product (AISB-orchestrated)
