//! Integration tests for `GET /v1/doctor`, `GET /v1/usage`, `GET
//! /v1/box-info`, `POST /v1/backup` — `crates/omega-gateway/src/routes_box.rs`
//! (wave6 task C).
//!
//! SAFETY (R-SEC / this task's explicit brief): every test that could reach a
//! real subprocess spawn sets `OMEGA_BIN` to a FAKE script FIRST, and every
//! backup test points `OMEGA_BACKUP_DIR` at a tempdir — never the operator's
//! real `~/.omega` or `~/omega-backup-*.tgz`. `usage`'s tests override `HOME`
//! (the env var `dirs::home_dir()` reads on Linux), since
//! `UsageSnapshot::read()` does NOT honor `$OMEGA_HOME`/`$OMEGA_STATE_DIR`.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_BIN / HOME / OMEGA_BACKUP_DIR are process-global env vars mutated by
// several tests below; serialize every test in this binary that touches any
// of them (same pattern as audit_test.rs's / dispatch_test.rs's LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// Writes an executable fake `omega` script and points `OMEGA_BIN` at it —
/// same idiom as `audit_test.rs::install_fake_omega`.
fn install_fake_omega(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("omega");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

fn clear_env() {
    std::env::remove_var("OMEGA_BIN");
    std::env::remove_var("HOME");
    std::env::remove_var("OMEGA_BACKUP_DIR");
}

// ── GET /v1/doctor ───────────────────────────────────────────────────────

/// A realistic multi-check fake `omega doctor` stdout, INCLUDING a check
/// name >16 characters ("binary provenance", 18 chars) — the exact case
/// where the naive `{:16}`-column-split assumption would misparse (there is
/// only ONE literal space between "provenance" and "built", not a fixed
/// column boundary). Also covers `[+]` (ok), `[!]` (warn), and `[x]` (fail).
const DOCTOR_STDOUT_MIXED: &str = "OmegaOS doctor\n\n  [+] daemon           running fine\n  [+] binary provenance built from HEAD (abc1234)\n  [!] telegram          poller stale (12m)\n  [x] disk              97% full\n\n[x] problems detected — see [x] lines above\n";

#[tokio::test]
async fn doctor_parses_mixed_checks_without_fixed_width_assumption() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega(
        bin_dir.path(),
        &format!("cat <<'EOF'\n{DOCTOR_STDOUT_MIXED}EOF\nexit 1"),
    );
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/doctor"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    // omega doctor exits 1 on overall Fail -- that is NOT treated as a spawn
    // error, so the endpoint still answers 200 with the parsed body.
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();

    assert_eq!(
        body["overall"], "fail",
        "aggregated from the parsed checks, not the trailing summary line"
    );
    let checks = body["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 4);

    assert_eq!(checks[0]["health"], "ok");
    assert_eq!(checks[0]["text"], "daemon           running fine");

    // The >16-char-name line: the FULL text (name + detail merged, single
    // space between them since the name already exceeds the {:16} minimum)
    // must survive intact -- a fixed-width split would have cut this wrong.
    assert_eq!(checks[1]["health"], "ok");
    assert_eq!(
        checks[1]["text"],
        "binary provenance built from HEAD (abc1234)"
    );

    assert_eq!(checks[2]["health"], "warn");
    assert_eq!(checks[2]["text"], "telegram          poller stale (12m)");

    assert_eq!(checks[3]["health"], "fail");
    assert_eq!(checks[3]["text"], "disk              97% full");

    clear_env();
}

#[tokio::test]
async fn doctor_all_ok_reports_overall_ok() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega(
        bin_dir.path(),
        "cat <<'EOF'\nOmegaOS doctor\n\n  [+] daemon running\n\n[+] all systems healthy\nEOF\nexit 0",
    );
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/doctor"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["overall"], "ok");
    assert_eq!(body["checks"].as_array().unwrap().len(), 1);

    clear_env();
}

#[tokio::test]
async fn doctor_warn_only_reports_overall_warn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega(
        bin_dir.path(),
        "cat <<'EOF'\nOmegaOS doctor\n\n  [+] daemon running\n  [!] telegram stale\n\n[!] healthy, with warnings above\nEOF\nexit 0",
    );
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/doctor"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["overall"], "warn");

    clear_env();
}

#[tokio::test]
async fn doctor_exit_code_1_on_fail_is_not_treated_as_an_error() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    // Real `omega doctor` calls std::process::exit(1) on overall Fail --
    // this fake mirrors that exactly.
    install_fake_omega(
        bin_dir.path(),
        "cat <<'EOF'\nOmegaOS doctor\n\n  [x] disk full\n\n[x] problems detected — see [x] lines above\nEOF\nexit 1",
    );
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/doctor"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        200,
        "a non-zero exit code alone must never become a 502"
    );
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["overall"], "fail");

    clear_env();
}

#[tokio::test]
async fn doctor_requires_auth() {
    let dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/doctor"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

// ── GET /v1/usage ────────────────────────────────────────────────────────

#[tokio::test]
async fn usage_reports_available_with_a_real_snapshot() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(fake_home.path().join(".omega/state")).unwrap();
    std::fs::write(
        fake_home.path().join(".omega/state/usage.json"),
        serde_json::json!({
            "session_pct": 42,
            "week_pct": 17,
            "sonnet_pct": 5,
            "extra_pct": 0,
            "tokens_5h": 12345,
            "tokens_7d": 67890,
            "account": "x@agentik-os.com",
            "ts": "2026-08-10T00:00:00Z"
        })
        .to_string(),
    )
    .unwrap();
    std::env::set_var("HOME", fake_home.path());

    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/usage"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["available"], true);
    assert_eq!(body["session_pct"], 42);
    assert_eq!(body["week_pct"], 17);
    assert_eq!(body["ts"], "2026-08-10T00:00:00Z");

    clear_env();
}

#[tokio::test]
async fn usage_reports_unavailable_when_cache_file_absent() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", fake_home.path());

    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/usage"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        200,
        "no cache yet is a normal state, never an error"
    );
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["available"], false);
    assert!(body.get("session_pct").is_none() || body["session_pct"].is_null());

    clear_env();
}

#[tokio::test]
async fn usage_requires_auth() {
    let dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/usage"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

// ── GET /v1/box-info ─────────────────────────────────────────────────────

#[tokio::test]
async fn box_info_reports_hostname_version_and_small_nonnegative_uptime() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega(bin_dir.path(), "echo 'omega 0.1.9'");
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/box-info"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(!body["hostname"].as_str().unwrap().is_empty());
    assert_eq!(body["omega_version"], "omega 0.1.9");
    assert!(!body["gateway_version"].as_str().unwrap().is_empty());
    // The request runs shortly after AppState::new() -- uptime must be a
    // small, non-negative number of seconds.
    let uptime = body["uptime_secs"].as_u64().unwrap();
    assert!(
        uptime < 30,
        "uptime {uptime}s is implausibly large for a fresh test process"
    );

    clear_env();
}

#[tokio::test]
async fn box_info_requires_auth() {
    let dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/box-info"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

// ── POST /v1/backup ──────────────────────────────────────────────────────

#[tokio::test]
async fn backup_happy_path_parses_archive_path_and_size() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let backup_scratch = tempfile::tempdir().unwrap();
    let capture = bin_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &format!(
            r#"
printf '%s\n' "$@" > '{capture}'
echo "OmegaOS backup"
echo ""
echo "  archive : /fake/omega-backup-20260810.tgz"
echo "  size    : 12.3 MB"
echo "  included: home"
exit 0
"#,
            capture = capture.display()
        ),
    );
    // ALWAYS a test-controlled scratch path -- never the operator's real home.
    std::env::set_var("OMEGA_BACKUP_DIR", backup_scratch.path());

    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/backup"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["path"], "/fake/omega-backup-20260810.tgz");
    assert_eq!(body["size"], "12.3 MB");

    // The server-chosen --out argument must land inside the scratch dir the
    // test controls, never a real home directory.
    let argv = std::fs::read_to_string(&capture).unwrap();
    assert!(argv.contains("backup\n"));
    assert!(argv.contains("--out\n"));
    assert!(
        argv.contains(backup_scratch.path().to_str().unwrap()),
        "expected --out under the OMEGA_BACKUP_DIR scratch dir, got: {argv}"
    );

    clear_env();
}

#[tokio::test]
async fn backup_nonzero_exit_is_a_502() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let backup_scratch = tempfile::tempdir().unwrap();
    install_fake_omega(bin_dir.path(), "echo 'disk full' >&2; exit 1");
    std::env::set_var("OMEGA_BACKUP_DIR", backup_scratch.path());

    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/backup"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        502,
        "a failed backup is a real error, unlike doctor's expected non-zero exit"
    );

    clear_env();
}

#[tokio::test]
async fn backup_requires_auth() {
    let dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/backup"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

// ── GET /v1/box-id ───────────────────────────────────────────────────────

#[tokio::test]
async fn box_id_requires_auth() {
    let dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/box-id"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn box_id_is_32_hex_chars_and_stable_across_repeated_calls() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res1 = reqwest::Client::new()
        .get(format!("{base}/v1/box-id"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res1.status(), 200);
    let body1: serde_json::Value = res1.json().await.unwrap();
    let id1 = body1["box_id"].as_str().unwrap().to_string();
    assert_eq!(id1.len(), 32, "box_id must be 32 hex chars");
    assert!(id1.chars().all(|c| c.is_ascii_hexdigit()));

    let res2 = reqwest::Client::new()
        .get(format!("{base}/v1/box-id"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let body2: serde_json::Value = res2.json().await.unwrap();
    assert_eq!(
        body2["box_id"].as_str().unwrap(),
        id1,
        "repeated calls must return the SAME id"
    );
}

#[tokio::test]
async fn box_id_persists_across_a_fresh_appstate_pointed_at_the_same_dir() {
    // Simulates a gateway restart: a brand-new AppState (and thus a brand-new
    // router) built against the SAME on-disk gateway_dir must serve the id
    // the first process already generated, never a fresh one.
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");

    let app1 = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base1 = spawn(app1).await;
    let res1 = reqwest::Client::new()
        .get(format!("{base1}/v1/box-id"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let body1: serde_json::Value = res1.json().await.unwrap();
    let id1 = body1["box_id"].as_str().unwrap().to_string();

    let app2 = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base2 = spawn(app2).await;
    let res2 = reqwest::Client::new()
        .get(format!("{base2}/v1/box-id"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let body2: serde_json::Value = res2.json().await.unwrap();
    assert_eq!(
        body2["box_id"].as_str().unwrap(),
        id1,
        "id must survive a restart against the same dir"
    );
}

/// Adversarial: many requests fire at a gateway whose `box_id.txt` does not
/// exist yet, all racing to be "the first" to create it. Every response must
/// carry the SAME id -- proves the `create_new` race guard actually works,
/// not just that it compiles.
#[tokio::test]
async fn box_id_concurrent_first_calls_converge_on_one_id() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let mut handles = Vec::new();
    for _ in 0..16 {
        let base = base.clone();
        let token = token.clone();
        handles.push(tokio::spawn(async move {
            let res = reqwest::Client::new()
                .get(format!("{base}/v1/box-id"))
                .bearer_auth(&token)
                .send()
                .await
                .unwrap();
            let body: serde_json::Value = res.json().await.unwrap();
            body["box_id"].as_str().unwrap().to_string()
        }));
    }
    let mut ids = std::collections::HashSet::new();
    for h in handles {
        ids.insert(h.await.unwrap());
    }
    assert_eq!(
        ids.len(),
        1,
        "every concurrent first-call must converge on exactly one id, got {ids:?}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn box_id_file_is_0600() {
    use std::os::unix::fs::PermissionsExt;
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/box-id"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let mode = std::fs::metadata(gateway_dir.path().join("box_id.txt"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600, "box_id.txt must be 0600");
}

/// Adversarial: a pre-existing but corrupted (empty) box_id.txt must not be
/// served as-is -- the endpoint must treat it like "absent" and regenerate.
#[tokio::test]
async fn box_id_regenerates_when_file_is_empty() {
    let gateway_dir = tempfile::tempdir().unwrap();
    std::fs::write(gateway_dir.path().join("box_id.txt"), "").unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/box-id"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["box_id"].as_str().unwrap().len(), 32);
}

/// Adversarial (review-fix regression test): a pre-existing but MALFORMED
/// (non-empty, wrong length/charset) box_id.txt must be rejected and
/// regenerated, not echoed back verbatim -- the endpoint's own contract is
/// "32 lowercase hex chars", never whatever garbage happened to be on disk.
#[tokio::test]
async fn box_id_regenerates_when_file_is_malformed_but_nonempty() {
    let gateway_dir = tempfile::tempdir().unwrap();
    // Too short, and contains a non-hex char ('z').
    std::fs::write(gateway_dir.path().join("box_id.txt"), "not-a-real-boxid-z").unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/box-id"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let id = body["box_id"].as_str().unwrap();
    assert_eq!(id.len(), 32);
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    assert_ne!(
        id, "not-a-real-boxid-z",
        "the malformed content must never be echoed back as-is"
    );
}

/// Adversarial (review-fix regression test): the SAME 16-way concurrent
/// race as `box_id_concurrent_first_calls_converge_on_one_id`, but starting
/// from a pre-existing EMPTY file rather than no file at all -- this is
/// exactly the scenario the review found unprotected in the first pass (the
/// empty-file branch did a direct, unguarded overwrite with no race
/// protection at all, so N concurrent callers could each persist and return
/// a DIFFERENT id).
#[tokio::test]
async fn box_id_concurrent_calls_against_a_preexisting_empty_file_converge_on_one_id() {
    let gateway_dir = tempfile::tempdir().unwrap();
    std::fs::write(gateway_dir.path().join("box_id.txt"), "").unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let mut handles = Vec::new();
    for _ in 0..16 {
        let base = base.clone();
        let token = token.clone();
        handles.push(tokio::spawn(async move {
            let res = reqwest::Client::new()
                .get(format!("{base}/v1/box-id"))
                .bearer_auth(&token)
                .send()
                .await
                .unwrap();
            let body: serde_json::Value = res.json().await.unwrap();
            let id = body["box_id"].as_str().unwrap().to_string();
            assert!(!id.is_empty(), "must never return an empty box_id");
            id
        }));
    }
    let mut ids = std::collections::HashSet::new();
    for h in handles {
        ids.insert(h.await.unwrap());
    }
    assert_eq!(ids.len(), 1, "every concurrent call against a pre-existing empty file must converge on one id, got {ids:?}");
}

/// Adversarial (review-fix regression test): box_id.txt must be 0600 the
/// moment it exists at its final path -- verified indirectly here by
/// confirming a fresh id-creation call never leaves a stray, wrongly-permissioned
/// `box_id.txt.tmp-*` file behind (the write-temp-then-rename must always
/// complete the rename, never abandon the temp file).
#[tokio::test]
async fn box_id_creation_leaves_no_stray_tmp_files() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/box-id"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let leftovers: Vec<_> = std::fs::read_dir(gateway_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("box_id.txt.tmp-")
        })
        .collect();
    assert!(
        leftovers.is_empty(),
        "no temp file should survive a successful creation: {leftovers:?}"
    );
}

/// Adversarial: `omega doctor` stdout with a check line whose glyph slot
/// holds a multi-byte UTF-8 character (`€`) instead of the expected
/// single-byte ASCII glyph. Before the fix, `parse_doctor_output`
/// byte-sliced `after_two_spaces[0..3]` unconditionally and panicked
/// ("byte index 3 is not a char boundary"), which broke the HTTP
/// connection for that request instead of returning a clean response. The
/// line must instead be treated like any other unrecognized glyph (skipped)
/// and the endpoint must answer 200, proving the connection survives and a
/// second request on the same server still succeeds.
#[tokio::test]
async fn doctor_survives_multibyte_glyph_in_check_line() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega(
        bin_dir.path(),
        "cat <<'EOF2'\nOmegaOS doctor\n\n  [\u{20ac}] weird glyph line\n\n[+] ok\nEOF2\nexit 0",
    );
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/doctor"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        200,
        "a malformed check line must never crash the connection"
    );
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(
        body["checks"].as_array().unwrap().len(),
        0,
        "the unrecognized-glyph line is skipped"
    );

    // The server process itself must still be alive for a second request.
    let res2 = reqwest::Client::new()
        .get(format!("{base}/v1/doctor"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status(), 200);

    clear_env();
}
