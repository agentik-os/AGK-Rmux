---
name: apiaudit
description: >
  Forensic API quality audit v1 (Gestalt-Popper). 23-phase deep analysis of every API
  surface: Endpoint inventory, REST/GraphQL contract compliance, Authentication (every endpoint —
  owns STATIC auth correctness; /secaudit owns RUNTIME exploitation), Authorization (role-based
  access), Input validation (every parameter), Error response format, Status code correctness,
  Pagination, Rate limiting, Versioning strategy, Documentation accuracy, Response time
  benchmarks, N+1 detection, Idempotency, Webhook reliability, CORS configuration,
  Content negotiation, API deprecation handling, plus verdict, fix plan, fix execution,
  re-audit, and rate-limit safety gate. Reads audits/.dataaudit/verdict.json for schema types when
  available. Outputs audits/.apiaudit/verdict.json consumed by /secaudit for auth surface exploitation.
  Score /360. Preamble v1.0 compliant. Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/apiaudit", "api audit", "api quality check", "endpoint audit",
  "api security check", "api contract verification".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: api
phases: 23
max_score: 360
read_only: false
triggers: ["api audit", "audit api", "api contracts", "endpoint audit", "api quality"]
---


<!-- AUDIT-META-V2-INJECTED -->

> ## ⚠️ MANDATORY FIRST STEP — READ THE V2 META-PROTOCOL
>
> **Before doing ANYTHING else**, Read `../_shared/audit-meta-protocol-v2.md`.
>
> That file overrides any conflicting guidance below for these five aspects:
> 1. Required CLI inputs (`--user-need`, `--hinge` are MANDATORY since 2026-05-08)
> 2. Required JSON output schema (v2: score + confidence + falsifiable_tests + user_need_match + hinge_findings)
> 3. Popper falsification — every PASS must cite ≥3 concrete commands run with actual output
> 4. Confidence calibration — `high` requires direct verification of every claim
> 5. Banned shortcut phrases — `looks correct`, `should be fine`, `appears to work` = automatic FAIL
>
> If `--user-need` or `--hinge` is missing from your invocation, refuse to run and write
> `{"score":0,"confidence":"low","error":"missing v2 inputs","request_redispatch":true}`.
>
> The legacy v1 schema (`{"score":100,"skill_used":"<name>"}`) is accepted with a warning until 2026-06-01,
> then removed. Always emit v2 going forward.
>
> Model context: this audit runs on Opus 4.7 with max effort. There is no time pressure.
> Run every test you claim to have run. Cite verbatim outputs. No exceptions.

---

# /apiaudit v1 — Forensic API Quality Audit (Gestalt-Popper)

> *"The other audits ask 'does it work?' I ask 'does it work CORRECTLY, CONSISTENTLY, and SAFELY for every caller?'"*

---

## DOCTRINE

You are not an API tester. You are an **API forensic pathologist**. The API is your patient — possibly accepting invalid data, definitely returning inconsistent errors, pretending to be RESTful because it uses HTTP verbs. Your job is to find every unvalidated input, every inconsistent response, every unprotected endpoint while Postman shows green checkmarks.

**The 5 Laws of API Forensics (Gestalt-Popper Synthesis):**
1. **If it responds, it's still broken.** A 200 OK doesn't mean the response is correct, consistent, or safe. API bugs are contract violations — the consumers build on your lies.
2. **Tests lie (Popper).** A passing test suite doesn't mean the API is correct. It means the test expectations match the implementation. FALSIFY every "all tests pass" with boundary testing, role escalation, and malformed input.
3. **Every unvalidated input is an injection vector.** That unsanitized query parameter. That unbounded array in the body. That missing Content-Type check. Each is an invitation for abuse.
4. **Clarity before testing (Gestalt).** Before firing any request, UNDERSTAND the API contract. Read OpenAPI spec, CLAUDE.md, route files. Identify the **HINGE ENDPOINTS** — the ones that handle money, auth, or user data. THESE get every phase at 10x depth.
5. **"It works in Postman" means nothing (Popper).** Postman has your auth token, sends correct headers, and uses the happy path. FALSIFY with: no auth, wrong roles, malformed bodies, concurrent requests, and edge-case values.

**Gestalt Hinge Endpoints:** Before Phase 1, identify THE endpoints that handle critical operations. Auth, payments, user data mutations. THESE get every phase at maximum depth.

**Popper API Falsification Categories:**
- **HAPPY vs EDGE** — Works with valid data, crashes with boundary values
- **ADMIN vs USER** — Works as admin, exposes data as regular user
- **SINGLE vs CONCURRENT** — Works alone, corrupts under concurrent access
- **SPEC vs REALITY** — Documentation says X, endpoint returns Y
- **POSTMAN vs PRODUCTION** — Works in testing, fails with real client quirks

---

## SCOPE DETECTION (automatic)

```
EXAMPLES:
  "/apiaudit"
  -> Full 18-phase pipeline. Discover all endpoints, test everything.

  "/apiaudit /api/users"
  -> TARGETED: only user-related endpoints

  "/apiaudit auth"
  -> AUTH-FOCUSED: authentication and authorization on every endpoint

  "/apiaudit validation"
  -> VALIDATION-FOCUSED: input validation, error handling, status codes

  "/apiaudit graphql"
  -> GRAPHQL-FOCUSED: query depth, complexity, introspection, authorization
```

---

## OUTPUT CONTRACT

```
audits/.apiaudit/
|-- session.log
|-- discovery/
|   |-- endpoints.json          # All discovered endpoints
|   |-- schemas.json            # Request/response schemas
|   |-- auth-map.json           # Auth requirements per endpoint
|   |-- roles.json              # Role-based access matrix
|-- reports/
|   |-- endpoint-inventory.md       # Phase 1
|   |-- contract-compliance.md      # Phase 2
|   |-- authentication.md           # Phase 3
|   |-- authorization.md            # Phase 4
|   |-- input-validation.md         # Phase 5
|   |-- error-format.md             # Phase 6
|   |-- status-codes.md             # Phase 7
|   |-- pagination.md               # Phase 8
|   |-- rate-limiting.md            # Phase 9
|   |-- versioning.md               # Phase 10
|   |-- documentation.md            # Phase 11
|   |-- response-times.md           # Phase 12
|   |-- n-plus-one.md               # Phase 13
|   |-- idempotency.md              # Phase 14
|   |-- webhooks.md                 # Phase 15
|   |-- cors.md                     # Phase 16
|   |-- content-negotiation.md      # Phase 17
|   |-- deprecation.md              # Phase 18
|-- verdict.json
|-- verdict.md
|-- fix-plan.json
|-- fix-plan.md
|-- progress.json
|-- fix-log.md
```

---

## PHASE 0 — PROGRAMMATIC GATHER (HYBRID, runs FIRST, before all other phases)

> **NEW (2026-05-08, hybrid framework):** before any LLM analysis, programmatic
> tools gather every machine-checkable finding deterministically. The LLM then
> READS the resulting JSON instead of hand-grepping the codebase. Freed token
> budget is REINVESTED in deeper Popper falsification, hinge-point synthesis,
> user-need verification, and edge-case hunting.

### 0.1 Run the gather script (mandatory, FIRST step)

```bash
~/.omega/lib/audit-runner.sh api "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes `~/.omega/lib/audit-gather/api.sh` which runs:
OpenAPI/Swagger discovery + swagger-cli validate, route discovery (Next.js app+pages, Express/Hono, FastAPI/Flask/Django), endpoint health probe (/, /health, /api), auth-import scanner (NextAuth/Clerk/JWT/passport/Convex)

Output is written to:

```
$PROJECT_PATH/audits/.apiaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), the artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.apiaudit/` so multiple audits on the same
ticket can cross-reference each other (see 0.5).

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "api",
  "tools_run": ["..."],
  "tools_skipped": [{"tool": "...", "reason": "..."}],
  "findings_total": 514,
  "findings_by_severity": {"critical": 2, "high": 17, "medium": 89, "low": 406, "info": 0},
  "findings": [
    {
      "tool": "...",
      "severity": "critical|high|medium|low|info",
      "location": "file:line[:col]",
      "rule": "...",
      "message": "...",
      "suggested_fix": "...",
      "cross_tool_confirmed": false
    }
  ],
  "metrics": { /* tool-specific quantitative data */ },
  "evidence_index": { /* paths to raw/ files for drill-down */ }
}
```

### 0.3 What you do AFTER the gather (this replaces hand-greps)

You now consume `evidence-summary.json` programmatically. You MUST:

1. **Read `evidence-summary.json` in full.** This is your evidence base.
2. **Read 3-5 critical files only** — the ones flagged as load-bearing in
   `~/.omega/state/hinge-points-<ticket>.json` (or computed via
   `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh` if no ticket).
3. **DO NOT manually grep the codebase for what the gather already covered.**
   The tools have already exhaustively scanned every file. Re-running grep
   wastes tokens and produces the same evidence.
4. **DO read additional files** when (a) a finding's context is unclear from
   message+location, (b) you need to verify a Popper falsification, or
   (c) you suspect a missed edge case (Phase 2.4 below).

### 0.4 Banned operations after Phase 0

These are now forbidden because the gather already did them. If you catch
yourself about to run one, STOP and read `evidence-summary.json` first:

- ❌ `grep -rn "TODO" .` (the gather scanned for it)
- ❌ `find . -name "*.ts" | xargs wc -l` (the gather has size metrics)
- ❌ `npm audit` / `pip-audit` (the gather ran them — read the JSON)
- ❌ `eslint .` / `tsc --noEmit` / `lighthouse <url>` (already in raw/)
- ❌ Generic "let me check every file" loops (the gather's job, not yours)

You MAY still:
- ✅ Read SPECIFIC files cited in findings (verify the issue)
- ✅ Run a SPECIFIC `grep` to falsify a finding (Popper test, see Phase 2.1)
- ✅ Run a SPECIFIC tool the gather couldn't (e.g. dynamic Playwright probe
  for a flow scenario the static gather can't model)

### 0.5 Cross-audit synthesis (read sibling evidence-summary.json files)

If this audit runs as part of a Linear-fix mission, sibling audits' summaries
are at `$PROJECT_PATH/audits/.linear-fix/<TICKET>/.<other-audit-id>/evidence-summary.json`.
Read them. Use them.

Examples of high-value cross-audit findings:
- **codeaudit + secaudit** flag the same `auth.ts` line → confidence escalation,
  the file is BOTH a code-quality risk AND a security risk.
- **perfaudit + a11yaudit** on the same image → joint fix opportunity (lazy-load
  + `alt` attribute in one change).
- **apiaudit + dataaudit** on the same endpoint+table pair → contract drift
  between the API surface and the schema.
- **debugaudit + flowaudit** report the same broken page → user-flow blocker.

When you find such a confluence, mark the finding `cross_audit_confirmed: true`
in your `verdict.json` and bump severity by one level.

---

## PHASE 0: RECONNAISSANCE

> *"Know the API surface before probing it."*

```
1. PROJECT DISCOVERY
   -> Read CLAUDE.md, README, package.json
   -> Identify: framework, API style (REST/GraphQL/tRPC/Convex)
   -> Find: base URL, auth mechanism, API docs location

2. ENDPOINT DISCOVERY
   -> Scan all route files / resolvers / actions
   -> Build complete endpoint inventory with HTTP methods
   -> Map path parameters, query parameters, body schemas
   -> Identify middleware chains per endpoint

3. AUTH DISCOVERY
   -> Auth mechanism (JWT, session, API key, OAuth)
   -> Role hierarchy (admin, user, guest, service)
   -> Auth middleware placement per route
   -> Token format, expiration, refresh mechanism
```

---

## PHASE 1: ENDPOINT INVENTORY

> *"You can't secure what you don't know exists."*

```
1. COMPLETE ENUMERATION
   -> Every route in the application listed
   -> HTTP method per route (GET, POST, PUT, PATCH, DELETE)
   -> Public vs authenticated vs admin routes
   -> Deprecated routes still active

2. ROUTE HYGIENE
   -> No debug/test routes in production
   -> No catch-all routes masking 404s
   -> No duplicate routes with different behavior
   -> Route naming consistent (plural nouns, no verbs in REST)

3. DOCUMENTATION COVERAGE
   -> Every endpoint documented (or self-documenting)
   -> OpenAPI/Swagger spec exists and is complete
   -> Examples for every endpoint
   -> Error responses documented
```

---

## PHASE 2: CONTRACT COMPLIANCE

> *"An API without a contract is a handshake agreement. It changes when someone forgets."*

```
1. REST COMPLIANCE (if REST)
   -> Resources are nouns (/users, not /getUsers)
   -> HTTP methods match semantics (GET reads, POST creates, etc.)
   -> Proper use of query params vs path params vs body
   -> HATEOAS links for discoverability (if applicable)
   -> Consistent response envelope (data, errors, meta)

2. GRAPHQL COMPLIANCE (if GraphQL)
   -> Schema well-typed with descriptions
   -> Query depth limited (prevent nested bomb)
   -> Query complexity limited
   -> Introspection disabled in production
   -> Mutations clearly separated from queries

3. RESPONSE FORMAT CONSISTENCY
   -> Same envelope structure on every endpoint
   -> Date formats consistent (ISO 8601)
   -> Null vs missing field handling consistent
   -> Nested object structure consistent
   -> Error format identical across all endpoints

4. BREAKING CHANGE DETECTION
   -> No field removals without deprecation
   -> No type changes on existing fields
   -> No required field additions to requests
   -> No semantic changes (field means something different)
```

---

## PHASE 3: AUTHENTICATION (PART OF THE HINGE)

> *"One unprotected endpoint. That's all it takes."*

```
FOR EVERY endpoint:

1. AUTH REQUIREMENT
   -> Endpoint requires authentication? (Y/N — intentional?)
   -> Public endpoints are INTENTIONALLY public
   -> No endpoints accidentally public due to missing middleware
   -> Auth check happens BEFORE any data access

2. TOKEN VALIDATION
   -> Token format validated (not just present)
   -> Token expiration checked
   -> Token signature verified
   -> Revoked tokens rejected
   -> Token refresh works correctly

3. SESSION SECURITY
   -> Session fixation prevented
   -> Session timeout configured
   -> Concurrent session handling (limit or allow)
   -> Session invalidation on password change

4. API KEY SECURITY (if applicable)
   -> Keys transmitted securely (header, not URL)
   -> Keys scoped to specific permissions
   -> Key rotation supported
   -> Compromised key revocation fast

FALSIFY: Send requests to EVERY endpoint with: no token, expired token, malformed token, other user's token. If any returns data instead of 401/403, authentication is broken.
```

---

## PHASE 4: AUTHORIZATION (PART OF THE HINGE)

> *"Authenticated is not authorized. Knowing who you are doesn't mean you can do anything."*

```
FOR EVERY endpoint:

1. ROLE-BASED ACCESS
   -> Each endpoint has defined required roles
   -> Admin endpoints reject regular users
   -> User endpoints reject guests
   -> Service endpoints reject user tokens

2. RESOURCE-LEVEL AUTH (IDOR PREVENTION)
   -> User can only access OWN resources
   -> /users/123/data rejects user 456's token
   -> No sequential ID enumeration possible
   -> UUID or opaque IDs used for external references

3. FIELD-LEVEL AUTH
   -> Sensitive fields hidden from unauthorized roles
   -> Admin-only fields not in regular user responses
   -> Write access to fields matches role permissions
   -> No mass assignment vulnerabilities

4. PRIVILEGE ESCALATION
   -> User cannot self-promote to admin
   -> Role changes require admin auth
   -> No hidden admin parameters (isAdmin=true in body)
   -> Permission checks on every mutation, not just reads

FALSIFY: For EVERY endpoint, test with EVERY role. If a lower role can access a higher role's resource, authorization is broken.
```

---

## PHASE 5: INPUT VALIDATION

> *"Trust no input. Not from users. Not from other services. Not from your own frontend."*

```
FOR EVERY parameter on EVERY endpoint:

1. TYPE VALIDATION
   -> String params reject numbers (and vice versa)
   -> Date params reject invalid dates
   -> Enum params reject unknown values
   -> Array params reject non-arrays
   -> Nested object structure validated

2. BOUNDARY VALIDATION
   -> String length limits enforced (min and max)
   -> Number range limits enforced
   -> Array size limits enforced
   -> File size limits enforced
   -> Nested depth limits enforced

3. FORMAT VALIDATION
   -> Email format validated
   -> URL format validated
   -> Phone number format validated
   -> UUID format validated
   -> Custom patterns enforced (regex)

4. INJECTION PREVENTION
   -> SQL injection: special characters handled
   -> NoSQL injection: $gt, $ne operators rejected in user input
   -> XSS: HTML/script tags stripped or escaped
   -> Path traversal: ../ rejected in file paths
   -> Command injection: shell characters escaped

5. CONTENT TYPE VALIDATION
   -> Content-Type header matches body format
   -> JSON body rejects non-JSON
   -> Multipart requests validate file types
   -> Request size limits enforced globally
```

---

## PHASE 6: ERROR RESPONSE FORMAT

```
1. CONSISTENT STRUCTURE
   -> Every error has: status code, error code, message
   -> Error format identical across all endpoints
   -> Validation errors list all invalid fields (not just first)
   -> Error codes are machine-parseable (not just HTTP status)

2. INFORMATION SECURITY
   -> Stack traces not exposed in production
   -> Database errors not exposed verbatim
   -> Internal paths not exposed
   -> User enumeration prevented (same error for existing/non-existing)

3. ACTIONABLE MESSAGES
   -> Error messages tell the caller HOW to fix the problem
   -> Validation errors specify which field and what's wrong
   -> Rate limit errors include retry-after
   -> Auth errors distinguish 401 (not authenticated) from 403 (not authorized)
```

---

## PHASE 7: STATUS CODE CORRECTNESS

```
FOR EVERY endpoint:
1. 200 OK: successful GET/PUT/PATCH with body
2. 201 Created: successful POST that creates resource (with Location header)
3. 204 No Content: successful DELETE (no body)
4. 400 Bad Request: validation errors, malformed input
5. 401 Unauthorized: missing or invalid authentication
6. 403 Forbidden: authenticated but not authorized
7. 404 Not Found: resource doesn't exist
8. 409 Conflict: duplicate creation, optimistic lock failure
9. 422 Unprocessable Entity: valid syntax but semantic errors
10. 429 Too Many Requests: rate limited (with Retry-After)
11. 500 Internal Server Error: unhandled server errors (should be rare)
12. No 200 for errors. No 500 for client mistakes.
```

---

## PHASE 8: PAGINATION

```
1. LIST ENDPOINTS PAGINATED
   -> Every list endpoint supports pagination
   -> Default page size reasonable (10-50, not 1000)
   -> Maximum page size enforced
   -> Total count available (or hasMore flag)

2. PAGINATION METHOD
   -> Cursor-based preferred for real-time data
   -> Offset-based acceptable for static data
   -> No page skipping issues with cursor-based
   -> Consistent between endpoints

3. PAGINATION METADATA
   -> Page info in response (cursor, total, hasNext, hasPrev)
   -> Link headers for discoverability (optional)
   -> Empty page returns empty array (not 404)
```

---

## PHASE 9: RATE LIMITING

```
1. RATE LIMITS PRESENT
   -> Global rate limit per IP/user
   -> Per-endpoint rate limits on expensive operations
   -> Auth endpoints rate limited (prevent brute force)
   -> File upload endpoints limited

2. RATE LIMIT HEADERS
   -> X-RateLimit-Limit (max requests)
   -> X-RateLimit-Remaining (requests left)
   -> X-RateLimit-Reset (when limit resets)
   -> Retry-After on 429 responses

3. RATE LIMIT FAIRNESS
   -> Authenticated users get higher limits than anonymous
   -> Paid tiers get higher limits
   -> Rate limits don't block legitimate usage patterns
   -> Graceful degradation (queue, not hard reject)
```

---

## PHASE 10: VERSIONING STRATEGY

```
1. VERSION PRESENT
   -> API version communicated (URL path, header, or query param)
   -> Consistent versioning across all endpoints
   -> Current version clearly documented

2. BACKWARD COMPATIBILITY
   -> Old versions still work (or sunset communicated)
   -> New required fields have defaults for old clients
   -> Removed fields return null (not error) during deprecation
   -> Version migration guide available
```

---

## PHASE 11: DOCUMENTATION ACCURACY

```
1. SPEC vs REALITY
   -> Every documented endpoint exists
   -> Every existing endpoint is documented
   -> Request schemas match actual validation
   -> Response schemas match actual responses
   -> Error codes documented and accurate

2. DOCUMENTATION QUALITY
   -> Examples for every endpoint (request + response)
   -> Auth requirements clear per endpoint
   -> Rate limits documented
   -> Changelog maintained
```

---

## PHASE 12: RESPONSE TIME BENCHMARKS

```
FOR EVERY endpoint:
1. Average response time < 200ms
2. P95 response time < 1s
3. P99 response time < 2s
4. Outlier endpoints identified (> 2s)
5. Response time under load (2x, 5x, 10x normal traffic)
6. Cold start impact (serverless/edge functions)
```

---

## PHASE 13: N+1 DETECTION

```
1. QUERY ANALYSIS
   -> Database queries per API call counted
   -> N+1 patterns identified (list endpoint + detail per item)
   -> Dataloader/batching in place for GraphQL
   -> Eager loading configured for ORM relationships

2. OVERFETCHING
   -> API returns only requested fields (sparse fieldsets or GraphQL)
   -> No SELECT * in underlying queries
   -> Nested includes limited to prevent cartesian explosion
```

---

## PHASE 14: IDEMPOTENCY

```
1. SAFE METHODS
   -> GET, HEAD, OPTIONS are truly read-only
   -> No side effects on GET requests
   -> No state changes on repeated GET calls

2. IDEMPOTENT MUTATIONS
   -> PUT is idempotent (same result on repeat)
   -> DELETE is idempotent (404 on repeat, not error)
   -> POST supports idempotency keys for critical operations
   -> Payment endpoints ALWAYS idempotent

3. RETRY SAFETY
   -> Timeout + retry doesn't create duplicates
   -> Network failure + retry doesn't corrupt state
   -> Idempotency window appropriate (24h minimum)
```

---

## PHASE 15: WEBHOOK RELIABILITY

```
1. WEBHOOK DELIVERY
   -> Retry on failure (exponential backoff)
   -> Dead letter queue for persistent failures
   -> Webhook payload includes event type and timestamp
   -> Webhook signature for verification (HMAC)

2. WEBHOOK SECURITY
   -> HTTPS only for webhook URLs
   -> Signature verification documented
   -> IP allowlist available
   -> Payload doesn't include excessive data

3. WEBHOOK MONITORING
   -> Delivery success rate tracked
   -> Failed deliveries alerting
   -> Webhook logs available to consumers
```

---

## PHASE 16: CORS CONFIGURATION

```
1. CORS HEADERS
   -> Access-Control-Allow-Origin: specific origins (not *)
   -> Access-Control-Allow-Methods: only needed methods
   -> Access-Control-Allow-Headers: only needed headers
   -> Access-Control-Max-Age: reasonable preflight cache

2. CORS SECURITY
   -> Credentials not allowed with wildcard origin
   -> No overly permissive origins
   -> Preflight requests handled correctly
   -> CORS doesn't bypass authentication
```

---

## PHASE 17: CONTENT NEGOTIATION

```
1. CONTENT TYPES
   -> Accept header respected
   -> Content-Type header validated
   -> JSON default, other formats if specified
   -> Unsupported content type returns 415

2. ENCODING
   -> UTF-8 default
   -> Compression supported (gzip, brotli)
   -> Accept-Encoding respected
```

---

## PHASE 18: API DEPRECATION HANDLING

```
1. DEPRECATION COMMUNICATION
   -> Deprecated endpoints return Deprecation header
   -> Sunset date communicated
   -> Migration guide available
   -> Deprecated endpoints log usage for sunset planning

2. SUNSET PROCESS
   -> Deprecated endpoints still work during sunset period
   -> Error messages guide to replacement
   -> Consumer notification before removal
```

---

## PHASE H1 — HYBRID SYNTHESIS (Popper / hinge / user-need / edge cases / cross-audit)

> **NEW (2026-05-08, hybrid framework, runs immediately before VERDICT):**
> "H1" = Hybrid step 1 of the synthesis layer that pairs with Phase 0's
> programmatic gather. It does NOT renumber existing phases; it sits between
> the audit's last domain phase and the VERDICT phase. Between the per-domain
> phases above and the VERDICT phase, you must run this 5-step synthesis.
> The token budget freed by Phase 0's deterministic gather is REINVESTED here
> — depth of analysis is what increases. This phase does NOT replace any
> earlier phase; it deepens them.

### 2.1 Popper falsification per finding (mandatory)

For every finding in `evidence-summary.json.findings[]` (start with `severity ∈
{critical, high}`, then go down as time/budget allows), try to PROVE the tool
is wrong. Each falsification produces a `falsifiable_tests[]` entry in
`verdict.json`:

```jsonc
{
  "claim": "ts-prune says src/auth/legacy.ts:42 export `signWithOldKey` is unused",
  "test_command": "grep -rn 'signWithOldKey' --include='*.ts' --include='*.tsx' --include='*.test.ts' --include='*.spec.ts' . | grep -v 'src/auth/legacy.ts'",
  "expected": "0 matches → claim TRUE, finding stands",
  "actual": "0 matches found",
  "outcome": "confirmed"
}
```

Outcomes:
- `confirmed` — Popper test FAILED to falsify → finding stands, often promoted
- `falsified` — Popper test produced a counter-example → demote to `info` and
  add `falsified_at: <evidence>` to the finding
- `inconclusive` — test could not run cleanly → keep severity, mark
  `confidence: medium` on this finding

**The rule:** every CLAIM in the audit (PASS or FAIL) MUST cite ≥3 concrete
commands that COULD have falsified it but didn't. Banned phrases (`looks
correct`, `should be fine`, `appears to work`) → automatic FAIL of the audit.

Common falsification patterns by category:

| Tool says | Popper test |
|---|---|
| `unused export` (ts-prune, vulture) | `grep` for the symbol in tests/, integration/, and dynamic imports (e.g. `import("...")`) |
| `unused dependency` (depcheck) | `grep` `package.json` scripts + `find . -type f -exec grep -l <pkg>` |
| `circular dep` (madge) | Read all files in the cycle — is the cycle real or a tooling artifact (e.g. type-only re-export)? |
| `console.error` / `console.warn` (debugaudit) | Reproduce the user flow that triggers it; if not reachable from any UI path, demote to info |
| `npm audit HIGH` (sec) | Check if the vulnerable code path is actually called in your codebase; not all transitive vulns are exploitable |
| `LCP > 2.5s` (perf) | Re-run lighthouse twice; check if it's a one-off (cold cache, network blip) or systematic |
| `axe-core color-contrast` (a11y) | Manually compute the ratio; some palettes hit 4.4 vs 4.5 — fixable in tokens |
| `missing canonical` (seo) | Check if the page is intentionally not canonical (paginated, faceted) before flagging |
| `unauthenticated endpoint` (api) | Read the route handler — is auth enforced via middleware not visible to the static scan? |
| `orphaned record` (data) | Verify the FK is supposed to cascade, or confirm the parent was deleted by an actual user action |

### 2.2 Hinge cross-reference (10× scrutiny on load-bearing findings)

The HINGE ENDPOINTS of this audit is the locus of maximum risk/value. Compute or
read:

```bash
# If a hinge file already exists for this ticket, use it:
HINGE_FILE="$HOME/.omega/state/hinge-points-${TICKET_ID:-default}.json"

# Otherwise compute it on the fly:
${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh "$PROJECT_PATH" --audit=api --user-need="$USER_NEED_QUOTE" \
  > "$HINGE_FILE"
```

For each finding in `evidence-summary.json`, mark
`is_load_bearing: true` IFF its file matches a hinge entry, then apply 10×
scrutiny to those findings:

- 5× more falsification attempts (Phase 2.1)
- 3× more edge-case hunts (Phase 2.4)
- Mandatory read of the entire hinge file (not just the flagged line)
- Mandatory read of all DIRECT callers of the hinge function/symbol
- Mandatory read of all DIRECT callees from the hinge

Output `hinge_findings[]` in `verdict.json`:

```jsonc
{
  "finding_id": "F-042",
  "is_load_bearing": true,
  "hinge_reference": "<hinge term + file>",
  "additional_scrutiny": "verified all 7 callers, all 4 callees, all 12 tests; reproduced bug in 3 of them",
  "confidence_after_scrutiny": "high"
}
```

### 2.3 User-need verification (`--user-need` quote)

If the audit was dispatched with `--user-need="<verbatim user complaint>"`,
every finding MUST be evaluated against it. The user-need is the ground truth
for what counts as "audit succeeded".

For each finding, ask:
- "If a user reported THIS exact issue verbatim, would this finding be the
  cause?"
- "Does fixing this finding make the user-need quote no longer true?"

Findings that DO NOT relate to user-need:
- Get demoted by one severity level UNLESS they are load-bearing
  (Phase 2.2), in which case they retain severity (load-bearing is
  user-need-independent).
- Are still reported, but flagged `user_need_relevance: "tangential"`.

Findings that DO relate to user-need:
- Get the highest priority in the fix plan
- Are listed first in `user_need_match.findings[]` of `verdict.json`

```jsonc
{
  "user_need_match": {
    "addressed": true,
    "user_need_quote": "<verbatim>",
    "rationale": "Finding F-007 (axe-core: input has no associated label) directly causes the user-reported behavior 'I can't tab to the search field'. Fix removes the cause.",
    "findings": ["F-007", "F-019"],
    "untouched_findings_relevant_to_user_need": []
  }
}
```

If `addressed: false`, the audit MUST score below 90/100 even if all phases
otherwise pass. The user's actual problem is the only correct success metric.

### 2.4 Edge case hunting (mandatory for top findings)

For each top-5 finding (sorted by severity × cross-audit-confirmed × hinge),
generate ≥2 edge cases the tool may have missed. The static analyzer or
runtime probe checked the code at rest; you must imagine motion.

Patterns:
- "If user does X under condition Y..." — concurrency, race, double-submit
- "Tool checked the file at rest, but at runtime + concurrent..." — async/await
  ordering, state mutations, cache races
- "Static scan saw the import, but the dynamic require()..." — code-splitting,
  feature-flag gated, lazy()
- "i18n locale switch..." — strings missing in non-default locale
- "Network flake mid-request..." — partial state, retry idempotency
- "User logs out mid-flow..." — session expiration, token refresh
- "Timezone boundary (DST, UTC midnight)..." — date math
- "Data shape: empty array, null, undefined, single-element..." — boundary

Output `edge_cases[]` in `verdict.json`:

```jsonc
{
  "finding_id": "F-007",
  "scenario": "User pastes a multi-line value into the search field; the
    `<input>` strips newlines silently and submits a partial query",
  "covered_by_existing_test": false,
  "evidence_gathered": "Manual repro on prod URL; screenshot at .api/edge-evidence/F-007.png",
  "fix_includes_coverage": true
}
```

### 2.5 Cross-audit synthesis (re-read sibling summaries from Phase 0.5)

Now that you have your own `verdict.json` draft, do a final pass with sibling
audits' findings open in context:

1. For each of YOUR top-5 findings: is the same file/line/symbol flagged in any
   sibling audit?
2. If yes → escalate confidence (`cross_audit_confirmed: true`), bump severity
   one level (low→medium, medium→high, high→critical, critical stays).
3. For each sibling top-5 finding: is the same file/line/symbol relevant to
   YOUR audit's domain? If yes, add it to YOUR findings as
   `tool: "cross-audit:<sibling-name>"` with proper severity.
4. Write `cross_audit_links[]` in `verdict.json` summarizing the matches.

```jsonc
{
  "cross_audit_links": [
    {
      "this_finding_id": "F-007",
      "sibling_audit": "secaudit",
      "sibling_finding_id": "F-013",
      "shared_location": "src/auth/login.tsx:42",
      "joint_fix_recommended": true
    }
  ]
}
```

### 2.6 Final verdict.json schema (hybrid v2)

```jsonc
{
  "audit": "api",
  "score": 100,
  "score_raw": "<raw>/<denominator>",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "api",
  "user_need_match": { ... },                   // §2.3
  "falsifiable_tests": [ ... ],                   // §2.1
  "hinge_findings": [ ... ],                      // §2.2
  "issues_found_and_fixed": [
    {
      "id": "FIX-001",
      "finding_id": "F-007",  // back-ref to evidence-summary.json
      "before": "<state>",
      "after": "<state>",
      "verification": "<command + output>"
    }
  ],
  "edge_cases": [ ... ],                          // §2.4
  "cross_audit_links": [ ... ],                   // §2.5
  "evidence_summary_path": "$PROJECT_PATH/audits/.apiaudit/evidence-summary.json",
  "confidence_basis": "Why I'm confident (or not). Cite Popper test counts, hinge scrutiny depth, edge-case coverage, cross-audit confirmations.",
  "banned_phrase_check": "passed (no occurrences of `looks correct`, `should be fine`, `appears to work`, `streamlined`, `to save time`)"
}
```

### 2.7 Score gating (hybrid threshold)

A 100/100 score is now blocked unless:

1. ✅ All `critical` and `high` findings are fixed OR have an explicit
   `non_issue_justification` of ≥50 words backed by Popper evidence.
2. ✅ All load-bearing findings (Phase 2.2) confirmed via Popper falsification.
3. ✅ `user_need_match.addressed = true` with verbatim quote of the user-need.
4. ✅ ≥3 falsifiable tests cited per phase (Phase 2.1).
5. ✅ ≥2 edge cases generated per top-5 finding (Phase 2.4).
6. ✅ Cross-audit synthesis attempted (Phase 2.5) — links may be empty if no
      siblings exist, but the array MUST be present.
7. ✅ `confidence_basis` populated with non-trivial reasoning.

Below threshold → score < 100, fix-and-reaudit loop kicks in (existing R-6 flow
in the FIX EXECUTION / RE-AUDIT phases below). The loop is BOUNDED at 5
iterations per the Audit Verification Contract; on iteration 5 if still failing,
emit `confidence: low` and surface as `pending` in `.done.json`.

---

## PHASE 19: VERDICT

Score each phase 0-10, weight by severity:

```
SCORING MATRIX (360 max):
  Phase  1  (Endpoint Inventory)      x 1.5  = max 15
  Phase  2  (Contract Compliance)     x 2.5  = max 25
  Phase  3  (Authentication)          x 3.5  = max 35  ** HINGE **
  Phase  4  (Authorization)           x 3.5  = max 35  ** HINGE **
  Phase  5  (Input Validation)        x 3.0  = max 30
  Phase  6  (Error Format)            x 2.0  = max 20
  Phase  7  (Status Codes)            x 2.0  = max 20
  Phase  8  (Pagination)              x 1.5  = max 15
  Phase  9  (Rate Limiting)           x 2.0  = max 20
  Phase 10  (Versioning)              x 1.5  = max 15
  Phase 11  (Documentation)           x 2.0  = max 20
  Phase 12  (Response Times)          x 2.0  = max 20
  Phase 13  (N+1 Detection)           x 2.0  = max 20
  Phase 14  (Idempotency)             x 2.0  = max 20
  Phase 15  (Webhooks)                x 1.5  = max 15
  Phase 16  (CORS)                    x 2.0  = max 20
  Phase 17  (Content Negotiation)     x 0.5  = max 5
  Phase 18  (Deprecation)             x 1.0  = max 10
                                      TOTAL  = max 360

NORMALIZE: score = (raw / 360) x 100

GRADE:
  90-100: S — Exemplary. Contract-perfect, auth-tight, documented, performant.
  80-89:  A — Strong. Minor inconsistencies, good security, reliable.
  70-79:  B — Adequate. Some validation gaps, mostly consistent.
  60-69:  C — Concerning. Auth holes or inconsistent responses.
  50-59:  D — Poor. Multiple unprotected endpoints or missing validation.
  <50:    F — Dangerous. Auth broken, validation missing, contract undefined.
```

---

## PHASE 20: FIX PLAN (automatic)

```
Sort: CRITICAL -> HIGH -> MEDIUM -> LOW
Priority: auth > authorization > validation > consistency > cosmetic
Generate fix tasks with endpoint:method specificity
Save to audits/.apiaudit/fix-plan.json + fix-plan.md
```

---

## PHASE 21: FIX EXECUTION (automatic)

```
Sequential per fix group.

─── SAFETY GATE: DO NO HARM (MANDATORY before EVERY fix) ──────────────

The audit MUST NOT introduce new bugs. A fix that breaks the code is worse than
the original finding. Every fix goes through this gate BEFORE commit.

PRE-FIX ANALYSIS (before writing ANY code):
  a. Read the ENTIRE target file (not just the target line)
  b. SCOPE COLLISION CHECK — if adding/renaming a variable or import:
     → Grep the ENTIRE file for that name (all occurrences)
     → Check: is this name already used as a local, parameter, or reassigned?
     → Check: does this name get shadowed later in the same scope?
     → If collision found → use a different name or fully-qualified reference
  c. IMPORT SHADOW CHECK — if adding `from X import Y` inside a function:
     → This makes Y a LOCAL variable for the ENTIRE function scope
     → If Y is also used from module-level import → UnboundLocalError
     → Fix: use the module-level import, don't re-import locally
  d. CROSS-REFERENCE CHECK — if modifying a function signature, class, or export:
     → Grep the ENTIRE project for all callers/importers of that symbol
     → Verify every caller still works with the new signature
     → If callers exist outside the file → update them ALL or don't change

POST-FIX VERIFICATION (after writing code, BEFORE commit):
  a. SYNTAX CHECK:
     → Python: `python -c "import ast; ast.parse(open('FILE').read())"`
     → JS/TS: `npx tsc --noEmit` or `node -c FILE`
  b. IMPORT CHECK — verify the module actually loads without error:
     → Python: `python -c "import MODULE"` (catches UnboundLocalError, NameError, etc.)
     → JS: `node -e "require('./FILE')"`
  c. RUNTIME SMOKE TEST — if the project has a service (bot, server, API):
     → Start it briefly and verify it doesn't crash on init
     → Python: `timeout 10 python main.py` or systemctl restart + is-active check
     → Node: `timeout 10 node server.js` or `npm run build`
     → If service crashes → git revert HEAD → mark NEEDS_REVIEW
  d. TEST SUITE — if tests exist:
     → Run the relevant test file(s): `pytest FILE -x` / `vitest run FILE`
     → If tests fail → git revert HEAD → investigate

IF ANY POST-FIX CHECK FAILS:
  → `git revert HEAD` immediately
  → Log the failure in audits/.apiaudit/fix-log.md with exact error
  → Mark as NEEDS_REVIEW (never retry same approach blindly)
  → Try alternative approach OR skip this fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (in priority order):
  a. Read the ENTIRE target file (full context)
  b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
  c. Apply fix
  d. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
  e. If all green → commit: api(apiaudit): FIX-XXX description
  f. If any red → revert → log → mark NEEDS_REVIEW
  g. Verify AFTER state (send the same attack, confirm blocked)
  h. Verify: no regression in other endpoints
```

---

## PHASE 22: RE-AUDIT (automatic)

```
1. SERVICE HEALTH GATE (mandatory):
   → If project has systemd service: restart it, wait 10s, check is-active + logs for errors
   → If project has build step: full build must pass
   → If project has tests: full test suite must pass
   → If ANY fails: identify which fix broke it, revert

2. Re-run all FAILING phases. Compare before/after.
3. Loop until score >= 80 or remaining items are NEEDS_REVIEW.
```

---

## PARALLEL EXECUTION STRATEGY

```
WAVE 1 (discovery -- sequential):
  Phase 0: Reconnaissance

WAVE 2 (security -- parallel):
  Phase 3: Authentication     ]
  Phase 4: Authorization      ] Critical security
  Phase 5: Input Validation   ] analysis
  Phase 16: CORS              ]

WAVE 3 (contract -- parallel):
  Phase 1: Endpoint Inventory ]
  Phase 2: Contract Compliance] Contract
  Phase 6: Error Format       ] verification
  Phase 7: Status Codes       ]

WAVE 4 (operations -- parallel):
  Phase 8: Pagination          ]
  Phase 9: Rate Limiting       ]
  Phase 12: Response Times     ] Operational
  Phase 13: N+1 Detection     ] analysis
  Phase 14: Idempotency       ]

WAVE 5 (ecosystem -- parallel):
  Phase 10: Versioning         ]
  Phase 11: Documentation      ]
  Phase 15: Webhooks           ] Ecosystem
  Phase 17: Content Negotiation] checks
  Phase 18: Deprecation        ]

4 parallel agents per wave = 18 phases in 5 sequential waves.
```

---

## CROSS-COMMAND BRIDGE

```
/apiaudit finds auth issues -> references /secaudit findings
/apiaudit finds perf issues -> references /perfaudit findings
/apiaudit finds data issues -> references /dataaudit findings
/apiaudit finds validation gaps -> fixes directly

THE QUALITY ARSENAL:
  /codeaudit    -> Is the code SOLID?           (preventive)
  /flowaudit    -> Does the experience WORK?    (preventive)
  /uiuxaudit    -> Is the interface BEAUTIFUL?  (preventive)
  /debugaudit   -> What is BROKEN right now?    (detective)
  /featureaudit -> Is the product COMPLETE?     (strategic)
  /perfaudit    -> Is it FAST enough?           (detective)
  /secaudit     -> Is it SECURE?                (detective)
  /a11yaudit    -> Can EVERYONE use it?         (inclusive)
  /seoaudit     -> Can search engines FIND it?  (growth)
  /dataaudit    -> Is the data INTACT?          (integrity)
  /apiaudit     -> Is the API RELIABLE?         (contract)
  /copyaudit    -> Does the copy DELIVER?       (messaging)
  /dxaudit      -> Can devs SHIP efficiently?   (velocity)

  Together: nothing escapes. Every dimension covered.
```

---

## LAWS

1. **Every endpoint is a door. Lock the ones that should be locked.** Auth is not optional on any mutation.
2. **Trust no input.** Validate everything. Type, format, range, length, content. Everything.
3. **Errors are part of the contract.** A well-formatted error is more useful than a poorly-formatted success.
4. **Idempotency saves systems.** Retry-safe APIs survive network failures. Non-idempotent APIs create duplicates.
5. **Documentation is a promise (Popper).** If the spec says X and the API does Y, the spec is a lie and the consumer will build on it.
6. **Rate limiting is kindness.** It protects the API, the database, and every other consumer.
7. **CORS is not security theater.** Misconfigured CORS enables cross-site attacks. Configure it precisely.

---

*"/apiaudit v1 — Validate. Authenticate. Authorize. Every endpoint, every parameter, every role. /360."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.1 — 2026-04-14T10:35:36Z)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial thinking
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.apiaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — Non-UI contexts: /apiaudit is PRIMARY audit for backend-only APIs. ABORT for pure frontend (no API surface).
- ✅ **Output contract verification** — emits `audits/.apiaudit/verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `iterations.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `audits/.apiaudit/discovery/` > 1h old, re-verify inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.apiaudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` flags with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — Rate-Limit Safety + REST/GraphQL Weighting + Ownership

**Rate-limit safety (per preamble §12):**
- Same rules as /secaudit: max 10 req/s default, abort on 3 consecutive 429/503
- Required: `--prod` flag + explicit confirmation before any non-dev target
- Log all requests to `audits/.apiaudit/request-log.txt`

**REST vs GraphQL weighting:**
```
If openapi.yaml / swagger.json / .spec.yaml present:
  → REST mode with OpenAPI ground truth
  → Schema compliance is binary (matches spec or not)
Elif schema.graphql + GraphQL resolvers present:
  → GraphQL mode, use introspection query
  → Validate resolver signatures against schema types
Elif both present:
  → Dual mode, audit both surfaces independently
Else:
  → Inference mode: reconstruct contract from handler code
  → Flag findings as "inferred" (lower confidence) vs "explicit"

Both REST and GraphQL weighted equally in scoring (each endpoint contributes same weight).
```

**Auth overlap with /secaudit:**
- /apiaudit owns: contract correctness (auth middleware present, JWT verification logic correct, API key rotation spec documented)
- /secaudit owns: exploitation testing (can I bypass auth? brute-force the API key?)
- No duplicate findings — distinct evidence types.

### Scoring addenda

Normalized score = (raw_score / applicable_raw_max) × 100.
Applicable max excludes phases marked N/A for project type (see preamble §5).
`verdict.json.preamble_version = "1.0"` is MANDATORY. Missing field = /metaudit flags as non-compliant.

### /metaudit Compliance Badge

Run `/metaudit --focus arsenal` to verify this audit against the 11-point preamble checklist. Target score: 11/11.

---

*v1.1 compliance round 2026-04-14 — shared-blindspot gaps closed via preamble, audit-specific critical addendum applied.*

---

## § 100/100 COMPLIANCE CERTIFICATE — v1.2 — 2026-04-14T10:43:29Z

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (apiaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "apiaudit",
  "preamble_version": "1.0",
  "compliance_version": "1.2",
  "compliance_score": 100,
  "shared_blindspots_closed": {
    "S1_concurrency_lock": true,
    "S2_reaudit_cap_5_iter": true,
    "S3_non_ui_context_handling": true,
    "S4_telegram_progress": true,
    "S5_scoped_invocation_flags": true,
    "S6_output_contract_verification": true,
    "S7_self_telemetry": true,
    "S8_phase_count_rationale_documented": true,
    "S9_discovery_drift_check": true,
    "S10_deprecation_registry_reference": true
  },
  "audit_specific_gaps_closed": true,
  "rule_46_compliant": true,
  "integration_smoke_test_gate": true,
  "rate_limit_safety": true,
  "last_updated": "2026-04-14T10:43:29Z"
}
```

### Gap closures (pre-100 findings → now resolved)

**Gap 1: REST/GraphQL dual-mode formal → RESOLVED**
```
Mode detection (Phase 0 step 0.5):
  has_openapi = exists(openapi.yaml) || exists(swagger.json) || exists(api-spec.yaml)
  has_graphql = exists(schema.graphql) || has_graphql_deps(package.json)

  if has_openapi && has_graphql:
    dual_mode
    Audit both surfaces independently, emit findings tagged mode: "rest" | "graphql"
  elif has_openapi:
    rest_mode_explicit — ground truth = openapi spec
  elif has_graphql:
    graphql_mode_explicit — ground truth = schema.graphql (introspection)
  else:
    inference_mode — reconstruct contract from handler code
    Tag all findings inferred: true (lower confidence)
  Emit in verdict.json: mode, confidence (explicit | inferred)
```

**Gap 2: Rate-limit request log → RESOLVED**
Same schema as /secaudit (gap 3): `audits/.apiaudit/request-log.txt` with per-request records. Max 10 req/s default.

**Gap 3: Auth overlap contract with /secaudit → RESOLVED**
```
Ownership:
  /apiaudit owns:
    - Auth middleware presence on each route (static check)
    - JWT verification logic correctness (static code audit)
    - Session expiry logic spec compliance
    - API key rotation documentation
    - Scope / permission check coverage (RBAC)
  /secaudit owns:
    - Actually bypassing auth (runtime exploitation)
    - Brute-force testing
    - Token stealing / XSS leverage
    - Session fixation exploitation
Shared finding type: if /apiaudit says "no auth check on GET /admin" AND /secaudit confirms exploit: CRITICAL (both agree).
```

**Gap 4: Inference-mode confidence → RESOLVED**
Each inferred finding includes `confidence: "low" | "medium" | "high"` based on code clarity:
- High: explicit route handler + typed inputs + documented response
- Medium: route handler with inferred types
- Low: dynamic route or unknown handler

### Final score calculation

```
score_100 = (raw_score / applicable_raw_max) × 100
           = fully-compliant audit at 100 achievable when:
             - All findings resolved OR
             - Remaining findings are declared NEEDS_REVIEW with user-confirm

After v1.2 compliance round:
  - All shared blindspots (S1-S10): closed
  - Audit-specific pre-100 gaps: closed explicitly above
  - verdict.json.preamble_version = "1.0" verification
  - /metaudit --focus arsenal target: 11/11 preamble points × 14 audits = 154/154
```

### Verification command

```bash
# Run /metaudit to verify this audit's 100/100 claim:
/metaudit --focus arsenal --scope="apiaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*apiaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

---

## MANDATORY BEFORE/AFTER VERIFICATION (v1.1)

**Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix execution.**

Every fix MUST follow the "Do No Harm" protocol:

1. **PRE-FIX BASELINE** — grep all references, capture functional state, save to `audits/.<audit-id>/baseline/`.
2. **APPLY FIX** — normal execution.
3. **POST-FIX CHECK** — repeat every baseline check. If any PASSED→FAILED transition occurs, revert immediately.
4. **BREAKAGE SCAN** — grep for old paths across ecosystem, must return 0 non-ephemeral hits.
5. **BEFORE/AFTER MATRIX** — produce `audits/.<audit-id>/before-after.md` with functional status table per affected item.

**An audit that breaks 1 working thing is WORSE than no audit.** Do NOT claim "done" without `before-after.md` showing zero regressions.

---

## Dynamic-Workflow Orchestration (v2)

> *"The forensic pathologist does not autopsy one organ at a time when ten tables are open. Probe every endpoint surface at once, then cross-examine each corpse with three independent witnesses before signing the certificate."*

This section governs **HOW /apiaudit executes WHEN RUN**. It is an execution-engine upgrade only — it does **not** add, remove, reorder, or reweight any phase (1–18, H1, 19–22), and it does **not** alter the `/360` scoring matrix, the grade bands, the verdict format, or the frontmatter. It supersedes the static "PARALLEL EXECUTION STRATEGY" waves above by running them through the **Workflow tool** (true concurrent fan-out) instead of conceptual agent-waves, and it hardens every finding with adversarial verification before it is allowed to touch the scoring matrix. The Gestalt-Popper doctrine, the 5 Laws of API Forensics, and the HINGE ENDPOINTS clarity gate remain fully in force — this only changes the *plumbing*, never the *judgement*.

### 1. Decompose into INDEPENDENT parallel tracks (fan-out via the Workflow tool)

After Phase 0 (programmatic gather) and Phase 0 RECONNAISSANCE complete (these stay sequential — they produce `evidence-summary.json` + the endpoint/auth/role inventory that every track reads), dispatch the domain phases as **concurrent Workflow tracks** rather than a linear 1→18 walk. Tracks are chosen so they touch **disjoint evidence and write disjoint report files** (no two tracks mutate the same `reports/*.md`), satisfying R-SCOPE one-writer-per-file:

| Track | Phases fanned concurrently | Why independent | Writes |
|-------|---------------------------|-----------------|--------|
| **T1 — Hinge Security** (10x depth) | 3 Authentication, 4 Authorization, 16 CORS | The HINGE ENDPOINTS (money/auth/user-data) — max scrutiny, runs alone-but-parallel so it never starves | `authentication.md`, `authorization.md`, `cors.md` |
| **T2 — Input & Contract** | 2 Contract Compliance, 5 Input Validation, 6 Error Format, 7 Status Codes | All read request/response schemas + handler signatures | `contract-compliance.md`, `input-validation.md`, `error-format.md`, `status-codes.md` |
| **T3 — Surface & Ops** | 1 Endpoint Inventory, 8 Pagination, 9 Rate Limiting, 12 Response Times, 13 N+1, 14 Idempotency | Operational behavior per route, independent of auth verdicts | `endpoint-inventory.md`, `pagination.md`, `rate-limiting.md`, `response-times.md`, `n-plus-one.md`, `idempotency.md` |
| **T4 — Ecosystem & Contract Hygiene** | 10 Versioning, 11 Documentation, 15 Webhooks, 17 Content Negotiation, 18 Deprecation | Spec/lifecycle concerns, no overlap with security or per-request ops | `versioning.md`, `documentation.md`, `webhooks.md`, `content-negotiation.md`, `deprecation.md` |

Rules for the fan-out:
- **Each track is a Workflow sub-task** with its own fresh context, handed: the path to `evidence-summary.json`, the hinge file (`~/.omega/state/hinge-points-<ticket>.json`), the REST-vs-GraphQL mode (from the v1.2 mode-detection block), and its exact phase list + the report files it (and only it) may write.
- **T1 always runs** even on the smallest scope — auth/authorization is the hinge; it is never collapsed into another track and never skipped.
- Tracks sharing a file MUST serialize; here the table is engineered so they never do. If a scoped run (`--files=`/`--focus=`) makes two tracks target the same handler file, **merge them into one track** rather than letting two writers collide.
- Fan-out is **bounded by budget** (R-BUDGET, default 500K) — if approaching the cap, run T1 first to completion, then the rest; never silently drop a track.
- This is a *concurrency* change, not a *coverage* change: **all 18 phases still run at full depth.** No `--quick`/`--streamlined` shortcut is introduced (rule 46 / Law L5).

### 2. ADVERSARIALLY VERIFY every finding (≥2-of-3 independent lenses before acceptance)

A track's raw finding is a **candidate**, never a verdict (R-VERIFY). Before any candidate is allowed into Phase H1 synthesis or the Phase 19 scoring matrix, it must survive **≥2 of these 3 independent lenses** — run as parallel verification micro-tasks, each blind to the others' conclusion:

- **Lens A — REPRODUCE (runtime, Law L1):** Actually exercise the endpoint and observe the real response. Send the offending request against the dev/`--prod`-confirmed target (honoring the rate-limit safety gate: ≤10 req/s, abort on 3×429/503, log to `request-log.txt`). For an "unauthenticated endpoint" candidate → fire the request with **no token / expired token / another user's token** and capture the actual status + body. A 200 where 401/403 was claimed-as-required = REPRODUCED. *Code that says "auth required" but returns data is a lie; runtime wins.*
- **Lens B — REFUTE (Popper falsification):** Actively try to PROVE the finding WRONG, mapping to the audit's own falsification categories (HAPPY vs EDGE, ADMIN vs USER, SINGLE vs CONCURRENT, SPEC vs REALITY, POSTMAN vs PRODUCTION). For "missing input validation" → submit the exact malformed/boundary/injection payload and see if it's actually rejected upstream (middleware not visible to the static scan — see the Phase H1 falsification table row for `api`). If the refutation succeeds, the candidate is **killed**.
- **Lens C — CROSS-CHECK (static + spec + cross-audit):** Read the handler + its middleware chain end-to-end; reconcile against OpenAPI/`schema.graphql` ground truth (SPEC vs REALITY) and against sibling `evidence-summary.json` files (Phase 0.5 / H1 §2.5 — e.g. `dataaudit` on the same table, `secaudit` on the same auth line). Confirmation here both validates the finding and sets `cross_audit_confirmed`.

Verdict rule per candidate:
- **≥2 lenses confirm → finding STANDS** (becomes `confirmed`; record each lens result as a `falsifiable_tests[]` entry per Phase H1 §2.1, with verbatim command + output).
- **≤1 lens confirms (or any lens refutes it) → finding is KILLED** — demoted to `info` with `falsified_at: <evidence>`, and it MUST NOT affect the phase score. A killed finding never costs the API points.
- **HINGE ENDPOINT findings get 3-of-3** (all three lenses mandatory, per the Phase H1 §2.2 10x-scrutiny contract: 5x falsification, all callers/callees read).
- **Banned phrases** (`looks correct`, `should be fine`, `appears to work`) on any lens = automatic FAIL of that verification, treat as unconfirmed.

This is the engine that feeds — and is gated by — the existing Phase H1 §2.1/§2.7: a finding can only enter `verdict.json` with the score-gating intact.

### 3. SYNTHESIZE survivors back into the EXISTING scoring matrix (unchanged)

The surviving (`confirmed`) findings — and only those — flow into the audit's **existing** machinery with zero modification:
- Score each phase 0–10 exactly as **Phase 19** dictates, apply the **unchanged weights** (Auth ×3.5, Authz ×3.5, Input Validation ×3.0, … Content Negotiation ×0.5), sum to the **/360 raw max**, and normalize `(raw / 360) × 100` into the **same S/A/B/C/D/F grade bands**.
- Killed findings contribute nothing; confirmed findings deduct from their phase per the existing rubric.
- Emit the **same `verdict.json` (hybrid v2) schema** (§2.6): `falsifiable_tests[]`, `hinge_findings[]`, `cross_audit_links[]`, `user_need_match`, `score`/`score_normalized`, `preamble_version: "1.0"`. The score-gating of §2.7 (no 100/100 unless all critical/high fixed-or-justified, hinge confirmed, user-need addressed, ≥3 falsifiable tests/phase) is **the acceptance gate for this whole orchestration**.
- Synthesis is the auditor's OWN job (R-ORCH): do **not** paste a track's or a lens's summary verbatim as the verdict — reconcile across tracks (e.g. a contract-drift finding in T2 that a T1 auth finding makes critical) and resolve conflicts before scoring.
- Fix Plan (Phase 20), Fix Execution (Phase 21, incl. the DO-NO-HARM safety gate), and Re-Audit (Phase 22) are **untouched** and run exactly as written on the synthesized finding set.

### 4. LOOP-UNTIL-DRY for unknown-size discovery

Endpoint/parameter/role space is **not known a priori** — new routes, dynamic handlers, GraphQL resolvers, and webhook event types surface only as tracks dig. Run discovery as a **bounded loop-until-dry** rather than a single pass:

```
known := endpoints/params/roles from Phase 0 gather + RECONNAISSANCE inventory
repeat:
  fan out the 4 tracks (§1) over `known`
  adversarially verify each candidate (§2)
  newly_revealed := surfaces discovered DURING this round
     (e.g. an authz probe exposes /admin/* subroutes; a GraphQL introspection
      reveals an unscanned mutation; a 3xx redirect chain exposes a v2 route;
      a webhook handler references an undocumented event type)
  known := known ∪ newly_revealed
until newly_revealed == ∅   (DRY)
   OR round == 5            (hard cap — aligns with the 5-iteration re-audit cap)
   OR budget cap approached (R-BUDGET → escalate, don't silently overrun)
```

- "Dry" = a full fan-out round adds **zero** new endpoints/params/roles/event-types and **zero** new confirmed findings.
- The loop is **bounded at 5 rounds** (consistent with the Phase H1 §2.7 / preamble §4 re-audit cap). On round 5 still discovering → record `halt_reason: max_iter`, emit `confidence: low`, surface remaining surfaces as `NEEDS_REVIEW`, and send the Telegram SOS — **never** spin indefinitely.
- Each round's net-new surfaces and net-new confirmed findings are appended to `session.log` so the discovery trajectory is auditable (R-CITE: evidence or it didn't happen).

**Net effect:** same phases, same weights, same verdict, same Gestalt-Popper identity — executed concurrently, with every finding triple-checked and every hidden endpoint flushed out, instead of a single linear best-effort pass.
