//! Shared types for quota and rate limit tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Quota metadata extracted from provider responses (HTTP headers or errors).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaMetadata {
    /// Number of requests remaining in current quota window
    pub rate_limit_remaining: Option<u64>,
    /// Timestamp when the rate limit resets (UTC)
    pub rate_limit_reset_at: Option<DateTime<Utc>>,
    /// Number of seconds to wait before retry (from Retry-After header)
    pub retry_after_seconds: Option<u64>,
    /// Maximum requests allowed in quota window (if available)
    pub rate_limit_total: Option<u64>,
}

/// Status of a provider's quota and circuit breaker state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuotaStatus {
    /// Provider is healthy and available
    Ok,
    /// Provider is rate-limited but circuit is still closed
    RateLimited,
    /// Circuit breaker is open (too many failures)
    CircuitOpen,
    /// OAuth profile quota exhausted
    QuotaExhausted,
}

/// Per-provider quota information combining health state and OAuth profile metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderQuotaInfo {
    pub provider: String,
    pub status: QuotaStatus,
    pub failure_count: u32,
    pub last_error: Option<String>,
    pub retry_after_seconds: Option<u64>,
    pub circuit_resets_at: Option<DateTime<Utc>>,
    pub profiles: Vec<ProfileQuotaInfo>,
}

/// Per-OAuth-profile quota information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileQuotaInfo {
    pub profile_name: String,
    pub status: QuotaStatus,
    pub rate_limit_remaining: Option<u64>,
    pub rate_limit_reset_at: Option<DateTime<Utc>>,
    pub rate_limit_total: Option<u64>,
    /// Account identifier (email, workspace ID, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// When the OAuth token / subscription expires
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<DateTime<Utc>>,
    /// Plan type (free, pro, enterprise) if known
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,
}

/// Summary of all providers' quota status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaSummary {
    pub timestamp: DateTime<Utc>,
    pub providers: Vec<ProviderQuotaInfo>,
}

impl QuotaSummary {
    /// Get available (healthy) providers
    pub fn available_providers(&self) -> Vec<&str> {
        self.providers
            .iter()
            .filter(|p| p.status == QuotaStatus::Ok)
            .map(|p| p.provider.as_str())
            .collect()
    }

    /// Get rate-limited providers
    pub fn rate_limited_providers(&self) -> Vec<&str> {
        self.providers
            .iter()
            .filter(|p| {
                p.status == QuotaStatus::RateLimited || p.status == QuotaStatus::QuotaExhausted
            })
            .map(|p| p.provider.as_str())
            .collect()
    }

    /// Get circuit-open providers
    pub fn circuit_open_providers(&self) -> Vec<&str> {
        self.providers
            .iter()
            .filter(|p| p.status == QuotaStatus::CircuitOpen)
            .map(|p| p.provider.as_str())
            .collect()
    }
}

/// Provider usage metrics (tracked per request).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsageMetrics {
    pub provider: String,
    pub requests_today: u64,
    pub requests_session: u64,
    pub tokens_input_today: u64,
    pub tokens_output_today: u64,
    pub tokens_input_session: u64,
    pub tokens_output_session: u64,
    pub cost_usd_today: f64,
    pub cost_usd_session: f64,
    pub daily_request_limit: u64,
    pub daily_token_limit: u64,
    pub last_reset_at: DateTime<Utc>,
}

impl Default for ProviderUsageMetrics {
    fn default() -> Self {
        Self {
            provider: String::new(),
            requests_today: 0,
            requests_session: 0,
            tokens_input_today: 0,
            tokens_output_today: 0,
            tokens_input_session: 0,
            tokens_output_session: 0,
            cost_usd_today: 0.0,
            cost_usd_session: 0.0,
            daily_request_limit: 0,
            daily_token_limit: 0,
            last_reset_at: Utc::now(),
        }
    }
}

impl ProviderUsageMetrics {
    pub fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_string(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_provider(name: &str, status: QuotaStatus) -> ProviderQuotaInfo {
        ProviderQuotaInfo {
            provider: name.to_string(),
            status,
            failure_count: 0,
            last_error: None,
            retry_after_seconds: None,
            circuit_resets_at: None,
            profiles: vec![],
        }
    }

    fn make_summary(providers: Vec<ProviderQuotaInfo>) -> QuotaSummary {
        QuotaSummary {
            timestamp: Utc::now(),
            providers,
        }
    }

    /// REQ-PROV-018-SC01
    #[test]
    fn quota_summary_available_providers() {
        let summary = make_summary(vec![
            make_provider("openai", QuotaStatus::Ok),
            make_provider("anthropic", QuotaStatus::RateLimited),
            make_provider("gemini", QuotaStatus::Ok),
            make_provider("bedrock", QuotaStatus::CircuitOpen),
        ]);
        let available = summary.available_providers();
        assert_eq!(available, vec!["openai", "gemini"]);
    }

    /// REQ-PROV-018-SC02
    #[test]
    fn quota_summary_rate_limited_providers() {
        let summary = make_summary(vec![
            make_provider("openai", QuotaStatus::Ok),
            make_provider("anthropic", QuotaStatus::RateLimited),
            make_provider("gemini", QuotaStatus::QuotaExhausted),
            make_provider("bedrock", QuotaStatus::CircuitOpen),
        ]);
        let rate_limited = summary.rate_limited_providers();
        assert_eq!(rate_limited, vec!["anthropic", "gemini"]);
    }

    /// REQ-PROV-018-SC03
    #[test]
    fn quota_summary_circuit_open_providers() {
        let summary = make_summary(vec![
            make_provider("openai", QuotaStatus::Ok),
            make_provider("anthropic", QuotaStatus::CircuitOpen),
            make_provider("gemini", QuotaStatus::RateLimited),
        ]);
        let circuit_open = summary.circuit_open_providers();
        assert_eq!(circuit_open, vec!["anthropic"]);
    }

    /// REQ-PROV-018-SC04
    #[test]
    fn quota_summary_empty_providers() {
        let summary = make_summary(vec![]);
        assert!(summary.available_providers().is_empty());
        assert!(summary.rate_limited_providers().is_empty());
        assert!(summary.circuit_open_providers().is_empty());
    }

    /// REQ-PROV-018-SC05
    #[test]
    fn quota_summary_all_ok() {
        let summary = make_summary(vec![
            make_provider("a", QuotaStatus::Ok),
            make_provider("b", QuotaStatus::Ok),
        ]);
        assert_eq!(summary.available_providers().len(), 2);
        assert!(summary.rate_limited_providers().is_empty());
        assert!(summary.circuit_open_providers().is_empty());
    }

    /// REQ-PROV-018-SC06
    #[test]
    fn provider_usage_metrics_new() {
        let metrics = ProviderUsageMetrics::new("openai");
        assert_eq!(metrics.provider, "openai");
        assert_eq!(metrics.requests_today, 0);
        assert_eq!(metrics.cost_usd_today, 0.0);
    }

    /// REQ-PROV-018-SC07
    #[test]
    fn provider_usage_metrics_default() {
        let metrics = ProviderUsageMetrics::default();
        assert_eq!(metrics.provider, "");
        assert_eq!(metrics.requests_session, 0);
        assert_eq!(metrics.tokens_input_today, 0);
    }

    /// REQ-PROV-018-SC08
    #[test]
    fn quota_status_serde_roundtrip() {
        let status = QuotaStatus::RateLimited;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"rate_limited\"");
        let parsed: QuotaStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, QuotaStatus::RateLimited);
    }

    /// REQ-PROV-018-SC09
    #[test]
    fn quota_metadata_construction() {
        let meta = QuotaMetadata {
            rate_limit_remaining: Some(100),
            rate_limit_reset_at: None,
            retry_after_seconds: Some(30),
            rate_limit_total: Some(1000),
        };
        assert_eq!(meta.rate_limit_remaining, Some(100));
        assert_eq!(meta.retry_after_seconds, Some(30));
    }

    /// REQ-PROV-018-SC10
    #[test]
    fn provider_quota_info_with_profiles() {
        let info = ProviderQuotaInfo {
            provider: "gemini".to_string(),
            status: QuotaStatus::Ok,
            failure_count: 0,
            last_error: None,
            retry_after_seconds: None,
            circuit_resets_at: None,
            profiles: vec![ProfileQuotaInfo {
                profile_name: "default".to_string(),
                status: QuotaStatus::Ok,
                rate_limit_remaining: Some(50),
                rate_limit_reset_at: None,
                rate_limit_total: Some(500),
                account_id: Some("user@example.com".to_string()),
                token_expires_at: None,
                plan_type: Some("pro".to_string()),
            }],
        };
        assert_eq!(info.profiles.len(), 1);
        assert_eq!(info.profiles[0].plan_type, Some("pro".to_string()));
    }
}
