# New Project — redesign spec (cross-user paths + multi-stack + launch prompt)

Status: DESIGN (read-only workflow `new-project-redesign-spec`, 4 Explore agents).
Author: synthesized by the cleanup/doctor session. Implementation is BLOCKED on
R-SCOPE coordination — the new-project feature is actively developed by a
concurrent `oracle-OmegaOS` session that owns app.rs / input.rs / main.rs /
omega-new-project.md. Do NOT implement in parallel; one writer only.

---

## 1. Cross-user work directory (kill the `~/VibeCoding` hardcode)

**Today** (8 hardcoded locations): app.rs:96-97 (menu labels), main.rs:1098-1099
(category routing), telegram_bridge.rs:1663/1709/2041 (project scans), plus 4
`/home/hacker` fallbacks (main.rs:1096, provisioning.rs:20, telegram_bridge.rs:2771/2839).
Only valid on the maintainer's VPS — breaks Law L0 for any other user.

**Key existing capability:** `crates/omega-core/src/projects.rs::discover()` ALREADY
scans `$HOME` (depth 2) for container dirs — CONTAINER_NAMES includes work, works,
clients, code, dev, repos, src, vibecoding, etc. So "an agent that finds the user's
work folders" is already built; we just wire it as the default.

**Design:**
- Add `OmegaConfig.projects_dir: PathBuf` (persisted in `~/.omega/config.toml`,
  `#[serde(default)]` for back-compat).
- Default resolution order: (1) existing config value, (2) `projects::discover()`
  best match (the user's real work container — finds `~/VibeCoding` for the
  maintainer, `~/projects`/`~/code`/… for others), (3) fallback `~/projects`.
- Add `OmegaConfig::resolve_category_path(category) -> PathBuf`
  (`works`→`projects_dir/work`, `client`→`projects_dir/clients`, else `projects_dir/<category>`).
- `NEW_PROJECT_CATEGORIES`: drop the hardcoded `~/VibeCoding` label strings →
  `&[&str]` ids; the picker renders `"{display} → {resolved path}"` from config.
- main.rs CreateProject: replace the hardcoded match with `resolve_category_path`.
- telegram_bridge scans: iterate config categories, not literal `VibeCoding/*`.
- Remove the 4 `/home/hacker` fallbacks → resolve home properly or error
  (`dirs::home_dir()` → `$HOME` → clear error). Never assume `hacker`.
- install.sh: seed `~/.omega/config.toml` with a commented `projects_dir`.

Files: config.rs, app.rs, input.rs, ui.rs, main.rs, telegram_bridge.rs,
provisioning.rs, .claude/commands/omega-new-project.md, install.sh.

---

## 2. Multi-stack catalog (by project type, R-STACK aligned)

**Today:** one stack — `nextstack` (Next.js 16 + Convex + Clerk + Stripe).

**Catalog** (NEW_PROJECT_STACKS → `(id, label, services)`):

| id | type | toolchain | services |
|---|---|---|---|
| `nextstack-saas` | SaaS product | Next.js 16 + Convex + Clerk + Stripe + shadcn | Vercel, Convex, Clerk, Stripe, GitHub |
| `nextstack-content` | content/multi-user | Next.js 16 + Convex (+Clerk opt) | Vercel, Convex, GitHub |
| `nextstack-static` | marketing/landing/docs | Next.js 16 static export, no backend | Vercel, GitHub |
| `rust-cli` | CLI / daemon / internal | Rust + clap + tokio | GitHub |
| `bun-script` | script / tooling / DOM | Bun + TypeScript | GitHub |
| `expo-mobile` | iOS/Android | Expo + RN + NativeWind + EAS | GitHub, EAS (+opt Clerk/Stripe) |

Keep `nextstack` as an alias for `nextstack-saas` so existing flows don't break.
`/omega-new-project` Phase 2/3/4/5 become conditional on stack id (skip Convex/
Clerk/Stripe for static; cargo/bun scaffold for rust/bun; skip /vision-/prd for
tools). This is faithful to R-STACK (rules.rs) — Rust internals, Bun tooling,
Next.js clients.

Files: app.rs (NEW_PROJECT_STACKS), .claude/commands/omega-new-project.md (6
scaffold blocks + conditional provisioning).

---

## 3. Launch prompt + docs (seed the project at creation)

**Today:** wizard spawns `/omega-new-project <stack> <category> <name>` with NO prompt
(main.rs:1103).

**Design (append-to-prompt — consistent with how oracle/worker prompts pass):**
- Two new OPTIONAL wizard steps after stack pick:
  - `InputMode::NewProjectLaunchPrompt(name, category, stack)` — kickoff idea.
  - `InputMode::NewProjectLaunchDocs(name, category, stack, kickoff)` — doc paths.
  - Esc at either step skips gracefully.
- `Action::CreateProject` gains `launch_prompt: Option<String>` + `launch_docs: Option<String>`.
- main.rs appends to the spawned prompt under `--- PROJECT KICKOFF BRIEF ---` /
  `--- REFERENCED DOCS ---` markers.
- Enhancement over the raw agent proposal: for doc paths, READ small files
  (<~10KB total) inline into the brief; reference larger ones by path. So "j'ai
  des docs que l'on peut directement utiliser" actually feeds the content in.

Files: app.rs (2 InputMode variants), input.rs (Action fields + 2 handlers),
main.rs (CreateProject prompt build), .claude/commands/omega-new-project.md (doc string).

---

## Implementation order (one writer, after R-SCOPE coordination)
1. config.rs `projects_dir` + `resolve_category_path` + `projects::discover` default.
2. Rewire app.rs/input.rs/ui.rs/main.rs/telegram_bridge.rs to config (kill hardcode).
3. NEW_PROJECT_STACKS 6-catalog + omega-new-project.md conditional scaffold.
4. Launch-prompt steps + CreateProject fields + prompt append.
5. install.sh config seed; verify-install; cross-user sandbox (fake `$HOME`, non-hacker).
