# Revenue Database Architecture

## Logical stores
1. **Operational relational store:** entities, customers, contacts, offers, opportunities, contracts, invoices, payments, expenses, forecasts and approvals.
2. **Immutable event/audit store:** who proposed, approved, changed, sent or reconciled each consequential record.
3. **Document object store:** original photos/PDFs/contracts/statements addressed by hash.
4. **Search/vector index:** optional derived index for conversation retrieval; never canonical by itself.
5. **Analytics layer:** period snapshots and reconciled metrics, separated from operational writes.

## Conversational read path
Question → permission/intent → entity resolution → structured query → source/provenance retrieval → answer with confidence and exceptions.

## Write path
Conversation/file → proposed mutation → validation → approval policy → transactional write → audit event → downstream integration → reconciliation.

## Never do
- Let the LLM issue arbitrary SQL.
- Treat vector retrieval as accounting truth.
- auto-post low-confidence extraction;
- combine business and personal ledgers;
- overwrite audit history.
