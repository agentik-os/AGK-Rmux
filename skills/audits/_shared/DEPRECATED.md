# Deprecated Commands & Skills Registry

> Referenced by QUALITY-ARSENAL-PREAMBLE.md §10 and /metaudit Phase 5.
> When renaming or removing a command/skill, add it here FIRST, then update references.
> /metaudit scans all .md files for references to deprecated names and flags them.
>
> Vendored into OmegaOS skills/audits/_shared/ for install parity (LAW 0): a fresh
> clone+install ships this registry alongside the audits, so no audit SKILL.md depends
> on a path outside the installed tree.

## Format

```
| Old name | Replacement | Date | Reason |
```

## Active Deprecations

| Old name | Replacement | Date | Reason |
|----------|------------|------|--------|
| `/hunt` | `/debugaudit` | 2026-03-26 | Renamed for consistency with Quality Arsenal naming |
| `/delegate` | `/ceo` routing | 2026-04-14 | Never implemented as standalone; use /ceo for task routing |
| `/remotion` | `/creative_director` pipeline | 2026-04-14 | Removed; use creative_director for video production |
| `/head_of_marketing` | `/cmo` or `/content-strategy` | 2026-04-14 | Phantom reference; CMO handles marketing leadership |
| `/landing_page_analysis` | `/market landing` | 2026-04-14 | Consolidated into AI Marketing Suite |
| `/website_brand_analysis` | `/market brand` | 2026-04-14 | Consolidated into AI Marketing Suite |
| `/ad_creative_analysis` | `/ads_analyst` | 2026-04-14 | Consolidated into ads orchestrator |
| `/performance_marketer` | `/market` suite skills | 2026-04-14 | Consolidated into AI Marketing Suite |
| `/bmad` | Removed (no replacement) | 2026-04-14 | Never implemented; agile workflows handled by /planner |
| `Skill("hunt")` | `Skill("debugaudit")` | 2026-03-26 | Skill() call target renamed |
| `--quick` flag (any audit) | `--focus <area>` flag | 2026-04-14 | Rule 46 compliance; narrower scope with full depth |
| `/xoxo` | `/debugaudit` | 2026-04-14 | Legacy verification command; debugaudit supersedes |

## How to deprecate

1. Add entry to this file with old name, replacement, date, reason
2. Update all command/agent .md files that reference the old name
3. Run `/metaudit --focus deprecation` to verify zero remaining references
4. Keep the entry here permanently (historical record)

## How /metaudit uses this

Phase 5 of /metaudit:
1. Reads this file
2. For each deprecated name, greps all installed command + agent .md files
3. Flags any remaining reference as HIGH severity
4. Reports replacement suggestion from the table above
