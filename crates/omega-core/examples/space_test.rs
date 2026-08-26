use omega_core::session::SessionManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mgr = SessionManager::connect().await?;
    let session = "test-space-bug";

    let _ = std::process::Command::new("rmux")
        .args(["kill-session", "-t", session])
        .status();
    std::process::Command::new("rmux")
        .args([
            "new-session",
            "-d",
            "-s",
            session,
            "bash --noprofile --norc",
        ])
        .status()?;
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // Type "ab cd ef" — spaces via send_key("Space"), letters via send_text_raw
    let seq = ["a", "b", "SPACE", "c", "d", "SPACE", "e", "f"];
    for tok in seq {
        if tok == "SPACE" {
            mgr.send_key(session, "Space").await?;
        } else {
            mgr.send_text_raw(session, tok).await?;
        }
        std::thread::sleep(std::time::Duration::from_millis(60));
    }
    std::thread::sleep(std::time::Duration::from_millis(500));

    let cap = mgr.capture_pane(session).await?;
    println!("=== capture (bottom) ===");
    for line in cap
        .lines()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        println!("|{}|", line);
    }
    if let Some(last) = cap.lines().rev().find(|l| !l.trim().is_empty()) {
        let hex: String = last.bytes().map(|b| format!("{:02x} ", b)).collect();
        println!("\nlast text: {}", last);
        println!("last hex:  {}", hex);
    }
    let _ = std::process::Command::new("rmux")
        .args(["kill-session", "-t", session])
        .status();
    Ok(())
}
