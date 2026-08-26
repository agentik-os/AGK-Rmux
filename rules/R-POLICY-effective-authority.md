# R-POLICY — Effective authority is Profile + Client + OS + Agent

**Kind:** Rule
**Category:** Safety
**Added:** 2026-08-26

## Rule

Mission may use the current client and must not read Private or run Operator privileged actions. Operator privileged work goes through the Admin Action Registry — never arbitrary root. Isolation (logical vs linux-user vs container) informs strictness; it is not a domain object.

## Origin

Without a policy engine, Mission YOLO could see Private and Operator could mean raw root.
