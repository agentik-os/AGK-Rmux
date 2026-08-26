# Storyteller OS, v1.0.0

**Category:** Commercial Stack (supporting) / Narrative Craft and Truth Stewardship
**Omega position:** Commercial Stack (supporting): narrative craft, upstream of Content OS packaging and distribution
**Primary interface:** conversational (natural language), plus an optional local CLI story bank
**Status:** installable reference implementation

## Purpose
Act as the user's story architect, interviewer, truth steward, voice guardian, narrative strategist, and (only when authorized) writer: coach, mine, verify, shape, write, perform, adapt, score, and preserve truthful stories without erasing the user's voice.

## Promise
A story stays recognizably theirs, holds to the chosen evidence standard, creates a felt change in the audience, and performs its intended job. Truth and voice are preserved above all.

## The golden law
Default to COACH, not ghostwriter.

- COACH: ask, reflect, diagnose, and map. Produce no draft sentences, hooks, templates, or prose in the user's voice.
- CO-CREATE: offer structures, beat options, questions, and short neutral fragments. Do not complete the story.
- WRITE: draft or rewrite only after the user explicitly asks (a direct request for a reel, post, script, speech, pitch, carousel, thread, or complete draft is that authorization).
- EDIT: preserve the user's language and intent unless a stronger transformation is authorized.

## Lifecycle
Material moves through: lived signal, evidence, meaning, tension, shape, voice, delivery, adaptation, learning. The states are INTENT, CAPTURE, MINE, VERIFY, DEEPEN, SHAPE, VOICE, CREATE, PERFORM, ADAPT, LEARN. VERIFY is a hard gate: it never passes while a load-bearing fact is uncertain or a third party's consent is unresolved.

## Position in the value chain
Storyteller owns narrative truth, story structure, voice, consent, and the canonical story objects. It does not own editorial strategy, packaging, channel adaptation, publishing, or content analytics: those belong to Content OS. Storyteller sits upstream, handing deepened, truth-checked story objects downstream for distribution.

## What this OS contains
- Canonical operating contract (`SKILL.md`) and the exportable system prompt (`references/system-prompt.md`)
- 9 reference documents: operating manual, command protocols, story models, canonical story object, channel playbooks, quality and evals, research and canon, the exportable system prompt, and an operator-context template
- 1 interface descriptor (`agents/openai.yaml`) declaring display, default prompt, and product policy (ChatGPT, Codex, API, Atlas). Note: this pack ships no separate specialist sub-agent personas and no `skills/` or `protocols/` directories; the command router in `SKILL.md` and `references/commands.md` is the working skill surface
- 2 Python scripts: a network-free, standard-library-only local story-bank CLI (`scripts/storyteller_os.py`, SQLite) and its test (`scripts/test_storyteller_os.py`)
- 1 asset (`assets/icon.svg`)
- Omega integration contract (`OMEGA_INTEGRATION.md`): registration, handoffs, event types, and state classification

## Commands (conversational router)
Natural language always works: the router below is optional, never required.

| Command | Family | Purpose |
| --- | --- | --- |
| `/story` | orient | Open the system, explain authorship, begin |
| `/story-setup` | orient | Build a compact Storyteller Profile |
| `/mine` | discover | Surface up to seven candidate story signals from a source |
| `/interview` | discover | One neutral question at a time toward a verified center of gravity |
| `/moment` | discover | Compress a memory into a Moment Card without drafting |
| `/deepen` | deepen | Explore one missing load-bearing dimension at a time |
| `/shape` | shape | Offer up to three structural options and recommend one |
| `/hook` | shape | Diagnose or produce openings honest to the payoff |
| `/scene` | shape | Build or diagnose a scene using only supported details |
| `/arc` | shape | Map starting state, pressure, pivot, ending, belief update |
| `/cowrite` | create | Co-write beat by beat, freezing accepted wording |
| `/write` | create | Draft a complete deliverable (WRITE contract) |
| `/rewrite` | edit | Lightest useful edit, preserving facts and intent |
| `/voice` | edit | Extract a voice fingerprint and edit rules |
| `/adapt` | adapt | Rebuild one story natively per channel, preserving DNA |
| `/content` | adapt | Produce a multi-channel content package with one story DNA |
| `/keynote` | adapt | Build a talk where each story changes how the next idea lands |
| `/pitch` | adapt | Separate story from proof for an offer |
| `/brandstory` | adapt | Develop a narrative system, not one origin myth |
| `/customerstory` | adapt | Consented, attributable, qualified customer case |
| `/datastory` | adapt | Connect a data pattern to a human or business decision |
| `/truthcheck` | prove | Claim ledger, truth class, consent risks, verdict |
| `/score` | prove | Structural scorecard with ethics and truth as gates |
| `/rehearse` | prove | A spoken-performance pass with breath and pause map |
| `/feedback` | prove | Evidence-ordered feedback tied to a next version |
| `/storybank` | operate | Initialize, capture, list, inspect, update, search, verify, export, archive, review |
| `/repurpose` | operate | Map reuse by job and channel without cheapening the story |
| `/story-review` | operate | Turn the bank into one concrete editorial decision |

Default command `/story`; registered aliases: `/mine`, `/interview`, `/deepen`, `/shape`, `/write`, `/adapt`, `/truthcheck`, `/score`, `/rehearse`, `/storybank`.

## Commands (local story-bank CLI)
`scripts/storyteller_os.py` is deterministic, offline, and standard-library only. It stores canonical Story Objects in SQLite and its score is a structural-completeness heuristic, never a truth or virality judgment.

| Subcommand | Purpose |
| --- | --- |
| `init` | Initialize a story bank |
| `capture` | Capture a new Story Object |
| `list` | List or search Story Objects |
| `show` | Show a complete Story Object |
| `update` | Set dotted Story Object fields |
| `add-claim` | Add a claim-ledger entry |
| `add-consent` | Add a consent record |
| `validate` | Validate one Story Object |
| `score` | Score structural completeness |
| `export` | Export Story Objects |
| `doctor` | Validate the complete bank |

Example: `python3 scripts/storyteller_os.py init --db stories.db`

## Main handoffs
- Content OS receives deepened, adaptation-ready story objects for packaging and channel adaptation.
- Content OS returns performance feedback for story-object learning only, never for publishing decisions.
- Context and Memory OS holds the canonical state: confirmed story objects, truth-checked evidence, and consent records route through it (Storyteller writes staged records, Context and Memory OS returns verified ones).
- Review and Governance OS approves any change to boundaries, schemas, or quality gates in production.

## Installation
This pack ships no `INSTALL.md`. See `OMEGA_INTEGRATION.md` for registration, context-injection order, handoffs, and state classification. The optional local story bank runs directly with `python3 scripts/storyteller_os.py` (Python standard library only, no network, no LLM). Real, private operator context lives at `~/.omega/os/storytelling-os/ledger/context.md`; `references/gareth-context.md` is the shipped generic fallback template.
