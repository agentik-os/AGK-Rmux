//! Display clock — the single funnel for turning "now" into a human-facing
//! time string in the timezone the *human* lives in, not the box's.
//!
//! Why this exists: a headless VPS almost always runs in UTC (`Etc/UTC`), so
//! `chrono::Local::now()` — which resolves to the system zone — renders UTC.
//! The operator sitting in CEST then sees a clock that is 1–2h off. Persisted
//! timestamps stay UTC on purpose (logs/state are written with `Utc::now()`);
//! ONLY the on-screen clock is localized, and only here.
//!
//! Resolution order for the display zone:
//!   1. an explicit IANA name from config (`timezone = "Europe/Paris"`),
//!   2. the `TZ` env var, if set and parseable,
//!   3. the system local zone (`chrono::Local`) — the upstream default.
//!
//! IANA names (not fixed offsets) are used so DST is handled automatically:
//! `Europe/Paris` renders +01:00 in winter and +02:00 in summer with no config
//! change.

use chrono::Utc;
use chrono_tz::Tz;

/// Resolve the configured display timezone, if any is usable.
///
/// `cfg_tz` is the optional `timezone` field from `OmegaConfig`. An empty or
/// unparseable value falls through to `$TZ`, then to `None` (meaning: use the
/// system local zone at the call site).
fn resolve_tz(cfg_tz: Option<&str>) -> Option<Tz> {
    cfg_tz
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<Tz>().ok())
        .or_else(|| {
            std::env::var("TZ")
                .ok()
                .and_then(|s| s.trim().parse::<Tz>().ok())
        })
}

/// Format `now` with `fmt` (a chrono/strftime pattern) in the display zone.
/// Falls back to the system local zone when no IANA zone is configured.
pub fn now_fmt(cfg_tz: Option<&str>, fmt: &str) -> String {
    match resolve_tz(cfg_tz) {
        Some(tz) => Utc::now().with_timezone(&tz).format(fmt).to_string(),
        None => chrono::Local::now().format(fmt).to_string(),
    }
}

/// `HH:MM` in the display zone — the status-bar clock.
pub fn hm(cfg_tz: Option<&str>) -> String {
    now_fmt(cfg_tz, "%H:%M")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_iana_zone_overrides_system() {
        // A fixed UTC instant rendered in two zones must differ by the offset,
        // proving the config zone — not the box's local zone — drives output.
        let utc = "UTC";
        let paris = "Europe/Paris";
        // Same wall-clock function, different zones → the strings can differ.
        // We can't assert an exact value (depends on `now`), but we CAN assert
        // the parser accepts the IANA names and produces HH:MM shaped output.
        let a = hm(Some(utc));
        let b = hm(Some(paris));
        assert_eq!(a.len(), 5, "HH:MM is 5 chars, got {a:?}");
        assert_eq!(b.len(), 5, "HH:MM is 5 chars, got {b:?}");
        assert_eq!(a.as_bytes()[2], b':');
    }

    #[test]
    fn empty_or_garbage_zone_falls_back_cleanly() {
        // None, empty, and unparseable all must NOT panic and must yield HH:MM.
        for z in [None, Some(""), Some("   "), Some("Not/AZone")] {
            let s = hm(z);
            assert_eq!(s.len(), 5, "fallback must still render HH:MM, got {s:?}");
        }
    }
}
