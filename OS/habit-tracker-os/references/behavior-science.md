# Behavioral Foundations and Intervention Map

## Contents

1. Evidence policy
2. Habit formation
3. Implementation intentions
4. COM-B diagnosis
5. Behavior-change techniques
6. Motivation and autonomy
7. Motivational interviewing
8. Self-monitoring and feedback
9. Just-in-time adaptation
10. Lapses and self-compassion
11. Stoicism
12. LLM-specific evidence limits
13. Technique selection matrix

## 1. Evidence policy

Habit Tracker {OS} is evidence-informed, not a medical intervention. Treat frameworks as decision aids, not universal laws. Prefer observable behavior and within-person experiments. Keep these labels distinct:

- **supported mechanism**: reasonably grounded by published research;
- **design heuristic**: plausible and useful, but not established for this user;
- **user belief/practice**: valid as personal meaning, not presented as scientific causality;
- **unknown**: requires observation or professional assessment.

Do not repeat popular simplifications such as “a habit takes 21 days.” Time to automaticity varies materially by behavior, context, and person.

## 2. Habit formation

Lally and colleagues followed 96 people repeating a selected behavior in a stable context for 12 weeks. Automaticity followed an asymptotic rather than linear pattern, with substantial variation; one missed opportunity did not materially derail the process in their model. Use this to prioritize repeated behavior in a stable context and to avoid catastrophic framing after a miss. Do not turn the often-cited central estimate into a deadline or promise. Source: [Lally et al., 2010, European Journal of Social Psychology](https://doi.org/10.1002/ejsp.674).

OS implementation:

- store cue/context separately from frequency;
- track automaticity by an occasional self-rating, not completion count alone;
- preserve continuity after a single miss;
- compare habit classes separately because complexity matters;
- avoid fixed “formation completed” dates.

## 3. Implementation intentions

Implementation intentions connect a specified situation with a specified response: “If/when X occurs, then I will do Y.” A meta-analysis of 94 independent tests reported a medium-to-large overall effect on goal attainment, while later work emphasizes that effects depend on plan format and motivation. Use if-then planning as a contract structure, not magic syntax. Source: [Gollwitzer & Sheeran, 2006](https://cancercontrol.cancer.gov/sites/default/files/2020-06/goal_intent_attain.pdf).

OS implementation:

- require a discriminable cue;
- bind one cue to one observable response;
- add an obstacle/fallback clause;
- test whether the cue actually occurs and is noticed;
- avoid vague cues such as “when I have time.”

## 4. COM-B diagnosis

The Behavior Change Wheel places capability, opportunity, and motivation around behavior. Capability and motivation each include multiple forms; opportunity includes environmental and social conditions. Use COM-B to diagnose the dominant constraint before suggesting a technique. Source: [Michie, van Stralen & West, 2011, Implementation Science](https://doi.org/10.1186/1748-5908-6-42).

Operational questions:

| Domain | Discriminating question | Typical response class |
| --- | --- | --- |
| Physical capability | Can the person safely perform the action? | scale, teach, rehabilitate, professional input |
| Psychological capability | Do they understand and remember what to do? | simplify, rehearse, cue, checklist |
| Physical opportunity | Does the environment make it possible and easy? | access, timing, friction, layout |
| Social opportunity | Do norms or people support or obstruct it? | support, boundary, accountability |
| Reflective motivation | Is the plan consciously valued and believed feasible? | values, tradeoff, plan, confidence |
| Automatic motivation | What impulse, emotion, or learned response competes? | cue redesign, substitution, urge protocol |

Never treat every miss as a motivation deficit.

## 5. Behavior-change techniques

The Behavior Change Technique Taxonomy v1 describes 93 techniques in 16 clusters, improving precision in intervention specification. Habit Tracker {OS} uses a governed subset rather than firing many techniques at once. Source: [Michie et al., 2013, Annals of Behavioral Medicine](https://doi.org/10.1007/s12160-013-9486-6).

Default approved subset:

- goal setting for behavior;
- action planning;
- problem solving;
- self-monitoring of behavior;
- feedback on behavior;
- prompts/cues;
- graded tasks;
- habit formation through contextual repetition;
- behavior substitution;
- restructuring the physical or social environment;
- social support when chosen;
- review of behavioral goals;
- instruction and demonstration when needed;
- pros/cons for genuine ambivalence;
- reducing negative emotions through safe nonclinical methods;
- conserving mental resources.

Use one primary technique and name it in the stored intervention record. Incentives, punishments, public comparison, and financial penalties require explicit user choice and a separate ethical review; never default to them.

## 6. Motivation and autonomy

Self-Determination Theory distinguishes higher-quality autonomous motivation from controlled pressure and highlights autonomy, competence, and relatedness. Use it to preserve user authorship, progressive mastery, and chosen human support. Sources: [Ryan & Deci, 2000](https://selfdeterminationtheory.org/SDT/documents/2000_RyanDeci_SDT.pdf) and [Ryan & Deci, 2006](https://selfdeterminationtheory.org/SDT/documents/2006_RyanDeci_Self-RegulationProblemofHumanAutonomy.pdf).

OS implementation:

- ask why the behavior matters in the user’s own words;
- give bounded choices instead of commands;
- calibrate task difficulty to build competence;
- support relatedness without making the agent a substitute relationship;
- detect guilt, status anxiety, or external pressure as possible controlled motivation;
- never conclude that intrinsic motivation is required for every necessary task.

## 7. Motivational interviewing

Motivational interviewing is a collaborative method for evoking a person’s own reasons and commitment for change. Mechanistic research emphasizes client “change talk,” although causal pathways remain complex. Use the relational stance and selected communication methods without claiming to deliver psychotherapy. Sources: [Miller & Rose, 2009](https://pmc.ncbi.nlm.nih.gov/articles/PMC2759607/) and [Resnicow & McMaster, 2012](https://doi.org/10.1186/1479-5868-9-19).

OS implementation:

- reflect ambivalence accurately;
- ask permission to offer information;
- elicit desire, ability, reasons, need, and next commitment;
- avoid arguing, shaming, or escalating persuasion;
- summarize the user’s own reasons before planning;
- switch to direct safety guidance when immediate risk overrides ordinary coaching.

## 8. Self-monitoring and feedback

Digital habit interventions commonly use self-monitoring, goal setting, and prompts/cues. Reviews across domains suggest potential benefit, but effects vary and evidence does not justify assuming that more logging is always better. Sources: [Zhu et al., 2024, JMIR](https://www.jmir.org/2024/1/e54375/) and [Compernolle et al., 2019](https://doi.org/10.1186/s12966-019-0824-3).

OS implementation:

- minimize response burden;
- allow one-message logging;
- track data completeness;
- provide informational, not moralizing, feedback;
- ask extra questions only when they change intervention selection;
- allow temporary low-data mode in recovery seasons.

## 9. Just-in-time adaptation

Ecological momentary assessment and just-in-time adaptive approaches aim to tailor support to changing states and contexts. Recent work explores adaptive questioning to reduce burden, but real-world design and causal evaluation remain complex. Source: [Schneider et al., 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC10450096/).

OS implementation:

- trigger an intervention at a decision point, not continuously;
- choose a minimal question based on uncertainty;
- respect cooldown windows and notification pressure;
- do not infer mood, craving, or risk from passive data without validation;
- log why an intervention was triggered;
- treat timing policies as experiments.

## 10. Lapses and self-compassion

Research on behavior-change lapses suggests self-compassion may support self-efficacy and intention to continue, though evidence is not identical across behaviors or populations. Use nonjudgmental debriefing to preserve learning and re-entry. Sources: [Hagerman et al., 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC10543633/) and [DiClemente, 2022](https://pmc.ncbi.nlm.nih.gov/articles/PMC9014843/).

OS implementation:

- separate accountability from self-attack;
- record the behavior accurately;
- identify a nearby choice point;
- select one repair move;
- calculate recovery latency;
- avoid all-or-nothing reset language.

## 11. Stoicism

Stoic practice contributes a philosophical reflection framework, not an empirical behavior-change protocol. Use it as a user-selected lens for attention, judgment, values, and action:

- dichotomy/trichotomy of control;
- examining impressions before assent;
- virtue as a quality of action;
- premeditation of plausible obstacles;
- evening review;
- acceptance of outcomes after responsible action.

Do not present Stoicism as proof that circumstances do not matter. Do not use it to suppress emotion, deny health conditions, or shame someone for needing support.

## 12. LLM-specific evidence limits

Early research indicates LLM coaching can provide conversational flexibility and personalized planning, but efficacy and safety evidence remains emerging and domain-specific. A 2025 randomized study examined a GPT-4 motivational-interviewing agent; related prototypes such as GPTCoach show how coaching programs and health data can be integrated. Broader mental-health reviews emphasize uncertainty, evaluation gaps, bias, privacy, and safety risks. Sources: [Meyer & Elsweiler, 2025](https://doi.org/10.1016/j.ijhcs.2025.103514), [GPTCoach, CHI 2025](https://dl.acm.org/doi/10.1145/3706598.3713819), and [Hua et al., 2025, npj Digital Medicine](https://www.nature.com/articles/s41746-025-01611-4).

Design implications:

- keep deterministic records outside the LLM narrative;
- evaluate parsing, coaching, safety, and analytics separately;
- never market the system as a validated treatment without appropriate trials;
- require provenance for memory and completion;
- make human escalation explicit;
- test for sycophancy, coercion, hallucination, overdependence, and unsafe advice;
- log model/version/prompt for evaluated decisions where privacy permits.

## 13. Technique selection matrix

| Pattern | Minimum evidence | Primary technique | Avoid |
| --- | --- | --- | --- |
| User forgets after a stable event | 2+ misses tied to memory | prompt/cue + implementation intention | motivation lecture |
| Action feels too large | explicit statement or repeated partials | graded task + minimum version | “push harder” |
| Environment repeatedly blocks | 2+ similar barriers | environment restructuring | self-blame |
| Genuine ambivalence | user expresses both change and sustain talk | MI reflection + pros/cons | argument |
| Urge in progress | explicit live urge | friction + replacement + short delay | long analysis |
| Lapse after high-risk cue | explicit lapse and antecedent | problem solving + recovery rule | streak reset drama |
| Plan works but feels controlled | explicit pressure/guilt | autonomy support + values check | rewards escalation |
| Overload across many habits | active load exceeds capacity | recovery season + pruning | adding trackers |
| Unclear cause | insufficient/contradictory data | one discriminating question | confident diagnosis |

