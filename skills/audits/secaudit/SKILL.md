---
name: secaudit
description: >
  Forensic security audit v1 (Gestalt-Popper). 25-phase deep analysis of everything that is
  VULNERABLE RIGHT NOW: OWASP Top 10 verification, XSS testing (25+ payload patterns),
  SQL/NoSQL injection, CORS misconfiguration, CSP headers audit, authentication bypass,
  session management, JWT security, IDOR detection, SSRF probing, open redirect testing,
  file upload vulnerabilities, rate limiting verification, brute force protection,
  secrets scanning (env vars, git history, JS bundles), dependency CVE audit,
  SSL/TLS configuration, security headers audit, API authentication verification,
  input validation completeness, plus verdict, fix plan, fix execution, re-audit,
  and rate-limit safety gate. Score /400. Preamble v1.0 compliant. Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/secaudit", "security audit", "is it secure", "vulnerability scan",
  "pentest the code", "find vulnerabilities", "security review", "owasp audit".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: security
phases: 25
max_score: 400
read_only: false
triggers: ["sec", "security", "owasp", "vulnerab", "pentest-light", "is it secure"]
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

# /secaudit v1 — Forensic Security Audit (Gestalt-Popper)

> *"The other audits ask 'does it work?' I ask 'can someone make it work AGAINST you?'"*

---

## DOCTRINE

You are not a security scanner. You are a **security forensic pathologist**. The running system is your patient — possibly hemorrhaging secrets, definitely over-trusting input, pretending to be secure because nobody attacked it yet. Your job is to find every vulnerability, every misconfiguration, every trust assumption that an attacker will exploit while your Lighthouse score says "best practices: 100."

**The 7 Laws of Security Forensics (Gestalt-Popper Synthesis):**
1. **If it accepts input, it's guilty.** Every user input, every API parameter, every URL segment, every cookie, every header is an attack surface. Treat all external data as hostile until proven sanitized.
2. **Green padlocks lie (Popper).** HTTPS doesn't mean secure. A valid SSL cert doesn't mean the app isn't leaking tokens in URLs. FALSIFY every green indicator — dig beneath the surface.
3. **Every secret is one commit away from public.** That API key "safely" in .env was in a commit 47 commits ago. That JWT secret "nobody knows" is in a minified bundle. Each secret has a blast radius — measure it.
4. **Clarity before attacking (Gestalt).** Before launching any scanner, UNDERSTAND the product. Read VISION.md, CLAUDE.md, README. Identify the **SECURITY HINGE POINT** — the authentication/authorization boundary that protects the entire system. Audit the hinge point with 10x depth.
5. **Defaults are the enemy (Popper).** Default CORS: permissive. Default CSP: none. Default rate limiting: absent. Default session timeout: never. Every framework default is a vulnerability until explicitly hardened. FALSIFY every "the framework handles it" claim.
6. **Defense in depth or no defense at all.** A single validation layer is a single point of failure. Client-side validation without server-side is decoration. Server-side without database constraints is optimism. Check every layer independently.
7. **The attacker only needs one path (Popper).** You secured 99 endpoints. The 100th has an IDOR. You validated 50 inputs. The 51st allows injection. FALSIFY the claim "we're secure" by finding the ONE path that breaks everything.

**Gestalt Security Hinge Point:** Before Phase 1, identify THE security boundary that protects the entire system. The auth middleware. The API gateway. The session validator. THIS boundary gets every phase at maximum depth. If it falls, everything falls.

**Popper Security Falsification Categories:**
- **CLAIM vs REALITY** — "We use bcrypt" but password reset tokens are predictable
- **CLIENT vs SERVER** — Validated in React, raw in the API handler
- **AUTH vs AUTHZ** — User is logged in, but can access other users' data
- **CONFIG vs RUNTIME** — .env says SECURE=true, but the middleware isn't loaded
- **FRAMEWORK vs APPLICATION** — Next.js handles CSRF, but the custom API route doesn't

---

## SCOPE DETECTION (automatic)

```
EXAMPLES:
  "/secaudit"
  -> Full 20-phase pipeline. Discover all attack surfaces, test everything.

  "/secaudit the auth system"
  -> TARGETED: only authentication and authorization
  -> All phases scoped to auth routes, session handling, token management

  "/secaudit api"
  -> API-FOCUSED: endpoint security, input validation, authz checks

  "/secaudit headers"
  -> HEADERS-FOCUSED: CSP, CORS, HSTS, X-Frame-Options, security headers

  "/secaudit secrets"
  -> SECRETS-FOCUSED: env vars, git history, JS bundles, exposed credentials

  "/secaudit after deploy"
  -> POST-DEPLOY mode: compare before/after, focus on new attack surfaces

  "/secaudit dependencies"
  -> SUPPLY-CHAIN mode: CVE audit, outdated packages, malicious deps
```

---

## OUTPUT CONTRACT

```
audits/.secaudit/
|-- session.log
|-- discovery/
|   |-- attack-surface.json       # All discovered endpoints/inputs
|   |-- auth-boundaries.json      # Authentication/authorization map
|   |-- secrets-inventory.json    # All secrets locations (redacted values)
|   |-- dependencies.json         # Full dependency tree with versions
|   |-- headers-baseline.json     # Current security headers per route
|-- reports/
|   |-- owasp-top-10.md              # Phase 1
|   |-- xss-testing.md               # Phase 2
|   |-- injection-testing.md         # Phase 3
|   |-- cors-audit.md                # Phase 4
|   |-- csp-audit.md                 # Phase 5
|   |-- auth-bypass.md               # Phase 6
|   |-- session-management.md        # Phase 7
|   |-- jwt-security.md              # Phase 8
|   |-- idor-detection.md            # Phase 9
|   |-- ssrf-probing.md              # Phase 10
|   |-- open-redirect.md             # Phase 11
|   |-- file-upload.md               # Phase 12
|   |-- rate-limiting.md             # Phase 13
|   |-- brute-force.md               # Phase 14
|   |-- secrets-scanning.md          # Phase 15
|   |-- dependency-cve.md            # Phase 16
|   |-- ssl-tls.md                   # Phase 17
|   |-- security-headers.md          # Phase 18
|   |-- api-auth.md                  # Phase 19
|   |-- input-validation.md          # Phase 20
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
~/.omega/lib/audit-runner.sh sec "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes `~/.omega/lib/audit-gather/sec.sh` which runs:
npm audit, pip-audit (Python), gitleaks (secrets in repo + git history), semgrep --config=auto (CWE/OWASP rules), eslint security plugin if present, .env file inventory + git-tracked classification, HTTP security-header probe

Output is written to:

```
$PROJECT_PATH/audits/.secaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), the artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.secaudit/` so multiple audits on the same
ticket can cross-reference each other (see 0.5).

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "sec",
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

> *"Map the fortress before testing the walls."*

```
1. PROJECT DISCOVERY
   -> Read CLAUDE.md, README, package.json/pyproject.toml
   -> Identify: stack, framework, auth provider, database, hosting
   -> Find: prod URL, dev URL, API base, admin panels
   -> Map: environment variables, config files, secrets management

2. ATTACK SURFACE MAPPING
   -> Scan all routes (API endpoints, pages, webhooks, WebSocket handlers)
   -> Identify all input vectors: forms, URL params, headers, cookies, file uploads
   -> Map authentication boundaries (public vs protected routes)
   -> Identify third-party integrations (OAuth, payment, email, etc.)

3. AUTH ARCHITECTURE ANALYSIS
   -> Authentication method: session, JWT, OAuth, API key, or hybrid
   -> Authorization model: RBAC, ABAC, row-level, or ad hoc
   -> Session storage: cookie, localStorage, sessionStorage, server-side
   -> Token lifecycle: creation, refresh, revocation, expiry

4. SECURITY HINGE POINT IDENTIFICATION
   -> Identify THE middleware/function that gates all protected resources
   -> Map trust boundaries: what's before auth, what's after
   -> Identify bypass paths: direct DB access, internal APIs, WebSocket
   -> This becomes ground zero for maximum-depth testing
```

---

## PHASE 1: OWASP TOP 10 VERIFICATION

> *"The industry's most exploited vulnerabilities. If you have even one, you're a target."*

```
FOR THE ENTIRE APPLICATION:

1. A01:2021 — BROKEN ACCESS CONTROL
   -> Vertical privilege escalation: can user access admin routes?
   -> Horizontal privilege escalation: can user A access user B's data?
   -> Missing function-level access control on API endpoints?
   -> Insecure direct object references (covered in depth in Phase 9)?
   -> Metadata manipulation: JWT claims, hidden form fields, cookies?

2. A02:2021 — CRYPTOGRAPHIC FAILURES
   -> Sensitive data transmitted over HTTP?
   -> Weak hashing algorithms (MD5, SHA1) for passwords?
   -> Hardcoded encryption keys or salts?
   -> Sensitive data in URL parameters (tokens, passwords, PII)?
   -> Missing encryption at rest for PII/payment data?

3. A03:2021 — INJECTION
   -> SQL injection in raw queries (covered in depth in Phase 3)?
   -> NoSQL injection in MongoDB/Firestore queries?
   -> Command injection via child_process/exec/system calls?
   -> Template injection (SSTI) in server-side templates?
   -> LDAP injection, XPath injection, header injection?

4. A04:2021 — INSECURE DESIGN
   -> Missing rate limiting on sensitive operations?
   -> No account lockout after failed login attempts?
   -> Predictable resource identifiers (sequential IDs)?
   -> Missing re-authentication for sensitive actions?
   -> Business logic flaws in payment/subscription flows?

5. A05:2021 — SECURITY MISCONFIGURATION
   -> Default credentials on admin panels/databases?
   -> Unnecessary features enabled (directory listing, debug mode)?
   -> Missing security headers (covered in Phase 18)?
   -> Overly permissive CORS (covered in Phase 4)?
   -> Stack traces/error details exposed to users?

6. A06:2021 — VULNERABLE COMPONENTS
   -> Known CVEs in dependencies (covered in Phase 16)?
   -> Outdated frameworks with known exploits?
   -> Unmaintained packages with no security patches?

7. A07:2021 — AUTH FAILURES
   -> Weak password policies (length, complexity, common passwords)?
   -> Missing MFA on sensitive accounts?
   -> Session fixation vulnerabilities?
   -> Credential stuffing protection?

8. A08:2021 — SOFTWARE/DATA INTEGRITY
   -> Unsigned updates or deployments?
   -> Missing SRI (Subresource Integrity) for CDN resources?
   -> Insecure deserialization of user-controlled data?
   -> CI/CD pipeline security (secrets in logs, unsigned artifacts)?

9. A09:2021 — LOGGING & MONITORING FAILURES
   -> Login failures not logged?
   -> Access control failures not logged?
   -> Sensitive data in logs (passwords, tokens, PII)?
   -> No alerting on suspicious activity patterns?

10. A10:2021 — SSRF (covered in depth in Phase 10)
    -> Server-side requests with user-controlled URLs?

FALSIFY: For each category, don't check if protection EXISTS. Prove it can be BYPASSED.
```

---

## PHASE 2: XSS TESTING (25+ PAYLOAD PATTERNS)

> *"Every unescaped output is a script injection waiting to happen."*

```
1. REFLECTED XSS — test every input that appears in response:
   -> Basic:           <script>alert(1)</script>
   -> Event handlers:  <img src=x onerror=alert(1)>
   -> SVG:             <svg onload=alert(1)>
   -> Body onload:     <body onload=alert(1)>
   -> Input autofocus: <input autofocus onfocus=alert(1)>
   -> Details/summary: <details open ontoggle=alert(1)>
   -> Iframe:          <iframe src="javascript:alert(1)">
   -> Marquee:         <marquee onstart=alert(1)>
   -> Object:          <object data="javascript:alert(1)">
   -> Math:            <math><mtext><table><mglyph><svg><mtext><textarea><path d="M 0 0"></textarea><img src=x onerror=alert(1)>

2. STORED XSS — test every input that persists:
   -> User profiles (name, bio, avatar URL)
   -> Comments, messages, reviews
   -> File names in uploads
   -> Form fields saved to database
   -> Markdown/rich text content

3. DOM-BASED XSS — analyze client-side JS:
   -> document.write with user input
   -> innerHTML/outerHTML assignments
   -> eval() with user-controllable data
   -> location.hash/search used unsafely
   -> postMessage handlers without origin check
   -> jQuery .html() with user data
   -> dangerouslySetInnerHTML in React

4. FILTER BYPASS PAYLOADS:
   -> Case variation:    <ScRiPt>alert(1)</ScRiPt>
   -> Null bytes:        <scr%00ipt>alert(1)</script>
   -> Double encoding:   %253Cscript%253E
   -> Unicode:           <script>alert\u0028document.domain\u0029</script>
   -> HTML entities:     &#x3C;script&#x3E;
   -> Polyglot:          jaVasCript:/*-/*`/*\`/*'/*"/**/(/* */oNcliCk=alert() )//%0D%0A
   -> Template literal:  ${alert(1)}
   -> Protocol bypass:   javascript:alert(1)//http://
   -> Mutation XSS:      <noscript><p title="</noscript><img src=x onerror=alert(1)>">

5. CONTEXT-SPECIFIC:
   -> Inside HTML attribute: " onmouseover="alert(1)
   -> Inside JavaScript:     ';alert(1)//
   -> Inside URL:            javascript:alert(1)
   -> Inside CSS:            expression(alert(1))
   -> Inside JSON:           {"key":"value</script><script>alert(1)//"}
   -> Inside template:       {{constructor.constructor('alert(1)')()}}

6. CSP BYPASS ATTEMPTS (if CSP exists):
   -> JSONP endpoints as script-src
   -> Angular/Vue template injection within allowed domains
   -> Base tag injection to redirect relative URLs
   -> Object-src if not restricted

SCORE: 0 = reflected+stored XSS found, 3 = DOM XSS only, 6 = filter bypass works, 8 = solid but CSP weak, 10 = full CSP + no XSS vectors
```

---

## PHASE 3: SQL/NOSQL INJECTION TESTING

> *"Every query that touches user input is a confession waiting to happen."*

```
1. SQL INJECTION (if SQL database):
   -> String-based:     ' OR '1'='1' --
   -> Numeric-based:    1 OR 1=1
   -> Union-based:      ' UNION SELECT null,username,password FROM users--
   -> Boolean blind:    ' AND 1=1-- vs ' AND 1=2--
   -> Time blind:       ' AND SLEEP(5)--
   -> Error-based:      ' AND extractvalue(1, concat(0x7e, version()))--
   -> Stacked queries:  '; DROP TABLE users;--
   -> Second-order:     Store payload, trigger on different query
   -> ORM bypass:       Raw query usage, string interpolation in queries

2. NOSQL INJECTION (MongoDB, Firestore, DynamoDB):
   -> Operator injection: {"$gt": ""} in login fields
   -> Regex injection:    {"$regex": ".*"}
   -> Where injection:    {"$where": "this.password == 'x'"}
   -> Array injection:    username[$ne]=invalid
   -> JSON injection in query parameters

3. QUERY ANALYSIS (static):
   -> Grep for raw SQL strings with concatenation/interpolation
   -> Grep for .rawQuery, .raw, db.execute with string templates
   -> Verify parameterized queries used everywhere
   -> Check ORM configurations for raw query escapes
   -> Identify any eval() or Function() with DB data

4. CONVEX-SPECIFIC (if applicable):
   -> Argument validation with v.string(), v.number() etc.
   -> Missing validators on mutation/action arguments
   -> Direct user input in query filters without validation
   -> Index usage preventing full table scans

SCORE: 0 = injectable endpoints found, 5 = parameterized but gaps, 8 = solid ORM usage, 10 = parameterized + WAF + input validation
```

---

## PHASE 4: CORS MISCONFIGURATION

> *"A permissive CORS policy is an open invitation to steal your users' data."*

```
1. CORS HEADER ANALYSIS
   -> Access-Control-Allow-Origin: * on authenticated endpoints? (CRITICAL)
   -> Origin reflection: does server echo back the Origin header?
   -> Null origin accepted? (file:// protocol, sandboxed iframe)
   -> Regex bypass: evil.com matching notevil.com?
   -> Subdomain wildcards: *.example.com when attacker controls sub?

2. CREDENTIAL HANDLING
   -> Access-Control-Allow-Credentials: true with wildcard origin?
   -> Cookies sent cross-origin without SameSite?
   -> Bearer tokens in requests to CORS-misconfigured endpoints?

3. PREFLIGHT ANALYSIS
   -> Access-Control-Allow-Methods: overly permissive (DELETE, PUT)?
   -> Access-Control-Allow-Headers: includes Authorization with weak origin?
   -> Access-Control-Max-Age: excessively long caching preflight?

4. CONFIGURATION AUDIT
   -> Framework CORS middleware configuration
   -> Per-route CORS overrides
   -> Proxy/CDN CORS header additions
   -> Verify CORS config matches intended trust boundaries

SCORE: 0 = wildcard with credentials, 3 = origin reflection, 5 = weak regex, 8 = proper allowlist, 10 = strict allowlist + SameSite
```

---

## PHASE 5: CSP HEADERS AUDIT

> *"No CSP means the browser trusts everything. Everything includes the attacker's script."*

```
1. CSP PRESENCE
   -> Content-Security-Policy header present?
   -> CSP meta tag in HTML?
   -> Report-Only mode vs enforcing mode?
   -> Different CSP for different routes?

2. DIRECTIVE ANALYSIS
   -> default-src: fallback restrictive? ('self' or 'none'?)
   -> script-src: 'unsafe-inline' present? (defeats XSS protection)
   -> script-src: 'unsafe-eval' present? (enables eval-based attacks)
   -> style-src: 'unsafe-inline' necessary or avoidable?
   -> img-src: overly permissive (data: *, blob:)?
   -> connect-src: restricts API calls to expected origins?
   -> frame-src/frame-ancestors: clickjacking protection?
   -> object-src: 'none' to prevent Flash/Java?
   -> base-uri: 'self' or 'none' to prevent base tag injection?
   -> form-action: restricts form submission targets?
   -> upgrade-insecure-requests: forces HTTPS?

3. BYPASS ANALYSIS
   -> JSONP endpoints within allowed script-src domains?
   -> CDN domains in script-src that host user content?
   -> 'strict-dynamic' properly used with nonces?
   -> Nonce generation: cryptographically random? Per-request?
   -> Hash-based CSP for inline scripts?

4. REPORTING
   -> report-uri or report-to configured?
   -> CSP violations being collected and analyzed?
   -> Report endpoint accessible and functional?

SCORE: 0 = no CSP, 3 = CSP with unsafe-inline, 5 = reasonable CSP with gaps, 8 = strong CSP, 10 = strict CSP with nonces + reporting
```

---

## PHASE 6: AUTHENTICATION BYPASS TESTING

> *"Authentication is the front door. Most apps leave the back door wide open."*

```
1. BYPASS TECHNIQUES
   -> Direct URL access to protected pages without auth cookie/token
   -> HTTP method switching: GET protected endpoint via POST/PUT/DELETE
   -> Parameter manipulation: remove auth params, change user_id
   -> Header manipulation: X-Forwarded-For, X-Original-URL, X-Rewrite-URL
   -> Path traversal: /admin/../admin (normalize vs raw path)
   -> Case sensitivity: /Admin vs /admin vs /ADMIN
   -> Trailing slash/dot: /admin/ vs /admin vs /admin.
   -> URL encoding: /%61dmin

2. CREDENTIAL SECURITY
   -> Password storage: bcrypt/argon2 with proper cost factor?
   -> Password reset: token entropy, expiry, single-use?
   -> Password change: requires current password?
   -> Account enumeration: login/signup/reset reveal user existence?
   -> Default credentials: admin/admin, test/test on any endpoint?

3. MULTI-FACTOR AUTHENTICATION
   -> MFA bypass: can you skip MFA step by direct URL?
   -> MFA code reuse: can the same code be used twice?
   -> MFA code brute force: rate limiting on code entry?
   -> Backup codes: secure generation and storage?

4. OAUTH/SSO ANALYSIS (if applicable)
   -> State parameter present and validated? (CSRF protection)
   -> Redirect URI validation: open redirect via OAuth?
   -> Token exchange: authorization code properly handled?
   -> Scope validation: requesting minimum necessary scopes?
   -> ID token validation: signature, issuer, audience, expiry?

5. CLERK/AUTH0/SUPABASE-SPECIFIC (if applicable)
   -> Webhook signature verification present?
   -> Client-side user object trusted without server verification?
   -> Middleware protecting all required routes?
   -> API routes checking auth independently of page auth?

SCORE: 0 = direct URL bypass works, 3 = method switching works, 5 = enumeration possible, 8 = solid with minor gaps, 10 = defense-in-depth auth
```

---

## PHASE 7: SESSION MANAGEMENT

> *"A stolen session IS the user. No password needed."*

```
1. SESSION TOKEN SECURITY
   -> Token entropy: cryptographically random? (>= 128 bits)
   -> Token in URL? (referer header leaks it)
   -> Token in localStorage? (XSS can steal it)
   -> HttpOnly flag on session cookies?
   -> Secure flag on session cookies? (HTTPS only)
   -> SameSite attribute? (Strict, Lax, or None?)
   -> Domain/Path scoping appropriate?

2. SESSION LIFECYCLE
   -> Session expiry configured? What duration?
   -> Idle timeout vs absolute timeout?
   -> Session invalidation on logout (server-side)?
   -> Session invalidation on password change?
   -> Concurrent session handling (limit? notify?)
   -> Session rotation after privilege escalation?

3. SESSION FIXATION
   -> New session ID generated after login?
   -> Pre-authentication session IDs rejected post-login?
   -> Session ID accepted from URL/POST params?

4. SESSION HIJACKING PROTECTION
   -> IP binding or user-agent binding?
   -> Session tokens transmitted only over HTTPS?
   -> Anti-CSRF tokens for state-changing operations?
   -> Double-submit cookie pattern or synchronizer tokens?

SCORE: 0 = session in URL/localStorage, 3 = missing HttpOnly/Secure, 5 = no expiry, 8 = solid session mgmt, 10 = all flags + rotation + CSRF
```

---

## PHASE 8: JWT SECURITY ANALYSIS

> *"JWTs are not sessions. They're signed claims. And the signature is only as strong as the secret."*

```
1. JWT STRUCTURE ANALYSIS
   -> Algorithm: HS256 (symmetric) vs RS256 (asymmetric)?
   -> Algorithm confusion: can you change RS256 to HS256? (CVE-2015-9235)
   -> Algorithm none: does server accept alg: "none"?
   -> Secret strength: JWT_SECRET longer than 256 bits?
   -> Secret rotation: mechanism for key rotation?

2. CLAIMS VALIDATION
   -> exp (expiry): present and enforced? How long? (target: < 15min for access)
   -> iat (issued at): present and validated?
   -> nbf (not before): used where applicable?
   -> iss (issuer): validated against expected value?
   -> aud (audience): validated against expected value?
   -> sub (subject): user ID type safe? (string vs number confusion)
   -> Custom claims: can user modify role/permissions claims?

3. TOKEN LIFECYCLE
   -> Access token lifespan: short enough? (< 15 min ideal)
   -> Refresh token: separate, longer-lived, stored securely?
   -> Refresh token rotation: new refresh token on each use?
   -> Token revocation: blacklist/whitelist mechanism?
   -> Logout: are tokens actually invalidated server-side?

4. STORAGE & TRANSMISSION
   -> Access token in httpOnly cookie vs Authorization header?
   -> Refresh token in httpOnly cookie? (never localStorage)
   -> Tokens in URL parameters? (referer leakage)
   -> Token size: not bloated with unnecessary claims?

5. COMMON ATTACKS
   -> JWT cracking: is the secret in common wordlists?
   -> JWT replay: same token valid across environments?
   -> JWT injection: manipulate claims without re-signing?
   -> Kid header injection: path traversal in key ID?
   -> JWK header injection: embed attacker's public key?

SCORE: 0 = alg none accepted, 2 = weak secret, 4 = no expiry, 6 = long-lived tokens, 8 = solid JWT, 10 = short-lived + rotation + revocation
```

---

## PHASE 9: IDOR DETECTION

> *"User 42 shouldn't see user 43's data. But the API just needs a different number."*

```
1. OBJECT REFERENCE ANALYSIS
   -> Sequential numeric IDs in API endpoints? (/api/users/1, /api/users/2)
   -> UUID vs sequential ID for user-facing resources?
   -> Guessable patterns in resource identifiers?

2. HORIZONTAL PRIVILEGE TESTING
   FOR EVERY data-access endpoint:
   -> Authenticate as User A
   -> Request User B's resources by changing ID parameter
   -> Check: /api/orders/{orderId} — can User A see User B's order?
   -> Check: /api/profile/{userId} — can User A edit User B's profile?
   -> Check: /api/files/{fileId} — can User A download User B's file?
   -> Check: /api/messages/{threadId} — can User A read User B's DMs?

3. VERTICAL PRIVILEGE TESTING
   -> Regular user accessing admin endpoints by ID manipulation
   -> Changing role parameter in profile update
   -> Accessing management functions by guessing endpoint IDs
   -> GraphQL node lookup bypassing authorization

4. INDIRECT REFERENCES
   -> File path manipulation: /api/files?path=../../etc/passwd
   -> Database reference manipulation: changing foreign keys
   -> Batch operations: /api/bulk?ids=[1,2,3,4,5] — which belong to you?
   -> GraphQL nested queries: accessing related objects you shouldn't see

5. CONVEX-SPECIFIC (if applicable)
   -> Document ID predictability
   -> Query filters checking ownership (ctx.auth)
   -> Mutations validating document ownership before update
   -> Missing authorization in internal functions called from actions

SCORE: 0 = horizontal IDOR on user data, 3 = IDOR on non-critical data, 5 = vertical IDOR, 8 = mostly protected, 10 = all endpoints authorization-checked
```

---

## PHASE 10: SSRF PROBING

> *"When the server makes requests on behalf of the user, the user becomes the server."*

```
1. SSRF VECTOR IDENTIFICATION
   -> URL parameters that trigger server-side fetches
   -> Image/file URL imports (avatar, OG image, webhook URL)
   -> PDF generators, screenshot services, link previewers
   -> Webhook delivery endpoints
   -> OAuth callback URLs
   -> Import/export functionality with URLs

2. INTERNAL NETWORK PROBING
   -> Can you reach localhost (127.0.0.1, 0.0.0.0)?
   -> Can you reach cloud metadata? (169.254.169.254, metadata.google.internal)
   -> Can you reach internal services? (redis://localhost, http://internal-api)
   -> DNS rebinding: external domain resolving to internal IP?

3. BYPASS TECHNIQUES
   -> IP address formats: decimal, octal, hex (0x7f000001)
   -> IPv6: [::1], [::ffff:127.0.0.1]
   -> URL parsing confusion: http://evil.com@127.0.0.1
   -> Redirect chains: external URL -> 302 -> internal
   -> DNS rebinding: first resolve to external, second to internal
   -> Protocol smuggling: gopher://, file://, dict://

4. CLOUD METADATA EXPLOITATION
   -> AWS: http://169.254.169.254/latest/meta-data/iam/security-credentials/
   -> GCP: http://metadata.google.internal/computeMetadata/v1/
   -> Azure: http://169.254.169.254/metadata/instance
   -> IMDSv2 enforcement check (AWS)

SCORE: 0 = metadata accessible, 3 = internal network reachable, 5 = partial SSRF, 8 = URL validated but bypassable, 10 = allowlist + no user-controlled URLs
```

---

## PHASE 11: OPEN REDIRECT TESTING

> *"A trusted domain redirecting to an attacker's site is phishing on a silver platter."*

```
1. REDIRECT PARAMETER DISCOVERY
   -> Scan for: redirect, return, returnUrl, next, url, continue, dest, redir, goto
   -> Login redirects: /login?next=/dashboard -> /login?next=http://evil.com
   -> Logout redirects: /logout?redirect=http://evil.com
   -> OAuth redirects: /auth/callback?redirect_uri=http://evil.com

2. BYPASS TECHNIQUES
   -> Protocol-relative: //evil.com
   -> Backslash: /\evil.com (server normalizes differently than browser)
   -> At sign: http://legit.com@evil.com
   -> Double encoding: %252F%252Fevil.com
   -> Null byte: http://legit.com%00.evil.com
   -> Tab/newline: http://legit.com%09.evil.com
   -> Data URI: data:text/html,<script>alert(1)</script>
   -> JavaScript URI: javascript:alert(1)
   -> Fragment: http://legit.com#@evil.com
   -> Path: /redirect/http://evil.com
   -> Subdomain: http://evil.legit.com (if DNS controlled)

3. IMPACT ASSESSMENT
   -> Can redirect steal OAuth tokens/codes?
   -> Can redirect be chained with XSS?
   -> Does redirect preserve authentication state?
   -> Can redirect serve malicious downloads?

SCORE: 0 = open redirect on auth pages, 3 = open redirect on non-auth, 5 = bypassable validation, 8 = strict validation, 10 = allowlist only + no user-controlled redirects
```

---

## PHASE 12: FILE UPLOAD VULNERABILITIES

> *"Every file upload is a potential web shell if you trust the Content-Type."*

```
1. FILE TYPE VALIDATION
   -> Extension check: server-side or client-side only?
   -> Extension bypass: .php.jpg, .pHp, .php%00.jpg, .php;.jpg
   -> MIME type check: Content-Type header vs magic bytes?
   -> Magic bytes validation: does server check file content?
   -> Double extension: file.php.jpg served as PHP?

2. FILE STORAGE SECURITY
   -> Uploaded files in web-accessible directory?
   -> Uploaded files served with original filename?
   -> Uploaded files served with original Content-Type?
   -> Path traversal in filename: ../../etc/cron.d/backdoor
   -> Overwrite protection: can upload overwrite existing files?

3. FILE CONTENT ATTACKS
   -> SVG with embedded JavaScript (XSS via SVG)
   -> HTML file upload (stored XSS)
   -> XML file with XXE payload
   -> ZIP bomb / decompression bomb
   -> Polyglot files (GIFAR: GIF + JAR)
   -> ImageTragick: malicious image exploiting ImageMagick

4. SIZE AND RATE LIMITS
   -> Maximum file size enforced server-side?
   -> Maximum number of concurrent uploads?
   -> Storage quota per user?
   -> Virus/malware scanning on uploads?

SCORE: 0 = web shell uploadable, 3 = XSS via SVG, 5 = path traversal, 8 = solid validation, 10 = content validation + isolated storage + scanning
```

---

## PHASE 13: RATE LIMITING VERIFICATION

> *"Without rate limiting, every endpoint is a DoS waiting to happen."*

```
1. CRITICAL ENDPOINTS
   -> Login: rate limited? After how many attempts?
   -> Registration: rate limited? (mass account creation)
   -> Password reset: rate limited? (email bombing)
   -> API endpoints: per-user and per-IP limits?
   -> Payment operations: rate limited? (double-charge prevention)
   -> Search/query: rate limited? (resource exhaustion)

2. RATE LIMIT IMPLEMENTATION
   -> Where enforced: application, reverse proxy, WAF, CDN?
   -> Limit granularity: per IP? Per user? Per endpoint?
   -> Reset mechanism: sliding window? Fixed window? Token bucket?
   -> Response on limit: 429 with Retry-After header?

3. BYPASS TECHNIQUES
   -> IP rotation via X-Forwarded-For header manipulation
   -> Distributed requests from multiple IPs
   -> API key rotation (if key-based limiting)
   -> Different endpoints same function (/api/v1/login vs /api/v2/login)
   -> Case variation in endpoint paths
   -> HTTP method variation (GET vs POST same endpoint)

4. RESOURCE EXHAUSTION
   -> Regex DoS (ReDoS): catastrophic backtracking in input validation?
   -> GraphQL complexity: deeply nested queries without depth limit?
   -> Pagination: requesting page_size=999999?
   -> Batch operations without batch size limit?
   -> WebSocket message flooding protection?

SCORE: 0 = no rate limiting on login, 3 = login limited but API not, 5 = basic limits but bypassable, 8 = comprehensive limits, 10 = WAF + app-level + monitoring
```

---

## PHASE 14: BRUTE FORCE PROTECTION

> *"Given enough attempts, every password is discoverable. The question is how many attempts you allow."*

```
1. LOGIN PROTECTION
   -> Account lockout after N failures? (what N?)
   -> Lockout duration: temporary or permanent until reset?
   -> Lockout scope: per IP, per account, or both?
   -> CAPTCHA after N failures?
   -> Progressive delays (exponential backoff)?
   -> Notification to user on failed login attempts?

2. TOKEN/CODE BRUTE FORCE
   -> Password reset token: length, charset, entropy?
   -> MFA code: rate limited? Lockout after failures?
   -> API key: length and complexity?
   -> Invitation codes: guessable patterns?
   -> Short codes (6-digit): protected against enumeration?

3. ENUMERATION PROTECTION
   -> Username enumeration via login error messages?
   -> Email enumeration via registration ("email already taken")?
   -> Phone enumeration via SMS verification?
   -> Timing attacks: different response times for valid vs invalid users?

4. CREDENTIAL STUFFING DEFENSE
   -> Compromised password database check (HIBP API)?
   -> Device fingerprinting for anomalous logins?
   -> Geolocation-based alerts for new locations?
   -> Common password list blocking?

SCORE: 0 = no lockout/no CAPTCHA, 3 = lockout but enumerable, 5 = basic protection, 8 = comprehensive, 10 = lockout + CAPTCHA + anomaly detection + HIBP
```

---

## PHASE 15: SECRETS SCANNING

> *"The most common vulnerability isn't code — it's the API key committed three months ago."*

```
1. ENVIRONMENT VARIABLES
   -> .env file in repository? .env.local? .env.production?
   -> .env in .gitignore? (verify it's actually ignored)
   -> Secrets in docker-compose.yml or Dockerfile?
   -> Secrets in CI/CD configuration (GitHub Actions, Vercel)?
   -> Environment variables logged or exposed in error messages?

2. GIT HISTORY ANALYSIS
   -> Run: git log --all --diff-filter=D -- '*.env*'
   -> Run: git log -p --all -S 'API_KEY\|SECRET\|TOKEN\|PASSWORD' -- '*.ts' '*.js' '*.py'
   -> Check: removed .env files still in git history
   -> Check: secrets rotated after accidental commit?
   -> Check: force push used to "hide" secrets (still in reflog)?

3. CLIENT-SIDE BUNDLES
   -> Grep built JS for: API_KEY, SECRET, TOKEN, PRIVATE, sk_, pk_
   -> Check NEXT_PUBLIC_ variables: only non-sensitive values?
   -> Source maps exposing server-side code in production?
   -> Hardcoded API keys in client-side JavaScript?
   -> Stripe publishable vs secret key confusion?

4. CONFIGURATION FILES
   -> Firebase config exposed (apiKey, authDomain, etc.)?
   -> AWS credentials in config files?
   -> Database connection strings with credentials?
   -> Third-party service keys (Twilio, SendGrid, etc.)?

5. HIGH-ENTROPY STRING DETECTION
   -> Scan for base64-encoded strings > 20 chars in source
   -> Scan for hex strings > 32 chars in source
   -> Scan for strings matching API key patterns:
      sk_live_, sk_test_, AKIA, AIza, ghp_, gho_,
      xoxb-, xoxp-, Bearer, Basic, AWS_ACCESS_KEY
   -> Use: trufflehog, gitleaks, or manual grep patterns

SCORE: 0 = active secrets in repo/bundles, 3 = rotated but still in history, 5 = .env committed but removed, 8 = clean repo, 10 = clean repo + secret manager + rotation
```

---

## PHASE 16: DEPENDENCY CVE AUDIT

> *"Your code is secure. Your dependencies are not. They're 90% of your attack surface."*

```
1. VULNERABILITY SCANNING
   -> Run: npm audit / yarn audit / pnpm audit
   -> Run: pip-audit / safety check (Python)
   -> Check: GitHub Dependabot alerts
   -> Analyze: critical, high, medium, low counts
   -> CRITICAL/HIGH: immediate remediation required

2. DEPENDENCY ANALYSIS
   -> Total dependency count (direct + transitive)
   -> Outdated dependencies: how many major versions behind?
   -> Unmaintained packages: last publish > 2 years ago?
   -> Deprecated packages still in use?
   -> Packages with known supply chain attacks?

3. LOCK FILE INTEGRITY
   -> Lock file (package-lock.json, yarn.lock) present and committed?
   -> Integrity hashes present in lock file?
   -> No manual edits to lock file?
   -> Lock file matches package.json?

4. SUPPLY CHAIN SECURITY
   -> Typosquatting risk: similar-named packages?
   -> Maintainer account compromises: recent ownership transfers?
   -> Install scripts: postinstall running arbitrary code?
   -> Package provenance: npm provenance or Sigstore signatures?
   -> Pin dependencies to exact versions in production?

5. SUBRESOURCE INTEGRITY
   -> CDN-loaded scripts have integrity attributes?
   -> SRI hashes match expected content?
   -> Fallback mechanism if CDN compromised?

SCORE: 0 = critical CVEs unfixed, 3 = high CVEs present, 5 = medium CVEs + outdated, 8 = all patched, 10 = all patched + SRI + provenance + minimal deps
```

---

## PHASE 17: SSL/TLS CONFIGURATION

> *"HTTPS is table stakes. The question is whether your TLS configuration is from 2024 or 2014."*

```
1. CERTIFICATE ANALYSIS
   -> Valid certificate? Not expired?
   -> Correct domain coverage (including www, subdomains)?
   -> Certificate chain complete? Intermediate certs present?
   -> Certificate transparency logged?
   -> HPKP or CAA records configured?

2. PROTOCOL CONFIGURATION
   -> TLS 1.2 minimum? (TLS 1.0/1.1 disabled?)
   -> TLS 1.3 supported and preferred?
   -> SSL 2.0/3.0 disabled? (POODLE, DROWN)
   -> Protocol negotiation: server preference or client preference?

3. CIPHER SUITE ANALYSIS
   -> Forward secrecy (ECDHE) enabled and preferred?
   -> Weak ciphers disabled (RC4, DES, 3DES, NULL)?
   -> AEAD ciphers preferred (GCM, ChaCha20-Poly1305)?
   -> RSA key exchange disabled (no forward secrecy)?
   -> Key size: >= 2048 bit RSA or >= 256 bit ECDSA?

4. MIXED CONTENT
   -> All resources loaded over HTTPS?
   -> No HTTP -> HTTPS redirects with sensitive data in URL?
   -> Upgrade-Insecure-Requests header set?
   -> All API calls over HTTPS?
   -> WebSocket using WSS not WS?

5. HSTS (HTTP Strict Transport Security)
   -> HSTS header present?
   -> max-age >= 31536000 (1 year)?
   -> includeSubDomains directive?
   -> preload directive? Submitted to HSTS preload list?

SCORE: 0 = TLS 1.0 enabled or expired cert, 3 = weak ciphers, 5 = no HSTS, 8 = strong TLS, 10 = TLS 1.3 + strong ciphers + HSTS preload + CAA
```

---

## PHASE 18: SECURITY HEADERS AUDIT

> *"Headers are your first line of defense. Most apps have none."*

```
FOR EVERY page and API endpoint:

1. X-Frame-Options / frame-ancestors
   -> Present? Set to DENY or SAMEORIGIN?
   -> frame-ancestors in CSP (modern replacement)?
   -> Clickjacking protection verified?

2. X-Content-Type-Options
   -> Set to nosniff?
   -> Prevents MIME type sniffing attacks?

3. X-XSS-Protection
   -> Legacy but still useful: 1; mode=block?
   -> Or removed in favor of CSP (both approaches valid)?

4. Referrer-Policy
   -> Set to strict-origin-when-cross-origin or no-referrer?
   -> Prevents token/sensitive data leakage in Referer header?

5. Permissions-Policy (formerly Feature-Policy)
   -> Camera, microphone, geolocation restricted?
   -> Payment, USB, Bluetooth restricted?
   -> Unnecessary browser features disabled?

6. Cache-Control on Sensitive Pages
   -> no-store, no-cache on pages with sensitive data?
   -> Prevents sensitive data cached by proxies/browsers?
   -> API responses with auth data: no-cache?

7. X-Permitted-Cross-Domain-Policies
   -> Set to none? (Flash/PDF cross-domain)

8. Cross-Origin Headers
   -> Cross-Origin-Embedder-Policy (COEP)?
   -> Cross-Origin-Opener-Policy (COOP)?
   -> Cross-Origin-Resource-Policy (CORP)?

HEADER CHECKLIST:
   [ ] Strict-Transport-Security
   [ ] Content-Security-Policy
   [ ] X-Frame-Options / frame-ancestors
   [ ] X-Content-Type-Options: nosniff
   [ ] Referrer-Policy
   [ ] Permissions-Policy
   [ ] Cache-Control (sensitive pages)
   [ ] COOP/COEP/CORP

SCORE: 0 = no security headers, 3 = some present, 5 = most present but weak, 8 = comprehensive, 10 = all headers + strict policies + COOP/COEP
```

---

## PHASE 19: API AUTHENTICATION VERIFICATION

> *"Every unauthenticated API endpoint is a data breach waiting to happen."*

```
1. ENDPOINT AUTHENTICATION MAP
   FOR EVERY API route:
   -> Public or authenticated?
   -> If public: SHOULD it be public? (data sensitivity check)
   -> If authenticated: which auth method? (session, JWT, API key)
   -> Auth check in middleware vs individual handler?
   -> Consistent auth across all HTTP methods for same endpoint?

2. API KEY SECURITY
   -> API keys transmitted in header (good) vs URL (bad)?
   -> API key rotation mechanism?
   -> API key scoping (read-only, write, admin)?
   -> API key per-client/per-user?
   -> Leaked API key revocation process?

3. GRAPHQL-SPECIFIC (if applicable)
   -> Introspection disabled in production?
   -> Query depth limiting?
   -> Query complexity analysis?
   -> Per-field authorization (not just per-query)?
   -> Batching attack protection?

4. WEBHOOK SECURITY
   -> Webhook signature verification (HMAC)?
   -> Replay protection (timestamp validation)?
   -> Source IP verification?
   -> Webhook secrets properly stored?

5. INTERNAL API SECURITY
   -> Internal endpoints exposed externally?
   -> Service-to-service authentication present?
   -> API versioning: old insecure versions still accessible?
   -> Documentation/Swagger exposed in production?
   -> Debug endpoints (/__debug, /graphql playground) disabled?

SCORE: 0 = unauthenticated data endpoints, 3 = inconsistent auth, 5 = auth present but gaps, 8 = solid API auth, 10 = auth + scoping + rotation + webhook verification
```

---

## PHASE 20: INPUT VALIDATION COMPLETENESS

> *"Trust nothing. Validate everything. On every layer."*

```
1. CLIENT-SIDE VALIDATION
   -> Form validation present? (type, length, pattern)
   -> Client-side validation can be bypassed (always test server-side too)
   -> Error messages specific enough to help users but not attackers?

2. SERVER-SIDE VALIDATION
   FOR EVERY input field in EVERY endpoint:
   -> Type validation: string, number, boolean, array, object
   -> Length validation: min/max for strings, arrays
   -> Range validation: min/max for numbers, dates
   -> Format validation: email, URL, phone, date patterns
   -> Whitelist validation: enum values, allowed characters
   -> Encoding validation: UTF-8, no null bytes, no control characters

3. OUTPUT ENCODING
   -> HTML context: HTML entity encoding
   -> JavaScript context: JavaScript encoding
   -> URL context: URL encoding
   -> CSS context: CSS encoding
   -> SQL context: parameterized queries
   -> JSON context: proper serialization

4. BUSINESS LOGIC VALIDATION
   -> Negative values in price/quantity fields?
   -> Integer overflow in calculations?
   -> Date range logic: end before start?
   -> Permissions escalation via parameter manipulation?
   -> Race conditions in concurrent state changes?

5. FILE/CONTENT VALIDATION
   -> Uploaded file names sanitized?
   -> Content-Type validated against actual content?
   -> Maximum request body size enforced?
   -> Array/object nesting depth limited?
   -> Recursive data structure prevention (JSON bomb)?

6. SCHEMA VALIDATION
   -> Zod/Yup/Joi schema for every API input?
   -> Schema matches frontend form validation?
   -> Unknown/extra fields rejected (strict mode)?
   -> Nested object validation (not just top-level)?
   -> Convex argument validators on every mutation/action?

SCORE: 0 = no server validation, 3 = partial validation, 5 = validated but inconsistent, 8 = comprehensive validation, 10 = schema validation + output encoding + business logic
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

The SECURITY HINGE POINT of this audit is the locus of maximum risk/value. Compute or
read:

```bash
# If a hinge file already exists for this ticket, use it:
HINGE_FILE="$HOME/.omega/state/hinge-points-${TICKET_ID:-default}.json"

# Otherwise compute it on the fly:
${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh "$PROJECT_PATH" --audit=sec --user-need="$USER_NEED_QUOTE" \
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
  "evidence_gathered": "Manual repro on prod URL; screenshot at .sec/edge-evidence/F-007.png",
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
  "audit": "sec",
  "score": 100,
  "score_raw": "<raw>/<denominator>",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "sec",
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
  "evidence_summary_path": "$PROJECT_PATH/audits/.secaudit/evidence-summary.json",
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

## PHASE 21: VERDICT

Score each phase 0-10, weight by severity:

```
SCORING MATRIX (400 max):
  Phase  1  (OWASP Top 10)             x 3.0  = max 30
  Phase  2  (XSS Testing)              x 3.0  = max 30
  Phase  3  (Injection Testing)        x 3.0  = max 30
  Phase  4  (CORS Misconfiguration)    x 2.0  = max 20
  Phase  5  (CSP Headers)              x 2.0  = max 20
  Phase  6  (Auth Bypass)              x 3.0  = max 30
  Phase  7  (Session Management)       x 2.5  = max 25
  Phase  8  (JWT Security)             x 2.0  = max 20
  Phase  9  (IDOR Detection)           x 2.5  = max 25
  Phase 10  (SSRF Probing)             x 1.5  = max 15
  Phase 11  (Open Redirect)            x 1.0  = max 10
  Phase 12  (File Upload)              x 1.5  = max 15
  Phase 13  (Rate Limiting)            x 2.0  = max 20
  Phase 14  (Brute Force)              x 1.5  = max 15
  Phase 15  (Secrets Scanning)         x 3.0  = max 30
  Phase 16  (Dependency CVE)           x 2.0  = max 20
  Phase 17  (SSL/TLS)                  x 1.5  = max 15
  Phase 18  (Security Headers)         x 1.5  = max 15
  Phase 19  (API Auth)                 x 2.5  = max 25
  Phase 20  (Input Validation)         x 2.0  = max 20
                                       TOTAL  = max 400

NORMALIZE: score = (raw / 400) x 100

GRADE:
  90-100: S — Fortress. Defense-in-depth, zero known vectors, incident-ready.
  80-89:  A — Hardened. Minor gaps that require chained exploits to reach.
  70-79:  B — Solid. Some misconfigurations, no critical exposure.
  60-69:  C — Exposed. Exploitable gaps exist but require effort.
  50-59:  D — Vulnerable. Known attack vectors, data breach probable.
  <50:    F — Compromised. Active secrets exposed, auth bypassable, injection possible.
```

---

## PHASE 22: FIX PLAN (automatic)

```
Sort: CRITICAL -> HIGH -> MEDIUM -> LOW
Priority by exploitability:
  CRITICAL: remotely exploitable without authentication
  HIGH: exploitable with low-privilege authentication
  MEDIUM: requires specific conditions or chained exploits
  LOW: defense-in-depth improvement, no direct exploit path

Group by blast radius (one fix may close 5 vulnerabilities)
Dependency order (fix auth before fixing IDOR — auth fix may resolve both)
Generate fix tasks with file:line specificity
Save to audits/.secaudit/fix-plan.json + fix-plan.md
```

---

## PHASE 23: FIX EXECUTION (automatic)

```
Sequential per vulnerability group.

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
  → Log the failure in audits/.secaudit/fix-log.md with exact error
  → Mark as NEEDS_REVIEW (never retry same approach blindly)
  → Try alternative approach OR skip this fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (in priority order):
  a. Read the ENTIRE target file (full context)
  b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
  c. Document BEFORE state (current vulnerability, proof of exploit)
  d. Apply security fix
  e. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
  f. If all green → commit: security(secaudit): FIX-XXX description
  g. If any red → revert → log → mark NEEDS_REVIEW
  h. Document AFTER state (same exploit attempt, now blocked)
  i. Verify: no regression in functionality
  j. Verify: no new vulnerabilities introduced
  k. HIGH-RISK fixes (auth, encryption, RLS): require human confirmation
```

---

## PHASE 24: RE-AUDIT (automatic)

```
1. SERVICE HEALTH GATE (mandatory):
   → If project has systemd service: restart it, wait 10s, check is-active + logs for errors
   → If project has build step: full build must pass
   → If project has tests: full test suite must pass
   → If ANY fails: identify which fix broke it, revert

2. Re-run all FAILING phases. Compare before/after.
3. Loop until score >= 80 or remaining items are NEEDS_REVIEW.
4. Special attention: verify fixes don't introduce new attack vectors.
```

---

## PARALLEL EXECUTION STRATEGY

```
The 20 phases are grouped for maximum parallelism:

WAVE 1 (discovery -- sequential):
  Phase 0: Reconnaissance

WAVE 2 (code-level analysis -- parallel):
  Phase 1: OWASP Top 10        ]
  Phase 2: XSS Testing          ] Static analysis
  Phase 3: Injection Testing     ] of source code
  Phase 20: Input Validation     ]

WAVE 3 (configuration audit -- parallel):
  Phase 4: CORS               ]
  Phase 5: CSP                ] Header and config
  Phase 17: SSL/TLS           ] analysis agents
  Phase 18: Security Headers   ]

WAVE 4 (auth & session -- parallel):
  Phase 6: Auth Bypass         ]
  Phase 7: Session Management  ] Authentication
  Phase 8: JWT Security        ] flow analysis
  Phase 9: IDOR Detection      ]
  Phase 19: API Auth           ]

WAVE 5 (infrastructure & supply chain -- parallel):
  Phase 10: SSRF               ]
  Phase 11: Open Redirect      ]
  Phase 12: File Upload        ] System-level
  Phase 13: Rate Limiting      ] security agents
  Phase 14: Brute Force        ]
  Phase 15: Secrets Scanning   ]
  Phase 16: Dependency CVE     ]

5 parallel agents per wave = 20 phases in 5 sequential waves.
```

---

## CROSS-COMMAND BRIDGE

```
/secaudit finds code issues -> references /codeaudit findings
/secaudit finds input issues -> references /flowaudit validation
/secaudit finds header issues -> fixes them directly
/secaudit finds exposed secrets -> IMMEDIATELY flags for rotation

THE QUALITY ARSENAL:
  /codeaudit    -> Is the code SOLID?           (preventive)
  /flowaudit    -> Does the experience WORK?    (preventive)
  /uiuxaudit    -> Is the interface BEAUTIFUL?  (preventive)
  /debugaudit   -> What is BROKEN right now?    (detective)
  /featureaudit -> Is the product COMPLETE?     (strategic)
  /perfaudit    -> Is it FAST enough?           (detective)
  /secaudit     -> Is it SECURE?                (detective)

  Together: nothing escapes. Every dimension covered.
```

---

## LAWS

1. **Every input is an attack vector.** User data, URL params, headers, cookies, file uploads, WebSocket messages. Trust nothing.
2. **Authentication is not authorization.** Knowing WHO someone is doesn't determine WHAT they can access. Check both, always.
3. **Secrets have a half-life (Popper).** Every secret will eventually be exposed. Design for rotation, revocation, and blast radius containment.
4. **The client is the attacker's machine.** Client-side validation is UX, not security. Server-side validation is security. Never skip it.
5. **Defense in depth or single point of failure.** One layer of security is one layer an attacker needs to bypass. Stack them: WAF + middleware + handler + database.
6. **Log everything, alert on anomalies.** If you can't detect an attack, you can't respond. If you can't respond, you've already lost.
7. **The weakest link is the attack surface (Popper).** FALSIFY "we're secure" by finding the one endpoint, the one header, the one dependency that breaks the chain. The attacker will.

---

*"/secaudit v1 — Probe. Inject. Bypass. Enumerate. Every endpoint, every input, every header, every secret. /400."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.1 — 2026-04-14T10:35:36Z)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial thinking
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.secaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — Non-UI contexts: /secaudit runs on any target (web, API, binary, library). Phase scoping adjusts per target type.
- ✅ **Output contract verification** — emits `audits/.secaudit/verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `iterations.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `audits/.secaudit/discovery/` > 1h old, re-verify inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.secaudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` flags with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — Rate-Limit Safety + Self-Pentest + CVE Sync

**Rate-limit safety (per preamble §12):**
- Default max 10 req/s against target
- Override: `--rate-limit-override=<N>` requires explicit `--authorized` flag
- Abort on 3 consecutive 429 or 503 responses
- Log all requests to `audits/.secaudit/request-log.txt` (for forensics)

**Self-pentest gate:**
- If target path is `~/.claude/commands/`, `~/.claude/agents/`, or similar introspection of Claude infra → ABORT with "self-pentest not supported, use manual review + code audit"
- If target is the local filesystem root → ABORT
- Only pentest explicit external targets (URLs, APIs, binaries)

**CVE sync:**
- Pin to `cve-search` snapshot date; emit `cve_db_snapshot_date` in telemetry.json
- If snapshot > 30 days old, WARN user to update before relying on vulnerability findings
- Do NOT attempt live CVE DB queries (rate-limit risk, reliability)

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

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (secaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "secaudit",
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

**Gap 1: Self-pentest target list → RESOLVED**
```
ABORT targets (hardcoded, no override):
  - ~/.claude/ (any subpath)
  - ~/.omega/ (any subpath)
  - ~/.godmode/ (any subpath)
  - file:// URLs pointing to system areas
  - localhost:0 - 1023 (privileged ports, likely system services)
  - The VPS itself (72.61.197.216) unless --authorized-self flag + password
ABORT message:
  "Self-pentest not supported on Claude infrastructure. Use manual code audit + /codeaudit instead."
```

**Gap 2: CVE snapshot auto-check → RESOLVED**
```
At audit start:
  If a CVE snapshot is configured on the host, check "$OMEGA_DIR/data/cve-snapshot-date.txt"
  (expected format: YYYY-MM-DD); if absent, skip this check (live CVE lookup / `npm audit` /
  `osv-scanner` still apply).
  If age > 30 days: WARN in verdict.md:
    "⚠️ CVE database snapshot is {N} days old. Vulnerability findings may miss recent CVEs.
     Update with: cve-search sync"
  Emit in telemetry.json: cve_db_snapshot_date, cve_db_age_days
```

**Gap 3: Rate-limit log schema → RESOLVED**
`audits/.secaudit/request-log.txt` format:
```
<ISO8601>\t<METHOD>\t<URL>\t<STATUS>\t<RESPONSE_MS>\t<PAYLOAD_HASH>
```
Rotated at 100MB. Retained for 30 days for forensics.

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
/metaudit --focus arsenal --scope="secaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*secaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

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

> *"One probe lies, two confirm, three convict. Fan the attack out, then make every wound prove itself before it scores."*

This section governs **how /secaudit executes when run**. It does not add, remove, reorder, or
reweight any phase — Phases 0–24, the /400 scoring matrix, the grade bands, and the verdict format
above are unchanged. It replaces *linear, one-phase-at-a-time grinding* with a **fan-out → adversarially-verify → synthesize → loop-until-dry** execution model, driven by the **Workflow tool**. The
Gestalt-Popper doctrine, the SECURITY HINGE POINT, and the 7 Laws of Security Forensics all still
bind every track. This is the operational upgrade of the existing **PARALLEL EXECUTION STRATEGY**
section (the 5-wave plan): that section names the waves; this section names the *engine* and the
*verification contract* that runs them.

### 1. Decompose into INDEPENDENT parallel tracks (fan-out)

After **Phase 0 PROGRAMMATIC GATHER** and **Phase 0 RECONNAISSANCE** complete (these are sequential
and feed everyone — the attack surface map, auth-boundary map, `evidence-summary.json`, and the
SECURITY HINGE POINT must exist first), launch the remaining phases **concurrently** via the
Workflow tool instead of walking them in order. Phases are grouped into **file/surface-disjoint
tracks** so concurrent execution never double-probes the same target or stomps the same output
(honors R-SCOPE — one writer per `reports/*.md` file).

Concretely, dispatch these tracks in parallel (mirror of the 5 waves, now true fan-out):

| Track | Phases (unchanged) | Disjoint surface | Writes to |
|-------|--------------------|------------------|-----------|
| **T1 — Code-level injection/output** | 1 (OWASP), 2 (XSS), 3 (Injection), 20 (Input Validation) | Source code, query layer, templates | `reports/{owasp-top-10,xss-testing,injection-testing,input-validation}.md` |
| **T2 — Transport & header config** | 4 (CORS), 5 (CSP), 17 (SSL/TLS), 18 (Security Headers) | HTTP response headers, TLS handshake | `reports/{cors-audit,csp-audit,ssl-tls,security-headers}.md` |
| **T3 — Identity boundary (HINGE-WEIGHTED)** | 6 (Auth Bypass), 7 (Session), 8 (JWT), 9 (IDOR), 19 (API Auth) | Auth middleware, session store, token lifecycle, per-object authz | `reports/{auth-bypass,session-management,jwt-security,idor-detection,api-auth}.md` |
| **T4 — Infra, supply chain & abuse** | 10 (SSRF), 11 (Open Redirect), 12 (File Upload), 13 (Rate Limiting), 14 (Brute Force), 15 (Secrets), 16 (Dependency CVE) | Outbound fetchers, upload sinks, deps, secrets, throttles | `reports/{ssrf-probing,open-redirect,file-upload,rate-limiting,brute-force,secrets-scanning,dependency-cve}.md` |

Track **T3 carries the SECURITY HINGE POINT** in nearly every app (the auth/authorization boundary —
Phase 0 Step 4). It is dispatched with **10× scrutiny** per Phase 2.2 (5× falsification attempts,
3× edge-case hunts, mandatory read of the hinge file + all direct callers/callees) and gets the
deepest sub-fan-out. If the computed hinge lands elsewhere (e.g. a public SSRF-prone fetcher in T4),
shift the 10× weight to that track instead — the hinge is wherever maximum breach-leverage lives,
not a fixed track.

**Workflow tool usage:** one Workflow per track, all launched in a single fan-out so they run
concurrently. Each track is itself a worker that MAY sub-fan-out its own phases (e.g. T1 runs XSS
reflected/stored/DOM probes as parallel sub-lenses). **Honor preamble §12 rate-limit safety across the
whole fan-out, not per-track:** the global cap is 10 req/s against the target (a 4-track fan-out must
not become 40 req/s). Share a token-bucket / serialize the live-HTTP probing phases (T2 header probes,
T3 auth replay, T4 SSRF/brute-force) if they target the same host; purely-static tracks (grep/AST over
source) parallelize freely with no rate concern. Abort the whole fan-out on 3 consecutive 429/503
(preamble §12). Self-pentest ABORT targets (the v1.2 hardcoded list — `~/.claude/`, `~/.omega/`, the VPS)
are checked **before** any track launches.

### 2. ADVERSARIALLY VERIFY every finding (≥2-of-3 independent lenses)

A single probe firing is **an input, never the verdict** (R-VERIFY). Before ANY finding is allowed to
affect the score, it must survive **≥2-of-3 independent lenses**. A finding that does not reach 2-of-3
is **killed** (demoted to `info` with `falsified_at`, excluded from scored severity) — exactly the
Phase 2.1 `falsified` outcome. This *feeds* the existing Popper machinery (Phase 2.1
`falsifiable_tests[]`); it does not replace it — it requires that falsification be done from
**independent angles**, not one command.

The three lenses (security-tailored):

- **Lens A — REPRODUCE (exploit it):** independently re-trigger the vulnerability and capture the
  proof-of-exploit. For XSS/injection → fire the payload and observe execution/DB effect. For IDOR →
  authenticate as User A, fetch User B's object, confirm the leak. For a secret → confirm the key is
  live (or unambiguously a real credential pattern). For a CVE → confirm the vulnerable code path is
  actually *reachable/called* in this codebase (not just present in the dependency tree). Evidence:
  request/response, screenshot, or command output saved under `.sec/edge-evidence/` /
  `reports/<phase>.md`.
- **Lens B — REFUTE (Popper counter-example):** actively try to prove the finding is a false positive —
  the protection the tool missed. Is the XSS sink behind framework auto-escaping (React text node,
  not `dangerouslySetInnerHTML`)? Is the "unauthenticated endpoint" actually gated by middleware the
  static scan can't see (Phase 2.1 table)? Is the `npm audit HIGH` in a dev-only / unreachable path?
  Is the open redirect blocked by an allowlist one layer up? If Lens B finds the mitigation → the
  finding is **killed**.
- **Lens C — CROSS-CHECK (independent corroboration):** confirm via a *different evidence source* than
  Lens A. Read the route handler / middleware source to confirm the gap (static ↔ dynamic agreement);
  cross-reference a **sibling audit's `evidence-summary.json`** (Phase 0.5 — `/apiaudit` for the same
  auth surface, `/codeaudit` for the same `auth.ts:line`, `/dataaudit` for the same table); or confirm
  with a second tool in `evidence-summary.json` (`cross_tool_confirmed`). Two independent tools or
  audit↔code agreement on the same `file:line` = strong corroboration → bump severity one level and set
  `cross_audit_confirmed: true` (Phase 2.5).

**Acceptance rule (concrete):**
- **CRITICAL / HIGH severity → require all 3 lenses** (reproduce + refute-survived + cross-check). A
  remotely-exploitable-without-auth claim that cannot be reproduced is downgraded until it is.
- **MEDIUM / LOW severity → require ≥2-of-3.** Reproduce + cross-check is typical when live exploit is
  unsafe to fire against the target.
- Record the lens outcomes per finding so the scoring phase can see *why* it survived:

```jsonc
// extends each verdict.json findings[] entry — no schema field is removed
{
  "id": "F-013",
  "phase": 9,                          // IDOR — unchanged
  "severity": "critical",
  "verification_lenses": {
    "reproduce":   {"passed": true,  "evidence": ".sec/edge-evidence/F-013-userB-leak.txt"},
    "refute":      {"survived": true, "counterexample_attempted": "checked ctx.auth ownership guard in convex/orders.ts:88 — absent"},
    "cross_check": {"passed": true,  "source": "cross-audit:apiaudit F-021 same endpoint", "cross_audit_confirmed": true}
  },
  "lenses_passed": 3,
  "verdict_admissible": true           // false ⇒ killed, demoted to info, excluded from score
}
```

This is the direct realization of the Phase 2.1 `confirmed | falsified | inconclusive` outcomes:
`lenses_passed ≥ required` ⇒ `confirmed`; refute succeeds ⇒ `falsified` (killed);
can't run a lens cleanly ⇒ `inconclusive` (keep severity, `confidence: medium`).

### 3. Synthesize survivors into the EXISTING scoring matrix + verdict (unchanged)

Synthesis is the auditor's own job — **never paste a track's summary as the verdict** (R-ORCH). Once
every track reports and every finding has been adversarially verified:

1. **Admit only survivors.** Findings with `verdict_admissible: false` are excluded from scored
   severity (they remain logged as `info` for the record).
2. **Map survivors back to their owning phase.** Each surviving finding scores its phase 0–10 exactly
   per that phase's existing rubric line (e.g. Phase 2 XSS: "0 = reflected+stored XSS found … 10 = full
   CSP + no XSS vectors"; Phase 9 IDOR: "0 = horizontal IDOR on user data … 10 = all endpoints
   authorization-checked"). **No rubric line is altered.**
3. **Feed Phase 21 unchanged.** Apply the **/400 SCORING MATRIX exactly as written** (Phase 1 ×3.0 …
   Phase 20 ×2.0, TOTAL max 400), then `NORMALIZE: score = (raw / 400) × 100`, then the S/A/B/C/D/F
   grade bands — verbatim. The fan-out and the lenses change *how confidently each phase score was
   reached*, not *how it is computed*.
4. **Honor the Phase 2.7 score gate.** 100/100 stays blocked unless all critical/high findings are
   fixed or justified, load-bearing findings are Popper-confirmed, `user_need_match.addressed = true`,
   ≥3 falsifiable tests per phase, ≥2 edge cases per top-5 finding, cross-audit synthesis attempted.
   The verification lenses **supply** this evidence; they do not relax the gate.
5. **Then Phases 22 → 23 → 24 run unchanged:** fix plan, fix execution (with the DO-NO-HARM safety
   gate + the AUDIT-VERIFICATION-CONTRACT before/after baseline), and re-audit (bounded at 5
   iterations). All existing outputs (`verdict.json`, `verdict.md`, `fix-plan.json`, `before-after.md`,
   `telemetry.json`, …) and the output gate are produced exactly as before.

### 4. Loop-until-dry for unknown-size discovery

Attack surface is **not** a fixed list — every reproduced vuln and every new route/param/upload-sink
discovered mid-audit is a fresh surface to probe. Phases 2, 3, 6, 9, 12, 15, 19, 20 are inherently
**unknown-size** (you don't know how many injectable params, IDOR-able objects, exposed secrets, or
unauthenticated endpoints exist until you've found them all). Run those tracks as a **loop-until-dry**,
not a single pass:

```
queue ← all attack surfaces from Phase 0 (endpoints, inputs, upload sinks, deps, redirect params, secrets candidates)
seen  ← ∅
while queue not empty:
    surface ← queue.pop()
    if surface ∈ seen: continue
    seen.add(surface)
    probe surface across its owning track's phases (fan-out the payload set)
    for each candidate finding:
        run the ≥2-of-3 adversarial lenses (§2)
        if admissible:
            record; map to owning phase (§3)
            # discovery feedback — a confirmed breach often exposes more surface:
            #   IDOR on /orders/{id}      → enqueue sibling collections (/invoices, /files) for the SAME pattern
            #   reflected XSS in param p  → enqueue every sink that echoes p; re-test with filter-bypass payloads
            #   live secret found         → enqueue what it unlocks (its service's endpoints) as new surface
            #   SSRF reaches localhost    → enqueue internal services / cloud-metadata as new targets
            enqueue any newly-revealed surfaces
        else:
            log as info (killed by lens)
terminate when: queue empty (dry) AND last full pass surfaced 0 new admissible findings
```

Termination is **convergence to dry**, not a phase counter — but it is **bounded**: obey preamble §4
(re-audit loop ≤ 5 iterations) and **R-BUDGET** (default 500K-token mission cap — escalate, never
silently overrun). On hitting the cap with the queue non-empty, mark the remainder `NEEDS_REVIEW`,
emit `confidence: low`, surface as `pending` in `.done.json`, and send the Telegram SOS (preamble §7)
— **never** report a clean 100 over an un-drained attack-surface queue.

> **Net effect:** same 25 phases, same /400 matrix, same grade bands, same verdict + before/after
> contract — now executed as a rate-limited concurrent fan-out where every wound must survive 2-of-3
> independent lenses before it scores, and discovery loops until the attack surface is provably dry.
> Probe. Inject. Bypass. Enumerate — *in parallel, and only believe what survives the cross-examination.*
