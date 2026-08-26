use omega_core::session::SessionManager;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mgr = SessionManager::connect().await?;
    let s = "keytest";
    for ch in ["a", "b", "c"] {
        mgr.send_text_raw(s, ch).await?;
    }
    mgr.send_key(s, "BSpace").await?;
    mgr.send_key(s, "BSpace").await?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    let cap = mgr.capture_pane(s).await?;
    let last = cap
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("");
    println!("OUTPUT LINE: |{}|", last);
    let hex: String = last
        .bytes()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    println!("HEX:         {}", hex);
    Ok(())
}
