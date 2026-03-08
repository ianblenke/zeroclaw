//! WebSocket agent chat handler.
//!
//! Protocol:
//! ```text
//! Client -> Server: {"type":"message","content":"Hello"}
//! Server -> Client: {"type":"chunk","content":"Hi! "}
//! Server -> Client: {"type":"tool_call","name":"shell","args":{...}}
//! Server -> Client: {"type":"tool_result","name":"shell","output":"..."}
//! Server -> Client: {"type":"done","full_response":"..."}
//! ```

use super::AppState;
use crate::agent::loop_::{build_shell_policy_instructions, build_tool_instructions_from_specs};
use crate::memory::MemoryCategory;
use crate::providers::ChatMessage;
use crate::tools::traits::Tool;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, RawQuery, State, WebSocketUpgrade,
    },
    http::{header, HeaderMap},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use uuid::Uuid;

const EMPTY_WS_RESPONSE_FALLBACK: &str =
    "Tool execution completed, but the model returned no final text response. Please ask me to summarize the result.";
const WS_HISTORY_MEMORY_KEY_PREFIX: &str = "gateway_ws_history";
const MAX_WS_PERSISTED_TURNS: usize = 128;
const MAX_WS_SESSION_ID_LEN: usize = 128;

#[derive(Debug, Default, PartialEq, Eq)]
struct WsQueryParams {
    token: Option<String>,
    session_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct WsHistoryTurn {
    role: String,
    content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
struct WsPersistedHistory {
    version: u8,
    messages: Vec<WsHistoryTurn>,
}

fn normalize_ws_session_id(candidate: Option<&str>) -> Option<String> {
    let raw = candidate?.trim();
    if raw.is_empty() || raw.len() > MAX_WS_SESSION_ID_LEN {
        return None;
    }

    if raw
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Some(raw.to_string());
    }

    None
}

fn parse_ws_query_params(raw_query: Option<&str>) -> WsQueryParams {
    let Some(query) = raw_query else {
        return WsQueryParams::default();
    };

    let mut params = WsQueryParams::default();
    for kv in query.split('&') {
        let mut parts = kv.splitn(2, '=');
        let key = parts.next().unwrap_or("").trim();
        let value = parts.next().unwrap_or("").trim();
        if value.is_empty() {
            continue;
        }

        match key {
            "token" if params.token.is_none() => {
                params.token = Some(value.to_string());
            }
            "session_id" if params.session_id.is_none() => {
                params.session_id = normalize_ws_session_id(Some(value));
            }
            _ => {}
        }
    }

    params
}

fn ws_history_memory_key(session_id: &str) -> String {
    format!("{WS_HISTORY_MEMORY_KEY_PREFIX}:{session_id}")
}

fn ws_history_turns_from_chat(history: &[ChatMessage]) -> Vec<WsHistoryTurn> {
    let mut turns = history
        .iter()
        .filter_map(|msg| match msg.role.as_str() {
            "user" | "assistant" => {
                let content = msg.content.trim();
                if content.is_empty() {
                    None
                } else {
                    Some(WsHistoryTurn {
                        role: msg.role.clone(),
                        content: content.to_string(),
                    })
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    if turns.len() > MAX_WS_PERSISTED_TURNS {
        let keep_from = turns.len().saturating_sub(MAX_WS_PERSISTED_TURNS);
        turns.drain(0..keep_from);
    }
    turns
}

fn restore_chat_history(system_prompt: &str, turns: &[WsHistoryTurn]) -> Vec<ChatMessage> {
    let mut history = vec![ChatMessage::system(system_prompt)];
    for turn in turns {
        match turn.role.as_str() {
            "user" => history.push(ChatMessage::user(&turn.content)),
            "assistant" => history.push(ChatMessage::assistant(&turn.content)),
            _ => {}
        }
    }
    history
}

async fn load_ws_history(
    state: &AppState,
    session_id: &str,
    system_prompt: &str,
) -> Vec<ChatMessage> {
    let key = ws_history_memory_key(session_id);
    let Some(entry) = state.mem.get(&key).await.ok().flatten() else {
        return vec![ChatMessage::system(system_prompt)];
    };

    let parsed = serde_json::from_str::<WsPersistedHistory>(&entry.content)
        .map(|history| history.messages)
        .or_else(|_| serde_json::from_str::<Vec<WsHistoryTurn>>(&entry.content));

    match parsed {
        Ok(turns) => restore_chat_history(system_prompt, &turns),
        Err(err) => {
            tracing::warn!(
                "Failed to parse persisted websocket history for session {}: {}",
                session_id,
                err
            );
            vec![ChatMessage::system(system_prompt)]
        }
    }
}

async fn persist_ws_history(state: &AppState, session_id: &str, history: &[ChatMessage]) {
    let payload = WsPersistedHistory {
        version: 1,
        messages: ws_history_turns_from_chat(history),
    };
    let serialized = match serde_json::to_string(&payload) {
        Ok(value) => value,
        Err(err) => {
            tracing::warn!(
                "Failed to serialize websocket history for session {}: {}",
                session_id,
                err
            );
            return;
        }
    };

    let key = ws_history_memory_key(session_id);
    if let Err(err) = state
        .mem
        .store(
            &key,
            &serialized,
            MemoryCategory::Conversation,
            Some(session_id),
        )
        .await
    {
        tracing::warn!(
            "Failed to persist websocket history for session {}: {}",
            session_id,
            err
        );
    }
}

fn sanitize_ws_response(
    response: &str,
    tools: &[Box<dyn crate::tools::Tool>],
    leak_guard: &crate::config::OutboundLeakGuardConfig,
) -> String {
    match crate::channels::sanitize_channel_response(response, tools, leak_guard) {
        crate::channels::ChannelSanitizationResult::Sanitized(sanitized) => {
            if sanitized.is_empty() && !response.trim().is_empty() {
                "I encountered malformed tool-call output and could not produce a safe reply. Please try again."
                    .to_string()
            } else {
                sanitized
            }
        }
        crate::channels::ChannelSanitizationResult::Blocked { .. } => {
            "I blocked a draft response because it appeared to contain credential material. Please ask for a redacted summary."
                .to_string()
        }
    }
}

fn normalize_prompt_tool_results(content: &str) -> Option<String> {
    let mut cleaned_lines: Vec<&str> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("<tool_result") || trimmed == "</tool_result>" {
            continue;
        }
        cleaned_lines.push(line.trim_end());
    }

    if cleaned_lines.is_empty() {
        None
    } else {
        Some(cleaned_lines.join("\n"))
    }
}

fn extract_latest_tool_output(history: &[ChatMessage]) -> Option<String> {
    for msg in history.iter().rev() {
        match msg.role.as_str() {
            "tool" => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&msg.content) {
                    if let Some(content) = value
                        .get("content")
                        .and_then(|v| v.as_str())
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                    {
                        return Some(content.to_string());
                    }
                }

                let trimmed = msg.content.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
            "user" => {
                if let Some(payload) = msg.content.strip_prefix("[Tool results]") {
                    let payload = payload.trim_start_matches('\n');
                    if let Some(cleaned) = normalize_prompt_tool_results(payload) {
                        return Some(cleaned);
                    }
                }
            }
            _ => {}
        }
    }

    None
}

fn finalize_ws_response(
    response: &str,
    history: &[ChatMessage],
    tools: &[Box<dyn crate::tools::Tool>],
    leak_guard: &crate::config::OutboundLeakGuardConfig,
) -> String {
    let sanitized = sanitize_ws_response(response, tools, leak_guard);
    let base = if !sanitized.trim().is_empty() {
        sanitized
    } else if let Some(tool_output) = extract_latest_tool_output(history) {
        let excerpt = crate::util::truncate_with_ellipsis(tool_output.trim(), 1200);
        format!(
            "Tool execution completed, but the model returned no final text response.\n\nLatest tool output:\n{excerpt}"
        )
    } else {
        EMPTY_WS_RESPONSE_FALLBACK.to_string()
    };

    // Strip LLM-fabricated [IMAGE:...] markers that don't contain real base64 data URIs.
    // Real markers look like [IMAGE:data:image/jpeg;base64,/9j/...] and come from tool results.
    // Fake ones look like [IMAGE: Front door area showing...] — just descriptions.
    strip_fake_image_markers(&base)
}

/// Remove `[IMAGE:...]` markers that don't contain valid `data:image/` base64 URIs.
fn strip_fake_image_markers(text: &str) -> String {
    if !text.contains("[IMAGE:") {
        return text.to_string();
    }
    let mut result = String::with_capacity(text.len());
    let mut search_from = 0;
    while let Some(start) = text[search_from..].find("[IMAGE:") {
        let abs_start = search_from + start;
        result.push_str(&text[search_from..abs_start]);
        if let Some(end) = text[abs_start..].find(']') {
            let marker = &text[abs_start..abs_start + end + 1];
            // Keep only real markers with data:image/ base64 URIs
            if marker.contains("data:image/") && marker.contains(";base64,") {
                result.push_str(marker);
            }
            // else: fake marker — skip it
            search_from = abs_start + end + 1;
        } else {
            // No closing bracket — keep remaining text as-is
            result.push_str(&text[abs_start..]);
            search_from = text.len();
        }
    }
    result.push_str(&text[search_from..]);
    // Clean up extra whitespace left by removed markers
    let cleaned = result.replace("  ", " ").trim().to_string();
    cleaned
}

/// Remove all `[VIDEO_EMBED:...]` markers from text.
/// Used to strip model-echoed markers that may be malformed/partial before
/// appending guaranteed well-formed markers collected server-side from tool results.
fn strip_video_embed_markers(text: &str) -> String {
    if !text.contains("[VIDEO_EMBED:") {
        return text.to_string();
    }
    let mut result = String::with_capacity(text.len());
    let mut search_from = 0;
    while let Some(start) = text[search_from..].find("[VIDEO_EMBED:") {
        let abs_start = search_from + start;
        result.push_str(&text[search_from..abs_start]);
        if let Some(end) = text[abs_start..].find(']') {
            search_from = abs_start + end + 1;
        } else {
            // No closing bracket — discard the rest of the partial marker
            search_from = text.len();
        }
    }
    result.push_str(&text[search_from..]);
    result.replace("  ", " ").trim().to_string()
}

fn build_ws_system_prompt(
    config: &crate::config::Config,
    model: &str,
    tools_registry: &[Box<dyn crate::tools::Tool>],
    native_tools: bool,
) -> String {
    let mut tool_specs: Vec<crate::tools::ToolSpec> =
        tools_registry.iter().map(|tool| tool.spec()).collect();
    tool_specs.sort_by(|a, b| a.name.cmp(&b.name));

    // Tool search filtering: when active, only show core tools + tool_search in prompt
    let tool_search_active = match config.tool_search.mode.as_str() {
        "enabled" => true,
        "auto" => tool_specs.len() > config.tool_search.threshold,
        _ => false,
    };

    if tool_search_active {
        let core_names: std::collections::HashSet<&str> = config
            .tool_search
            .always_loaded_tools
            .iter()
            .map(|s| s.as_str())
            .collect();
        tool_specs.retain(|s| core_names.contains(s.name.as_str()));
        let search_tool = crate::tools::ToolSearchTool::new(Vec::new(), 0);
        tool_specs.push(search_tool.spec());
    }

    let tool_descs: Vec<(&str, &str)> = tool_specs
        .iter()
        .map(|spec| (spec.name.as_str(), spec.description.as_str()))
        .collect();

    let bootstrap_max_chars = if config.agent.compact_context {
        Some(6000)
    } else {
        None
    };

    let mut prompt = crate::channels::build_system_prompt_with_mode(
        &config.workspace_dir,
        model,
        &tool_descs,
        &[],
        Some(&config.identity),
        bootstrap_max_chars,
        native_tools,
        config.skills.prompt_injection_mode,
    );
    if !native_tools {
        prompt.push_str(&build_tool_instructions_from_specs(&tool_specs));
    }
    prompt.push_str(&build_shell_policy_instructions(&config.autonomy));

    prompt
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WsAuthRejection {
    MissingPairingToken,
    NonLocalWithoutAuthLayer,
}

fn evaluate_ws_auth(
    pairing_required: bool,
    is_loopback_request: bool,
    has_valid_pairing_token: bool,
) -> Option<WsAuthRejection> {
    if pairing_required {
        return (!has_valid_pairing_token).then_some(WsAuthRejection::MissingPairingToken);
    }

    if !is_loopback_request && !has_valid_pairing_token {
        return Some(WsAuthRejection::NonLocalWithoutAuthLayer);
    }

    None
}

/// GET /ws/chat — WebSocket upgrade for agent chat
pub async fn handle_ws_chat(
    State(state): State<AppState>,
    ConnectInfo(peer_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let query_params = parse_ws_query_params(query.as_deref());
    let token =
        extract_ws_bearer_token(&headers, query_params.token.as_deref()).unwrap_or_default();
    let has_valid_pairing_token = !token.is_empty() && state.pairing.is_authenticated(&token);
    let is_loopback_request =
        super::is_loopback_request(Some(peer_addr), &headers, state.trust_forwarded_headers);

    match evaluate_ws_auth(
        state.pairing.require_pairing(),
        is_loopback_request,
        has_valid_pairing_token,
    ) {
        Some(WsAuthRejection::MissingPairingToken) => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "Unauthorized — provide Authorization: Bearer <token>, Sec-WebSocket-Protocol: bearer.<token>, or ?token=<token>",
            )
                .into_response();
        }
        Some(WsAuthRejection::NonLocalWithoutAuthLayer) => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "Unauthorized — enable gateway pairing or provide a valid paired bearer token for non-local /ws/chat access",
            )
                .into_response();
        }
        None => {}
    }

    let session_id = query_params
        .session_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    ws.on_upgrade(move |socket| handle_socket(socket, state, session_id))
        .into_response()
}

async fn handle_socket(socket: WebSocket, state: AppState, session_id: String) {
    let ws_session_id = format!("ws_{}", Uuid::new_v4());

    // Build system prompt once for the session
    let system_prompt = {
        let config_guard = state.config.lock();
        let mut prompt = build_ws_system_prompt(
            &config_guard,
            &state.model,
            state.tools_registry_exec.as_ref(),
            state.provider.supports_native_tools(),
        );
        prompt.push_str(&format!(
            "\n\n## Session Context\nThis voice session's ID is `{session_id}`. \
             Use this when calling tools that require a session_id parameter (e.g. nats_subscribe)."
        ));
        prompt
    };

    // Restore persisted history (if any) and replay to the client before processing new input.
    let mut history = load_ws_history(&state, &session_id, &system_prompt).await;
    let persisted_turns = ws_history_turns_from_chat(&history);
    let history_payload = serde_json::json!({
        "type": "history",
        "session_id": session_id.as_str(),
        "messages": persisted_turns,
    });

    // Split socket into reader/writer halves for bidirectional communication:
    // - Writer sends responses + receives broadcast events (proactive push)
    // - Reader receives client messages
    let (mut ws_writer, mut ws_reader) = socket.split();
    let mut broadcast_rx = state.event_tx.subscribe();

    let _ = ws_writer
        .send(Message::Text(history_payload.to_string().into()))
        .await;

    loop {
        tokio::select! {
            // ── Client sent a message ──
            client_msg = ws_reader.next() => {
                let text = match client_msg {
                    Some(Ok(Message::Text(text))) => text,
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    _ => continue,
                };

                // Parse incoming message
                let parsed: serde_json::Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(_) => {
                        let err = serde_json::json!({"type": "error", "message": "Invalid JSON"});
                        let _ = ws_writer.send(Message::Text(err.to_string().into())).await;
                        continue;
                    }
                };

                let msg_type = parsed["type"].as_str().unwrap_or("");
                let is_audio = msg_type == "audio";
                let content = match msg_type {
                    "message" => {
                        let c = parsed["content"].as_str().unwrap_or("").to_string();
                        if c.is_empty() { continue; }
                        c
                    }
                    "audio" => {
                        // Audio-native: embed as marker for Ultravox/multimodal provider
                        let data = parsed["data"].as_str().unwrap_or("");
                        let fmt = parsed["format"].as_str().unwrap_or("wav");
                        if data.is_empty() { continue; }
                        // Prepend speaker context if provided
                        let speaker = parsed["speaker"].as_str().unwrap_or("");
                        let audio_marker = format!("[AUDIO:data:audio/{fmt};base64,{data}]");
                        if speaker.is_empty() {
                            audio_marker
                        } else {
                            format!("[Speaker: {speaker}] {audio_marker}")
                        }
                    }
                    _ => continue,
                };
                // Skip perplexity filter for audio (base64 is not adversarial text)
                if !is_audio {
                    let perplexity_cfg = { state.config.lock().security.perplexity_filter.clone() };
                    if let Some(assessment) =
                        crate::security::detect_adversarial_suffix(&content, &perplexity_cfg)
                    {
                        let err = serde_json::json!({
                            "type": "error",
                            "message": format!(
                                "Input blocked by security.perplexity_filter: perplexity={:.2} (threshold {:.2}), symbol_ratio={:.2} (threshold {:.2}), suspicious_tokens={}.",
                                assessment.perplexity,
                                perplexity_cfg.perplexity_threshold,
                                assessment.symbol_ratio,
                                perplexity_cfg.symbol_ratio_threshold,
                                assessment.suspicious_token_count
                            ),
                        });
                        let _ = ws_writer.send(Message::Text(err.to_string().into())).await;
                        continue;
                    }
                }

                // Add user message to history
                history.push(ChatMessage::user(&content));
                persist_ws_history(&state, &session_id, &history).await;

                // Get provider info
                let provider_label = state
                    .config
                    .lock()
                    .default_provider
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());

                // Broadcast agent_start event
                let _ = state.event_tx.send(serde_json::json!({
                    "type": "agent_start",
                    "provider": provider_label,
                    "model": state.model,
                }));

                // Full agentic loop with tools — streams text chunks to client
                // as the final response is generated for early TTS start.
                let (delta_tx, mut delta_rx) = tokio::sync::mpsc::channel::<String>(64);
                let state_for_task = state.clone();
                let content_for_task = content.clone();
                let ws_session_for_task = ws_session_id.clone();

                let agent_handle = tokio::spawn(async move {
                    super::run_gateway_streaming_chat_with_tools(
                        &state_for_task,
                        &content_for_task,
                        Some(&ws_session_for_task),
                        delta_tx,
                    )
                    .await
                });

                // Stream word-level deltas directly to the client as they arrive.
                // Also listen for broadcast events so proactive messages arrive
                // even during agent response streaming.
                let mut chunks_sent = false;
                let mut in_final_response = false;
                let mut collected_images: Vec<String> = Vec::new();
                let mut collected_videos: Vec<String> = Vec::new();

                loop {
                    tokio::select! {
                        delta = delta_rx.recv() => {
                            match delta {
                                Some(d) => {
                                    if d == crate::agent::loop_::DRAFT_CLEAR_SENTINEL {
                                        in_final_response = true;
                                    } else if d.starts_with(crate::agent::loop_::IMAGE_DATA_SENTINEL) {
                                        // Collect image markers from tool results for inline rendering
                                        collected_images.push(d[crate::agent::loop_::IMAGE_DATA_SENTINEL.len()..].to_string());
                                    } else if d.starts_with(crate::agent::loop_::VIDEO_EMBED_SENTINEL) {
                                        // Collect video embed markers from tool results for inline rendering
                                        collected_videos.push(d[crate::agent::loop_::VIDEO_EMBED_SENTINEL.len()..].to_string());
                                    } else if d.starts_with(crate::agent::loop_::REASONING_CONTENT_SENTINEL) {
                                        // Forward reasoning/thinking content as a separate message
                                        let content = &d[crate::agent::loop_::REASONING_CONTENT_SENTINEL.len()..];
                                        let thinking_msg = serde_json::json!({
                                            "type": "thinking",
                                            "content": content,
                                        });
                                        let _ = ws_writer
                                            .send(Message::Text(thinking_msg.to_string().into()))
                                            .await;
                                    } else if d.starts_with(crate::agent::loop_::DRAFT_PROGRESS_SENTINEL) {
                                        // Forward progress updates as thinking messages
                                        let content = &d[crate::agent::loop_::DRAFT_PROGRESS_SENTINEL.len()..];
                                        let thinking_msg = serde_json::json!({
                                            "type": "thinking",
                                            "content": content,
                                        });
                                        let _ = ws_writer
                                            .send(Message::Text(thinking_msg.to_string().into()))
                                            .await;
                                    } else if d.starts_with(crate::agent::loop_::DRAFT_PROGRESS_BLOCK_SENTINEL) {
                                        // Forward progress block updates as thinking messages
                                        let content = &d[crate::agent::loop_::DRAFT_PROGRESS_BLOCK_SENTINEL.len()..];
                                        let thinking_msg = serde_json::json!({
                                            "type": "thinking",
                                            "content": content,
                                        });
                                        let _ = ws_writer
                                            .send(Message::Text(thinking_msg.to_string().into()))
                                            .await;
                                    } else if in_final_response {
                                        let chunk_msg = serde_json::json!({
                                            "type": "chunk",
                                            "content": d,
                                        });
                                        let _ = ws_writer
                                            .send(Message::Text(chunk_msg.to_string().into()))
                                            .await;
                                        chunks_sent = true;
                                    }
                                }
                                None => break, // Channel closed — agent task done
                            }
                        }
                        broadcast = broadcast_rx.recv() => {
                            if let Ok(event) = broadcast {
                                if event.get("type").and_then(|t| t.as_str()) == Some("proactive") {
                                    let _ = ws_writer.send(Message::Text(event.to_string().into())).await;
                                }
                            }
                        }
                    }
                }

                // Channel closed — await the agent task result.
                match agent_handle.await {
                    Ok(Ok(response)) => {
                        let leak_guard_cfg =
                            { state.config.lock().security.outbound_leak_guard.clone() };
                        let mut safe_response = finalize_ws_response(
                            &response,
                            &history,
                            state.tools_registry_exec.as_ref(),
                            &leak_guard_cfg,
                        );

                        // Append any images collected from tool results
                        if !collected_images.is_empty() {
                            for img in &collected_images {
                                safe_response.push('\n');
                                safe_response.push_str(img);
                            }
                        }

                        // Strip model-echoed [VIDEO_EMBED:...] markers from the
                        // response text — they may be malformed or partial.  The
                        // server-collected markers below are guaranteed well-formed.
                        if !collected_videos.is_empty() {
                            safe_response = strip_video_embed_markers(&safe_response);
                            for vid in &collected_videos {
                                safe_response.push('\n');
                                safe_response.push_str(vid);
                            }
                        }

                        // If nothing was streamed, send the full response as a chunk.
                        if !chunks_sent {
                            let chunk_msg = serde_json::json!({
                                "type": "chunk",
                                "content": safe_response,
                            });
                            let _ = ws_writer
                                .send(Message::Text(chunk_msg.to_string().into()))
                                .await;
                        }

                        history.push(ChatMessage::assistant(&safe_response));
                        persist_ws_history(&state, &session_id, &history).await;

                        let done = serde_json::json!({
                            "type": "done",
                            "full_response": safe_response,
                        });
                        let _ = ws_writer
                            .send(Message::Text(done.to_string().into()))
                            .await;

                        let _ = state.event_tx.send(serde_json::json!({
                            "type": "agent_end",
                            "provider": provider_label,
                            "model": state.model,
                        }));
                    }
                    Ok(Err(e)) => {
                        let sanitized = crate::providers::sanitize_api_error(&e.to_string());
                        let err = serde_json::json!({
                            "type": "error",
                            "message": sanitized,
                        });
                        let _ = ws_writer
                            .send(Message::Text(err.to_string().into()))
                            .await;

                        let _ = state.event_tx.send(serde_json::json!({
                            "type": "error",
                            "component": "ws_chat",
                            "message": sanitized,
                        }));
                    }
                    Err(e) => {
                        let err = serde_json::json!({
                            "type": "error",
                            "message": format!("Agent task panicked: {e}"),
                        });
                        let _ = ws_writer
                            .send(Message::Text(err.to_string().into()))
                            .await;
                    }
                }
            }
            // ── Broadcast event (proactive push) while idle ──
            broadcast = broadcast_rx.recv() => {
                if let Ok(event) = broadcast {
                    if event.get("type").and_then(|t| t.as_str()) != Some("proactive") {
                        continue;
                    }

                    // Server-side target_session filtering (match internal UUID or user session ID)
                    let target = event.get("target_session").and_then(|t| t.as_str());
                    let is_for_me = target.is_none()
                        || target == Some(ws_session_id.as_str())
                        || target == Some(session_id.as_str());
                    if !is_for_me {
                        continue;
                    }

                    let content = event.get("content").and_then(|c| c.as_str()).unwrap_or("");
                    let has_audio = event.get("audio").and_then(|a| a.as_str()).map_or(false, |a| !a.is_empty());

                    if !content.is_empty() || has_audio {
                        // Send chime before proactive message
                        let chime = serde_json::json!({"type": "notification_chime"});
                        let _ = ws_writer.send(Message::Text(chime.to_string().into())).await;

                        // Forward pre-rendered event (text + optional audio)
                        let _ = ws_writer.send(Message::Text(event.to_string().into())).await;
                    }
                }
            }
        }
    }
}

fn extract_ws_bearer_token(headers: &HeaderMap, query_token: Option<&str>) -> Option<String> {
    if let Some(auth_header) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
    {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if !token.trim().is_empty() {
                return Some(token.trim().to_string());
            }
        }
    }

    if let Some(offered) = headers
        .get(header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|value| value.to_str().ok())
    {
        for protocol in offered.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            if let Some(token) = protocol.strip_prefix("bearer.") {
                if !token.trim().is_empty() {
                    return Some(token.trim().to_string());
                }
            }
        }
    }

    query_token
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
}

fn extract_query_token(raw_query: Option<&str>) -> Option<String> {
    parse_ws_query_params(raw_query).token
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Tool, ToolResult};
    use async_trait::async_trait;
    use axum::http::HeaderValue;

    #[test]
    fn extract_ws_bearer_token_prefers_authorization_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer from-auth-header"),
        );
        headers.insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static("zeroclaw.v1, bearer.from-protocol"),
        );

        assert_eq!(
            extract_ws_bearer_token(&headers, None).as_deref(),
            Some("from-auth-header")
        );
    }

    #[test]
    fn extract_ws_bearer_token_reads_websocket_protocol_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static("zeroclaw.v1, bearer.protocol-token"),
        );

        assert_eq!(
            extract_ws_bearer_token(&headers, None).as_deref(),
            Some("protocol-token")
        );
    }

    #[test]
    fn extract_ws_bearer_token_rejects_empty_tokens() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer    "),
        );
        headers.insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static("zeroclaw.v1, bearer."),
        );

        assert!(extract_ws_bearer_token(&headers, None).is_none());
    }

    #[test]
    fn extract_ws_bearer_token_reads_query_token_fallback() {
        let headers = HeaderMap::new();
        assert_eq!(
            extract_ws_bearer_token(&headers, Some("query-token")).as_deref(),
            Some("query-token")
        );
    }

    #[test]
    fn extract_ws_bearer_token_prefers_protocol_over_query_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static("zeroclaw.v1, bearer.protocol-token"),
        );

        assert_eq!(
            extract_ws_bearer_token(&headers, Some("query-token")).as_deref(),
            Some("protocol-token")
        );
    }

    #[test]
    fn extract_query_token_reads_token_param() {
        assert_eq!(
            extract_query_token(Some("foo=1&token=query-token&bar=2")).as_deref(),
            Some("query-token")
        );
        assert!(extract_query_token(Some("foo=1")).is_none());
    }

    #[test]
    fn parse_ws_query_params_reads_token_and_session_id() {
        let parsed = parse_ws_query_params(Some("foo=1&session_id=sess_123&token=query-token"));
        assert_eq!(parsed.token.as_deref(), Some("query-token"));
        assert_eq!(parsed.session_id.as_deref(), Some("sess_123"));
    }

    #[test]
    fn parse_ws_query_params_rejects_invalid_session_id() {
        let parsed = parse_ws_query_params(Some("session_id=../../etc/passwd"));
        assert!(parsed.session_id.is_none());
    }

    #[test]
    fn ws_history_turns_from_chat_skips_system_and_non_dialog_turns() {
        let history = vec![
            ChatMessage::system("sys"),
            ChatMessage::user(" hello "),
            ChatMessage {
                role: "tool".to_string(),
                content: "ignored".to_string(),
            },
            ChatMessage::assistant(" world "),
        ];

        let turns = ws_history_turns_from_chat(&history);
        assert_eq!(
            turns,
            vec![
                WsHistoryTurn {
                    role: "user".to_string(),
                    content: "hello".to_string()
                },
                WsHistoryTurn {
                    role: "assistant".to_string(),
                    content: "world".to_string()
                }
            ]
        );
    }

    #[test]
    fn restore_chat_history_applies_system_prompt_once() {
        let turns = vec![
            WsHistoryTurn {
                role: "user".to_string(),
                content: "u1".to_string(),
            },
            WsHistoryTurn {
                role: "assistant".to_string(),
                content: "a1".to_string(),
            },
        ];

        let restored = restore_chat_history("sys", &turns);
        assert_eq!(restored.len(), 3);
        assert_eq!(restored[0].role, "system");
        assert_eq!(restored[0].content, "sys");
        assert_eq!(restored[1].role, "user");
        assert_eq!(restored[1].content, "u1");
        assert_eq!(restored[2].role, "assistant");
        assert_eq!(restored[2].content, "a1");
    }

    #[test]
    fn evaluate_ws_auth_requires_pairing_token_when_pairing_is_enabled() {
        assert_eq!(
            evaluate_ws_auth(true, true, false),
            Some(WsAuthRejection::MissingPairingToken)
        );
        assert_eq!(evaluate_ws_auth(true, false, true), None);
    }

    #[test]
    fn evaluate_ws_auth_rejects_public_without_auth_layer_when_pairing_disabled() {
        assert_eq!(
            evaluate_ws_auth(false, false, false),
            Some(WsAuthRejection::NonLocalWithoutAuthLayer)
        );
    }

    #[test]
    fn evaluate_ws_auth_allows_loopback_or_valid_token_when_pairing_disabled() {
        assert_eq!(evaluate_ws_auth(false, true, false), None);
        assert_eq!(evaluate_ws_auth(false, false, true), None);
    }

    struct MockScheduleTool;

    #[async_trait]
    impl Tool for MockScheduleTool {
        fn name(&self) -> &str {
            "schedule"
        }

        fn description(&self) -> &str {
            "Mock schedule tool"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string" }
                }
            })
        }

        async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
            Ok(ToolResult {
                success: true,
                output: "ok".to_string(),
                error: None,
            })
        }
    }

    #[test]
    fn sanitize_ws_response_removes_tool_call_tags() {
        let input = r#"Before
<tool_call>
{"name":"schedule","arguments":{"action":"create"}}
</tool_call>
After"#;

        let leak_guard = crate::config::OutboundLeakGuardConfig::default();
        let result = sanitize_ws_response(input, &[], &leak_guard);
        let normalized = result
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(normalized, "Before\nAfter");
        assert!(!result.contains("<tool_call>"));
        assert!(!result.contains("\"name\":\"schedule\""));
    }

    #[test]
    fn sanitize_ws_response_removes_isolated_tool_json_artifacts() {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(MockScheduleTool)];
        let input = r#"{"name":"schedule","parameters":{"action":"create"}}
{"result":{"status":"scheduled"}}
Reminder set successfully."#;

        let leak_guard = crate::config::OutboundLeakGuardConfig::default();
        let result = sanitize_ws_response(input, &tools, &leak_guard);
        assert_eq!(result, "Reminder set successfully.");
        assert!(!result.contains("\"name\":\"schedule\""));
        assert!(!result.contains("\"result\""));
    }

    #[test]
    fn sanitize_ws_response_blocks_detected_credentials_when_configured() {
        let tools: Vec<Box<dyn Tool>> = Vec::new();
        let leak_guard = crate::config::OutboundLeakGuardConfig {
            enabled: true,
            action: crate::config::OutboundLeakGuardAction::Block,
            sensitivity: 0.7,
        };

        let result =
            sanitize_ws_response("Temporary key: AKIAABCDEFGHIJKLMNOP", &tools, &leak_guard);
        assert!(result.contains("blocked a draft response"));
    }

    #[test]
    fn build_ws_system_prompt_includes_tool_protocol_for_prompt_mode() {
        let config = crate::config::Config::default();
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(MockScheduleTool)];

        let prompt = build_ws_system_prompt(&config, "test-model", &tools, false);

        assert!(prompt.contains("## Tool Use Protocol"));
        assert!(prompt.contains("**schedule**"));
        assert!(prompt.contains("## Shell Policy"));
    }

    #[test]
    fn build_ws_system_prompt_omits_xml_protocol_for_native_mode() {
        let config = crate::config::Config::default();
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(MockScheduleTool)];

        let prompt = build_ws_system_prompt(&config, "test-model", &tools, true);

        assert!(!prompt.contains("## Tool Use Protocol"));
        assert!(prompt.contains("**schedule**"));
        assert!(prompt.contains("## Shell Policy"));
    }

    #[test]
    fn finalize_ws_response_uses_prompt_mode_tool_output_when_final_text_empty() {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(MockScheduleTool)];
        let history = vec![
            ChatMessage::system("sys"),
            ChatMessage::user(
                "[Tool results]\n<tool_result name=\"schedule\">\nDisk usage: 72%\n</tool_result>",
            ),
        ];

        let leak_guard = crate::config::OutboundLeakGuardConfig::default();
        let result = finalize_ws_response("", &history, &tools, &leak_guard);
        assert!(result.contains("Latest tool output:"));
        assert!(result.contains("Disk usage: 72%"));
        assert!(!result.contains("<tool_result"));
    }

    #[test]
    fn finalize_ws_response_uses_native_tool_message_output_when_final_text_empty() {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(MockScheduleTool)];
        let history = vec![ChatMessage {
            role: "tool".to_string(),
            content: r#"{"tool_call_id":"call_1","content":"Filesystem /dev/disk3s1: 210G free"}"#
                .to_string(),
        }];

        let leak_guard = crate::config::OutboundLeakGuardConfig::default();
        let result = finalize_ws_response("", &history, &tools, &leak_guard);
        assert!(result.contains("Latest tool output:"));
        assert!(result.contains("/dev/disk3s1"));
    }

    #[test]
    fn finalize_ws_response_uses_static_fallback_when_nothing_available() {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(MockScheduleTool)];
        let history = vec![ChatMessage::system("sys")];

        let leak_guard = crate::config::OutboundLeakGuardConfig::default();
        let result = finalize_ws_response("", &history, &tools, &leak_guard);
        assert_eq!(result, EMPTY_WS_RESPONSE_FALLBACK);
    }
}
