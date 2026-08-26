# Multimodal Ingestion Pipeline

1. Authenticate user/entity and purpose.
2. Virus/malware and prompt-injection screen.
3. Hash/deduplicate original file.
4. Classify: receipt, supplier invoice, customer invoice, contract, statement, screenshot, identity/tax or unknown.
5. Extract fields with field-level confidence and page/region provenance.
6. Resolve entity, currency, date and duplicate candidates.
7. Validate arithmetic and cross-document consistency.
8. Propose mutations in a **staging queue**.
9. Human or policy approval.
10. Transactional write and immutable audit event.
11. Reconcile against bank/payment/accounting/CRM source.
12. Keep original document according to retention policy.

## Fields that always require caution
Currency, decimal separators, tax, invoice number, due date, legal entity, bank details, credit/refund signs and handwritten changes.
