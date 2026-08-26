//! `POST /v1/sessions` — `crates/omega-gateway/src/routes_sessions.rs::create`.
//! Wraps `omega new [OPTIONS] <NAME>`.
//!
//! A dedicated file (rather than extending `sessions_test.rs`, which is
//! scoped to `GET /v1/sessions` and mutates the unrelated `OMEGA_RMUX_BIN`
//! env var): this endpoint's tests mutate `OMEGA_BIN` and `HOME` instead,
//! and keeping the two env-var concerns in separate files/binaries avoids
//! any risk of one file's `LOCK` scope leaking into the other's.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_BIN and HOME are process-global; serialize every test in this binary
// that touches either (same pattern as dispatch_test.rs's LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// Writes an executable fake `omega` script that also appends its full argv
/// (one per line) to `capture_file` — same idiom `dispatch_test.rs::
/// install_fake_omega` uses.
fn install_fake_omega(
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

async fn app_and_token(gateway_dir: &std::path::Path) -> (axum::Router, String) {
    let (_, token) = DeviceStore::open(gateway_dir).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.to_path_buf(),
        GatewayConfig::default(),
    ));
    (app, token)
}

fn clear_env() {
    std::env::remove_var("OMEGA_BIN");
    std::env::remove_var("HOME");
}

/// A real agent name off the actual roster, so this test tracks the real
/// roster rather than a hardcoded guess (same posture
/// `dispatch_test.rs::known_agent_dispatches_normally` uses).
fn real_agent_name() -> &'static str {
    let agents = omega_core::agents::Agent::all();
    assert!(!agents.is_empty());
    agents[0].name()
}

#[tokio::test]
async fn happy_path_with_explicit_name_builds_exact_argv() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "printf 'Created session: my-session\\n'; exit 0",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "agent": agent,
            "name": "my-session",
            "dir": null,
            "prompt": "do the thing"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "my-session");
    assert_eq!(body["agent"], agent);
    assert!(body["output"].as_str().unwrap().contains("Created session"));

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    // Finding 5 (adversarial review round): `--prompt` is now a single
    // `=`-joined argv element, never two separate elements.
    assert_eq!(
        argv,
        vec![
            "new",
            "--agent",
            agent,
            "--prompt=do the thing",
            "--",
            "my-session"
        ]
    );

    clear_env();
}

#[tokio::test]
async fn happy_path_with_dir_builds_exact_argv_in_order() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let fake_home = tempfile::tempdir().unwrap();
    let agent = real_agent_name();

    std::env::set_var("HOME", fake_home.path());
    install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let dir_str = fake_home
        .path()
        .join("Station")
        .join("Proj")
        .display()
        .to_string();
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "agent": agent,
            "name": "my-session",
            "dir": dir_str,
            "prompt": "hello"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    // Finding 5 (adversarial review round): `--dir`/`--prompt` are now
    // single `=`-joined argv elements, never two separate elements each.
    let dir_flag = format!("--dir={dir_str}");
    assert_eq!(
        argv,
        vec![
            "new",
            "--agent",
            agent,
            dir_flag.as_str(),
            "--prompt=hello",
            "--",
            "my-session"
        ]
    );

    clear_env();
}

#[tokio::test]
async fn happy_path_with_no_name_generates_one_and_uses_it_as_the_positional() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let name = body["name"].as_str().unwrap().to_string();
    // Server-generated pattern: "gw-" + 12 lowercase hex chars (6 random bytes).
    assert!(name.starts_with("gw-"), "expected gw- prefix, got {name}");
    let hex_part = &name["gw-".len()..];
    assert_eq!(hex_part.len(), 12, "expected 12 hex chars, got {hex_part}");
    assert!(hex_part
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(argv, vec!["new", "--agent", agent, "--", name.as_str()]);

    clear_env();
}

#[tokio::test]
async fn unknown_agent_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "not-a-real-agent" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("not-a-real-agent"));
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for an unknown agent"
    );

    clear_env();
}

#[tokio::test]
async fn invalid_caller_name_leading_dash_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "name": "-leading-dash" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for an invalid name"
    );

    clear_env();
}

#[tokio::test]
async fn invalid_caller_name_slash_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "name": "has/slash" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for an invalid name"
    );

    clear_env();
}

#[tokio::test]
async fn invalid_caller_name_nul_byte_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "name": "has\u{0000}nul" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for a NUL-containing name"
    );

    clear_env();
}

#[tokio::test]
async fn dir_outside_home_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let fake_home = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let agent = real_agent_name();

    std::env::set_var("HOME", fake_home.path());
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "agent": agent,
            "dir": outside.path().display().to_string(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("home directory"));
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for a dir outside home"
    );

    clear_env();
}

/// Finding 5 (adversarial review round): a `--` separator protects
/// POSITIONALS, not flag VALUES — `clap` parses `--prompt -x` as TWO
/// flags, not one flag plus a hyphen-leading value, since
/// `allow_hyphen_values` isn't set on `--prompt` in the real CLI. A prompt
/// starting with `-` used to be a guaranteed clap parse error (502) even
/// though it is perfectly legitimate input. Fixed by emitting the flag as a
/// single argv element via the `=` form (`--prompt=-x`), which clap always
/// accepts regardless of what the value starts with. (`dir` gets the same
/// code-level fix for consistency, but is not separately regression-tested
/// here with a leading-`-` value: `dir_under_home` only ever returns a
/// value that resolves under `$HOME` via `std::fs::canonicalize`, so an
/// ACCEPTED `dir` value is always an absolute path — it can never itself
/// start with `-`. The existing `happy_path_with_dir_builds_exact_argv_in_
/// order` test below already proves `dir`'s new single-element `=` argv
/// shape on a normal path.)
#[tokio::test]
async fn prompt_starting_with_dash_reaches_argv_intact_as_a_single_element() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "name": "my-session", "prompt": "-x" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "a hyphen-leading prompt must not 502");

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(
        argv,
        vec!["new", "--agent", agent, "--prompt=-x", "--", "my-session"]
    );

    clear_env();
}

/// Finding 4 (adversarial review round): `valid_new_session_name` allows up
/// to 100 bytes, but the real session name `cmd_new` creates goes through
/// `omega_core::session::sanitize_session_name`, which truncates at
/// `MAX_SESSION_NAME_LEN` (48). A 60-char name survives the charset check
/// but would be silently truncated downstream -- the response would then
/// lie about the session the caller can actually address. Must now be a
/// structural 400 BEFORE any spawn, not a silent truncate-and-create.
#[tokio::test]
async fn caller_name_that_sanitize_would_truncate_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    // 60 chars: passes `valid_new_session_name`'s 100-byte cap, but
    // `sanitize_session_name` truncates at 48.
    let long_name = "a".repeat(60);
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "name": long_name }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for a name sanitize would truncate"
    );

    clear_env();
}

/// Finding 4 companion: a name with a legitimate trailing `-` is allowed by
/// `valid_new_session_name`'s charset check (it only rejects a LEADING
/// dash) but `sanitize_session_name` trims a trailing `-`/`.` -- must also
/// be a structural 400, not a silent trim-and-create.
#[tokio::test]
async fn caller_name_with_trailing_dash_that_sanitize_would_trim_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "name": "my-session-" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for a name sanitize would trim"
    );

    clear_env();
}

/// Finding 1 (adversarial review round, `dir_under_home` in
/// `routes_sessions.rs`): a `dir` shaped like
/// `<home>/does-not-exist-yet/../../../tmp/evil/PWNED` used to defeat the
/// ancestor-walk canonicalization -- every ancestor containing
/// `does-not-exist-yet` fails to canonicalize (it does not exist), so the
/// walk kept popping until it reached `$HOME` itself (which DOES
/// canonicalize and pass the prefix check), then returned the ORIGINAL
/// uncanonicalized string -- still containing the escaping `..` sequences.
/// Must now be a structural 400 (a `..` component is rejected before the
/// ancestor walk ever runs), never a spawn.
#[tokio::test]
async fn dir_traversal_via_nonexistent_leading_component_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let fake_home = tempfile::tempdir().unwrap();
    let agent = real_agent_name();

    std::env::set_var("HOME", fake_home.path());
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let escaping = fake_home
        .path()
        .join("does-not-exist-yet")
        .join("..")
        .join("..")
        .join("..")
        .join("tmp")
        .join("evil")
        .join("PWNED");
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "agent": agent,
            "dir": escaping.display().to_string(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for a traversal-shaped dir"
    );

    clear_env();
}

#[tokio::test]
async fn prompt_over_length_cap_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    // One byte over routes_sessions.rs's MAX_PROMPT_LEN (8000).
    let too_long = "x".repeat(8001);
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "prompt": too_long }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("too long"));
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for an over-length prompt"
    );

    clear_env();
}

#[tokio::test]
async fn nonzero_exit_surfaces_stdout_and_stderr_as_502() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'partial output' ; echo 'session name already in use' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "name": "dup-session" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let body: serde_json::Value = res.json().await.unwrap();
    // M-1 (Codex cross-model review, 2026-08-11; gap found during
    // whole-branch review): the raw stdout/stderr must never reach the
    // client, only a generic sanitized message. The full raw text is still
    // captured in the gateway's own tracing log, not asserted here.
    let error_text = body["error"].as_str().unwrap();
    assert!(!error_text.contains("partial output"));
    assert!(!error_text.contains("session name already in use"));
    assert!(body.get("stdout").is_none());
    assert!(body.get("stderr").is_none());
    assert!(
        body.get("name").is_none(),
        "must never fabricate a session on failure"
    );

    clear_env();
}

/// Finding 2 (adversarial review round): `POST /v1/sessions` now shares
/// `AppState::session_spawn_permits` with `POST /v1/team` (server.rs) --
/// exhausted permits get a 429, never an unbounded pile of concurrent
/// `omega new` subprocesses. Same idiom `pdf_test.rs`'s
/// `concurrency_cap_returns_429_when_pdf_permits_exhausted` uses: a
/// sleeping fake `omega`, N in-flight requests, assert the N+1th gets 429.
#[tokio::test]
async fn concurrency_cap_returns_429_when_session_spawn_permits_exhausted() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(bin_dir.path(), &capture_file, "sleep 0.15\nexit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // Must match server.rs's MAX_CONCURRENT_SESSION_SPAWNS.
    const MAX_CONCURRENT_SESSION_SPAWNS: usize = 4;

    let mut in_flight = Vec::new();
    for _ in 0..MAX_CONCURRENT_SESSION_SPAWNS {
        let client = client.clone();
        let base = base.clone();
        let token = token.clone();
        let agent = agent.to_string();
        in_flight.push(tokio::spawn(async move {
            client
                .post(format!("{base}/v1/sessions"))
                .bearer_auth(&token)
                .json(&serde_json::json!({ "agent": agent }))
                .send()
                .await
                .unwrap()
                .status()
                .as_u16()
        }));
    }

    tokio::time::sleep(std::time::Duration::from_millis(60)).await;

    let busy_res = client
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent }))
        .send()
        .await
        .unwrap();
    assert_eq!(busy_res.status(), 429);
    let body: serde_json::Value = busy_res.json().await.unwrap();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("too many concurrent"));

    for task in in_flight {
        let status = task.await.unwrap();
        assert_eq!(status, 200);
    }

    clear_env();
}

/// I-2 (Codex cross-model review, 2026-08-11): `POST /v1/sessions` now runs
/// `omega new` through `omega_cli::run_with_timeout` instead of the old
/// uncancellable blocking `run` -- a hung/adversarially-slow subprocess must
/// be killed (whole process group, including a nested child it spawned) and
/// reported as a 504, not left running forever pinning a permit and a
/// blocking-pool thread. `OMEGA_CLI_TIMEOUT_SECS` overrides the default
/// ceiling for this test.
#[tokio::test]
async fn create_times_out_with_504_and_kills_the_whole_process_group() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let marker = capture_dir.path().join("marker");
    let agent = real_agent_name();

    std::env::set_var("OMEGA_CLI_TIMEOUT_SECS", "1");
    // Backgrounds a nested `bash -c` (inheriting the fake omega's process
    // group) that sleeps well past the 1s ceiling before touching a marker
    // file, then waits on it -- mirroring a real CLI that runs its own
    // foreground nested work. Proves the WHOLE group is killed, not just
    // the direct `omega` process.
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        &format!(
            "bash -c 'sleep 4; touch \"{}\"' &\nwait\n",
            marker.display()
        ),
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 504);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(
        body["error"].as_str().unwrap().contains("timed out"),
        "body: {body}"
    );

    // Generous buffer past the nested sleep's 4s -- if the group kill
    // missed the nested child, the marker appears around the 4s mark.
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    assert!(
        !marker.exists(),
        "the nested child survived the gateway's timeout kill"
    );

    std::env::remove_var("OMEGA_CLI_TIMEOUT_SECS");
    clear_env();
}

#[tokio::test]
async fn post_sessions_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .json(&serde_json::json!({ "agent": "claude" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}
