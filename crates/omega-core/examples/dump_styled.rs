// Empirical: dump the per-span foreground of a session's styled capture,
// exactly via the production capture_pane_styled path, to learn whether
// rmux's SDK snapshot preserves Claude's slash-menu selection color.
use omega_core::session::SessionManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let target = std::env::args().nth(1).unwrap_or_else(|| "slugtest".into());
    let mgr = SessionManager::connect().await?;
    let (rows, crow, ccol, cvis) = match mgr.capture_pane_styled(&target, 0).await? {
        omega_core::session::StyledCapture::Changed {
            rows,
            cursor_row,
            cursor_col,
            cursor_visible,
            ..
        } => (rows, cursor_row, cursor_col, cursor_visible),
        omega_core::session::StyledCapture::Unchanged => {
            eprintln!("(snapshot unchanged — nothing to dump)");
            return Ok(());
        }
    };
    eprintln!("cursor: row={crow} col={ccol} visible={cvis}");
    for row in &rows {
        let text: String = row.iter().map(|s| s.text.clone()).collect();
        if text.contains("/team") || text.contains("/codeaudit") || text.contains("/debugaudit") {
            print!("ROW {:?}\n   spans:", text.trim());
            for s in row {
                if s.text.trim().is_empty() {
                    continue;
                }
                print!(
                    " [{:?} fg={:?} bg={:?} bold={}]",
                    s.text.trim(),
                    s.fg,
                    s.bg,
                    s.bold
                );
            }
            println!();
        }
    }
    Ok(())
}
