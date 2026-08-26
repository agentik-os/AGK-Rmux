# R-APPROVE — Sensitive actions resume only after server-side approval

**Kind:** Rule
**Category:** Safety
**Added:** 2026-08-26

## Rule

Production deploy, client message, delete, financial action, dangerous shell, and system restart emit approval.required. Desktop/Discord may Approve/Reject. Button state is not authority — the backend validates, then resumes the run.

## Origin

A Discord click without a server check is not a control plane.
