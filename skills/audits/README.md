# Quality Arsenal — 23 Forensic Audits

OmegaOS ships with 23 Gestalt-Popper forensic audits (+ 2 orchestrators) that systematically verify every surface of a software project. Each audit is a multi-phase protocol that observes, measures, scores, fixes, and re-audits.

## Architecture

All audits share the **Gestalt-Popper doctrine**:

- **Gestalt clarity gate** — every finding must be independently verifiable
- **Popper falsification** — at least 12 challenges attempt to disprove each claim
- **Hinge point 10x** — critical findings get 10x scrutiny before reporting
- **Auto-fix** — audits don't just report, they fix and re-audit
- **Normalized scoring** — raw scores normalize to /100 for cross-audit comparison

## Categories

### Preventive (16) — Architecture & Design

| Audit | Domain | Phases | Score | What it answers |
|-------|--------|--------|-------|-----------------|
| `/codeaudit` | Code | 23 | /420 | Is the code SOLID? |
| `/flowaudit` | Flows | 25 | /400 | Does the experience WORK? |
| `/uiuxaudit` | Design | 23 | /420 | Is the interface BEAUTIFUL? |
| `/refontaudit` | Design | 25 | /540 | How to REDESIGN it to senior level? |
| `/featureaudit` | Features | 19 | /320 | Is the product COMPLETE? |
| `/a11yaudit` | Accessibility | 21 | /320 | Is it ACCESSIBLE? |
| `/seoaudit` | SEO | 25 | /400 | Is it DISCOVERABLE? |
| `/copyaudit` | Copy | 19 | /280 | Is the copy CLEAR? |
| `/dxaudit` | DX | 21 | /320 | Is the DX SMOOTH? |
| `/motionaudit` | Motion | 23 | /360 | Is the motion PURPOSEFUL? |
| `/automationaudit` | Automation | 22 | /330 | Is automation RELIABLE? |
| `/logicaudit` | Logic | 20 | /360 | Is the logic OPTIMAL? |
| `/i18naudit` | Localization | 18 | /360 | Is it WORLD-READY? |
| `/retentionaudit` | Retention | 20 | /400 | What FEATURES are missing? (READ-ONLY) |
| `/depaudit` | Dependencies | 18 | /360 | Is the supply chain SAFE? |
| `/releaseaudit` | Release | 20 | /400 | Is shipping SAFE + reversible? |

### Detective (7) — Runtime & Security

| Audit | Domain | Phases | Score | What it answers |
|-------|--------|--------|-------|-----------------|
| `/debugaudit` | Runtime | 23 | /360 | What is BROKEN right now? |
| `/perfaudit` | Performance | 23 | /360 | Is it FAST enough? |
| `/secaudit` | Security | 25 | /400 | Is it SECURE? |
| `/dataaudit` | Data | 21 | /320 | Is the data INTACT? |
| `/privacyaudit` | Privacy | 18 | /360 | Is user data HANDLED LAWFULLY? |
| `/apiaudit` | API | 23 | /360 | Is the API SOLID? |
| `/observabilityaudit` | Observability | 18 | /360 | Can we SEE what it does in prod? |

## How Oracles Use Audits

### Auto-selection

Oracles automatically select relevant audits at end of mission based on what changed:

| Mission type | Audits auto-triggered |
|-------------|----------------------|
| Code changes | `/codeaudit` |
| UI changes | `/uiuxaudit`, `/a11yaudit` |
| API changes | `/apiaudit`, `/secaudit` |
| Database changes | `/dataaudit` |
| Bug fixes | `/debugaudit` |
| Performance work | `/perfaudit` |
| Content/copy | `/copyaudit`, `/seoaudit` |
| Full product audit | All 23 in file-safe waves |

### Manual invocation

```bash
# Run a single audit
omega audit run codeaudit --dir /path/to/project

# List all available audits
omega audit list

# Auto-select audits for a mission description
omega audit select "fix the authentication flow"
```

### Configuration

In `~/.omega/config.toml`:

```toml
[audits]
auto_audits = []           # Empty = auto-select based on mission
pass_threshold = 70        # Minimum /100 score for PASS
end_of_mission_hook = true # Run audits at end of mission
max_parallel = 4           # Max concurrent audit workers
```

## Scoring System

Each audit has a domain-specific raw score reflecting its number of weighted phases:

```
normalized_score = (raw_score / max_score) * 100
```

- **90-100**: Excellent — exceeds standards
- **70-89**: Pass — meets quality bar
- **50-69**: Marginal — needs attention
- **0-49**: Fail — critical issues found

## Inter-Audit Dependencies

Some audits consume output from others:

```
/perfaudit → CWV data → /seoaudit
/dataaudit → schema types → /apiaudit
/apiaudit → auth surface → /secaudit
/copyaudit → hardcoded strings → /a11yaudit (locale verification)
```

## Adding a New Audit

1. Create `skills/audits/<name>audit/SKILL.md` following the template
2. Add entry to `skills/audits/registry.toml`
3. Create the full protocol as a Claude Code skill in `~/.claude/commands/<name>audit.md`
4. Run `omega sync` to distribute

## File Structure

```
skills/audits/
├── README.md            # This file
├── registry.toml        # Machine-readable registry
├── codeaudit/SKILL.md
├── flowaudit/SKILL.md
├── uiuxaudit/SKILL.md
├── debugaudit/SKILL.md
├── featureaudit/SKILL.md
├── perfaudit/SKILL.md
├── secaudit/SKILL.md
├── a11yaudit/SKILL.md
├── seoaudit/SKILL.md
├── dataaudit/SKILL.md
├── apiaudit/SKILL.md
├── copyaudit/SKILL.md
├── dxaudit/SKILL.md
├── motionaudit/SKILL.md
├── automationaudit/SKILL.md
├── logicaudit/SKILL.md
├── i18naudit/SKILL.md
├── depaudit/SKILL.md
├── releaseaudit/SKILL.md
├── privacyaudit/SKILL.md
├── observabilityaudit/SKILL.md
├── refontaudit/SKILL.md
└── retentionaudit/SKILL.md
```
