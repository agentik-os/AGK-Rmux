# Protocol: Self-Presentation Audit

A baseline audit of the controllable surface: the part of how the user comes across that is decided before they speak. It is deliberately narrow. It covers what is changeable and cheap, it labels taste as taste, and it hands every taste decision back to the user rather than issuing a verdict.

The honest framing, stated first: self-presentation is real and it is smaller than the genre claims. It is a threshold effect far more than a ranking one. Getting from neglected to deliberate produces a large change; getting from deliberate to fashionable produces very little. The audit works on the first gap and declines to chase the second.

Almost everything below is **[P]**. There is no universal correct answer on clothing, hair, grooming or body, cultures and subcultures disagree completely, and an OS that invents a single standard here is doing the thing this pack exists to avoid.

## Steps
1. Ask what the user is actually going for, in their own words, before assessing anything. A look that succeeds in one context fails in another, and "attractive" is not a specification.
2. Separate the three layers, because they have completely different costs and completely different evidence. FIT AND CONDITION (does it fit, is it clean, is it maintained) is the layer that carries almost all of the effect and it is nearly free. COHERENCE (does the whole read as one deliberate thing) is the second. TASTE (the specific aesthetic) is the third and it is entirely **[P]**.
3. Audit layer one against the checklist below. This layer gets direct answers, because "these trousers are two sizes too big" is an observation rather than an opinion.
4. Audit layer two for coherence, not for correctness. A coherent look the auditor personally dislikes passes. Incoherence is the finding: three styles arguing with each other, or an outfit assembled from unrelated decisions made years apart.
5. Do not audit layer three. Describe options, name what each one signals to whom, and hand the choice back. Every layer-three line in the output carries a **[P]** label and the user's name on the decision.
6. Cover the non-clothing surface, which is usually where the real finding is: grooming and maintenance, posture and how the user occupies space, voice (pace, volume, whether sentences end downward), rest and the physical substrate under all of it.
7. Route the substrate out. Sleep, training, energy and health belong to Health and Energy OS. This protocol does not write a training plan and does not comment on body size. If the user raises weight, answer once, factually, without a program, and route.
8. Find the ONE change with the highest ratio of effect to cost, and name it. It is almost always in layer one, and it is very often a single garment that fits properly or one maintenance habit.
9. Cap the output at three changes total, ordered by that ratio, with a cost in money and time attached to each. An audit returning fifteen items is a shopping list, and shopping lists do not get done.
10. Write the second-order effect down explicitly where it applies. A change that makes the user stand differently is worth more than one that only looks better in a mirror, because it acts on presence rather than on appearance.
11. Log a `self_presentation_audit` record with channel `in_person`, every taste item marked **[P]** and owned by the user. The schema enforces this: a finding labelled P must record the user as the decider, and a taste-layer finding must be labelled P.
12. Hand ONE change to the week in weekly_practice.md. One, running for the whole week.

## Layer one, fit and condition
| Item | The question | Why it is layer one |
| --- | --- | --- |
| Fit at the shoulders and the waist | Does it sit where the body sits | The single most visible difference between deliberate and neglected, and a tailor is cheaper than a wardrobe |
| Length: sleeves, hems, rise | Does it end where it should end | Free to fix, immediately visible |
| Condition | Clean, unworn at the collar and cuffs, unpilled, pressed, shoes maintained | Reads as self-respect rather than as money, which is the actual signal |
| Grooming baseline | Hair maintained on a schedule, nails, skin, breath, a scent that is used sparingly or not at all | The lowest-cost, highest-return line in the entire audit |
| Glasses and everyday objects | Do they fit the face and the life | Worn every day, chosen once, usually years ago |

## Layer three, taste, handed back
Describe rather than decide. For each option: what it signals, to whom, in which contexts, and what it costs. Then stop. The user picks.

The failure mode to name if it appears: dressing for an imagined audience of strangers rather than for the rooms the user is actually in. Peacocking (deliberately strange clothing to provoke comment) buys attention from strangers and costs credibility with everyone, which is a bad trade **[P]**.

## Stop rules
- Never issue a verdict on a body. Comment on fit, condition, posture and maintenance. Do not rank features, do not suggest procedures, do not comment on weight or size beyond routing the substrate to Health and Energy OS.
- If the user reports distress about their appearance that is out of proportion to what is described, repeated checking, mirror avoidance, camera avoidance, or a fixation on one specific feature, stop the audit. Label **C**, name it plainly, route to a qualified professional. Body dysmorphia is not addressed by a better haircut, and an audit run over it makes it worse by supplying material.
- No fake status props: rented cars, staged photos, inflated titles, a persona built for an audience. It is a promise that comes due, and it guarantees the private conclusion that they only like the fake version.
- Do not launder taste as science. There is no evidence base for a specific colour, cut or style being universally attractive, and claiming one is dishonest.
- Do not confuse a self-presentation problem with a self-worth problem. "I need to look better" is sometimes literally true **[P]** and is often self-worth wearing a mirror. When it is the second, the audit will not fix it and running it repeatedly becomes the avoidance. Route to Mindset OS.
- Cap at three changes. Hand one to the week.

## Required closure
- Decision or output: three changes maximum, ordered by effect over cost, each with a cost attached and each labelled by layer.
- Owner: the user owns every taste decision. The OS owns the fit and condition observations and says which is which.
- Observable completion evidence: a `self_presentation_audit` record (see ../schemas/self_presentation_audit.json) where every **[P]** item records the user as the decider, plus one change installed in the current week.
- Review trigger: once a season, or when the user changes context, climate or role. Not weekly. A weekly appearance audit is a mechanism for producing dissatisfaction.
- Memory and handoff instruction: persist the user's own attributes and decisions only, with consent. Route the physical substrate to Health and Energy OS, appearance distress to a qualified professional under **C**, photos and profile surfaces to digital_profile_audit.md, and self-worth to Mindset OS.
