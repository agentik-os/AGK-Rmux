# Specialist Council Bench

## Contents

1. Selection law
2. Domain specialists
3. Council recipes
4. Specialist prompt contract
5. Anti-overstaffing rules

## 1. Selection law

The three core cells remain mandatory in Council and Deep mode. Add specialists only when a named uncertainty could change the recommendation. Prefer one specialist with a sharp mandate over five generic expert personas.

Score candidate specialists from 0–2 on:

- decision impact;
- knowledge specificity;
- independence from existing cells;
- cost of being wrong.

Select the highest one or two scores. Bundle their hats into an existing cell when agent slots are limited. Do not ask specialists to repeat the whole brainstorm.

## 2. Domain specialists

| Specialist | Use when | Must return | Must not do |
| --- | --- | --- | --- |
| Customer Researcher | Actor, problem, behavior, switching, or trust is uncertain | Research questions, evidence gaps, interview/prototype tests, invented-evidence warnings | Fabricate quotes, demand, or segments |
| Market/Category Strategist | Category, alternatives, market timing, or positioning matters | Category map, substitutes, wedge hypotheses, research needs | Present unsourced market size as fact |
| Distribution/Growth Strategist | Adoption or channel is central | Channel-mechanism fit, loops, acquisition constraints, anti-spam guardrails | Assume virality or cheap acquisition |
| Sales/Enterprise Buyer | B2B buying, procurement, or stakeholder politics matter | Buying committee, objections, proof, cycle risks, pilot logic | Treat user and buyer as identical |
| Behavioral Scientist | Habits, motivation, attention, or social behavior matters | Mechanism hypothesis, biases, harms, measurement plan | Diagnose or claim clinical efficacy |
| Product/UX Critic | Interaction model or comprehension is uncertain | Critical journeys, friction/trust states, prototype questions | Jump to a complete UI specification |
| Surface/Embodiment Strategist | Mobile, web, desktop, multi-surface, chat, physical, service, ambient, or no-interface form could change value | User-moment map, native-affordance comparison, surface thesis, role map, discriminating prototype | Default to an app, mirror features, design screens, or choose a stack |
| Technical Feasibility Architect | A mechanism depends on uncertain technical capability | Feasibility envelope, dependencies, failure modes, spike plan | Choose a stack prematurely |
| AI Systems/Evals Specialist | AI autonomy, memory, tools, quality, or safety is central | AI job boundary, context/tool risks, evals, fallback, cost/latency hypotheses | Treat model output as deterministic |
| Data/Privacy/Security Examiner | Sensitive data, identity, surveillance, or abuse is material | Threat actors, data minimization, consent, failure containment | Produce generic compliance boilerplate |
| Legal/Regulatory Scout | Regulation could invalidate the model | Jurisdictional unknowns, counsel questions, avoid/verify boundaries | Give definitive legal advice without sources |
| Economist/Game Theorist | Marketplace, network, community, incentives, or pricing matters | Value flows, adverse selection, gaming, equilibrium risks | Assume actors behave altruistically |
| Finance/Capital Strategist | Capital intensity, margin, runway, or portfolio choice matters | Economic sensitivities, capital gates, downside cases | Invent precise forecasts |
| Network/Community Architect | Membership, reputation, matching, or contribution matters | Cold-start, density, governance, status, moderation, exit risks | Equate member count with network value |
| Luxury/Hospitality Curator | Rarity, access, service, curation, or premium trust matters | Value beyond price, service burden, taste/trust risks, scarcity integrity | Use luxury as decoration or artificial scarcity |
| Brand/Culture Semiotician | Meaning, identity, language, status, or cultural fit matters | Symbolic territory, narrative tensions, rejection signals | Replace product value with aesthetics |
| Creative Director/Story Architect | A creative concept, campaign, format, or world needs originality | Territories, emotional effect, narrative engine, clichés to avoid | Generate only taglines or mood adjectives |
| Futures/Scenario Strategist | Long horizon or unstable technology makes one forecast fragile | 3–4 plausible scenarios, signposts, robust/no-regret moves | Predict one future with false certainty |
| Scientific Evidence Reviewer | Health, education, psychology, or science claims matter | Evidence hierarchy, uncertainty, study/review questions | Overstate causality or medical efficacy |
| Operations/Service Designer | Repeated human delivery is part of the product | Front/backstage workflow, bottlenecks, quality variance, recovery | Hide service labor behind “AI” |
| Sustainability/Externalities Examiner | Physical supply, environmental or social externalities matter | Boundary, rebound effects, externalized costs, mitigations | Moralize without causal analysis |
| Founder-Fit/Portfolio Critic | Opportunity cost, identity, focus, or capability matters | Founder edge, energy burden, distraction risk, kill/park criteria | Turn preference into objective market fact |

## 3. Council recipes

Recipes select hats, not fixed numbers of agents.

### AI/software product

Core cells + Customer Researcher + AI Systems/Evals Specialist. Add Surface/Embodiment Strategist when the primary surface or multi-surface sequence is unresolved. Add Data/Privacy/Security only when the data or autonomy risk is material.

### Mobile, desktop, web, or multi-surface product

Core cells + Surface/Embodiment Strategist + Product/UX Critic. Require a concept-level surface decision, but defer journeys, screens, requirements, and technical architecture to Blueprint {OS}.

### Marketplace or private network

Core cells + Economist/Game Theorist + Network/Community Architect. Add Luxury/Hospitality Curator only when premium access or service is a causal part of value.

### New business or offer

Core cells + Market/Category Strategist + Distribution/Growth or Sales/Enterprise Buyer according to buyer type.

### Creative work, media, brand, or content

Core cells + Creative Director/Story Architect + Brand/Culture Semiotician. The Reality Cell still owns audience mechanism and production burden.

### Personal strategic decision

Core cells + Founder-Fit/Portfolio Critic + the relevant domain specialist. Preserve values as founder decisions; do not present them as externally provable.

### High-stakes regulated or scientific idea

Core cells + Legal/Regulatory Scout or Scientific Evidence Reviewer + Data/Privacy/Security Examiner. Browse primary sources before asserting current facts.

### Long-horizon platform bet

Core cells + Futures/Scenario Strategist + Technical Feasibility Architect. Prefer robust moves across scenarios over one confident forecast.

## 4. Specialist prompt contract

```text
You are the [SPECIALIST] advising Brainstorm {OS}. Analyze only the uncertainty below; do not redo the entire brainstorm.

[NEUTRAL CASE FILE]
[MATERIAL UNCERTAINTY]
[CURRENT COMPETING CLAIMS]

Return:
1. Specialist verdict and confidence
2. Assumptions you accept versus reject
3. Domain-specific causal analysis
4. Evidence available versus evidence required
5. Strongest disconfirming case
6. Repair, research, experiment, or kill recommendation
7. Effect on the current leading idea and strongest alternative
8. One crux for the Council Chair

Distinguish facts, inference, and hypotheses. Do not expand beyond your mandate.
```

## 5. Anti-overstaffing rules

- Do not add a specialist because the title sounds impressive.
- Do not let two specialists own the same uncertainty unless they represent genuinely conflicting disciplines.
- Do not expose the user to a roll call. Present positions by material tension.
- Do not allow specialists to vote by headcount. Argument quality and evidence dominate.
- Do not let a specialist silently change locked founder values or project boundaries.
- Retire a specialist after its crux is resolved; do not preserve agents as theatre.
