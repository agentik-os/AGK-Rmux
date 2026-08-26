# CAIO Implementation Runbook

> Phase 2 of the Chief-AI-Officer engagement: **BUILD**. Take the architect's `company-ai-os/` blueprint, **realize** it into the offer's centralized federated topology, then **build it operationally** — readable, auditable, transferable, live in production with value in week 1.

> The architect made the company legible. I make it run — in the open, never in a black box.

Built by [Agentik OS](https://agentik-os.com). Phase 2 of the CAIO accompaniment chain. Composes with [caio-enterprise-workflow-architect](https://skills.agentik-os.com/caio-enterprise-workflow-architect) (upstream), [caio-enablement-and-transfer](https://skills.agentik-os.com/caio-enablement-and-transfer) (downstream), [agentic-systems-builder](https://skills.agentik-os.com/agentic-systems-builder) + [agentik-skill-forge](https://skills.agentik-os.com/agentik-skill-forge) (delegated builders), [caio-run-and-optimize](https://skills.agentik-os.com/caio-run-and-optimize) (reads the baseline this skill lays).

---

## What it produces

A `./caio-build/` directory — the build of the centralized, federated Company-AI-OS:

1. **01-Architecture-Realization-Spec.md** — **the design gate.** Translates the architect's *generic* blueprint into the offer's *signature* topology: one dedicated server + one micro-SaaS per C-Level + the inter-dashboard API contract map + the Composio topology. **Approved by the sponsor before any build starts.**
2. **02-Server-Provisioning-Runbook.md** — the single dedicated, client-owned centralized server (5.1): readable stack, migratable, data stays with the client.
3. **03-MicroSaaS-Build-Plan.md** — a per-C-Level build checklist (CIO/CTO, CMO, CFO, CDO, COO, CHRO, CSO — only the seats that exist), each built for that person's real job, not a recycled template.
4. **04-Inter-Dashboard-API-Contract.md** — the differentiator (5.3): each dashboard exposes + consumes APIs so a COO metric can trigger a CFO alert. The system runs as one interconnected organism.
5. **05-Integration-Wiring-Guide.md** — Composio wiring (5.4): the 6 critical connectors that actually work (auth, rate limits, live-read proofs), not a 200-connector list.
6. **06-Automated-Report-Specs.md** — the auto-falling-out report (5.5): exact frequency, format, indicators, recipients. Analysts analyze instead of copy-pasting.
7. **07-Monitoring-And-Instrumentation.md** — observability (5.8) + the North-Star/cost/usage baseline (mm-11) wired at build time so ROI is measurable later.
8. **08-Ship-Gate-Ledger.md** — per-deliverable acceptance, criteria pulled from the architect's feature specs. Value in week 1, not a POC.
9. **09-Sponsor-Communication-Plan.md** — milestone demos, approval gates, progress briefs, go-live announcement (mm-04) to keep executive sponsorship alive through the longest phase.
10. **builds/** — per-micro-SaaS build dossiers. **metadata.json** — machine-readable handoff header (seats, t0, gate verdicts).

## When to use it

After `caio-enterprise-workflow-architect` has produced `company-ai-os/` (at least through `07-Dashboard-Feature-Specs.md`) and the SOW is signed. Use it to:

- **realize-architecture** — produce + gate the Architecture-Realization spec (design stage only).
- **provision-server** — stand up the dedicated centralized server.
- **build-microsaas** — build one or more C-Level dashboards to ship-gate.
- **wire-federation** — implement the inter-dashboard API contract.
- **wire-integrations** — Composio: the 6 critical connectors.
- **full-build** — realize → provision → build all → wire → integrate → instrument → ship.

Do **not** use it for the upstream audit/blueprint (that's the architect), for team training/transfer (that's `caio-enablement-and-transfer`), or for measuring post-go-live ROI (that's `caio-run-and-optimize`).

## Chain position

```
caio-ai-readiness-assessment   (pre-sign go/no-go)
        -> /market-proposal     (signed SOW)
caio-discovery-interview        (Phase 1 immersion: per-person dossiers + rollup)
caio-enterprise-workflow-architect (Phase 1 architecture: company-ai-os/ blueprint + backlog + ROI)
==> caio-implementation-runbook  (Phase 2: REALIZE the federated topology, then BUILD)  <== YOU ARE HERE
caio-enablement-and-transfer    (Phase 3 adoption + Phase 4 transfer-to-autonomy)
caio-run-and-optimize           (Phase 5: measure ROI vs baseline, optimize, expand) -> loops to architect
```

## Composes with / delegates to

| Relationship | Skill | Contract |
|---|---|---|
| Reads | `caio-enterprise-workflow-architect` | `company-ai-os/` (backlog, blueprints, feature specs + acceptance, roadmap, ROI projection) |
| Reads (optional) | `caio-discovery-interview` | `company-rollup.md` — which C-Level seats actually exist, system-of-record per data type |
| Hands to | `caio-enablement-and-transfer` | the **live system + its internal docs** (`caio-build/`) |
| Seeds | `caio-run-and-optimize` | the instrumentation baseline (t0) + `metadata.json` for ROI-vs-projection |
| Delegates | `agentic-systems-builder` | per-agent implementation, one dispatch per `F-XXX` feature spec |
| Delegates | `agentik-skill-forge` | codifying company-specific repeatable skills (e.g. "monthly-close") |
| Delegates (optional) | `creator-media-engine` | public case-studies from the engagement (with consent) |
| Ship-gate | `/omg-acceptance` | the browser/console/golden-path acceptance gate per micro-SaaS |

## The stack (justified, not cargo-culted)

Next.js (seven dashboards, server-side secrets, client-readable) · Convex (the shared event bus for the federation + client-owned deployment) · Clerk (per-C-Level RBAC + HITL roles) · Stripe (usage metering / internal chargeback) · Composio (one auth+action layer, the 6-critical-connectors rule) · Claude Code SDK (the agent runtime per `F-XXX`). Each justified to the client in the realization spec; adapted for data-residency / SOC2 / air-gapped constraints.

## Doctrine grounding

- **mm-11 (measure-loops-retention)** — the *instrument-for-baseline* slice: wire the North-Star metric + cost/usage events per dashboard **at build time** (three clean events, no vanity metrics) so `caio-run-and-optimize` can later measure actual ROI against the architect's projection. The built-in tracking (5.8) is that baseline substrate, captured at t0.
- **mm-04 (messaging/copy/offer)** — the *build-milestone communication* slice: frame milestone demos, the realization approval gate, progress briefs, and the go-live announcement (value-equation applied to the sponsor; clear-beats-clever; honest-runtime-only demos) to keep executive sponsorship + budget confidence alive through the longest, most vulnerable phase.

## References

- `references/01-architecture-realization.md` — the design gate: generic blueprint → centralized federated topology.
- `references/02-server-and-stack-provisioning.md` — the dedicated server (5.1) + stack justification + provisioning runbook.
- `references/03-microsaas-and-inter-dashboard-api.md` — per-C-Level build checklist + the inter-dashboard API contract pattern (5.3).
- `references/04-composio-integration-and-reports.md` — Composio wiring (5.4) + automated reports (5.5).
- `references/05-instrumentation-shipgate-and-sponsor-comms.md` — monitoring/instrumentation (5.8 + mm-11) + ship-gate + sponsor communication (mm-04).

## Templates (the client deliverables)

`assets/templates/`: Architecture-Realization-Spec.md · Server-Provisioning-Runbook.md · MicroSaaS-Build-Checklist.md · Inter-Dashboard-API-Contract.md · Integration-Wiring-Guide.md · Automated-Report-Spec.md · Monitoring-Setup.md · Ship-Gate-Checklist.md · Sponsor-Communication-Plan.md · metadata.json.

## License

MIT — Agentik OS.
