# Memory Model

## Memory domains
- Requirements and traceability
- Test/eval plans and results
- Defects and dispositions
- Security/privacy/accessibility findings
- Release candidates and approvals
- Deployments, rollbacks and incidents
- Baselines and regression history

## Write rules
- distinguish observation, user statement, extraction, inference and recommendation;
- attach provenance, time and confidence;
- do not convert a temporary event into a permanent identity;
- use event history rather than destructive overwrites for consequential records;
- expire or review time-sensitive data;
- require user confirmation for low-confidence sensitive extraction.

## Suggested record lifecycle
`captured → staged → verified → active → superseded → archived/deleted`
