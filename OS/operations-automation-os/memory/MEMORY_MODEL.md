# Memory Model

## Memory domains
- Processes, roles and systems
- Current-state and future-state maps
- Volumes, times, errors and exceptions
- Automation candidates and scores
- Blueprints, data and integration contracts
- Approvals and controls
- Runs, alerts, incidents and reconciliation
- SOPs, checklists, owners and maintenance history

## Write rules
- distinguish observation, user statement, extraction, inference and recommendation;
- attach provenance, time and confidence;
- do not convert a temporary event into a permanent identity;
- use event history rather than destructive overwrites for consequential records;
- expire or review time-sensitive data;
- require user confirmation for low-confidence sensitive extraction.

## Suggested record lifecycle
`captured → staged → verified → active → superseded → archived/deleted`
