//! Focused check for the `chats` CLI subcommand (symmetry with `devices`).

use omega_gateway::chat_store::ChatStore;
use omega_gateway::protocol::ChatAgent;
use std::process::Command;

fn run_chats(gateway_dir: &std::path::Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_omega-gatewayd"))
        .arg("chats")
        .env("OMEGA_GATEWAY_DIR", gateway_dir)
        .output()
        .expect("failed to run omega-gatewayd chats");
    assert!(
        output.status.success(),
        "chats command exited non-zero: {output:?}"
    );
    String::from_utf8(output.stdout).expect("stdout should be utf8")
}

#[test]
fn chats_command_reports_none_when_empty() {
    let dir = tempfile::tempdir().unwrap();
    let stdout = run_chats(dir.path());
    assert!(
        stdout.contains("no chats"),
        "expected 'no chats', got: {stdout}"
    );
}

#[test]
fn chats_command_lists_created_chat() {
    let dir = tempfile::tempdir().unwrap();
    let store = ChatStore::open(dir.path());
    let meta = store.create(
        ChatAgent::Claude,
        "/tmp/proj".to_string(),
        Some("hello world".to_string()),
        None,
    );

    let stdout = run_chats(dir.path());
    assert!(
        stdout.contains(&meta.id),
        "expected chat id {} in output: {stdout}",
        meta.id
    );
    assert!(
        stdout.contains("hello world"),
        "expected title in output: {stdout}"
    );
    assert!(
        stdout.contains("claude"),
        "expected agent in output: {stdout}"
    );
}
