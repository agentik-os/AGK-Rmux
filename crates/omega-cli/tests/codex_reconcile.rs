#[cfg(unix)]
#[test]
fn codex_reconcile_json_uses_isolated_credential_roots() {
    use serde_json::Value;
    use std::fs;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "omega-cli-codex-reconcile-{}-{}",
        std::process::id(),
        nonce
    ));
    fs::create_dir(&root).expect("create uniquely-owned test root");
    let home = root.join("home");
    let omega_dir = root.join("omega");
    let codex_home = root.join("codex-home");
    let canonical = omega_dir.join("credentials").join("codex.json");
    let native = codex_home.join("auth.json");
    fs::create_dir_all(canonical.parent().unwrap()).expect("create canonical parent");
    fs::create_dir_all(&home).expect("create isolated HOME");
    fs::create_dir_all(&codex_home).expect("create isolated CODEX_HOME");
    fs::write(
        &canonical,
        br#"{"auth_mode":"apikey","OPENAI_API_KEY":"isolated-test-key"}"#,
    )
    .expect("write isolated canonical credential");

    let output = Command::new(env!("CARGO_BIN_EXE_omega"))
        .args(["codex-reconcile", "--json"])
        .env("HOME", &home)
        .env("OMEGA_DIR", &omega_dir)
        .env("CODEX_HOME", &codex_home)
        .output()
        .expect("run real omega binary");
    assert!(
        output.status.success(),
        "codex-reconcile failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("parse codex-reconcile JSON contract");
    assert_eq!(json["ok"], true);
    assert_eq!(json["status"], "reconciled");

    let metadata = fs::symlink_metadata(&native).expect("native credential exists");
    assert!(metadata.file_type().is_symlink());
    let link = fs::read_link(&native).expect("read native credential symlink");
    let resolved = if link.is_absolute() {
        link
    } else {
        native.parent().unwrap().join(link)
    };
    assert_eq!(
        fs::canonicalize(resolved).expect("resolve native symlink"),
        fs::canonicalize(&canonical).expect("resolve canonical credential")
    );
    assert!(native.starts_with(&codex_home));

    let _ = fs::remove_dir_all(root);
}
