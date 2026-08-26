//! Integration tests for `GET /v1/marketing` —
//! `crates/omega-gateway/src/routes_marketing.rs` (wave8 task C).
//!
//! `omega_core::marketing::list_marketing_projects()` scans TWO sources: (a)
//! the project registry (`omega_core::project_manager::ProjectRegistry`,
//! hardcoded to `dirs::home_dir()/.omega/projects.json` — NOT configurable
//! via `OMEGA_STATION_DIR`) and (b) a filesystem scan of `OMEGA_STATION_DIR`
//! itself. So every test here overrides BOTH `$HOME` (to neutralize the
//! operator's real registry, same pattern `telegram_test.rs`/`box_test.rs`
//! use for `$HOME`-hardcoded lookups) AND `OMEGA_STATION_DIR` (to scope the
//! filesystem scan to a scratch tempdir) -- never the operator's real
//! `~/Station` or `~/.omega/projects.json`.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// $HOME / OMEGA_STATION_DIR are process-global; serialize every test in this
// binary that touches either (same pattern as telegram_test.rs's LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

async fn app_and_token(gateway_dir: &std::path::Path) -> (axum::Router, String) {
    let (_, token) = DeviceStore::open(gateway_dir).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.to_path_buf(),
        GatewayConfig::default(),
    ));
    (app, token)
}

fn clear_env() {
    std::env::remove_var("HOME");
    std::env::remove_var("OMEGA_STATION_DIR");
}

/// Prepends a scratch dir holding a fake `crontab` script to `PATH`, so
/// `omega_core::marketing::read_crontab()`'s `crontab -l` subprocess call
/// (`marketing.rs`) resolves to a SCRIPT WE CONTROL instead of the real
/// operator's crontab. Review-fix Finding 3: without this, `engine_on`'s
/// value depends on whatever happens to be in the real crontab on whatever
/// machine runs the test (e.g. it only read `false` for the `alpha`/`zebra`
/// fixtures because the operator's real crontab didn't happen to contain a
/// matching `daily-engine` + slug substring) — a real cron change, or a
/// different machine, could silently flip the assertion. Returns the
/// PRE-MUTATION `PATH` so the caller can restore it via
/// [`restore_path`] at the end of the test.
///
/// `script_body` is the fake `crontab` binary's script body — e.g. `"exit
/// 1"` to simulate "no crontab for this user" (empty `read_crontab()`
/// output), or `"echo '<line>'"` for a controlled, fixed crontab listing.
fn install_fake_crontab(bin_dir: &std::path::Path, script_body: &str) -> String {
    use std::os::unix::fs::PermissionsExt;
    let path = bin_dir.join("crontab");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    let old_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), old_path));
    old_path
}

/// Restores `PATH` to the value [`install_fake_crontab`] captured before
/// mutating it.
fn restore_path(old_path: &str) {
    std::env::set_var("PATH", old_path);
}

/// Writes `content` to `dir/name`, padded past the source's 300-byte
/// "non-trivial" threshold (`layer_filled` in `marketing.rs`) so the write
/// unambiguously counts, regardless of the literal content length.
fn write_layer_file(dir: &std::path::Path, name: &str, content: &str) {
    std::fs::create_dir_all(dir).unwrap();
    let padded = format!("{content}\n{}", "x".repeat(400));
    std::fs::write(dir.join(name), padded).unwrap();
}

/// A fully "green" marketing project: every layer filled, a 3-post
/// calendar, and the daily-engine dir present -- the exact marker files
/// `marketing.rs`'s `build()` checks for each `has_*` flag.
fn write_full_marketing_project(station: &std::path::Path, dir_name: &str) {
    let root = station.join(dir_name);
    let marketing = root.join("marketing");
    write_layer_file(
        &marketing.join("00-context"),
        "product-marketing.md",
        "context",
    );
    write_layer_file(
        &marketing.join("01-strategy"),
        "gtm-strategy.md",
        "strategy",
    );
    write_layer_file(&marketing.join("02-copy"), "copywriting.md", "copy");
    write_layer_file(&marketing.join("03-visual-identity"), "DA.md", "visual");
    write_layer_file(
        &marketing.join("06-branding"),
        "SOCIAL-BRAND-BOOK.md",
        "branding",
    );
    std::fs::create_dir_all(marketing.join("04-publishing").join("daily-engine")).unwrap();
    let cal_dir = marketing.join("05-calendar");
    std::fs::create_dir_all(&cal_dir).unwrap();
    std::fs::write(
        cal_dir.join("calendar-90d.json"),
        r#"{"posts": ["p1", "p2", "p3"]}"#,
    )
    .unwrap();
}

/// A bare marketing-enabled project: just the `marketing/` dir itself, no
/// layer content -- every `has_*` flag should read false and
/// `calendar_posts` should read 0.
fn write_bare_marketing_project(station: &std::path::Path, dir_name: &str) {
    std::fs::create_dir_all(station.join(dir_name).join("marketing")).unwrap();
}

#[tokio::test]
async fn get_marketing_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/marketing"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn get_marketing_returns_empty_when_no_marketing_projects() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let station = tempfile::tempdir().unwrap();
    let cron_bin = tempfile::tempdir().unwrap();
    let old_path = install_fake_crontab(cron_bin.path(), "exit 1");
    std::env::set_var("HOME", home.path());
    std::env::set_var("OMEGA_STATION_DIR", station.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/marketing"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["projects"].as_array().unwrap().len(), 0);

    clear_env();
    restore_path(&old_path);
}

#[tokio::test]
async fn get_marketing_returns_full_status_flags_and_never_populates_accounts() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let station = tempfile::tempdir().unwrap();
    let cron_bin = tempfile::tempdir().unwrap();
    let old_path = install_fake_crontab(cron_bin.path(), "exit 1");
    write_full_marketing_project(station.path(), "Verba");
    std::env::set_var("HOME", home.path());
    std::env::set_var("OMEGA_STATION_DIR", station.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/marketing"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let projects = body["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 1);
    let p = &projects[0];

    assert_eq!(p["name"], "Verba");
    assert_eq!(p["slug"], "verba");
    assert_eq!(p["has_content"], true);
    assert_eq!(p["calendar_posts"], 3);
    assert_eq!(p["engine_on"], true);
    assert!(
        p["accounts"].is_null(),
        "accounts must never be populated by the list endpoint"
    );
    assert_eq!(p["accounts_tried"], false);
    assert_eq!(p["has_context"], true);
    assert_eq!(p["has_strategy"], true);
    assert_eq!(p["has_copy"], true);
    assert_eq!(p["has_visual"], true);
    assert_eq!(p["has_branding"], true);
    // `path` is deliberately never sent over the wire (server-internal, same
    // posture ProjectEntry already takes for /v1/projects).
    assert!(
        p.get("path").is_none(),
        "path must not be exposed on the wire"
    );

    clear_env();
    restore_path(&old_path);
}

#[tokio::test]
async fn get_marketing_returns_multiple_projects_name_sorted() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let station = tempfile::tempdir().unwrap();
    let cron_bin = tempfile::tempdir().unwrap();
    // Review-fix Finding 3: a deterministic fake crontab that never matches
    // "daily-engine" for either fixture, so `engine_on == false` for the
    // bare `alpha` fixture holds on every machine, not just one whose real
    // crontab happens not to mention it.
    let old_path = install_fake_crontab(cron_bin.path(), "exit 1");
    write_full_marketing_project(station.path(), "Zebra");
    write_bare_marketing_project(station.path(), "alpha");
    std::env::set_var("HOME", home.path());
    std::env::set_var("OMEGA_STATION_DIR", station.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/marketing"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let projects = body["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 2);

    // list_marketing_projects() sorts by name.to_lowercase(): "alpha" < "zebra".
    assert_eq!(projects[0]["name"], "alpha");
    assert_eq!(projects[0]["has_content"], false);
    assert_eq!(projects[0]["calendar_posts"], 0);
    assert_eq!(projects[0]["engine_on"], false);
    assert_eq!(projects[1]["name"], "Zebra");
    assert_eq!(projects[1]["has_content"], true);

    clear_env();
    restore_path(&old_path);
}

#[tokio::test]
async fn get_marketing_returns_504_when_crontab_hangs_past_the_timeout() {
    // Review-fix Finding 1: a hung / adversarially slow `crontab -l` on
    // `PATH` must not hang this endpoint's HTTP request forever. A fake
    // `crontab` that sleeps well past a short, test-only timeout (set via
    // `OMEGA_MARKETING_LIST_TIMEOUT_MS`, which `marketing_list_timeout()` in
    // `routes_marketing.rs` honors) must still make the endpoint answer in
    // bounded wall-clock time with a clear error status, not the full sleep
    // duration.
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let station = tempfile::tempdir().unwrap();
    let cron_bin = tempfile::tempdir().unwrap();
    let old_path = install_fake_crontab(cron_bin.path(), "sleep 3; exit 0");
    std::env::set_var("HOME", home.path());
    std::env::set_var("OMEGA_STATION_DIR", station.path());
    std::env::set_var("OMEGA_MARKETING_LIST_TIMEOUT_MS", "200");
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let started = std::time::Instant::now();
    let res = reqwest::Client::new()
        .get(format!("{base}/v1/marketing"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let elapsed = started.elapsed();

    assert_eq!(res.status(), 504);
    assert!(
        elapsed < std::time::Duration::from_secs(2),
        "endpoint should have answered well before the fake crontab's 3s sleep (timeout=200ms), took {elapsed:?}"
    );
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("timed out"),
        "expected a 'timed out' error message, got: {body}"
    );

    clear_env();
    restore_path(&old_path);
    std::env::remove_var("OMEGA_MARKETING_LIST_TIMEOUT_MS");
}

#[tokio::test]
async fn get_marketing_dedupes_when_registry_name_differs_from_directory_name() {
    // Review-fix Finding 2: `list_marketing_projects()` dedupes its two
    // sources (the project registry + the station-root filesystem scan) by
    // lowercased `name`, but `slug` is derived from the directory basename.
    // A registry entry whose display `name` differs from its directory name
    // survives that dedup and would otherwise appear as TWO response
    // entries sharing one `slug`. Reproduce it: a registry project named
    // "Verba Marketing" living in a directory literally named "verba" — the
    // registry-sourced entry (name "Verba Marketing", slug "verba") and the
    // filesystem-scan-sourced entry (name "verba", slug "verba") are
    // different by name and so both survive `list_marketing_projects()`'s
    // dedup, but must collapse to ONE entry in the gateway's response since
    // they are the same on-disk project (same canonicalized path).
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let station = tempfile::tempdir().unwrap();
    let cron_bin = tempfile::tempdir().unwrap();
    let old_path = install_fake_crontab(cron_bin.path(), "exit 1");
    write_bare_marketing_project(station.path(), "verba");
    std::env::set_var("HOME", home.path());
    std::env::set_var("OMEGA_STATION_DIR", station.path());

    // Registry entry pointing at the SAME directory but a DIFFERENT display
    // name, so upstream's name-keyed dedup does not catch it.
    let registry_path = home.path().join(".omega").join("projects.json");
    std::fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    let project_path = station.path().join("verba");
    std::fs::write(
        &registry_path,
        serde_json::json!({
            "projects": [
                {
                    "name": "Verba Marketing",
                    "path": project_path.to_string_lossy(),
                    "telegram_topic_id": null,
                    "oracle_session": null,
                    "git_email": null,
                    "created_at": ""
                }
            ]
        })
        .to_string(),
    )
    .unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/marketing"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let projects = body["projects"].as_array().unwrap();
    assert_eq!(
        projects.len(),
        1,
        "same on-disk project registered under two names must collapse to one response entry, got: {body}"
    );

    clear_env();
    restore_path(&old_path);
}
