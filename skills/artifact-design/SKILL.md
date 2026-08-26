---
name: artifact-design
description: Publishes deliverable reports (audit, research memo, strategy doc, mission recap, dashboard, brief) as LIVE claude.ai Artifacts via the native Artifact tool; surface 1 of the OmegaOS report router (R-ARTIFACT). Use when the user says "report", "live report", "publish an artifact", "make this a report", "dashboard report", "share a page", or in French "rapport", "rapport live", "publie un artifact", "fais-moi un rapport", "tableau de bord". NOT for explicit PDF asks (omega pdf, R-PDF), plain file-only asks (R-HTML), generating videos or images, or scripted browser E2E.
---

# artifact-design: publish deliverable reports as LOCAL self-hosted Artifacts (Tailscale)

> **DOCTRINE OVERRIDE (2026-07-03, authoritative, mirrors R-ARTIFACT).** "Artifact" means a LOCAL,
> self-hosted page on the machine, reachable over Tailscale (like kairos), NOT a claude.ai-account
> artifact. DEFAULT surface = drop a standalone self-contained HTML into `~/.omega/artifacts/`, which is
> served tailnet-only by `tailscale serve --bg --https=8443 ~/.omega/artifacts`, live at
> `https://station.tail64d114.ts.net:8443/<file>.html` (update the `/index.html` listing, verify HTTP 200,
> hand back that URL + the repo file path). The claude.ai native Artifact tool is permitted ONLY when the
> active account is `x@agentik-os.com` (check with
> `python3 -c "import json,os;print(json.load(open(os.path.expanduser('~/.claude.json'))).get('oauthAccount',{}).get('emailAddress'))"`);
> on any other account, never publish to claude.ai. If already published to the wrong account, redact by
> republishing the same URL (Artifact tool `url` param) with a tombstone, then ask the operator to delete
> the shell from the UI. Everything below about the native tool applies ONLY inside that single-account
> exception; the design contract (self-contained HTML, both themes, R-NODASH kill pass) is unchanged.

## 0. What this is

An OmegaOS protocol layer over Claude Code's NATIVE artifact path. It vendors no code,
pulls no external dependency, and auto-installs nothing: everything it orchestrates is
already inside the Claude Code harness. The native facts it builds on (runtime-verified
2026-07-03 on Claude Code 2.1.199):

- Entitled interactive sessions expose an `Artifact` tool. It renders an HTML or Markdown
  file to a private claude.ai page. The same sessions also carry a harness-bundled
  `artifact-design` design-fundamentals skill.
- Headless sessions (`claude -p`, cron) have NEITHER the Artifact tool nor the bundled
  skill. Probe evidence: the full tool list was captured with Artifact absent, and a Skill
  invocation returned "Unknown skill".
- Artifacts are private to their author by default, shareable to the org from the page
  itself, and versioned: republishing the same file path updates the SAME URL. The feature
  is in beta for Claude Team and Enterprise orgs (source:
  claude.com/blog/artifacts-in-claude-code, 2026-06-18).
- This OmegaOS copy installs at ~/.omega/skills/artifact-design and is symlinked into
  ~/.claude/skills by omega sync. Where the harness also bundles its own artifact-design
  skill, either copy resolving is safe: the routing decision lives in rule R-ARTIFACT,
  and this file is design-self-sufficient.

## 1. The router

Mirror of rule R-ARTIFACT. Keep the two in sync: an edit here without the rule (or the
reverse) splits the doctrine.

| Ask | Surface |
| --- | --- |
| A report, no format named | 1: LOCAL self-hosted artifact in ~/.omega/artifacts served over Tailscale (:8443), plus its HTML twin under agentic/reports/. claude.ai native tool ONLY on account x@agentik-os.com |
| A FILE is wanted (attachment, email, repo doc, offline reading) | 2: self-contained HTML only (R-HTML) |
| An explicit PDF ask | 3: omega pdf (R-PDF), never a hand-rolled generator |
| A complex interactive app artifact (state, routing, shadcn/ui) | web-artifacts-builder skill (IF installed: local-only today, not shipped by install.sh), then publish its bundle.html via the Artifact tool |

Reading the table: surface 1 is the DEFAULT for any deliverable report where the operator
did not name a format. It is not exclusive: the artifact and its committed HTML twin are
the same file, so surface 1 always produces surface 2 as a side effect. Surfaces 2 and 3
fire only on an explicit signal (a file is wanted, a PDF is asked for). The fourth row is
an escalation, not a surface: when a report needs real interactivity beyond what one
hand-written page carries, build it with web-artifacts-builder first (if installed), then publish the
resulting bundle.html through the exact same protocol below.

## 2. Preconditions

Step 0, before anything else: confirm the `Artifact` tool exists in the CURRENT session's
tool list. If it is absent (headless run, cron, non-entitled account), fall back to
surface 2: deliver the self-contained HTML path, and SAY explicitly that the artifact
surface was unavailable and why. Never fabricate a URL. L1 applies: a live URL exists
only if a real publish call returned it.

## 3. Publish protocol

1. If the session lists a bundled artifact-design skill distinct from this file, load it
   via the Skill tool for design fundamentals. Otherwise this file's design contract
   (section 4) suffices.
2. Write CONTENT-ONLY HTML: no doctype, no `html`, `head`, or `body` tags (the Artifact
   tool wraps the file in that skeleton at publish time). Set a concise, stable `<title>`.
   One file per artifact: the same path republishes to the same URL, so keep the path
   stable across updates and pick a new path only for a genuinely new deliverable.
3. File location: `agentic/reports/<slug>.html` in OmegaOS-convention repos, otherwise
   the project's deliverable folder; the scratchpad only for throwaway pages. The
   committed file IS the offline twin (surface 2) of the published artifact.
4. Publish with the Artifact tool: pass the file_path, a STABLE one-emoji favicon, and a
   one-sentence description. Report BOTH the live URL and the file path to the operator.

A worked pass, end to end. Mission: a security audit report for project Foo.

```text
1. Session tool list contains Artifact          (precondition, section 2)
2. Write agentic/reports/foo-security-audit.html (content-only HTML, <title> set)
3. Artifact { file_path: ".../foo-security-audit.html",
              favicon: "🛡️",
              description: "Security audit of Foo: findings, PoCs, remediations" }
4. Tool returns the live URL
5. Report to the operator: the URL AND agentic/reports/foo-security-audit.html
6. A week later, findings re-verified: edit the SAME file, republish the SAME
   path, the SAME URL now serves the update (keep the same favicon)
```

Favicon discipline: the emoji is how the operator finds the tab, so it stays identical
across every republish of one artifact. Pick a new emoji only for a genuinely new
deliverable, never for an incremental update.

## 4. Design contract (self-sufficient digest)

- Self-contained under a strict CSP: inline all CSS and JS, no CDN, no external fonts,
  images, or fetch calls. Use system font stacks or @font-face data URIs only.
- Both themes, token-level. Define the palette as custom properties on `:root`; redefine
  the tokens under `@media (prefers-color-scheme: dark)`; then add
  `:root[data-theme="dark"]` and `:root[data-theme="light"]` overrides so the viewer's
  theme toggle wins in both directions. Style components through the tokens only, never
  through hardcoded colors. The cascade shape:

  ```css
  :root { --bg: ...; --ink: ...; --accent: ...; }
  @media (prefers-color-scheme: dark) { :root { --bg: ...; --ink: ...; } }
  :root[data-theme="dark"]  { --bg: ...; --ink: ...; }
  :root[data-theme="light"] { --bg: ...; --ink: ...; }
  ```

  The two `data-theme` blocks come LAST so the viewer's explicit toggle beats the OS
  preference in both directions.
- Typography: a real hierarchy, running text near 65ch, `text-wrap: balance` on headings,
  letter-spaced uppercase labels, and `font-variant-numeric: tabular-nums` wherever
  digits align (tables, stat tiles, timelines).
- Layout: flex or grid with `gap` (not margin stacks). Wide tables, code blocks, and
  diagrams scroll inside their own `overflow-x: auto` container; the page body never
  scrolls sideways; `max-width: 100%` on images.
- Craft: visible `:focus-visible` states, `prefers-reduced-motion` respected, a
  print-friendly `@media print` block for report pages, and a linked table of contents
  on long documents.
- Anti AI-slop, banned defaults: the warm-cream + serif + terracotta combo, the
  purple-to-blue gradient hero, a lone acid green on near-black, emoji section markers,
  everything centered, rounded-lg on everything, and bare Inter/Roboto/Arial as the
  page's voice. Choose a palette and type pairing specific to the subject instead.
- Premium or editorial surfaces: load high-end-visual-design (installed at
  ~/.omega/skills/high-end-visual-design) for the taste layer. Charts, dashboards, and
  stat tiles: load the dataviz skill by name IF the session lists it (it is
  harness-bundled and may be absent).
- Real content only: real numbers, real citations (R-CITE). Never lorem ipsum, never an
  invented metric.

## 5. The kill pass (R-NODASH)

Before publishing, strip every em dash (U+2014) and en dash (U+2013) from the visible
copy; replace each by meaning: comma, period, colon, or parentheses. Verify with:

```bash
grep -P '[\x{2013}\x{2014}]' <file>
```

The command must return nothing. Sole exception: verbatim quoted code samples.

## 6. Language

R-STYLE governs: write the artifact in the operator's language; French-only projects get
French artifacts. Code and identifiers stay English.

## 7. References (reuse, never duplicate: R-KARPATHY)

- `~/.omega/skills/web-artifacts-builder/SKILL.md` (if installed; not yet shipped by install.sh): multi-component React/Vite builds
  bundled into one self-contained bundle.html (it carries its own init and bundle
  scripts and its own verification checklist).
- `~/.omega/skills/high-end-visual-design/SKILL.md`: the premium taste engine and its
  reference canon.
- dataviz (harness skill, load by name only): chart form, the palette formula, mark and
  interaction rules.
- Rules R-ARTIFACT, R-HTML, R-PDF: the router doctrine this skill executes.
