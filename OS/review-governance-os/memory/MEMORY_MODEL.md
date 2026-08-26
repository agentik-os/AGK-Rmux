# Memory Model

## Memory domains
- Review records and outcomes
- Metrics and definitions
- Incidents and near misses
- Risks and controls
- Policies and exceptions
- Change requests and approvals
- Decisions, dissent and verification

## Write rules
- distinguish observation, user statement, extraction, inference and recommendation;
- attach provenance, time and confidence;
- do not convert a temporary event into a permanent identity;
- use event history rather than destructive overwrites for consequential records;
- expire or review time-sensitive data;
- require user confirmation for low-confidence sensitive extraction.

## Suggested record lifecycle
`captured → staged → verified → active → superseded → archived/deleted`
