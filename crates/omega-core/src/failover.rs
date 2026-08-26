//! OmegaFailover — error classifier + provider-failover taxonomy.
//!
//! When a provider call fails, the bridge / oracle needs to decide:
//! retry on the same provider? switch to a different model? abort?
//! Classification turns raw error text into a discrete decision lane.
//!
//! Rebrand of the upstream `FailoverReason` enum, extended for the
//! OmegaOS multi-provider matrix (Claude, Codex, Gemini, GLM, Pi,
//! Hermes). We intentionally start with a small skeleton + per-message
//! patterns — richer per-provider parsing comes in a later pass.

use serde::{Deserialize, Serialize};

/// Why a provider call failed and what the orchestrator should do next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailoverReason {
    /// HTTP 429 / explicit rate-limit message. Retry after backoff.
    RateLimit,
    /// HTTP 401/403 — token revoked, expired, or wrong scope.
    Auth,
    /// HTTP 5xx — upstream is broken; switch provider if possible.
    UpstreamError,
    /// Network-level error (DNS, TCP, TLS, timeout). Local retry.
    Network,
    /// Context window exceeded — the request is too large; compress or
    /// split before retrying.
    ContextOverflow,
    /// Model deprecated / unavailable. Switch model.
    ModelUnavailable,
    /// Tool / function call schema rejected. The LLM produced invalid
    /// JSON or referenced an unknown tool — refeed the schema.
    InvalidToolCall,
    /// Content moderation refusal. Don't retry; escalate or rephrase.
    ContentRefused,
    /// Budget exhausted (see `OmegaBudget`). Stop spending.
    BudgetExhausted,
    /// Catch-all for unrecognized errors.
    Unknown,
}

impl FailoverReason {
    /// Suggested orchestrator action.
    pub fn next_action(&self) -> NextAction {
        match self {
            Self::RateLimit => NextAction::Backoff,
            Self::Auth => NextAction::Escalate,
            Self::UpstreamError => NextAction::SwitchProvider,
            Self::Network => NextAction::Retry,
            Self::ContextOverflow => NextAction::Compress,
            Self::ModelUnavailable => NextAction::SwitchModel,
            Self::InvalidToolCall => NextAction::RefeedSchema,
            Self::ContentRefused => NextAction::Escalate,
            Self::BudgetExhausted => NextAction::Stop,
            Self::Unknown => NextAction::Retry,
        }
    }

    /// Secret-safe message suitable for a remote client. Raw provider errors
    /// stay in local logs because they may echo credentials, prompts, or file
    /// content.
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::RateLimit => "provider rate limit reached; retry after backoff",
            Self::Auth => "provider authentication failed; re-authenticate this agent",
            Self::UpstreamError => "provider service is unavailable; retry or switch provider",
            Self::Network => "provider network request failed; check connectivity and retry",
            Self::ContextOverflow => "agent context is full; compact or start a new session",
            Self::ModelUnavailable => "selected model is unavailable; choose a current model",
            Self::InvalidToolCall => "provider rejected an agent tool call; inspect local logs",
            Self::ContentRefused => "provider refused the request under its content policy",
            Self::BudgetExhausted => "provider budget is exhausted",
            Self::Unknown => "agent turn failed; inspect local gateway logs",
        }
    }
}

/// What the orchestrator should do based on a classified failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NextAction {
    /// Wait (exponential backoff) then retry the same call.
    Backoff,
    /// Retry once on the same provider.
    Retry,
    /// Pick a different provider (Claude → Gemini, etc.).
    SwitchProvider,
    /// Same provider, different model.
    SwitchModel,
    /// Reduce the input (compress context) and retry.
    Compress,
    /// Re-send the tool schemas and retry the model call.
    RefeedSchema,
    /// Stop trying. Surface to the user.
    Stop,
    /// Stop AND raise alarm — surfaces in Telegram with priority.
    Escalate,
}

/// Classify a raw error message + optional HTTP status into a reason.
/// Patterns are matched in priority order. First hit wins.
pub fn classify(http_status: Option<u16>, message: &str) -> FailoverReason {
    // 1. HTTP status — fast-path the obvious lanes.
    if let Some(code) = http_status {
        match code {
            429 => return FailoverReason::RateLimit,
            401 | 403 => return FailoverReason::Auth,
            413 => return FailoverReason::ContextOverflow,
            500..=599 => return FailoverReason::UpstreamError,
            _ => {}
        }
    }

    let lower = message.to_lowercase();

    // 2. Message patterns. Ordered most-specific → most-generic.
    let patterns: &[(&[&str], FailoverReason)] = &[
        (
            &["budget exhausted", "no budget left"],
            FailoverReason::BudgetExhausted,
        ),
        (
            &["context length", "maximum context", "token limit"],
            FailoverReason::ContextOverflow,
        ),
        (
            &["model not found", "model_not_found", "deprecated"],
            FailoverReason::ModelUnavailable,
        ),
        (
            &["unknown tool", "invalid tool", "tool schema"],
            FailoverReason::InvalidToolCall,
        ),
        (
            &["refused", "content policy", "safety"],
            FailoverReason::ContentRefused,
        ),
        (
            &["rate limit", "too many requests", "quota"],
            FailoverReason::RateLimit,
        ),
        (
            &["unauthorized", "invalid api key", "authentication"],
            FailoverReason::Auth,
        ),
        (
            &["timeout", "timed out", "connection reset", "dns"],
            FailoverReason::Network,
        ),
        (
            &["internal server error", "service unavailable", "upstream"],
            FailoverReason::UpstreamError,
        ),
    ];

    for (needles, reason) in patterns {
        if needles.iter().any(|n| lower.contains(n)) {
            return *reason;
        }
    }
    FailoverReason::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_429_rate_limit() {
        assert_eq!(classify(Some(429), "whatever"), FailoverReason::RateLimit);
    }

    #[test]
    fn http_401_auth() {
        assert_eq!(classify(Some(401), ""), FailoverReason::Auth);
    }

    #[test]
    fn http_500_upstream() {
        assert_eq!(classify(Some(503), ""), FailoverReason::UpstreamError);
    }

    #[test]
    fn message_context_overflow() {
        assert_eq!(
            classify(None, "Error: maximum context length exceeded"),
            FailoverReason::ContextOverflow
        );
    }

    #[test]
    fn message_budget() {
        assert_eq!(
            classify(None, "Mission aborted — budget exhausted"),
            FailoverReason::BudgetExhausted
        );
    }

    #[test]
    fn message_network_timeout() {
        assert_eq!(classify(None, "request timed out"), FailoverReason::Network);
    }

    #[test]
    fn unknown_fallback() {
        assert_eq!(classify(None, "some weird thing"), FailoverReason::Unknown);
    }

    #[test]
    fn next_action_consistency() {
        assert_eq!(FailoverReason::RateLimit.next_action(), NextAction::Backoff);
        assert_eq!(FailoverReason::Auth.next_action(), NextAction::Escalate);
        assert_eq!(
            FailoverReason::ContextOverflow.next_action(),
            NextAction::Compress
        );
        assert_eq!(
            FailoverReason::BudgetExhausted.next_action(),
            NextAction::Stop
        );
        assert!(FailoverReason::Auth
            .user_message()
            .contains("re-authenticate"));
    }
}
