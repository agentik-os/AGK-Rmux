# Ownership Handover Checklist — {{company}}

> Transfer is not complete while only the CAIO can fix something. No bus factor of one — including you. A system where the consultant still holds the keys is, by definition, not transferred.

- **Handover date:** {{date}} · **CAIO:** {{name}} · **Client lead:** {{name}}

## Named ownership (every component has a person)
| Component | Type | Owner | Backup (critical → required) |
|---|---|---|---|
| {{name}} | {{agent/dashboard/integration}} | {{owner}} | {{backup}} |

- [ ] Every component has a named owner.
- [ ] Every CRITICAL component has a backup owner (bus factor ≥ 2).

## Credentials & access (the most-skipped, most-important step)
- [ ] Every key/token/secret rotated to the client's vault ({{1Password/Vault/Convex env}}).
- [ ] **ZERO CAIO-only credentials remain** — list any that did, with rotation date: {{none / ...}}
- [ ] CAIO access reduced to advisory (or revoked): {{state}}.
- [ ] Client can independently rotate a secret without the CAIO: {{verified yes}}.

## Human-in-the-loop ownership (Iron Law 7)
- [ ] For every sensitive decision (financial / legal / customer-facing / regulated / headcount), the named approver is a CLIENT employee, not the CAIO.
- [ ] HITL gates verified still active after handover (not silently removed in the name of "autonomy").

| Sensitive decision | Agent/feature | Client approver (named) |
|---|---|---|
| {{...}} | {{...}} | {{name}} |

## Process & cadence
- [ ] The evolution process documented AND run at least once by the team.
- [ ] The weekly guardian routine run ≥ twice WITHOUT the CAIO present. Dates: {{...}}
- [ ] Escalation path documented: in-house vs. agentic-systems-builder vs. agentik-skill-forge.

## Documentation
- [ ] Internal Documentation Pack at ≥ 9/10 legibility, no layer at 0.
- [ ] No black-box / tribal-knowledge component remains.
- [ ] Docs verified against runtime (no doc that disagrees with the live system — L1).

## Handoff to the run phase
- [ ] `08-Adoption-Tracker.md` handed to the team + to caio-run-and-optimize (usage baseline).
- [ ] `04-Validated-Use-Cases-Log.md` handed forward (seeds the ROI re-measure).
- [ ] `07-Autonomy-Readiness-Gate.md` result attached.

## Sign-off
- Client lead confirms ownership: {{name / date}}
- CAIO confirms zero remaining single points of failure (incl. self): {{name / date}}
- **Outstanding items (with owner + date):** {{none / list}}
