# Architecture

```text
USER / FILES / EVENTS
        │
        ▼
INTENT + SAFETY ROUTER
        │
        ▼
AUTHORIZED CONTEXT COMPILER
        │
        ├── MEMORY / RECORDS
        ├── KNOWLEDGE / STANDARDS
        └── CURRENT EVIDENCE
        │
        ▼
SPECIALIST COUNCIL
        │
        ▼
REVIEW & GOVERNANCE OS INTEGRATOR
        │
        ▼
DECISION / PLAN / ARTIFACT / ACTION
        │
        ▼
EVIDENCE + REVIEW + HANDOFF
```

## Components
- **Router:** intent, mode and safety classification
- **Context compiler:** minimum necessary context with provenance
- **Specialists:** independent lenses
- **Integrator:** coherent output and tradeoff resolution
- **Record layer:** schemas in `schemas/`
- **Event log:** append-only reference log
- **Evaluation layer:** acceptance and adversarial tests
