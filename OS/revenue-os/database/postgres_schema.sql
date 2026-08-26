-- Revenue OS reference PostgreSQL schema (adapt before production)
create extension if not exists pgcrypto;
create schema if not exists revenue;

create table if not exists revenue.business_entity (
  id uuid primary key default gen_random_uuid(), legal_name text not null,
  jurisdiction text not null, base_currency text not null,
  created_at timestamptz not null default now(), updated_at timestamptz not null default now()
);
create table if not exists revenue.customer (
  id uuid primary key default gen_random_uuid(), entity_id uuid not null references revenue.business_entity(id),
  name text not null, status text not null check(status in ('prospect','customer','churned','archived')),
  owner_id text not null, billing_profile jsonb not null default '{}',
  created_at timestamptz not null default now(), updated_at timestamptz not null default now()
);
create table if not exists revenue.contact (
  id uuid primary key default gen_random_uuid(), customer_id uuid not null references revenue.customer(id),
  name text not null, email text, role text, consent_status text not null default 'unknown',
  created_at timestamptz not null default now(), updated_at timestamptz not null default now()
);
create table if not exists revenue.offer (
  id uuid primary key default gen_random_uuid(), entity_id uuid not null references revenue.business_entity(id),
  name text not null, version text not null, best_fit_customer text not null,
  outcome text not null, scope jsonb not null, exclusions jsonb not null,
  pricing jsonb not null, status text not null default 'draft',
  created_at timestamptz not null default now(), updated_at timestamptz not null default now(),
  unique(entity_id,name,version)
);
create table if not exists revenue.opportunity (
  id uuid primary key default gen_random_uuid(), customer_id uuid not null references revenue.customer(id),
  offer_id uuid references revenue.offer(id), stage text not null, amount numeric(18,2), currency text,
  stage_evidence jsonb not null default '[]', next_step text not null, next_step_at timestamptz,
  owner_id text not null, probability numeric(5,4), close_date date,
  created_at timestamptz not null default now(), updated_at timestamptz not null default now()
);
create table if not exists revenue.activity (
  id uuid primary key default gen_random_uuid(), customer_id uuid not null references revenue.customer(id),
  opportunity_id uuid references revenue.opportunity(id), type text not null, occurred_at timestamptz not null,
  facts jsonb not null default '[]', commitments jsonb not null default '[]', source_id uuid,
  created_at timestamptz not null default now()
);
create table if not exists revenue.document (
  id uuid primary key default gen_random_uuid(), entity_id uuid not null references revenue.business_entity(id),
  file_hash text not null, document_type text not null, object_uri text not null,
  extracted_fields jsonb not null default '{}', confidence jsonb not null default '{}',
  verification_status text not null check(verification_status in ('staged','verified','rejected')),
  created_at timestamptz not null default now(), unique(entity_id,file_hash)
);
create table if not exists revenue.contract (
  id uuid primary key default gen_random_uuid(), customer_id uuid not null references revenue.customer(id),
  opportunity_id uuid references revenue.opportunity(id), source_document_id uuid not null references revenue.document(id),
  effective_date date not null, end_date date, terms jsonb not null,
  approval_status text not null default 'staged', created_at timestamptz not null default now()
);
create table if not exists revenue.invoice (
  id uuid primary key default gen_random_uuid(), customer_id uuid not null references revenue.customer(id),
  contract_id uuid references revenue.contract(id), invoice_number text not null,
  issue_date date not null, due_date date not null, amount numeric(18,2) not null, currency text not null,
  status text not null check(status in ('draft','approved','sent','part_paid','paid','overdue','disputed','void')),
  source_document_id uuid references revenue.document(id), created_at timestamptz not null default now(),
  unique(customer_id,invoice_number)
);
create table if not exists revenue.payment (
  id uuid primary key default gen_random_uuid(), invoice_id uuid references revenue.invoice(id),
  received_at timestamptz not null, amount numeric(18,2) not null, currency text not null,
  external_reference text, reconciliation_status text not null default 'staged',
  source_document_id uuid references revenue.document(id), created_at timestamptz not null default now()
);
create table if not exists revenue.expense (
  id uuid primary key default gen_random_uuid(), entity_id uuid not null references revenue.business_entity(id),
  vendor text not null, expense_date date not null, amount numeric(18,2) not null, currency text not null,
  tax_data jsonb not null default '{}', classification text,
  verification_status text not null default 'staged', source_document_id uuid references revenue.document(id),
  created_at timestamptz not null default now()
);
create table if not exists revenue.forecast_snapshot (
  id uuid primary key default gen_random_uuid(), entity_id uuid not null references revenue.business_entity(id),
  period_start date not null, period_end date not null, scenario text not null,
  assumptions jsonb not null, outputs jsonb not null, created_at timestamptz not null default now()
);
create table if not exists revenue.approval (
  id uuid primary key default gen_random_uuid(), action_type text not null, resource_type text not null,
  resource_id uuid not null, requested_by text not null, approver_id text,
  status text not null check(status in ('pending','approved','rejected','expired')),
  rationale text, created_at timestamptz not null default now(), decided_at timestamptz
);
create table if not exists revenue.audit_event (
  id uuid primary key default gen_random_uuid(), event_type text not null, actor_id text not null,
  resource_type text, resource_id uuid, payload jsonb not null default '{}',
  correlation_id uuid not null default gen_random_uuid(), created_at timestamptz not null default now()
);

create index if not exists idx_opportunity_stage on revenue.opportunity(stage,close_date);
create index if not exists idx_invoice_status_due on revenue.invoice(status,due_date);
create index if not exists idx_document_hash on revenue.document(file_hash);
create index if not exists idx_audit_resource on revenue.audit_event(resource_type,resource_id,created_at);
