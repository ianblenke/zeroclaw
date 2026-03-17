//! Internal MCP HTTP server — exposes selected ZeroClaw tools via MCP protocol.
//!
//! This handler implements the MCP JSON-RPC 2.0 protocol at `POST /mcp`,
//! allowing external MCP clients (e.g. Claude Code CLI via `--mcp-config`)
//! to call ZeroClaw's internal tools: `bg_run`, `bg_status`, `tool_search`,
//! `memory_store`, `memory_recall`, `memory_forget`, `cron_list`, etc.
//!
//! The handler dispatches tool calls to the same `Tool` trait implementations
//! used by ZeroClaw's agent loop, with full access to in-process state
//! (BgJobStore, memory backend, cron scheduler, tool registry).

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use crate::tools::mcp_protocol::{
    JsonRpcError, JsonRpcResponse, INTERNAL_ERROR, INVALID_PARAMS, JSONRPC_VERSION,
    MCP_PROTOCOL_VERSION, METHOD_NOT_FOUND,
};
use crate::tools::Tool;

use super::AppState;

/// Tools exposed via the internal MCP server.
/// Only a curated subset — not all 122 tools.
/// `tool_search` is handled specially (synthetic tool, not in registry).
const EXPOSED_TOOLS: &[&str] = &[
    "bg_run",
    "bg_status",
    "tool_search",
    "memory_store",
    "memory_recall",
    "memory_forget",
    "memory_observe",
    "cron_list",
    "cron_add",
    "cron_remove",
    "cron_run",
    // MCP tools exposed directly so Claude doesn't need tool_search for common ops
    "knowledge-store__remember",
    "knowledge-store__list_knowledge",
    "knowledge-store__search_knowledge",
    "knowledge-store__get_knowledge",
    "deep-research__start_deep_research",
    "deep-research__check_research_status",
    "deep-research__get_research_report",
    // Docmunch section-level retrieval (Discord content, research docs, etc.)
    "docmunch__search_sections",
    "docmunch__get_section",
    "docmunch__list_repos",
    "docmunch__get_toc",
    // Discord scraper
    "discord-scraper__scrape_now",
    "discord-scraper__scrape_status",
    "discord-scraper__update_interests",
    "discord-scraper__list_digests",
    "discord-scraper__get_digest",
    // NATS event subscriptions (camera alerts, HA events)
    "nats-bridge__nats_subscribe",
    "nats-bridge__nats_unsubscribe",
    "nats-bridge__nats_list_subscriptions",
    "nats-bridge__nats_list_subjects",
    // Special: notification history (handled inline, not from tool registry)
    "list_notifications",
    "get_notification",
];

/// Build a ToolSearchTool on the fly from the current tool registry.
fn build_tool_search(
    tools: &[Box<dyn crate::tools::Tool>],
) -> crate::tools::ToolSearchTool {
    let catalog: Vec<crate::tools::ToolSpec> = tools.iter().map(|t| t.spec()).collect();
    crate::tools::ToolSearchTool::new(catalog, 10)
}

fn mcp_response(
    id: Option<serde_json::Value>,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
) -> Json<serde_json::Value> {
    let resp = JsonRpcResponse {
        jsonrpc: JSONRPC_VERSION.to_string(),
        id,
        result,
        error,
    };
    Json(serde_json::to_value(resp).unwrap_or_default())
}

fn mcp_error(id: Option<serde_json::Value>, code: i32, message: &str) -> Json<serde_json::Value> {
    mcp_response(
        id,
        None,
        Some(JsonRpcError {
            code,
            message: message.to_string(),
            data: None,
        }),
    )
}

/// Handle MCP JSON-RPC 2.0 requests.
///
/// Supports: `initialize`, `notifications/initialized`, `tools/list`, `tools/call`.
pub async fn handle_mcp(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let method = body.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let req_id = body.get("id").cloned();
    let params = body.get("params").cloned().unwrap_or(serde_json::json!({}));

    // Notifications (no id) — acknowledge silently
    if req_id.is_none() {
        return Json(serde_json::json!({}));
    }

    match method {
        "initialize" => mcp_response(
            req_id,
            Some(serde_json::json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {"tools": {}},
                "serverInfo": {
                    "name": "zeroclaw",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            })),
            None,
        ),

        "tools/list" => {
            let mut tools: Vec<serde_json::Value> = state
                .tools_registry_exec
                .iter()
                .filter(|t| EXPOSED_TOOLS.contains(&t.name()) && t.name() != "tool_search")
                .map(|t| {
                    serde_json::json!({
                        "name": t.name(),
                        "description": t.description(),
                        "inputSchema": t.parameters_schema(),
                    })
                })
                .collect();

            // Add notification history tools (not in registry, handled inline)
            tools.push(serde_json::json!({
                "name": "list_notifications",
                "description": "List recent proactive notifications (Discord digests, bg_run completions, cron results). Returns IDs, timestamps, sources, and previews.",
                "inputSchema": {"type": "object", "properties": {"limit": {"type": "integer", "description": "Number of recent notifications (default: 10, max: 100)", "default": 10}}},
            }));
            tools.push(serde_json::json!({
                "name": "get_notification",
                "description": "Get the full content of a specific notification by ID. Use list_notifications first to see available IDs. Read it back to the user so they can hear it via TTS.",
                "inputSchema": {"type": "object", "properties": {"id": {"type": "integer", "description": "Notification ID from list_notifications"}}, "required": ["id"]},
            }));

            // Add tool_search (synthetic — not in the registry)
            let search = build_tool_search(&state.tools_registry_exec);
            tools.push(serde_json::json!({
                "name": search.name(),
                "description": search.description(),
                "inputSchema": search.parameters_schema(),
            }));

            mcp_response(req_id, Some(serde_json::json!({"tools": tools})), None)
        }

        "tools/call" => {
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            if tool_name.is_empty() {
                return mcp_error(req_id, INVALID_PARAMS, "tool name is required");
            }

            // Only allow exposed tools
            if !EXPOSED_TOOLS.contains(&tool_name) {
                return mcp_error(
                    req_id,
                    INVALID_PARAMS,
                    &format!("Unknown tool: {tool_name}"),
                );
            }

            // Handle notification history (accesses AppState directly)
            if tool_name == "list_notifications" {
                let history = state.notification_history.lock().await;
                let limit = arguments
                    .get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;
                let start = if history.len() > limit { history.len() - limit } else { 0 };
                let recent: Vec<_> = history[start..].iter().rev().enumerate().collect();

                if recent.is_empty() {
                    return mcp_response(
                        req_id,
                        Some(serde_json::json!({"content": [{"type": "text", "text": "No notifications yet."}]})),
                        None,
                    );
                }

                let mut lines = vec![format!("**Recent Notifications** ({} of {} total)\n", recent.len(), history.len())];
                for (i, notif) in &recent {
                    let ts = notif.get("timestamp").and_then(|t| t.as_str()).unwrap_or("?");
                    let source = notif.get("source").and_then(|s| s.as_str()).unwrap_or("?");
                    let content = notif.get("content").and_then(|c| c.as_str()).unwrap_or("");
                    let preview: String = content.chars().take(100).collect();
                    let idx = history.len() - i;
                    lines.push(format!("- **#{}** ({} UTC, source: {}): {}...", idx, &ts[..16.min(ts.len())], source, preview));
                }
                let text = lines.join("\n");
                return mcp_response(
                    req_id,
                    Some(serde_json::json!({"content": [{"type": "text", "text": text}]})),
                    None,
                );
            }

            if tool_name == "get_notification" {
                let idx = arguments
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let history = state.notification_history.lock().await;
                if idx == 0 || idx > history.len() {
                    return mcp_error(req_id, INVALID_PARAMS, &format!("Notification #{idx} not found"));
                }
                let notif = &history[idx - 1];
                let ts = notif.get("timestamp").and_then(|t| t.as_str()).unwrap_or("?");
                let source = notif.get("source").and_then(|s| s.as_str()).unwrap_or("?");
                let content = notif.get("content").and_then(|c| c.as_str()).unwrap_or("");
                let text = format!("**Notification #{}** ({} UTC, source: {})\n\n{}", idx, ts, source, content);
                return mcp_response(
                    req_id,
                    Some(serde_json::json!({"content": [{"type": "text", "text": text}]})),
                    None,
                );
            }

            // Handle tool_search specially (synthetic, not in registry)
            if tool_name == "tool_search" {
                let search = build_tool_search(&state.tools_registry_exec);
                return match search.execute(arguments).await {
                    Ok(result) => {
                        let text = if result.success {
                            result.output
                        } else {
                            format!("Error: {}", result.error.as_deref().unwrap_or("unknown"))
                        };
                        mcp_response(
                            req_id,
                            Some(serde_json::json!({"content": [{"type": "text", "text": text}]})),
                            None,
                        )
                    }
                    Err(e) => mcp_error(req_id, INTERNAL_ERROR, &format!("Tool error: {e}")),
                };
            }

            // Find the tool in the registry
            let tool = state
                .tools_registry_exec
                .iter()
                .find(|t| t.name() == tool_name);

            let tool = match tool {
                Some(t) => t,
                None => {
                    return mcp_error(
                        req_id,
                        INVALID_PARAMS,
                        &format!("Tool not found: {tool_name}"),
                    );
                }
            };

            // Execute the tool
            match tool.execute(arguments).await {
                Ok(result) => {
                    let text = if result.success {
                        result.output
                    } else {
                        format!(
                            "Error: {}",
                            result.error.as_deref().unwrap_or("unknown error")
                        )
                    };

                    mcp_response(
                        req_id,
                        Some(serde_json::json!({
                            "content": [{"type": "text", "text": text}]
                        })),
                        None,
                    )
                }
                Err(e) => mcp_error(req_id, INTERNAL_ERROR, &format!("Tool error: {e}")),
            }
        }

        _ => mcp_error(req_id, METHOD_NOT_FOUND, &format!("Method not found: {method}")),
    }
}
