# AI-Readiness Scorecard — {{Company}}

> Pre-engagement qualification gate. Scored at QUALIFICATION ALTITUDE from three sources only:
> company context · the qualification call ({{date}}) · a public-site scan. All detailed technical
> mapping is deferred to E2 (the engagement audit, caio-enterprise-workflow-architect).

| | |
|---|---|
| **Company** | {{Company}} |
| **Size / sector / country** | {{size}} · {{sector}} · {{country}} |
| **Sponsor (budget holder?)** | {{name + title}} · budget authority: {{yes/no}} |
| **Why now (trigger)** | {{the dated compelling event, or _(none stated)_}} |
| **Assessed by / date** | {{operator}} · {{date}} |
| **Weights** | default (1:18 2:14 3:12 4:12 5:11 6:10 7:8 8:8 9:7) {{or: OVERRIDE — reason}} |

---

## Dimension scores (0-4, each with cited evidence)

### D1 · Leadership & executive sponsorship — Level {{0-4}}  (weight 18% → contributes {{18×L/4}})
- (call) "{{verbatim quote}}" [source: qualification call, {{date}}]
- (scan) {{observation, or _(not provided)_}}
- **Read:** {{what this level means for the build}}
- **Gate G1 (≥2):** {{PASS / FAIL}}
- **Defer to E2:** {{none / detail not mapped here}}

### D2 · Tooling & API-exposure — Level {{0-4}}  (weight 14% → contributes {{14×L/4}})
- (call) "{{verbatim}}" [source: qualification call, {{date}}]
- (scan) {{tech hints from site, or _(not provided)_}}
- **Read:** {{which core tools appear to expose APIs / where the path is}}
- **Gate G2 (≥2):** {{PASS / FAIL}}
- **Defer to E2:** endpoint / auth / rate-limit verification — to be produced in the engagement (E2)

### D3 · Strategy & use-case clarity — Level {{0-4}}  (weight 12% → contributes {{12×L/4}})
- (call) "{{verbatim — the named pain}}" [source: qualification call, {{date}}]
- **Beachhead (JTBD):** "When {{trigger}}, help me {{progress}}, so I can {{benefit}}."
- **Read:** {{is there one painful, frequent, namable workflow?}}
- **Gate G3 (≥1):** {{PASS / FAIL}}
- **Defer to E2:** full opportunity backlog + scoring — to be produced in the engagement (E2)

### D4 · Data readiness — Level {{0-4}}  (weight 12% → contributes {{12×L/4}})
- (call) "{{verbatim}}" [source: qualification call, {{date}}]
- **Read:** {{can a pilot read the beachhead data today?}}
- **Gate:** none (risk dial — low score → Gap-To-Target item, not a blocker)
- **Defer to E2:** the data-and-permission map — to be produced in the engagement (E2)

### D5 · Governance, risk & compliance — Level {{0-4}}  (weight 11% → contributes {{11×L/4}})
- (call) "{{verbatim}}" [source: qualification call, {{date}}]
- **Read:** {{is the build permitted? is there a path? any hard blocker?}}
- **Gate G4 (≥1 AND no hard blocker):** {{PASS / FAIL}}
- **Defer to E2:** AI usage policy + HITL matrix + vendor-risk audit — to be produced in the engagement (E2)

### D6 · Culture & change-appetite — Level {{0-4}}  (weight 10% → contributes {{10×L/4}})
- (call) "{{verbatim — how the team feels}}" [source: qualification call, {{date}}]
- **4-forces read:** Push {{0-3}} · Pull {{0-3}} · Anxiety {{0-3}} · Habit {{0-3}}
  → Change-appetite = (Push+Pull) − (Anxiety+Habit) = {{value}} → {{net-positive / net-negative}}
- **Read:** {{will adoption happen?}}
- **Defer to E2:** per-team change-management plan — to be produced in the engagement (E2)

### D7 · Talent & AI-literacy — Level {{0-4}}  (weight 8% → contributes {{8×L/4}})
- (call) "{{verbatim — shadow IT / skills}}" [source: qualification call, {{date}}]
- **Read:** {{who could own this post-transfer? how long is the enablement tail?}}
- **Defer to E2:** skills inventory — to be produced in the engagement (E2)

### D8 · Infrastructure — Level {{0-4}}  (weight 8% → contributes {{8×L/4}})
- (call) "{{verbatim}}" [source: qualification call, {{date}}]
- **Read:** {{cloud / hybrid / on-prem — can an AI layer sit on top?}}
- **Defer to E2:** stack decision (cloud vs on-prem/private-LLM) — to be produced in the engagement (E2)

### D9 · Budget & commitment — Level {{0-4}}  (weight 7% → contributes {{7×L/4}})
- (call) "{{verbatim}}" [source: qualification call, {{date}}]
- **Read:** {{willing + able to start the monthly model?}}
- **Gate G5 (≥1):** {{PASS / FAIL}}

---

## Readiness Index

```
Index = Σ (weight × level/4)
      = {{D1}} + {{D2}} + {{D3}} + {{D4}} + {{D5}} + {{D6}} + {{D7}} + {{D8}} + {{D9}}
      = {{TOTAL}} / 100

Tier: {{Nascent (0-25) | Emerging (26-50) | Ready (51-75) | Leading (76-100)}}
```

## Hard-gate summary

| Gate | Floor | Score | Result |
|---|---|---|---|
| G1 Sponsorship | dim1 ≥ 2 | {{L}} | {{PASS/FAIL}} |
| G2 API-exposure | dim2 ≥ 2 | {{L}} | {{PASS/FAIL}} |
| G3 Use-case | dim3 ≥ 1 | {{L}} | {{PASS/FAIL}} |
| G4 Compliance | dim5 ≥ 1, no blocker | {{L}} | {{PASS/FAIL}} |
| G5 Commitment | dim9 ≥ 1 | {{L}} | {{PASS/FAIL}} |

**4-forces verdict:** {{net-positive / net-negative}} — {{one line}}

→ Verdict reasoning continues in `Go-No-Go-Brief.md`.

---
*Scored at qualification altitude. Every level traces to a cited source (R-CITE). Unverified levels marked `_(unverified — confirm on call)_` and scored conservatively. Detailed technical mapping deferred to E2.*
