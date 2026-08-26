# Server-Provisioning Runbook — {{company}} (5.1)

> One dedicated, **client-owned** centralized server. Data stays with the client. Readable + migratable. (Iron Law 2.)

- **Region:** {{region}} · **Residency:** {{GDPR/SOC2/HIPAA/none}}
- **Provisioned by:** {{caio_name}} · **Date:** {{YYYY-MM-DD}}
- **Realization spec:** APPROVED {{spec_version}} ({{approval_date}})  ← required before STEP 1

---

## Stack justification (defend to CTO + CFO + security)

| Layer | Choice | Justification (one line) |
|---|---|---|
| Frontend | Next.js (App Router) | {{seven dashboards, server-side secrets, client-readable}} |
| Backend / DB | Convex | {{reactive event bus = the federation; client-owned; one-command export}} |
| Auth / RBAC | Clerk | {{per-seat access + HITL roles map to permissions}} |
| Billing | Stripe | {{usage metering / internal chargeback — or OMITTED because {{reason}}}} |
| Integrations | Composio | {{one auth layer; 6-critical-connectors; managed refresh}} |
| Agent runtime | Claude Code SDK | {{tool-use agents with logged cost+confidence (delegated builds)}} |

**Adaptation applied:** {{existing ERP/CRM kept as SoR / warehouse read / air-gapped swap / none}}

---

## Provisioning steps (each ends VERIFIED)

- [ ] **STEP 0 — Pre-flight.** Realization spec approved · region+residency decided · accounts under {{client_org}} · secrets in {{client_secret_manager}}
- [ ] **STEP 1 — Repo + scaffold.** Repo: {{client_repo_url}} · route groups: {{(cio-cto)(cmo)(cfo)(coo)(chro)(cso) — only existing}}
- [ ] **STEP 2 — Convex.** Deployed in {{region}} · schema applied · `convex export` tested → {{export_file}}
- [ ] **STEP 3 — Clerk.** Roles: {{per-seat + HITL approver roles}} · VERIFY 403 on cross-seat admin
- [ ] **STEP 4 — Secrets/env.** No secret tracked (grep clean) · app boots from env-injected secrets
- [ ] **STEP 5 — Deploy shell.** `vercel --prod --token=$VERCEL_TOKEN` (or on-prem) · HTTP 200 each seat route · Clerk+Convex connect
- [ ] **STEP 6 — Observability skeleton (BEFORE features).** agentRuns + costEvents + monitoring view live · test run writes a row
- [ ] **STEP 7 — Provision acceptance.** Run the acceptance checklist below → record verdict

---

## Export path (the ownership proof — Iron Law 2)

- **Data export:** {{`npx convex export` scheduled to client bucket: details}}
- **Repo ownership:** {{client GitHub org; CAIO is collaborator not owner}}
- **Secrets handover:** {{list + location + rotation procedure}}
- **Redeploy runbook:** {{steps to redeploy elsewhere — proves migratability}}

---

## Provision acceptance (the server ship-gate)

| Check | Pass? | Evidence (R-CITE) |
|---|---|---|
| Convex deployed in correct region; schema typed; export tested | {{yes/no}} | {{export file}} |
| Clerk RBAC enforces per-seat access + HITL roles | {{yes/no}} | {{403 evidence}} |
| No secret tracked in git; boots from env only | {{yes/no}} | {{grep result}} |
| Authenticated shell loads; HTTP 200 every seat route; clean console | {{yes/no}} | {{log/screenshot}} |
| Observability skeleton writes agentRuns + costEvents | {{yes/no}} | {{test row}} |
| Accounts under the CLIENT's org | {{yes/no}} | {{account proof}} |
| Export path documented (data + repo + secrets + redeploy) | {{yes/no}} | {{this doc §export}} |
| Region + residency match the regulatory constraint | {{yes/no}} | {{region proof}} |

**Verdict:** {{PROVISIONED / BLOCKED}} · recorded in `08-Ship-Gate-Ledger.md` + `00-Build-Log.md`
