// Empirical end-to-end test of amplify_mission against the real claude CLI.
fn main() {
    let raw = std::env::args().nth(1).unwrap_or_else(|| {
        "ajoute un systeme de notifications quand un worker termine".to_string()
    });
    let project = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "OmegaOS".to_string());
    let wd = std::env::args().nth(3).unwrap_or_else(|| ".".to_string());
    eprintln!("--- amplifying (raw={} chars) ---", raw.len());
    let t = std::time::Instant::now();
    let brief = omega_core::amplify::amplify_mission(&raw, &project, &wd);
    eprintln!("--- done in {:?} ---", t.elapsed());
    println!("{}", brief);
}
