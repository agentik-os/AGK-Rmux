---
name: depaudit
description: >
  Forensic dependency & supply-chain audit v1 (Gestalt-Popper). 18-phase deep analysis of everything
  the project TRUSTS from third parties: dependency CVE exposure (direct + transitive), outdated and
  abandoned packages, license compliance and contamination, lockfile integrity and reproducible builds,
  transitive dependency bloat and duplication, typosquatting / dependency-confusion / namespace-takeover
  risk, postinstall and lifecycle script auditing, pinned vs floating version policy, SBOM generation
  and completeness, registry trust and provenance, bundle exposure (server deps leaking to client),
  monorepo workspace hygiene, plus verdict, fix plan, fix execution, re-audit, and build-integrity
  safety gate. Answers "Is the supply chain SAFE?" Score /360. Preamble v1.0 compliant.
  Complements /secaudit (which owns RUNTIME exploitation of CVEs) — depaudit owns STATIC supply-chain
  hygiene, provenance, licensing, and reproducibility. Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/depaudit", "dependency audit", "supply chain audit", "audit dependencies",
  "is the supply chain safe", "are my packages safe", "outdated packages", "abandoned dependencies",
  "license audit", "license compliance", "lockfile integrity", "reproducible build", "typosquatting",
  "dependency confusion", "postinstall scripts", "SBOM", "software bill of materials", "audit deps",
  "package audit", "vendor audit", "third-party audit".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: dependencies
phases: 18
max_score: 360
read_only: false
triggers: ["dep", "depaudit", "dependency audit", "supply chain", "audit dependencies", "license audit", "lockfile integrity", "sbom", "typosquatting", "postinstall scripts", "outdated packages"]
---


<!-- AUDIT-META-V2-INJECTED -->

> ## ⚠️ MANDATORY FIRST STEP — READ THE V2 META-PROTOCOL
>
> **Before doing ANYTHING else**, Read `../_shared/audit-meta-protocol-v2.md`,
> then `../_shared/QUALITY-ARSENAL-PREAMBLE.md`, then
> `../_shared/AUDIT-VERIFICATION-CONTRACT.md`. (Relative paths — these are
> vendored next to this skill so a blank-VPS clone resolves them; never read
> `~/.claude/...` copies.)
>
> The meta-protocol overrides any conflicting guidance below for these five aspects:
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

# /depaudit v1 — Forensic Supply-Chain Interrogation (Gestalt-Popper)

> *"You did not write 95% of your code. Someone you've never met did. This audit asks: who, and can you trust them?"*

---

## DOCTRINE

You are not a `npm audit` wrapper. You are a **supply-chain forensic investigator**. The dependency tree is a chain of trust that runs from your `package.json` through hundreds of strangers' packages, their maintainers, their CI tokens, and the registries that host them. Any single link can be compromised, abandoned, typosquatted, or silently swapped. Your job: trace every link, and prove which ones will break.

**The 7 Laws of Supply-Chain Forensics (Gestalt-Popper Synthesis):**
1. **Your code is the minority.** A typical Node project ships 5% first-party code and 95% transitive dependencies. The attack surface is the 95% you didn't read. Audit accordingly.
2. **Every pinned version is a snapshot of trust at one moment (Popper).** `^1.2.3` means "I trust this maintainer's future judgment." A floating range is a standing authorization for code you haven't seen yet. FALSIFY the claim "we pin our deps."
3. **Absence of a lockfile is non-determinism, and non-determinism is a vulnerability.** Two installs producing two trees means the audit you ran yesterday describes code you aren't running today.
4. **Clarity before scanning (Gestalt).** Before any tool, UNDERSTAND the project: runtime (Node/Bun/Python/Rust/Go), package manager, monorepo or single, what ships to the client vs the server. Identify the **SUPPLY-CHAIN HINGE POINT** — the single dependency whose compromise grants the widest blast radius (the deepest-imported, highest-privilege, most-transitively-relied-upon package). Audit it with 10x depth.
5. **A green `npm audit` is not a clean supply chain (Popper).** `npm audit` finds *known* CVEs. It does not find abandoned packages, typosquats, license bombs, malicious postinstall scripts, or unpinned drift. FALSIFY "we ran npm audit, we're fine."
6. **A dependency is a person, not a string.** Behind every package is a maintainer account that can be phished, a 2FA that may be off, a publish token that may leak. Bus factor 1 + no 2FA + high download count = the next `event-stream`.
7. **Reproducible or it didn't happen.** If `git clone && install` on a fresh machine produces a different tree than CI, your SBOM is fiction and your CVE scan describes a phantom. Prove reproducibility or treat every other finding as provisional.

**Gestalt Supply-Chain Hinge Point:** Before Phase 1, compute the one dependency that maximizes `(transitive_dependents × privilege × install_script_presence)`. This is the package that — if hijacked — owns your build, your CI secrets, or your runtime. It gets every phase at maximum depth.

**Popper Supply-Chain Falsification Categories:**
- **CLAIM vs REALITY** — `package.json` says `react@^18` but the lockfile resolves `18.0.0-canary`
- **MANIFEST vs INSTALLED** — a package is `require`d in code but absent from any manifest (ghost dep)
- **PINNED vs RESOLVED** — the range is pinned but the lockfile drifted via a manual edit
- **DECLARED vs LICENSED** — `license: "MIT"` in `package.json` but the bundled source is GPL
- **AUDITED vs EXPLOITABLE** — `npm audit` reports HIGH, but the vulnerable code path is never reached (and vice-versa: a path IS reached that audit missed)

---

## RELATIONSHIP TO /secaudit (no overlap, explicit handoff)

| Concern | Owner |
|---|---|
| Is a CVE *reachable / exploitable at runtime*? | `/secaudit` Phase 16 (runtime exploitation) |
| Is a CVE *present in the resolved tree*, what severity, what fix path? | **`/depaudit`** (Phase 1) |
| Is a package abandoned / unmaintained / single-maintainer? | **`/depaudit`** (Phase 3) |
| License compatibility & contamination | **`/depaudit`** (Phase 4) |
| Lockfile integrity & reproducible build | **`/depaudit`** (Phase 5) |
| Secrets committed in the repo | `/secaudit` (Phase 15) |
| Postinstall scripts running arbitrary code | **`/depaudit`** (Phase 8) |
| SBOM generation & completeness | **`/depaudit`** (Phase 16) |

When a finding lands in both domains (e.g. a CVE that depaudit confirms present AND secaudit confirms reachable), mark it `cross_audit_confirmed: true` and bump severity one level (see Phase H1.5).

---

## SCOPE DETECTION (automatic from user prompt)

```
EXAMPLES:
  "/depaudit"
  → ALL manifests + lockfiles. Full 18-phase pipeline across every package ecosystem present.

  "/depaudit licenses"
  → LICENSE-FOCUSED: Phase 4 at max depth (compatibility matrix, contamination, attribution).

  "/depaudit are my packages outdated"
  → FRESHNESS-FOCUSED: Phase 2 (CVE) + Phase 3 (outdated/abandoned) deep, others light.

  "/depaudit lockfile / reproducible build"
  → INTEGRITY-FOCUSED: Phase 5 (lockfile) + Phase 6 (transitive resolution) deep.

  "/depaudit typosquatting / supply chain attack"
  → TRUST-FOCUSED: Phase 7 (typosquat/confusion) + Phase 8 (install scripts) + Phase 11 (provenance) deep.

  "/depaudit generate an SBOM"
  → SBOM-FOCUSED: Phase 16 produces a complete CycloneDX/SPDX SBOM as primary deliverable.

  "/depaudit frontend bundle"
  → BUNDLE-FOCUSED: Phase 12 (client bundle exposure) deep — what server-only deps leaked client-side.

RULES:
- Specific ecosystem mentioned (npm/cargo/pip/go) → scope to that manifest set
- Specific concern described → run the matching phase(s) at MAX depth, others at proportional depth (never skip — rule 46)
- "all" / "everything" / "full" → all phases, all ecosystems
- If audits/.depaudit/fix-plan.json exists and no new scope → resume fixing
- Parse intent, do not ask for clarification (Third Law — decide and proceed)
```

---

## OUTPUT CONTRACT — Omega Integration

```
audits/.depaudit/
├── session.log
├── discovery/
│   ├── ecosystems.json          # detected package managers + manifest/lockfile paths
│   ├── dependency-tree.json      # full resolved tree (direct + transitive) per ecosystem
│   ├── manifest-vs-installed.json# ghost deps + phantom deps reconciliation
│   └── hinge-package.json         # the supply-chain hinge dependency + blast radius
├── reports/
│   ├── cve-exposure.md            # Phase 1
│   ├── freshness.md               # Phase 2
│   ├── abandonment.md             # Phase 3
│   ├── licenses.md                # Phase 4
│   ├── lockfile-integrity.md      # Phase 5
│   ├── transitive-resolution.md   # Phase 6
│   ├── typosquat-confusion.md     # Phase 7
│   ├── install-scripts.md         # Phase 8
│   ├── version-policy.md          # Phase 9
│   ├── bloat-duplication.md       # Phase 10
│   ├── provenance-trust.md        # Phase 11
│   ├── bundle-exposure.md         # Phase 12
│   ├── monorepo-hygiene.md        # Phase 13
│   ├── deprecations.md            # Phase 14
│   └── reproducibility.md         # Phase 15
├── sbom/
│   ├── sbom.cyclonedx.json        # Phase 16 — machine-readable SBOM (CycloneDX)
│   └── sbom.spdx.json             # Phase 16 — SPDX alternative
├── verdict.json
├── verdict.md
├── fix-plan.json
├── fix-plan.md
├── progress.json
├── before-after.md
└── fix-log.md
```

**CRITICAL:** `progress.json` is read by the Telegram bot monitor for live progress cards.
Format: `{"total": 41, "done": 9, "failed": 1, "skipped": 0, "remaining": 31, "current": "FIX-010 — bump lodash 4.17.11→4.17.21"}`

**CRITICAL:** `fix-plan.json` is read by oracles to resume interrupted audits.
Format: `{"tasks": [{"id": "FIX-001", "finding": "...", "package": "lodash", "ecosystem": "npm", "from": "4.17.11", "to": "4.17.21", "fix": "...", "status": "pending|done|failed|skipped", "severity": "CRITICAL|HIGH|MEDIUM|LOW"}]}`

---

## PHASE 0 — PROGRAMMATIC GATHER (HYBRID, runs FIRST, before all other phases)

> **Hybrid framework (2026-05-08):** before any LLM analysis, deterministic tools gather every
> machine-checkable finding. The LLM then READS the resulting JSON instead of hand-running scanners.
> Freed token budget is REINVESTED in Popper falsification, hinge synthesis, trust analysis, and
> edge-case hunting.

### 0.1 Run the gather script (mandatory, FIRST step)

```bash
~/.omega/lib/audit-runner.sh dep "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes the dependency gather, which (per detected ecosystem) runs:
`npm audit --json` / `pnpm audit` / `yarn npm audit`, `osv-scanner` (cross-ecosystem CVE), `npm outdated --json`,
`depcheck` (unused + missing/ghost deps), `license-checker` / `license-checker-rseidelsohn` (npm) + `cargo-deny`/`cargo-license` (Rust) + `pip-licenses` (Python),
`npm ls --all` / `cargo tree` / `pipdeptree` (full resolved tree), lockfile presence + manifest-drift check,
postinstall/lifecycle-script extractor, `cyclonedx`/`syft` (SBOM), `pip-audit` (Python), `cargo-audit` (Rust), `govulncheck` (Go).

Output is written to:

```
$PROJECT_PATH/audits/.depaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.depaudit/` for cross-audit reference (see 0.5).

> The canonical runner accepts every registry audit. If no dependency-specific
> gatherer is installed, it emits an explicit `llm-only` evidence envelope at
> `$PROJECT_PATH/audits/.depaudit/evidence-summary.json`; never write a second root.

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "dep",
  "tools_run": ["npm-audit", "osv-scanner", "depcheck", "license-checker", "..."],
  "tools_skipped": [{"tool": "...", "reason": "..."}],
  "ecosystems": ["npm", "cargo", "pip"],
  "findings_total": 312,
  "findings_by_severity": {"critical": 1, "high": 9, "medium": 44, "low": 258, "info": 0},
  "findings": [
    {
      "tool": "osv-scanner",
      "severity": "critical|high|medium|low|info",
      "ecosystem": "npm",
      "package": "lodash",
      "installed_version": "4.17.11",
      "advisory": "GHSA-jf85-cpcp-j695 / CVE-2019-10744",
      "fixed_in": "4.17.12",
      "dependency_path": "myapp > some-lib > lodash",
      "message": "...",
      "suggested_fix": "...",
      "cross_tool_confirmed": false
    }
  ],
  "metrics": { "direct_deps": 41, "transitive_deps": 1183, "lockfile_present": true, "duplicate_packages": 7 },
  "evidence_index": { /* paths to raw/ files for drill-down */ }
}
```

### 0.3 What you do AFTER the gather (this replaces hand-running scanners)

1. **Read `evidence-summary.json` in full** — your evidence base.
2. **Read the manifests + lockfiles directly** (`package.json`, `package-lock.json`/`pnpm-lock.yaml`/`yarn.lock`, `Cargo.toml`/`Cargo.lock`, `pyproject.toml`/`requirements*.txt`/`poetry.lock`, `go.mod`/`go.sum`) — these are short and load-bearing.
3. **Read the hinge package's actual source** when its install scripts or privilege warrant it (Phase 8, 11).
4. **DO NOT re-run** the scanners the gather already ran (see 0.4).

### 0.4 Banned operations after Phase 0

If you catch yourself about to run one of these, STOP and read `evidence-summary.json` first:

- ❌ `npm audit` / `pip-audit` / `cargo audit` / `osv-scanner` (the gather ran them — read the JSON)
- ❌ `npm outdated` (already captured)
- ❌ `license-checker` / `cargo-license` (already captured)
- ❌ Generic "let me list every dependency" loops (the tree is in `discovery/dependency-tree.json`)

You MAY still:
- ✅ Read SPECIFIC manifests/lockfiles to verify a drift or pin claim
- ✅ Run a SPECIFIC query to FALSIFY a finding (Popper test, e.g. `npm ls <pkg>` to confirm a dependency path, `npm view <pkg> time.modified` to confirm abandonment)
- ✅ Run a tool the gather couldn't (e.g. `npm view <pkg> maintainers` for provenance, a registry metadata fetch)

### 0.5 Cross-audit synthesis (read sibling evidence-summary.json files)

If this audit runs in a Linear-fix mission, sibling summaries are at
`$PROJECT_PATH/audits/.linear-fix/<TICKET>/.<other-audit-id>/evidence-summary.json`. Read them.

High-value confluences:
- **depaudit + secaudit** flag the same package → depaudit confirms it's *present*, secaudit confirms it's *exploitable* → CRITICAL, joint fix (the version bump closes both).
- **depaudit + codeaudit** both flag an unused/ghost dependency → confirmed dead weight, safe to remove.
- **depaudit + perfaudit** flag the same heavy client-bundle dependency → joint fix (remove/replace + tree-shake).

Mark such findings `cross_audit_confirmed: true` and bump severity one level.

---

## PHASE 0b: SUPPLY-CHAIN CRIME SCENE SETUP

```bash
SESSION_ID="depaudit-$(date +%Y%m%d-%H%M%S)"
mkdir -p audits/.depaudit/{discovery,reports,sbom}
echo "AUDIT STARTED: $(date -Iseconds)" > audits/.depaudit/session.log

# ECOSYSTEM CENSUS — know exactly which supply chains exist
# (read-only discovery; the gather already populated the tree, this confirms the manifests)
for mf in package.json pnpm-workspace.yaml Cargo.toml pyproject.toml requirements.txt \
          go.mod Gemfile composer.json; do
  [ -f "$mf" ] && echo "MANIFEST: $mf" >> audits/.depaudit/session.log
done
for lf in package-lock.json pnpm-lock.yaml yarn.lock Cargo.lock poetry.lock go.sum; do
  [ -f "$lf" ] && echo "LOCKFILE: $lf" >> audits/.depaudit/session.log
done
```

Then read `discovery/dependency-tree.json` and compute the **hinge package** into
`discovery/hinge-package.json` (deepest-imported × privilege × has-install-script).

---

## PHASE 1: CVE EXPOSURE FORENSICS

> *"A known vulnerability is a published exploit with your name on the affected list."*

```
1. ADVISORY RECONCILIATION (from evidence-summary.json, cross-ecosystem)
   For each CVE/GHSA/RUSTSEC/PYSEC advisory:
   → Which package, which installed version, which advisory ID?
   → Is it DIRECT or TRANSITIVE? (transitive = you can't just `npm i pkg@latest`)
   → What is the dependency PATH? (myapp > lib-a > lib-b > vulnerable-pkg)
   → Is a fixed version available? Is it a major bump (breaking)?
   → Severity (CVSS) + exploit maturity (PoC public? in-the-wild?)

2. FIX REACHABILITY
   → Direct dep → bump in manifest
   → Transitive via a maintained parent → bump the parent
   → Transitive via an abandoned parent → override/resolution pin, or replace the parent
   → No fix available → record as accepted-risk with justification + mitigation

3. CROSS-TOOL CONFIRMATION
   → Does more than one scanner (npm audit + osv-scanner) flag the same advisory?
   → Agreement = confidence high; disagreement = investigate (version range edge)

4. DEDUPLICATION
   → The same CVE may appear via N dependency paths. Count unique advisories, not raw rows.
```

**FALSIFY:** Don't trust the count. For each top advisory, run `npm ls <pkg>` (or `cargo tree -i <pkg>`) to PROVE the version is actually installed and find every path.

**Severity:** Critical/High advisory with a fix available and direct = CRITICAL. No fix + reachable = HIGH (accepted risk requires written justification).

---

## PHASE 2: FRESHNESS & DRIFT

> *"Outdated isn't just old. It's the distance between the code you trust and the code that's been audited since."*

```
1. VERSION LAG (from npm outdated / equivalent)
   For each direct dependency:
   → current vs wanted vs latest
   → How many MAJOR versions behind? (>2 major = upgrade debt, likely breaking)
   → How many MINOR/PATCH behind? (patch lag = missing security fixes)

2. RELEASE CADENCE
   → When was the installed version published? (npm view <pkg> time)
   → When was the latest published? Gap = how stale is your pin?
   → Is the project keeping pace with its own dependencies' release rhythm?

3. UPGRADE BLAST RADIUS
   → For each behind-by-major dep: what's the migration cost? (changelog severity)
   → Group: trivial patch bumps (do now) vs major migrations (plan separately)

4. SECURITY-RELEVANT LAG
   → Cross-reference Phase 1: is any outdated dep also CVE-affected?
   → A patch behind that closes a CVE = HIGH priority, not cosmetic.
```

---

## PHASE 3: ABANDONMENT & MAINTENANCE RISK

> *"An unmaintained dependency is a CVE that hasn't been written yet, with nobody to fix it when it is."*

```
1. ACTIVITY SIGNALS (per direct dep + the hinge package)
   → Last publish date (>18 months with no release = abandonment risk)
   → Last commit / last release on the source repo
   → Open-issue count vs response rate (issues pile up, no replies = dead)
   → Is it ARCHIVED on GitHub? Explicitly deprecated on the registry?

2. BUS FACTOR
   → Maintainer count (npm view <pkg> maintainers / crates.io owners)
   → Bus factor 1 = single point of human failure (illness, account loss, malice)
   → High download count + bus factor 1 + no recent release = the next incident

3. SUCCESSOR / FORK STATUS
   → Is there an official successor? (e.g. request → got/axios/undici, moment → dayjs/luxon)
   → Is the community fork the de-facto maintained version?
   → Is the dep deprecated-with-replacement on the registry?

4. CRITICALITY WEIGHTING
   → Abandoned leaf util used in one place = MEDIUM
   → Abandoned package on the hinge path = HIGH (its blast radius is the whole build/runtime)
```

**FALSIFY:** Before declaring "abandoned", check `npm view <pkg> time.modified` and the repo — a stable package with no recent release may be *complete*, not *dead*. Distinguish "finished" from "neglected" (does it still have unpatched CVEs? unanswered security issues? → neglected).

---

## PHASE 4: LICENSE COMPLIANCE & CONTAMINATION

> *"A copyleft license deep in your transitive tree can legally compel you to open-source your proprietary product."*

```
1. LICENSE CENSUS (from license-checker / cargo-license / pip-licenses)
   → Build the full license distribution: {MIT: 800, Apache-2.0: 200, ISC: 90, GPL-3.0: 1, ...}
   → Flag every COPYLEFT license: GPL, AGPL, LGPL, MPL, EUPL, CDDL, EPL
   → Flag every UNKNOWN / UNLICENSED / "SEE LICENSE IN" / missing license

2. COMPATIBILITY MATRIX
   → Determine the PROJECT's own license + distribution model (proprietary? OSS? SaaS?)
   → For each dependency license, is it compatible with that model?
   → AGPL anywhere + SaaS = contamination risk (the network-use clause)
   → GPL in a distributed binary = obligation to release source
   → "Custom" / non-OSI licenses = legal review required

3. ATTRIBUTION OBLIGATIONS
   → Which licenses require attribution / NOTICE files? (Apache-2.0, BSD, MIT)
   → Does the project ship the required attribution? (THIRD-PARTY-NOTICES, about page)
   → Missing required attribution = compliance gap even for permissive licenses

4. DUAL & CHANGED LICENSES
   → Any package that recently changed license? (e.g. Elastic, MongoDB SSPL, Terraform BSL)
   → Pinned to the last OSI-licensed version, or already on the restrictive one?
   → Dual-licensed deps: which license are you electing? Is it recorded?

5. DECLARED vs ACTUAL (Popper)
   → package.json says "MIT" — does the LICENSE file in the package actually say MIT?
   → Bundled/vendored sub-components with their own (different) licenses?
```

**Severity:** Strong copyleft (AGPL/GPL) incompatible with the distribution model = CRITICAL. Missing/unknown license on a shipped dep = HIGH. Missing attribution for permissive licenses = MEDIUM.

---

## PHASE 5: LOCKFILE INTEGRITY & REPRODUCIBLE BUILDS

> *"No lockfile, no truth. A manually-edited lockfile, a lie with a timestamp."*

```
1. LOCKFILE PRESENCE & SCOPE
   → Does a lockfile exist for EVERY ecosystem? (package-lock/pnpm-lock/yarn.lock, Cargo.lock, poetry.lock, go.sum)
   → Is it COMMITTED to git? (a gitignored lockfile = non-reproducible CI)
   → Library vs app: Cargo.lock for a library is debatable; for a binary/app it's mandatory.

2. MANIFEST ↔ LOCKFILE CONSISTENCY
   → Run the manager's verify mode: `npm ci --dry-run`, `pnpm install --frozen-lockfile --dry-run`,
     `cargo verify-project`, `poetry check`
   → Does the lockfile satisfy the manifest, or has the manifest drifted?
   → Are there manifest entries with NO lockfile resolution? (install was never run)

3. INTEGRITY HASHES
   → Does each lockfile entry carry an integrity hash (sha512 / checksum)?
   → Any entries WITHOUT integrity? (tarball could be swapped without detection)
   → npm: `"integrity"` present on every resolved package?

4. MANUAL-EDIT FORENSICS
   → git blame / git log the lockfile: was it ever hand-edited (not by the tool)?
   → A diff that changes a resolved version without a manifest change = suspicious
   → Resolved URL pointing at a non-canonical registry mirror?

5. REPRODUCIBILITY PROOF (Popper)
   → THE falsification: would `rm -rf node_modules && npm ci` produce the SAME tree?
   → If feasible in a scratch dir, do it and diff the resolved versions.
   → If not feasible, verify `--frozen-lockfile`/`npm ci` would succeed without modifying the lockfile.
   → A build that mutates its own lockfile during install is NOT reproducible.
```

**Severity:** No committed lockfile on an app = CRITICAL. Missing integrity hashes = HIGH. Drift between manifest and lockfile = HIGH. Hand-edited lockfile = HIGH (investigate intent).

---

## PHASE 6: TRANSITIVE RESOLUTION & DEPTH

> *"You approved 40 packages. You installed 1,200. Who let in the other 1,160?"*

```
1. TREE SHAPE
   → Direct count vs transitive count vs total unique packages
   → Maximum depth (A→B→C→...→Z): >7 levels = fragile, hard to reason about
   → Fan-out hotspots: which transitive packages are pulled by the MOST parents?

2. SINGLE POINTS OF SUPPLY FAILURE
   → A transitive package that, if compromised, lands in N% of the tree
   → Cross-reference with Phase 3 (abandonment) and Phase 8 (install scripts):
     deep + abandoned + has-postinstall = the hinge of maximum risk

3. PHANTOM / GHOST DEPENDENCY RECONCILIATION (from depcheck)
   → GHOST: `require`d/imported in code but NOT in any manifest (works only because a
     transitive dep happens to hoist it — breaks the day that transitive dep is removed)
   → PHANTOM: declared in manifest but never imported anywhere (dead weight, audit surface)
   → Reconcile manifest-vs-installed.json — every code import must trace to a DECLARED dep.

4. RESOLUTION OVERRIDES
   → Any `overrides` / `resolutions` / `[patch]` forcing a version?
   → Why? (usually to force a CVE fix into a transitive dep) — is it still needed?
   → Does the override actually take effect in the resolved tree? (verify with npm ls)
```

---

## PHASE 7: TYPOSQUATTING, DEPENDENCY CONFUSION & NAMESPACE RISK

> *"`crossenv` is not `cross-env`. One of them mined cryptocurrency on install."*

```
1. TYPOSQUAT DETECTION
   For every direct dependency name:
   → Levenshtein-distance-1 from a far-more-popular package? (reqeusts vs requests, loadash vs lodash)
   → Suspicious homoglyphs / hyphenation swaps (cross-env vs crossenv vs cross_env)
   → Recently published, low-download package with a name near a famous one = RED FLAG

2. DEPENDENCY CONFUSION (internal/private package risk)
   → Any scoped/internal package names (@company/foo) ALSO claimable on the public registry?
   → Is the package manager configured to prefer the public registry for those scopes? (the
     classic confusion attack — public version with a higher number wins)
   → Are private scopes pinned to a specific registry in .npmrc / .cargo/config / pip.conf?

3. NAMESPACE / OWNERSHIP TAKEOVER
   → Any dependency whose npm/crates owner recently changed?
   → Any dependency pointing at a git URL or tarball instead of the registry? (unverifiable provenance)
   → GitHub-installed deps (`user/repo`) with no commit pin = moving target

4. INSTALL-SOURCE TRUST
   → Are all deps installed from the canonical registry, or from mirrors/forks/CDNs?
   → Any `file:` / `link:` / `http(s):` tarball dependencies? (bypass registry integrity)
```

**FALSIFY:** For each typosquat suspicion, check `npm view <suspect> downloads` and publish date — a low-download, recently-published near-miss of a famous name is the strong signal; a legitimately-named low-download package is not. Don't flag every short name.

---

## PHASE 8: INSTALL & LIFECYCLE SCRIPT AUDIT

> *"`npm install` is `curl | bash` with extra steps. Every postinstall script runs on your machine and your CI, with your tokens in the environment."*

```
1. SCRIPT INVENTORY
   → Enumerate every dependency (direct + transitive) declaring a lifecycle script:
     preinstall, install, postinstall, prepare, prepublish (npm);
     build.rs (Cargo); setup.py exec (pip).
   → How many run on a fresh `npm ci`? (this is the actual code that executes on install)

2. SCRIPT CONTENT FORENSICS (read the actual script for flagged packages)
   → Does it download from the network? (curl/wget/fetch to a non-registry URL)
   → Does it read env vars / files outside the package dir? (token exfiltration shape)
   → Does it write outside its own directory? (filesystem tampering)
   → Does it spawn shells / eval dynamic strings?
   → Is the script obfuscated / minified / base64-encoded? (legitimate install scripts are readable)

3. PRIVILEGE & TRUST
   → Cross-reference with Phase 3/7: install script + abandoned/typosquat/low-trust = CRITICAL
   → Hinge-package install script = max scrutiny (read it line by line)

4. MITIGATION POSTURE
   → Could install scripts be disabled? (`npm ci --ignore-scripts` + explicit allowlist)
   → Is the project already using `--ignore-scripts` or a scripts allowlist?
   → For Cargo: are build.rs scripts from untrusted crates sandboxed/reviewed?
```

**Severity:** Network-fetching or env-reading postinstall on a low-trust/abandoned package = CRITICAL. Any unreviewed obfuscated install script = HIGH.

---

## PHASE 9: VERSION POLICY (PINNED vs FLOATING)

> *"`^` means 'I pre-approve code this maintainer hasn't written yet.' For production, that's a standing authorization you can't audit."*

```
1. RANGE POLICY CENSUS
   → For each direct dep: exact pin (1.2.3) vs caret (^1.2.3) vs tilde (~1.2.3) vs wildcard (*/latest/x)?
   → Count: how many are floating? What proportion of the manifest?
   → `*` or `latest` or `>=` anywhere = unbounded trust = CRITICAL for a production app

2. APP vs LIBRARY EXPECTATION
   → APPLICATION: lockfile pins exact, manifest ranges are fine IF the lockfile is committed.
   → LIBRARY (published): ranges SHOULD be reasonably permissive (don't over-pin and create
     diamond conflicts downstream) but MUST avoid `*`.
   → Judge the policy against what the project IS.

3. CONSISTENCY
   → Is the pinning policy consistent, or ad-hoc per-dependency? (entropy signal)
   → Are dev vs prod dependencies pinned with different rigor? (prod should be at least as strict)

4. FLOATING-RANGE DRIFT RISK
   → Combined with Phase 5: a floating range + a committed lockfile = OK (lockfile pins reality).
   → A floating range + NO lockfile = the build is non-deterministic by design = CRITICAL.
```

---

## PHASE 10: BLOAT & DUPLICATION

> *"You installed three versions of the same package, two utility libraries that do the same thing, and a 4MB dep to left-pad a string."*

```
1. DUPLICATE VERSIONS
   → Same package present at multiple versions in the resolved tree (npm ls / dedupe report)
   → Each duplicate = wasted install size, larger bundle, divergent CVE exposure
   → Can `npm dedupe` / pnpm's stricter resolution collapse them?

2. REDUNDANT CAPABILITY
   → Multiple deps providing the SAME capability (axios + got + node-fetch + undici; moment + dayjs;
     lodash + ramda + underscore)
   → Each redundant lib = more surface, more to keep patched. Consolidate.

3. HEAVYWEIGHT FOR TRIVIAL USE
   → Large dependency used for one trivial function (a whole date lib for one format call;
     a kitchen-sink util for one helper)
   → Candidate for inlining a small helper or swapping to a focused micro-dep.

4. DEV-DEP LEAKAGE
   → Anything in `dependencies` that should be in `devDependencies`? (ships to prod/bundle needlessly)
   → Build/test tooling in prod deps = bloat + attack surface in the deployed artifact.

5. TREE-SHAKEABILITY (frontend)
   → CJS-only deps that defeat tree-shaking → entire library bundled even for one import.
```

**Note:** Bloat findings are mostly MEDIUM/LOW (hygiene), but a duplicate that pins a CVE-affected version alongside a fixed one escalates to HIGH (cross-ref Phase 1).

---

## PHASE 11: REGISTRY TRUST & PROVENANCE

> *"Integrity proves the tarball wasn't tampered with in transit. Provenance proves it came from the source it claims."*

```
1. PROVENANCE ATTESTATIONS
   → Do critical deps publish npm provenance / Sigstore attestations? (built from public CI, verifiable)
   → Cargo: crates from verified sources? Go: module checksums in sum DB?
   → Provenance present on the hinge package and high-privilege deps?

2. REGISTRY CONFIGURATION
   → Read .npmrc / .yarnrc / .cargo/config.toml / pip.conf
   → Which registries are configured? Any non-default mirror? Is it trusted?
   → Are private scopes correctly bound to the private registry (Phase 7 confusion defense)?
   → Any `registry=http://` (non-TLS)? Any auth token committed in a tracked .npmrc?

3. MAINTAINER ACCOUNT HYGIENE (best-effort, for hinge + high-privilege deps)
   → Maintainer count, account age signals, recent ownership transfers (Phase 3 overlap)
   → 2FA-on-publish enforced where the registry exposes it?

4. SUBRESOURCE INTEGRITY (CDN-loaded deps, frontend)
   → CDN <script>/<link> with integrity + crossorigin attributes?
   → SRI hashes present and current? Fallback if CDN compromised?
```

---

## PHASE 12: BUNDLE EXPOSURE (server deps leaking to client)

> *"A server-only secret manager imported into a client component ships its API to every browser."*

```
1. CLIENT BUNDLE COMPOSITION (frontend projects)
   → Build the frontend (or read an existing build report / analyzer output)
   → Which dependencies end up in the client bundle?
   → Any server-only package (db driver, secrets SDK, fs/crypto-heavy lib) in the client chunk?

2. SECRET / CONFIG LEAKAGE
   → Cross-ref /secaudit Phase 15: any dep embedding a secret that gets bundled?
   → NEXT_PUBLIC_/VITE_ exposure of values that came from a dependency's default config?

3. SIZE BUDGET
   → Heaviest deps in the bundle (cross-ref /perfaudit if available)
   → Any dep contributing disproportionate KB for marginal value? (Phase 10 overlap)

4. DUAL-PACKAGE HAZARD
   → ESM/CJS dual packages loaded twice (state duplication, larger bundle)?
```

> Non-frontend project (CLI/library/backend-only)? Mark this phase N/A in the scoring and
> redistribute its weight per the preamble normalization rule. Do NOT fabricate findings.

---

## PHASE 13: MONOREPO & WORKSPACE HYGIENE

> *"In a monorepo, one mispinned shared dependency is mispinned everywhere — or worse, pinned five different ways."*

```
1. WORKSPACE TOPOLOGY
   → Detect workspaces (pnpm-workspace.yaml, package.json workspaces, Cargo workspace, Turbo/Nx)
   → Map internal package graph: which workspace packages depend on which?

2. VERSION CONSISTENCY ACROSS WORKSPACES
   → Is the same external dependency pinned to DIFFERENT versions in different packages?
   → Divergent versions = duplicate installs + inconsistent CVE exposure + bundle bloat
   → Is there a single-version policy (syncpack / pnpm catalog / Cargo workspace deps) enforced?

3. INTERNAL vs EXTERNAL BOUNDARIES
   → Are workspace-internal deps referenced via workspace protocol (workspace:*) not a published range?
   → Phantom hoisting: a package using a dep it doesn't declare because the monorepo root hoisted it?

4. ROOT vs PACKAGE DEPS
   → Dev tooling correctly at the root, runtime deps in the owning package?
   → Hoisting strategy (pnpm strict vs hoisted) understood and intentional?
```

> Single-package project? Mark N/A, redistribute weight (preamble normalization).

---

## PHASE 14: DEPRECATIONS & EOL RUNTIME

> *"A deprecation warning is a countdown. A package and a runtime both have expiry dates — and neither sends a reminder."*

```
1. PACKAGE DEPRECATIONS
   → Registry-level deprecation messages on any installed package (npm shows "deprecated: ...")
   → Deprecated-with-replacement: is the migration tracked?
   → Deprecated APIs within deps you use (changelog / type deprecations)?

2. RUNTIME / ENGINE EOL
   → Declared engines (node, npm, python, rust msrv, go) — are any past EOL or near it?
   → Does a dependency require a newer engine than the project declares? (silent incompatibility)
   → `engines` field present and honored, or ignored?

3. PEER DEPENDENCY HEALTH
   → Unmet peer dependencies? (works today by luck, breaks on the next install)
   → Conflicting peer ranges across the tree?
   → Peer deps auto-installed (npm 7+) vs expected-but-absent?

4. PLATFORM / ABI DEPS
   → Native modules (node-gyp, prebuilt binaries) — supported on the deploy platform/arch?
   → Optional deps that fail silently on the target platform?
```

---

## PHASE 15: REPRODUCIBILITY & BUILD INTEGRITY

> *"Reproducibility is the property that lets you trust every other finding. Without it, you audited a tree you'll never run again."*

```
1. CLEAN-INSTALL DETERMINISM (Popper, the keystone test)
   → In a scratch directory (NEVER the working tree), attempt a frozen install:
     `npm ci` / `pnpm install --frozen-lockfile` / `yarn install --immutable` /
     `cargo build --locked` / `poetry install --no-update`
   → Does it succeed WITHOUT mutating the lockfile? (mutation = non-reproducible)
   → Diff the resolved versions against the committed lockfile — any drift = finding.

2. CI vs LOCAL PARITY
   → Does CI use the frozen/locked install command, or a loose `npm install`?
   → A CI that runs `npm install` (not `npm ci`) can silently resolve different versions than dev.

3. POST-INSTALL ARTIFACT INTEGRITY
   → Generated artifacts (patches via patch-package, prebuilt binaries) checked into git with hashes?
   → Any `patch-package` patches — do they still apply cleanly? Are they reviewed?

4. SBOM ↔ TREE PARITY
   → Does the generated SBOM (Phase 16) match the actually-resolved tree exactly?
   → A drifted SBOM describes phantom software — useless for incident response.
```

> SAFETY: all reproducibility tests run in a throwaway scratch dir or `--dry-run` mode.
> NEVER mutate the project's `node_modules`, lockfile, or `target/` during the audit phase.

---

## PHASE 16: SBOM GENERATION & COMPLETENESS

> *"When the next supply-chain incident drops, the only question that matters is 'are we affected?' An SBOM answers it in seconds. No SBOM answers it in days."*

```
1. SBOM GENERATION
   → Produce a complete CycloneDX SBOM (sbom/sbom.cyclonedx.json) AND an SPDX alternative
     (sbom/sbom.spdx.json) covering EVERY ecosystem present.
   → One unified SBOM if multi-ecosystem, or one per ecosystem, clearly scoped.

2. COMPLETENESS CHECK
   → Component count in SBOM == unique packages in the resolved tree? (parity, Phase 15.4)
   → Each component carries: name, version, PURL (package URL), license, hashes, supplier.
   → Any component missing a license or hash = incomplete SBOM = finding.

3. VULNERABILITY-LINKED SBOM
   → Annotate (or cross-link) the SBOM with the Phase 1 CVE findings (VEX-style).
   → The SBOM should be a queryable source of truth for "do we use package X?".

4. MAINTENANCE POSTURE
   → Is SBOM generation wired into CI (regenerated on every dependency change)?
   → Or is this audit the first SBOM the project has ever had? (recommend CI integration)
```

**Deliverable:** `audits/.depaudit/sbom/sbom.cyclonedx.json` is a primary artifact of every full run.

---

## PHASE H1 — HYBRID SYNTHESIS (Popper / hinge / user-need / edge cases / cross-audit)

> Runs immediately before VERDICT. "H1" does NOT renumber existing phases — it sits between the last
> domain phase and the verdict, and REINVESTS the token budget freed by Phase 0's deterministic gather
> into depth. It deepens earlier phases; it replaces none.

### H1.1 Popper falsification per finding (mandatory)

For every finding (start with `severity ∈ {critical, high}`), try to PROVE the tool wrong. Each
falsification produces a `falsifiable_tests[]` entry in `verdict.json`:

```jsonc
{
  "claim": "osv-scanner says lodash@4.17.11 is vulnerable to CVE-2019-10744 (prototype pollution)",
  "test_command": "npm ls lodash --all",
  "expected": "lodash@4.17.11 present in the resolved tree → claim TRUE",
  "actual": "myapp > some-lib > lodash@4.17.11 (1 path)",
  "outcome": "confirmed"
}
```

Outcomes: `confirmed` (test failed to falsify → finding stands), `falsified` (counter-example → demote
to info + record `falsified_at`), `inconclusive` (couldn't run cleanly → keep severity, `confidence: medium`).

**The rule:** every CLAIM (PASS or FAIL) MUST cite ≥3 concrete commands that COULD have falsified it but didn't. Banned phrases (`looks correct`, `should be fine`, `appears to work`) → automatic FAIL.

Common falsification patterns for THIS domain:

| Tool says | Popper test |
|---|---|
| `CVE present` (osv/npm audit) | `npm ls <pkg>` / `cargo tree -i <pkg>` — confirm the version is actually resolved, find every path |
| `package abandoned` | `npm view <pkg> time.modified` + repo — is it *finished* (stable, no open CVEs) or *neglected* (unpatched issues)? |
| `unused dependency` (depcheck) | `grep -rn "<pkg>"` across src, config files, dynamic `import()`, and package scripts before removing |
| `typosquat suspicion` | `npm view <suspect> downloads` + publish date — low-download near-miss of a famous name vs legitimately small pkg |
| `license GPL` | Read the package's actual LICENSE file — registry metadata sometimes mislabels dual/changed licenses |
| `lockfile drift` | `npm ci --dry-run` in a scratch dir — does it mutate, or is the diff a tooling artifact? |
| `postinstall malicious` | Read the script verbatim — network/env access vs a benign native-build step |

### H1.2 Hinge cross-reference (10× scrutiny on the load-bearing dependency)

The SUPPLY-CHAIN HINGE POINT (`discovery/hinge-package.json`) is the dependency of maximum blast radius.
Apply 10× scrutiny:
- 5× more falsification attempts (H1.1) on findings touching it
- Read its install scripts line-by-line (Phase 8), its license file (Phase 4), its maintainer set (Phase 11)
- Trace EVERY dependency path that reaches it (`npm ls <hinge>` / `cargo tree -i`)
- Mark each such finding `is_load_bearing: true` and emit `hinge_findings[]` in `verdict.json`.

### H1.3 User-need verification (`--user-need` quote)

If dispatched with `--user-need="<verbatim>"`, evaluate every finding against it. Findings unrelated to
the user-need get demoted one severity (unless load-bearing). Findings that DO address it get top fix
priority and populate `user_need_match.findings[]`. If `addressed: false`, the audit MUST score < 90.

### H1.4 Edge-case hunting (mandatory for top-5 findings)

For each top-5 finding generate ≥2 edge cases the scanner couldn't model. Supply-chain patterns:
- "The CVE scanner saw the version at rest, but a `resolutions`/`overrides` entry actually pins a
  different version at install time..."
- "depcheck saw no import, but the dep is loaded via a dynamic `require()` / feature-flag / CLI plugin..."
- "License looks MIT in the manifest, but a vendored sub-component in `/dist` is GPL..."
- "Lockfile is committed, but CI runs `npm install` not `npm ci`, so prod resolves a different tree..."
- "Private scope is pinned to the internal registry locally, but the CI `.npmrc` omits the scope →
  dependency-confusion window..."
- "Native module resolves on the dev mac (arm64) but not the deploy target (linux/amd64)..."

Emit `edge_cases[]` in `verdict.json` (scenario, covered_by_existing_test, evidence_gathered, fix_includes_coverage).

### H1.5 Cross-audit synthesis (re-read sibling summaries from Phase 0.5)

For each top-5 finding, check if the same package/file/advisory is flagged by a sibling audit:
- depaudit (present) + secaudit (exploitable) on the same CVE → `cross_audit_confirmed: true`, bump severity.
- depaudit (ghost/unused) + codeaudit (dead import) → confirmed safe-to-remove.
- depaudit (heavy bundle) + perfaudit (bundle bloat) → joint fix.
Write `cross_audit_links[]` in `verdict.json`.

### H1.6 Final verdict.json schema (hybrid v2)

```jsonc
{
  "audit": "dep",
  "score": 100,
  "score_raw": "<raw>/360",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "dep",
  "preamble_version": "1.0",
  "ecosystems": ["npm", "cargo"],
  "user_need_match": { /* H1.3 */ },
  "falsifiable_tests": [ /* H1.1 */ ],
  "hinge_findings": [ /* H1.2 */ ],
  "issues_found_and_fixed": [
    { "id": "FIX-001", "finding_id": "F-003", "package": "lodash", "before": "4.17.11", "after": "4.17.21",
      "verification": "npm ls lodash → 4.17.21; osv-scanner re-run → 0 advisories" }
  ],
  "edge_cases": [ /* H1.4 */ ],
  "cross_audit_links": [ /* H1.5 */ ],
  "sbom_path": "audits/.depaudit/sbom/sbom.cyclonedx.json",
  "evidence_summary_path": "$PROJECT_PATH/audits/.depaudit/evidence-summary.json",
  "confidence_basis": "Why I'm confident: Popper test counts, hinge scrutiny depth, reproducibility proof, edge-case coverage, cross-audit confirmations.",
  "banned_phrase_check": "passed (no `looks correct`, `should be fine`, `appears to work`, `streamlined`, `to save time`)"
}
```

### H1.7 Score gating (hybrid threshold)

100/100 is blocked unless: all critical/high findings fixed or justified (≥50 words + Popper evidence);
all load-bearing findings Popper-confirmed; `user_need_match.addressed=true` with verbatim quote;
≥3 falsifiable tests per phase; ≥2 edge cases per top-5 finding; reproducibility proven (Phase 15);
SBOM generated and parity-checked; cross-audit array present. Below threshold → score < 100, fix loop
kicks in (bounded at 5 iterations per the Audit Verification Contract; on iteration 5 still failing →
`confidence: low`, surface as `pending` in `.done.json`).

---

## PHASE 17: VERDICT

Score each domain phase 0-10, weight by supply-chain severity:

```
SCORING MATRIX (360 max):
  Phase  1  (CVE Exposure)              x 3.0  = max 30
  Phase  2  (Freshness & Drift)         x 1.5  = max 15
  Phase  3  (Abandonment / Maintenance) x 2.5  = max 25
  Phase  4  (License Compliance)        x 3.0  = max 30
  Phase  5  (Lockfile Integrity)        x 3.0  = max 30
  Phase  6  (Transitive Resolution)     x 2.0  = max 20
  Phase  7  (Typosquat / Confusion)     x 3.0  = max 30
  Phase  8  (Install / Lifecycle Scripts) x 3.0 = max 30
  Phase  9  (Version Policy)            x 2.0  = max 20
  Phase 10  (Bloat & Duplication)       x 1.0  = max 10
  Phase 11  (Registry Trust / Provenance) x 2.0 = max 20
  Phase 12  (Bundle Exposure)           x 2.0  = max 20
  Phase 13  (Monorepo Hygiene)          x 1.5  = max 15
  Phase 14  (Deprecations / EOL)        x 1.5  = max 15
  Phase 15  (Reproducibility)           x 3.0  = max 30
  Phase 16  (SBOM)                      x 2.0  = max 20
                                        TOTAL  = max 360

NORMALIZE: score = (raw / applicable_max) × 100
  (applicable_max excludes phases marked N/A — e.g. Bundle Exposure on a CLI,
   Monorepo Hygiene on a single package — per preamble normalization.)

PASS THRESHOLD: normalized >= 70.

GRADE:
  90-100: S — Trusted chain. Pinned + locked + reproducible, zero open CVEs, all licenses clear, SBOM in CI.
  80-89:  A — Hardened. Minor lag or a few low advisories, no critical exposure, reproducible build.
  70-79:  B — Acceptable. Some outdated/abandoned deps or attribution gaps, no critical CVE/license risk. (PASS floor)
  60-69:  C — Drifting. Floating ranges, lockfile gaps, or unreviewed install scripts present.
  50-59:  D — Exposed. Open high CVEs, abandoned hinge deps, or copyleft contamination risk.
  <50:    F — Compromised chain. Critical CVE with fix unapplied, malicious-shaped install script,
              license bomb, or non-reproducible build with no lockfile.
```

---

## PHASE 18: FIX PLAN → FIX EXECUTION → RE-AUDIT

### Fix plan (automatic)

```
Sort: CRITICAL → HIGH → MEDIUM → LOW.
Priority by supply-chain exploitability:
  CRITICAL: open critical/high CVE with a fix available; malicious-shaped install script;
            non-reproducible production build; copyleft contamination of a proprietary product.
  HIGH:     abandoned hinge dependency; missing lockfile integrity; dependency-confusion window;
            unknown/missing license on a shipped dep.
  MEDIUM:   outdated-but-unaffected deps; missing attribution; bloat/duplication that pins a stale version.
  LOW:      cosmetic version lag; redundant utility libraries; tree-shake opportunities.

Group by blast radius — ONE version bump may close N transitive CVEs.
Dependency order — fix the parent before the transitive (the parent bump may resolve both).
Generate fix tasks with package + ecosystem + from→to specificity.
Save to audits/.depaudit/fix-plan.json + fix-plan.md.
```

### Fix execution (automatic) — DO NO HARM safety gate

> **Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix.** A dependency bump that breaks the
> build or a transitive API is WORSE than the original finding.

```
─── BUILD-INTEGRITY SAFETY GATE (MANDATORY before EVERY fix) ───────────

PRE-FIX BASELINE:
  a. Capture the current resolved version of the target + its direct dependents.
  b. Capture build/test green state: run `~/.omega/lib/safe-npm-build.sh` (or the project build),
     record exit code. Run the test suite if present.
  c. Read the target dep's changelog between from→to — is the bump SemVer-minor/patch (safe)
     or major (breaking API)? Major bumps require reading every call site of the dep's API.

APPLY FIX (one dependency / group at a time):
  - Direct CVE/outdated: bump the version in the manifest, then run the manager's lock-update
    for that package ONLY (`npm install <pkg>@<ver>` / `cargo update -p <pkg> --precise <ver>`).
  - Transitive via maintained parent: bump the parent.
  - Transitive via abandoned parent: add an `overrides`/`resolutions`/`[patch]` entry, document why.
  - Unused/phantom dep: remove from manifest (only after the depcheck Popper test in H1.1 confirms 0 imports).
  - License/typosquat/malicious: REPLACE the package (record the chosen successor) — do not just pin.

POST-FIX VERIFICATION (BEFORE commit):
  a. Re-resolve: `npm ci` (scratch) / lock update — must succeed WITHOUT errors or peer conflicts.
  b. BUILD: `~/.omega/lib/safe-npm-build.sh` (or project build) — exit 0.
  c. TESTS: run the relevant suite — must stay green.
  d. RE-SCAN the single fixed advisory: re-run the matching scanner scoped to the package → 0 advisories.
  e. Reproducibility: confirm the lockfile changed ONLY in the intended way (git diff the lockfile).

IF ANY POST-FIX CHECK FAILS:
  → revert the manifest + lockfile change immediately
  → log the failure in audits/.depaudit/fix-log.md with the exact error
  → mark NEEDS_REVIEW (a major-version bump that breaks an API needs a code migration, not a blind retry)
  → try the next-best fix (e.g. pin to the last safe minor) OR skip + record accepted risk.

HIGH-RISK fixes (major bumps of the hinge dep, framework upgrades, native-module swaps):
  → require human confirmation; never auto-merge a breaking major bump unattended.
────────────────────────────────────────────────────────────────────────

Produce audits/.depaudit/before-after.md — per fixed package: from→to, advisory closed,
build status before/after, tests before/after. Zero regressions required to claim done.
```

### Re-audit (automatic)

```
1. BUILD-HEALTH GATE: full build must pass; full test suite must pass; for a service, restart + is-active.
2. Re-run the gather (Phase 0) on the new tree → confirm fixed advisories are GONE and no NEW ones appeared.
3. Regenerate the SBOM (Phase 16) → parity with the new resolved tree.
4. Re-prove reproducibility (Phase 15) on the new lockfile.
5. Re-score. Loop until normalized >= 70 (PASS) and all CRITICAL/HIGH resolved or NEEDS_REVIEW.
6. Special attention: a bump must not have INTRODUCED a new advisory, a new license, or a new install script.
```

---

## Dynamic-Workflow Orchestration (v2)

> *"The dependency tree is not a list to walk top-to-bottom — it is a forest of independent supply chains. Audit them in parallel, prove each link adversarially, and only the surviving links earn a place in the score."*

This section governs HOW the 18 phases above EXECUTE. It changes nothing about WHAT they assess: every phase, weight, threshold, verdict format, and the Gestalt-Popper doctrine remain exactly as written. It replaces linear phase-walking with a fan-out → adversarial-verify → synthesize → loop-until-dry execution model, run via the **Workflow** tool.

### 1. Fan-out — decompose phases into INDEPENDENT parallel tracks

Phase 0 (the deterministic gather) and Phase 0b (crime-scene setup + hinge computation) ALWAYS run FIRST and serially — they produce the shared evidence base (`evidence-summary.json`, `discovery/dependency-tree.json`, `discovery/hinge-package.json`) every track reads. Once that base exists, the domain phases are **file-disjoint by data dependency**, not by ordering, so dispatch them as concurrent Workflow tracks instead of one linear pass:

| Track | Phases | Why it is independent | Shared input (read-only) |
|---|---|---|---|
| **A — Vulnerability & freshness** | 1 (CVE), 2 (Freshness/Drift), 14 (Deprecations/EOL) | All read advisory + version-lag data; no cross-write | `evidence-summary.json`, manifests/lockfiles |
| **B — Trust & provenance** | 3 (Abandonment), 7 (Typosquat/Confusion), 8 (Install scripts), 11 (Registry/Provenance) | All interrogate maintainer/source/script trust signals | `evidence-summary.json`, hinge source, registry metadata |
| **C — Integrity & reproducibility** | 5 (Lockfile), 6 (Transitive resolution), 9 (Version policy), 15 (Reproducibility) | All operate on lock/manifest resolution + determinism | lockfiles, scratch-dir frozen installs |
| **D — Licensing** | 4 (License compliance) | Pure license-graph analysis, no overlap | `evidence-summary.json`, package LICENSE files |
| **E — Footprint & exposure** | 10 (Bloat/Duplication), 12 (Bundle exposure), 13 (Monorepo hygiene) | All read tree shape + bundle/workspace topology | dependency-tree, build/analyzer output |
| **F — SBOM** | 16 (SBOM generation) | Consumes the resolved tree; emits its own artifact | dependency-tree, Track A advisories (VEX link) |

Rules for the fan-out:
- **Read-only concurrency.** Tracks only READ the shared evidence base + the repo; none mutate `node_modules`, lockfiles, or `target/` during the audit phase (Phase 15's frozen-install determinism test runs in a throwaway scratch dir per the existing safety note). This makes the tracks safe to run simultaneously (no shared-writer conflict).
- **Each track is its own Workflow sub-task** with a written brief: the phase numbers it owns, the exact `evidence-summary.json` slice + manifests it may read, and its Done Criteria (every owned phase scored with ≥3 Popper tests cited, per Phase H1.1).
- **Hinge gets 10× depth IN-TRACK.** Whichever track touches the supply-chain hinge package (`discovery/hinge-package.json`) — typically B (install scripts/provenance) and C (resolution) — applies the Phase H1.2 10× scrutiny inside that track, not as a separate pass.
- **Scope-narrowing flags compose with fan-out.** A `--focus=licenses` run still fans out, but Track D runs at full depth and the others at proportional depth (never skipped — rule 46 / `--quick` remains forbidden).

### 2. Adversarial verification — ≥2-of-3 independent lenses per finding

A finding emitted by ANY track is a CANDIDATE, never an accepted finding. Before it may enter the scoring matrix it must survive **≥2 of these 3 independent lenses** (this operationalizes R-VERIFY + the existing Phase H1.1 falsification, and the audit's own Popper categories CLAIM-vs-REALITY / MANIFEST-vs-INSTALLED / PINNED-vs-RESOLVED / DECLARED-vs-LICENSED / AUDITED-vs-EXPLOITABLE):

- **Lens 1 — REPRODUCE (does it actually resolve in MY tree?).** Run the concrete command that proves presence: `npm ls <pkg> --all` / `cargo tree -i <pkg>` / `pipdeptree -p <pkg>` confirms the flagged version is genuinely in the resolved tree and enumerates every dependency path. A CVE/abandonment/typosquat candidate that does not reproduce here is dead.
- **Lens 2 — REFUTE (try to make the tool wrong).** Actively seek the counter-example using the Phase H1.1 patterns: `npm view <pkg> time.modified` to separate *finished* from *neglected*; `grep -rn "<pkg>"` across src + config + dynamic `import()` + package scripts before accepting an "unused/ghost" verdict; read the actual LICENSE file before accepting a registry "GPL" label; read the postinstall script verbatim before accepting "malicious"; `npm ci --dry-run` in a scratch dir before accepting "lockfile drift" (tooling artifact vs real drift). If the counter-example holds → the candidate is FALSIFIED.
- **Lens 3 — CROSS-CHECK (independent corroboration).** Confirm via a second, independent source: a second scanner agreeing (`npm audit` + `osv-scanner` on the same advisory → `cross_tool_confirmed`), or a sibling audit's `evidence-summary.json` (Phase 0.5 / H1.5: secaudit reachability, codeaudit dead-import, perfaudit bundle bloat → `cross_audit_confirmed`), or the override/`resolutions` entry that actually changes the install-time version vs the at-rest manifest.

Verdict rule per candidate:
- **≥2 lenses confirm → ACCEPTED** → carries its `falsifiable_tests[]` (≥3 commands, verbatim output) into Phase H1.6 `verdict.json`. Two confirming lenses where one is Lens 3 (cross-tool or cross-audit) → set `cross_tool_confirmed` / `cross_audit_confirmed` and bump severity one level exactly as Phase 0.5 / H1.5 / the /secaudit relationship table already prescribe.
- **Refuted by Lens 2 (counter-example) → KILLED** → demoted to `info`, recorded with `outcome: "falsified"` + `falsified_at`; it MUST NOT contribute to the score. Killing unsurvivable findings is the point — a green-but-unverified candidate is worse than no finding (L2).
- **Only 1 lens, others inconclusive → HELD** → keep at stated severity with `confidence: "medium"` and `outcome: "inconclusive"`; never promote a single-lens candidate to `high` confidence.

Banned shortcut phrases (`looks correct`, `should be fine`, `appears to work`) auto-fail a lens, exactly as the meta-protocol header requires.

### 3. Synthesize — fold survivors back into THIS audit's UNCHANGED scoring

Synthesis is the auditor's own job (R-ORCH: never paste a track's summary as the verdict). After all tracks return and every candidate has been run through the lenses:

1. **Merge + dedupe across tracks.** The same advisory can surface from Track A (CVE) and Track F (SBOM VEX link); the same package can surface from Track B (abandoned) and Track E (bloat). Collapse to unique findings (dedupe by advisory ID / package+ecosystem), preserving the highest justified severity and the union of `falsifiable_tests[]`.
2. **Score with the EXISTING matrix only.** Map each surviving finding to its Phase (1–16) and apply the **Phase 17 SCORING MATRIX exactly as written** (CVE ×3.0 … SBOM ×2.0, max 360), the existing N/A renormalization for non-applicable phases (12/13 on CLI/single-package), and the existing normalization `(raw / applicable_max) × 100`. The fan-out changes the order findings were produced in — it does NOT change a single weight, the PASS threshold (≥70), or the S/A/B/C/D/F grade bands.
3. **Run Phase H1 as the synthesis gate.** H1.2 (hinge 10× cross-reference), H1.3 (`--user-need` match), H1.4 (≥2 edge cases per top-5), H1.5 (cross-audit links), and the H1.7 score-gating threshold all run here, over the merged survivor set. Killed candidates are excluded; held candidates cap confidence.
4. **Emit the SAME contract.** `verdict.json` (the unchanged Phase H1.6 hybrid-v2 schema), `verdict.md`, `fix-plan.json/.md`, `progress.json`, `before-after.md`, `fix-log.md`, and `sbom/sbom.cyclonedx.json` — identical to the existing OUTPUT CONTRACT. Phase 18 (fix → build-integrity gate → re-audit, capped at 5 iterations) is unchanged.

### 4. Loop-until-dry — for unknown-size discovery

Some surfaces have no known bound up front: the count of unique advisories across N ecosystems, the set of transitive packages declaring lifecycle scripts, the number of duplicate-version clusters, every dependency path that reaches the hinge package. For these, run the owning track as a **loop-until-dry** Workflow rather than a fixed single pass:

- **Iterate** the discovery within the track (e.g. resolve advisories → for each, enumerate its paths via `npm ls` → those paths may reveal further flagged transitive packages → repeat) until a pass yields **zero new candidates** (dry) — bounded by the Workflow budget primitive (default 500K-token mission cap, R-BUDGET).
- Each newly discovered candidate enters the SAME ≥2-of-3 lens gate in §2 before acceptance — discovery breadth never bypasses verification.
- Track F's SBOM completeness check (Phase 16.2 parity: component count == unique packages in the resolved tree) is the natural "dry" signal for the inventory loop: loop until the SBOM is parity-complete with the resolved tree (Phase 15.4 / H1 parity), then stop.
- This is distinct from the Phase 18 fix→re-audit loop (also capped at 5): §4 is DISCOVERY loop-until-dry (finding everything); Phase 18 is REMEDIATION loop-until-clean (fixing what was found). Both coexist unchanged.

> **Invariant:** This orchestration layer is purely about execution shape — parallel tracks, adversarial gating, synthesis, discovery loops. The audit's identity (forensic supply-chain investigation), its Gestalt-Popper doctrine, its 7 Laws of Supply-Chain Forensics, every phase, the /360 scoring matrix, the verdict format, and the frontmatter are all preserved verbatim. Fan-out finds faster and proves harder; the verdict it feeds is the same verdict this audit always produced.

---

## CROSS-COMMAND BRIDGE

```
/depaudit confirms a CVE is PRESENT      → /secaudit confirms it's REACHABLE/exploitable
/depaudit flags an unused/ghost dep       → /codeaudit confirms the dead import
/depaudit flags a heavy client dep        → /perfaudit confirms bundle bloat → joint fix
/depaudit flags a license obligation      → surfaced for legal review (never auto-resolved)
/depaudit flags a malicious install script→ IMMEDIATE quarantine + replace + alert

THE QUALITY ARSENAL (supply-chain seat):
  /codeaudit  → Is the code SOLID?            (preventive)
  /secaudit   → Is it SECURE at runtime?       (detective)
  /depaudit   → Is the supply chain SAFE?      (preventive)   ← this audit

  Together: your code, your runtime, AND the 95% you didn't write — all covered.
```

---

## LAWS

1. **You did not write the majority of your software.** Audit the 95% you imported, not just the 5% you typed.
2. **A green `npm audit` is a partial result, not a clean bill of health.** Abandonment, typosquats, licenses, install scripts, and drift live outside it.
3. **Pin, lock, reproduce.** A floating range without a committed lockfile is a non-deterministic build, and a non-deterministic build cannot be audited.
4. **Every dependency is a person you're trusting.** Bus factor, 2FA, account age, and provenance matter as much as the CVE count.
5. **A license deep in the tree can own your product (Popper).** Falsify "we're all MIT" by reading the actual license files, including vendored sub-components.
6. **Install scripts are arbitrary code execution on your machine and CI.** Treat every postinstall as `curl | bash` until you've read it.
7. **An SBOM you can't regenerate is fiction.** Reproducibility is the property that makes every other finding trustworthy.

---

## COMPLIANCE ADDENDA (v1.0)

This audit implements `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial framing
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.depaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit bounded at 5; on cap → NEEDS_REVIEW + Telegram SOS
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=` (FULL depth per focus; rule 46 — no `--quick`/`--streamlined`/`--lightweight`)
- ✅ **Non-UI context gate** — runs on any target (web, API, CLI, library, binary). Phase 12/13 mark N/A and renormalize when not applicable; never fabricate findings.
- ✅ **Output contract** — emits `verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `progress.json`, `before-after.md`, `fix-log.md`, `sbom/sbom.cyclonedx.json`
- ✅ **Telegram progress** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos`
- ✅ **Score normalization** — raw / applicable-phase-max × 100; PASS threshold 70
- ✅ **preamble_version** — `"1.0"` in verdict.json for `/metaudit` compliance scan
- ✅ **Build-integrity safety gate** — every dependency change verified via `~/.omega/lib/safe-npm-build.sh` + tests + scoped re-scan before commit; revert on any regression

Run `/metaudit --focus arsenal --scope="depaudit only"` to verify the 11-point preamble checklist.

---

*"/depaudit v1 — Trace every link in the chain you didn't forge. CVEs, licenses, lockfiles, typosquats, install scripts, provenance, SBOM. The 95% you imported is the 95% of your attack surface. /360."*
