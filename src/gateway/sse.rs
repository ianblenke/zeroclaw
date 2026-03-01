//! Server-Sent Events (SSE) stream for real-time event delivery.
//!
//! Wraps the broadcast channel in AppState to deliver events to web dashboard clients.

use super::AppState;
use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SseAuthRejection {
    MissingPairingToken,
    NonLocalWithoutAuthLayer,
}

fn evaluate_sse_auth(
    pairing_required: bool,
    is_loopback_request: bool,
    has_valid_pairing_token: bool,
) -> Option<SseAuthRejection> {
    if pairing_required {
        return (!has_valid_pairing_token).then_some(SseAuthRejection::MissingPairingToken);
    }

    if !is_loopback_request && !has_valid_pairing_token {
        return Some(SseAuthRejection::NonLocalWithoutAuthLayer);
    }

    None
}

/// GET /api/events — SSE event stream
pub async fn handle_sse_events(
    State(state): State<AppState>,
    ConnectInfo(peer_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .unwrap_or("")
        .trim();
    let has_valid_pairing_token = !token.is_empty() && state.pairing.is_authenticated(token);
    let is_loopback_request =
        super::is_loopback_request(Some(peer_addr), &headers, state.trust_forwarded_headers);

    match evaluate_sse_auth(
        state.pairing.require_pairing(),
        is_loopback_request,
        has_valid_pairing_token,
    ) {
        Some(SseAuthRejection::MissingPairingToken) => {
            return (
                StatusCode::UNAUTHORIZED,
                "Unauthorized — provide Authorization: Bearer <token>",
            )
                .into_response();
        }
        Some(SseAuthRejection::NonLocalWithoutAuthLayer) => {
            return (
                StatusCode::UNAUTHORIZED,
                "Unauthorized — enable gateway pairing or provide a valid paired bearer token for non-local /api/events access",
            )
                .into_response();
        }
        None => {}
    }

    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(
        |result: Result<
            serde_json::Value,
            tokio_stream::wrappers::errors::BroadcastStreamRecvError,
        >| {
            match result {
                Ok(value) => Some(Ok::<_, Infallible>(
                    Event::default().data(value.to_string()),
                )),
                Err(_) => None, // Skip lagged messages
            }
        },
    );

    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

/// Broadcast observer that forwards events to the SSE broadcast channel.
pub struct BroadcastObserver {
    inner: Box<dyn crate::observability::Observer>,
    tx: tokio::sync::broadcast::Sender<serde_json::Value>,
}

impl BroadcastObserver {
    pub fn new(
        inner: Box<dyn crate::observability::Observer>,
        tx: tokio::sync::broadcast::Sender<serde_json::Value>,
    ) -> Self {
        Self { inner, tx }
    }
}

impl crate::observability::Observer for BroadcastObserver {
    fn record_event(&self, event: &crate::observability::ObserverEvent) {
        // Forward to inner observer
        self.inner.record_event(event);

        // Broadcast to SSE subscribers
        let json = match event {
            crate::observability::ObserverEvent::LlmRequest {
                provider, model, ..
            } => serde_json::json!({
                "type": "llm_request",
                "provider": provider,
                "model": model,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
            crate::observability::ObserverEvent::ToolCall {
                tool,
                duration,
                success,
            } => serde_json::json!({
                "type": "tool_call",
                "tool": tool,
                "duration_ms": duration.as_millis(),
                "success": success,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
            crate::observability::ObserverEvent::ToolCallStart { tool } => serde_json::json!({
                "type": "tool_call_start",
                "tool": tool,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
            crate::observability::ObserverEvent::Error { component, message } => {
                serde_json::json!({
                    "type": "error",
                    "component": component,
                    "message": message,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })
            }
            crate::observability::ObserverEvent::AgentStart { provider, model } => {
                serde_json::json!({
                    "type": "agent_start",
                    "provider": provider,
                    "model": model,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })
            }
            crate::observability::ObserverEvent::AgentEnd {
                provider,
                model,
                duration,
                tokens_used,
                cost_usd,
            } => serde_json::json!({
                "type": "agent_end",
                "provider": provider,
                "model": model,
                "duration_ms": duration.as_millis(),
                "tokens_used": tokens_used,
                "cost_usd": cost_usd,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
            _ => return, // Skip events we don't broadcast
        };

        let _ = self.tx.send(json);
    }

    fn record_metric(&self, metric: &crate::observability::traits::ObserverMetric) {
        self.inner.record_metric(metric);
    }

    fn flush(&self) {
        self.inner.flush();
    }

    fn name(&self) -> &str {
        "broadcast"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::traits::{ObserverEvent, ObserverMetric};
    use crate::observability::Observer;
    use std::time::Duration;

    #[test]
    fn evaluate_sse_auth_requires_pairing_token_when_pairing_is_enabled() {
        assert_eq!(
            evaluate_sse_auth(true, true, false),
            Some(SseAuthRejection::MissingPairingToken)
        );
        assert_eq!(evaluate_sse_auth(true, false, true), None);
    }

    #[test]
    fn evaluate_sse_auth_rejects_public_without_auth_layer_when_pairing_disabled() {
        assert_eq!(
            evaluate_sse_auth(false, false, false),
            Some(SseAuthRejection::NonLocalWithoutAuthLayer)
        );
    }

    #[test]
    fn evaluate_sse_auth_allows_loopback_or_valid_token_when_pairing_disabled() {
        assert_eq!(evaluate_sse_auth(false, true, false), None);
        assert_eq!(evaluate_sse_auth(false, false, true), None);
    }

    /// No-op observer for testing BroadcastObserver inner delegation.
    struct NoopObserver;

    impl Observer for NoopObserver {
        fn record_event(&self, _event: &ObserverEvent) {}
        fn record_metric(&self, _metric: &ObserverMetric) {}
        fn flush(&self) {}
        fn name(&self) -> &str {
            "noop"
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    /// REQ-GW-007-SC01
    #[test]
    fn broadcast_observer_name() {
        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let observer = BroadcastObserver::new(Box::new(NoopObserver), tx);
        assert_eq!(observer.name(), "broadcast");
    }

    /// REQ-GW-007-SC02
    #[test]
    fn broadcast_observer_broadcasts_tool_call_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let observer = BroadcastObserver::new(Box::new(NoopObserver), tx);

        observer.record_event(&ObserverEvent::ToolCall {
            tool: "shell".to_string(),
            duration: Duration::from_millis(50),
            success: true,
        });

        let event = rx.try_recv().unwrap();
        assert_eq!(event["type"], "tool_call");
        assert_eq!(event["tool"], "shell");
        assert_eq!(event["success"], true);
        assert!(event["timestamp"].is_string());
    }

    /// REQ-GW-007-SC03
    #[test]
    fn broadcast_observer_broadcasts_llm_request_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let observer = BroadcastObserver::new(Box::new(NoopObserver), tx);

        observer.record_event(&ObserverEvent::LlmRequest {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            messages_count: 3,
        });

        let event = rx.try_recv().unwrap();
        assert_eq!(event["type"], "llm_request");
        assert_eq!(event["provider"], "openai");
        assert_eq!(event["model"], "gpt-4");
    }

    /// REQ-GW-007-SC04
    #[test]
    fn broadcast_observer_broadcasts_agent_start_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let observer = BroadcastObserver::new(Box::new(NoopObserver), tx);

        observer.record_event(&ObserverEvent::AgentStart {
            provider: "anthropic".to_string(),
            model: "claude-3".to_string(),
        });

        let event = rx.try_recv().unwrap();
        assert_eq!(event["type"], "agent_start");
        assert_eq!(event["provider"], "anthropic");
    }

    /// REQ-GW-007-SC05
    #[test]
    fn broadcast_observer_broadcasts_error_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let observer = BroadcastObserver::new(Box::new(NoopObserver), tx);

        observer.record_event(&ObserverEvent::Error {
            component: "gateway".to_string(),
            message: "connection failed".to_string(),
        });

        let event = rx.try_recv().unwrap();
        assert_eq!(event["type"], "error");
        assert_eq!(event["component"], "gateway");
        assert_eq!(event["message"], "connection failed");
    }

    /// REQ-GW-007-SC06
    #[test]
    fn broadcast_observer_skips_non_broadcast_events() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let observer = BroadcastObserver::new(Box::new(NoopObserver), tx);

        // LlmResponse is not handled by BroadcastObserver (falls to _ => return)
        observer.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            duration: Duration::from_millis(100),
            success: true,
            error_message: None,
            input_tokens: Some(10),
            output_tokens: Some(20),
        });

        // No event should have been broadcast
        assert!(rx.try_recv().is_err());
    }
}
