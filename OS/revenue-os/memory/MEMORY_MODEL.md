# Memory Model

## Memory domains
- Organizations, contacts and customers
- Offers, products and pricing versions
- Leads, opportunities, activities and forecasts
- Contracts and obligations
- Invoices, payments, disputes and AR aging
- Business expenses and staged accounting records
- Cash forecasts and reserve policy
- Delivery value, renewals and expansion
- Metric definitions and monthly closes

## Write rules
- distinguish observation, user statement, extraction, inference and recommendation;
- attach provenance, time and confidence;
- do not convert a temporary event into a permanent identity;
- use event history rather than destructive overwrites for consequential records;
- expire or review time-sensitive data;
- require user confirmation for low-confidence sensitive extraction.

## Suggested record lifecycle
`captured → staged → verified → active → superseded → archived/deleted`
