//! Lightweight system info — /proc on Linux, sysctl/vm_stat on macOS. No external crates.
//! Every reader tries /proc first (free on Linux) and falls back to the
//! macOS tools at runtime, so a single build works on both platforms.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_load: f32,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub ram_pct: u8,
    pub disk_used_pct: u8,
}

impl SystemStats {
    pub fn read() -> Self {
        Self {
            cpu_load: read_loadavg(),
            ram_used_mb: 0,
            ram_total_mb: 0,
            ram_pct: read_ram_pct(),
            disk_used_pct: read_disk_pct("/"),
        }
        .fill_ram()
    }

    fn fill_ram(mut self) -> Self {
        if let Some((used, total)) = read_meminfo() {
            self.ram_used_mb = used;
            self.ram_total_mb = total;
        }
        self
    }
}

fn cmd_stdout(cmd: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn read_loadavg() -> f32 {
    if let Ok(s) = std::fs::read_to_string("/proc/loadavg") {
        return s
            .split_whitespace()
            .next()
            .and_then(|n| n.parse::<f32>().ok())
            .unwrap_or(0.0);
    }
    // macOS: `sysctl -n vm.loadavg` → "{ 1.84 2.11 2.29 }"
    cmd_stdout("sysctl", &["-n", "vm.loadavg"])
        .and_then(|s| {
            s.split_whitespace()
                .find(|t| t.chars().next().is_some_and(|c| c.is_ascii_digit()))
                .and_then(|n| n.parse::<f32>().ok())
        })
        .unwrap_or(0.0)
}

fn read_ram_pct() -> u8 {
    if let Some((used, total)) = read_meminfo() {
        if total > 0 {
            return ((used as f64 / total as f64) * 100.0) as u8;
        }
    }
    0
}

/// (used MB, total MB) — Linux /proc/meminfo, else macOS hw.memsize + vm_stat.
fn read_meminfo() -> Option<(u64, u64)> {
    if let Ok(s) = std::fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0u64;
        let mut avail_kb = 0u64;
        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                total_kb = rest.split_whitespace().next()?.parse().ok()?;
            } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
                avail_kb = rest.split_whitespace().next()?.parse().ok()?;
            }
        }
        if total_kb == 0 {
            return None;
        }
        let used_kb = total_kb.saturating_sub(avail_kb);
        return Some((used_kb / 1024, total_kb / 1024));
    }
    macos_meminfo()
}

/// macOS RAM: total from `sysctl -n hw.memsize`, used = (active + wired +
/// compressor) pages from `vm_stat` — the same notion of "used" Activity
/// Monitor reports (file cache stays out, like Linux MemAvailable).
fn macos_meminfo() -> Option<(u64, u64)> {
    let total_mb = cmd_stdout("sysctl", &["-n", "hw.memsize"])?
        .trim()
        .parse::<u64>()
        .ok()?
        / (1024 * 1024);
    let (page_size, pages) = read_vm_stat()?;
    let used_pages =
        pages("Pages active") + pages("Pages wired down") + pages("Pages occupied by compressor");
    Some(((used_pages * page_size) / (1024 * 1024), total_mb))
}

/// Parse `vm_stat`: returns the page size in bytes and a lookup closure over
/// the "Pages …: N." lines.
fn read_vm_stat() -> Option<(u64, impl Fn(&str) -> u64)> {
    let out = cmd_stdout("vm_stat", &[])?;
    // Header: "Mach Virtual Memory Statistics: (page size of 16384 bytes)"
    let page_size = out
        .lines()
        .next()?
        .split("page size of ")
        .nth(1)?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    let lookup = move |key: &str| -> u64 {
        out.lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.rsplit(':').next())
            .and_then(|v| v.trim().trim_end_matches('.').parse::<u64>().ok())
            .unwrap_or(0)
    };
    Some((page_size, lookup))
}

/// Available RAM in MB — Linux MemAvailable, else macOS reclaimable pages
/// (free + inactive + purgeable + speculative). 0 if unreadable.
pub fn available_ram_mb() -> u64 {
    if let Ok(s) = std::fs::read_to_string("/proc/meminfo") {
        return s
            .lines()
            .find(|l| l.starts_with("MemAvailable:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|kb| kb.parse::<u64>().ok())
            .map(|kb| kb / 1024)
            .unwrap_or(0);
    }
    read_vm_stat()
        .map(|(page_size, pages)| {
            let avail_pages = pages("Pages free")
                + pages("Pages inactive")
                + pages("Pages purgeable")
                + pages("Pages speculative");
            (avail_pages * page_size) / (1024 * 1024)
        })
        .unwrap_or(0)
}

fn read_disk_pct(path: &str) -> u8 {
    // POSIX `df -P` (works on Linux AND macOS, unlike GNU `--output=pcent`):
    // the capacity column is the one field ending in '%'. Single fork is OK
    // at TUI refresh rate.
    if let Some(s) = cmd_stdout("df", &["-P", path]) {
        for line in s.lines().skip(1) {
            for field in line.split_whitespace() {
                if let Some(num) = field.strip_suffix('%') {
                    if let Ok(n) = num.parse::<u8>() {
                        return n;
                    }
                }
            }
        }
    }
    0
}
