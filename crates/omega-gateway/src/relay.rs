//! Persistent outbound connector from a box gateway to the omega-app relay.
//!
//! Each relay channel is bridged to a fresh loopback TCP connection. Keeping
//! the bridge byte-opaque means normal HTTP responses, multipart uploads,
//! streamed downloads and HTTP WebSocket upgrades all use the gateway's real
//! server implementation without a second protocol stack here.

use crate::clerk::{ClerkClaims, ClerkVerifier};
use crate::fsperm::{harden_dir, harden_file};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::{
    frame::coding::CloseCode, CloseFrame, Role, WebSocketConfig,
};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

const REGISTRATION_FILE: &str = "relay_registration.json";
const REGISTRATION_VERSION: u8 = 1;
const MAX_OWNER_BYTES: usize = 256;
const MAX_CHANNEL_ID_BYTES: usize = 128;
const MAX_REGISTRATION_BYTES: u64 = 4096;
const CONTROL_MESSAGE_BYTES: usize = 64 * 1024;
const DATA_MESSAGE_BYTES: usize = 64 * 1024 * 1024;
const MAX_CONCURRENT_RELAY_CHANNELS: usize = 32;
const LOCAL_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const CHANNEL_CONNECT_TIMEOUT: Duration = Duration::from_secs(8);
const INITIAL_BACKOFF: Duration = Duration::from_millis(250);
const MAX_BACKOFF: Duration = Duration::from_secs(30);
const HEALTHY_CONNECTION: Duration = Duration::from_secs(60);
const DEFAULT_HEARTBEAT: Duration = Duration::from_secs(20);
const DEFAULT_LIVENESS_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayError {
    InvalidConfiguration,
    StoreUnavailable,
    CorruptRegistration,
    OwnershipConflict,
    ClerkRejected,
    ConnectionFailed,
    ProtocolViolation,
    LocalGatewayUnavailable,
}

impl std::fmt::Display for RelayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidConfiguration => "invalid relay configuration",
            Self::StoreUnavailable => "relay registration store unavailable",
            Self::CorruptRegistration => "relay registration store is corrupt",
            Self::OwnershipConflict => "box is already registered to another Clerk user",
            Self::ClerkRejected => "Clerk token rejected",
            Self::ConnectionFailed => "relay connection failed",
            Self::ProtocolViolation => "relay protocol violation",
            Self::LocalGatewayUnavailable => "local gateway unavailable",
        })
    }
}

impl std::error::Error for RelayError {}

struct RelayConfig {
    relay_url: reqwest::Url,
    http: reqwest::Client,
    heartbeat: Duration,
    liveness_timeout: Duration,
}

impl RelayConfig {
    fn production() -> Result<Self, RelayError> {
        let relay_url =
            std::env::var("OMEGA_RELAY_URL").map_err(|_| RelayError::InvalidConfiguration)?;
        Self::new(
            &relay_url,
            false,
            DEFAULT_HEARTBEAT,
            DEFAULT_LIVENESS_TIMEOUT,
        )
    }

    fn new(
        relay_url: &str,
        allow_insecure_ws: bool,
        heartbeat: Duration,
        liveness_timeout: Duration,
    ) -> Result<Self, RelayError> {
        let relay_url =
            reqwest::Url::parse(relay_url).map_err(|_| RelayError::InvalidConfiguration)?;
        let scheme_ok =
            relay_url.scheme() == "wss" || (allow_insecure_ws && relay_url.scheme() == "ws");
        if !scheme_ok
            || relay_url.host_str().is_none()
            || !relay_url.username().is_empty()
            || relay_url.password().is_some()
            || relay_url.query().is_some()
            || relay_url.fragment().is_some()
            || relay_url.path() != "/v1/register"
            || heartbeat.is_zero()
            || liveness_timeout <= heartbeat
        {
            return Err(RelayError::InvalidConfiguration);
        }
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .http1_only()
            .build()
            .map_err(|_| RelayError::InvalidConfiguration)?;
        Ok(Self {
            relay_url,
            http,
            heartbeat,
            liveness_timeout,
        })
    }
}

/// Stored only in the gateway's owner-only data directory. Deliberately does
/// not implement `Debug`: accidental structured logging must not reveal the
/// reusable relay credential.
#[derive(Clone)]
struct RelayRegistration {
    clerk_user_id: String,
    box_id: String,
    relay_credential: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredRegistration {
    version: u8,
    clerk_user_id: String,
    box_id: String,
    relay_credential: String,
}

#[derive(Clone)]
struct RelayRegistrationStore {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl RelayRegistrationStore {
    fn new(dir: &Path) -> Self {
        Self {
            path: dir.join(REGISTRATION_FILE),
            lock: Arc::new(Mutex::new(())),
        }
    }

    fn read(&self) -> Result<Option<RelayRegistration>, RelayError> {
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.read_unlocked()
    }

    fn read_unlocked(&self) -> Result<Option<RelayRegistration>, RelayError> {
        let metadata = match std::fs::symlink_metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(RelayError::StoreUnavailable),
        };
        if !metadata.file_type().is_file() || metadata.len() > MAX_REGISTRATION_BYTES {
            return Err(RelayError::CorruptRegistration);
        }
        harden_file(&self.path);
        let file = std::fs::File::open(&self.path).map_err(|_| RelayError::StoreUnavailable)?;
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.take(MAX_REGISTRATION_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| RelayError::StoreUnavailable)?;
        if bytes.len() as u64 > MAX_REGISTRATION_BYTES {
            return Err(RelayError::CorruptRegistration);
        }
        let stored: StoredRegistration =
            serde_json::from_slice(&bytes).map_err(|_| RelayError::CorruptRegistration)?;
        if stored.version != REGISTRATION_VERSION
            || !valid_owner(&stored.clerk_user_id)
            || !valid_box_id(&stored.box_id)
            || !valid_credential(&stored.relay_credential)
        {
            return Err(RelayError::CorruptRegistration);
        }
        Ok(Some(RelayRegistration {
            clerk_user_id: stored.clerk_user_id,
            box_id: stored.box_id,
            relay_credential: stored.relay_credential,
        }))
    }

    fn register(&self, clerk_user_id: &str, box_id: &str) -> Result<RelayRegistration, RelayError> {
        if !valid_owner(clerk_user_id) || !valid_box_id(box_id) {
            return Err(RelayError::CorruptRegistration);
        }
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(existing) = self.read_unlocked()? {
            if existing.clerk_user_id != clerk_user_id || existing.box_id != box_id {
                return Err(RelayError::OwnershipConflict);
            }
            return Ok(existing);
        }

        let registration = RelayRegistration {
            clerk_user_id: clerk_user_id.to_string(),
            box_id: box_id.to_string(),
            relay_credential: crate::util::random_hex(32),
        };
        self.write_unlocked(&registration)?;
        Ok(registration)
    }

    fn write_unlocked(&self, registration: &RelayRegistration) -> Result<(), RelayError> {
        let parent = self.path.parent().ok_or(RelayError::StoreUnavailable)?;
        std::fs::create_dir_all(parent).map_err(|_| RelayError::StoreUnavailable)?;
        harden_dir(parent);
        let stored = StoredRegistration {
            version: REGISTRATION_VERSION,
            clerk_user_id: registration.clerk_user_id.clone(),
            box_id: registration.box_id.clone(),
            relay_credential: registration.relay_credential.clone(),
        };
        let bytes = serde_json::to_vec(&stored).map_err(|_| RelayError::StoreUnavailable)?;
        let temp_path = parent.join(format!(
            ".{REGISTRATION_FILE}.tmp-{}",
            crate::util::random_hex(8)
        ));
        let result = (|| {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options
                .open(&temp_path)
                .map_err(|_| RelayError::StoreUnavailable)?;
            file.write_all(&bytes)
                .map_err(|_| RelayError::StoreUnavailable)?;
            file.sync_all().map_err(|_| RelayError::StoreUnavailable)?;
            harden_file(&temp_path);
            std::fs::rename(&temp_path, &self.path).map_err(|_| RelayError::StoreUnavailable)?;
            harden_file(&self.path);
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temp_path);
        }
        result
    }
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_OWNER_BYTES
        && value.bytes().all(|byte| byte.is_ascii_graphic())
}

fn valid_box_id(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_credential(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Clone)]
pub struct RelayManager {
    config: Arc<Result<RelayConfig, RelayError>>,
    clerk: Arc<ClerkVerifier>,
    store: RelayRegistrationStore,
    local_addr: Arc<RwLock<Option<SocketAddr>>>,
    supervisor: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
}

impl RelayManager {
    pub fn production(dir: PathBuf) -> Self {
        Self {
            config: Arc::new(RelayConfig::production()),
            clerk: Arc::new(ClerkVerifier::production()),
            store: RelayRegistrationStore::new(&dir),
            local_addr: Arc::new(RwLock::new(None)),
            supervisor: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// Local-only constructor used by in-crate tests. Production always
    /// requires `wss://`; this insecure transport escape hatch is not compiled
    /// into release builds.
    #[cfg(test)]
    pub(crate) fn for_test(
        dir: PathBuf,
        relay_url: &str,
        jwks_url: &str,
        issuer: &str,
        heartbeat: Duration,
        liveness_timeout: Duration,
    ) -> Result<Self, RelayError> {
        Ok(Self {
            config: Arc::new(Ok(RelayConfig::new(
                relay_url,
                true,
                heartbeat,
                liveness_timeout,
            )?)),
            clerk: Arc::new(ClerkVerifier::new(jwks_url.to_string(), issuer.to_string())),
            store: RelayRegistrationStore::new(&dir),
            local_addr: Arc::new(RwLock::new(None)),
            supervisor: Arc::new(tokio::sync::Mutex::new(None)),
        })
    }

    pub async fn verify_clerk(&self, token: &str) -> Result<ClerkClaims, RelayError> {
        self.clerk
            .verify(token)
            .await
            .map_err(|_| RelayError::ClerkRejected)
    }

    pub async fn register(&self, clerk_user_id: String, box_id: String) -> Result<(), RelayError> {
        self.config.as_ref().as_ref().map_err(|error| *error)?;
        let store = self.store.clone();
        let registration =
            tokio::task::spawn_blocking(move || store.register(&clerk_user_id, &box_id))
                .await
                .map_err(|_| RelayError::StoreUnavailable)??;
        if let Some(local_addr) = *self.local_addr.read().await {
            self.spawn_supervisor(registration, local_addr).await?;
        }
        Ok(())
    }

    /// Attach the connector to the actual listener address and resume any
    /// previously persisted registration. An unspecified bind address is
    /// normalized to loopback: the connector must never hairpin through a LAN
    /// interface or bypass the local server process.
    pub async fn start(&self, listener_addr: SocketAddr) -> Result<(), RelayError> {
        let local_addr = loopback_addr(listener_addr);
        *self.local_addr.write().await = Some(local_addr);
        let store = self.store.clone();
        let registration = tokio::task::spawn_blocking(move || store.read())
            .await
            .map_err(|_| RelayError::StoreUnavailable)??;
        if let Some(registration) = registration {
            self.spawn_supervisor(registration, local_addr).await?;
        }
        Ok(())
    }

    async fn spawn_supervisor(
        &self,
        registration: RelayRegistration,
        local_addr: SocketAddr,
    ) -> Result<(), RelayError> {
        let config = self.config.as_ref().as_ref().map_err(|error| *error)?;
        let config = Arc::new(RelayConfig {
            relay_url: config.relay_url.clone(),
            http: config.http.clone(),
            heartbeat: config.heartbeat,
            liveness_timeout: config.liveness_timeout,
        });
        let mut supervisor = self.supervisor.lock().await;
        if let Some(previous) = supervisor.take() {
            previous.abort();
        }
        *supervisor = Some(tokio::spawn(async move {
            supervise(config, registration, local_addr).await;
        }));
        Ok(())
    }
}

fn loopback_addr(listener_addr: SocketAddr) -> SocketAddr {
    match listener_addr.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), listener_addr.port())
        }
        IpAddr::V6(ip) if ip.is_unspecified() => {
            SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), listener_addr.port())
        }
        _ => listener_addr,
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum OutboundControl<'a> {
    Register {
        box_id: &'a str,
        clerk_user_id: &'a str,
        relay_credential: &'a str,
    },
    ChannelConnect {
        channel_id: &'a str,
        box_id: &'a str,
        relay_credential: &'a str,
    },
    Ping,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum InboundControl {
    Ok,
    OpenChannel { channel_id: String },
    Pong,
    Error { error: String },
}

async fn supervise(
    config: Arc<RelayConfig>,
    registration: RelayRegistration,
    local_addr: SocketAddr,
) {
    let channels = Arc::new(Semaphore::new(MAX_CONCURRENT_RELAY_CHANNELS));
    let mut backoff = INITIAL_BACKOFF;
    loop {
        let started = Instant::now();
        let result = run_control(
            config.clone(),
            registration.clone(),
            local_addr,
            channels.clone(),
        )
        .await;
        if started.elapsed() >= HEALTHY_CONNECTION {
            backoff = INITIAL_BACKOFF;
        }
        let error_kind = match result {
            Ok(()) => "closed",
            Err(RelayError::ProtocolViolation) => "protocol",
            Err(RelayError::ConnectionFailed) => "network",
            Err(_) => "internal",
        };
        tracing::warn!(error_kind, "outbound relay disconnected; retrying");
        let jitter_percent = rand::random::<u8>() % 41;
        let factor = 80_u32 + u32::from(jitter_percent);
        let jittered = backoff.mul_f64(f64::from(factor) / 100.0);
        tokio::time::sleep(jittered).await;
        backoff = backoff.saturating_mul(2).min(MAX_BACKOFF);
    }
}

async fn run_control(
    config: Arc<RelayConfig>,
    registration: RelayRegistration,
    local_addr: SocketAddr,
    channels: Arc<Semaphore>,
) -> Result<(), RelayError> {
    let mut socket = websocket_connect(&config, CONTROL_MESSAGE_BYTES).await?;
    let register = serde_json::to_string(&OutboundControl::Register {
        box_id: &registration.box_id,
        clerk_user_id: &registration.clerk_user_id,
        relay_credential: &registration.relay_credential,
    })
    .map_err(|_| RelayError::ProtocolViolation)?;
    socket
        .send(Message::Text(register))
        .await
        .map_err(|_| RelayError::ConnectionFailed)?;
    let first = tokio::time::timeout(Duration::from_secs(10), socket.next())
        .await
        .map_err(|_| RelayError::ConnectionFailed)?
        .ok_or(RelayError::ConnectionFailed)?
        .map_err(|_| RelayError::ConnectionFailed)?;
    match parse_control(first)? {
        InboundControl::Ok => {}
        _ => return Err(RelayError::ProtocolViolation),
    }

    let mut last_seen = Instant::now();
    let mut heartbeat = tokio::time::interval(config.heartbeat);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    heartbeat.tick().await;

    loop {
        tokio::select! {
            incoming = socket.next() => {
                let message = incoming
                    .ok_or(RelayError::ConnectionFailed)?
                    .map_err(|_| RelayError::ConnectionFailed)?;
                last_seen = Instant::now();
                match message {
                    Message::Text(_) => match parse_control(message)? {
                        InboundControl::OpenChannel { channel_id } => {
                            if !valid_channel_id(&channel_id) {
                                return Err(RelayError::ProtocolViolation);
                            }
                            let permit = match channels.clone().try_acquire_owned() {
                                Ok(permit) => permit,
                                Err(_) => continue,
                            };
                            let config = config.clone();
                            let registration = registration.clone();
                            tokio::spawn(async move {
                                let _permit = permit;
                                if serve_channel(config, registration, local_addr, channel_id)
                                    .await
                                    .is_err()
                                {
                                    tracing::warn!("relay channel closed with an error");
                                }
                            });
                        }
                        InboundControl::Pong => {}
                        _ => return Err(RelayError::ProtocolViolation),
                    },
                    Message::Ping(payload) => {
                        socket.send(Message::Pong(payload)).await
                            .map_err(|_| RelayError::ConnectionFailed)?;
                    }
                    Message::Pong(_) => {}
                    Message::Close(_) => return Ok(()),
                    _ => return Err(RelayError::ProtocolViolation),
                }
            }
            _ = heartbeat.tick() => {
                if last_seen.elapsed() > config.liveness_timeout {
                    return Err(RelayError::ConnectionFailed);
                }
                let ping = serde_json::to_string(&OutboundControl::Ping)
                    .map_err(|_| RelayError::ProtocolViolation)?;
                socket.send(Message::Text(ping)).await
                    .map_err(|_| RelayError::ConnectionFailed)?;
            }
        }
    }
}

fn parse_control(message: Message) -> Result<InboundControl, RelayError> {
    let Message::Text(text) = message else {
        return Err(RelayError::ProtocolViolation);
    };
    let parsed: InboundControl =
        serde_json::from_str(&text).map_err(|_| RelayError::ProtocolViolation)?;
    if let InboundControl::Error { error } = &parsed {
        if error.is_empty() || error.len() > 256 {
            return Err(RelayError::ProtocolViolation);
        }
    }
    Ok(parsed)
}

fn valid_channel_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CHANNEL_ID_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

async fn serve_channel(
    config: Arc<RelayConfig>,
    registration: RelayRegistration,
    local_addr: SocketAddr,
    channel_id: String,
) -> Result<(), RelayError> {
    let (socket, local) = tokio::time::timeout(
        CHANNEL_CONNECT_TIMEOUT,
        connect_channel(config, registration, local_addr, channel_id),
    )
    .await
    .map_err(|_| RelayError::ConnectionFailed)??;
    pump_channel(socket, local).await
}

async fn connect_channel(
    config: Arc<RelayConfig>,
    registration: RelayRegistration,
    local_addr: SocketAddr,
    channel_id: String,
) -> Result<(RelaySocket, TcpStream), RelayError> {
    let local = tokio::time::timeout(LOCAL_CONNECT_TIMEOUT, TcpStream::connect(local_addr))
        .await
        .map_err(|_| RelayError::LocalGatewayUnavailable)?
        .map_err(|_| RelayError::LocalGatewayUnavailable)?;
    let mut socket = websocket_connect(&config, DATA_MESSAGE_BYTES).await?;
    let connect = serde_json::to_string(&OutboundControl::ChannelConnect {
        channel_id: &channel_id,
        box_id: &registration.box_id,
        relay_credential: &registration.relay_credential,
    })
    .map_err(|_| RelayError::ProtocolViolation)?;
    socket
        .send(Message::Text(connect))
        .await
        .map_err(|_| RelayError::ConnectionFailed)?;
    Ok((socket, local))
}

type RelaySocket = WebSocketStream<reqwest::Upgraded>;

async fn websocket_connect(
    config: &RelayConfig,
    max_message_size: usize,
) -> Result<RelaySocket, RelayError> {
    let request = tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(
        config.relay_url.as_str(),
    )
    .map_err(|_| RelayError::InvalidConfiguration)?;
    let websocket_key = request
        .headers()
        .get("sec-websocket-key")
        .and_then(|value| value.to_str().ok())
        .ok_or(RelayError::InvalidConfiguration)?
        .to_string();
    let mut http_url = config.relay_url.clone();
    let http_scheme = if http_url.scheme() == "wss" {
        "https"
    } else {
        "http"
    };
    http_url
        .set_scheme(http_scheme)
        .map_err(|_| RelayError::InvalidConfiguration)?;

    let response = tokio::time::timeout(
        Duration::from_secs(10),
        config
            .http
            .get(http_url)
            .version(reqwest::Version::HTTP_11)
            .header("connection", "Upgrade")
            .header("upgrade", "websocket")
            .header("sec-websocket-version", "13")
            .header("sec-websocket-key", &websocket_key)
            .send(),
    )
    .await
    .map_err(|_| RelayError::ConnectionFailed)?
    .map_err(|_| RelayError::ConnectionFailed)?;
    if response.status() != reqwest::StatusCode::SWITCHING_PROTOCOLS
        || !header_has_token(response.headers(), "connection", "upgrade")
        || !header_has_token(response.headers(), "upgrade", "websocket")
        || response
            .headers()
            .get("sec-websocket-accept")
            .and_then(|value| value.to_str().ok())
            != Some(derive_accept_key(websocket_key.as_bytes()).as_str())
    {
        return Err(RelayError::ProtocolViolation);
    }
    let upgraded = tokio::time::timeout(Duration::from_secs(5), response.upgrade())
        .await
        .map_err(|_| RelayError::ConnectionFailed)?
        .map_err(|_| RelayError::ConnectionFailed)?;
    let websocket_config = WebSocketConfig {
        max_message_size: Some(max_message_size),
        max_frame_size: Some(max_message_size),
        ..WebSocketConfig::default()
    };
    Ok(WebSocketStream::from_raw_socket(upgraded, Role::Client, Some(websocket_config)).await)
}

fn header_has_token(headers: &reqwest::header::HeaderMap, name: &str, token: &str) -> bool {
    headers
        .get_all(name)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .any(|value| value.trim().eq_ignore_ascii_case(token))
}

async fn pump_channel(mut socket: RelaySocket, mut local: TcpStream) -> Result<(), RelayError> {
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        tokio::select! {
            incoming = socket.next() => {
                match incoming {
                    Some(Ok(Message::Binary(bytes))) => {
                        local.write_all(&bytes).await
                            .map_err(|_| RelayError::LocalGatewayUnavailable)?;
                    }
                    Some(Ok(Message::Text(text))) => {
                        local.write_all(text.as_bytes()).await
                            .map_err(|_| RelayError::LocalGatewayUnavailable)?;
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        socket.send(Message::Pong(payload)).await
                            .map_err(|_| RelayError::ConnectionFailed)?;
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | None => {
                        let _ = local.shutdown().await;
                        return Ok(());
                    }
                    Some(Ok(_)) => return Err(RelayError::ProtocolViolation),
                    Some(Err(_)) => return Err(RelayError::ConnectionFailed),
                }
            }
            read = local.read(&mut buffer) => {
                let count = read.map_err(|_| RelayError::LocalGatewayUnavailable)?;
                if count == 0 {
                    let _ = socket.send(Message::Close(Some(CloseFrame {
                        code: CloseCode::Normal,
                        reason: "local response complete".into(),
                    }))).await;
                    return Ok(());
                }
                socket.send(Message::Binary(buffer[..count].to_vec())).await
                    .map_err(|_| RelayError::ConnectionFailed)?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::DeviceStore;
    use crate::clerk::testkit::{sign_token, start_jwks_stub, TEST_ISSUER};
    use crate::config::GatewayConfig;
    use crate::server::{build_router, AppState};
    use axum::extract::ws::{Message as AxumMessage, WebSocket, WebSocketUpgrade};
    use axum::extract::State;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum::Router;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::{oneshot, Notify};

    #[derive(Clone)]
    struct FakeRelayState {
        registration_count: Arc<AtomicUsize>,
        heartbeat_count: Arc<AtomicUsize>,
        registered: Arc<Notify>,
        heartbeat: Arc<Notify>,
        drop_first_registration: bool,
        respond_to_heartbeat: bool,
        channel_frames: Arc<Vec<Vec<u8>>>,
        channel_response: Arc<Mutex<Option<oneshot::Sender<Vec<u8>>>>>,
        credential: Arc<Mutex<Option<String>>>,
    }

    impl FakeRelayState {
        fn control_only(drop_first_registration: bool, respond_to_heartbeat: bool) -> Self {
            Self {
                registration_count: Arc::new(AtomicUsize::new(0)),
                heartbeat_count: Arc::new(AtomicUsize::new(0)),
                registered: Arc::new(Notify::new()),
                heartbeat: Arc::new(Notify::new()),
                drop_first_registration,
                respond_to_heartbeat,
                channel_frames: Arc::new(Vec::new()),
                channel_response: Arc::new(Mutex::new(None)),
                credential: Arc::new(Mutex::new(None)),
            }
        }
    }

    async fn fake_relay_handler(
        ws: WebSocketUpgrade,
        State(state): State<FakeRelayState>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| fake_relay_socket(socket, state))
    }

    async fn fake_relay_socket(mut socket: WebSocket, state: FakeRelayState) {
        let Some(Ok(AxumMessage::Text(first))) = socket.recv().await else {
            return;
        };
        let Ok(message) = serde_json::from_str::<serde_json::Value>(&first) else {
            return;
        };
        match message.get("type").and_then(|value| value.as_str()) {
            Some("register") => {
                let count = state.registration_count.fetch_add(1, Ordering::SeqCst) + 1;
                state.registered.notify_one();
                if state.drop_first_registration && count == 1 {
                    return;
                }
                let Some(credential) = message
                    .get("relay_credential")
                    .and_then(|value| value.as_str())
                else {
                    return;
                };
                *state
                    .credential
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                    Some(credential.to_string());
                if socket
                    .send(AxumMessage::Text(r#"{"type":"ok"}"#.into()))
                    .await
                    .is_err()
                {
                    return;
                }
                if !state.channel_frames.is_empty()
                    && socket
                        .send(AxumMessage::Text(
                            r#"{"type":"open_channel","channel_id":"channel_1"}"#.into(),
                        ))
                        .await
                        .is_err()
                {
                    return;
                }
                while let Some(Ok(message)) = socket.recv().await {
                    match message {
                        AxumMessage::Text(text)
                            if serde_json::from_str::<serde_json::Value>(&text)
                                .ok()
                                .and_then(|value| {
                                    value
                                        .get("type")
                                        .and_then(|kind| kind.as_str())
                                        .map(|kind| kind == "ping")
                                })
                                .unwrap_or(false) =>
                        {
                            state.heartbeat_count.fetch_add(1, Ordering::SeqCst);
                            state.heartbeat.notify_one();
                            if state.respond_to_heartbeat
                                && socket
                                    .send(AxumMessage::Text(r#"{"type":"pong"}"#.into()))
                                    .await
                                    .is_err()
                            {
                                return;
                            }
                        }
                        AxumMessage::Close(_) => return,
                        _ => {}
                    }
                }
            }
            Some("channel_connect") => {
                let presented = message
                    .get("relay_credential")
                    .and_then(|value| value.as_str());
                let expected = state
                    .credential
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .clone();
                if presented != expected.as_deref()
                    || message.get("channel_id").and_then(|value| value.as_str())
                        != Some("channel_1")
                {
                    return;
                }
                for frame in state.channel_frames.iter() {
                    if socket
                        .send(AxumMessage::Binary(frame.clone().into()))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                let mut response = Vec::new();
                while let Some(Ok(message)) = socket.recv().await {
                    match message {
                        AxumMessage::Binary(bytes) => response.extend_from_slice(&bytes),
                        AxumMessage::Text(text) => response.extend_from_slice(text.as_bytes()),
                        AxumMessage::Close(_) => break,
                        _ => {}
                    }
                }
                if let Some(sender) = state
                    .channel_response
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .take()
                {
                    let _ = sender.send(response);
                }
            }
            _ => {}
        }
    }

    async fn start_fake_relay(state: FakeRelayState) -> String {
        let app = Router::new()
            .route("/v1/register", get(fake_relay_handler))
            .with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("ws://{address}/v1/register")
    }

    fn persist_test_registration(dir: &Path) {
        RelayRegistrationStore::new(dir)
            .register("user_owner", "0123456789abcdef0123456789abcdef")
            .unwrap();
    }

    #[test]
    fn production_config_rejects_insecure_and_secret_bearing_urls() {
        assert!(RelayConfig::new(
            "ws://relay.example/v1/register",
            false,
            Duration::from_secs(1),
            Duration::from_secs(2)
        )
        .is_err());
        assert!(RelayConfig::new(
            "wss://user:secret@relay.example/v1/register",
            false,
            Duration::from_secs(1),
            Duration::from_secs(2)
        )
        .is_err());
        assert!(RelayConfig::new(
            "wss://relay.example/v1/register?token=secret",
            false,
            Duration::from_secs(1),
            Duration::from_secs(2)
        )
        .is_err());
        assert!(RelayConfig::new(
            "wss://relay.example/",
            false,
            Duration::from_secs(1),
            Duration::from_secs(2)
        )
        .is_err());
    }

    #[tokio::test]
    async fn missing_configuration_refuses_registration_without_writing_secret() {
        let dir = tempfile::tempdir().unwrap();
        let manager = RelayManager {
            config: Arc::new(Err(RelayError::InvalidConfiguration)),
            clerk: Arc::new(ClerkVerifier::new(
                "http://127.0.0.1:9/jwks".to_string(),
                "https://unused.invalid".to_string(),
            )),
            store: RelayRegistrationStore::new(dir.path()),
            local_addr: Arc::new(RwLock::new(None)),
            supervisor: Arc::new(tokio::sync::Mutex::new(None)),
        };
        assert_eq!(
            manager
                .register(
                    "user_owner".to_string(),
                    "0123456789abcdef0123456789abcdef".to_string()
                )
                .await,
            Err(RelayError::InvalidConfiguration)
        );
        assert!(!dir.path().join(REGISTRATION_FILE).exists());
    }

    #[test]
    fn loopback_normalization_never_uses_unspecified_address() {
        assert_eq!(
            loopback_addr("0.0.0.0:4477".parse().unwrap()),
            "127.0.0.1:4477".parse().unwrap()
        );
        assert_eq!(
            loopback_addr("[::]:4477".parse().unwrap()),
            "[::1]:4477".parse().unwrap()
        );
    }

    #[test]
    fn registration_is_stable_owner_only_and_never_debuggable() {
        let dir = tempfile::tempdir().unwrap();
        let store = RelayRegistrationStore::new(dir.path());
        let first = store
            .register("user_owner", "0123456789abcdef0123456789abcdef")
            .unwrap();
        let second = store
            .register("user_owner", "0123456789abcdef0123456789abcdef")
            .unwrap();
        assert_eq!(first.relay_credential, second.relay_credential);
        assert_eq!(first.relay_credential.len(), 64);
        assert!(matches!(
            store.register("user_other", "0123456789abcdef0123456789abcdef"),
            Err(RelayError::OwnershipConflict)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn registration_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = RelayRegistrationStore::new(dir.path());
        store
            .register("user_owner", "0123456789abcdef0123456789abcdef")
            .unwrap();
        let mode = std::fs::metadata(dir.path().join(REGISTRATION_FILE))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn corrupt_or_unknown_registration_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(REGISTRATION_FILE);
        std::fs::write(&path, br#"{"version":1,"clerk_user_id":"user","box_id":"0123456789abcdef0123456789abcdef","relay_credential":"bad","extra":true}"#).unwrap();
        let store = RelayRegistrationStore::new(dir.path());
        assert!(matches!(store.read(), Err(RelayError::CorruptRegistration)));
    }

    #[cfg(unix)]
    #[test]
    fn registration_symlink_is_rejected() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.json");
        std::fs::write(&target, "{}").unwrap();
        symlink(&target, dir.path().join(REGISTRATION_FILE)).unwrap();
        let store = RelayRegistrationStore::new(dir.path());
        assert!(matches!(store.read(), Err(RelayError::CorruptRegistration)));
    }

    #[tokio::test]
    async fn connector_reconnects_with_backoff_and_heartbeats() {
        let dir = tempfile::tempdir().unwrap();
        persist_test_registration(dir.path());
        let state = FakeRelayState::control_only(true, true);
        let relay_url = start_fake_relay(state.clone()).await;
        let manager = RelayManager::for_test(
            dir.path().to_path_buf(),
            &relay_url,
            "http://127.0.0.1:9/jwks",
            "https://unused.invalid",
            Duration::from_millis(20),
            Duration::from_millis(100),
        )
        .unwrap();
        manager.start("127.0.0.1:9".parse().unwrap()).await.unwrap();

        tokio::time::timeout(Duration::from_secs(3), async {
            while state.registration_count.load(Ordering::SeqCst) < 2 {
                state.registered.notified().await;
            }
        })
        .await
        .unwrap();
        tokio::time::timeout(Duration::from_secs(1), state.heartbeat.notified())
            .await
            .unwrap();
        assert!(state.heartbeat_count.load(Ordering::SeqCst) >= 1);
    }

    #[tokio::test]
    async fn missing_pong_is_fail_closed_and_forces_reconnect() {
        let dir = tempfile::tempdir().unwrap();
        persist_test_registration(dir.path());
        let state = FakeRelayState::control_only(false, false);
        let relay_url = start_fake_relay(state.clone()).await;
        let manager = RelayManager::for_test(
            dir.path().to_path_buf(),
            &relay_url,
            "http://127.0.0.1:9/jwks",
            "https://unused.invalid",
            Duration::from_millis(10),
            Duration::from_millis(40),
        )
        .unwrap();
        manager.start("127.0.0.1:9".parse().unwrap()).await.unwrap();

        tokio::time::timeout(Duration::from_secs(3), async {
            while state.registration_count.load(Ordering::SeqCst) < 2 {
                state.registered.notified().await;
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn connector_preserves_upload_download_stream_and_websocket_bytes() {
        let dir = tempfile::tempdir().unwrap();
        persist_test_registration(dir.path());

        let mut upload_body = vec![0_u8; 2 * 1024 * 1024];
        for (index, byte) in upload_body.iter_mut().enumerate() {
            *byte = (index % 251) as u8;
        }
        let header = format!(
            "POST /v1/deposit HTTP/1.1\r\nHost: box\r\nContent-Length: {}\r\n\r\n",
            upload_body.len()
        )
        .into_bytes();
        let channel_frames = vec![
            header.clone(),
            upload_body[..1_000_000].to_vec(),
            upload_body[1_000_000..].to_vec(),
        ];
        let expected_upload: Vec<u8> = channel_frames.iter().flatten().copied().collect();

        let response_first = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n\x81\x05hello".to_vec();
        let response_second = b"\x82\x04\x00\x01\x02\x03".to_vec();
        let mut expected_response = response_first.clone();
        expected_response.extend_from_slice(&response_second);

        let local_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = local_listener.local_addr().unwrap();
        let (local_seen_tx, local_seen_rx) = oneshot::channel();
        let expected_upload_len = expected_upload.len();
        tokio::spawn(async move {
            let (mut stream, _) = local_listener.accept().await.unwrap();
            let mut received = vec![0_u8; expected_upload_len];
            stream.read_exact(&mut received).await.unwrap();
            let _ = local_seen_tx.send(received);
            stream.write_all(&response_first).await.unwrap();
            stream.flush().await.unwrap();
            tokio::time::sleep(Duration::from_millis(20)).await;
            stream.write_all(&response_second).await.unwrap();
            stream.shutdown().await.unwrap();
        });

        let (channel_response_tx, channel_response_rx) = oneshot::channel();
        let state = FakeRelayState {
            registration_count: Arc::new(AtomicUsize::new(0)),
            heartbeat_count: Arc::new(AtomicUsize::new(0)),
            registered: Arc::new(Notify::new()),
            heartbeat: Arc::new(Notify::new()),
            drop_first_registration: false,
            respond_to_heartbeat: true,
            channel_frames: Arc::new(channel_frames),
            channel_response: Arc::new(Mutex::new(Some(channel_response_tx))),
            credential: Arc::new(Mutex::new(None)),
        };
        let relay_url = start_fake_relay(state).await;
        let manager = RelayManager::for_test(
            dir.path().to_path_buf(),
            &relay_url,
            "http://127.0.0.1:9/jwks",
            "https://unused.invalid",
            Duration::from_secs(1),
            Duration::from_secs(3),
        )
        .unwrap();
        manager.start(local_addr).await.unwrap();

        assert_eq!(
            tokio::time::timeout(Duration::from_secs(5), local_seen_rx)
                .await
                .unwrap()
                .unwrap(),
            expected_upload
        );
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(5), channel_response_rx)
                .await
                .unwrap()
                .unwrap(),
            expected_response
        );
    }

    #[tokio::test]
    async fn cloud_registration_activates_real_gateway_http_forwarding() {
        let dir = tempfile::tempdir().unwrap();
        let jwks_url = start_jwks_stub().await;
        let request = b"GET /v1/health HTTP/1.1\r\nHost: box\r\nConnection: close\r\n\r\n".to_vec();
        let (response_tx, response_rx) = oneshot::channel();
        let relay_state = FakeRelayState {
            registration_count: Arc::new(AtomicUsize::new(0)),
            heartbeat_count: Arc::new(AtomicUsize::new(0)),
            registered: Arc::new(Notify::new()),
            heartbeat: Arc::new(Notify::new()),
            drop_first_registration: false,
            respond_to_heartbeat: true,
            channel_frames: Arc::new(vec![request]),
            channel_response: Arc::new(Mutex::new(Some(response_tx))),
            credential: Arc::new(Mutex::new(None)),
        };
        let relay_url = start_fake_relay(relay_state).await;
        let manager = RelayManager::for_test(
            dir.path().to_path_buf(),
            &relay_url,
            &jwks_url,
            TEST_ISSUER,
            Duration::from_secs(1),
            Duration::from_secs(3),
        )
        .unwrap();
        let (_, device_token) = DeviceStore::open(dir.path()).issue("test-device");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let gateway_addr = listener.local_addr().unwrap();
        manager.start(gateway_addr).await.unwrap();
        let app = build_router(AppState::with_relay(
            dir.path().to_path_buf(),
            GatewayConfig::default(),
            manager,
        ));
        let gateway_task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let now = chrono::Utc::now().timestamp();
        let registration_response = reqwest::Client::new()
            .post(format!("http://{gateway_addr}/v1/cloud/register"))
            .bearer_auth(device_token)
            .json(&serde_json::json!({
                "clerk_jwt": sign_token("user_owner", TEST_ISSUER, now, now + 300)
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(registration_response.status(), reqwest::StatusCode::OK);

        let forwarded = tokio::time::timeout(Duration::from_secs(5), response_rx)
            .await
            .unwrap()
            .unwrap();
        let forwarded_text = String::from_utf8(forwarded).unwrap();
        assert!(forwarded_text.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(forwarded_text.contains(&format!(
            r#"{{"ok":true,"version":"{}"}}"#,
            env!("CARGO_PKG_VERSION")
        )));
        gateway_task.abort();
    }
}
