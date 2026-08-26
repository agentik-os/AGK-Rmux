//! The AgentikOS operative-systems suite — registry + status for the OS tab
//! (TUI). 28 operative systems along the value chain, in four groups: PERSONAL
//! (Mindset, Health & Energy, Habit Tracker, Alignment, Seductive, Intuitive,
//! Identity Shift, Journal — the BE / ENERGY layer), the BUILD CHAIN (01 Strategy & Portfolio → 02 Brainstorm →
//! 03 Market Research → 04 Blueprint → 05 Design → 06 Stepper → 07 Builder →
//! 08 Quality/Evaluation/Release), GROWTH (Storyteller, Revenue, Delivery &
//! Customer Success, Relationship & Network, Wealth & Capital — the
//! COMMUNICATE → SELL → DELIVER → CAPITAL layer), and SYSTEMS (Execution,
//! Operations & Automation, Review & Governance, Context & Memory, AI Logic,
//! Content, Books — the AUTOMATE / LEARN / meta layer). Each lives under
//! `OS/<slug>/` in the repo (installed to `~/.omega/os/`); payloads arrive as
//! zips via the Deposit box and are unpacked in place. This module answers,
//! cheaply and with NO network: which OSes exist, where they live on THIS
//! machine, and which concrete readiness surfaces are present. Static presence
//! is never reported as runtime verification.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Which group of the suite an OS belongs to — the TUI renders one section
/// per group, in declaration order: Personal, Build chain, Growth, Systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsGroup {
    /// The personal BE / ENERGY layer (Mindset, Health & Energy, Habit
    /// Tracker, Alignment, Seductive, Intuitive, Identity Shift, Journal).
    Personal,
    /// The product pipeline, in chain order:
    /// 01 Strategy & Portfolio → 02 Brainstorm → 03 Market Research →
    /// 04 Blueprint → 05 Design → 06 Stepper → 07 Builder →
    /// 08 Quality / Evaluation / Release.
    BuildChain,
    /// The go-to-market layer, COMMUNICATE → SELL → DELIVER → CAPITAL
    /// (Storyteller, Revenue, Delivery & Customer Success, Relationship &
    /// Network, Wealth & Capital).
    Growth,
    /// Systems / meta OSes that operate ON a system rather than a product or a
    /// person (Execution, Operations & Automation, Review & Governance,
    /// Context & Memory, AI Logic, Content, Books).
    Systems,
}

/// One operative system of the suite — the static half (identity). The single
/// source of truth: the TUI tab, `OS/README.md` and install parity all derive
/// from `OsProduct::all()`; add or reorder an OS HERE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsProduct {
    /// Directory name under `OS/` — also the id used everywhere.
    pub slug: &'static str,
    /// Display name.
    pub name: &'static str,
    /// One-line focus shown in the detail pane.
    pub tagline: &'static str,
    /// The suite group this OS renders under.
    pub group: OsGroup,
    /// What you can do with this OS — the command surface shown in the OS-tab
    /// detail pane as declared capabilities. One line per entry; empty for a
    /// pre-integration OS (its detail shows the integration pipeline instead).
    pub commands: &'static [&'static str],
}

impl OsProduct {
    /// The whole suite, grouped and contiguous: PERSONAL, then the BUILD CHAIN
    /// in pipeline order (01→08), then GROWTH, then SYSTEMS. Chain position =
    /// index within the BuildChain group + 1 (derived, never hand-numbered).
    pub fn all() -> &'static [OsProduct] {
        &[
            // ── PERSONAL — the BE / ENERGY layer ───────────────────────────
            OsProduct {
                slug: "mindset-os",
                name: "Mindset OS",
                tagline: "Jim Rohn identity/wellbeing/wealth OS: evidence-labeled coaching, philosophy compiler, 90-day program.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex:",
                    "  /mindset-os   Jim Rohn identity/wellbeing/wealth coach",
                    "  labels every claim E1(established) · E2(promising) · S(spiritual)",
                    "                     P(personal) · C(clinical → a professional)",
                    "omega-mindset CLI (workspaces):",
                    "  new --name --output      the weekly workspace (19 files)",
                    "  score <scorecard.json>   validate + summarize a weekly scorecard",
                    "  challenge --output       the 6-month identity challenge",
                    "                           (180 daily + 26 weekly + 6 monthly + state)",
                    "  coach <dir>              one AI growth pass now (Telegram card)",
                    "  coach <dir> --arm        daily 07:00 auto-coaching (disarmed by default)",
                    "  coach <dir> --disarm     stop the loop",
                ],
            },
            OsProduct {
                slug: "health-energy-os",
                name: "Health & Energy OS",
                tagline: "Build and protect physical and cognitive capacity: sleep, movement, training, nutrition, recovery and stress.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /health · /health-energy-os ):",
                    "  /health             open Health & Energy OS (daily check-in)",
                    "  /readiness          assess today's capacity",
                    "  /health-audit       build a capacity baseline",
                    "  /sleep              audit sleep and circadian constraints",
                    "  /training           build or revise training",
                    "  /nutrition          review fuel and adherence",
                    "  /recovery           respond to fatigue or overload",
                    "  /travel-health      design a travel and jet-lag protocol",
                    "  /health-experiment  create an N-of-1 experiment",
                    "  /wearable           interpret device trends conservatively",
                    "  12 agents · 18 skills · 8 protocols · 6 schemas (say the word or the /command)",
                    "reference runtime (stdlib Python, provider-neutral):",
                    "  runtime/os_runtime.py info | route /health | event | validate",
                    "→ upstream capacity provider for Habit, Execution and Strategy OS",
                ],
            },
            OsProduct {
                slug: "habit-tracker-os",
                name: "Habit Tracker OS",
                tagline: "Habit Tracker {OS}: conversation-first habit system, deterministic state, adaptive reviews and seasons.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /habits · /habits-os · /habit-tracker-os ):",
                    "  conversation-first habit coach (the chat is the interface)",
                    "  modes it routes: SETUP · TODAY · CHECK_IN · URGE · LAPSE",
                    "                   REVIEW · RECOVER · ADAPT · VISUALIZE",
                    "  build good habits · reduce unwanted ones (friction + replacement)",
                    "  labels every record explicit · observed · inferred · proposed",
                    "  seasons: build · maintain · recover · travel · crisis",
                    "  safety-and-boundaries.md routes clinical risk to a professional",
                    "omega-habits CLI (deterministic state, per-user SQLite):",
                    "  init                     create/update the user profile",
                    "  add · update · list      accept, version, list habit contracts",
                    "  log · correct            append then supersede explicit/observed evidence",
                    "  today                    rank today's primary habits (seven maximum)",
                    "  review                   compute an evidence-bounded review",
                    "  chart                    render a Mermaid progress diagram",
                    "  context · export         compact LLM context · export user-owned state",
                    "  season · experiment      change season · create a bounded experiment",
                    "  delete · doctor          delete user-owned state · validate integrity",
                    "→ integrates with Mindset {OS} and Context & Memory {OS}",
                ],
            },
            OsProduct {
                slug: "alignment-os",
                name: "Alignment OS",
                tagline: "Alignment Coach {OS}: a wisdom + decision second brain — clarity, values, right effort, next action.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /coach · /align · /alignment-os ):",
                    "  a wisdom + decision inner counsel (Stoic · Daoist · Rohn synthesis)",
                    "  labels every claim E1..E5 (never metaphysics as science)",
                    "protocols / skills (say the word or the /command):",
                    "  /morning /evening /weekly   the daily/weekly protocols",
                    "  /decision      a structured decision protocol",
                    "  /true_north    reconnect to chosen values",
                    "  /virtue_check  virtue-before-outcome check",
                    "  /dichotomy_control   separate control from non-control",
                    "  /wu_wei        right effort vs forcing",
                    "  /reframe       reframe a situation   ·   /shadow   shadow work",
                    "  /belief_audit  audit a limiting belief   ·   /fear   fear work",
                    "  /meaning       meaning work   ·   /manifestation   grounded",
                    "  /quantum_truth accurate quantum guardrails",
                    "  /personal_philosophy   compile your philosophy",
                    "  /anti_dependency   hand agency back   ·   /reset   3-min reset",
                    "council: stoic·daoist·rohn·scientist·quantum·manifestation·shadow",
                    "         ·compassion·challenger·action·meaning·integrator (routed)",
                    "omega-align   open the coach master agent in a session",
                ],
            },
            OsProduct {
                slug: "seductive-os",
                name: "Seductive OS",
                tagline: "Seductive {OS}: personal magnetism — presence, conversation, warmth, style and romantic confidence, consent-first.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /seduction · /charisma · /seductive-os ):",
                    "  the personal magnetism coach (presence, conversation, inner game)",
                    "  builds a more compelling PERSON, never scripts, lines or routines",
                    "  labels every claim E1(replicated) · E2(thin) · E3(craft)",
                    "                     P(personal taste) · C(clinical → a professional)",
                    "modes (say the word or the /command):",
                    "  /presence      embodiment, voice, stillness, attention",
                    "  /conversation  curiosity, stories, humour, depth   ·   /style",
                    "  /innergame     self-worth under the skill   ·   /rejection",
                    "  /calibrate     read real interest vs your own inference",
                    "  /flirt         consent-gated   ·   /date   design a real one",
                    "  /apps          the digital half   ·   /desire   long-term",
                    "  /audit /practice /debrief /reset",
                    "ethics: consent is the product, not the constraint — refusals.md",
                    "        names the manipulative playbook it declines, and why each",
                    "        one also fails on YOUR terms. Ethics guardian holds a veto.",
                    "omega-seduction   open the magnetism master agent in a session",
                    "→ pairs with Mindset (identity), Health & Energy (the substrate),",
                    "  Alignment (values) and Relationship & Network (the platonic half)",
                ],
            },
            OsProduct {
                slug: "intuitive-os",
                name: "Intuitive OS",
                tagline: "Intuitive {OS}: train intuition as a measurable skill — capture, classify, forecast, score, calibrate.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /intuition · /intuitive-os ):",
                    "  turns instinct into a measurable learning loop, never a feeling as fact",
                    "  intuition is a HYPOTHESIS; the raw capture happens before analysis",
                    "  five layers never collapse: observation · interpretation · affect",
                    "                             · intuition · decision",
                    "  provenance: expert pattern · weak signal · emotion · incentive",
                    "              · bias · noise (several may hold at once)",
                    "modes (say the word or the /command):",
                    "  setup      domains, baselines, cadence   ·   capture   raw, uncontaminated",
                    "  signal     facts → anomalies → alternatives → baseline → test",
                    "  predict    a falsifiable 1-99% forecast with a resolution date",
                    "  decide     the full protocol (base rates, counter-evidence,",
                    "             reversibility, second-order effects, premortem)",
                    "  counter    red-team the hunch   ·   review   resolve and score",
                    "  daily /weekly /patterns /models /profile /90d",
                    "discipline: original predictions are IMMUTABLE after the outcome,",
                    "  domains score SEPARATELY, decision quality != outcome quality,",
                    "  small samples are flagged, never dressed as reliability.",
                    "engine: Brier score + calibration buckets (stdlib, deterministic)",
                    "omega-intuition   open the calibration master agent in a session",
                    "→ pairs with Alignment (values, decisions) and Mindset (identity);",
                    "  feeds Execution and Strategy with calibrated forecasts",
                ],
            },
            OsProduct {
                slug: "identity-shift-os",
                name: "Identity Shift OS",
                tagline: "Identity Shift {OS}: identity is accumulated evidence — contract, journal, mirror, contradictions, tomorrow.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /identity · /identity-shift-os ):",
                    "  converts a declared future identity into DAILY EVIDENCE",
                    "  identity is not a statement, it is what the record shows",
                    "  loop: vision → contract → journal → mirror → gap → tomorrow",
                    "        → action → evidence → memory → review → adaptation",
                    "modes (say the word or the /command):",
                    "  journal    what actually happened   ·   mirror   the honest pass",
                    "  tomorrow   max 3 objectives, each with a success condition",
                    "  contradiction   declared priority vs where the time went",
                    "  weekly /monthly /180   the review and recalibration cadence",
                    "  content    score it, then PUBLISH / LATER / PRIVATE / NO",
                    "  start /reset",
                    "discipline: life first and content second, NO shame (a gap is data,",
                    "  never moral failure), challenge proportional to capacity, and",
                    "  health/sleep/sobriety are never traded for vanity metrics.",
                    "  Journal is private by default; medical territory routes out.",
                    "orchestration: decides WHY and WHAT NEXT, specialists decide HOW",
                    "omega-identity   open the transformation master agent in a session",
                    "→ hands off to Mindset (identity), Habits (consistency), Execution",
                    "  (delivery), Intuitive (judgment), Alignment (values), and the chain",
                ],
            },
            OsProduct {
                slug: "journal-os",
                name: "Journal OS",
                tagline: "Journal {OS}: MIRROR, the nightly journal — reconstruct the day truthfully and turn it into evidence.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /journal · /journal-os ):",
                    "  MIRROR interviews you ONE question at a time, voice-first, FR/EN",
                    "  it does not motivate, flatter, or make your day sound interesting",
                    "  keeps FACT · EMOTION · INTERPRETATION · LESSON strictly apart",
                    "modes (say the word or the /command):",
                    "  /journal       the full nightly session   ·   quick   depleted pass",
                    "  mirror         the honest pass over a captured day",
                    "  tomorrow       regenerate the protocol (max 3 missions)",
                    "  contradiction  declared intention vs where the time went",
                    "  loops /memory /weekly /content",
                    "discipline: a contradiction needs a PATTERN, never one day; only",
                    "  behavioral language (avoided the task 90 min, never 'you are lazy');",
                    "  no artificial positivity AND no artificial negativity, so a failed",
                    "  day stays a failed day and no lesson is manufactured.",
                    "philosophy: Stoicism · Jim Rohn · Taoism as a LENS on the evidence,",
                    "  at most one per entry and usually none. Seneca's De Ira III.36 is",
                    "  the nightly review itself; Meditations was a private journal.",
                    "privacy: private by default, third parties by first name only, and",
                    "  MIRROR NEVER writes a post (it stops at the content handoff).",
                    "omega-journal   open MIRROR in a session",
                    "→ Journal owns the interview and the artifact; Identity Shift",
                    "  consumes it into longitudinal state and decides what next",
                ],
            },
            // ── BUILD CHAIN — CHOOSE → BUILD → CERTIFY (01→08) ──────────────
            OsProduct {
                slug: "strategy-portfolio-os",
                name: "Strategy & Portfolio OS",
                tagline: "Turn ambition, evidence and constraints into explicit choices, a ranked portfolio of bets and disciplined allocation.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex ( /strategy · /strategy-portfolio-os ):",
                    "  turn ambition + evidence into explicit choices and a ranked portfolio",
                    "  diagnose the critical challenge · set the strategy kernel · rank bets",
                    "router modes (say the word or the /command):",
                    "  /strategy            open strategic design (the strategy kernel)",
                    "  /diagnosis           define the critical challenge",
                    "  /portfolio           review all projects and bets",
                    "  /prioritize          rank competing initiatives",
                    "  /scenario            build future scenarios and signposts",
                    "  /strategic-decision  structure a consequential choice",
                    "  /quarter-plan        create the quarterly strategy",
                    "  /kill-review         decide continue / pivot / pause / kill",
                    "  /one-page-strategy   a concise strategy memo",
                    "  /not-doing           define the exclusions (the not-doing list)",
                    "  fund / pause / kill each bet · allocate time, attention, people, capital",
                    "  labels claims E1(evidence) · E2 · E3 · E4 · E5(preference)",
                    "  12 specialist agents · 20 skills · 6 protocols · 7 schemas",
                    "→ feeds Blueprint (a product bet) and Execution (personal outcomes)",
                ],
            },
            OsProduct {
                slug: "brainstorm-os",
                name: "Brainstorm OS",
                tagline: "Brainstorm {OS}: multi-agent imagination and decision council, lineage and a frozen concept handoff.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex ( /brainstorm · /brainstorm-os ):",
                    "  run the multi-agent imagination + decision council",
                    "  modes: spark · imagination · council · deep · red-team · converge · audit",
                    "  keeps lineage across: challenge · continue · evolve · go-deeper",
                    "  /ideate --wild   diverge without premature feasibility filtering",
                    "  /frame-fission   structurally different problem + worldview frames",
                    "  /signature       extract confirmed + inferred Founder DNA",
                    "  /genome · /evolve --generations N   encode + mutate concept loci",
                    "  /collision · /worlds · /anomaly     distant transfer, counterfactuals, survivor",
                    "  /surface --compare mobile,web,desktop,multi   embodiment lab matrix",
                    "  /challenge · /reframe · /mutate · /redteam    attack the leading concept",
                    "  /council <specialists> · /research · /compare · /converge · /experiment",
                    "  /audit · /freeze · /incubate · /portfolio · /continue",
                    "  /handoff blueprint | research | brief   package the frozen concept",
                    "brainstorm_os.py session engine (scripts/, when a filesystem is present):",
                    "  init · migrate · record frames/genomes/generations · compare surfaces",
                    "  incubate · audit · freeze · export · handoff · validate (session.schema.json)",
                    "→ emits a frozen concept handoff for Market Research / Blueprint",
                ],
            },
            OsProduct {
                slug: "market-research-os",
                name: "Market Research OS",
                tagline: "Market Research {OS}: evidence, validation and a bounded decision before Blueprint.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex ( /market-research · /market-research-os ):",
                    "  compile an idea into a decision-grade research + validation pack",
                    "  modes: NEW · RECOVER · RAPID_SCAN · FULL_VALIDATION · DILIGENCE",
                    "         DEEP_DIVE · MONITOR · AUDIT · DELTA (inferred per run)",
                    "  depths: SIGNAL(desk) · VALIDATION(primary+behavioral) · INVESTMENT_GRADE",
                    "  decisions: GO · PIVOT · HOLD · NO-GO · INSUFFICIENT (always bounded)",
                    "  never validated on desk research alone; ethical-scraping preflight first",
                    "market_research_os.py CLI (deterministic stdlib workspace):",
                    "  init        create the workspace state file (--project-id --project-name --decision)",
                    "  status      run status: record counts, artifact + gate progress, continuation pointer",
                    "  score       gate diagnostic vs the depth threshold + per-hypothesis evidence balance",
                    "  allocate    hand out monotonic stable IDs by prefix (SRC · HYP · DEC · PRC ...)",
                    "  validate    schema + 24 quality gates + checksum (exit 1 on critical, --strict)",
                    "  checkpoint  advance the revision + save a restart-safe continuation snapshot",
                    "  export      write the pack as json, or a markdown status view (--format --output)",
                    "  demo        create + validate a minimal valid workspace to read",
                    "→ handoffs: emits the validated decision + WTP evidence to Blueprint and Revenue (on GO / PIVOT)",
                ],
            },
            OsProduct {
                slug: "blueprint-os",
                name: "Blueprint OS",
                tagline: "The product-definition compiler: idea to a traceable, gated definition pack.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex:",
                    "  /blueprint <idea> · /blueprint-os   the product-definition compiler",
                    "  modes: NEW(compile) · RECOVER(rebuild truth) · EXTEND(add module)",
                    "         REVISE(supersede) · AUDIT(gaps/gates) · DELTA(diff+impact)",
                    "  continue(resume) · status · export <view>",
                    "omega-blueprint CLI (canonical state):",
                    "  init         create the state file (--project-id --project-name --request)",
                    "  validate     schema + 20 gates + handoff + checksum (exit 1 on critical)",
                    "  status       progress, revision, next section, checksum",
                    "  checkpoint   advance revision + save the continuation pointer",
                    "  demo         a valid minimal state to read",
                    "→ stops at BLUEPRINT COMPLETE — STEPPER READY, frozen handoff downstream",
                ],
            },
            OsProduct {
                slug: "design-os",
                name: "Design OS (UX/UI)",
                tagline: "Design {OS}: challenge the blueprint into flows, screens, states and a validated Design Handoff.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex ( /design-os ):",
                    "  compile an approved Blueprint into a challenged UX/UI definition + a validated Design Handoff",
                    "  challenge the flow · information architecture · screen/state contracts",
                    "  interaction + visual system · responsive + accessibility + trust states",
                    "  maps to shadcn/ui + STAX (the OmegaOS stack)",
                    "  modes: FULL(default) · AUDIT(challenge an existing design) · FLOW(selected journeys)",
                    "         AI_APP(chat/agents/artifacts) · STAX_FIT(panel-model decision) · REVISION(impacted IDs)",
                    "omega-designer CLI (contract validators, stdlib Python):",
                    "  intake <blueprint-intake.json>   validate the Blueprint intake schema",
                    "  handoff <design-handoff.json>    validate the Design Handoff (flows/surfaces/states/evals/seeds)",
                    "  self-test                        run the validator self-test",
                    "→ consumes the Blueprint handoff; emits STEPPER_READY design-handoff.json for Stepper",
                ],
            },
            OsProduct {
                slug: "stepper-os",
                name: "Stepper OS",
                tagline: "Step-by-step execution: a blueprint walked one verified step at a time.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex:",
                    "  /stepper-os   execute a blueprint step by step (verifier-gated DONE)",
                    "omega-stepper CLI (execution DAG):",
                    "  init         scaffold the project (manifest + sources + first step)",
                    "  validate     schema + DAG + audit Blueprint/Design references resolve",
                    "  status       weighted + raw progress, per-status counts",
                    "  plan         ranked READY candidates + the safe execution wave",
                    "  show / agent-brief   one step's spec / the self-contained agent brief",
                    "  start        claim a READY step (prints the brief: Blueprint + Design refs)",
                    "  verify       run the deterministic checks (no state change)",
                    "  done         verify then close — DONE only if every check passes",
                    "  fail / block / unblock   record a failure / block / lift a block",
                    "  review       record a review verdict (role gate)",
                    "  resume       reconcile interrupted attempts after a restart",
                    "  release-check   PASS only when every target-priority step is DONE",
                    "  report / events   status report / the append-only event log",
                    "→ references BOTH the Blueprint and the Design docs per step",
                ],
            },
            OsProduct {
                slug: "builder-os",
                name: "Builder OS",
                tagline: "The implementation runtime: the Stepper roadmap executed into release-ready code.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex:",
                    "  /build · /builder-os   the autonomous implementation runtime",
                    "  /build preflight · status · plan · run · step <id> · test · verify",
                    "         repair <id> · audit · resume · pause · release-check · report",
                    "omega-builder CLI (evidence state):",
                    "  init         init from the frozen Blueprint handoff + BUILD READY graph",
                    "  validate / status   structure / evidence-backed status",
                    "  sync-step    mirror one Stepper step (spec-hash, required-for-release)",
                    "  claim        claim a READY step (locks, worktree, branch)",
                    "  transition   apply a valid attempt transition",
                    "  record-check   record deterministic check evidence",
                    "  mark-step    mirror a Stepper status after the Verifier decision",
                    "  gate         evaluate a release gate BG01–BG20",
                    "  checkpoint   a recovery checkpoint",
                    "  set-release / finalize   set candidate / final handoff when all gates pass",
                    "  release-check   evaluate terminal release readiness   ·   demo (self-test)",
                    "→ executes the Stepper roadmap; never a competing TODO list",
                ],
            },
            OsProduct {
                slug: "quality-evaluation-release-os",
                name: "Quality, Evaluation & Release OS",
                tagline: "Independent certification between Builder and production: conformance, observability, recovery and a go/no-go.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex ( /quality · /quality-evaluation-release-os ):",
                    "  independent quality, evaluation and release authority",
                    "  positioned between Builder OS and production (certify on evidence)",
                    "  16 specialist agents · 26 skills · 7 protocols · 8 schemas",
                    "modes (say the word or the /command):",
                    "  /quality             open the quality authority (intake contracts + scope)",
                    "  /test-plan           build a risk-based test plan",
                    "  /traceability        map requirements to evidence (bidirectional matrix)",
                    "  /qa                  run functional and exploratory QA",
                    "  /ai-eval             design and run AI / agent evaluations",
                    "  /security-review     apply OWASP / threat-model security standards",
                    "  /accessibility       audit WCAG 2.2 / mobile accessibility",
                    "  /release-candidate   assemble the release-candidate evidence pack",
                    "  /release-gate        issue the go / no-go release decision",
                    "  /deploy              execute a controlled (canary / progressive) release",
                    "  /rollback            trigger or prepare a rollback",
                    "reference runtime (stdlib Python, integrity + routing):",
                    "  os_runtime.py info                 name / version / slug / purpose",
                    "  os_runtime.py route \"/quality\"      resolve a command to its mode",
                    "  os_runtime.py validate             sha256-check every packaged file",
                    "  os_runtime.py event <kind> <json>  append an event record",
                    "→ Builder OS builds; Quality certifies, gates and authorizes release",
                ],
            },
            // ── GROWTH — COMMUNICATE → SELL → DELIVER → CAPITAL ─────────────
            OsProduct {
                slug: "storyteller-os",
                name: "Storyteller OS",
                tagline: "Storyteller {OS}: coach, mine, verify and shape truthful stories without erasing your voice.",
                group: OsGroup::Growth,
                commands: &[
                    "Claude / Codex ( /story · /storyteller-os ):",
                    "  default to COACH, never ghost-write; never erase your voice",
                    "  /story        find + build a story from a real moment",
                    "  /mine · /interview · /moment    mine lived material for story signal",
                    "  /deepen · /shape · /hook · /scene · /arc    meaning + tension + structure",
                    "  /cowrite · /write · /rewrite · /voice    create + edit, only when authorized",
                    "  /adapt · /content · /keynote · /pitch    adapt to a channel, keep story DNA",
                    "  /brandstory · /customerstory · /datastory    business + evidence stories",
                    "  /truthcheck   verify claims against the evidence standard (a hard gate)",
                    "  /score · /rehearse · /feedback   score, perform, learn from response",
                    "  /storybank · /repurpose · /story-review   run your bank of stories",
                    "omega-story CLI (local story bank, SQLite, no network):",
                    "  init · capture · list · show · update    the canonical Story Objects",
                    "  add-claim · add-consent    claim ledger + consent records",
                    "  validate · score · doctor    structural completeness + bank health",
                    "  export --format jsonl|json|markdown    portable export",
                    "  default bank ~/.omega/os/storytelling-os/ledger/story-bank.db",
                    "→ coach, do not ghost-write; truth + consent are gates, not bonus points",
                ],
            },
            OsProduct {
                slug: "revenue-os",
                name: "Revenue OS",
                tagline: "The conversational revenue brain: offers, pricing, pipeline, invoicing, collections, cash flow and forecasting.",
                group: OsGroup::Growth,
                commands: &[
                    "Claude / Codex ( /revenue · /revenue-os ):",
                    "  conversational revenue brain + governed CRM/finance database",
                    "  /revenue        dashboard: integrated revenue + cash overview",
                    "  /offer          create or audit an offer",
                    "  /positioning    define category and differentiation",
                    "  /pricing        build a pricing architecture",
                    "  /pipeline       review CRM and forecast",
                    "  /lead           create or analyze a lead",
                    "  /sales-call     prepare or debrief a call",
                    "  /proposal       build proposal + commercial logic",
                    "  /invoice        create or inspect an invoice",
                    "  /collections    manage overdue receivables",
                    "  /business-cashflow   analyze business cash flow",
                    "  /receipt-business    stage a business receipt/photo",
                    "  /contract       stage contract data",
                    "  /revenue-close  run the monthly commercial/financial close",
                    "  /revenue-scenario    model revenue/cash scenarios",
                    "  /renewal        plan a renewal or expansion",
                    "  24 specialist agents · 40 skills · 10 protocols · 14 schemas",
                    "reference runtime (stdlib python, self-describing):",
                    "  runtime/os_runtime.py info · route \"/revenue\" · validate",
                    "  runtime/bootstrap_revenue_db.py   seed the reference records",
                    "→ handoffs: Market Research (WTP evidence), Delivery & Customer",
                    "            Success (signed scope), Wealth & Capital (only verified",
                    "            owner distributions cross the business/personal line)",
                ],
            },
            OsProduct {
                slug: "delivery-customer-success-os",
                name: "Delivery & Customer Success OS",
                tagline: "The customer journey after the sale: onboarding, delivery, adoption, value proof, renewal and expansion.",
                group: OsGroup::Growth,
                commands: &[
                    "Claude / Codex ( /delivery ):",
                    "  /delivery         open the delivery portfolio (review)",
                    "  /handoff-client   run the sales-to-delivery transfer",
                    "  /onboard-client   create the onboarding plan (access, roles, kickoff)",
                    "  /success-plan     define outcomes and measures with the customer",
                    "  /client-plan      milestones and governance (RACI)",
                    "  /client-update    draft a transparent status update",
                    "  /scope-change     process a change request (protect scope + margin)",
                    "  /client-risk      create an issue/escalation plan",
                    "  /adoption         build an adoption intervention",
                    "  /value-proof      compile outcome evidence (attribution honest)",
                    "  /qbr              prepare the business / value review",
                    "  /renew-client     prepare renewal or expansion",
                    "  /case-study       request and build a consented case study",
                    "  /offboard         close the engagement responsibly",
                    "  post-payment gate: contract.signed -> payment.reconciled -> handoff.accepted",
                    "  19 specialist agents · 30 skills · 9 protocols · 9 schemas",
                    "reference runtime (stdlib Python, provider-neutral, no LLM):",
                    "  runtime/os_runtime.py info | validate | route \"/delivery\" | event <kind> <json>",
                ],
            },
            OsProduct {
                slug: "relationship-network-os",
                name: "Relationship & Network OS",
                tagline: "Build, protect and deepen valuable relationships: attention, memory, generous relevance, follow-through, introductions.",
                group: OsGroup::Growth,
                commands: &[
                    "Claude / Codex ( /network · /relationship-network-os ):",
                    "  ethical relationship steward, connector and gathering architect",
                    "  /network                open relationship overview (audit mode)",
                    "  /person                 prepare a person brief",
                    "  /meeting-prep           prepare for a meeting",
                    "  /interaction            capture an interaction and its commitments",
                    "  /follow-up              draft a relevant, loop-closing follow-up",
                    "  /intro                  create a consent-based introduction",
                    "  /nurture                design a relationship rhythm (cadence)",
                    "  /difficult-conversation prepare a truthful conversation",
                    "  /boundary               set or reinforce a boundary",
                    "  /gathering              design a meaningful gathering",
                    "  12 agents · 18 skills · 7 protocols · 6 schemas",
                    "reference runtime (stdlib Python, not a production DB/LLM):",
                    "  python runtime/os_runtime.py info|validate|route|event",
                ],
            },
            OsProduct {
                slug: "wealth-capital-os",
                name: "Wealth & Capital OS",
                tagline: "The personal financial brain: cash flow, savings, emergency resilience, debt, investment policy and allocation.",
                group: OsGroup::Growth,
                commands: &[
                    "Claude / Codex ( /wealth · /wealth-capital-os ):",
                    "  a conversational personal CFO, behavior coach and capital planner",
                    "  /wealth          open the personal CFO dashboard",
                    "  /cashflow        analyze personal cash flow (inflows, outflows, margin)",
                    "  /money-close     reconcile and close the month",
                    "  /saving          create or revise a savings plan",
                    "  /emergency-fund  size and fund the resilience reserve",
                    "  /debt            choose a debt payoff strategy",
                    "  /invest-policy   write an investment policy statement",
                    "  /purchase        evaluate a major purchase",
                    "  /money-scenario  model a what-if financial scenario",
                    "  /receipt         stage a document or receipt (verified before it posts)",
                    "  labels every claim E1..E5 · owns personal money, Revenue OS owns the business",
                    "  12 agents · 20 skills · 7 protocols · 7 schemas",
                    "reference runtime (stdlib Python, self-describe + integrity only):",
                    "  python runtime/os_runtime.py   info · route · event · validate",
                ],
            },
            // ── SYSTEMS — AUTOMATE / LEARN / meta ──────────────────────────
            OsProduct {
                slug: "execution-os",
                name: "Execution OS",
                tagline: "LLM-first personal execution: ambitions into focused commitments, protected work and shipped evidence.",
                group: OsGroup::Systems,
                commands: &[
                    "/execution-os · /execute   personal execution as a closed control loop",
                    "  Capture → Clarify → Select → Commit → Focus → Prove → Review → Adapt",
                    "omega-execution CLI:",
                    "  init --owner        create your execution state",
                    "  boot                open the day (capacity GREEN/AMBER/RED + must-win)",
                    "  capture             inbox a raw outcome/commitment/idea",
                    "  add-outcome         a measurable outcome to pursue",
                    "  add-commitment      a commitment with one physical next action",
                    "  start / complete    complete requires evidence + acceptance",
                    "  focus / focus-end   protect a 15/25/50/90-min block",
                    "  block / unblock     adaptive recovery, each with a next action",
                    "  defer / cancel / delegate   move or hand off a commitment",
                    "  add-promise         a stakeholder promise (notice-by + consequence)",
                    "  halt                close the day (proof + tomorrow's first action)",
                    "  reset               weekly reset (truth + next-week win + experiment)",
                    "  audit               monthly system audit (system change)",
                ],
            },
            OsProduct {
                slug: "operations-automation-os",
                name: "Operations & Automation OS",
                tagline: "See how the work really runs, cut waste, then standardize, delegate or automate with monitoring and recovery.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /operations · /operations-automation-os ):",
                    "  challenge the system before adding tools, automate only suitable work",
                    "  /operations           open the operations diagnostic (default mode)",
                    "  /process-interview    interview process owners and users with examples",
                    "  /process-map          map the current state",
                    "  /value-stream         analyze flow and waste",
                    "  /simplify             remove and simplify work before any tool",
                    "  /automation-audit     find and score automation candidates",
                    "  /automate             create a production-ready automation blueprint",
                    "  /agent-automation     assess an AI-agent workflow (suitability + contract)",
                    "  /future-state         design the target operating model",
                    "  /runbook              create the operating runbook",
                    "  /automation-review    audit live automations and operational health",
                    "  /automation-incident  contain and recover a failed automation",
                    "  every automation carries inputs, exceptions, approvals, evidence,",
                    "  observability and a human recovery path",
                    "  24 specialist agents · 39 skills · 9 protocols · 9 schemas",
                    "runtime/os_runtime.py (reference CLI, stdlib): info · validate · route · event",
                    "→ Builder builds, Quality gates, Review & Governance authorizes risk/policy change",
                ],
            },
            OsProduct {
                slug: "review-governance-os",
                name: "Review & Governance OS",
                tagline: "Turn actions, incidents, metrics and decisions into honest learning, controlled change and explicit policy.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /review · /review-governance-os ):",
                    "  /review            open review (the default command)",
                    "  /daily-review      run daily reflection",
                    "  /weekly-review     run weekly operating review",
                    "  /monthly-review    run monthly metrics review",
                    "  /quarterly-review  run strategic governance",
                    "  /postmortem        analyze an incident or failure (blameless)",
                    "  /policy            create or audit a policy",
                    "  /change-request    submit a consequential change (approval authority)",
                    "  /risk-register     review risks, controls and residual exposure",
                    "  /ai-governance     apply AI risk governance (Govern/Map/Measure/Manage)",
                    "  13 agents · 20 skills · 7 protocols · 7 schemas",
                    "reference runtime (provider-neutral, stdlib only):",
                    "  python runtime/os_runtime.py info       name, version, purpose",
                    "  python runtime/os_runtime.py validate   sha256 integrity check of the pack",
                    "  python runtime/os_runtime.py route /review   resolve a command to its mode",
                    "  python runtime/os_runtime.py event <kind> <json>   append-only event log",
                    "→ governs consequential change across every OS · closes the learning loop with Context & Memory OS",
                ],
            },
            OsProduct {
                slug: "context-memory-os",
                name: "Context & Memory OS",
                tagline: "One trustworthy, inspectable, permissioned memory layer: fact vs inference, no cross-project or identity bleed.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /memory · /context-memory-os ):",
                    "  one trustworthy, inspectable, permissioned memory layer for every OS",
                    "  separates fact from inference and temporary state, no cross-project bleed",
                    "modes (say the word or the /command):",
                    "  /memory          search or inspect authorized memory",
                    "  /remember        propose a memory write (staged with provenance)",
                    "  /ingest          ingest a file or event (screened for injection)",
                    "  /context         compile a purpose-specific context pack",
                    "  /snapshot        create a versioned project/person snapshot",
                    "  /decision-log    record a decision and its rationale",
                    "  /contradiction   resolve conflicting records",
                    "  /memory-audit    audit provenance and access",
                    "  /forget          delete or archive authorized memory",
                    "  /export-memory   create a user-readable export",
                    "  14 agents · 20 skills · 7 protocols · 8 schemas",
                    "reference runtime (python runtime/os_runtime.py, stdlib only):",
                    "  info · route · event · validate   self-describing, integrity-checkable",
                    "→ the canonical shared context layer the other OSes recover from",
                ],
            },
            OsProduct {
                slug: "ai-logic-os",
                name: "AI Logic OS",
                tagline: "Workflow optimizer + agentic-system challenger: code-vs-AI arbitration, default bias NO, finds the logic gaps and specs the fix.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /ai-logic · /ailogic · /ai-logic-os ):",
                    "  a technical adviser whose default bias is NO",
                    "  ~80% deterministic code / ~20% AI judgment — a model call must justify itself",
                    "two jobs:",
                    "  1. optimize a workflow — map · instrument · triage · design · spec · measure · loop",
                    "     triage bins: Codifier · Augmenter · Garder humain · Supprimer",
                    "  2. challenge an agentic system (OmegaOS / an agent / a pipeline / a tool):",
                    "     - where does a LLM do an `if`'s job?",
                    "     - where is a consequential output unverifiable?",
                    "     - where does an irreversible action lack a human gate?",
                    "     - where is the feedback loop missing?",
                    "     - what primitive is absent and should exist?",
                    "  every finding cites proof (file:line / a rule / a log)",
                    "  output always ends with what it does NOT recommend, and why",
                    "omega-ailogic   open the AI Logic master agent in a session",
                ],
            },
            OsProduct {
                slug: "content-os",
                name: "Content OS",
                tagline: "The full content lifecycle: capture, story mining, writing, production, platform-native packaging, publishing and learning.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /content ):",
                    "  /content           open Content OS (strategy mode)",
                    "  /content-gps       define positioning and content system",
                    "  /capture-day       ingest the day as source material",
                    "  /story-mine        find stories, insights and proof",
                    "  /pillar            create a pillar asset",
                    "  /cascade           build a multi-platform waterfall",
                    "  /instagram         Instagram-native package",
                    "  /tiktok            TikTok-native package",
                    "  /youtube           YouTube package",
                    "  /linkedin          LinkedIn package",
                    "  /x                 X package",
                    "  /newsletter        newsletter edition",
                    "  /article           create an article",
                    "  /visual-brief      image/design direction",
                    "  /video-brief       script, shots and edit plan",
                    "  /sound-brief       sound/music/voice direction",
                    "  /content-calendar  build the editorial calendar",
                    "  /content-review    run the performance council",
                    "38 specialist agents · 44 skills · 12 protocols · 10 schemas",
                    "reference runtime (stdlib Python, provider-neutral, not an LLM adapter):",
                    "  runtime/os_runtime.py validate   integrity-check the pack (MANIFEST sha256)",
                    "  runtime/os_runtime.py route /content   resolve a command to its mode",
                    "  runtime/os_runtime.py event <kind> <json>   append a provenance event",
                    "→ Communication Stack: turns life, expertise, products and proof into native multi-platform content",
                ],
            },
            OsProduct {
                slug: "books-os",
                name: "Books OS",
                tagline: "Your library as an operating system: reading, retention and living knowledge.",
                group: OsGroup::Systems,
                commands: &[
                    "Claude / Codex ( /books-os = /alexandria — the librarian ):",
                    "  /setup       calibrate on how YOU learn   ·   /language   reply language",
                    "  /book        full X-Ray of a book   ·   /espresso   90-second version",
                    "  /chapter     a book chapter by chapter   ·   /idea   atlas across many books",
                    "  /compare (/vs)   authors in combat   ·   /apply   to a real business",
                    "  /challenge   10-round sparring on your idea   ·   /decision   decision lab",
                    "  /council     3-5 perspectives   ·   /teach   Feynman triple explanation",
                    "  /quiz /drill   adaptive recall   ·   /cards   flashcards   ·   /review   spaced rep",
                    "  /map (/visual)   diagram   ·   /memory   memory forge   ·   /focus   5-min session",
                    "  /best [topic]   the 50 best books + 50 tips   ·   /bestsellers [niche]   top 100",
                    "  /gem   an underrated idea   ·   /capture /save /applylog   feed the ledger",
                    "omega-books CLI:   open the librarian master agent in a terminal session",
                ],
            },
        ]
    }

    /// "01"…"08" for build-chain OSes (their pipeline position), None for
    /// every other group. Derived from registry order — never hand-numbered.
    pub fn chain_position(&self) -> Option<usize> {
        if self.group != OsGroup::BuildChain {
            return None;
        }
        Self::all()
            .iter()
            .filter(|p| p.group == OsGroup::BuildChain)
            .position(|p| p.slug == self.slug)
            .map(|i| i + 1)
    }

    /// Resolve both the canonical value-chain slug and the five historical
    /// payload names retained for command/install compatibility. Aliases never
    /// create extra products or inflate the canonical suite count.
    pub fn from_slug(slug: &str) -> Option<Self> {
        let canonical = match slug {
            "ideation-os" => "brainstorm-os",
            "researcher-os" => "market-research-os",
            "designer-os" => "design-os",
            "habits-os" => "habit-tracker-os",
            "storytelling-os" => "storyteller-os",
            other => other,
        };
        Self::all()
            .iter()
            .find(|product| product.slug == canonical)
            .copied()
    }

    pub fn legacy_aliases(&self) -> &'static [&'static str] {
        match self.slug {
            "brainstorm-os" => &["ideation-os"],
            "market-research-os" => &["researcher-os"],
            "design-os" => &["designer-os"],
            "habit-tracker-os" => &["habits-os"],
            "storyteller-os" => &["storytelling-os"],
            _ => &[],
        }
    }
}

/// Coarse stage derived from concrete local surfaces. The strongest stage is
/// deliberately `Testable`, not `Verified`: finding test files does not prove
/// that anybody ran them successfully against this revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsReadinessLevel {
    /// Directory missing or empty scaffold.
    Scaffold,
    /// Prompts/docs/packaging exist, but no local runtime entrypoint was found.
    Reference,
    /// A runtime surface exists, but no test surface was found.
    Runnable,
    /// Runtime and test surfaces both exist; tests have not been executed here.
    Testable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsManifestStatus {
    Missing,
    Invalid,
    Valid,
}

/// Evidence behind the readiness label. These fields let the TUI state what it
/// actually observed instead of compressing any arbitrary extra file into a
/// green "integrated" badge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OsReadiness {
    pub level: OsReadinessLevel,
    pub directory_present: bool,
    pub master_present: bool,
    pub payload_present: bool,
    pub manifest: OsManifestStatus,
    pub runtime_present: bool,
    pub tests_present: bool,
    /// Raw `events.schema_status` from MANIFEST.json when present. Values such
    /// as `stub` remain visible and never imply runtime verification.
    pub event_schema_status: Option<String>,
}

impl OsReadiness {
    fn missing() -> Self {
        Self {
            level: OsReadinessLevel::Scaffold,
            directory_present: false,
            master_present: false,
            payload_present: false,
            manifest: OsManifestStatus::Missing,
            runtime_present: false,
            tests_present: false,
            event_schema_status: None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self.level {
            OsReadinessLevel::Scaffold if !self.directory_present => "directory missing",
            OsReadinessLevel::Scaffold => "scaffold only",
            OsReadinessLevel::Reference => "reference surface only",
            OsReadinessLevel::Runnable => "runtime present (tests not found)",
            OsReadinessLevel::Testable => "runtime + tests present (not executed)",
        }
    }

    pub fn manifest_label(&self) -> &'static str {
        match self.manifest {
            OsManifestStatus::Missing => "missing",
            OsManifestStatus::Invalid => "invalid manifest",
            OsManifestStatus::Valid => "valid manifest",
        }
    }
}

/// One row for the OS tab: identity + where it lives here + integration state.
#[derive(Debug, Clone)]
pub struct OsEntry {
    pub product: OsProduct,
    pub readiness: OsReadiness,
    /// `<os_root>/<slug>` when a root was found (the dir itself may not exist
    /// yet for an OS added to the registry before its folder).
    pub path: Option<PathBuf>,
    /// A dedicated Telegram bot is wired for this OS (`os-<slug>` entry in
    /// `~/.omega/agent-bots.json`, linked via `omega-os-bot`).
    pub bot_linked: bool,
}

/// Locate the `OS/` suite root. Order: `OMEGA_OS_ROOT` env override, then the
/// repo relative to the running exe (a dev box running `target/…/omega`), then
/// a walk up from the current dir, then well-known checkouts, then the
/// INSTALLED copy `~/.omega/os` — last, so a checkout always wins and an
/// operator editing the suite sees it immediately. Same resolution grammar as
/// `marketing::capabilities_toml_path`.
pub fn os_root() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("OMEGA_OS_ROOT") {
        let p = p.trim();
        if !p.is_empty() {
            let pb = PathBuf::from(p);
            if pb.is_dir() {
                return Some(pb);
            }
        }
    }

    let is_suite_root = |d: &Path| -> Option<PathBuf> {
        let cand = d.join("OS");
        // Require a known slug inside, so a random `OS/` dir on the walk-up
        // path can't hijack the suite.
        if cand.is_dir() && cand.join("mindset-os").is_dir() {
            return Some(cand);
        }
        None
    };

    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        while let Some(d) = dir {
            if let Some(found) = is_suite_root(&d) {
                return Some(found);
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = Some(cwd);
        while let Some(d) = dir {
            if let Some(found) = is_suite_root(&d) {
                return Some(found);
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
    for base in [
        home.join("Station").join("SideBusiness").join("OmegaOS"),
        home.join("OmegaOS"),
    ] {
        if let Some(found) = is_suite_root(&base) {
            return Some(found);
        }
    }

    let omega_dir = std::env::var("OMEGA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".omega"));
    let installed = omega_dir.join("os");
    if installed.is_dir() {
        return Some(installed);
    }
    None
}

/// Inspect bounded local filesystem evidence. Symlinked directories are not
/// followed, and the walk is capped, so entering the OS tab cannot recurse
/// forever through a malformed payload.
fn dir_readiness(dir: &Path) -> OsReadiness {
    if !dir.is_dir() {
        return OsReadiness::missing();
    }

    let manifest_path = dir.join("MANIFEST.json");
    let (manifest, event_schema_status) = if manifest_path.is_file() {
        match std::fs::read_to_string(&manifest_path)
            .ok()
            .and_then(|raw| {
                serde_json::from_str::<serde_json::Value>(&raw)
                    .ok()
                    .filter(|manifest| valid_os_manifest(manifest, dir))
            }) {
            Some(value) => (
                OsManifestStatus::Valid,
                value
                    .pointer("/events/schema_status")
                    .and_then(|value| value.as_str())
                    .map(str::to_string),
            ),
            None => (OsManifestStatus::Invalid, None),
        }
    } else {
        (OsManifestStatus::Missing, None)
    };

    let master_present = dir.join("MASTER.md").is_file();
    let mut payload_present = false;
    let mut runtime_present = false;
    let mut tests_present = false;
    let mut pending = vec![(dir.to_path_buf(), 0usize)];
    let mut visited = 0usize;

    while let Some((current, depth)) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            visited += 1;
            if visited > 4096 {
                break;
            }
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) if !file_type.is_symlink() => file_type,
                _ => continue,
            };
            let Ok(relative) = path.strip_prefix(dir) else {
                continue;
            };
            let components: Vec<String> = relative
                .components()
                .map(|part| part.as_os_str().to_string_lossy().to_lowercase())
                .collect();
            let Some(name) = components.last().map(String::as_str) else {
                continue;
            };
            let top = components.first().map(String::as_str).unwrap_or(name);
            if file_type.is_file() {
                let is_scaffold = depth == 0 && matches!(name, "readme.md" | "master.md")
                    || name.starts_with('.');
                if !is_scaffold {
                    payload_present = true;
                }

                let in_tests = components
                    .iter()
                    .any(|part| part == "tests" || part == "test");
                let test_name = name.starts_with("test_")
                    || name.contains(".test.")
                    || name.contains("_test.")
                    || name.ends_with("_spec.rs");
                if (in_tests || test_name) && is_executable_surface(&path, name, top) {
                    tests_present = true;
                }

                let runtime_dir =
                    matches!(top, "bin" | "scripts" | "src" | "runtime" | "app" | "cli");
                let root_entrypoint = depth == 0
                    && matches!(
                        name,
                        "main.rs" | "main.py" | "app.py" | "index.ts" | "index.js"
                    );
                if root_entrypoint || (runtime_dir && is_executable_surface(&path, name, top)) {
                    runtime_present = true;
                }
            }

            if file_type.is_dir() && depth < 4 {
                pending.push((path, depth + 1));
            }
        }
        if visited > 4096 {
            break;
        }
    }

    let level = if runtime_present && tests_present {
        OsReadinessLevel::Testable
    } else if runtime_present {
        OsReadinessLevel::Runnable
    } else if master_present || payload_present || dir.join("README.md").is_file() {
        OsReadinessLevel::Reference
    } else {
        OsReadinessLevel::Scaffold
    };

    OsReadiness {
        level,
        directory_present: true,
        master_present,
        payload_present,
        manifest,
        runtime_present,
        tests_present,
        event_schema_status,
    }
}

fn valid_os_manifest(value: &serde_json::Value, dir: &Path) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let has_identity = ["name", "display_name", "id"].iter().any(|field| {
        object
            .get(*field)
            .and_then(serde_json::Value::as_str)
            .is_some_and(|v| !v.trim().is_empty())
    });
    let slug = object
        .get("slug")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty());
    let identity_matches_directory = dir
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(OsProduct::from_slug)
        .is_none_or(|product| slug == product.slug.strip_suffix("-os"));
    has_identity
        && identity_matches_directory
        && slug.is_some()
        && object
            .get("version")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
        && object
            .get("files")
            .and_then(serde_json::Value::as_array)
            .is_some()
}

fn is_executable_surface(path: &Path, name: &str, top: &str) -> bool {
    if top == "bin" && !matches!(name, "readme" | "readme.md") {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if path
                .metadata()
                .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
            {
                return true;
            }
        }
    }
    Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension,
                "rs" | "py"
                    | "js"
                    | "mjs"
                    | "cjs"
                    | "ts"
                    | "tsx"
                    | "jsx"
                    | "sh"
                    | "bash"
                    | "zsh"
                    | "go"
                    | "rb"
                    | "php"
                    | "bats"
            )
        })
}

/// Bot keys present in `~/.omega/agent-bots.json` (one read for the list).
fn linked_bot_keys() -> std::collections::HashSet<String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
    let omega_dir = std::env::var("OMEGA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".omega"));
    let Ok(raw) = std::fs::read_to_string(omega_dir.join("agent-bots.json")) else {
        return Default::default();
    };
    serde_json::from_str::<serde_json::Value>(&raw)
        .ok()
        .and_then(|v| v.as_object().map(|map| map.keys().cloned().collect()))
        .unwrap_or_default()
}

fn product_path(root: &Path, product: &OsProduct) -> PathBuf {
    let canonical = root.join(product.slug);
    if canonical.is_dir() {
        return canonical;
    }
    product
        .legacy_aliases()
        .iter()
        .map(|alias| root.join(alias))
        .find(|alias| alias.is_dir())
        .unwrap_or(canonical)
}

/// The whole suite with per-machine status, in product order.
pub fn list_os_entries() -> Vec<OsEntry> {
    let root = os_root();
    let bots = linked_bot_keys();
    OsProduct::all()
        .iter()
        .map(|p| {
            let path = root.as_ref().map(|root| product_path(root, p));
            let readiness = path
                .as_deref()
                .map(dir_readiness)
                .unwrap_or_else(OsReadiness::missing);
            OsEntry {
                product: *p,
                readiness,
                path,
                bot_linked: bots.contains(&format!("os-{}", p.slug)),
            }
        })
        .collect()
}

impl OsEntry {
    /// Readiness glyph: absence/scaffold, reference, runnable, then testable.
    pub fn glyph(&self) -> &'static str {
        match self.readiness.level {
            OsReadinessLevel::Scaffold => "⚪",
            OsReadinessLevel::Reference => "🔵",
            OsReadinessLevel::Runnable => "🟡",
            OsReadinessLevel::Testable => "🧪",
        }
    }

    pub fn status_label(&self) -> &'static str {
        self.readiness.label()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The value-chain suite, in registry order, grouped and contiguous:
    /// PERSONAL, then the BUILD CHAIN (01→08), then GROWTH, then SYSTEMS.
    /// 28 = 23 value-chain OSes + books-os (a working OS with no value-chain
    /// twin, kept in Systems rather than dropped) + seductive-os (personal
    /// magnetism) + intuitive-os (judgment calibration) + identity-shift-os
    /// (evidence-based transformation) + journal-os (the nightly MIRROR
    /// review), all four Personal group.
    const EXPECTED: &[(&str, OsGroup)] = &[
        ("mindset-os", OsGroup::Personal),
        ("health-energy-os", OsGroup::Personal),
        ("habit-tracker-os", OsGroup::Personal),
        ("alignment-os", OsGroup::Personal),
        ("seductive-os", OsGroup::Personal),
        ("intuitive-os", OsGroup::Personal),
        ("identity-shift-os", OsGroup::Personal),
        ("journal-os", OsGroup::Personal),
        ("strategy-portfolio-os", OsGroup::BuildChain),
        ("brainstorm-os", OsGroup::BuildChain),
        ("market-research-os", OsGroup::BuildChain),
        ("blueprint-os", OsGroup::BuildChain),
        ("design-os", OsGroup::BuildChain),
        ("stepper-os", OsGroup::BuildChain),
        ("builder-os", OsGroup::BuildChain),
        ("quality-evaluation-release-os", OsGroup::BuildChain),
        ("storyteller-os", OsGroup::Growth),
        ("revenue-os", OsGroup::Growth),
        ("delivery-customer-success-os", OsGroup::Growth),
        ("relationship-network-os", OsGroup::Growth),
        ("wealth-capital-os", OsGroup::Growth),
        ("execution-os", OsGroup::Systems),
        ("operations-automation-os", OsGroup::Systems),
        ("review-governance-os", OsGroup::Systems),
        ("context-memory-os", OsGroup::Systems),
        ("ai-logic-os", OsGroup::Systems),
        ("content-os", OsGroup::Systems),
        ("books-os", OsGroup::Systems),
    ];

    #[test]
    fn suite_is_the_full_value_chain_in_order() {
        let got: Vec<(&str, OsGroup)> =
            OsProduct::all().iter().map(|p| (p.slug, p.group)).collect();
        assert_eq!(
            got.len(),
            28,
            "28 = 23 value-chain OSes + books-os + seductive-os + intuitive-os\
             + identity-shift-os + journal-os"
        );
        assert_eq!(got, EXPECTED);
    }

    #[test]
    fn legacy_slugs_resolve_without_inflating_the_canonical_suite() {
        let aliases = [
            ("ideation-os", "brainstorm-os"),
            ("researcher-os", "market-research-os"),
            ("designer-os", "design-os"),
            ("habits-os", "habit-tracker-os"),
            ("storytelling-os", "storyteller-os"),
        ];
        assert_eq!(OsProduct::all().len(), EXPECTED.len());
        for (alias, canonical) in aliases {
            let product = OsProduct::from_slug(alias).expect("legacy slug must resolve");
            assert_eq!(product.slug, canonical);
            assert!(product.legacy_aliases().contains(&alias));
        }
        assert!(OsProduct::from_slug("not-an-os").is_none());
    }

    #[test]
    fn legacy_directory_is_a_fallback_but_canonical_directory_wins() {
        let tmp = tempfile::tempdir().unwrap();
        let product = OsProduct::from_slug("brainstorm-os").unwrap();
        let legacy = tmp.path().join("ideation-os");
        std::fs::create_dir(&legacy).unwrap();
        assert_eq!(product_path(tmp.path(), &product), legacy);

        let canonical = tmp.path().join("brainstorm-os");
        std::fs::create_dir(&canonical).unwrap();
        assert_eq!(product_path(tmp.path(), &product), canonical);
    }

    #[test]
    fn groups_are_contiguous_in_declaration_order() {
        // Each group appears exactly once as a contiguous run, in the order
        // Personal → BuildChain → Growth → Systems (what the TUI renders).
        let order = [
            OsGroup::Personal,
            OsGroup::BuildChain,
            OsGroup::Growth,
            OsGroup::Systems,
        ];
        let mut seen: Vec<OsGroup> = Vec::new();
        let mut last: Option<OsGroup> = None;
        for p in OsProduct::all() {
            if last != Some(p.group) {
                assert!(
                    !seen.contains(&p.group),
                    "group {:?} is not contiguous",
                    p.group
                );
                seen.push(p.group);
                last = Some(p.group);
            }
        }
        assert_eq!(seen, order, "groups render in value-chain order");
    }

    #[test]
    fn build_chain_positions_are_01_to_08_unique() {
        let positions: Vec<usize> = OsProduct::all()
            .iter()
            .filter(|p| p.group == OsGroup::BuildChain)
            .map(|p| p.chain_position().expect("build-chain OS has a position"))
            .collect();
        assert_eq!(positions, (1..=8).collect::<Vec<_>>());
        // Every non-BuildChain OS returns None.
        for p in OsProduct::all() {
            if p.group != OsGroup::BuildChain {
                assert_eq!(p.chain_position(), None, "{} must have no position", p.slug);
            }
        }
        // Named anchors of the chain.
        let pos = |slug: &str| {
            OsProduct::all()
                .iter()
                .find(|p| p.slug == slug)
                .unwrap()
                .chain_position()
        };
        assert_eq!(pos("strategy-portfolio-os"), Some(1));
        assert_eq!(pos("blueprint-os"), Some(4));
        assert_eq!(pos("builder-os"), Some(7));
        assert_eq!(pos("quality-evaluation-release-os"), Some(8));
        assert_eq!(pos("books-os"), None);
        assert_eq!(pos("mindset-os"), None);
    }

    #[test]
    fn every_slug_resolves_to_an_os_dir_when_a_root_is_present() {
        // With OMEGA_OS_ROOT unset the suite root may not resolve on a bare
        // CI box; only assert dir presence when a root actually resolves.
        if let Some(root) = os_root() {
            for p in OsProduct::all() {
                let dir = root.join(p.slug);
                assert!(
                    dir.is_dir(),
                    "OS payload dir missing for {}: {}",
                    p.slug,
                    dir.display()
                );
            }
        }
    }

    #[test]
    fn every_physical_os_directory_is_canonical_or_a_known_legacy_alias() {
        if let Some(root) = os_root() {
            for entry in std::fs::read_dir(root).unwrap().flatten() {
                if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                    continue;
                }
                let name = entry.file_name();
                let name = name.to_string_lossy();
                assert!(
                    OsProduct::from_slug(&name).is_some(),
                    "unclassified OS payload directory: {name}"
                );
            }
        }
    }

    #[test]
    fn canonical_os_manifests_are_structurally_valid_and_match_identity() {
        if let Some(root) = os_root() {
            for product in OsProduct::all() {
                let dir = root.join(product.slug);
                assert_eq!(
                    dir_readiness(&dir).manifest,
                    OsManifestStatus::Valid,
                    "invalid or identity-drifted manifest for {}",
                    product.slug
                );
            }
        }
    }

    #[test]
    fn readiness_distinguishes_reference_runtime_and_test_surfaces() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        let empty = dir_readiness(dir);
        assert_eq!(empty.level, OsReadinessLevel::Scaffold);
        assert!(!empty.payload_present);

        std::fs::write(dir.join("README.md"), "# reference").unwrap();
        std::fs::write(dir.join("MASTER.md"), "# master prompt").unwrap();
        let reference = dir_readiness(dir);
        assert_eq!(reference.level, OsReadinessLevel::Reference);
        assert!(reference.master_present);

        // An arbitrary extra document is payload, but it is not a runtime.
        std::fs::write(dir.join("notes.txt"), "not executable").unwrap();
        let documented = dir_readiness(dir);
        assert_eq!(documented.level, OsReadinessLevel::Reference);
        assert!(documented.payload_present);
        assert!(!documented.runtime_present);

        std::fs::create_dir_all(dir.join("scripts")).unwrap();
        std::fs::write(dir.join("scripts/run.py"), "print('run')").unwrap();
        let runnable = dir_readiness(dir);
        assert_eq!(runnable.level, OsReadinessLevel::Runnable);
        assert!(runnable.runtime_present);
        assert!(!runnable.tests_present);

        std::fs::write(dir.join("scripts/test_os.py"), "assert True").unwrap();
        let testable = dir_readiness(dir);
        assert_eq!(testable.level, OsReadinessLevel::Testable);
        assert!(testable.tests_present);
        assert!(testable.label().contains("not executed"));
    }

    #[test]
    fn empty_or_narrative_runtime_and_test_directories_do_not_promote_readiness() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("MASTER.md"), "# reference").unwrap();
        std::fs::create_dir(dir.join("scripts")).unwrap();
        std::fs::create_dir(dir.join("tests")).unwrap();
        std::fs::create_dir(dir.join("bin")).unwrap();
        std::fs::write(dir.join("scripts/README.md"), "not executable").unwrap();
        std::fs::write(dir.join("tests/plan.md"), "not a test program").unwrap();
        std::fs::write(dir.join("bin/notes.txt"), "not executable").unwrap();

        let readiness = dir_readiness(dir);
        assert_eq!(readiness.level, OsReadinessLevel::Reference);
        assert!(!readiness.runtime_present);
        assert!(!readiness.tests_present);
    }

    #[test]
    fn readiness_reports_manifest_validity_and_stub_schema_without_promoting_it() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("MASTER.md"), "# prompt").unwrap();
        std::fs::write(dir.join("MANIFEST.json"), "not json").unwrap();
        let invalid = dir_readiness(dir);
        assert_eq!(invalid.manifest, OsManifestStatus::Invalid);
        assert_eq!(invalid.level, OsReadinessLevel::Reference);

        std::fs::write(
            dir.join("MANIFEST.json"),
            r#"{"events":{"schema_status":"stub"}}"#,
        )
        .unwrap();
        let incomplete = dir_readiness(dir);
        assert_eq!(incomplete.manifest, OsManifestStatus::Invalid);

        std::fs::write(
            dir.join("MANIFEST.json"),
            r#"{"name":"Fixture OS","slug":"fixture","version":"1.0.0","files":[],"events":{"schema_status":"stub"}}"#,
        )
        .unwrap();
        std::fs::create_dir_all(dir.join("runtime")).unwrap();
        std::fs::write(dir.join("runtime/main.rs"), "fn main() {}").unwrap();
        let stub = dir_readiness(dir);
        assert_eq!(stub.manifest, OsManifestStatus::Valid);
        assert_eq!(stub.event_schema_status.as_deref(), Some("stub"));
        assert_eq!(stub.level, OsReadinessLevel::Runnable);
        assert!(!stub.tests_present);
    }

    #[test]
    fn manifest_identity_must_match_a_known_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("brainstorm-os");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(
            dir.join("MANIFEST.json"),
            r#"{"name":"Wrong OS","slug":"market-research","version":"1.0.0","files":[]}"#,
        )
        .unwrap();
        assert_eq!(dir_readiness(&dir).manifest, OsManifestStatus::Invalid);
    }

    #[test]
    fn missing_directory_is_not_reported_as_an_awaiting_payload() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = dir_readiness(&tmp.path().join("not-created"));
        assert_eq!(missing.level, OsReadinessLevel::Scaffold);
        assert!(!missing.directory_present);
        assert_eq!(missing.label(), "directory missing");
    }

    #[test]
    fn entries_cover_the_whole_suite() {
        let entries = list_os_entries();
        assert_eq!(entries.len(), OsProduct::all().len());
    }
}
