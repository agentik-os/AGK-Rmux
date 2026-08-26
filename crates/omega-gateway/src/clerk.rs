//! Strict Clerk session-token verification for the cloud-registration route.
//!
//! Only RS256 is accepted. Signing keys are fetched from Clerk's JWKS and
//! cached for 24 hours; an unknown `kid` causes one forced refresh. Tokens and
//! key material are never included in errors or logs.

use ring::signature::{RsaPublicKeyComponents, RSA_PKCS1_2048_8192_SHA256};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub const CLERK_ISSUER: &str = "https://faithful-cobra-38.clerk.accounts.dev";
pub const CLERK_JWKS_URL: &str =
    "https://faithful-cobra-38.clerk.accounts.dev/.well-known/jwks.json";

const CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const HTTP_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_JWKS_BYTES: usize = 64 * 1024;
const MAX_JWKS_KEYS: usize = 64;
const MAX_TOKEN_BYTES: usize = 16 * 1024;
const MAX_SUB_BYTES: usize = 256;
const MAX_KID_BYTES: usize = 128;
const MAX_CLOCK_SKEW_SECONDS: i64 = 60;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClerkClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClerkVerifyError {
    Malformed,
    UnsupportedAlgorithm,
    UnknownKid,
    JwksUnavailable,
    InvalidSignature,
    InvalidClaims,
}

impl std::fmt::Display for ClerkVerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Malformed => "malformed clerk token",
            Self::UnsupportedAlgorithm => "unsupported clerk token algorithm",
            Self::UnknownKid => "unknown clerk signing key",
            Self::JwksUnavailable => "clerk signing keys unavailable",
            Self::InvalidSignature => "invalid clerk token signature",
            Self::InvalidClaims => "invalid clerk token claims",
        })
    }
}

impl std::error::Error for ClerkVerifyError {}

#[derive(Deserialize)]
struct JwtHeader {
    alg: String,
    kid: String,
}

#[derive(Deserialize)]
struct WireClaims {
    sub: String,
    iss: String,
    exp: i64,
    iat: i64,
    #[serde(default)]
    nbf: Option<i64>,
}

#[derive(Deserialize)]
struct JwksDocument {
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Jwk {
    kty: String,
    kid: String,
    n: String,
    e: String,
    #[serde(default)]
    alg: Option<String>,
    #[serde(rename = "use", default)]
    key_use: Option<String>,
}

#[derive(Clone)]
struct RsaKey {
    modulus: Vec<u8>,
    exponent: Vec<u8>,
}

struct KeyCache {
    keys: HashMap<String, RsaKey>,
    fetched_at: Option<Instant>,
}

pub struct ClerkVerifier {
    jwks_url: String,
    issuer: String,
    http: reqwest::Client,
    cache: RwLock<KeyCache>,
}

impl ClerkVerifier {
    pub fn production() -> Self {
        Self::new(CLERK_JWKS_URL.to_string(), CLERK_ISSUER.to_string())
    }

    pub fn new(jwks_url: String, issuer: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("fixed Clerk HTTP client configuration must be valid");
        Self {
            jwks_url,
            issuer,
            http,
            cache: RwLock::new(KeyCache {
                keys: HashMap::new(),
                fetched_at: None,
            }),
        }
    }

    async fn fetch_jwks(&self) -> Result<HashMap<String, RsaKey>, ClerkVerifyError> {
        let mut response = self
            .http
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|_| ClerkVerifyError::JwksUnavailable)?;
        if !response.status().is_success() {
            return Err(ClerkVerifyError::JwksUnavailable);
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_JWKS_BYTES as u64)
        {
            return Err(ClerkVerifyError::JwksUnavailable);
        }

        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| ClerkVerifyError::JwksUnavailable)?
        {
            if body.len().saturating_add(chunk.len()) > MAX_JWKS_BYTES {
                return Err(ClerkVerifyError::JwksUnavailable);
            }
            body.extend_from_slice(&chunk);
        }

        let document: JwksDocument =
            serde_json::from_slice(&body).map_err(|_| ClerkVerifyError::JwksUnavailable)?;
        if document.keys.is_empty() || document.keys.len() > MAX_JWKS_KEYS {
            return Err(ClerkVerifyError::JwksUnavailable);
        }

        let mut keys = HashMap::new();
        for jwk in document.keys {
            if jwk.kty != "RSA"
                || jwk.alg.as_deref().is_some_and(|alg| alg != "RS256")
                || jwk.key_use.as_deref().is_some_and(|usage| usage != "sig")
                || jwk.kid.is_empty()
                || jwk.kid.len() > MAX_KID_BYTES
            {
                continue;
            }
            let modulus =
                decode_base64url(&jwk.n, 1024).map_err(|_| ClerkVerifyError::JwksUnavailable)?;
            let exponent =
                decode_base64url(&jwk.e, 8).map_err(|_| ClerkVerifyError::JwksUnavailable)?;
            if modulus.len() < 256 || modulus.len() > 1024 || exponent.is_empty() {
                continue;
            }
            if keys.insert(jwk.kid, RsaKey { modulus, exponent }).is_some() {
                return Err(ClerkVerifyError::JwksUnavailable);
            }
        }
        if keys.is_empty() {
            return Err(ClerkVerifyError::JwksUnavailable);
        }
        Ok(keys)
    }

    async fn replace_cache(&self) -> Result<(), ClerkVerifyError> {
        let keys = self.fetch_jwks().await?;
        let mut cache = self.cache.write().await;
        cache.keys = keys;
        cache.fetched_at = Some(Instant::now());
        Ok(())
    }

    async fn key_for(&self, kid: &str) -> Result<RsaKey, ClerkVerifyError> {
        let fresh_key = {
            let cache = self.cache.read().await;
            let fresh = cache
                .fetched_at
                .is_some_and(|instant| instant.elapsed() <= CACHE_TTL);
            fresh.then(|| cache.keys.get(kid).cloned()).flatten()
        };
        if let Some(key) = fresh_key {
            return Ok(key);
        }

        self.replace_cache().await?;
        let cache = self.cache.read().await;
        cache
            .keys
            .get(kid)
            .cloned()
            .ok_or(ClerkVerifyError::UnknownKid)
    }

    pub async fn verify(&self, token: &str) -> Result<ClerkClaims, ClerkVerifyError> {
        self.verify_at(token, chrono::Utc::now().timestamp()).await
    }

    async fn verify_at(&self, token: &str, now: i64) -> Result<ClerkClaims, ClerkVerifyError> {
        if token.is_empty() || token.len() > MAX_TOKEN_BYTES || !token.is_ascii() {
            return Err(ClerkVerifyError::Malformed);
        }
        let mut parts = token.split('.');
        let header_segment = parts.next().ok_or(ClerkVerifyError::Malformed)?;
        let claims_segment = parts.next().ok_or(ClerkVerifyError::Malformed)?;
        let signature_segment = parts.next().ok_or(ClerkVerifyError::Malformed)?;
        if parts.next().is_some()
            || header_segment.is_empty()
            || claims_segment.is_empty()
            || signature_segment.is_empty()
        {
            return Err(ClerkVerifyError::Malformed);
        }

        let header_bytes =
            decode_base64url(header_segment, 4096).map_err(|_| ClerkVerifyError::Malformed)?;
        let header: JwtHeader =
            serde_json::from_slice(&header_bytes).map_err(|_| ClerkVerifyError::Malformed)?;
        if header.alg != "RS256" {
            return Err(ClerkVerifyError::UnsupportedAlgorithm);
        }
        if header.kid.is_empty() || header.kid.len() > MAX_KID_BYTES {
            return Err(ClerkVerifyError::Malformed);
        }

        let signature =
            decode_base64url(signature_segment, 1024).map_err(|_| ClerkVerifyError::Malformed)?;
        let key = self.key_for(&header.kid).await?;
        let signing_input = format!("{header_segment}.{claims_segment}");
        RsaPublicKeyComponents {
            n: &key.modulus,
            e: &key.exponent,
        }
        .verify(
            &RSA_PKCS1_2048_8192_SHA256,
            signing_input.as_bytes(),
            &signature,
        )
        .map_err(|_| ClerkVerifyError::InvalidSignature)?;

        let claims_bytes =
            decode_base64url(claims_segment, 12 * 1024).map_err(|_| ClerkVerifyError::Malformed)?;
        let claims: WireClaims =
            serde_json::from_slice(&claims_bytes).map_err(|_| ClerkVerifyError::Malformed)?;
        if claims.iss != self.issuer
            || claims.sub.is_empty()
            || claims.sub.len() > MAX_SUB_BYTES
            || claims.exp <= now
            || claims.exp <= claims.iat
            || claims.iat > now.saturating_add(MAX_CLOCK_SKEW_SECONDS)
            || claims
                .nbf
                .is_some_and(|nbf| nbf > now.saturating_add(MAX_CLOCK_SKEW_SECONDS))
        {
            return Err(ClerkVerifyError::InvalidClaims);
        }

        Ok(ClerkClaims {
            sub: claims.sub,
            exp: claims.exp,
            iat: claims.iat,
        })
    }
}

/// Decode an unpadded RFC 4648 base64url value with a caller-supplied output
/// bound. JWT and JWK fields must be canonical and therefore reject padding.
fn decode_base64url(input: &str, max_output: usize) -> Result<Vec<u8>, ()> {
    if input.is_empty() || input.contains('=') || input.len() % 4 == 1 {
        return Err(());
    }
    let expected = input.len().saturating_mul(3).saturating_add(3) / 4;
    if expected > max_output {
        return Err(());
    }

    let mut output = Vec::with_capacity(expected);
    let mut accumulator = 0_u32;
    let mut bits = 0_u8;
    for byte in input.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => return Err(()),
        };
        accumulator = (accumulator << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((accumulator >> bits) as u8);
            accumulator &= (1_u32 << bits).saturating_sub(1);
        }
    }
    if accumulator != 0 || output.len() > max_output {
        return Err(());
    }
    Ok(output)
}

#[cfg(test)]
pub(crate) mod testkit {
    use axum::routing::get;
    use axum::{Json, Router};
    use ring::rand::SystemRandom;
    use ring::signature::{RsaKeyPair, RSA_PKCS1_SHA256};
    use serde_json::{json, Value};
    use tokio::net::TcpListener;

    pub const TEST_ISSUER: &str = "https://test.clerk.accounts.dev";
    pub const TEST_KID: &str = "test-key-1";
    pub const TEST_N: &str = "k7J-wkkgQJ7FRIMm7jTt8HGra8AIv54C2KpdqkiJg1ygEnspzBZAtMeizLI2cReTic4KwNH7h8Me8Xa4szNJyxk6w11kU9ylHERgseL_BgvV81vTGUdGaqKlzm5S7jbMHaBUP-UysnrcTllzlyXJTHJXdR9kH8iR5XJZiHd3wSMbsi25wAFH0olb8yQUI9I8d5XXcfddTv-xI7UDGiZaubNsozedUPqYzr3VOoJGc4ugVZYVgQqcV64Bx62wJynhDxaYCoJBL0UYMEHyfCb6nOrPdyypBHXFerb4fLsd1orKiI-ck94bpgTYpMYHXoeel7fv6x6mKPOpBDU8qDaRTQ";
    pub const TEST_E: &str = "AQAB";
    const TEST_PRIVATE_KEY: &str = r#"
MIIEpAIBAAKCAQEAk7J+wkkgQJ7FRIMm7jTt8HGra8AIv54C2KpdqkiJg1ygEnsp
zBZAtMeizLI2cReTic4KwNH7h8Me8Xa4szNJyxk6w11kU9ylHERgseL/BgvV81vT
GUdGaqKlzm5S7jbMHaBUP+UysnrcTllzlyXJTHJXdR9kH8iR5XJZiHd3wSMbsi25
wAFH0olb8yQUI9I8d5XXcfddTv+xI7UDGiZaubNsozedUPqYzr3VOoJGc4ugVZYV
gQqcV64Bx62wJynhDxaYCoJBL0UYMEHyfCb6nOrPdyypBHXFerb4fLsd1orKiI+c
k94bpgTYpMYHXoeel7fv6x6mKPOpBDU8qDaRTQIDAQABAoIBABN7fplYiwlrHWtk
VMzO149uzGVktI1jevhN6hgNrxylqbR2YZ+QVI25gxc2736Af8+JicVAjZ+Cv1YA
9UEneUrx84c/Y6i6Q19UrQqZ/k4zs8+ro6TDJZ3oVtsW6203y49MOFYgq3d77wvj
ZHTMy7OqpjBcE9mAxq9Po6bK14BMKCJFYRpms6qKhvGtl9/2t141f8nR0DXVNIV8
igEwPbiyXtLJeQW+oAUH6giafTuqDu5tiSWl8xdBw4KjrdYWoV3M8Qz0TM3W7qN3
Sy/R73q4RV730GXL/lQ4mk6YMqKjHRS2WXc5l5lV+KDoNUjhaJfVtI7pJN4CmPcO
YKRieaMCgYEAwpz0bl60RZtwyLk1fzIynMY+QHC4L4HdsxSR0txozd1wj44fp0VH
L1758SmD/Tbv3mzhMfSbPNT/2jNSLoqOIbzNsqXJw1j9WfKBWaRUE6aWcPJzHNmR
xnC17UyokyPi8IgpBwOg57LAV+ZiIvm7J0fP4WNE1uY+LFsuVb6iE8sCgYEAwkkT
bZHm0SEaQjBitTcoQOsYs6BXCAaHxFewCxHuNQjqOynS53b3ADOycmarDHEHnD8d
EgGKv4thhx++1s98EA258D/tCYcQMI47lyMOoLhqgdmBpuPaIuTmo0yfzJY+k1rZ
18aqiiGf8HFfGBcxIu6wFNRy6m0OgSqxQpTuvEcCgYEAttI3lY+GDnXtx8KljO4d
OcXKKUM64/Y9zMOgEdRY6DwER/edqMeeDdRNPM5hXfjD8dGa5BED3GvERrk7lNk4
mF0DZ7XzCn6I0nzMIugKy8MNROGeXhXNqfusDFadbkyiHo/q6tnvyHnV0z1sJ6su
s8H0eamFu9PjyEuIBajmHW0CgYEAis00YRJcicoRfXod+wNV8dNECYiixOwNcPuI
nNAJk8Azv2Lo906ptm44rbyltTTHUBxTA3swihnk6mmGqOcA5mM4FOlGyojAyz/C
WP7Fw5MdHHmtQ/MC8+z+zWm/vKUWPaYpfhvD2P2ey2PjoU1oYGGQUMFa/Bo/w44h
p8TJLp8CgYBWxDZdjolvi8eONZmsiTBV5MKmVs9f5Xu7phuVnOK6A8ZC5GRqbvpL
Ku+c4e46F80daULY3OEau2cPR/DBEvCJZgMmILoolXUFLg3E3fB/BRMqBPMFOPBD
5nkdlwaE8kcx8UfbhBlQsSmWlsyIlwTL9kSpXrLSPBq8jOyfRuEhXg==
"#;

    pub async fn start_jwks_stub() -> String {
        let body = json!({
            "keys": [{
                "kty": "RSA",
                "use": "sig",
                "alg": "RS256",
                "kid": TEST_KID,
                "n": TEST_N,
                "e": TEST_E
            }]
        });
        let app = Router::new().route(
            "/.well-known/jwks.json",
            get(move || {
                let body = body.clone();
                async move { Json(body) }
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{address}/.well-known/jwks.json")
    }

    pub fn sign_token(sub: &str, issuer: &str, iat: i64, exp: i64) -> String {
        sign_custom(
            TEST_KID,
            "RS256",
            json!({
                "sub": sub,
                "iss": issuer,
                "iat": iat,
                "exp": exp
            }),
        )
    }

    pub fn sign_custom(kid: &str, algorithm: &str, claims: Value) -> String {
        let header = json!({ "alg": algorithm, "typ": "JWT", "kid": kid });
        let header = encode_base64url(&serde_json::to_vec(&header).unwrap());
        let claims = encode_base64url(&serde_json::to_vec(&claims).unwrap());
        let signing_input = format!("{header}.{claims}");
        let key_bytes = decode_base64_standard(TEST_PRIVATE_KEY);
        let key_pair = RsaKeyPair::from_der(&key_bytes).unwrap();
        let mut signature = vec![0_u8; key_pair.public().modulus_len()];
        key_pair
            .sign(
                &RSA_PKCS1_SHA256,
                &SystemRandom::new(),
                signing_input.as_bytes(),
                &mut signature,
            )
            .unwrap();
        format!("{signing_input}.{}", encode_base64url(&signature))
    }

    fn encode_base64url(bytes: &[u8]) -> String {
        encode_base64(
            bytes,
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
        )
    }

    fn encode_base64(bytes: &[u8], alphabet: &[u8; 64]) -> String {
        let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
        for chunk in bytes.chunks(3) {
            let word = (u32::from(chunk[0]) << 16)
                | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
                | u32::from(*chunk.get(2).unwrap_or(&0));
            output.push(alphabet[((word >> 18) & 63) as usize] as char);
            output.push(alphabet[((word >> 12) & 63) as usize] as char);
            if chunk.len() > 1 {
                output.push(alphabet[((word >> 6) & 63) as usize] as char);
            }
            if chunk.len() > 2 {
                output.push(alphabet[(word & 63) as usize] as char);
            }
        }
        output
    }

    fn decode_base64_standard(input: &str) -> Vec<u8> {
        let normalized: String = input.chars().filter(|ch| !ch.is_whitespace()).collect();
        let translated = normalized
            .trim_end_matches('=')
            .replace('+', "-")
            .replace('/', "_");
        super::decode_base64url(&translated, 4096).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::testkit::{sign_custom, sign_token, start_jwks_stub, TEST_ISSUER};
    use super::{decode_base64url, ClerkVerifier, ClerkVerifyError};
    use serde_json::json;

    #[test]
    fn base64url_decoder_accepts_canonical_unpadded_values() {
        assert_eq!(
            decode_base64url("eyJhbGciOiJSUzI1NiJ9", 32).unwrap(),
            br#"{"alg":"RS256"}"#
        );
        assert_eq!(decode_base64url("AQAB", 8).unwrap(), [1, 0, 1]);
    }

    #[test]
    fn base64url_decoder_rejects_padding_invalid_tail_and_oversize() {
        assert!(decode_base64url("AQAB=", 8).is_err());
        assert!(decode_base64url("A", 8).is_err());
        assert!(decode_base64url("AB", 8).is_err());
        assert!(decode_base64url("AQAB", 2).is_err());
    }

    #[tokio::test]
    async fn valid_rs256_token_verifies_against_local_jwks() {
        let jwks = start_jwks_stub().await;
        let verifier = ClerkVerifier::new(jwks, TEST_ISSUER.to_string());
        let now = chrono::Utc::now().timestamp();
        let claims = verifier
            .verify(&sign_token("user_123", TEST_ISSUER, now, now + 300))
            .await
            .unwrap();
        assert_eq!(claims.sub, "user_123");
    }

    #[tokio::test]
    async fn expired_wrong_issuer_future_and_bad_signature_are_rejected() {
        let jwks = start_jwks_stub().await;
        let verifier = ClerkVerifier::new(jwks, TEST_ISSUER.to_string());
        let now = chrono::Utc::now().timestamp();
        assert_eq!(
            verifier
                .verify(&sign_token("user", TEST_ISSUER, now - 600, now - 1))
                .await,
            Err(ClerkVerifyError::InvalidClaims)
        );
        assert_eq!(
            verifier
                .verify(&sign_token(
                    "user",
                    "https://attacker.invalid",
                    now,
                    now + 300
                ))
                .await,
            Err(ClerkVerifyError::InvalidClaims)
        );
        assert_eq!(
            verifier
                .verify(&sign_token("user", TEST_ISSUER, now + 3600, now + 7200))
                .await,
            Err(ClerkVerifyError::InvalidClaims)
        );
        // Corrupt the FIRST character of the signature segment, never the last.
        // A 2048-bit RS256 signature base64url-encodes to 342 characters whose
        // LAST one carries only 2 significant bits, so replacing it with 'A'
        // decodes to the SAME bytes for every original value below 16 — the
        // signature stayed valid and this assertion failed about one run in
        // four. The first character carries a full 6 bits, so changing it
        // always changes the signature.
        let bad_signature = {
            let (head, signature) = sign_token("user", TEST_ISSUER, now, now + 300)
                .rsplit_once('.')
                .map(|(h, s)| (h.to_string(), s.to_string()))
                .expect("a JWT has three dot-separated segments");
            let mut chars = signature.chars();
            let first = chars.next().expect("a non-empty signature");
            let replacement = if first == 'A' { 'B' } else { 'A' };
            format!("{head}.{replacement}{}", chars.as_str())
        };
        assert_eq!(
            verifier.verify(&bad_signature).await,
            Err(ClerkVerifyError::InvalidSignature)
        );
    }

    #[tokio::test]
    async fn non_rs256_unknown_kid_and_future_nbf_are_rejected() {
        let jwks = start_jwks_stub().await;
        let verifier = ClerkVerifier::new(jwks, TEST_ISSUER.to_string());
        let now = chrono::Utc::now().timestamp();
        let claims = json!({
            "sub": "user",
            "iss": TEST_ISSUER,
            "iat": now,
            "exp": now + 300
        });
        assert_eq!(
            verifier
                .verify(&sign_custom("test-key-1", "HS256", claims.clone()))
                .await,
            Err(ClerkVerifyError::UnsupportedAlgorithm)
        );
        assert_eq!(
            verifier
                .verify(&sign_custom("missing-key", "RS256", claims))
                .await,
            Err(ClerkVerifyError::UnknownKid)
        );
        assert_eq!(
            verifier
                .verify(&sign_custom(
                    "test-key-1",
                    "RS256",
                    json!({
                        "sub": "user",
                        "iss": TEST_ISSUER,
                        "iat": now,
                        "exp": now + 7200,
                        "nbf": now + 3600
                    })
                ))
                .await,
            Err(ClerkVerifyError::InvalidClaims)
        );
    }
}
