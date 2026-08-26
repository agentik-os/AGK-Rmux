# Security, Privacy and Trust

## Baseline controls
- least-privilege access;
- encryption in transit and at rest in production adapters;
- immutable audit events for consequential writes;
- file hashing and deduplication for ingestion;
- staged extraction with confidence;
- user-visible correction and deletion;
- explicit export boundaries;
- secret and credential redaction;
- retention policies by data class;
- no training on private records without explicit agreement.

## Domain-specific controls
- Use customer-specific access partitions.
- Store secrets in a secrets manager, not project notes.
- Revoke access and verify retention/deletion at offboarding.
- Require consent before testimonial/referral/case-study use.

## Threats to test
- prompt injection from uploaded files;
- fabricated source/provenance;
- cross-user data leakage;
- unauthorized automated action;
- silent schema drift;
- stale recommendations based on outdated records;
- over-collection of sensitive data.
