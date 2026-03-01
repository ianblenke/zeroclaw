# Gateway Specification

## Purpose

Define requirements for the HTTP gateway server: routing, rate limiting, idempotency, SSE, WebSocket, OpenAI/OpenClaw compatibility, static file serving, and REST API handlers.

## Scope

- Files: 7 files in `src/gateway/` (~9,320 LOC total)
- Risk tier: HIGH (internet-facing HTTP server with auth, rate limiting, and webhook processing)
- Existing tests: 140 across 7 files
  - `src/gateway/mod.rs`: 75 tests
  - `src/gateway/ws.rs`: 21 tests
  - `src/gateway/openai_compat.rs`: 14 tests
  - `src/gateway/openclaw_compat.rs`: 11 tests
  - `src/gateway/sse.rs`: 9 tests
  - `src/gateway/api.rs`: 5 tests
  - `src/gateway/static_files.rs`: 5 tests

## Requirements

### REQ-GW-001: Gateway Server Orchestration

`run_gateway` MUST start the HTTP server with configured host/port, mount all routes, and apply rate limiting, auth middleware, body limits, and request timeouts.

#### Scenario: Body size limit enforcement

- WHEN a request body exceeds 64 KB
- THEN MUST reject the request
- Test: `security_body_limit_is_64kb` in `src/gateway/mod.rs`

#### Scenario: Request timeout enforcement

- WHEN a request exceeds the configured timeout
- THEN MUST terminate after 30 seconds
- Test: `security_timeout_is_30_seconds` in `src/gateway/mod.rs`

#### Scenario: App state cloneability

- WHEN the gateway state is shared across handlers
- THEN `AppState` MUST implement `Clone`
- Test: `app_state_is_clone` in `src/gateway/mod.rs`

#### Scenario: GET request to webhook endpoint

- WHEN a GET request is sent to the webhook endpoint
- THEN MUST return an explicit method hint (POST required)
- Test: `webhook_get_usage_returns_explicit_method_hint` in `src/gateway/mod.rs`

### REQ-GW-002: Rate Limiting

`GatewayRateLimiter` and `SlidingWindowRateLimiter` MUST enforce per-client rate limits on API and webhook requests with bounded cardinality, stale entry eviction, and concurrent safety.

#### Scenario: Rate limit blocks after threshold

- WHEN request rate exceeds configured limit
- THEN MUST deny further requests
- Test: `gateway_rate_limiter_blocks_after_limit` in `src/gateway/mod.rs`

#### Scenario: Stale entry eviction

- WHEN rate limiter entries become stale
- THEN MUST prune them during sweep
- Test: `rate_limiter_sweep_removes_stale_entries` in `src/gateway/mod.rs`

#### Scenario: Zero limit passthrough

- WHEN rate limit is configured to zero
- THEN MUST allow all requests (disabled mode)
- Test: `rate_limiter_zero_limit_always_allows` in `src/gateway/mod.rs`

#### Scenario: Bounded cardinality eviction

- WHEN unique client keys exceed max_keys limit
- THEN MUST evict oldest key to stay within bounds
- Test: `rate_limiter_bounded_cardinality_evicts_oldest_key` in `src/gateway/mod.rs`

#### Scenario: Window expiration recovery

- WHEN time window expires after rate limit is hit
- THEN MUST allow requests again
- Test: `rate_limiter_allows_after_window_expires` in `src/gateway/mod.rs`

#### Scenario: Independent key tracking

- WHEN multiple clients make requests
- THEN MUST track rate limits independently per client
- Test: `rate_limiter_independent_keys_tracked_separately` in `src/gateway/mod.rs`

#### Scenario: Exact boundary behavior at max keys

- WHEN key count is exactly at max_keys
- THEN MUST handle boundary correctly without data loss
- Test: `rate_limiter_exact_boundary_at_max_keys` in `src/gateway/mod.rs`

#### Scenario: Pair and webhook rate independence

- WHEN pairing and webhook requests arrive
- THEN MUST enforce separate rate limits for each category
- Test: `gateway_rate_limiter_pair_and_webhook_are_independent` in `src/gateway/mod.rs`

#### Scenario: Single key max capacity

- WHEN max_keys is 1
- THEN MUST allow exactly one tracked client
- Test: `rate_limiter_single_key_max_allows_one_request` in `src/gateway/mod.rs`

#### Scenario: Concurrent access safety

- WHEN multiple threads access the rate limiter simultaneously
- THEN MUST remain consistent without data races
- Test: `rate_limiter_concurrent_access_safe` in `src/gateway/mod.rs`

#### Scenario: Rapid burst then cooldown

- WHEN a client sends a rapid burst exceeding the limit then waits
- THEN MUST block during burst and allow after cooldown
- Test: `rate_limiter_rapid_burst_then_cooldown` in `src/gateway/mod.rs`

#### Scenario: Max keys normalization fallback

- WHEN max_keys is configured as zero
- THEN MUST use the fallback default value
- Test: `normalize_max_keys_uses_fallback_for_zero` in `src/gateway/mod.rs`

#### Scenario: Max keys normalization preserves nonzero

- WHEN max_keys is configured as a nonzero value
- THEN MUST preserve the configured value
- Test: `normalize_max_keys_preserves_nonzero_values` in `src/gateway/mod.rs`

### REQ-GW-003: Idempotency

`IdempotencyStore` MUST deduplicate webhook deliveries using idempotency keys with TTL expiration, bounded cardinality, and concurrent safety.

#### Scenario: Duplicate key rejection

- WHEN a duplicate idempotency key is submitted
- THEN MUST reject the duplicate and return cached status
- Test: `idempotency_store_rejects_duplicate_key` in `src/gateway/mod.rs`

#### Scenario: Bounded cardinality eviction

- WHEN stored keys exceed max_keys limit
- THEN MUST evict oldest key to stay within bounds
- Test: `idempotency_store_bounded_cardinality_evicts_oldest_key` in `src/gateway/mod.rs`

#### Scenario: Webhook-level idempotency skips duplicate provider calls

- WHEN a webhook with the same idempotency key is re-delivered
- THEN MUST skip the duplicate provider call
- Test: `webhook_idempotency_skips_duplicate_provider_calls` in `src/gateway/mod.rs`

#### Scenario: Different keys accepted

- WHEN distinct idempotency keys are submitted
- THEN MUST accept each independently
- Test: `idempotency_store_allows_different_keys` in `src/gateway/mod.rs`

#### Scenario: Max keys clamped to minimum of one

- WHEN max_keys is configured below 1
- THEN MUST clamp to a minimum of 1
- Test: `idempotency_store_max_keys_clamped_to_one` in `src/gateway/mod.rs`

#### Scenario: Rapid duplicate rejection

- WHEN the same key is submitted in rapid succession
- THEN MUST reject the second submission immediately
- Test: `idempotency_store_rapid_duplicate_rejected` in `src/gateway/mod.rs`

#### Scenario: TTL expiration re-accepts key

- WHEN the TTL expires for a previously recorded key
- THEN MUST accept the key again
- Test: `idempotency_store_accepts_after_ttl_expires` in `src/gateway/mod.rs`

#### Scenario: Eviction preserves newest entries

- WHEN eviction occurs due to cardinality overflow
- THEN MUST retain the newest entries and evict the oldest
- Test: `idempotency_store_eviction_preserves_newest` in `src/gateway/mod.rs`

#### Scenario: Concurrent access safety

- WHEN multiple threads access the idempotency store simultaneously
- THEN MUST remain consistent without data races
- Test: `idempotency_store_concurrent_access_safe` in `src/gateway/mod.rs`

### REQ-GW-004: Webhook Routing and Processing

Gateway MUST route incoming webhooks to the correct channel handler based on path and payload, enforce auth, validate message bodies, and persist memory keys.

#### Scenario: Webhook body requires message field

- WHEN a webhook body is missing the message field
- THEN MUST reject the request
- Test: `webhook_body_requires_message_field` in `src/gateway/mod.rs`

#### Scenario: Webhook rejects empty message

- WHEN a webhook body contains an empty message
- THEN MUST reject with an appropriate error
- Test: `webhook_rejects_empty_message` in `src/gateway/mod.rs`

#### Scenario: Webhook rejects public traffic without auth

- WHEN a webhook arrives from a public IP without auth layers
- THEN MUST reject the request
- Test: `webhook_rejects_public_traffic_without_auth_layers` in `src/gateway/mod.rs`

#### Scenario: Webhook streaming response format

- WHEN webhook streaming mode is active
- THEN MUST use SSE content type for response
- Test: `webhook_stream_response_uses_sse_content_type` in `src/gateway/mod.rs`

#### Scenario: Webhook autosave stores distinct memory keys

- WHEN multiple webhook requests arrive
- THEN MUST store distinct memory keys per request
- Test: `webhook_autosave_stores_distinct_keys_per_request` in `src/gateway/mod.rs`

#### Scenario: Webhook memory key uniqueness

- WHEN webhook memory keys are generated
- THEN MUST produce unique keys per invocation
- Test: `webhook_memory_key_is_unique` in `src/gateway/mod.rs`

#### Scenario: WhatsApp query fields optional

- WHEN WhatsApp webhook query parameters are missing
- THEN MUST accept the request with defaults
- Test: `whatsapp_query_fields_are_optional` in `src/gateway/mod.rs`

#### Scenario: WhatsApp memory key format

- WHEN a WhatsApp message arrives
- THEN memory key MUST include sender and message ID
- Test: `whatsapp_memory_key_includes_sender_and_message_id` in `src/gateway/mod.rs`

#### Scenario: QQ memory key format

- WHEN a QQ message arrives
- THEN memory key MUST include sender and message ID
- Test: `qq_memory_key_includes_sender_and_message_id` in `src/gateway/mod.rs`

### REQ-GW-005: WhatsApp Signature Verification

`verify_whatsapp_signature` MUST validate webhook signatures using HMAC-SHA256 with strict input handling.

#### Scenario: Valid signature accepted

- WHEN a webhook with valid HMAC-SHA256 signature arrives
- THEN MUST accept the webhook
- Test: `whatsapp_signature_valid` in `src/gateway/mod.rs`

#### Scenario: Wrong secret rejected

- WHEN signature is computed with a different secret
- THEN MUST reject the webhook
- Test: `whatsapp_signature_invalid_wrong_secret` in `src/gateway/mod.rs`

#### Scenario: Wrong body rejected

- WHEN signature does not match the request body
- THEN MUST reject the webhook
- Test: `whatsapp_signature_invalid_wrong_body` in `src/gateway/mod.rs`

#### Scenario: Missing prefix rejected

- WHEN signature header lacks the `sha256=` prefix
- THEN MUST reject the webhook
- Test: `whatsapp_signature_missing_prefix` in `src/gateway/mod.rs`

#### Scenario: Empty header rejected

- WHEN signature header is empty
- THEN MUST reject the webhook
- Test: `whatsapp_signature_empty_header` in `src/gateway/mod.rs`

#### Scenario: Invalid hex rejected

- WHEN signature contains invalid hexadecimal characters
- THEN MUST reject the webhook
- Test: `whatsapp_signature_invalid_hex` in `src/gateway/mod.rs`

#### Scenario: Empty body accepted with valid signature

- WHEN request body is empty but signature matches
- THEN MUST accept the webhook
- Test: `whatsapp_signature_empty_body` in `src/gateway/mod.rs`

#### Scenario: Unicode body handled correctly

- WHEN request body contains unicode characters
- THEN MUST compute and validate signature correctly
- Test: `whatsapp_signature_unicode_body` in `src/gateway/mod.rs`

#### Scenario: JSON payload handled correctly

- WHEN request body is a JSON payload
- THEN MUST compute and validate signature correctly
- Test: `whatsapp_signature_json_payload` in `src/gateway/mod.rs`

#### Scenario: Case-sensitive prefix enforcement

- WHEN signature prefix uses incorrect casing
- THEN MUST reject the webhook
- Test: `whatsapp_signature_case_sensitive_prefix` in `src/gateway/mod.rs`

#### Scenario: Truncated hex rejected

- WHEN signature hex is truncated
- THEN MUST reject the webhook
- Test: `whatsapp_signature_truncated_hex` in `src/gateway/mod.rs`

#### Scenario: Extra bytes rejected

- WHEN signature hex contains extra trailing bytes
- THEN MUST reject the webhook
- Test: `whatsapp_signature_extra_bytes` in `src/gateway/mod.rs`

### REQ-GW-005A: Webhook Secret Hash Verification

Gateway MUST verify webhook requests against a hashed shared secret header for non-WhatsApp webhooks.

#### Scenario: Hash determinism

- WHEN the same secret is hashed multiple times
- THEN MUST produce the same non-empty hash
- Test: `webhook_secret_hash_is_deterministic_and_nonempty` in `src/gateway/mod.rs`

#### Scenario: Missing header rejected

- WHEN the webhook secret header is missing
- THEN MUST reject the request
- Test: `webhook_secret_hash_rejects_missing_header` in `src/gateway/mod.rs`

#### Scenario: Invalid header rejected

- WHEN the webhook secret header contains an incorrect hash
- THEN MUST reject the request
- Test: `webhook_secret_hash_rejects_invalid_header` in `src/gateway/mod.rs`

#### Scenario: Valid header accepted

- WHEN the webhook secret header contains a correct hash
- THEN MUST accept the request
- Test: `webhook_secret_hash_accepts_valid_header` in `src/gateway/mod.rs`

### REQ-GW-005B: GitHub Webhook Verification

Gateway MUST verify GitHub webhook signatures and enforce idempotent delivery processing.

#### Scenario: Not-found when not configured

- WHEN GitHub webhook endpoint is hit but GitHub channel is not configured
- THEN MUST return 404 Not Found
- Test: `github_webhook_returns_not_found_when_not_configured` in `src/gateway/mod.rs`

#### Scenario: Invalid signature rejected

- WHEN GitHub webhook has an invalid HMAC-SHA256 signature
- THEN MUST reject the request
- Test: `github_webhook_rejects_invalid_signature` in `src/gateway/mod.rs`

#### Scenario: Duplicate delivery returns duplicate status

- WHEN a GitHub webhook delivery ID has already been processed
- THEN MUST return duplicate status without re-processing
- Test: `github_webhook_duplicate_delivery_returns_duplicate_status` in `src/gateway/mod.rs`

### REQ-GW-005C: Nextcloud Talk Webhook Verification

Gateway MUST verify Nextcloud Talk webhook signatures.

#### Scenario: Not-found when not configured

- WHEN Nextcloud Talk webhook endpoint is hit but not configured
- THEN MUST return 404 Not Found
- Test: `nextcloud_talk_webhook_returns_not_found_when_not_configured` in `src/gateway/mod.rs`

#### Scenario: Invalid signature rejected

- WHEN Nextcloud Talk webhook has an invalid signature
- THEN MUST reject the request
- Test: `nextcloud_talk_webhook_rejects_invalid_signature` in `src/gateway/mod.rs`

### REQ-GW-005D: QQ Webhook Handling

Gateway MUST handle QQ webhook validation challenges and route QQ messages.

#### Scenario: Not-found when not configured

- WHEN QQ webhook endpoint is hit but not configured
- THEN MUST return 404 Not Found
- Test: `qq_webhook_returns_not_found_when_not_configured` in `src/gateway/mod.rs`

#### Scenario: Validation returns signed challenge

- WHEN QQ sends a validation challenge request
- THEN MUST return a correctly signed challenge response
- Test: `qq_webhook_validation_returns_signed_challenge` in `src/gateway/mod.rs`

### REQ-GW-006: SSE Events

`handle_sse_events` MUST stream Server-Sent Events to connected clients with auth enforcement.

#### Scenario: Pairing token required when pairing is enabled

- WHEN SSE connection is attempted with pairing enabled
- THEN MUST require a valid pairing token
- Test: `evaluate_sse_auth_requires_pairing_token_when_pairing_is_enabled` in `src/gateway/sse.rs`

#### Scenario: Public access rejected without auth layer when pairing disabled

- WHEN SSE connection is attempted from a public IP with pairing disabled and no auth layer
- THEN MUST reject the connection
- Test: `evaluate_sse_auth_rejects_public_without_auth_layer_when_pairing_disabled` in `src/gateway/sse.rs`

#### Scenario: Loopback or valid token allowed when pairing disabled

- WHEN SSE connection is from loopback or carries a valid token with pairing disabled
- THEN MUST allow the connection
- Test: `evaluate_sse_auth_allows_loopback_or_valid_token_when_pairing_disabled` in `src/gateway/sse.rs`

### REQ-GW-007: BroadcastObserver

`BroadcastObserver` MUST implement the Observer trait to broadcast agent events to SSE clients.

#### Scenario: Observer name identity

- WHEN `BroadcastObserver::name()` is called
- THEN MUST return the correct observer name
- Test: `broadcast_observer_name` in `src/gateway/sse.rs`

#### Scenario: Tool call event broadcast

- WHEN a tool call event occurs
- THEN MUST broadcast it to connected SSE clients
- Test: `broadcast_observer_broadcasts_tool_call_event` in `src/gateway/sse.rs`

#### Scenario: LLM request event broadcast

- WHEN an LLM request event occurs
- THEN MUST broadcast it to connected SSE clients
- Test: `broadcast_observer_broadcasts_llm_request_event` in `src/gateway/sse.rs`

#### Scenario: Agent start event broadcast

- WHEN an agent start event occurs
- THEN MUST broadcast it to connected SSE clients
- Test: `broadcast_observer_broadcasts_agent_start_event` in `src/gateway/sse.rs`

#### Scenario: Error event broadcast

- WHEN an error event occurs
- THEN MUST broadcast it to connected SSE clients
- Test: `broadcast_observer_broadcasts_error_event` in `src/gateway/sse.rs`

#### Scenario: Non-broadcast events skipped

- WHEN an event that should not be broadcast occurs
- THEN MUST skip it without broadcasting
- Test: `broadcast_observer_skips_non_broadcast_events` in `src/gateway/sse.rs`

### REQ-GW-008: WebSocket Chat

`handle_ws_chat` MUST support WebSocket chat with token auth, session management, history restoration, query parameter parsing, and tool-call sanitization.

#### Scenario: Token extracted from Authorization header

- WHEN WebSocket connection has an Authorization header with bearer token
- THEN MUST extract bearer token from header preferentially
- Test: `extract_ws_bearer_token_prefers_authorization_header` in `src/gateway/ws.rs`

#### Scenario: Token extracted from WebSocket protocol

- WHEN WebSocket connection has a token in the protocol field
- THEN MUST extract bearer token from protocol
- Test: `extract_ws_bearer_token_reads_websocket_protocol_token` in `src/gateway/ws.rs`

#### Scenario: Empty tokens rejected

- WHEN WebSocket connection provides an empty bearer token
- THEN MUST reject the empty token
- Test: `extract_ws_bearer_token_rejects_empty_tokens` in `src/gateway/ws.rs`

#### Scenario: Token extracted from query parameter fallback

- WHEN no header or protocol token is present but query parameter has token
- THEN MUST fall back to query parameter token
- Test: `extract_ws_bearer_token_reads_query_token_fallback` in `src/gateway/ws.rs`

#### Scenario: Protocol token preferred over query token

- WHEN both protocol and query parameter tokens are present
- THEN MUST prefer the protocol token
- Test: `extract_ws_bearer_token_prefers_protocol_over_query_token` in `src/gateway/ws.rs`

#### Scenario: Query token extraction

- WHEN query string contains a token parameter
- THEN MUST extract the token value
- Test: `extract_query_token_reads_token_param` in `src/gateway/ws.rs`

#### Scenario: Query parameter parsing for token and session ID

- WHEN query string contains token and session_id parameters
- THEN MUST parse both correctly
- Test: `parse_ws_query_params_reads_token_and_session_id` in `src/gateway/ws.rs`

#### Scenario: Invalid session ID rejected in query params

- WHEN query string contains an invalid session_id
- THEN MUST reject the invalid session ID
- Test: `parse_ws_query_params_rejects_invalid_session_id` in `src/gateway/ws.rs`

#### Scenario: History turns skip system and non-dialog turns

- WHEN chat history contains system messages and non-dialog turns
- THEN MUST skip them when extracting history turns
- Test: `ws_history_turns_from_chat_skips_system_and_non_dialog_turns` in `src/gateway/ws.rs`

#### Scenario: History restoration applies system prompt once

- WHEN session has prior chat history
- THEN MUST restore history with system prompt applied exactly once
- Test: `restore_chat_history_applies_system_prompt_once` in `src/gateway/ws.rs`

#### Scenario: Pairing token required when pairing is enabled

- WHEN WebSocket connection is attempted with pairing enabled
- THEN MUST require a valid pairing token
- Test: `evaluate_ws_auth_requires_pairing_token_when_pairing_is_enabled` in `src/gateway/ws.rs`

#### Scenario: Public access rejected without auth layer when pairing disabled

- WHEN WebSocket connection is attempted from a public IP with pairing disabled and no auth layer
- THEN MUST reject the connection
- Test: `evaluate_ws_auth_rejects_public_without_auth_layer_when_pairing_disabled` in `src/gateway/ws.rs`

#### Scenario: Loopback or valid token allowed when pairing disabled

- WHEN WebSocket connection is from loopback or carries a valid token with pairing disabled
- THEN MUST allow the connection
- Test: `evaluate_ws_auth_allows_loopback_or_valid_token_when_pairing_disabled` in `src/gateway/ws.rs`

#### Scenario: Response sanitization removes tool-call tags

- WHEN response contains tool-call XML tags
- THEN MUST strip them from the response
- Test: `sanitize_ws_response_removes_tool_call_tags` in `src/gateway/ws.rs`

#### Scenario: Response sanitization removes isolated tool JSON artifacts

- WHEN response contains isolated tool JSON artifacts
- THEN MUST strip them from the response
- Test: `sanitize_ws_response_removes_isolated_tool_json_artifacts` in `src/gateway/ws.rs`

#### Scenario: Response sanitization blocks detected credentials

- WHEN response contains detected credentials
- THEN MUST block the response when credential scanning is configured
- Test: `sanitize_ws_response_blocks_detected_credentials_when_configured` in `src/gateway/ws.rs`

#### Scenario: System prompt includes tool protocol for prompt mode

- WHEN system prompt is built for prompt tool-call mode
- THEN MUST include the XML tool protocol instructions
- Test: `build_ws_system_prompt_includes_tool_protocol_for_prompt_mode` in `src/gateway/ws.rs`

#### Scenario: System prompt omits XML protocol for native mode

- WHEN system prompt is built for native tool-call mode
- THEN MUST omit the XML tool protocol instructions
- Test: `build_ws_system_prompt_omits_xml_protocol_for_native_mode` in `src/gateway/ws.rs`

#### Scenario: Response finalization uses prompt mode tool output

- WHEN final response text is empty in prompt mode
- THEN MUST use extracted tool output as the response
- Test: `finalize_ws_response_uses_prompt_mode_tool_output_when_final_text_empty` in `src/gateway/ws.rs`

#### Scenario: Response finalization uses native tool message output

- WHEN final response text is empty in native mode
- THEN MUST use tool message output as the response
- Test: `finalize_ws_response_uses_native_tool_message_output_when_final_text_empty` in `src/gateway/ws.rs`

#### Scenario: Response finalization uses static fallback

- WHEN final response text is empty and no tool output available
- THEN MUST use static fallback message
- Test: `finalize_ws_response_uses_static_fallback_when_nothing_available` in `src/gateway/ws.rs`

### REQ-GW-009: OpenAI Compatibility

`handle_v1_chat_completions` and `handle_v1_models` MUST implement OpenAI-compatible API endpoints with request/response serialization, streaming, auth, and credential sanitization.

#### Scenario: Minimal request deserialization

- WHEN an OpenAI-format chat completion request with only required fields arrives
- THEN MUST deserialize successfully
- Test: `chat_completions_request_deserializes_minimal` in `src/gateway/openai_compat.rs`

#### Scenario: Full request deserialization

- WHEN an OpenAI-format chat completion request with all optional fields arrives
- THEN MUST deserialize successfully
- Test: `chat_completions_request_deserializes_full` in `src/gateway/openai_compat.rs`

#### Scenario: Response serialization

- WHEN a chat completion response is generated
- THEN MUST serialize in OpenAI-compatible JSON format
- Test: `chat_completions_response_serializes` in `src/gateway/openai_compat.rs`

#### Scenario: Models response serialization

- WHEN a models list response is generated
- THEN MUST serialize in OpenAI-compatible format
- Test: `models_response_serializes` in `src/gateway/openai_compat.rs`

#### Scenario: Streaming chunk serialization

- WHEN streaming mode produces a chunk
- THEN MUST serialize with all required fields
- Test: `streaming_chunk_serializes` in `src/gateway/openai_compat.rs`

#### Scenario: Streaming chunk omits None fields

- WHEN streaming chunk has optional None fields
- THEN MUST omit them from serialization
- Test: `streaming_chunk_omits_none_fields` in `src/gateway/openai_compat.rs`

#### Scenario: Unix timestamp is reasonable

- WHEN a unix timestamp is generated
- THEN MUST produce a reasonable current-epoch value
- Test: `unix_timestamp_is_reasonable` in `src/gateway/openai_compat.rs`

#### Scenario: Body size limit enforcement

- WHEN a request body exceeds 512 KB for OpenAI-compat endpoints
- THEN MUST reject the request
- Test: `body_size_limit_is_512kb` in `src/gateway/openai_compat.rs`

#### Scenario: Credential redaction in response

- WHEN response contains detected credentials with redact mode
- THEN MUST redact the credentials
- Test: `sanitize_openai_compat_response_redacts_detected_credentials` in `src/gateway/openai_compat.rs`

#### Scenario: Credential blocking in response

- WHEN response contains detected credentials with block mode
- THEN MUST block the entire response
- Test: `sanitize_openai_compat_response_blocks_detected_credentials_when_configured` in `src/gateway/openai_compat.rs`

#### Scenario: Credential scan skipped when disabled

- WHEN credential scanning is disabled
- THEN MUST skip scanning and pass response through
- Test: `sanitize_openai_compat_response_skips_scan_when_disabled` in `src/gateway/openai_compat.rs`

#### Scenario: Pairing token required when pairing is enabled

- WHEN API request is made with pairing enabled
- THEN MUST require a valid pairing token
- Test: `evaluate_openai_gateway_auth_requires_pairing_token_when_pairing_is_enabled` in `src/gateway/openai_compat.rs`

#### Scenario: Public access rejected without auth layer when pairing disabled

- WHEN API request is from a public IP with pairing disabled and no auth layer
- THEN MUST reject the request
- Test: `evaluate_openai_gateway_auth_rejects_public_without_auth_layer_when_pairing_disabled` in `src/gateway/openai_compat.rs`

#### Scenario: Loopback or secondary auth layer allowed

- WHEN API request is from loopback or has a secondary auth layer
- THEN MUST allow the request
- Test: `evaluate_openai_gateway_auth_allows_loopback_or_secondary_auth_layer` in `src/gateway/openai_compat.rs`

### REQ-GW-010: OpenClaw Compatibility

`handle_api_chat` and `handle_v1_chat_completions_with_tools` MUST implement OpenClaw-specific chat endpoints with request parsing, validation, context extraction, and response serialization.

#### Scenario: Minimal request deserialization

- WHEN an OpenClaw chat request with only required fields arrives
- THEN MUST deserialize successfully
- Test: `api_chat_body_deserializes_minimal` in `src/gateway/openclaw_compat.rs`

#### Scenario: Full request deserialization

- WHEN an OpenClaw chat request with all optional fields arrives
- THEN MUST deserialize successfully
- Test: `api_chat_body_deserializes_full` in `src/gateway/openclaw_compat.rs`

#### Scenario: OAI request with extra fields

- WHEN an OpenAI-format request with extra unrecognized fields arrives
- THEN MUST deserialize successfully and ignore extra fields
- Test: `oai_request_deserializes_with_extra_fields` in `src/gateway/openclaw_compat.rs`

#### Scenario: OAI response serialization

- WHEN an OpenClaw OAI response is generated
- THEN MUST serialize correctly
- Test: `oai_response_serializes_correctly` in `src/gateway/openclaw_compat.rs`

#### Scenario: Streaming chunk omits None fields

- WHEN streaming chunk has optional None fields
- THEN MUST omit them from serialization
- Test: `streaming_chunk_omits_none_fields` in `src/gateway/openclaw_compat.rs`

#### Scenario: Memory key uniqueness

- WHEN API chat memory keys are generated
- THEN MUST produce unique keys per invocation
- Test: `memory_key_is_unique` in `src/gateway/openclaw_compat.rs`

#### Scenario: Missing message rejected

- WHEN request is missing the required message field
- THEN MUST reject with appropriate error
- Test: `api_chat_body_rejects_missing_message` in `src/gateway/openclaw_compat.rs`

#### Scenario: Empty messages rejected

- WHEN OAI request contains an empty messages array
- THEN MUST reject with appropriate error
- Test: `oai_request_rejects_empty_messages` in `src/gateway/openclaw_compat.rs`

#### Scenario: No user message detection

- WHEN OAI request has no user message
- THEN MUST detect and handle the missing user message
- Test: `oai_request_no_user_message_detected` in `src/gateway/openclaw_compat.rs`

#### Scenario: Whitespace-only user message detection

- WHEN OAI request has a whitespace-only user message
- THEN MUST detect the empty content
- Test: `oai_request_whitespace_only_user_message` in `src/gateway/openclaw_compat.rs`

#### Scenario: Context extraction skips last user message

- WHEN multiple messages include user messages
- THEN MUST extract context from prior messages, skipping the last user message
- Test: `oai_context_extraction_skips_last_user_message` in `src/gateway/openclaw_compat.rs`

### REQ-GW-011: REST API Handlers

API handlers MUST serve status, config, tools, cron, integrations, doctor, memory, cost, health, and pairing endpoints with proper secret masking and config hydration.

#### Scenario: Config masking preserves TOML validity and API key types

- WHEN config is retrieved via API
- THEN MUST mask sensitive fields while keeping TOML valid and preserving API key array types
- Test: `masking_keeps_toml_valid_and_preserves_api_keys_type` in `src/gateway/api.rs`

#### Scenario: Config hydration restores masked secrets and paths

- WHEN config is saved via API after editing
- THEN MUST restore masked secrets and paths from the current running config
- Test: `hydrate_config_for_save_restores_masked_secrets_and_paths` in `src/gateway/api.rs`

#### Scenario: Dashboard config normalization promotes single API key string to array

- WHEN dashboard config has a single API key string instead of array
- THEN MUST promote it to an array for consistency
- Test: `normalize_dashboard_config_toml_promotes_single_api_key_string_to_array` in `src/gateway/api.rs`

#### Scenario: Masking covers WATI email and Feishu secrets

- WHEN config contains WATI email and Feishu secrets
- THEN MUST mask those fields
- Test: `mask_sensitive_fields_covers_wati_email_and_feishu_secrets` in `src/gateway/api.rs`

#### Scenario: Hydration restores WATI email and Feishu secrets

- WHEN config is saved with masked WATI email and Feishu secrets
- THEN MUST restore the original values from running config
- Test: `hydrate_config_for_save_restores_wati_email_and_feishu_secrets` in `src/gateway/api.rs`

### REQ-GW-012: Static File Serving and Cache Control

`handle_static` and `handle_spa_fallback` MUST serve embedded static files with appropriate cache-control headers and correct path handling.

#### Scenario: Assets path gets immutable cache-control

- WHEN a request targets an assets path (e.g., `/_app/assets/...`)
- THEN MUST set cache-control to immutable with long max-age
- Test: `cache_control_assets_path_is_immutable` in `src/gateway/static_files.rs`

#### Scenario: Non-asset path gets no-cache

- WHEN a request targets a non-asset path
- THEN MUST set cache-control to no-cache
- Test: `cache_control_non_asset_is_no_cache` in `src/gateway/static_files.rs`

#### Scenario: Static handler strips app prefix

- WHEN a request targets `/_app/index.html`
- THEN MUST strip the `/_app/` prefix before resolving the embedded file
- Test: `handle_static_strips_app_prefix` in `src/gateway/static_files.rs`

#### Scenario: Missing file returns 404

- WHEN a request targets a non-existent embedded file
- THEN MUST return 404 Not Found
- Test: `handle_static_missing_file_returns_404` in `src/gateway/static_files.rs`

#### Scenario: SPA fallback returns valid response

- WHEN a non-API, non-static GET request arrives
- THEN MUST serve index.html (or 404 if not embedded) without panicking
- Test: `spa_fallback_behavior` in `src/gateway/static_files.rs`

### REQ-GW-013: Client IP Resolution and Proxy Trust

Gateway MUST resolve client IP addresses correctly based on proxy trust configuration, for both rate limiting and loopback detection.

#### Scenario: Client key defaults to peer address in untrusted proxy mode

- WHEN proxy is not trusted
- THEN MUST use peer socket address as client key
- Test: `client_key_defaults_to_peer_addr_when_untrusted_proxy_mode` in `src/gateway/mod.rs`

#### Scenario: Client key uses forwarded IP in trusted proxy mode

- WHEN proxy is trusted and forwarded header is present
- THEN MUST use forwarded IP as client key
- Test: `client_key_uses_forwarded_ip_only_in_trusted_proxy_mode` in `src/gateway/mod.rs`

#### Scenario: Client key falls back to peer when forwarded header is invalid

- WHEN proxy is trusted but forwarded header is invalid
- THEN MUST fall back to peer socket address
- Test: `client_key_falls_back_to_peer_when_forwarded_header_invalid` in `src/gateway/mod.rs`

#### Scenario: Loopback detection uses peer address in untrusted proxy mode

- WHEN proxy is not trusted
- THEN MUST evaluate loopback status from peer address
- Test: `is_loopback_request_uses_peer_addr_when_untrusted_proxy_mode` in `src/gateway/mod.rs`

#### Scenario: Loopback detection uses forwarded IP in trusted proxy mode

- WHEN proxy is trusted and forwarded header is present
- THEN MUST evaluate loopback status from forwarded IP
- Test: `is_loopback_request_uses_forwarded_ip_in_trusted_proxy_mode` in `src/gateway/mod.rs`

#### Scenario: Loopback detection falls back to peer when forwarded is invalid

- WHEN proxy is trusted but forwarded header is invalid
- THEN MUST fall back to peer address for loopback detection
- Test: `is_loopback_request_falls_back_to_peer_when_forwarded_invalid` in `src/gateway/mod.rs`

### REQ-GW-014: Gateway Response Sanitization

Gateway MUST sanitize outbound responses to remove tool-call artifacts and block credential leaks before returning to clients.

#### Scenario: Tool-call XML tags removed

- WHEN gateway response contains tool-call XML tags
- THEN MUST strip them from the response
- Test: `sanitize_gateway_response_removes_tool_call_tags` in `src/gateway/mod.rs`

#### Scenario: Isolated tool JSON artifacts removed

- WHEN gateway response contains isolated tool JSON artifacts
- THEN MUST strip them from the response
- Test: `sanitize_gateway_response_removes_isolated_tool_json_artifacts` in `src/gateway/mod.rs`

#### Scenario: Detected credentials blocked

- WHEN gateway response contains detected credentials and blocking is configured
- THEN MUST block the response
- Test: `sanitize_gateway_response_blocks_detected_credentials_when_configured` in `src/gateway/mod.rs`

### REQ-GW-015: Node Control API

Gateway MUST expose node control endpoints for multi-node management with proper auth enforcement.

#### Scenario: Not found when disabled

- WHEN node control endpoint is hit but feature is disabled
- THEN MUST return 404 Not Found
- Test: `node_control_returns_not_found_when_disabled` in `src/gateway/mod.rs`

#### Scenario: List returns stub nodes when enabled

- WHEN node control list endpoint is hit with feature enabled
- THEN MUST return stub node list
- Test: `node_control_list_returns_stub_nodes_when_enabled` in `src/gateway/mod.rs`

#### Scenario: Rejects public requests without auth layers

- WHEN node control endpoint is hit from public IP without auth
- THEN MUST reject the request
- Test: `node_control_rejects_public_requests_without_auth_layers` in `src/gateway/mod.rs`

### REQ-GW-016: Node ID Allow-List Enforcement

Gateway MUST enforce node ID allow-lists for multi-node webhook routing.

#### Scenario: Empty allowlist accepts any node ID

- WHEN node ID allowlist is empty
- THEN MUST accept any node ID
- Test: `node_id_allowed_with_empty_allowlist_accepts_any` in `src/gateway/mod.rs`

#### Scenario: Non-empty allowlist enforced

- WHEN node ID allowlist contains specific IDs
- THEN MUST only accept listed node IDs
- Test: `node_id_allowed_respects_allowlist` in `src/gateway/mod.rs`

### REQ-GW-017: Metrics Endpoint

Gateway MUST expose a `/metrics` endpoint with Prometheus output when enabled, with proper auth enforcement.

#### Scenario: Hint returned when Prometheus is disabled

- WHEN metrics endpoint is hit with Prometheus disabled
- THEN MUST return a hint message
- Test: `metrics_endpoint_returns_hint_when_prometheus_is_disabled` in `src/gateway/mod.rs`

#### Scenario: Prometheus output rendered when enabled

- WHEN metrics endpoint is hit with Prometheus enabled
- THEN MUST render Prometheus-format output
- Test: `metrics_endpoint_renders_prometheus_output` in `src/gateway/mod.rs`

#### Scenario: Public clients rejected when pairing is disabled

- WHEN metrics endpoint is hit from public IP with pairing disabled
- THEN MUST reject the request
- Test: `metrics_endpoint_rejects_public_clients_when_pairing_is_disabled` in `src/gateway/mod.rs`

#### Scenario: Bearer token required when pairing is enabled

- WHEN metrics endpoint is hit with pairing enabled
- THEN MUST require a valid bearer token
- Test: `metrics_endpoint_requires_bearer_token_when_pairing_is_enabled` in `src/gateway/mod.rs`

### REQ-GW-018: Pairing Token Persistence

Gateway MUST persist pairing tokens to the configuration file.

#### Scenario: Persist pairing tokens writes config

- WHEN pairing tokens are saved
- THEN MUST write them to the config file
- Test: `persist_pairing_tokens_writes_config_tokens` in `src/gateway/mod.rs`

## Coverage Notes

All 140 test functions across 7 source files are explicitly mapped to scenarios:

| File | Tests | Requirements |
|------|-------|-------------|
| `src/gateway/mod.rs` | 75 | REQ-GW-001, 002, 003, 004, 005, 005A, 005B, 005C, 005D, 013, 014, 015, 016, 017, 018 |
| `src/gateway/ws.rs` | 21 | REQ-GW-008 |
| `src/gateway/openai_compat.rs` | 14 | REQ-GW-009 |
| `src/gateway/openclaw_compat.rs` | 11 | REQ-GW-010 |
| `src/gateway/sse.rs` | 9 | REQ-GW-006, 007 |
| `src/gateway/api.rs` | 5 | REQ-GW-011 |
| `src/gateway/static_files.rs` | 5 | REQ-GW-012 |

New requirements added to cover previously ungrouped tests:
- REQ-GW-005A: Webhook Secret Hash Verification (4 tests from mod.rs)
- REQ-GW-005B: GitHub Webhook Verification (3 tests from mod.rs)
- REQ-GW-005C: Nextcloud Talk Webhook Verification (2 tests from mod.rs)
- REQ-GW-005D: QQ Webhook Handling (2 tests from mod.rs)
- REQ-GW-013: Client IP Resolution and Proxy Trust (6 tests from mod.rs)
- REQ-GW-014: Gateway Response Sanitization (3 tests from mod.rs)
- REQ-GW-015: Node Control API (3 tests from mod.rs)
- REQ-GW-016: Node ID Allow-List Enforcement (2 tests from mod.rs)
- REQ-GW-017: Metrics Endpoint (4 tests from mod.rs)
- REQ-GW-018: Pairing Token Persistence (1 test from mod.rs)

Expanded existing requirements with additional explicit scenarios:
- REQ-GW-001: Added 4 explicit scenarios (was 1 vague reference)
- REQ-GW-002: Expanded from 1 vague scenario to 13 explicit scenarios
- REQ-GW-003: Expanded from 1 vague scenario to 9 explicit scenarios
- REQ-GW-004: Expanded from 1 vague scenario to 9 explicit scenarios
- REQ-GW-005: Expanded from 1 vague scenario to 12 explicit scenarios
- REQ-GW-006: Expanded from 1 vague scenario to 3 explicit scenarios with exact names
- REQ-GW-007: Expanded from 1 untested note to 6 explicit scenarios with exact names
- REQ-GW-008: Expanded from 6 vague references to 21 explicit scenarios with exact names
- REQ-GW-009: Expanded from 5 vague references to 14 explicit scenarios with exact names
- REQ-GW-010: Expanded from 3 vague references to 11 explicit scenarios with exact names
- REQ-GW-011: Expanded from 1 vague reference to 5 explicit scenarios with exact names
- REQ-GW-012: Expanded from 2 scenarios to 5 explicit scenarios with exact names

## Mock Strategy

- HTTP requests: `axum::test` utilities for handler testing
- Rate limiting: Direct construction and testing of `SlidingWindowRateLimiter` and `GatewayRateLimiter`
- Idempotency: In-memory `IdempotencyStore` with test data
- Auth: Test-safe `SecurityPolicy` with configurable pairing
- Config: Default construction with test values
- Memory: `MockMemory` and `TrackingMemory` test doubles
- SSE/WebSocket: Test utility functions directly; skip full connection lifecycle
- Webhooks: `MockScheduleTool` for tool registration; direct handler invocation with test `ConnectInfo`
