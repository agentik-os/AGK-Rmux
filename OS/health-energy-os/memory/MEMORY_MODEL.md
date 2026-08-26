# Memory Model

## Memory domains
- Health goals and constraints
- Daily readiness observations
- Sleep/training/nutrition trends
- Experiments and outcomes
- Professional-care questions and handoffs
- User-approved wearable summaries

## Write rules
- distinguish observation, user statement, extraction, inference and recommendation;
- attach provenance, time and confidence;
- do not convert a temporary event into a permanent identity;
- use event history rather than destructive overwrites for consequential records;
- expire or review time-sensitive data;
- require user confirmation for low-confidence sensitive extraction.

## Suggested record lifecycle
`captured → staged → verified → active → superseded → archived/deleted`
