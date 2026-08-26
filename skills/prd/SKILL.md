---
name: prd
description: >
  OmegaOS-shipped product documentation suite generator (requirements, features, user stories,
  tech stack, milestones) for AI-agent implementation. Use when user says "/omg-prd",
  "generate PRD", "product docs", "product requirements". Supports --new, --skip-research,
  --input, --platform flags. Step 2 of the OmegaOS new-project pipeline; run /omg-vision
  first, then /omg-planner.
triggers: ["omg-prd", "generate PRD", "product docs", "product requirements"]
---

# /omg-prd - Complete Product Documentation Generator

**Command:** `/omg-prd`
**Skill:** `prd.md`
**Mode:** Plan Mode → Execution

---

## Command Behavior

When user types `/prd`, invoke the prd skill:

```
Use the prd skill to generate complete 11-file documentation suite for AI agent implementation.
```

This will:
1. Check for existing VISION.md (from /vision)
2. Ask clarifying questions if needed
3. Generate all 11 documentation files
4. Save to `/docs` folder
5. Provide implementation roadmap

---

## Arguments

```bash
# Standard (after /vision - recommended)
/prd

# From scratch (asks all questions)
/prd --new

# From existing brief/doc
/prd --input /path/to/brief.md

# Quick mode (MVP only, minimal docs)
/prd --quick

# Platform-specific
/prd --platform web
/prd --platform mobile
/prd --platform desktop

# Skip certain files
/prd --skip-testing
/prd --skip-design-system

# Verbose mode (more detailed specs)
/prd --verbose
```

---

## Output Structure

Generates `/docs` folder with:

```
/docs
├── VISION.md                    # Product identity (from /vision or generated)
├── PRD.md                       # Complete product requirements
├── TECH-ARCHITECTURE.md         # Technical architecture
├── DESIGN-SYSTEM.md             # Design system & visual language
├── LAYOUTS.md                   # Screen layouts & navigation
├── DATA-MODEL.md                # Complete data architecture
├── AGENT-PLAYBOOK.md            # AI agent implementation guide
├── TESTING-STRATEGY.md          # Testing plan
├── CHANGELOG-TEMPLATE.md        # Living documentation
├── GLOSSARY.md                  # Project terminology
└── FEATURES/                    # Individual feature specs
    ├── F-001-feature-name.md
    ├── F-002-feature-name.md
    └── ...
```

---

## Workflow Integration

### With /build (Recommended - Full Pipeline)

```bash
# Full pipeline: /vision -> /prd -> /brand-identity -> create a plan -> execution
/build
```

### With /vision (Manual Step-by-Step)

```bash
# Step 1: Define emotional foundation
/vision

# Step 2: Generate complete docs from vision
/prd

# Step 3: Brand identity system
/brand-identity

# Step 4: Plan with context-aware steps
create a plan

# Step 5: Implement with agents
/team
```

### Standalone

```bash
# Generate from scratch
/prd --new
```

### With existing brief

```bash
# Convert existing doc to full suite
/prd --input PROJECT_BRIEF.md
```

---

## What Gets Generated

### Core Documents (always)
- VISION.md
- PRD.md
- TECH-ARCHITECTURE.md
- AGENT-PLAYBOOK.md
- DATA-MODEL.md

### Design Documents (unless --skip-design-system)
- DESIGN-SYSTEM.md
- LAYOUTS.md

### Testing Documents (unless --skip-testing)
- TESTING-STRATEGY.md

### Meta Documents (always)
- CHANGELOG-TEMPLATE.md
- GLOSSARY.md

### Feature Specs (based on feature count)
- FEATURES/F-XXX-name.md for each P0 and P1 feature

---

## Discovery Questions

If VISION.md doesn't exist, will ask:

**Foundation:**
1. Project name and one-liner?
2. Core problem (emotional)?
3. Target user mindset?
4. Desired experience?
5. Anti-positioning?

**Technical:**
6. Platform (web/mobile/desktop)?
7. Tech stack preference?
8. Business model?
9. Auth needed?
10. Payments needed?

**Scope:**
11. MVP features (P0)?
12. Nice-to-have (P1)?
13. Post-MVP (P2-P3)?

**Design:**
14. Design references?
15. Aesthetic preference?
16. Color/branding?

**Constraints:**
17. Budget?
18. Timeline?
19. Compliance?
20. Technical limitations?

**Integrations:**
21. Third-party services?
22. Existing systems?
23. Analytics?

---

## Quality Gates

Before finalizing, verifies:

**Completeness:**
- [ ] All 11 files generated
- [ ] All P0 features have specs
- [ ] **GUARDIAN: Feature count in PRD.md (P0+P1) == file count in FEATURES/** ← NON-NEGOTIABLE
- [ ] All tech choices justified
- [ ] All dependencies mapped

**Specificity:**
- [ ] No vague requirements
- [ ] All acceptance criteria testable
- [ ] All API endpoints specified
- [ ] All data fields typed

**Agent-Ready:**
- [ ] AGENT-PLAYBOOK.md has ordered tasks
- [ ] Each task has exact file paths
- [ ] Gotchas documented
- [ ] Validation steps provided

**Cross-References:**
- [ ] Files reference each other correctly
- [ ] Feature IDs consistent across files
- [ ] Screen IDs match layouts
- [ ] Task IDs match dependencies

---

## Example Session

**User:** `/prd`

**Claude checks for VISION.md:**

```
Found VISION.md from /vision!
Using "TapeLog - Voice Journal" as foundation.

I'll generate the complete 11-file documentation suite.
Starting with technical questions...

Platform: You mentioned mobile. iOS only, or cross-platform?
```

**User:** "iOS only for MVP"

**Claude:**

```
Good. Tech stack recommendation:
- React Native + Expo (fast iteration, good audio support)
- Supabase (backend, auth, storage)
- Clerk (if you want premium auth UX)

Sound good, or do you have preferences?
```

**User:** "React Native + Supabase sounds great"

**Claude:**

```
Perfect. Now let's define MVP scope.

From VISION.md, core experience is "pressing record on a cassette deck".

MVP P0 features:
1. Record audio entry
2. Playback entries
3. Organize by date
4. Simple auth (email)

Any others critical for launch?
```

**User:** "That's it"

**Claude:**

```
Generating documentation suite...

✅ VISION.md (already exists)
✅ PRD.md (4 P0 features, 3 user journeys, success metrics)
✅ TECH-ARCHITECTURE.md (React Native + Expo + Supabase)
✅ DATA-MODEL.md (Entry, User tables)
✅ FEATURES/ (4 files)
✅ DESIGN-SYSTEM.md (vintage cassette aesthetic)
✅ LAYOUTS.md (3 screens: Record, Library, Entry Detail)
✅ AGENT-PLAYBOOK.md (18 ordered tasks)
✅ TESTING-STRATEGY.md
✅ CHANGELOG-TEMPLATE.md
✅ GLOSSARY.md

Documentation complete! 📚

Next steps:
1. Review /docs folder
2. Run /team team build to implement
3. Or manually assign tasks to agents
```

---

## Integration with /team

After generating docs:

```bash
# Option 1: Team Build (recommended)
/team team build

# Agents will:
# - Read AGENT-PLAYBOOK.md for tasks
# - Reference TECH-ARCHITECTURE.md for setup
# - Follow DESIGN-SYSTEM.md for UI
# - Use FEATURES/ for detailed specs
# - Implement in parallel with no ambiguity

# Option 2: Manual
# Review /docs and assign tasks yourself
```

---

## Advanced Usage

### Custom Tech Stack

```bash
/prd --stack "Next.js,Convex,Clerk,Stripe"
```

### Specific Compliance

```bash
/prd --compliance hipaa
/prd --compliance gdpr
/prd --compliance sox
```

### Multi-Language

```bash
/prd --i18n en,fr,es
```

### Performance-Critical

```bash
/prd --perf-target "load<1s,interaction<100ms"
```

---

## Troubleshooting

**Q: VISION.md not found, but I ran /vision?**
A: Check if VISION.md is in project root. If not, /prd will generate it.

**Q: Too many features, docs are huge?**
A: Use `--skip-research` for MVP only, or manually trim P1/P2 features after generation.

**Q: Want to regenerate single file?**
A: Edit the file manually, or delete it and run `/prd --update` to regenerate missing files.

**Q: Agents not following specs?**
A: Check AGENT-PLAYBOOK.md has concrete file paths and gotchas. Update and re-run agents.

---

## Output Location

**Default:** `/docs` folder in project root

**Custom:**
```bash
/prd --output /path/to/custom/location
```

**Git Integration:**
- Automatically creates `.gitignore` entry for temp files
- Commits `/docs` folder
- Adds PR template referencing VISION.md

---

## See Also

- `/vision` - Create VISION.md first (was /branding)
- `/build` - Full pipeline: /vision + /prd + /brand-identity + create a plan + execution
- `/team` - Complete product workflow
- `/team` - Implement from docs with parallel agent teams
- Skill file: `$HOME/.claude/skills/prd.md`
- Methodology: `~/projects/work/team/docs/prdide.md`

---

**Last Updated:** 2026-02-13
**Output:** 11-file documentation suite in `/docs`
