//! Marketing projects — discovery + fast filesystem status for the Marketing
//! tab (TUI), the `omega marketing` CLI, and the Telegram `nav:marketing` hub.
//!
//! A project is "marketing-enabled" when it has a `<path>/marketing/` directory
//! (created by the marketing machine + zernio + higgsfield pipeline). This
//! module answers TWO questions cheaply, with NO network:
//!   1. which projects have marketing? (`list_marketing_projects`)
//!   2. what's their at-a-glance status? (content ✓, calendar post count,
//!      daily-engine on/off)
//!
//! Connected-account counts touch `omega-zernio` and are therefore ON-DEMAND
//! only (`project_accounts`) — never fetched per-frame.
//!

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A project with a `marketing/` directory, plus fast filesystem status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketingProject {
    /// Display name (the registry name, else the directory name).
    pub name: String,
    /// Slug = directory name lowercased — the id zernio/higgsfield use.
    pub slug: String,
    /// Absolute project root (the parent of `marketing/`).
    pub path: PathBuf,
    /// `marketing/05-calendar/calendar-90d.json` (or `calendar.json`) exists.
    pub has_content: bool,
    /// Total posts counted across days in the calendar json (0 on any failure).
    pub calendar_posts: usize,
    /// The daily-publishing engine is wired (cron line present OR the
    /// `marketing/04-publishing/daily-engine/` dir exists).
    pub engine_on: bool,
    /// Connected social accounts — `None` in the list (populated on-demand only
    /// via `project_accounts`, never per-frame).
    pub accounts: Option<usize>,
    /// Whether a lookup has already been ATTEMPTED for this project.
    ///
    /// `accounts` alone cannot express "we asked and it failed": it stays
    /// `None`, so a zernio that is absent or erroring (e.g. HTTP 402) was
    /// re-shelled on every single cursor move, forever — the Marketing tab's
    /// navigation lag. A refresh rebuilds this vec, which resets the flag, so
    /// F5 still retries.
    pub accounts_tried: bool,
    /// 00-context filled (product-marketing.md present + non-trivial).
    pub has_context: bool,
    /// 01-strategy filled (gtm-strategy.md or content-strategy.md).
    pub has_strategy: bool,
    /// 02-copy filled (copywriting.md or social-content.md).
    pub has_copy: bool,
    /// 03-visual-identity filled (DA.md).
    pub has_visual: bool,
    /// 06-branding filled (SOCIAL-BRAND-BOOK.md or tokens.json).
    pub has_branding: bool,
}

/// The projects root to scan for `marketing/` subdirs. Configurable via
/// `OMEGA_STATION_DIR`; defaults to `~/Station/SideBusiness` (the operator's
/// station root), falling back to `~/Station` then `$HOME`.
fn station_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_STATION_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
    let side = home.join("Station").join("SideBusiness");
    if side.is_dir() {
        return side;
    }
    let station = home.join("Station");
    if station.is_dir() {
        return station;
    }
    home
}

/// True if `<path>/marketing/` is a directory.
fn has_marketing_dir(path: &Path) -> bool {
    path.join("marketing").is_dir()
}

/// The calendar json for a project, if either canonical name exists.
/// Prefers `calendar-90d.json`, then `calendar.json`.
fn calendar_json_path(marketing: &Path) -> Option<PathBuf> {
    let cal_dir = marketing.join("05-calendar");
    for name in ["calendar-90d.json", "calendar.json"] {
        let p = cal_dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Best-effort count of posts across days in a calendar json. Returns 0 on any
/// parse failure. Handles the common shapes:
///   { "days": [ { "posts": [...] }, ... ] }
///   { "<date>": { "posts": [...] }, ... }  (map of days)
///   [ { "posts": [...] }, ... ]            (array of days)
///   { "posts": [...] }                     (flat)
fn count_calendar_posts(json_path: &Path) -> usize {
    let Ok(raw) = std::fs::read_to_string(json_path) else {
        return 0;
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return 0;
    };

    fn posts_in(day: &serde_json::Value) -> usize {
        match day.get("posts") {
            Some(serde_json::Value::Array(a)) => a.len(),
            _ => 0,
        }
    }

    match &val {
        // Flat: a single object carrying "posts".
        serde_json::Value::Object(map) if map.contains_key("posts") => posts_in(&val),
        // { "days": [ ... ] }
        serde_json::Value::Object(map)
            if map.get("days").map(|d| d.is_array()).unwrap_or(false) =>
        {
            map["days"]
                .as_array()
                .map(|days| days.iter().map(posts_in).sum())
                .unwrap_or(0)
        }
        // Map of days → each value is a day object.
        serde_json::Value::Object(map) => map.values().map(posts_in).sum(),
        // Array of day objects.
        serde_json::Value::Array(days) => days.iter().map(posts_in).sum(),
        _ => 0,
    }
}

/// Whether the daily-publishing engine is wired for this project. Fast + local:
///   • the `marketing/04-publishing/daily-engine/` dir exists, OR
///   • a `crontab -l` line mentions this project's daily-engine (matched on the
///     slug + "daily-engine"). The crontab is read ONCE per call.
fn engine_on(marketing: &Path, slug: &str, crontab: &str) -> bool {
    if marketing
        .join("04-publishing")
        .join("daily-engine")
        .is_dir()
    {
        return true;
    }
    let needle = slug.to_lowercase();
    crontab.lines().any(|line| {
        let l = line.to_lowercase();
        l.contains("daily-engine") && (l.contains(&needle) || l.contains(&format!("/{needle}/")))
    })
}

/// Read `crontab -l` once (empty string on any failure / no crontab).
fn read_crontab() -> String {
    std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// Build a `MarketingProject` from a discovered root. `crontab` is passed in so
/// the (potentially slow) `crontab -l` is read once for the whole list.
fn build(name: String, path: PathBuf, crontab: &str) -> MarketingProject {
    let slug = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| name.to_lowercase());
    let marketing = path.join("marketing");
    let cal = calendar_json_path(&marketing);
    let has_content = cal.is_some();
    let calendar_posts = cal.as_deref().map(count_calendar_posts).unwrap_or(0);
    let engine_on = engine_on(&marketing, &slug, crontab);
    MarketingProject {
        name,
        slug,
        path,
        has_content,
        calendar_posts,
        engine_on,
        accounts: None,
        accounts_tried: false,
        has_context: layer_filled(&marketing, "00-context", &["product-marketing.md"]),
        has_strategy: layer_filled(
            &marketing,
            "01-strategy",
            &["gtm-strategy.md", "content-strategy.md"],
        ),
        has_copy: layer_filled(
            &marketing,
            "02-copy",
            &["copywriting.md", "social-content.md"],
        ),
        has_visual: layer_filled(&marketing, "03-visual-identity", &["DA.md"]),
        has_branding: layer_filled(
            &marketing,
            "06-branding",
            &["SOCIAL-BRAND-BOOK.md", "tokens.json"],
        ),
    }
}

/// A marketing layer is "filled" if any of the named files exists in the given
/// subdir AND is non-trivial (>300 bytes — past the scaffold skeleton). Fast + local.
fn layer_filled(marketing: &Path, sub: &str, files: &[&str]) -> bool {
    let dir = marketing.join(sub);
    files.iter().any(|f| {
        std::fs::metadata(dir.join(f))
            .map(|m| m.is_file() && m.len() > 300)
            .unwrap_or(false)
    })
}

/// List marketing-enabled projects: the UNION of the project registry and a
/// filesystem scan of the station root, deduped by name (case-insensitive).
/// Fast, local-only — safe to call on tab entry / F5.
pub fn list_marketing_projects() -> Vec<MarketingProject> {
    let crontab = read_crontab();
    let mut out: Vec<MarketingProject> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut push = |name: String, path: PathBuf| {
        let key = name.to_lowercase();
        if seen.insert(key) {
            out.push(build(name, path, &crontab));
        }
    };

    // (a) Registry projects whose path/marketing/ exists.
    let registry = crate::project_manager::ProjectRegistry::load();
    for p in registry.projects {
        if has_marketing_dir(&p.path) {
            push(p.name, p.path);
        }
    }

    // (b) Filesystem scan: immediate subdirs of the station root with marketing/.
    let root = station_dir();
    if let Ok(entries) = std::fs::read_dir(&root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && has_marketing_dir(&path) {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if !name.is_empty() {
                    push(name, path);
                }
            }
        }
    }

    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

/// Connected-account count for a project — shells out to `omega-zernio`.
/// ON-DEMAND ONLY (detail pane / CLI), never per-frame. Bounded so a hung
/// zernio can't stall the UI: a background thread with a short join timeout.
/// Returns `None` if zernio is absent, errors, times out, or yields nothing.
pub fn project_accounts(slug: &str) -> Option<usize> {
    let slug = slug.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        // Prefer JSON (count array/object entries); fall back to line count.
        let json = std::process::Command::new("omega-zernio")
            .args(["accounts", &slug, "--json"])
            .output();
        // The plain-text fallback exists for a zernio that PRINTS non-JSON, not
        // for a zernio that FAILS: when the command itself exits non-zero (not
        // installed, HTTP 402/401, network down) re-running the identical
        // command without --json cannot succeed — it just doubles the cost of
        // every lookup. So only fall back when the JSON call ran fine and the
        // output merely wasn't parseable.
        let count = match json {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout);
                parse_accounts_json(&text).or_else(|| {
                    let plain = std::process::Command::new("omega-zernio")
                        .args(["accounts", &slug])
                        .output()
                        .ok()?;
                    if !plain.status.success() {
                        return None;
                    }
                    let text = String::from_utf8_lossy(&plain.stdout);
                    Some(text.lines().filter(|l| !l.trim().is_empty()).count())
                })
            }
            _ => None,
        };
        let _ = tx.send(count);
    });
    rx.recv_timeout(std::time::Duration::from_secs(4))
        .ok()
        .flatten()
}

/// Count entries in a zernio `accounts --json` payload. Accepts an array, an
/// object with an `accounts` array, or a bare object (keyed by account).
fn parse_accounts_json(text: &str) -> Option<usize> {
    let val = serde_json::from_str::<serde_json::Value>(text.trim()).ok()?;
    match val {
        serde_json::Value::Array(a) => Some(a.len()),
        serde_json::Value::Object(ref map) => {
            if let Some(serde_json::Value::Array(a)) = map.get("accounts") {
                Some(a.len())
            } else {
                Some(map.len())
            }
        }
        _ => None,
    }
}

// ===========================================================================
// Capabilities registry — the anti-forgetting SSOT (capabilities.toml)
// ===========================================================================

/// One capability group (the inventory's spine).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGroup {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub order: u32,
}

/// One capability row — mirrors a [[capability]] block in capabilities.toml.
/// All non-id fields default so a partial/older manifest still parses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub does: String,
    #[serde(default)]
    pub run: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub layer: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub deps: Vec<String>,
    #[serde(default)]
    pub paid: bool,
    #[serde(default)]
    pub skill: String,
    #[serde(default)]
    pub notes: String,
}

/// The whole parsed capabilities.toml manifest.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilitiesRegistry {
    #[serde(default)]
    pub schema: String,
    #[serde(default, rename = "group")]
    pub groups: Vec<CapabilityGroup>,
    #[serde(default, rename = "capability")]
    pub capabilities: Vec<Capability>,
}

impl CapabilitiesRegistry {
    /// Capabilities for a group id, in file order.
    pub fn in_group<'a>(&'a self, group_id: &str) -> Vec<&'a Capability> {
        self.capabilities
            .iter()
            .filter(|c| c.group == group_id)
            .collect()
    }
    /// Groups sorted by their `order` field (then name).
    pub fn groups_ordered(&self) -> Vec<&CapabilityGroup> {
        let mut g: Vec<&CapabilityGroup> = self.groups.iter().collect();
        g.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.name.cmp(&b.name)));
        g
    }
    /// Count by status across all capabilities.
    pub fn status_counts(&self) -> (usize, usize, usize) {
        let mut built = 0;
        let mut partial = 0;
        let mut missing = 0;
        for c in &self.capabilities {
            match c.status.as_str() {
                "built" => built += 1,
                "partial" => partial += 1,
                "missing" => missing += 1,
                _ => {}
            }
        }
        (built, partial, missing)
    }
}

/// Locate `capabilities.toml`. Order: `OMEGA_MKT_CAPS` env override, then the
/// repo path relative to the running exe (…/OmegaOS/tools/marketing-machine/),
/// then a scan up from the current dir, then a couple of well-known checkouts.
pub fn capabilities_toml_path() -> Option<PathBuf> {
    // 1. Explicit override.
    if let Ok(p) = std::env::var("OMEGA_MKT_CAPS") {
        let p = p.trim();
        if !p.is_empty() {
            let pb = PathBuf::from(p);
            if pb.is_file() {
                return Some(pb);
            }
        }
    }
    let rel = Path::new("tools")
        .join("marketing-machine")
        .join("capabilities.toml");

    // 2. Relative to the running executable: …/OmegaOS/target/<profile>/omega.
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        while let Some(d) = dir {
            let cand = d.join(&rel);
            if cand.is_file() {
                return Some(cand);
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }

    // 3. Walk up from the current working dir.
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = Some(cwd);
        while let Some(d) = dir {
            let cand = d.join(&rel);
            if cand.is_file() {
                return Some(cand);
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }

    // 4. Well-known checkouts.
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
    for base in [
        home.join("Station").join("SideBusiness").join("OmegaOS"),
        home.join("OmegaOS"),
    ] {
        let cand = base.join(&rel);
        if cand.is_file() {
            return Some(cand);
        }
    }

    // 5. The INSTALLED copy — last, so a checkout always wins (an operator
    // editing the registry sees it immediately, without reinstalling). This is
    // what makes the registry work on a machine that has no checkout at all, or
    // whose checkout lives somewhere step 4 never guesses: install.sh drops the
    // whole marketing-machine payload here.
    let omega_dir = std::env::var("OMEGA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".omega"));
    let installed = omega_dir
        .join("marketing-machine")
        .join("capabilities.toml");
    if installed.is_file() {
        return Some(installed);
    }
    None
}

/// Load + parse the capabilities registry. Returns `Ok(None)` if the manifest
/// is absent (graceful degradation), `Err` only on a real parse failure.
pub fn load_capabilities() -> anyhow::Result<Option<CapabilitiesRegistry>> {
    let Some(path) = capabilities_toml_path() else {
        return Ok(None);
    };
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("reading {}: {e}", path.display()))?;
    let reg: CapabilitiesRegistry =
        toml::from_str(&raw).map_err(|e| anyhow::anyhow!("parsing {}: {e}", path.display()))?;
    Ok(Some(reg))
}

/// A per-layer, per-project view for `omega marketing status`: which capability
/// groups are built / partial / missing FOR THIS PROJECT, cross-referencing the
/// filesystem status against the registry.
#[derive(Debug, Clone, Serialize)]
pub struct GroupStatus {
    pub group: String,
    pub name: String,
    /// Whether this project has the artifact(s) this group represents.
    pub present: bool,
    /// Human note (e.g. which file drives the flag).
    pub detail: String,
}

/// Cross-reference a project's filesystem layers against the group inventory.
pub fn project_group_status(p: &MarketingProject) -> Vec<GroupStatus> {
    let g = |group: &str, name: &str, present: bool, detail: &str| GroupStatus {
        group: group.to_string(),
        name: name.to_string(),
        present,
        detail: detail.to_string(),
    };
    vec![
        g(
            "research-strategy",
            "Research & Strategy",
            p.has_context || p.has_strategy,
            if p.has_context {
                "00-context filled"
            } else {
                "00-context/product-marketing.md missing"
            },
        ),
        g(
            "copy",
            "Copy",
            p.has_copy,
            if p.has_copy {
                "02-copy filled"
            } else {
                "02-copy (copywriting/social-content) missing"
            },
        ),
        g(
            "branding",
            "Branding",
            p.has_branding,
            if p.has_branding {
                "06-branding (tokens/brand-book) present"
            } else {
                "06-branding missing"
            },
        ),
        g(
            "visual-image",
            "Visual — Image",
            p.has_visual,
            if p.has_visual {
                "03-visual-identity/DA.md present"
            } else {
                "03-visual-identity/DA.md missing"
            },
        ),
        g(
            "calendar",
            "Calendar",
            p.has_content,
            if p.calendar_posts > 0 {
                "calendar-90d.json with posts"
            } else if p.has_content {
                "calendar present (0 posts)"
            } else {
                "no calendar"
            },
        ),
        g(
            "publishing",
            "Publishing",
            p.engine_on,
            if p.engine_on {
                "daily-engine wired"
            } else {
                "no daily-engine"
            },
        ),
    ]
}

/// The next-best-action for a project, from a deterministic top-down rule list
/// over its filesystem status. Returns `(id, why, command)`.
pub fn next_best_action(p: &MarketingProject) -> (String, String, String) {
    let mk = |id: &str, why: &str, cmd: &str| (id.to_string(), why.to_string(), cmd.to_string());

    if !p.has_context {
        return mk(
            "product-marketing-context",
            "No 00-context/product-marketing.md — the SSOT every other layer reads.",
            "/omg-product-marketing-context",
        );
    }
    if !p.has_strategy {
        return mk(
            "content-strategy",
            "Context is set but no 01-strategy — pillars/clusters drive the calendar.",
            "/omg-content-strategy",
        );
    }
    if !p.has_branding {
        return mk(
            "brand-setup",
            "No 06-branding — tokens.json/SOCIAL-BRAND-BOOK.md gate every generation.",
            "/omg-brand-identity  (then compile 06-branding: tokens.json + SOCIAL-BRAND-BOOK.md)",
        );
    }
    if !p.has_copy {
        return mk(
            "copy",
            "Branding is set but 02-copy is empty — no posts/hooks to schedule.",
            "/omg-social-content",
        );
    }
    if !p.has_content || p.calendar_posts == 0 {
        return mk(
            "calendar",
            "Copy exists but the 90-day calendar has no posts to run.",
            "/omg-content-strategy  (produce 05-calendar/calendar-90d.json)",
        );
    }
    // Content is ready. Are accounts connected? `Some(0)` = definitely none;
    // `None` = unknown (zernio absent / timed out) — don't force a connect step
    // on an unknown, especially when the engine is already wired.
    if matches!(p.accounts, Some(0)) {
        return mk(
            "connect-accounts",
            "Content is ready but no social account is connected — nothing can publish.",
            &format!("omega-zernio connect {} <platform>", p.slug),
        );
    }
    if p.accounts.is_none() && !p.engine_on {
        return mk(
            "connect-accounts",
            "Content is ready but no account is confirmed connected — connect one to publish.",
            &format!("omega-zernio connect {} <platform>", p.slug),
        );
    }
    if !p.engine_on {
        return mk(
            "activate-engine",
            "Connected with content, but the daily engine is off — activate it to publish.",
            &format!(
                "omega marketing run {} --dry-run   (then --publish / cron)",
                p.slug
            ),
        );
    }
    mk(
        "publish",
        "Fully wired (content + accounts + engine). Run the day and publish.",
        &format!("omega marketing run {} --publish", p.slug),
    )
}

/// Convenience paths surfaced in the detail pane.
impl MarketingProject {
    pub fn marketing_dir(&self) -> PathBuf {
        self.path.join("marketing")
    }
    /// Human-readable path to the 90-day calendar markdown (may not exist).
    pub fn calendar_md(&self) -> PathBuf {
        self.marketing_dir()
            .join("05-calendar")
            .join("calendar-90d.md")
    }
    /// Status glyph: 🟢 content ✓ + engine on / 🟡 content ✓ engine off / ⚪ empty.
    pub fn glyph(&self) -> &'static str {
        match (self.has_content, self.engine_on) {
            (true, true) => "🟢",
            (true, false) => "🟡",
            (false, _) => "⚪",
        }
    }
}
