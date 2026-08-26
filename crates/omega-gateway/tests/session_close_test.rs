//! `POST /v1/sessions/{name}/close` — runs `omega kill <name>` (never
//! `--force`) via `omega_cli::run`, classifies the session as
//! oracle/non-oracle purely off its NAME (no I/O), and parses `omega kill`'s
//! real stdout shape (`cmd_kill` in `crates/omega-cli/src/main.rs`) for
//! `cascaded_count` / `already_closed`. Mirrors `dispatch_test.rs`'s
//! fake-omega-with-argv-capture idiom.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_BIN is process-global; serialize every test in this binary that
// touches it (same pattern as dispatch_test.rs).
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

/// Writes an executable fake `omega` script and points `OMEGA_BIN` at it.
fn install_fake_omega(bin_dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = bin_dir.join("omega");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

/// Fails loudly if invoked at all, proving a subprocess spawn never happened
/// (validation rejected before touching it) — mirrors
/// `session_keys_test.rs::install_fake_rmux_that_must_not_run`.
fn install_fake_omega_that_must_not_run(bin_dir: &std::path::Path) {
    install_fake_omega(bin_dir, "echo 'SHOULD NEVER RUN' >&2; exit 1");
}

/// Writes an executable fake `omega` script that ALSO appends its full argv
/// (one element per line) to `capture_file` — same idiom
/// `sessions_create_test.rs::install_fake_omega` uses, needed here to prove
/// the EXACT argv `close` builds (I-7).
fn install_fake_omega_capturing(
    bin_dir: &std::path::Path,
    capture_file: &std::path::Path,
    script_body: &str,
) {
    use std::os::unix::fs::PermissionsExt;
    let path = bin_dir.join("omega");
    let capture = capture_file.display();
    std::fs::write(
        &path,
        format!("#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > '{capture}'\n{script_body}\n"),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

#[tokio::test]
async fn happy_path_close_of_a_plain_session() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    // Real cmd_kill success-path shape: only the final "Killed session: ..."
    // line, no cascaded-worker lines, exit 0.
    install_fake_omega(
        bin_dir.path(),
        "printf 'Killed session: worker-Foo-1\\n'; exit 0",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/worker-Foo-1/close"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["killed"], true);
    assert_eq!(body["already_closed"], false);
    assert_eq!(body["is_oracle"], false);
    assert_eq!(body["cascaded_count"], 0);
    assert!(body["message"].as_str().unwrap().contains("Killed session"));

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn oracle_close_with_two_cascaded_workers() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    // Real cmd_kill shape for an oracle with two cascaded workers: both the
    // success and failure per-worker line forms count.
    install_fake_omega(
        bin_dir.path(),
        r#"printf '  cascaded worker closed: oracle-Foo-1-worker-a\n  cascaded worker oracle-Foo-1-worker-b could not be killed (session not found)\nKilled session: oracle-Foo-1\n'; exit 0"#,
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/oracle-Foo-1/close"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["killed"], true);
    assert_eq!(body["already_closed"], false);
    assert_eq!(body["is_oracle"], true);
    assert_eq!(body["cascaded_count"], 2);

    std::env::remove_var("OMEGA_BIN");
}

/// M-1 (Codex cross-model review, 2026-08-11) changed this test's own
/// contract: the FAILURE-path `message` (this endpoint always returns 200,
/// even on a REFUSED kill — see this function's own doc comment) used to be
/// raw stdout+stderr concatenated, which can leak environment-derived
/// secrets or other sensitive subprocess text. It is now a fixed, sanitized
/// string; the full raw REFUSED text (and any secret it might carry) still
/// reaches the gateway's own tracing log, never the HTTP response. `killed`
/// and `is_oracle` — the fields that actually drive UI behavior — are
/// unaffected and still correctly reflect the REFUSED outcome.
#[tokio::test]
async fn refused_kill_returns_200_with_killed_false_and_a_sanitized_message() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    // Real cmd_kill REFUSED shape: nothing on stdout, the anyhow::bail!
    // message lands on stderr prefixed "Error: " (Rust's default
    // Termination impl for a failing `async fn main() -> Result<()>`),
    // verified via a throwaway reproduction — see routes_sessions.rs's
    // `close` doc comment.
    install_fake_omega(
        bin_dir.path(),
        r#"echo 'Error: kill REFUSED — 2 worker(s) of this oracle are still running: a, b.' >&2; exit 1"#,
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/oracle-Foo-1/close"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        200,
        "a REFUSED kill is a normal outcome, never a gateway error"
    );
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["killed"], false);
    assert_eq!(body["is_oracle"], true);
    let message = body["message"].as_str().unwrap();
    assert!(
        !message.contains("REFUSED"),
        "the failure-path message must be sanitized, not raw CLI text: {message:?}"
    );
    assert!(
        !message.contains("worker(s)"),
        "the failure-path message must be sanitized, not raw CLI text: {message:?}"
    );

    std::env::remove_var("OMEGA_BIN");
}

/// M-1 (Codex cross-model review, 2026-08-11) changed this test's own
/// contract, same reasoning as
/// `refused_kill_returns_200_with_killed_false_and_a_sanitized_message`'s
/// doc comment. This test's remaining real purpose: `is_oracle` must still
/// classify off the RESOLVED name (via `resolved_oracle_name`, parsed from
/// stdout SERVER-SIDE) even on the REFUSED path — that parsing happens
/// before the success/failure split and is entirely independent of the
/// now-sanitized `message` field.
#[tokio::test]
async fn refused_kill_still_classifies_is_oracle_off_the_resolved_alias() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    // Reproduces the exact sequence `cmd_kill` produces when the caller
    // supplies an alias-shaped name that needs `resolve_oracle_alias`: the
    // resolution line lands on stdout FIRST, then the REFUSED bail lands on
    // stderr and the process exits non-zero.
    install_fake_omega(
        bin_dir.path(),
        r#"printf '[i] Verba-3 resolved to the oracle session oracle-Verba-3\n'; echo 'Error: kill REFUSED — 1 worker(s) of this oracle are still running: a.' >&2; exit 1"#,
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/Verba-3/close"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["killed"], false);
    assert_eq!(
        body["is_oracle"], true,
        "expected classification off the RESOLVED name even on the REFUSED path"
    );
    let message = body["message"].as_str().unwrap();
    assert!(
        !message.contains("REFUSED"),
        "the failure-path message must be sanitized, not raw CLI text: {message:?}"
    );

    std::env::remove_var("OMEGA_BIN");
}

/// M-1 (Codex cross-model review, 2026-08-11): a secret-shaped string
/// written to stdout/stderr by a failing `omega kill` must never reach the
/// HTTP response body — only the gateway's own log.
#[tokio::test]
async fn failed_close_never_leaks_a_secret_shaped_string_into_the_response() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    install_fake_omega(
        bin_dir.path(),
        "echo 'sk-ProjSECRETVALUE1234567890'; echo 'ANTHROPIC_API_KEY=sk-ProjSECRETVALUE1234567890' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/worker-Foo-1/close"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let raw_body = res.text().await.unwrap();
    assert!(
        !raw_body.contains("sk-ProjSECRETVALUE1234567890"),
        "response body leaked the raw secret-shaped subprocess output: {raw_body}"
    );
    assert!(
        !raw_body.contains("ANTHROPIC_API_KEY"),
        "response body leaked the raw secret-shaped subprocess output: {raw_body}"
    );

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn is_oracle_true_when_the_raw_path_param_is_an_unresolved_alias() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    // The request path param is the raw alias "Verba-3" (classifies as
    // SessionRole::Home on its own — no "oracle-" prefix), but `cmd_kill`
    // resolves and kills "oracle-Verba-3", printing the resolution line
    // before its normal cascaded-worker + Killed-session success output.
    // Bug 2: the old logic classified the raw path param instead of the
    // resolved name, so `is_oracle` came back false here.
    install_fake_omega(
        bin_dir.path(),
        r#"printf '[i] Verba-3 resolved to the oracle session oracle-Verba-3\n  cascaded worker closed: oracle-Verba-3-worker-a\nKilled session: oracle-Verba-3\n'; exit 0"#,
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/Verba-3/close"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["killed"], true);
    assert_eq!(
        body["is_oracle"], true,
        "expected classification off the RESOLVED name"
    );
    assert_eq!(body["cascaded_count"], 1);

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn already_closed_session() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    install_fake_omega(
        bin_dir.path(),
        "printf 'Session worker-Foo-1 is already closed — nothing live to kill.\\n'; exit 0",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/worker-Foo-1/close"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["already_closed"], true);
    assert_eq!(body["cascaded_count"], 0);

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn invalid_session_name_rejects_before_any_subprocess_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();

    install_fake_omega_that_must_not_run(bin_dir.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    for bad_name in ["..%2F..%2Fetc", "foo%2Fbar"] {
        let res = reqwest::Client::new()
            .post(format!("{base}/v1/sessions/{bad_name}/close"))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            400,
            "expected 400 for session name {bad_name}"
        );
    }

    std::env::remove_var("OMEGA_BIN");
}

/// I-7 (Codex cross-model review, 2026-08-11): `valid_session_name` permits
/// a name whose FIRST byte is `-` (unlike `valid_new_session_name`, which
/// rejects a leading `-`/`.` for a BRAND NEW name) -- `"-x"` only fails on
/// `/`, `..`, NUL, or a >200-byte length, none of which apply here. Without
/// a `"--"` separator before the positional, `close` used to build
/// `["kill", "-x"]`, which a real clap CLI parses `-x` as an (unknown) FLAG
/// rather than the session name. Proven directly against the recorded argv,
/// not just the HTTP status, so a regression that silently drops the
/// separator again is caught even if the fake CLI would still exit 0 either
/// way.
#[tokio::test]
async fn close_uses_a_double_dash_separator_so_a_leading_dash_session_name_is_never_parsed_as_a_flag(
) {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega_capturing(
        bin_dir.path(),
        &capture_file,
        "printf 'Killed session: -x\\n'; exit 0",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/-x/close"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        200,
        "\"-x\" must pass valid_session_name (only '/', '..', NUL, and length are checked) and reach the CLI"
    );

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    // Without the "--" separator this would be exactly ["kill", "-x"] --
    // the exact bug I-7 fixes.
    assert_eq!(argv, vec!["kill", "--", "-x"]);

    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn post_close_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions/oracle-Foo-1/close"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}
