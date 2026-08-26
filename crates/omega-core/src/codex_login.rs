//! Persistent, headless Codex device authentication.
//!
//! `codex login --device-auth` removes the native `auth.json` before the
//! browser flow completes. Omega therefore records one exclusive flow, keeps
//! its backup and output in a unique owner-only directory, and supervises the
//! exact child until its exit code has been persisted. A shallow
//! `codex login status` is topology information only; it can never prove that
//! a recorded flow succeeded.

use anyhow::{Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
#[cfg(all(
    target_os = "linux",
    any(
        target_arch = "x86_64",
        target_arch = "x86",
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "riscv64",
        target_arch = "s390x",
        target_arch = "powerpc64"
    )
))]
use std::{
    os::fd::{AsRawFd, FromRawFd},
    os::raw::c_long,
};

use crate::credentials::{
    codex_credential_fresh_relative, inspect_codex_credential, CodexCredentialValidity,
    CredentialStore,
};

/// Where the device-code flow sends the operator. Static; only the code rotates.
pub const DEVICE_URL: &str = "https://auth.openai.com/codex/device";

const CODE_TIMEOUT: Duration = Duration::from_secs(25);
const STATUS_TIMEOUT: Duration = Duration::from_secs(10);
const PROBE_TIMEOUT: Duration = Duration::from_secs(30);
const TERMINATE_TIMEOUT: Duration = Duration::from_secs(5);
const SUPERVISOR_SHELL: &str = "/bin/sh";
const SUPERVISOR_GATE_TOKEN: &str = "omega-codex-login-go";

#[derive(Debug, Clone)]
struct LoginPaths {
    codex_home: PathBuf,
    omega_dir: PathBuf,
}

impl LoginPaths {
    fn current() -> Self {
        Self {
            codex_home: crate::credentials::codex_home_dir(),
            omega_dir: crate::config::omega_dir(),
        }
    }

    #[cfg(test)]
    fn new(codex_home: PathBuf, omega_dir: PathBuf) -> Self {
        Self {
            codex_home,
            omega_dir,
        }
    }

    fn native_auth(&self) -> PathBuf {
        self.codex_home.join("auth.json")
    }

    fn state_root(&self) -> PathBuf {
        self.omega_dir.join("state").join("codex-login")
    }

    fn flow_root(&self) -> PathBuf {
        self.state_root().join("flows")
    }

    fn lock(&self) -> PathBuf {
        self.state_root().join("flow.lock")
    }

    fn active(&self) -> PathBuf {
        self.state_root().join("active.json")
    }

    fn flow_dir(&self, id: &str) -> PathBuf {
        self.flow_root().join(id)
    }

    fn backup(&self, id: &str) -> PathBuf {
        self.flow_dir(id).join("backup.json")
    }

    fn log(&self, id: &str) -> PathBuf {
        self.flow_dir(id).join("device-login.log")
    }

    fn exit_status(&self, id: &str) -> PathBuf {
        self.flow_dir(id).join("child-exit")
    }

    fn store(&self) -> Result<CredentialStore> {
        CredentialStore::from_roots(&self.omega_dir, &self.codex_home)
    }
}

#[derive(Debug, Clone)]
struct Runtime {
    codex_bin: PathBuf,
    code_timeout: Duration,
    status_timeout: Duration,
    probe_timeout: Duration,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            codex_bin: PathBuf::from("codex"),
            code_timeout: CODE_TIMEOUT,
            status_timeout: STATUS_TIMEOUT,
            probe_timeout: PROBE_TIMEOUT,
        }
    }
}

/// The device-code challenge to show the operator.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceLogin {
    pub url: String,
    pub code: String,
    /// PID of Omega's recorded supervisor, not an arbitrary Codex process.
    pub pid: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum LoginStatus {
    /// `Logged in using ChatGPT` / `Logged in using an API key`.
    LoggedIn {
        mode: String,
    },
    NotLoggedIn,
    /// The status command could not establish either known state. Unknown is
    /// informational and must never trigger credential restoration or signals.
    Unknown {
        reason: String,
    },
}

/// The result of settling one recorded device flow.
#[derive(Debug, Clone)]
pub struct FinishResult {
    pub status: LoginStatus,
    pub restored: bool,
    /// True only when the recorded child exited zero and produced a valid,
    /// flow-fresh native credential.
    pub flow_succeeded: bool,
}

/// Result of an explicit abort request. `aborted` is true only when Omega sent
/// TERM through a stable process handle and subsequently observed the owned
/// supervisor's exit sentinel. A flow that had already exited is merely
/// settled and never reported as aborted.
#[derive(Debug, Clone)]
pub struct AbortResult {
    pub status: LoginStatus,
    pub restored: bool,
    pub flow_succeeded: bool,
    pub aborted: bool,
}

fn abort_result(result: FinishResult, aborted: bool) -> AbortResult {
    AbortResult {
        status: result.status,
        restored: result.restored,
        flow_succeeded: result.flow_succeeded,
        aborted,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthProbe {
    Usable,
    Unauthenticated,
    QuotaLimited,
    Unknown { reason: String },
}

#[derive(Debug, Clone)]
pub struct LoginDiagnostics {
    pub active_flow: bool,
    pub active_pid: Option<u32>,
    pub active_process: String,
    pub stale_backups: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupReconcile {
    Reconciled,
    DeferredActiveFlow,
    DeferredLocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FlowRecord {
    id: String,
    supervisor_pid: u32,
    process_identity: Option<String>,
    /// The exact argv observed while the supervisor was blocked on its launch
    /// gate. Legacy records do not have it and therefore fail closed.
    #[serde(default)]
    process_argv: Option<Vec<String>>,
    started_unix_ms: u128,
    had_valid_backup: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordedProcess {
    Running,
    Exited,
    Unspawned,
    IdentityMismatch,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProcessProbe<T> {
    Found(T),
    Exited,
    Unknown,
}

fn set_owner_only_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).with_context(|| format!("creating {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
            .with_context(|| format!("chmod 700 {}", path.display()))?;
    }
    Ok(())
}

fn set_owner_only_file(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("chmod 600 {}", path.display()))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn prepare_state(paths: &LoginPaths) -> Result<()> {
    set_owner_only_dir(&paths.state_root())?;
    set_owner_only_dir(&paths.flow_root())?;
    Ok(())
}

struct FlowLock(File);

impl Drop for FlowLock {
    fn drop(&mut self) {
        let _ = fs2::FileExt::unlock(&self.0);
    }
}

fn try_flow_lock(paths: &LoginPaths) -> Result<FlowLock> {
    prepare_state(paths)?;
    let lock = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(paths.lock())
        .with_context(|| format!("opening {}", paths.lock().display()))?;
    set_owner_only_file(&paths.lock())?;
    lock.try_lock_exclusive()
        .context("another Codex login operation holds the flow lock")?;
    Ok(FlowLock(lock))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .context("credential state path has no parent")?;
    set_owner_only_dir(parent)?;
    let tmp = parent.join(format!(
        ".{}.tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("state"),
        std::process::id()
    ));
    let _ = std::fs::remove_file(&tmp);
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .with_context(|| format!("creating {}", tmp.display()))?;
        set_owner_only_file(&tmp)?;
        file.write_all(bytes)
            .with_context(|| format!("writing {}", tmp.display()))?;
        file.sync_all()
            .with_context(|| format!("syncing {}", tmp.display()))?;
        drop(file);
        std::fs::rename(&tmp, path)
            .with_context(|| format!("renaming {} to {}", tmp.display(), path.display()))?;
        File::open(parent)
            .with_context(|| format!("opening {}", parent.display()))?
            .sync_all()
            .with_context(|| format!("syncing {}", parent.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

fn write_record(paths: &LoginPaths, record: &FlowRecord) -> Result<()> {
    let bytes = serde_json::to_vec(record).context("serializing Codex login flow")?;
    atomic_write(&paths.active(), &bytes)
}

fn read_record(paths: &LoginPaths) -> Result<Option<FlowRecord>> {
    let bytes = match std::fs::read(paths.active()) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).with_context(|| format!("reading {}", paths.active().display())),
    };
    let record: FlowRecord =
        serde_json::from_slice(&bytes).context("parsing the active Codex login flow record")?;
    if record.id.is_empty()
        || !record
            .id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        anyhow::bail!("active Codex login flow has an invalid identifier");
    }
    Ok(Some(record))
}

fn unique_flow_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{}", std::process::id(), nanos)
}

/// Strip ANSI SGR sequences. Codex colorizes both the code and the URL.
fn strip_ansi(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == 0x1b {
            i += 1;
            if i < b.len() && b[i] == b'[' {
                i += 1;
                while i < b.len() && !(b[i] as char).is_ascii_alphabetic() {
                    i += 1;
                }
            }
            i += 1;
        } else {
            out.push(b[i] as char);
            i += 1;
        }
    }
    out
}

/// Pull the structurally rendered one-time code (`XXXX-XXXXX`) from output.
fn parse_code(out: &str) -> Option<String> {
    for tok in strip_ansi(out).split_whitespace() {
        let t = tok.trim();
        let Some((a, b)) = t.split_once('-') else {
            continue;
        };
        let ok = |s: &str, n: usize| {
            s.len() == n
                && s.chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        };
        if ok(a, 4) && ok(b, 5) {
            return Some(t.to_string());
        }
    }
    None
}

fn process_identity(pid: u32) -> ProcessProbe<String> {
    #[cfg(target_os = "linux")]
    {
        let stat = match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(stat) => stat,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return pid_existence_probe(pid)
            }
            Err(_) => return ProcessProbe::Unknown,
        };
        let Some(close) = stat.rfind(')') else {
            return ProcessProbe::Unknown;
        };
        // Fields after the command start at proc field 3. Start time is field
        // 22, therefore index 19 in this suffix.
        let Some(start) = stat
            .get(close + 1..)
            .and_then(|suffix| suffix.split_whitespace().nth(19))
        else {
            return ProcessProbe::Unknown;
        };
        ProcessProbe::Found(format!("linux-start:{start}"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let out = match Command::new("ps")
            .args(["-o", "lstart=", "-p", &pid.to_string()])
            .output()
        {
            Ok(out) => out,
            Err(_) => return pid_existence_probe(pid),
        };
        if !out.status.success() {
            return pid_existence_probe(pid);
        }
        let start = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if start.is_empty() {
            pid_existence_probe(pid)
        } else {
            ProcessProbe::Found(format!("ps-start:{start}"))
        }
    }
}

fn pid_existence_probe(pid: u32) -> ProcessProbe<String> {
    match Command::new("kill")
        .args(["-0", &pid.to_string()])
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output()
    {
        Ok(output) => classify_kill_zero_output(output.status.success(), &output.stderr),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ProcessProbe::Unknown,
        Err(_) => ProcessProbe::Unknown,
    }
}

fn classify_kill_zero_output(success: bool, stderr: &[u8]) -> ProcessProbe<String> {
    if success {
        // Existence alone cannot prove ownership or a stable process identity.
        return ProcessProbe::Unknown;
    }
    let error = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    if error.contains("no such process") {
        ProcessProbe::Exited
    } else {
        // Permission failures and unfamiliar/localized diagnostics are not
        // evidence that the process exited.
        ProcessProbe::Unknown
    }
}

fn process_argv(pid: u32) -> ProcessProbe<Vec<String>> {
    #[cfg(target_os = "linux")]
    {
        let raw = match std::fs::read(format!("/proc/{pid}/cmdline")) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return match pid_existence_probe(pid) {
                    ProcessProbe::Exited => ProcessProbe::Exited,
                    ProcessProbe::Found(_) | ProcessProbe::Unknown => ProcessProbe::Unknown,
                }
            }
            Err(_) => return ProcessProbe::Unknown,
        };
        let argv = raw
            .split(|b| *b == 0)
            .filter(|arg| !arg.is_empty())
            .map(|arg| String::from_utf8_lossy(arg).into_owned())
            .collect::<Vec<_>>();
        if argv.is_empty() {
            ProcessProbe::Unknown
        } else {
            ProcessProbe::Found(argv)
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        // `ps command=` is a rendered string and cannot preserve argv
        // boundaries safely. Unknown is preferable to classifying task prose
        // as Omega's recorded supervisor.
        ProcessProbe::Unknown
    }
}

fn expected_supervisor_argv(status_path: &str, codex_bin: &Path) -> Vec<String> {
    vec![
        SUPERVISOR_SHELL.to_string(),
        "-c".to_string(),
        supervisor_script().to_string(),
        "omega-codex-login-supervisor".to_string(),
        status_path.to_string(),
        codex_bin.as_os_str().to_string_lossy().into_owned(),
        "login".to_string(),
        "--device-auth".to_string(),
    ]
}

fn supervisor_launch_argv_matches(argv: &[String], status_path: &str, codex_bin: &Path) -> bool {
    argv == expected_supervisor_argv(status_path, codex_bin)
}

fn classify_recorded_process(
    expected_identity: Option<&str>,
    expected_argv: Option<&[String]>,
    identity: ProcessProbe<String>,
    argv: ProcessProbe<Vec<String>>,
) -> RecordedProcess {
    let current_identity = match identity {
        ProcessProbe::Found(identity) => identity,
        ProcessProbe::Exited => return RecordedProcess::Exited,
        ProcessProbe::Unknown => return RecordedProcess::Unknown,
    };
    let Some(expected_identity) = expected_identity else {
        return RecordedProcess::Unknown;
    };
    if current_identity != expected_identity {
        return RecordedProcess::IdentityMismatch;
    }
    let Some(expected_argv) = expected_argv else {
        return RecordedProcess::Unknown;
    };
    let argv = match argv {
        ProcessProbe::Found(argv) => argv,
        ProcessProbe::Exited => return RecordedProcess::Exited,
        ProcessProbe::Unknown => return RecordedProcess::Unknown,
    };
    if argv == expected_argv {
        RecordedProcess::Running
    } else {
        RecordedProcess::IdentityMismatch
    }
}

fn recorded_process(_paths: &LoginPaths, record: &FlowRecord) -> RecordedProcess {
    if record.supervisor_pid == 0 {
        return RecordedProcess::Unspawned;
    }
    classify_recorded_process(
        record.process_identity.as_deref(),
        record.process_argv.as_deref(),
        process_identity(record.supervisor_pid),
        process_argv(record.supervisor_pid),
    )
}

fn copy_valid_backup(source: &Path, destination: &Path) -> Result<bool> {
    if !inspect_codex_credential(source).is_valid() {
        return Ok(false);
    }
    let bytes = std::fs::read(source).with_context(|| {
        format!(
            "reading the canonical Codex credential at {}",
            source.display()
        )
    })?;
    atomic_write(destination, &bytes)?;
    Ok(true)
}

fn supervisor_script() -> &'static str {
    r#"
status_file=$1
shift
write_status() {
  status_tmp="${status_file}.tmp.$$"
  (umask 077 && printf '%s\n' "$1" > "$status_tmp") &&
    mv -f "$status_tmp" "$status_file"
}
if ! IFS= read -r launch_gate || [ "$launch_gate" != "omega-codex-login-go" ]; then
  write_status 125
  exit 125
fi
"$@" &
child=$!
terminate_child() {
  kill -TERM "$child" 2>/dev/null || true
  wait "$child"
  child_rc=$?
  write_status "$child_rc"
  exit "$child_rc"
}
trap terminate_child TERM INT HUP
wait "$child"
child_rc=$?
write_status "$child_rc"
exit "$child_rc"
"#
}

fn spawn_supervisor(
    paths: &LoginPaths,
    runtime: &Runtime,
    record: &FlowRecord,
    log: File,
) -> Result<Child> {
    let err_log = log
        .try_clone()
        .context("cloning the Codex device log handle")?;
    Command::new(SUPERVISOR_SHELL)
        .arg("-c")
        .arg(supervisor_script())
        .arg("omega-codex-login-supervisor")
        .arg(paths.exit_status(&record.id))
        .arg(&runtime.codex_bin)
        .args(["login", "--device-auth"])
        .env("CODEX_HOME", &paths.codex_home)
        // The supervisor cannot run Codex until start_with has durably
        // recorded its real PID, identity, and exact argv and releases this
        // one-shot pipe gate. If Omega crashes, EOF makes it exit 125.
        .stdin(Stdio::piped())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(err_log))
        .spawn()
        .context("spawning the recorded Codex device-login supervisor")
}

fn capture_supervisor_identity(
    child: &mut Child,
    status_path: &str,
    codex_bin: &Path,
) -> Result<(String, Vec<String>)> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                anyhow::bail!(
                    "Codex login supervisor exited before launch authorization ({status})"
                )
            }
            Err(error) => return Err(error).context("checking the gated Codex login supervisor"),
            Ok(None) => {}
        }
        #[cfg(target_os = "linux")]
        match (process_identity(child.id()), process_argv(child.id())) {
            (ProcessProbe::Found(identity), ProcessProbe::Found(argv))
                if supervisor_launch_argv_matches(&argv, status_path, codex_bin) =>
            {
                return Ok((identity, argv))
            }
            (ProcessProbe::Exited, _) | (_, ProcessProbe::Exited) => {
                anyhow::bail!("Codex login supervisor exited before its identity was captured")
            }
            _ if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            _ => {
                anyhow::bail!(
                    "could not establish the gated Codex login supervisor's exact identity and argv"
                )
            }
        }
        #[cfg(not(target_os = "linux"))]
        match process_identity(child.id()) {
            ProcessProbe::Found(identity) => {
                // These platforms have no lossless /proc-style argv reader.
                // Record the exact argv Omega supplied, but abort will still
                // fail closed because it cannot revalidate or signal by a
                // stable process handle.
                return Ok((identity, expected_supervisor_argv(status_path, codex_bin)));
            }
            ProcessProbe::Exited => {
                anyhow::bail!("Codex login supervisor exited before its identity was captured")
            }
            ProcessProbe::Unknown if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            ProcessProbe::Unknown => {
                anyhow::bail!("could not establish the gated Codex login supervisor's identity")
            }
        }
    }
}

fn release_supervisor_gate(child: &mut Child) -> Result<()> {
    let mut gate = child
        .stdin
        .take()
        .context("Codex login supervisor launch gate is unavailable")?;
    gate.write_all(format!("{SUPERVISOR_GATE_TOKEN}\n").as_bytes())
        .context("authorizing the recorded Codex login supervisor")?;
    gate.flush()
        .context("flushing the Codex login supervisor launch gate")?;
    drop(gate);
    Ok(())
}

/// Stop the exact child represented by this unreaped handle. Success means
/// exit was confirmed and the handle was reaped. Any indeterminate wait is an
/// error, so callers preserve the active record and credentials.
fn terminate_child_handle(child: &mut Child) -> Result<()> {
    // `try_wait` both proves exit and reaps. Never signal by numeric PID after
    // that point because the kernel may already have reused it.
    match child.try_wait() {
        Ok(Some(_)) => return Ok(()),
        Err(error) => return Err(error).context("checking the Codex login supervisor before stop"),
        Ok(None) => {}
    }
    let pid = child.id();
    let signal = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("signalling the spawned Codex login supervisor")?;
    if !signal.success() {
        anyhow::bail!("spawned Codex login supervisor rejected TERM");
    }
    let deadline = Instant::now() + TERMINATE_TIMEOUT;
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) => {}
            Err(error) => {
                return Err(error).context("waiting for the Codex login supervisor after TERM")
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    // Do not SIGKILL only the supervisor: its Codex child could survive and
    // mutate credentials after callers restored them. Preserve active state
    // instead; a later finish/startup pass can settle after confirmed exit.
    anyhow::bail!("Codex login supervisor did not exit after TERM")
}

#[derive(Debug)]
enum StableProcessOpen {
    Open(StableProcessHandle),
    Exited,
    Unsupported,
    Unknown(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StableSignal {
    Sent,
    Exited,
}

#[derive(Debug)]
struct StableProcessHandle {
    #[cfg(all(
        target_os = "linux",
        any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv64",
            target_arch = "s390x",
            target_arch = "powerpc64"
        )
    ))]
    pidfd: File,
}

#[cfg(all(
    target_os = "linux",
    any(
        target_arch = "x86_64",
        target_arch = "x86",
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "riscv64",
        target_arch = "s390x",
        target_arch = "powerpc64"
    )
))]
unsafe extern "C" {
    fn syscall(number: c_long, ...) -> c_long;
}

#[cfg(all(
    target_os = "linux",
    any(
        target_arch = "x86_64",
        target_arch = "x86",
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "riscv64",
        target_arch = "s390x",
        target_arch = "powerpc64"
    )
))]
fn open_stable_process(pid: u32) -> StableProcessOpen {
    // pidfd_open(2) and pidfd_send_signal(2) bind all later signalling to this
    // exact process instance, even if its numeric PID exits and is reused.
    const SYS_PIDFD_OPEN: c_long = 434;
    let fd = unsafe { syscall(SYS_PIDFD_OPEN, pid as c_long, 0 as c_long) };
    if fd >= 0 {
        let pidfd = unsafe { File::from_raw_fd(fd as i32) };
        return StableProcessOpen::Open(StableProcessHandle { pidfd });
    }
    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(3) => StableProcessOpen::Exited,            // ESRCH
        Some(22 | 38) => StableProcessOpen::Unsupported, // EINVAL / ENOSYS
        _ => StableProcessOpen::Unknown(error.to_string()),
    }
}

#[cfg(not(all(
    target_os = "linux",
    any(
        target_arch = "x86_64",
        target_arch = "x86",
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "riscv64",
        target_arch = "s390x",
        target_arch = "powerpc64"
    )
)))]
fn open_stable_process(_pid: u32) -> StableProcessOpen {
    StableProcessOpen::Unsupported
}

impl StableProcessHandle {
    #[cfg(all(
        target_os = "linux",
        any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv64",
            target_arch = "s390x",
            target_arch = "powerpc64"
        )
    ))]
    fn send_term(&self) -> Result<StableSignal> {
        const SYS_PIDFD_SEND_SIGNAL: c_long = 424;
        const SIGTERM: c_long = 15;
        let result = unsafe {
            syscall(
                SYS_PIDFD_SEND_SIGNAL,
                c_long::from(self.pidfd.as_raw_fd()),
                SIGTERM,
                std::ptr::null::<std::ffi::c_void>(),
                0 as c_long,
            )
        };
        if result == 0 {
            return Ok(StableSignal::Sent);
        }
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(3) {
            return Ok(StableSignal::Exited);
        }
        Err(error).context("signalling the exact Codex login supervisor through pidfd")
    }

    #[cfg(not(all(
        target_os = "linux",
        any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv64",
            target_arch = "s390x",
            target_arch = "powerpc64"
        )
    )))]
    fn send_term(&self) -> Result<StableSignal> {
        anyhow::bail!("stable process signalling is unavailable on this platform")
    }
}

fn read_exit_code(paths: &LoginPaths, record: &FlowRecord) -> Result<Option<i32>> {
    let raw = match std::fs::read_to_string(paths.exit_status(&record.id)) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(e)
                .with_context(|| format!("reading {}", paths.exit_status(&record.id).display()))
        }
    };
    let code = raw
        .trim()
        .parse::<i32>()
        .context("parsing the recorded Codex child exit status")?;
    Ok(Some(code))
}

fn cleanup_flow(paths: &LoginPaths, record: &FlowRecord) -> Result<()> {
    match std::fs::remove_file(paths.active()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("removing {}", paths.active().display()))
        }
    }
    match std::fs::remove_dir_all(paths.flow_dir(&record.id)) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("removing {}", paths.flow_dir(&record.id).display()))
        }
    }
    Ok(())
}

fn expected_native_link(paths: &LoginPaths, store: &CredentialStore) -> bool {
    let topology = store.codex_topology();
    topology.native_path == paths.native_auth() && topology.native_links_to_canonical
}

fn structural_status(store: &CredentialStore) -> LoginStatus {
    let canonical = store.active_path("codex");
    if !inspect_codex_credential(&canonical).is_valid() {
        return LoginStatus::NotLoggedIn;
    }
    let mode = std::fs::read(&canonical)
        .ok()
        .and_then(|raw| serde_json::from_slice::<serde_json::Value>(&raw).ok())
        .and_then(|value| {
            if value.get("auth_mode").and_then(|mode| mode.as_str()) == Some("chatgpt")
                || value.get("tokens").is_some()
            {
                Some("ChatGPT".to_string())
            } else if value
                .get("OPENAI_API_KEY")
                .and_then(|key| key.as_str())
                .is_some_and(|key| !key.is_empty())
            {
                Some("API key".to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "canonical credential".to_string());
    LoginStatus::LoggedIn { mode }
}

fn candidate_belongs_to_flow(paths: &LoginPaths, record: &FlowRecord, backup: &Path) -> bool {
    let native = paths.native_auth();
    let Ok(meta) = std::fs::symlink_metadata(&native) else {
        return false;
    };
    if !meta.file_type().is_file() {
        return false;
    }
    let inspection = inspect_codex_credential(&native);
    if inspection.validity != CodexCredentialValidity::Valid {
        return false;
    }
    let candidate_ms = inspection
        .modified
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    // Filesystems with coarse mtimes can round down. A two-second tolerance
    // still rejects an old pre-flow file while allowing a same-tick write.
    if candidate_ms.saturating_add(2_000) < record.started_unix_ms {
        return false;
    }
    if record.had_valid_backup {
        codex_credential_fresh_relative(&native, backup)
    } else {
        true
    }
}

fn settle_abandoned(
    paths: &LoginPaths,
    store: &CredentialStore,
    record: &FlowRecord,
) -> Result<FinishResult> {
    let backup = paths.backup(&record.id);
    let was_disrupted = !expected_native_link(paths, store)
        || !inspect_codex_credential(&store.active_path("codex")).is_valid();
    if record.had_valid_backup && backup.exists() {
        store.reconcile_codex_candidate(&backup)
    } else {
        store.reconcile_codex()
    }
    .context("reconciling credentials for an abandoned Codex login flow")?;
    let status = structural_status(store);
    let restored = was_disrupted
        && record.had_valid_backup
        && matches!(status, LoginStatus::LoggedIn { .. })
        && expected_native_link(paths, store);
    cleanup_flow(paths, record)?;
    Ok(FinishResult {
        status,
        restored,
        flow_succeeded: false,
    })
}

fn settle_success(
    paths: &LoginPaths,
    store: &CredentialStore,
    record: &FlowRecord,
) -> Result<FinishResult> {
    let backup = paths.backup(&record.id);
    if !candidate_belongs_to_flow(paths, record, &backup) {
        return settle_abandoned(paths, store, record);
    }
    store
        .reconcile_codex()
        .context("adopting the successful Codex login credential")?;
    // Feed the backup through the same monotonic reconciler. If it is a
    // distinct valid loser it is preserved in the owner-only quarantine; it
    // can never roll the new credential back.
    if record.had_valid_backup && backup.exists() {
        store
            .reconcile_codex_candidate(&backup)
            .context("preserving the previous Codex credential")?;
    }
    let status = structural_status(store);
    if !matches!(status, LoginStatus::LoggedIn { .. }) || !expected_native_link(paths, store) {
        anyhow::bail!("successful Codex flow did not produce canonical credential topology");
    }
    cleanup_flow(paths, record)?;
    Ok(FinishResult {
        status,
        restored: false,
        flow_succeeded: true,
    })
}

fn settlement_failure(context: &str, error: anyhow::Error) -> FinishResult {
    FinishResult {
        status: LoginStatus::Unknown {
            reason: format!("{context}: {error:#}"),
        },
        restored: false,
        flow_succeeded: false,
    }
}

fn parse_status_output(success: bool, stdout: &[u8], stderr: &[u8]) -> LoginStatus {
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(stdout),
        String::from_utf8_lossy(stderr)
    );
    let clean = strip_ansi(&text);
    // A non-zero status command may print stale or diagnostic text containing
    // the success phrase. Never promote that failed command to authenticated.
    if success {
        if let Some((_, rest)) = clean.split_once("Logged in using") {
            return LoginStatus::LoggedIn {
                mode: rest.trim().lines().next().unwrap_or("?").trim().to_string(),
            };
        }
    }
    if clean.to_ascii_lowercase().contains("not logged in") {
        return LoginStatus::NotLoggedIn;
    }
    LoginStatus::Unknown {
        reason: if success {
            "unparseable `codex login status` output"
        } else {
            "`codex login status` exited without a recognized status"
        }
        .to_string(),
    }
}

fn status_with(paths: &LoginPaths, runtime: &Runtime) -> LoginStatus {
    let mut child = match Command::new(&runtime.codex_bin)
        .args(["login", "status"])
        .env("CODEX_HOME", &paths.codex_home)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            return LoginStatus::Unknown {
                reason: "could not spawn `codex login status`".to_string(),
            }
        }
    };
    let deadline = Instant::now() + runtime.status_timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return match child.wait_with_output() {
                    Ok(output) => {
                        parse_status_output(status.success(), &output.stdout, &output.stderr)
                    }
                    Err(_) => LoginStatus::Unknown {
                        reason: "could not collect `codex login status` output".to_string(),
                    },
                }
            }
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Ok(None) => {
                // Deliberately do not kill the status process. Unknown is
                // observation-only and never signals or restores anything.
                let _ = std::thread::Builder::new()
                    .name("codex-status-reaper".to_string())
                    .spawn(move || {
                        let _ = child.wait();
                    });
                return LoginStatus::Unknown {
                    reason: "`codex login status` timed out".to_string(),
                };
            }
            Err(_) => {
                return LoginStatus::Unknown {
                    reason: "could not wait for `codex login status`".to_string(),
                }
            }
        }
    }
}

/// Topology-only status. Spawn, timeout, and parse failures are Unknown.
pub fn status() -> LoginStatus {
    status_with(&LoginPaths::current(), &Runtime::default())
}

fn start_with(paths: &LoginPaths, runtime: &Runtime) -> Result<DeviceLogin> {
    let _lock = try_flow_lock(paths)?;
    if let Some(active) = read_record(paths)? {
        anyhow::bail!(
            "Codex device login flow {} is already recorded (pid {}); settle it before starting another",
            active.id,
            active.supervisor_pid
        );
    }

    let store = paths.store()?;
    store
        .reconcile_codex()
        .context("reconciling Codex credentials before device login")?;

    let id = unique_flow_id();
    let flow_dir = paths.flow_dir(&id);
    set_owner_only_dir(&flow_dir)?;
    let started_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let had_valid_backup = copy_valid_backup(&store.active_path("codex"), &paths.backup(&id))?;
    let mut record = FlowRecord {
        id,
        supervisor_pid: 0,
        process_identity: None,
        process_argv: None,
        started_unix_ms,
        had_valid_backup,
    };
    // Persist a starting record before spawning so another Omega process never
    // heals the native symlink into the middle of this destructive window.
    write_record(paths, &record)?;

    let log_path = paths.log(&record.id);
    let log = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&log_path)
        .with_context(|| format!("creating {}", log_path.display()))
        .and_then(|log| {
            set_owner_only_file(&log_path)?;
            Ok(log)
        });
    let log = match log {
        Ok(log) => log,
        Err(error) => {
            return match settle_abandoned(paths, &store, &record) {
                Ok(_) => Err(error),
                Err(settlement) => Err(error).context(format!(
                    "creating the device log failed; credential reconciliation also failed: {settlement:#}"
                )),
            };
        }
    };
    let mut child = match spawn_supervisor(paths, runtime, &record, log) {
        Ok(child) => child,
        Err(e) => {
            if let Err(settlement) = settle_abandoned(paths, &store, &record) {
                return Err(e).context(format!(
                    "spawn failed and credential reconciliation also failed: {settlement:#}"
                ));
            }
            return Err(e);
        }
    };
    record.supervisor_pid = child.id();
    let status_path = paths.exit_status(&record.id).display().to_string();
    let (identity, argv) = match capture_supervisor_identity(
        &mut child,
        &status_path,
        &runtime.codex_bin,
    ) {
        Ok(captured) => captured,
        Err(capture_error) => {
            if let Err(stop_error) = terminate_child_handle(&mut child) {
                return Err(capture_error).context(format!(
                        "supervisor identity capture failed and exit could not be confirmed; active flow preserved: {stop_error:#}"
                    ));
            }
            settle_abandoned(paths, &store, &record)
                .context("restoring credentials after supervisor identity failure")?;
            return Err(capture_error);
        }
    };
    record.process_identity = Some(identity);
    record.process_argv = Some(argv);
    if let Err(e) = write_record(paths, &record) {
        if let Err(stop_error) = terminate_child_handle(&mut child) {
            return Err(e).context(format!(
                "supervisor record failed and exit could not be confirmed; active flow preserved: {stop_error:#}"
            ));
        }
        settle_abandoned(paths, &store, &record)
            .context("restoring credentials after supervisor record failure")?;
        return Err(e).context("persisting the Codex supervisor record");
    }
    if let Err(e) = release_supervisor_gate(&mut child) {
        if let Err(stop_error) = terminate_child_handle(&mut child) {
            return Err(e).context(format!(
                "launch authorization failed and exit could not be confirmed; active flow preserved: {stop_error:#}"
            ));
        }
        settle_abandoned(paths, &store, &record)
            .context("restoring credentials after launch authorization failure")?;
        return Err(e);
    }

    let deadline = Instant::now() + runtime.code_timeout;
    while Instant::now() < deadline {
        if let Ok(out) = std::fs::read_to_string(&log_path) {
            if let Some(code) = parse_code(&out) {
                return Ok(DeviceLogin {
                    url: DEVICE_URL.to_string(),
                    code,
                    pid: child.id(),
                });
            }
        }
        if matches!(child.try_wait(), Ok(Some(_))) {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    terminate_child_handle(&mut child).context(
        "Codex did not provide a device code and supervisor exit could not be confirmed; active flow preserved",
    )?;
    settle_abandoned(paths, &store, &record).context(
        "Codex did not provide a device code and canonical credential reconciliation failed",
    )?;
    anyhow::bail!(
        "Codex did not provide a device code; the recorded flow was stopped and canonical credential topology restored"
    )
}

/// Start one exclusive recorded device-code flow.
pub fn start() -> Result<DeviceLogin> {
    start_with(&LoginPaths::current(), &Runtime::default())
}

fn finish_with(paths: &LoginPaths, runtime: &Runtime, requested_pid: Option<u32>) -> FinishResult {
    let _lock = match try_flow_lock(paths) {
        Ok(lock) => lock,
        Err(_) => {
            return FinishResult {
                status: LoginStatus::Unknown {
                    reason: "Codex login flow is busy".to_string(),
                },
                restored: false,
                flow_succeeded: false,
            }
        }
    };
    let record = match read_record(paths) {
        Ok(Some(record)) => record,
        Ok(None) => {
            return FinishResult {
                status: LoginStatus::Unknown {
                    reason: "no recorded Codex login flow".to_string(),
                },
                restored: false,
                flow_succeeded: false,
            }
        }
        Err(_) => {
            return FinishResult {
                status: LoginStatus::Unknown {
                    reason: "active Codex login flow record is unreadable".to_string(),
                },
                restored: false,
                flow_succeeded: false,
            }
        }
    };
    let store = match paths.store() {
        Ok(store) => store,
        Err(_) => {
            return FinishResult {
                status: LoginStatus::Unknown {
                    reason: "could not open the canonical credential store".to_string(),
                },
                restored: false,
                flow_succeeded: false,
            }
        }
    };
    if record.supervisor_pid == 0 {
        return settle_abandoned(paths, &store, &record).unwrap_or_else(|error| {
            settlement_failure("could not settle an unspawned Codex login flow", error)
        });
    }
    if requested_pid.is_some_and(|pid| pid != record.supervisor_pid) {
        return FinishResult {
            status: LoginStatus::Unknown {
                reason: "requested pid does not own the active Codex login flow".to_string(),
            },
            restored: false,
            flow_succeeded: false,
        };
    }

    match read_exit_code(paths, &record) {
        Ok(Some(0)) => {
            return settle_success(paths, &store, &record).unwrap_or_else(|error| {
                settlement_failure("could not settle successful Codex login flow", error)
            })
        }
        Ok(Some(_)) => {
            return settle_abandoned(paths, &store, &record).unwrap_or_else(|error| {
                settlement_failure("could not settle abandoned Codex login flow", error)
            })
        }
        Err(_) => {
            return FinishResult {
                status: LoginStatus::Unknown {
                    reason: "recorded Codex child exit status is unreadable".to_string(),
                },
                restored: false,
                flow_succeeded: false,
            }
        }
        Ok(None) => {}
    }

    match recorded_process(paths, &record) {
        RecordedProcess::Unspawned => {
            // The launch gate proves Codex never ran while the record still
            // had pid=0, so this state can be settled safely.
            settle_abandoned(paths, &store, &record).unwrap_or_else(|error| {
                settlement_failure("could not settle unspawned Codex login flow", error)
            })
        }
        RecordedProcess::Exited => FinishResult {
            // Without the supervisor-written sentinel, its background Codex
            // child may have outlived the shell. Preserve credentials and flow
            // state until an operator can establish a terminal outcome.
            status: LoginStatus::Unknown {
                reason:
                    "recorded supervisor exited without an exit sentinel; no restore was attempted"
                        .to_string(),
            },
            restored: false,
            flow_succeeded: false,
        },
        RecordedProcess::IdentityMismatch => FinishResult {
            status: LoginStatus::Unknown {
                reason: "recorded Codex supervisor identity does not match; no signal or restore was attempted".to_string(),
            },
            restored: false,
            flow_succeeded: false,
        },
        RecordedProcess::Unknown => FinishResult {
            status: LoginStatus::Unknown {
                reason: "recorded Codex supervisor identity is indeterminate".to_string(),
            },
            restored: false,
            flow_succeeded: false,
        },
        RecordedProcess::Running => match status_with(paths, runtime) {
            LoginStatus::Unknown { reason } => FinishResult {
                status: LoginStatus::Unknown { reason },
                restored: false,
                flow_succeeded: false,
            },
            LoginStatus::LoggedIn { .. } => FinishResult {
                status: LoginStatus::Unknown {
                    reason:
                        "Codex reports a login, but the recorded child has not exited successfully"
                            .to_string(),
                },
                restored: false,
                flow_succeeded: false,
            },
            LoginStatus::NotLoggedIn => FinishResult {
                status: LoginStatus::NotLoggedIn,
                restored: false,
                flow_succeeded: false,
            },
        },
    }
}

/// Settle the single recorded device flow. The optional pid is a match check;
/// it is never used directly as a signal target.
pub fn finish(pid: Option<u32>) -> FinishResult {
    finish_with(&LoginPaths::current(), &Runtime::default(), pid)
}

fn abort_unknown(reason: impl Into<String>) -> AbortResult {
    abort_result(
        FinishResult {
            status: LoginStatus::Unknown {
                reason: reason.into(),
            },
            restored: false,
            flow_succeeded: false,
        },
        false,
    )
}

fn abort_settlement_failure(context: &str, error: anyhow::Error) -> AbortResult {
    abort_result(settlement_failure(context, error), false)
}

/// Settle a supervisor that was already gone before Omega sent any signal.
/// Missing/unreadable sentinel state is Unknown: the shell wrapper may have
/// died while its background Codex child remained capable of writing auth.
fn settle_preexisting_abort_exit(
    paths: &LoginPaths,
    store: &CredentialStore,
    record: &FlowRecord,
) -> AbortResult {
    let result = match read_exit_code(paths, record) {
        Ok(Some(0)) => settle_success(paths, store, record).unwrap_or_else(|error| {
            settlement_failure("could not settle completed Codex login flow", error)
        }),
        Ok(Some(_)) => settle_abandoned(paths, store, record).unwrap_or_else(|error| {
            settlement_failure("could not settle exited Codex login flow", error)
        }),
        Ok(None) => {
            return abort_unknown(
                "recorded supervisor exited without an exit sentinel; no restore was attempted",
            )
        }
        Err(error) => {
            return abort_settlement_failure(
                "recorded Codex child exit status is unreadable",
                error,
            )
        }
    };
    abort_result(result, false)
}

fn abort_with(paths: &LoginPaths, requested_pid: u32) -> AbortResult {
    let _lock = match try_flow_lock(paths) {
        Ok(lock) => lock,
        Err(_) => {
            return abort_settlement_failure(
                "Codex login flow is busy",
                anyhow::anyhow!("flow lock unavailable"),
            )
        }
    };
    let record = match read_record(paths) {
        Ok(Some(record)) => record,
        Ok(None) => {
            return abort_settlement_failure(
                "no recorded Codex login flow",
                anyhow::anyhow!("no active flow"),
            )
        }
        Err(error) => {
            return abort_settlement_failure("active Codex login flow record is unreadable", error)
        }
    };
    if requested_pid == 0 || requested_pid != record.supervisor_pid {
        return abort_unknown("requested pid does not own the active Codex login flow");
    }
    let store = match paths.store() {
        Ok(store) => store,
        Err(error) => {
            return abort_settlement_failure("could not open the canonical credential store", error)
        }
    };

    match read_exit_code(paths, &record) {
        Ok(Some(0)) => {
            let result = settle_success(paths, &store, &record).unwrap_or_else(|error| {
                settlement_failure("could not settle completed Codex login flow", error)
            });
            return abort_result(result, false);
        }
        Ok(Some(_)) => {
            let result = settle_abandoned(paths, &store, &record).unwrap_or_else(|error| {
                settlement_failure("could not settle exited Codex login flow", error)
            });
            return abort_result(result, false);
        }
        Ok(None) => {}
        Err(error) => {
            return abort_settlement_failure(
                "recorded Codex child exit status is unreadable",
                error,
            )
        }
    }

    let stable_process = match open_stable_process(record.supervisor_pid) {
        StableProcessOpen::Open(handle) => handle,
        StableProcessOpen::Exited => {
            return settle_preexisting_abort_exit(paths, &store, &record);
        }
        StableProcessOpen::Unsupported => {
            // On platforms without pidfd, absence can still be classified and
            // settled safely. A live/unknown PID is never signalled numerically.
            if matches!(
                process_identity(record.supervisor_pid),
                ProcessProbe::Exited
            ) {
                return settle_preexisting_abort_exit(paths, &store, &record);
            }
            return abort_unknown(
                "stable process signalling is unavailable on this platform; no signal or restore was attempted",
            );
        }
        StableProcessOpen::Unknown(reason) => {
            return abort_unknown(format!(
                "could not bind the recorded supervisor to a stable process handle: {reason}"
            ))
        }
    };

    // Re-read both immutable process identity and the literally exact argv
    // immediately before signalling. The pidfd remains bound to this process
    // instance across the check and signal, eliminating numeric PID reuse.
    match recorded_process(paths, &record) {
        RecordedProcess::Running => {}
        RecordedProcess::Exited => {
            return settle_preexisting_abort_exit(paths, &store, &record);
        }
        RecordedProcess::IdentityMismatch => {
            return abort_unknown(
                "recorded supervisor identity or exact argv does not match; no signal or restore was attempted",
            )
        }
        RecordedProcess::Unspawned | RecordedProcess::Unknown => {
            return abort_unknown(
                "recorded supervisor identity is indeterminate; no signal or restore was attempted",
            )
        }
    }

    match stable_process.send_term() {
        Ok(StableSignal::Sent) => {}
        Ok(StableSignal::Exited) => {
            return settle_preexisting_abort_exit(paths, &store, &record);
        }
        Err(error) => {
            return abort_settlement_failure("owned Codex supervisor could not be stopped", error)
        }
    }

    let deadline = Instant::now() + TERMINATE_TIMEOUT;
    while Instant::now() < deadline {
        match read_exit_code(paths, &record) {
            Ok(Some(code)) => {
                let result = if code == 0 {
                    settle_success(paths, &store, &record).unwrap_or_else(|error| {
                        settlement_failure("could not settle completed Codex login flow", error)
                    })
                } else {
                    settle_abandoned(paths, &store, &record).unwrap_or_else(|error| {
                        settlement_failure("could not settle aborted Codex login flow", error)
                    })
                };
                return abort_result(result, true);
            }
            Ok(None) => {}
            Err(error) => {
                return abort_settlement_failure(
                    "recorded Codex child exit status became unreadable",
                    error,
                )
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    abort_unknown("owned supervisor did not publish its exit sentinel")
}

/// Stop and settle only the recorded supervisor whose identity and exact argv match.
pub fn abort(pid: u32) -> AbortResult {
    abort_with(&LoginPaths::current(), pid)
}

fn auth_diagnostic(text: &str) -> Option<AuthProbe> {
    let lower = text.to_ascii_lowercase();
    let unauthenticated = [
        "401",
        "401 unauthorized",
        "token_expired",
        "token_invalidated",
        "refresh_token_invalidated",
        "access token could not be refreshed",
        "refresh token was revoked",
        "not logged in",
        "log out/sign in again",
        "run codex login",
    ];
    if unauthenticated.iter().any(|needle| lower.contains(needle)) {
        return Some(AuthProbe::Unauthenticated);
    }
    let quota = [
        "usage limit",
        "quota",
        "too many requests",
        "429",
        "insufficient_quota",
        "plan limit",
    ];
    quota
        .iter()
        .any(|needle| lower.contains(needle))
        .then_some(AuthProbe::QuotaLimited)
}

fn exact_auth_marker(stdout: &str) -> bool {
    strip_ansi(stdout).trim() == "AUTH_OK"
}

fn probe_auth_with(paths: &LoginPaths, runtime: &Runtime, cwd: &Path) -> AuthProbe {
    let prompt = "Reply with exactly AUTH_OK";
    let mut child = match Command::new(&runtime.codex_bin)
        .args([
            "exec",
            "--sandbox",
            "read-only",
            "--skip-git-repo-check",
            "-",
        ])
        .current_dir(cwd)
        .env("CODEX_HOME", &paths.codex_home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            return AuthProbe::Unknown {
                reason: "could not spawn Codex auth probe".to_string(),
            }
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        if stdin.write_all(prompt.as_bytes()).is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return AuthProbe::Unknown {
                reason: "could not deliver the Codex auth probe".to_string(),
            };
        }
    }
    let deadline = Instant::now() + runtime.probe_timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = match child.wait_with_output() {
                    Ok(output) => output,
                    Err(_) => {
                        return AuthProbe::Unknown {
                            reason: "could not collect the Codex auth probe".to_string(),
                        }
                    }
                };
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let diagnostics = format!("{}\n{}", stdout, stderr).replace(prompt, "");
                if let Some(classified) = auth_diagnostic(&diagnostics) {
                    return classified;
                }
                if status.success() && exact_auth_marker(&stdout) {
                    return AuthProbe::Usable;
                }
                return AuthProbe::Unknown {
                    reason: "Codex auth probe returned no verifiable marker".to_string(),
                };
            }
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Ok(None) => {
                // This is Omega's own probe child, never a pre-existing Codex
                // or rmux session.
                let _ = child.kill();
                let _ = child.wait();
                return AuthProbe::Unknown {
                    reason: "Codex auth probe timed out".to_string(),
                };
            }
            Err(_) => {
                return AuthProbe::Unknown {
                    reason: "could not wait for the Codex auth probe".to_string(),
                }
            }
        }
    }
}

/// Prove that the currently resolved Codex credential can complete a real,
/// read-only request. No token or raw provider output is returned.
pub fn probe_auth() -> AuthProbe {
    let paths = LoginPaths::current();
    let _lock = match try_flow_lock(&paths) {
        Ok(lock) => lock,
        Err(_) => {
            return AuthProbe::Unknown {
                reason: "Codex auth probe skipped while login state is busy".to_string(),
            }
        }
    };
    match read_record(&paths) {
        Ok(Some(_)) | Err(_) => {
            return AuthProbe::Unknown {
                reason: "Codex auth probe skipped while a login flow is recorded".to_string(),
            }
        }
        Ok(None) => {}
    }

    let probe_root = paths.state_root().join("auth-probes");
    let cwd = probe_root.join(unique_flow_id());
    if set_owner_only_dir(&cwd).is_err() {
        return AuthProbe::Unknown {
            reason: "could not create an isolated Codex auth probe directory".to_string(),
        };
    }
    let result = probe_auth_with(&paths, &Runtime::default(), &cwd);
    let _ = std::fs::remove_dir(&cwd);
    let _ = std::fs::remove_dir(&probe_root);
    result
}

/// Reconcile Codex on normal Omega startup, unless an exclusive login flow is
/// recorded or another process currently owns the flow lock.
pub fn reconcile_on_startup() -> Result<StartupReconcile> {
    reconcile_on_startup_with(&LoginPaths::current())
}

fn reconcile_on_startup_with(paths: &LoginPaths) -> Result<StartupReconcile> {
    prepare_state(paths)?;
    let lock = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(paths.lock())
        .with_context(|| format!("opening {}", paths.lock().display()))?;
    set_owner_only_file(&paths.lock())?;
    if lock.try_lock_exclusive().is_err() {
        return Ok(StartupReconcile::DeferredLocked);
    }
    let _lock = FlowLock(lock);
    let store = paths.store()?;
    let Some(record) = read_record(paths)? else {
        store.reconcile_codex()?;
        return Ok(StartupReconcile::Reconciled);
    };

    match read_exit_code(paths, &record)? {
        Some(0) => {
            settle_success(paths, &store, &record)?;
            return Ok(StartupReconcile::Reconciled);
        }
        Some(_) => {
            settle_abandoned(paths, &store, &record)?;
            return Ok(StartupReconcile::Reconciled);
        }
        None => {}
    }

    match recorded_process(paths, &record) {
        RecordedProcess::Running => Ok(StartupReconcile::DeferredActiveFlow),
        RecordedProcess::Exited => Ok(StartupReconcile::DeferredActiveFlow),
        RecordedProcess::IdentityMismatch => Ok(StartupReconcile::DeferredActiveFlow),
        RecordedProcess::Unspawned => {
            settle_abandoned(paths, &store, &record)?;
            Ok(StartupReconcile::Reconciled)
        }
        RecordedProcess::Unknown => Ok(StartupReconcile::DeferredActiveFlow),
    }
}

/// Read-only flow diagnostics. No process is signalled and no credential is
/// changed.
pub fn diagnostics() -> LoginDiagnostics {
    diagnostics_with(&LoginPaths::current())
}

fn diagnostics_with(paths: &LoginPaths) -> LoginDiagnostics {
    let (active, unreadable_record) = match read_record(paths) {
        Ok(active) => (active, false),
        Err(_) => (None, true),
    };
    let active_id = active.as_ref().map(|record| record.id.as_str());
    let active_process = if unreadable_record {
        "unreadable active record (new login blocked)".to_string()
    } else {
        active
            .as_ref()
            .map(|record| match recorded_process(paths, record) {
                RecordedProcess::Running => "running",
                RecordedProcess::Exited => "exited without settlement",
                RecordedProcess::IdentityMismatch => "stale pid (unrelated process preserved)",
                RecordedProcess::Unspawned => "stale starting record",
                RecordedProcess::Unknown => "unknown",
            })
            .unwrap_or("none")
            .to_string()
    };
    let mut stale_backups = usize::from(
        paths
            .omega_dir
            .join("state")
            .join("codex-auth.backup.json")
            .exists(),
    );
    if let Ok(entries) = std::fs::read_dir(paths.flow_root()) {
        for entry in entries.flatten() {
            let id = entry.file_name();
            let id = id.to_string_lossy();
            if Some(id.as_ref()) == active_id {
                continue;
            }
            if entry.path().join("backup.json").exists() {
                stale_backups += 1;
            }
        }
    }
    LoginDiagnostics {
        active_flow: active.is_some() || unreadable_record,
        active_pid: active.as_ref().map(|record| record.supervisor_pid),
        active_process,
        stale_backups,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credential(last_refresh: &str) -> String {
        serde_json::json!({
            "auth_mode": "chatgpt",
            "tokens": {
                "id_token": "id-value",
                "access_token": "access-value",
                "refresh_token": "refresh-value"
            },
            "last_refresh": last_refresh
        })
        .to_string()
    }

    fn test_runtime(bin: &Path) -> Runtime {
        Runtime {
            codex_bin: bin.to_path_buf(),
            code_timeout: Duration::from_secs(3),
            status_timeout: Duration::from_millis(500),
            probe_timeout: Duration::from_secs(2),
        }
    }

    #[cfg(unix)]
    fn script(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, body).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }

    fn setup() -> (tempfile::TempDir, LoginPaths, CredentialStore) {
        let tmp = tempfile::tempdir().unwrap();
        let paths = LoginPaths::new(tmp.path().join("codex"), tmp.path().join("omega"));
        std::fs::create_dir_all(&paths.codex_home).unwrap();
        let store = paths.store().unwrap();
        store
            .write(
                "codex",
                &serde_json::from_str::<serde_json::Value>(&credential("2026-07-20T00:00:00Z"))
                    .unwrap(),
            )
            .unwrap();
        store.reconcile_codex().unwrap();
        (tmp, paths, store)
    }

    fn wait_for_exit(paths: &LoginPaths, pid: u32) {
        let record = read_record(paths).unwrap().unwrap();
        assert_eq!(record.supervisor_pid, pid);
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if read_exit_code(paths, &record).unwrap().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("recorded child did not persist its exit");
    }

    #[test]
    fn strips_the_sgr_codex_wraps_its_code_in() {
        assert_eq!(strip_ansi("\x1b[94mB6BU-SV81P\x1b[0m"), "B6BU-SV81P");
    }

    #[test]
    fn parses_the_code_from_real_device_auth_output() {
        let out = "Welcome to Codex [v\x1b[90m0.144.5\x1b[0m]\n\n\
                   1. Open this link in your browser\n   \
                   \x1b[94mhttps://auth.openai.com/codex/device\x1b[0m\n\n\
                   2. Enter this one-time code\n   \
                   \x1b[94mB6BU-SV81P\x1b[0m\n";
        assert_eq!(parse_code(out).as_deref(), Some("B6BU-SV81P"));
    }

    #[test]
    fn version_banner_and_device_url_are_never_codes() {
        assert_eq!(
            parse_code("Codex v0.144.5\nhttps://auth.openai.com/codex/device"),
            None
        );
        assert_eq!(parse_code(DEVICE_URL), None);
    }

    #[test]
    fn status_parser_distinguishes_unknown() {
        assert_eq!(
            parse_status_output(true, b"Logged in using ChatGPT\n", b""),
            LoginStatus::LoggedIn {
                mode: "ChatGPT".to_string()
            }
        );
        assert_eq!(
            parse_status_output(false, b"", b"Not logged in\n"),
            LoginStatus::NotLoggedIn
        );
        assert!(matches!(
            parse_status_output(true, b"unexpected", b""),
            LoginStatus::Unknown { .. }
        ));
        assert!(matches!(
            parse_status_output(false, b"Logged in using stale-cache\n", b"fatal"),
            LoginStatus::Unknown { .. }
        ));
    }

    #[test]
    fn recorded_process_classification_rejects_task_text() {
        let status_path = "/tmp/omega/child-exit";
        let codex_bin = Path::new("/usr/local/bin/codex");
        let task_argv = vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("review task text mentioning omega-codex-login-supervisor and {status_path}"),
        ];
        assert!(!supervisor_launch_argv_matches(
            &task_argv,
            status_path,
            codex_bin
        ));

        let mut supervisor_argv = expected_supervisor_argv(status_path, codex_bin);
        assert!(supervisor_launch_argv_matches(
            &supervisor_argv,
            status_path,
            codex_bin
        ));
        supervisor_argv[5] = "/different/codex".to_string();
        assert!(!supervisor_launch_argv_matches(
            &supervisor_argv,
            status_path,
            codex_bin
        ));
        supervisor_argv[5] = codex_bin.display().to_string();
        supervisor_argv.push("task text mentioning codex login".to_string());
        assert!(!supervisor_launch_argv_matches(
            &supervisor_argv,
            status_path,
            codex_bin
        ));
    }

    #[test]
    fn indeterminate_process_probes_remain_unknown() {
        let expected = Some("linux-start:123");
        let expected_argv = ["exact".to_string()];
        let argv = ProcessProbe::Found(vec!["unrelated".to_string()]);
        assert_eq!(
            classify_recorded_process(expected, Some(&expected_argv), ProcessProbe::Unknown, argv),
            RecordedProcess::Unknown
        );
        assert_eq!(
            classify_recorded_process(
                expected,
                Some(&expected_argv),
                ProcessProbe::Found("linux-start:123".to_string()),
                ProcessProbe::Unknown,
            ),
            RecordedProcess::Unknown
        );
        assert_eq!(
            classify_recorded_process(
                expected,
                None,
                ProcessProbe::Found("linux-start:123".to_string()),
                ProcessProbe::Found(expected_argv.to_vec()),
            ),
            RecordedProcess::Unknown,
            "legacy records without exact argv must fail closed"
        );
        assert_eq!(
            classify_recorded_process(
                expected,
                Some(&expected_argv),
                ProcessProbe::Found("linux-start:123".to_string()),
                ProcessProbe::Found(vec!["almost".to_string()]),
            ),
            RecordedProcess::IdentityMismatch
        );
    }

    #[test]
    fn kill_zero_output_only_classifies_explicit_absence_as_exited() {
        assert_eq!(
            classify_kill_zero_output(false, b"kill: 42: No such process\n"),
            ProcessProbe::Exited
        );
        assert_eq!(
            classify_kill_zero_output(false, b"kill: 42: Operation not permitted\n"),
            ProcessProbe::Unknown
        );
        assert_eq!(classify_kill_zero_output(true, b""), ProcessProbe::Unknown);
    }

    #[cfg(unix)]
    #[test]
    fn status_spawn_parse_and_timeout_failures_are_unknown_without_signalling() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = LoginPaths::new(tmp.path().join("codex"), tmp.path().join("omega"));

        let missing = test_runtime(Path::new("/definitely/missing/codex"));
        assert!(matches!(
            status_with(&paths, &missing),
            LoginStatus::Unknown { .. }
        ));

        let unparseable = tmp.path().join("codex-unparseable");
        script(
            &unparseable,
            "#!/bin/sh\nprintf 'unexpected status shape\\n'\nexit 0\n",
        );
        assert!(matches!(
            status_with(&paths, &test_runtime(&unparseable)),
            LoginStatus::Unknown { .. }
        ));

        let completed = tmp.path().join("status-completed");
        let killed = tmp.path().join("status-killed");
        let slow = tmp.path().join("codex-slow");
        script(
            &slow,
            &format!(
                "#!/bin/sh\ntrap \"printf killed > '{}'\" TERM INT HUP\nsleep 0.2\nprintf completed > '{}'\nprintf 'Not logged in\\n'\n",
                killed.display(),
                completed.display()
            ),
        );
        let mut slow_runtime = test_runtime(&slow);
        slow_runtime.status_timeout = Duration::from_millis(20);
        assert!(matches!(
            status_with(&paths, &slow_runtime),
            LoginStatus::Unknown { .. }
        ));
        std::thread::sleep(Duration::from_millis(400));
        assert!(
            completed.exists(),
            "timed-out status child should finish naturally"
        );
        assert!(!killed.exists(), "Unknown must not signal the status child");
    }

    #[cfg(unix)]
    #[test]
    fn supervisor_gate_eof_exits_without_running_codex() {
        let (tmp, paths, _store) = setup();
        prepare_state(&paths).unwrap();
        let id = unique_flow_id();
        set_owner_only_dir(&paths.flow_dir(&id)).unwrap();
        let marker = tmp.path().join("codex-ran");
        let fake = tmp.path().join("fake-codex-gated");
        script(
            &fake,
            &format!(
                "#!/bin/sh\nprintf ran > '{}'\nprintf 'GATE-12345\\n'\n",
                marker.display()
            ),
        );
        let record = FlowRecord {
            id,
            supervisor_pid: 0,
            process_identity: None,
            process_argv: None,
            started_unix_ms: 0,
            had_valid_backup: false,
        };
        let log_path = paths.log(&record.id);
        let log = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&log_path)
            .unwrap();
        let mut child = spawn_supervisor(&paths, &test_runtime(&fake), &record, log).unwrap();

        std::thread::sleep(Duration::from_millis(100));
        assert!(!marker.exists(), "Codex ran before launch authorization");
        drop(child.stdin.take());
        let status = child.wait().unwrap();
        assert_eq!(status.code(), Some(125));
        assert!(!marker.exists(), "Codex ran after the launch gate closed");
        assert_eq!(read_exit_code(&paths, &record).unwrap(), Some(125));
    }

    #[cfg(unix)]
    #[test]
    fn codex_observes_a_durable_owned_record_before_it_can_run() {
        let (tmp, paths, store) = setup();
        let fake = tmp.path().join("fake-codex-record-check");
        let observed = tmp.path().join("active-observed-by-codex.json");
        script(
            &fake,
            &format!(
                "#!/bin/sh\nif [ \"$1 $2\" = \"login --device-auth\" ]; then\n  cp '{}' '{}'\n  printf 'DURA-12345\\n'\n  exit 1\nfi\nexit 2\n",
                paths.active().display(),
                observed.display()
            ),
        );
        let runtime = test_runtime(&fake);
        let login = start_with(&paths, &runtime).unwrap();
        let observed_record: FlowRecord =
            serde_json::from_slice(&std::fs::read(&observed).unwrap()).unwrap();

        assert_eq!(observed_record.supervisor_pid, login.pid);
        assert!(observed_record.process_identity.is_some());
        let observed_argv = observed_record.process_argv.as_deref().unwrap();
        assert!(supervisor_launch_argv_matches(
            observed_argv,
            &paths.exit_status(&observed_record.id).display().to_string(),
            &fake
        ));

        wait_for_exit(&paths, login.pid);
        let result = finish_with(&paths, &runtime, Some(login.pid));
        assert!(!result.flow_succeeded);
        assert!(expected_native_link(&paths, &store));
    }

    #[cfg(unix)]
    #[test]
    fn recorded_child_exit_and_fresh_native_are_required_for_success() {
        let (tmp, paths, store) = setup();
        let fake = tmp.path().join("fake-codex");
        script(
            &fake,
            r#"#!/bin/sh
if [ "$1 $2" = "login --device-auth" ]; then
  rm -f "$CODEX_HOME/auth.json"
  printf 'ABCD-12345\n'
  printf '%s\n' '{"auth_mode":"chatgpt","tokens":{"id_token":"new-id","access_token":"new-access","refresh_token":"new-refresh"},"last_refresh":"2026-07-21T00:00:00Z"}' > "$CODEX_HOME/auth.json"
  exit 0
fi
if [ "$1 $2" = "login status" ]; then
  printf 'Logged in using ChatGPT\n'
  exit 0
fi
exit 2
"#,
        );
        let runtime = test_runtime(&fake);
        let login = start_with(&paths, &runtime).unwrap();
        wait_for_exit(&paths, login.pid);
        let result = finish_with(&paths, &runtime, Some(login.pid));
        assert!(result.flow_succeeded, "finish result: {result:?}");
        assert!(!result.restored);
        assert!(expected_native_link(&paths, &store));
        assert!(std::fs::read_to_string(store.active_path("codex"))
            .unwrap()
            .contains("2026-07-21"));
    }

    #[cfg(unix)]
    #[test]
    fn zero_exit_api_key_flow_without_last_refresh_can_succeed_by_mtime() {
        let (tmp, paths, store) = setup();
        let fake = tmp.path().join("fake-codex-api-key");
        script(
            &fake,
            r#"#!/bin/sh
if [ "$1 $2" = "login --device-auth" ]; then
  rm -f "$CODEX_HOME/auth.json"
  sleep 0.05
  printf '%s\n' '{"auth_mode":"apikey","OPENAI_API_KEY":"test-api-key"}' > "$CODEX_HOME/auth.json"
  printf 'APIK-12345\n'
  exit 0
fi
exit 2
"#,
        );
        let runtime = test_runtime(&fake);
        let login = start_with(&paths, &runtime).unwrap();
        wait_for_exit(&paths, login.pid);
        let result = finish_with(&paths, &runtime, Some(login.pid));

        assert!(result.flow_succeeded, "finish result: {result:?}");
        assert!(!result.restored);
        assert!(expected_native_link(&paths, &store));
        assert!(std::fs::read_to_string(store.active_path("codex"))
            .unwrap()
            .contains("\"auth_mode\":\"apikey\""));
    }

    #[cfg(unix)]
    #[test]
    fn concurrent_start_is_rejected_and_abandon_restores_topology() {
        let (tmp, paths, store) = setup();
        let fake = tmp.path().join("fake-codex");
        script(
            &fake,
            r#"#!/bin/sh
if [ "$1 $2" = "login --device-auth" ]; then
  rm -f "$CODEX_HOME/auth.json"
  printf 'WXYZ-98765\n'
  sleep 0.2
  exit 1
fi
if [ "$1 $2" = "login status" ]; then
  printf 'Not logged in\n'
  exit 1
fi
"#,
        );
        let runtime = test_runtime(&fake);
        let login = start_with(&paths, &runtime).unwrap();
        let second = start_with(&paths, &runtime).unwrap_err().to_string();
        assert!(
            second.contains("already recorded") || second.contains("holds the flow lock"),
            "unexpected second-start error: {second}"
        );
        wait_for_exit(&paths, login.pid);
        let result = finish_with(&paths, &runtime, Some(login.pid));
        assert!(!result.flow_succeeded);
        assert!(result.restored);
        assert!(expected_native_link(&paths, &store));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn abort_requires_owned_pid_and_settles_after_exit_sentinel() {
        let (tmp, paths, store) = setup();
        let fake = tmp.path().join("fake-codex-abort");
        script(
            &fake,
            r#"#!/bin/sh
if [ "$1 $2" = "login --device-auth" ]; then
  rm -f "$CODEX_HOME/auth.json"
  printf 'ABRT-12345\n'
  trap 'exit 143' TERM INT HUP
  while :; do sleep 1; done
fi
exit 2
"#,
        );
        let runtime = test_runtime(&fake);
        let login = start_with(&paths, &runtime).unwrap();
        let wrong = abort_with(&paths, login.pid + 1);
        assert!(matches!(wrong.status, LoginStatus::Unknown { .. }));
        assert!(!wrong.aborted);
        assert!(!wrong.restored);
        assert!(paths.active().exists());

        let result = abort_with(&paths, login.pid);
        assert!(!result.flow_succeeded);
        assert!(result.aborted);
        assert!(result.restored, "abort result: {result:?}");
        assert!(expected_native_link(&paths, &store));
        assert!(!paths.active().exists());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn abort_settles_preexisting_failure_without_claiming_a_signal() {
        let (tmp, paths, store) = setup();
        let fake = tmp.path().join("fake-codex-already-failed");
        script(
            &fake,
            r#"#!/bin/sh
if [ "$1 $2" = "login --device-auth" ]; then
  rm -f "$CODEX_HOME/auth.json"
  printf 'FAIL-12345\n'
  exit 1
fi
exit 2
"#,
        );
        let runtime = test_runtime(&fake);
        let login = start_with(&paths, &runtime).unwrap();
        wait_for_exit(&paths, login.pid);

        let result = abort_with(&paths, login.pid);
        assert!(!result.aborted);
        assert!(!result.flow_succeeded);
        assert!(result.restored, "abort result: {result:?}");
        assert!(expected_native_link(&paths, &store));
        assert!(!paths.active().exists());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn abort_settles_preexisting_success_without_claiming_a_signal() {
        let (tmp, paths, store) = setup();
        let fake = tmp.path().join("fake-codex-already-succeeded");
        script(
            &fake,
            r#"#!/bin/sh
if [ "$1 $2" = "login --device-auth" ]; then
  rm -f "$CODEX_HOME/auth.json"
  printf '%s\n' '{"auth_mode":"chatgpt","tokens":{"id_token":"new-id","access_token":"new-access","refresh_token":"new-refresh"},"last_refresh":"2026-07-21T00:00:00Z"}' > "$CODEX_HOME/auth.json"
  printf 'DONE-12345\n'
  exit 0
fi
exit 2
"#,
        );
        let runtime = test_runtime(&fake);
        let login = start_with(&paths, &runtime).unwrap();
        wait_for_exit(&paths, login.pid);

        let result = abort_with(&paths, login.pid);
        assert!(!result.aborted);
        assert!(result.flow_succeeded, "abort result: {result:?}");
        assert!(expected_native_link(&paths, &store));
        assert!(!paths.active().exists());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn exited_supervisor_without_sentinel_is_unknown_and_never_restores() {
        let (_tmp, paths, store) = setup();
        prepare_state(&paths).unwrap();
        let mut child = Command::new("true").spawn().unwrap();
        let pid = child.id();
        child.wait().unwrap();

        let id = unique_flow_id();
        set_owner_only_dir(&paths.flow_dir(&id)).unwrap();
        let mut record = FlowRecord {
            id,
            supervisor_pid: pid,
            process_identity: Some("exited-test-process".to_string()),
            process_argv: Some(vec!["true".to_string()]),
            started_unix_ms: 0,
            had_valid_backup: false,
        };
        record.had_valid_backup =
            copy_valid_backup(&store.active_path("codex"), &paths.backup(&record.id)).unwrap();
        write_record(&paths, &record).unwrap();
        std::fs::remove_file(paths.native_auth()).unwrap();

        let result = abort_with(&paths, pid);
        assert!(matches!(result.status, LoginStatus::Unknown { .. }));
        assert!(!result.aborted);
        assert!(!result.restored);
        assert!(paths.active().exists());
        assert!(
            !paths.native_auth().exists(),
            "Unknown must not recreate credential topology"
        );

        let finished = finish_with(&paths, &Runtime::default(), Some(pid));
        assert!(matches!(finished.status, LoginStatus::Unknown { .. }));
        assert!(!finished.restored);
        assert!(paths.active().exists());
        assert!(!paths.native_auth().exists());

        assert_eq!(
            reconcile_on_startup_with(&paths).unwrap(),
            StartupReconcile::DeferredActiveFlow
        );
        assert!(paths.active().exists());
        assert!(
            !paths.native_auth().exists(),
            "startup must preserve no-sentinel credential state"
        );
    }

    #[cfg(all(
        target_os = "linux",
        any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv64",
            target_arch = "s390x",
            target_arch = "powerpc64"
        )
    ))]
    #[test]
    fn pidfd_signal_targets_the_opened_process_instance() {
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        let handle = match open_stable_process(child.id()) {
            StableProcessOpen::Open(handle) => handle,
            other => panic!("pidfd_open did not bind the child: {other:?}"),
        };
        assert_eq!(handle.send_term().unwrap(), StableSignal::Sent);
        let status = child.wait().unwrap();
        assert!(!status.success());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn abort_preserves_a_live_process_when_exact_argv_does_not_match() {
        let (_tmp, paths, store) = setup();
        prepare_state(&paths).unwrap();
        let canonical_before = std::fs::read(store.active_path("codex")).unwrap();
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        let id = unique_flow_id();
        set_owner_only_dir(&paths.flow_dir(&id)).unwrap();
        let identity = match process_identity(child.id()) {
            ProcessProbe::Found(identity) => identity,
            other => panic!("could not identify test child: {other:?}"),
        };
        let record = FlowRecord {
            id,
            supervisor_pid: child.id(),
            process_identity: Some(identity),
            process_argv: Some(vec!["forged-supervisor-argv".to_string()]),
            started_unix_ms: 0,
            had_valid_backup: false,
        };
        write_record(&paths, &record).unwrap();

        let result = abort_with(&paths, child.id());
        assert!(matches!(result.status, LoginStatus::Unknown { .. }));
        assert!(!result.aborted);
        assert!(!result.restored);
        assert!(paths.active().exists());
        assert!(matches!(child.try_wait(), Ok(None)));
        assert_eq!(
            std::fs::read(store.active_path("codex")).unwrap(),
            canonical_before
        );
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[cfg(all(unix, not(target_os = "linux")))]
    #[test]
    fn abort_fails_closed_without_stable_process_signalling() {
        let (_tmp, paths, _store) = setup();
        prepare_state(&paths).unwrap();
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        let id = unique_flow_id();
        set_owner_only_dir(&paths.flow_dir(&id)).unwrap();
        let record = FlowRecord {
            id,
            supervisor_pid: child.id(),
            process_identity: match process_identity(child.id()) {
                ProcessProbe::Found(identity) => Some(identity),
                other => panic!("could not identify test child: {other:?}"),
            },
            process_argv: Some(vec!["sleep".to_string(), "30".to_string()]),
            started_unix_ms: 0,
            had_valid_backup: false,
        };
        write_record(&paths, &record).unwrap();

        let result = abort_with(&paths, child.id());
        assert!(matches!(result.status, LoginStatus::Unknown { .. }));
        assert!(!result.aborted);
        assert!(!result.restored);
        assert!(matches!(child.try_wait(), Ok(None)));
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn startup_reconciliation_defers_a_live_recorded_flow_without_mutation() {
        let (tmp, paths, store) = setup();
        let fake = tmp.path().join("fake-codex-live");
        script(
            &fake,
            r#"#!/bin/sh
if [ "$1 $2" = "login --device-auth" ]; then
  printf 'LIVE-12345\n'
  sleep 0.5
  exit 1
fi
exit 2
"#,
        );
        let runtime = test_runtime(&fake);
        let canonical = store.active_path("codex");
        let native = paths.native_auth();
        let login = start_with(&paths, &runtime).unwrap();
        let canonical_before = std::fs::read(&canonical).unwrap();
        let native_before = std::fs::read(&native).unwrap();

        assert_eq!(
            reconcile_on_startup_with(&paths).unwrap(),
            StartupReconcile::DeferredActiveFlow
        );
        assert!(paths.active().exists());
        assert_eq!(std::fs::read(&canonical).unwrap(), canonical_before);
        assert_eq!(std::fs::read(&native).unwrap(), native_before);

        wait_for_exit(&paths, login.pid);
        let cleanup = finish_with(&paths, &runtime, Some(login.pid));
        assert!(!cleanup.restored, "no topology was disrupted: {cleanup:?}");
    }

    #[cfg(unix)]
    #[test]
    fn startup_reconciliation_settles_an_exited_flow() {
        let (tmp, paths, store) = setup();
        let fake = tmp.path().join("fake-codex-exited");
        script(
            &fake,
            r#"#!/bin/sh
if [ "$1 $2" = "login --device-auth" ]; then
  rm -f "$CODEX_HOME/auth.json"
  printf 'EXIT-12345\n'
  exit 1
fi
exit 2
"#,
        );
        let runtime = test_runtime(&fake);
        let canonical = store.active_path("codex");
        let canonical_before = std::fs::read(&canonical).unwrap();
        let login = start_with(&paths, &runtime).unwrap();
        wait_for_exit(&paths, login.pid);
        let record = read_record(&paths).unwrap().unwrap();
        let flow_dir = paths.flow_dir(&record.id);

        assert_eq!(
            reconcile_on_startup_with(&paths).unwrap(),
            StartupReconcile::Reconciled
        );
        assert!(!paths.active().exists());
        assert!(!flow_dir.exists());
        assert_eq!(std::fs::read(&canonical).unwrap(), canonical_before);
        assert!(expected_native_link(&paths, &store));
        assert_eq!(
            std::fs::read(paths.native_auth()).unwrap(),
            canonical_before
        );
    }

    #[cfg(unix)]
    #[test]
    fn missing_binary_restores_the_single_topology() {
        let (_tmp, paths, store) = setup();
        let runtime = test_runtime(Path::new("/definitely/missing/codex"));
        assert!(start_with(&paths, &runtime).is_err());
        assert!(expected_native_link(&paths, &store));
        assert!(!paths.active().exists());
    }

    #[cfg(unix)]
    #[test]
    fn unknown_status_does_not_signal_or_restore() {
        let (tmp, paths, _store) = setup();
        let fake = tmp.path().join("fake-codex");
        script(
            &fake,
            r#"#!/bin/sh
if [ "$1 $2" = "login status" ]; then
  printf 'Not logged in\n'
  exit 1
fi
rm -f "$CODEX_HOME/auth.json"
printf 'LMNO-24680\n'
sleep 0.2
exit 1
"#,
        );
        let running = test_runtime(&fake);
        let login = start_with(&paths, &running).unwrap();
        let unknown = test_runtime(Path::new("/definitely/missing/codex"));
        let result = finish_with(&paths, &unknown, Some(login.pid));
        assert!(matches!(result.status, LoginStatus::Unknown { .. }));
        assert!(!result.restored);
        assert_eq!(
            recorded_process(&paths, &read_record(&paths).unwrap().unwrap()),
            RecordedProcess::Running
        );
        wait_for_exit(&paths, login.pid);
        let cleanup = finish_with(&paths, &running, Some(login.pid));
        assert!(cleanup.restored, "cleanup result: {cleanup:?}");
    }

    #[test]
    fn stale_pid_is_diagnosed_without_signalling_the_reused_process() {
        let (_tmp, paths, store) = setup();
        prepare_state(&paths).unwrap();
        let id = unique_flow_id();
        set_owner_only_dir(&paths.flow_dir(&id)).unwrap();
        let record = FlowRecord {
            id,
            supervisor_pid: std::process::id(),
            process_identity: Some("deliberately-not-this-process".to_string()),
            process_argv: Some(vec!["deliberately-not-this-process".to_string()]),
            started_unix_ms: 0,
            had_valid_backup: false,
        };
        // Use a real per-flow backup and leave the current test process alive.
        let mut record = record;
        record.had_valid_backup =
            copy_valid_backup(&store.active_path("codex"), &paths.backup(&record.id)).unwrap();
        write_record(&paths, &record).unwrap();
        let result = finish_with(&paths, &Runtime::default(), Some(std::process::id()));
        assert!(!result.flow_succeeded);
        assert!(matches!(result.status, LoginStatus::Unknown { .. }));
        assert!(paths.active().exists());
        let aborted = abort_with(&paths, std::process::id());
        assert!(matches!(aborted.status, LoginStatus::Unknown { .. }));
        assert!(!aborted.aborted);
        assert!(!aborted.restored);
        assert!(paths.active().exists());
        assert!(matches!(
            structural_status(&store),
            LoginStatus::LoggedIn { .. }
        ));
    }

    #[test]
    fn auth_diagnostic_prefers_unauthenticated_over_quota_words() {
        assert_eq!(
            auth_diagnostic("401 Unauthorized; task text mentioned quota"),
            Some(AuthProbe::Unauthenticated)
        );
        assert_eq!(
            auth_diagnostic("HTTP 429 usage limit"),
            Some(AuthProbe::QuotaLimited)
        );
    }

    #[test]
    fn auth_probe_marker_must_be_the_entire_response() {
        assert!(exact_auth_marker("AUTH_OK\n"));
        assert!(exact_auth_marker("\x1b[32mAUTH_OK\x1b[0m\n"));
        assert!(!exact_auth_marker("I should reply AUTH_OK"));
        assert!(!exact_auth_marker("AUTH_OK extra"));
    }

    #[test]
    fn diagnostics_count_orphaned_legacy_and_flow_backups() {
        let (_tmp, paths, _store) = setup();
        let legacy = paths.omega_dir.join("state").join("codex-auth.backup.json");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(&legacy, "{}").unwrap();
        let orphan = paths.flow_dir("orphan");
        std::fs::create_dir_all(&orphan).unwrap();
        std::fs::write(orphan.join("backup.json"), "{}").unwrap();
        assert_eq!(diagnostics_with(&paths).stale_backups, 2);
    }
}
