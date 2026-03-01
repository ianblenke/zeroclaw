# Providers Specification

## Purpose

Define requirements for the provider subsystem: trait contract, all provider implementations (Anthropic, OpenAI, Gemini, Bedrock, Ollama, OpenRouter, Compatible, GLM, Telnyx, Cursor, Copilot, Codex), reliable wrapper, health tracking, router, backoff, and quota management.

## Scope

- Files: 21 files in `src/providers/` (~21,978 LOC total)
- Risk tier: HIGH (providers handle API keys, model routing, and cost-bearing API calls)
- Existing tests: 660 across 21 files
  - `src/providers/traits.rs` — 27 tests
  - `src/providers/mod.rs` — 128 tests
  - `src/providers/anthropic.rs` — 50 tests
  - `src/providers/openai.rs` — 29 tests
  - `src/providers/gemini.rs` — 47 tests
  - `src/providers/bedrock.rs` — 55 tests
  - `src/providers/ollama.rs` — 30 tests
  - `src/providers/openrouter.rs` — 27 tests
  - `src/providers/compatible.rs` — 96 tests
  - `src/providers/glm.rs` — 10 tests
  - `src/providers/telnyx.rs` — 8 tests
  - `src/providers/cursor.rs` — 8 tests
  - `src/providers/copilot.rs` — 12 tests
  - `src/providers/openai_codex.rs` — 29 tests
  - `src/providers/reliable.rs` — 50 tests
  - `src/providers/health.rs` — 11 tests
  - `src/providers/router.rs` — 13 tests
  - `src/providers/backoff.rs` — 5 tests
  - `src/providers/quota_types.rs` — 10 tests
  - `src/providers/quota_adapter.rs` — 11 tests
  - `src/providers/quota_cli.rs` — 4 tests

## Requirements

### REQ-PROV-001: Provider Trait Contract

All providers MUST implement the `Provider` trait with capabilities, chat methods, streaming, tool support, and warmup.

#### Scenario: ChatMessage constructors produce correct roles

- WHEN `ChatMessage::user()` or `ChatMessage::assistant()` is called
- THEN MUST produce messages with the correct role and content
- Test: `chat_message_constructors` in `src/providers/traits.rs`

#### Scenario: ChatResponse helper methods

- WHEN `ChatResponse` helpers are used
- THEN MUST correctly extract text and metadata
- Test: `chat_response_helpers` in `src/providers/traits.rs`

#### Scenario: TokenUsage defaults to None

- WHEN `TokenUsage` is default-constructed
- THEN all fields MUST be None
- Test: `token_usage_default_is_none` in `src/providers/traits.rs`

#### Scenario: ChatResponse with usage metadata

- WHEN a ChatResponse includes usage data
- THEN MUST preserve token counts
- Test: `chat_response_with_usage` in `src/providers/traits.rs`

#### Scenario: ToolCall serialization

- WHEN a ToolCall is serialized
- THEN MUST round-trip correctly
- Test: `tool_call_serialization` in `src/providers/traits.rs`

#### Scenario: ConversationMessage variant construction

- WHEN ConversationMessage variants are created
- THEN MUST preserve role and content
- Test: `conversation_message_variants` in `src/providers/traits.rs`

#### Scenario: ProviderCapabilities default values

- WHEN ProviderCapabilities is default-constructed
- THEN MUST have expected default flags
- Test: `provider_capabilities_default` in `src/providers/traits.rs`

#### Scenario: ProviderCapabilities equality comparison

- WHEN two ProviderCapabilities are compared
- THEN MUST compare field-by-field
- Test: `provider_capabilities_equality` in `src/providers/traits.rs`

#### Scenario: supports_native_tools reflects capabilities

- WHEN default Provider::supports_native_tools is called
- THEN MUST reflect capabilities mapping
- Test: `supports_native_tools_reflects_capabilities_default_mapping` in `src/providers/traits.rs`

#### Scenario: supports_vision reflects capabilities

- WHEN default Provider::supports_vision is called
- THEN MUST reflect capabilities mapping
- Test: `supports_vision_reflects_capabilities_default_mapping` in `src/providers/traits.rs`

#### Scenario: ToolsPayload variants

- WHEN ToolsPayload is constructed
- THEN MUST support all variant types
- Test: `tools_payload_variants` in `src/providers/traits.rs`

#### Scenario: Build tool instructions in text format

- WHEN build_tool_instructions is called with tools
- THEN MUST produce valid text-format instructions
- Test: `build_tool_instructions_text_format` in `src/providers/traits.rs`

#### Scenario: Build tool instructions with empty tools

- WHEN build_tool_instructions is called with empty tools
- THEN MUST produce empty instruction block
- Test: `build_tool_instructions_text_empty` in `src/providers/traits.rs`

#### Scenario: Provider convert_tools default implementation

- WHEN a provider uses default convert_tools
- THEN MUST return text payload
- Test: `provider_convert_tools_default` in `src/providers/traits.rs`

#### Scenario: Provider chat with prompt-guided fallback

- WHEN a provider without native tools uses chat
- THEN MUST fall back to prompt-guided tool instructions
- Test: `provider_chat_prompt_guided_fallback` in `src/providers/traits.rs`

#### Scenario: Provider chat without tools

- WHEN a provider calls chat without tools
- THEN MUST send plain chat request
- Test: `provider_chat_without_tools` in `src/providers/traits.rs`

#### Scenario: Provider chat prompt-guided preserves existing system when not first

- WHEN prompt-guided tools are injected and system prompt is not first
- THEN MUST preserve existing system content
- Test: `provider_chat_prompt_guided_preserves_existing_system_not_first` in `src/providers/traits.rs`

#### Scenario: Provider chat prompt-guided uses convert_tools override

- WHEN a provider overrides convert_tools
- THEN MUST use the override in prompt-guided fallback
- Test: `provider_chat_prompt_guided_uses_convert_tools_override` in `src/providers/traits.rs`

#### Scenario: StreamChunk delta creates non-final chunk

- WHEN StreamChunk::delta is called
- THEN MUST create a non-final chunk with text
- Test: `stream_chunk_delta_creates_non_final` in `src/providers/traits.rs`

#### Scenario: StreamChunk final creates final chunk

- WHEN StreamChunk::final_chunk is called
- THEN MUST create a final chunk
- Test: `stream_chunk_final_creates_final` in `src/providers/traits.rs`

#### Scenario: StreamChunk error creates final with message

- WHEN StreamChunk::error is called
- THEN MUST create a final chunk with error message
- Test: `stream_chunk_error_creates_final_with_message` in `src/providers/traits.rs`

#### Scenario: StreamChunk with token estimate

- WHEN StreamChunk includes token estimate
- THEN MUST preserve the estimate
- Test: `stream_chunk_with_token_estimate` in `src/providers/traits.rs`

#### Scenario: StreamOptions builder

- WHEN StreamOptions is constructed
- THEN MUST set model and system prompt
- Test: `stream_options_builder` in `src/providers/traits.rs`

#### Scenario: Role classification for user and assistant

- WHEN is_user_or_assistant_role is called
- THEN MUST correctly classify user and assistant roles
- Test: `is_user_or_assistant_role_classifies_correctly` in `src/providers/traits.rs`

#### Scenario: Default warmup succeeds

- WHEN default Provider::warmup is called
- THEN MUST succeed without error
- Test: `provider_default_warmup_succeeds` in `src/providers/traits.rs`

#### Scenario: Default supports_streaming is false

- WHEN default Provider::supports_streaming is called
- THEN MUST return false
- Test: `provider_default_supports_streaming_is_false` in `src/providers/traits.rs`

#### Scenario: Prompt-guided chat rejects non-prompt payload

- WHEN chat is called with non-prompt tools payload
- THEN MUST reject with error
- Test: `provider_chat_prompt_guided_rejects_non_prompt_payload` in `src/providers/traits.rs`

### REQ-PROV-002: Anthropic Provider

`AnthropicProvider` MUST support Claude models with streaming, vision, native tools, and system prompt handling.

#### Scenario: Creates with API key

- WHEN AnthropicProvider is created with a key
- THEN MUST store the key
- Test: `creates_with_key` in `src/providers/anthropic.rs`

#### Scenario: Creates without API key

- WHEN AnthropicProvider is created without a key
- THEN MUST create successfully with None key
- Test: `creates_without_key` in `src/providers/anthropic.rs`

#### Scenario: Creates with empty key

- WHEN AnthropicProvider is created with empty string key
- THEN MUST treat as no key
- Test: `creates_with_empty_key` in `src/providers/anthropic.rs`

#### Scenario: Creates with whitespace key

- WHEN AnthropicProvider is created with whitespace-only key
- THEN MUST treat as no key
- Test: `creates_with_whitespace_key` in `src/providers/anthropic.rs`

#### Scenario: Creates with custom base URL

- WHEN AnthropicProvider is created with custom base URL
- THEN MUST use the custom URL
- Test: `creates_with_custom_base_url` in `src/providers/anthropic.rs`

#### Scenario: Custom base URL trims trailing slash

- WHEN custom base URL has trailing slash
- THEN MUST trim it
- Test: `custom_base_url_trims_trailing_slash` in `src/providers/anthropic.rs`

#### Scenario: Default base URL when none provided

- WHEN no base URL is provided
- THEN MUST use default Anthropic API URL
- Test: `default_base_url_when_none_provided` in `src/providers/anthropic.rs`

#### Scenario: Chat fails without key

- WHEN chat is called without an API key
- THEN MUST return error
- Test: `chat_fails_without_key` in `src/providers/anthropic.rs`

#### Scenario: Setup token detection

- WHEN a token is checked for setup status
- THEN MUST correctly identify setup vs regular tokens
- Test: `setup_token_detection_works` in `src/providers/anthropic.rs`

#### Scenario: Auth uses Bearer and beta for setup tokens

- WHEN auth is applied with a setup token
- THEN MUST use Bearer auth with beta header
- Test: `apply_auth_uses_bearer_and_beta_for_setup_tokens` in `src/providers/anthropic.rs`

#### Scenario: Auth uses x-api-key for regular tokens

- WHEN auth is applied with a regular token
- THEN MUST use x-api-key header
- Test: `apply_auth_uses_x_api_key_for_regular_tokens` in `src/providers/anthropic.rs`

#### Scenario: Chat with system fails without key

- WHEN chat_with_system is called without key
- THEN MUST return error
- Test: `chat_with_system_fails_without_key` in `src/providers/anthropic.rs`

#### Scenario: Chat request serializes without system

- WHEN a chat request is built without system prompt
- THEN MUST serialize correctly
- Test: `chat_request_serializes_without_system` in `src/providers/anthropic.rs`

#### Scenario: Chat request serializes with system

- WHEN a chat request is built with system prompt
- THEN MUST include system in serialized output
- Test: `chat_request_serializes_with_system` in `src/providers/anthropic.rs`

#### Scenario: Chat response deserializes

- WHEN API response JSON is received
- THEN MUST deserialize correctly
- Test: `chat_response_deserializes` in `src/providers/anthropic.rs`

#### Scenario: Chat response with empty content

- WHEN response has empty content array
- THEN MUST handle gracefully
- Test: `chat_response_empty_content` in `src/providers/anthropic.rs`

#### Scenario: Chat response with multiple content blocks

- WHEN response has multiple content blocks
- THEN MUST concatenate text blocks
- Test: `chat_response_multiple_blocks` in `src/providers/anthropic.rs`

#### Scenario: Temperature range serialization

- WHEN temperature values are set
- THEN MUST serialize within valid range
- Test: `temperature_range_serializes` in `src/providers/anthropic.rs`

#### Scenario: Auth detection from JWT shape

- WHEN a token has JWT structure
- THEN MUST detect auth type correctly
- Test: `detects_auth_from_jwt_shape` in `src/providers/anthropic.rs`

#### Scenario: Cache control serialization

- WHEN cache control is set
- THEN MUST serialize correctly
- Test: `cache_control_serializes_correctly` in `src/providers/anthropic.rs`

#### Scenario: System prompt string variant serialization

- WHEN system prompt is a string
- THEN MUST serialize as string variant
- Test: `system_prompt_string_variant_serializes` in `src/providers/anthropic.rs`

#### Scenario: System prompt blocks variant serialization

- WHEN system prompt is blocks
- THEN MUST serialize as blocks variant
- Test: `system_prompt_blocks_variant_serializes` in `src/providers/anthropic.rs`

#### Scenario: System prompt blocks without cache control

- WHEN system prompt blocks lack cache control
- THEN MUST serialize without cache_control field
- Test: `system_prompt_blocks_without_cache_control` in `src/providers/anthropic.rs`

#### Scenario: Native content text without cache control

- WHEN native content text has no cache control
- THEN MUST omit cache_control
- Test: `native_content_text_without_cache_control` in `src/providers/anthropic.rs`

#### Scenario: Native content text with cache control

- WHEN native content text has cache control
- THEN MUST include cache_control
- Test: `native_content_text_with_cache_control` in `src/providers/anthropic.rs`

#### Scenario: Native content tool use without cache control

- WHEN native tool use content has no cache control
- THEN MUST omit cache_control
- Test: `native_content_tool_use_without_cache_control` in `src/providers/anthropic.rs`

#### Scenario: Native content tool result with cache control

- WHEN native tool result has cache control
- THEN MUST include cache_control
- Test: `native_content_tool_result_with_cache_control` in `src/providers/anthropic.rs`

#### Scenario: Native tool spec without cache control

- WHEN native tool spec has no cache control
- THEN MUST omit cache_control
- Test: `native_tool_spec_without_cache_control` in `src/providers/anthropic.rs`

#### Scenario: Native tool spec with cache control

- WHEN native tool spec has cache control
- THEN MUST include cache_control
- Test: `native_tool_spec_with_cache_control` in `src/providers/anthropic.rs`

#### Scenario: Should cache system small prompt

- WHEN system prompt is small
- THEN MUST NOT cache
- Test: `should_cache_system_small_prompt` in `src/providers/anthropic.rs`

#### Scenario: Should cache system large prompt

- WHEN system prompt is large
- THEN MUST cache
- Test: `should_cache_system_large_prompt` in `src/providers/anthropic.rs`

#### Scenario: Should cache system at boundary

- WHEN system prompt is at caching boundary
- THEN MUST apply boundary logic correctly
- Test: `should_cache_system_boundary` in `src/providers/anthropic.rs`

#### Scenario: Should cache conversation short

- WHEN conversation is short
- THEN MUST NOT cache
- Test: `should_cache_conversation_short` in `src/providers/anthropic.rs`

#### Scenario: Should cache conversation long

- WHEN conversation is long
- THEN MUST cache
- Test: `should_cache_conversation_long` in `src/providers/anthropic.rs`

#### Scenario: Should cache conversation at boundary

- WHEN conversation is at caching boundary
- THEN MUST apply boundary logic
- Test: `should_cache_conversation_boundary` in `src/providers/anthropic.rs`

#### Scenario: Apply cache to last message text

- WHEN cache is applied to last text message
- THEN MUST add cache_control to last text block
- Test: `apply_cache_to_last_message_text` in `src/providers/anthropic.rs`

#### Scenario: Apply cache to last message tool result

- WHEN cache is applied to last tool result message
- THEN MUST add cache_control to last tool result
- Test: `apply_cache_to_last_message_tool_result` in `src/providers/anthropic.rs`

#### Scenario: Apply cache does not affect tool use

- WHEN cache application encounters tool use
- THEN MUST NOT modify tool use blocks
- Test: `apply_cache_to_last_message_does_not_affect_tool_use` in `src/providers/anthropic.rs`

#### Scenario: Apply cache to empty messages

- WHEN cache is applied to empty message list
- THEN MUST handle gracefully
- Test: `apply_cache_empty_messages` in `src/providers/anthropic.rs`

#### Scenario: Convert tools adds cache to last tool

- WHEN tools are converted
- THEN MUST add cache_control to last tool
- Test: `convert_tools_adds_cache_to_last_tool` in `src/providers/anthropic.rs`

#### Scenario: Convert single tool gets cache

- WHEN a single tool is converted
- THEN MUST get cache_control
- Test: `convert_tools_single_tool_gets_cache` in `src/providers/anthropic.rs`

#### Scenario: Convert messages with small system prompt

- WHEN messages have small system prompt
- THEN MUST extract system without cache
- Test: `convert_messages_small_system_prompt` in `src/providers/anthropic.rs`

#### Scenario: Convert messages with large system prompt

- WHEN messages have large system prompt
- THEN MUST extract system with cache
- Test: `convert_messages_large_system_prompt` in `src/providers/anthropic.rs`

#### Scenario: Backward compatibility of native chat request

- WHEN a native chat request is constructed
- THEN MUST maintain backward compatibility
- Test: `backward_compatibility_native_chat_request` in `src/providers/anthropic.rs`

#### Scenario: Warmup without key is noop

- WHEN warmup is called without API key
- THEN MUST succeed as noop
- Test: `warmup_without_key_is_noop` in `src/providers/anthropic.rs`

#### Scenario: Convert messages preserves multi-turn history

- WHEN multi-turn conversation messages are converted
- THEN MUST preserve all turns in order
- Test: `convert_messages_preserves_multi_turn_history` in `src/providers/anthropic.rs`

#### Scenario: Chat with tools sends full history and native tools

- WHEN chat_with_tools is called with history
- THEN MUST include full history and native tool specs
- Test: `chat_with_tools_sends_full_history_and_native_tools` in `src/providers/anthropic.rs`

#### Scenario: Native response parses usage

- WHEN native response includes usage metadata
- THEN MUST parse token counts
- Test: `native_response_parses_usage` in `src/providers/anthropic.rs`

#### Scenario: Native response parses without usage

- WHEN native response lacks usage metadata
- THEN MUST return None for usage
- Test: `native_response_parses_without_usage` in `src/providers/anthropic.rs`

#### Scenario: Capabilities report vision and native tool calling

- WHEN capabilities are queried
- THEN MUST report vision and native tool calling support
- Test: `capabilities_reports_vision_and_native_tool_calling` in `src/providers/anthropic.rs`

### REQ-PROV-003: OpenAI Provider

`OpenAiProvider` MUST support GPT models with streaming, native tools, and multi-modal inputs.

#### Scenario: Creates with key

- WHEN OpenAiProvider is created with a key
- THEN MUST store the key
- Test: `creates_with_key` in `src/providers/openai.rs`

#### Scenario: Creates without key

- WHEN OpenAiProvider is created without a key
- THEN MUST create successfully with None key
- Test: `creates_without_key` in `src/providers/openai.rs`

#### Scenario: Creates with empty key

- WHEN OpenAiProvider is created with empty string key
- THEN MUST treat as no key
- Test: `creates_with_empty_key` in `src/providers/openai.rs`

#### Scenario: Chat fails without key

- WHEN chat is called without API key
- THEN MUST return error
- Test: `chat_fails_without_key` in `src/providers/openai.rs`

#### Scenario: Chat with system fails without key

- WHEN chat_with_system is called without key
- THEN MUST return error
- Test: `chat_with_system_fails_without_key` in `src/providers/openai.rs`

#### Scenario: Request serializes with system message

- WHEN request is built with system message
- THEN MUST serialize system as first message
- Test: `request_serializes_with_system_message` in `src/providers/openai.rs`

#### Scenario: Request serializes without system

- WHEN request is built without system message
- THEN MUST serialize without system message
- Test: `request_serializes_without_system` in `src/providers/openai.rs`

#### Scenario: Response deserializes single choice

- WHEN API response has one choice
- THEN MUST deserialize correctly
- Test: `response_deserializes_single_choice` in `src/providers/openai.rs`

#### Scenario: Response deserializes empty choices

- WHEN API response has empty choices
- THEN MUST handle gracefully
- Test: `response_deserializes_empty_choices` in `src/providers/openai.rs`

#### Scenario: Response deserializes multiple choices

- WHEN API response has multiple choices
- THEN MUST take first choice
- Test: `response_deserializes_multiple_choices` in `src/providers/openai.rs`

#### Scenario: Response with unicode

- WHEN response contains unicode content
- THEN MUST preserve unicode correctly
- Test: `response_with_unicode` in `src/providers/openai.rs`

#### Scenario: Response with long content

- WHEN response contains long content
- THEN MUST preserve all content
- Test: `response_with_long_content` in `src/providers/openai.rs`

#### Scenario: Warmup without key is noop

- WHEN warmup is called without API key
- THEN MUST succeed as noop
- Test: `warmup_without_key_is_noop` in `src/providers/openai.rs`

#### Scenario: Reasoning content fallback with empty content

- WHEN response content is empty but reasoning_content exists
- THEN MUST fall back to reasoning_content
- Test: `reasoning_content_fallback_empty_content` in `src/providers/openai.rs`

#### Scenario: Reasoning content fallback with null content

- WHEN response content is null but reasoning_content exists
- THEN MUST fall back to reasoning_content
- Test: `reasoning_content_fallback_null_content` in `src/providers/openai.rs`

#### Scenario: Reasoning content not used when content present

- WHEN response has both content and reasoning_content
- THEN MUST prefer content
- Test: `reasoning_content_not_used_when_content_present` in `src/providers/openai.rs`

#### Scenario: Native response reasoning content fallback

- WHEN native response uses reasoning_content fallback
- THEN MUST extract reasoning content correctly
- Test: `native_response_reasoning_content_fallback` in `src/providers/openai.rs`

#### Scenario: Native response reasoning content ignored when content present

- WHEN native response has content
- THEN MUST ignore reasoning_content
- Test: `native_response_reasoning_content_ignored_when_content_present` in `src/providers/openai.rs`

#### Scenario: Chat with tools fails without key

- WHEN chat_with_tools is called without key
- THEN MUST return error
- Test: `chat_with_tools_fails_without_key` in `src/providers/openai.rs`

#### Scenario: Chat with tools rejects invalid tool shape

- WHEN chat_with_tools receives invalid tool specification
- THEN MUST return error
- Test: `chat_with_tools_rejects_invalid_tool_shape` in `src/providers/openai.rs`

#### Scenario: Native tool spec deserializes from OpenAI format

- WHEN tool spec is in OpenAI format
- THEN MUST deserialize correctly
- Test: `native_tool_spec_deserializes_from_openai_format` in `src/providers/openai.rs`

#### Scenario: Native response parses usage

- WHEN native response includes usage metadata
- THEN MUST parse token counts
- Test: `native_response_parses_usage` in `src/providers/openai.rs`

#### Scenario: Native response parses without usage

- WHEN native response lacks usage metadata
- THEN MUST return None for usage
- Test: `native_response_parses_without_usage` in `src/providers/openai.rs`

#### Scenario: Parse native response captures reasoning content

- WHEN native response includes reasoning content
- THEN MUST capture it in parsed response
- Test: `parse_native_response_captures_reasoning_content` in `src/providers/openai.rs`

#### Scenario: Parse native response none reasoning content for normal model

- WHEN normal model response is parsed
- THEN MUST not extract reasoning content
- Test: `parse_native_response_none_reasoning_content_for_normal_model` in `src/providers/openai.rs`

#### Scenario: Convert messages round-trips reasoning content

- WHEN messages with reasoning content are converted
- THEN MUST preserve reasoning content through round-trip
- Test: `convert_messages_round_trips_reasoning_content` in `src/providers/openai.rs`

#### Scenario: Convert messages no reasoning content when absent

- WHEN messages without reasoning content are converted
- THEN MUST not inject reasoning content
- Test: `convert_messages_no_reasoning_content_when_absent` in `src/providers/openai.rs`

#### Scenario: Native message omits reasoning content when none

- WHEN native message has no reasoning content
- THEN MUST omit reasoning_content field
- Test: `native_message_omits_reasoning_content_when_none` in `src/providers/openai.rs`

#### Scenario: Native message includes reasoning content when some

- WHEN native message has reasoning content
- THEN MUST include reasoning_content field
- Test: `native_message_includes_reasoning_content_when_some` in `src/providers/openai.rs`

### REQ-PROV-004: Gemini Provider

`GeminiProvider` MUST support Google Gemini models with OAuth token refresh, streaming, and vision.

#### Scenario: Normalize non-empty trims and filters

- WHEN normalize_non_empty is called
- THEN MUST trim whitespace and filter empty strings
- Test: `normalize_non_empty_trims_and_filters` in `src/providers/gemini.rs`

#### Scenario: OAuth refresh form uses provided client credentials

- WHEN OAuth refresh form is built with client credentials
- THEN MUST include client_id and client_secret
- Test: `oauth_refresh_form_uses_provided_client_credentials` in `src/providers/gemini.rs`

#### Scenario: OAuth refresh form omits client credentials when missing

- WHEN OAuth refresh form is built without client credentials
- THEN MUST omit client_id and client_secret
- Test: `oauth_refresh_form_omits_client_credentials_when_missing` in `src/providers/gemini.rs`

#### Scenario: Extract client ID from ID token prefers aud claim

- WHEN ID token contains aud claim
- THEN MUST prefer aud as client ID
- Test: `extract_client_id_from_id_token_prefers_aud_claim` in `src/providers/gemini.rs`

#### Scenario: Extract client ID from ID token uses azp when aud missing

- WHEN ID token lacks aud but has azp
- THEN MUST use azp as client ID
- Test: `extract_client_id_from_id_token_uses_azp_when_aud_missing` in `src/providers/gemini.rs`

#### Scenario: Extract client ID returns none for invalid tokens

- WHEN ID token is invalid
- THEN MUST return None
- Test: `extract_client_id_from_id_token_returns_none_for_invalid_tokens` in `src/providers/gemini.rs`

#### Scenario: Try load CLI token derives client ID from ID token when missing

- WHEN CLI token file lacks explicit client ID but has ID token
- THEN MUST derive client ID from ID token
- Test: `try_load_cli_token_derives_client_id_from_id_token_when_missing` in `src/providers/gemini.rs`

#### Scenario: Provider creates without key

- WHEN GeminiProvider is created without key
- THEN MUST create successfully
- Test: `provider_creates_without_key` in `src/providers/gemini.rs`

#### Scenario: Provider creates with key

- WHEN GeminiProvider is created with API key
- THEN MUST store key
- Test: `provider_creates_with_key` in `src/providers/gemini.rs`

#### Scenario: Provider rejects empty key

- WHEN GeminiProvider is created with empty key
- THEN MUST reject
- Test: `provider_rejects_empty_key` in `src/providers/gemini.rs`

#### Scenario: Gemini CLI dir returns path

- WHEN gemini_cli_dir is called
- THEN MUST return a path
- Test: `gemini_cli_dir_returns_path` in `src/providers/gemini.rs`

#### Scenario: Auth source for explicit key

- WHEN explicit API key is provided
- THEN MUST identify as API key auth
- Test: `auth_source_explicit_key` in `src/providers/gemini.rs`

#### Scenario: Auth source none without credentials

- WHEN no credentials are provided
- THEN MUST return None auth source
- Test: `auth_source_none_without_credentials` in `src/providers/gemini.rs`

#### Scenario: Auth source OAuth

- WHEN OAuth credentials are provided
- THEN MUST identify as OAuth auth
- Test: `auth_source_oauth` in `src/providers/gemini.rs`

#### Scenario: Model name formatting

- WHEN model names are formatted
- THEN MUST add correct prefix
- Test: `model_name_formatting` in `src/providers/gemini.rs`

#### Scenario: API key URL includes key query param

- WHEN URL is built for API key auth
- THEN MUST include key as query parameter
- Test: `api_key_url_includes_key_query_param` in `src/providers/gemini.rs`

#### Scenario: OAuth URL uses internal endpoint

- WHEN URL is built for OAuth auth
- THEN MUST use internal endpoint
- Test: `oauth_url_uses_internal_endpoint` in `src/providers/gemini.rs`

#### Scenario: API key URL uses public endpoint

- WHEN URL is built for API key auth
- THEN MUST use public endpoint
- Test: `api_key_url_uses_public_endpoint` in `src/providers/gemini.rs`

#### Scenario: OAuth request uses Bearer auth header

- WHEN OAuth request is built
- THEN MUST set Bearer authorization header
- Test: `oauth_request_uses_bearer_auth_header` in `src/providers/gemini.rs`

#### Scenario: OAuth request wraps payload in request envelope

- WHEN OAuth request is built
- THEN MUST wrap payload in internal request envelope
- Test: `oauth_request_wraps_payload_in_request_envelope` in `src/providers/gemini.rs`

#### Scenario: API key request does not set Bearer header

- WHEN API key request is built
- THEN MUST NOT set Bearer authorization header
- Test: `api_key_request_does_not_set_bearer_header` in `src/providers/gemini.rs`

#### Scenario: Request serialization

- WHEN generate content request is serialized
- THEN MUST produce valid JSON
- Test: `request_serialization` in `src/providers/gemini.rs`

#### Scenario: Internal request includes model

- WHEN internal request is built
- THEN MUST include model field
- Test: `internal_request_includes_model` in `src/providers/gemini.rs`

#### Scenario: Internal request omits generation config when none

- WHEN generation config is not set
- THEN MUST omit generationConfig field
- Test: `internal_request_omits_generation_config_when_none` in `src/providers/gemini.rs`

#### Scenario: Internal request includes project

- WHEN internal request has project
- THEN MUST include project field
- Test: `internal_request_includes_project` in `src/providers/gemini.rs`

#### Scenario: Internal response deserialize nested

- WHEN internal response JSON is nested
- THEN MUST deserialize correctly
- Test: `internal_response_deserialize_nested` in `src/providers/gemini.rs`

#### Scenario: Credentials deserialize with expiry date

- WHEN OAuth credentials include expiry date
- THEN MUST deserialize expiry
- Test: `creds_deserialize_with_expiry_date` in `src/providers/gemini.rs`

#### Scenario: Credentials deserialize accepts camelCase fields

- WHEN OAuth credentials use camelCase
- THEN MUST accept both naming conventions
- Test: `creds_deserialize_accepts_camel_case_fields` in `src/providers/gemini.rs`

#### Scenario: OAuth retry detection for generation config rejection

- WHEN OAuth request fails due to generation config rejection
- THEN MUST detect retry opportunity
- Test: `oauth_retry_detection_for_generation_config_rejection` in `src/providers/gemini.rs`

#### Scenario: Response deserialization

- WHEN Gemini API response is received
- THEN MUST deserialize correctly
- Test: `response_deserialization` in `src/providers/gemini.rs`

#### Scenario: Error response deserialization

- WHEN Gemini API error response is received
- THEN MUST deserialize error details
- Test: `error_response_deserialization` in `src/providers/gemini.rs`

#### Scenario: Internal response deserialization

- WHEN internal API response is received
- THEN MUST deserialize correctly
- Test: `internal_response_deserialization` in `src/providers/gemini.rs`

#### Scenario: Thinking response extracts non-thinking text

- WHEN response contains thinking and non-thinking parts
- THEN MUST extract non-thinking text
- Test: `thinking_response_extracts_non_thinking_text` in `src/providers/gemini.rs`

#### Scenario: Non-thinking response unaffected

- WHEN response has no thinking parts
- THEN MUST return text unmodified
- Test: `non_thinking_response_unaffected` in `src/providers/gemini.rs`

#### Scenario: Thinking-only response falls back to thinking text

- WHEN response has only thinking parts
- THEN MUST fall back to thinking text
- Test: `thinking_only_response_falls_back_to_thinking_text` in `src/providers/gemini.rs`

#### Scenario: Empty parts returns none

- WHEN response parts are empty
- THEN MUST return None
- Test: `empty_parts_returns_none` in `src/providers/gemini.rs`

#### Scenario: Multiple text parts concatenated

- WHEN response has multiple text parts
- THEN MUST concatenate them
- Test: `multiple_text_parts_concatenated` in `src/providers/gemini.rs`

#### Scenario: Thought signature only parts skipped

- WHEN response has only thought-signature parts
- THEN MUST skip them
- Test: `thought_signature_only_parts_skipped` in `src/providers/gemini.rs`

#### Scenario: Internal response thinking model

- WHEN internal response is from thinking model
- THEN MUST handle thinking parts
- Test: `internal_response_thinking_model` in `src/providers/gemini.rs`

#### Scenario: Warmup without key is noop

- WHEN warmup is called without key
- THEN MUST succeed as noop
- Test: `warmup_without_key_is_noop` in `src/providers/gemini.rs`

#### Scenario: Warmup OAuth is noop

- WHEN warmup is called with OAuth auth
- THEN MUST succeed as noop
- Test: `warmup_oauth_is_noop` in `src/providers/gemini.rs`

#### Scenario: Discover OAuth cred paths does not panic

- WHEN discover_oauth_cred_paths is called
- THEN MUST not panic
- Test: `discover_oauth_cred_paths_does_not_panic` in `src/providers/gemini.rs`

#### Scenario: Rotate OAuth without alternatives returns false

- WHEN OAuth rotation has no alternatives
- THEN MUST return false
- Test: `rotate_oauth_without_alternatives_returns_false` in `src/providers/gemini.rs`

#### Scenario: Response parses usage metadata

- WHEN response includes usage metadata
- THEN MUST parse token counts
- Test: `response_parses_usage_metadata` in `src/providers/gemini.rs`

#### Scenario: Response parses without usage metadata

- WHEN response lacks usage metadata
- THEN MUST return None for usage
- Test: `response_parses_without_usage_metadata` in `src/providers/gemini.rs`

#### Scenario: Warmup managed OAuth requires auth service

- WHEN warmup is called with managed OAuth
- THEN MUST require auth service
- Test: `warmup_managed_oauth_requires_auth_service` in `src/providers/gemini.rs`

#### Scenario: Warmup CLI OAuth skips validation

- WHEN warmup is called with CLI OAuth
- THEN MUST skip validation
- Test: `warmup_cli_oauth_skips_validation` in `src/providers/gemini.rs`

### REQ-PROV-005: AWS Bedrock Provider

`AwsBedrockProvider` MUST support Bedrock models with AWS SigV4 authentication and streaming.

#### Scenario: SHA256 hex of empty string

- WHEN sha256_hex is called with empty string
- THEN MUST produce correct hash
- Test: `sha256_hex_empty_string` in `src/providers/bedrock.rs`

#### Scenario: SHA256 hex of known input

- WHEN sha256_hex is called with known input
- THEN MUST produce expected hash
- Test: `sha256_hex_known_input` in `src/providers/bedrock.rs`

#### Scenario: HMAC-SHA256 known input

- WHEN hmac_sha256 is called with known input
- THEN MUST produce expected MAC
- Test: `hmac_sha256_known_input` in `src/providers/bedrock.rs`

#### Scenario: Derive signing key structure

- WHEN signing key is derived
- THEN MUST produce correct key structure
- Test: `derive_signing_key_structure` in `src/providers/bedrock.rs`

#### Scenario: Derive signing key known test vector

- WHEN signing key is derived with known inputs
- THEN MUST match AWS test vector
- Test: `derive_signing_key_known_test_vector` in `src/providers/bedrock.rs`

#### Scenario: Build authorization header format

- WHEN authorization header is built
- THEN MUST follow AWS SigV4 format
- Test: `build_authorization_header_format` in `src/providers/bedrock.rs`

#### Scenario: Build authorization header includes security token in signed headers

- WHEN security token is present
- THEN MUST include x-amz-security-token in signed headers
- Test: `build_authorization_header_includes_security_token_in_signed_headers` in `src/providers/bedrock.rs`

#### Scenario: Credentials host formats correctly

- WHEN credentials host is computed
- THEN MUST format region correctly
- Test: `credentials_host_formats_correctly` in `src/providers/bedrock.rs`

#### Scenario: Creates without credentials

- WHEN BedrockProvider is created without credentials
- THEN MUST create successfully
- Test: `creates_without_credentials` in `src/providers/bedrock.rs`

#### Scenario: Chat fails without credentials

- WHEN chat is called without credentials
- THEN MUST return error
- Test: `chat_fails_without_credentials` in `src/providers/bedrock.rs`

#### Scenario: Endpoint URL formats correctly

- WHEN endpoint URL is computed
- THEN MUST format region and model correctly
- Test: `endpoint_url_formats_correctly` in `src/providers/bedrock.rs`

#### Scenario: Endpoint URL keeps raw colon

- WHEN model ID contains colon
- THEN endpoint URL MUST keep raw colon
- Test: `endpoint_url_keeps_raw_colon` in `src/providers/bedrock.rs`

#### Scenario: Canonical URI encodes colon

- WHEN model ID contains colon
- THEN canonical URI MUST encode colon
- Test: `canonical_uri_encodes_colon` in `src/providers/bedrock.rs`

#### Scenario: Canonical URI no colon unchanged

- WHEN model ID has no colon
- THEN canonical URI MUST be unchanged
- Test: `canonical_uri_no_colon_unchanged` in `src/providers/bedrock.rs`

#### Scenario: Convert messages extracts system

- WHEN messages are converted and first is system
- THEN MUST extract system prompt
- Test: `convert_messages_system_extracted` in `src/providers/bedrock.rs`

#### Scenario: Convert messages user and assistant

- WHEN messages are converted with user and assistant
- THEN MUST map roles correctly
- Test: `convert_messages_user_and_assistant` in `src/providers/bedrock.rs`

#### Scenario: Convert messages tool role to tool result

- WHEN tool role message is converted
- THEN MUST map to tool result block
- Test: `convert_messages_tool_role_to_tool_result` in `src/providers/bedrock.rs`

#### Scenario: Convert messages assistant tool calls parsed

- WHEN assistant message with tool calls is converted
- THEN MUST parse tool call blocks
- Test: `convert_messages_assistant_tool_calls_parsed` in `src/providers/bedrock.rs`

#### Scenario: Convert messages plain assistant text

- WHEN plain assistant text is converted
- THEN MUST produce text block
- Test: `convert_messages_plain_assistant_text` in `src/providers/bedrock.rs`

#### Scenario: Should cache system small prompt

- WHEN system prompt is small
- THEN MUST NOT cache
- Test: `should_cache_system_small_prompt` in `src/providers/bedrock.rs`

#### Scenario: Should cache system large prompt

- WHEN system prompt is large
- THEN MUST cache
- Test: `should_cache_system_large_prompt` in `src/providers/bedrock.rs`

#### Scenario: Should cache system at boundary

- WHEN system prompt is at boundary
- THEN MUST apply boundary logic
- Test: `should_cache_system_boundary` in `src/providers/bedrock.rs`

#### Scenario: Should cache conversation short

- WHEN conversation is short
- THEN MUST NOT cache
- Test: `should_cache_conversation_short` in `src/providers/bedrock.rs`

#### Scenario: Should cache conversation long

- WHEN conversation is long
- THEN MUST cache
- Test: `should_cache_conversation_long` in `src/providers/bedrock.rs`

#### Scenario: Convert tools to converse formats correctly

- WHEN tools are converted to Converse format
- THEN MUST produce valid tool config
- Test: `convert_tools_to_converse_formats_correctly` in `src/providers/bedrock.rs`

#### Scenario: Convert tools to converse empty returns none

- WHEN empty tool list is converted
- THEN MUST return None
- Test: `convert_tools_to_converse_empty_returns_none` in `src/providers/bedrock.rs`

#### Scenario: Converse request serializes without system

- WHEN converse request is built without system
- THEN MUST serialize correctly
- Test: `converse_request_serializes_without_system` in `src/providers/bedrock.rs`

#### Scenario: Converse response deserializes text

- WHEN converse response has text output
- THEN MUST deserialize text
- Test: `converse_response_deserializes_text` in `src/providers/bedrock.rs`

#### Scenario: Converse response deserializes tool use

- WHEN converse response has tool use output
- THEN MUST deserialize tool use
- Test: `converse_response_deserializes_tool_use` in `src/providers/bedrock.rs`

#### Scenario: Converse response empty output

- WHEN converse response has empty output
- THEN MUST handle gracefully
- Test: `converse_response_empty_output` in `src/providers/bedrock.rs`

#### Scenario: Content block text serializes as flat string

- WHEN text content block is serialized
- THEN MUST produce flat string
- Test: `content_block_text_serializes_as_flat_string` in `src/providers/bedrock.rs`

#### Scenario: Content block tool use serializes with nested object

- WHEN tool use content block is serialized
- THEN MUST produce nested object
- Test: `content_block_tool_use_serializes_with_nested_object` in `src/providers/bedrock.rs`

#### Scenario: Content block cache point serializes

- WHEN cache point content block is serialized
- THEN MUST serialize correctly
- Test: `content_block_cache_point_serializes` in `src/providers/bedrock.rs`

#### Scenario: Content block text round trips

- WHEN text content block is round-tripped through serde
- THEN MUST produce identical output
- Test: `content_block_text_round_trips` in `src/providers/bedrock.rs`

#### Scenario: Cache point serializes

- WHEN cache point is serialized
- THEN MUST produce correct JSON
- Test: `cache_point_serializes` in `src/providers/bedrock.rs`

#### Scenario: Warmup without credentials is noop

- WHEN warmup is called without credentials
- THEN MUST succeed as noop
- Test: `warmup_without_credentials_is_noop` in `src/providers/bedrock.rs`

#### Scenario: Capabilities reports native tool calling

- WHEN capabilities are queried
- THEN MUST report native tool calling
- Test: `capabilities_reports_native_tool_calling` in `src/providers/bedrock.rs`

#### Scenario: Converse response parses usage

- WHEN converse response includes usage metadata
- THEN MUST parse token counts
- Test: `converse_response_parses_usage` in `src/providers/bedrock.rs`

#### Scenario: Converse response parses without usage

- WHEN converse response lacks usage metadata
- THEN MUST return None
- Test: `converse_response_parses_without_usage` in `src/providers/bedrock.rs`

#### Scenario: Fallback tool result emits tool result block not text

- WHEN fallback tool result is created
- THEN MUST emit tool_result block not text
- Test: `fallback_tool_result_emits_tool_result_block_not_text` in `src/providers/bedrock.rs`

#### Scenario: Supports streaming returns true

- WHEN supports_streaming is checked
- THEN MUST return true
- Test: `supports_streaming_returns_true` in `src/providers/bedrock.rs`

#### Scenario: Stream endpoint URL formats correctly

- WHEN stream endpoint URL is computed
- THEN MUST format correctly
- Test: `stream_endpoint_url_formats_correctly` in `src/providers/bedrock.rs`

#### Scenario: Fallback recovers tool use ID from assistant

- WHEN tool result has no ID but assistant has pending tool use
- THEN MUST recover tool use ID from assistant
- Test: `fallback_recovers_tool_use_id_from_assistant` in `src/providers/bedrock.rs`

#### Scenario: Consecutive tool results merged into single message

- WHEN consecutive tool result messages are converted
- THEN MUST merge into single message
- Test: `consecutive_tool_results_merged_into_single_message` in `src/providers/bedrock.rs`

#### Scenario: Extract tool call ID tries multiple field names

- WHEN tool call ID is extracted
- THEN MUST try multiple field names
- Test: `extract_tool_call_id_tries_multiple_field_names` in `src/providers/bedrock.rs`

#### Scenario: Stream canonical URI encodes colon

- WHEN stream canonical URI has colon in model
- THEN MUST encode colon
- Test: `stream_canonical_uri_encodes_colon` in `src/providers/bedrock.rs`

#### Scenario: Parse tool result accepts alternate ID fields

- WHEN tool result has alternate ID field names
- THEN MUST accept them
- Test: `parse_tool_result_accepts_alternate_id_fields` in `src/providers/bedrock.rs`

#### Scenario: Stream canonical URI no colon

- WHEN stream canonical URI has no colon
- THEN MUST be unchanged
- Test: `stream_canonical_uri_no_colon` in `src/providers/bedrock.rs`

#### Scenario: Parse event stream message content block delta

- WHEN event stream contains content block delta
- THEN MUST parse event correctly
- Test: `parse_event_stream_message_content_block_delta` in `src/providers/bedrock.rs`

#### Scenario: Parse event stream message stop

- WHEN event stream contains stop event
- THEN MUST parse stop correctly
- Test: `parse_event_stream_message_stop` in `src/providers/bedrock.rs`

#### Scenario: Parse event stream message insufficient data

- WHEN event stream has insufficient data
- THEN MUST return None
- Test: `parse_event_stream_message_insufficient_data` in `src/providers/bedrock.rs`

#### Scenario: Parse event stream message incomplete message

- WHEN event stream has incomplete message
- THEN MUST return None
- Test: `parse_event_stream_message_incomplete_message` in `src/providers/bedrock.rs`

#### Scenario: Parse event stream multiple messages

- WHEN event stream has multiple messages
- THEN MUST parse all messages
- Test: `parse_event_stream_multiple_messages` in `src/providers/bedrock.rs`

#### Scenario: Content block delta deserializes

- WHEN content block delta JSON is received
- THEN MUST deserialize correctly
- Test: `content_block_delta_deserializes` in `src/providers/bedrock.rs`

#### Scenario: Content block delta empty text

- WHEN content block delta has empty text
- THEN MUST handle gracefully
- Test: `content_block_delta_empty_text` in `src/providers/bedrock.rs`

### REQ-PROV-006: Ollama Provider

`OllamaProvider` MUST support local Ollama models with streaming and native tool calling.

#### Scenario: Default URL

- WHEN OllamaProvider is created without custom URL
- THEN MUST use default localhost URL
- Test: `default_url` in `src/providers/ollama.rs`

#### Scenario: Custom URL trailing slash

- WHEN custom URL has trailing slash
- THEN MUST trim trailing slash
- Test: `custom_url_trailing_slash` in `src/providers/ollama.rs`

#### Scenario: Custom URL no trailing slash

- WHEN custom URL has no trailing slash
- THEN MUST use as-is
- Test: `custom_url_no_trailing_slash` in `src/providers/ollama.rs`

#### Scenario: Custom URL strips api suffix

- WHEN custom URL ends with /api
- THEN MUST strip the suffix
- Test: `custom_url_strips_api_suffix` in `src/providers/ollama.rs`

#### Scenario: Empty URL uses empty

- WHEN URL is empty
- THEN MUST use empty string
- Test: `empty_url_uses_empty` in `src/providers/ollama.rs`

#### Scenario: Cloud suffix strips model name

- WHEN cloud suffix is used
- THEN MUST strip model name
- Test: `cloud_suffix_strips_model_name` in `src/providers/ollama.rs`

#### Scenario: Cloud suffix with local endpoint errors

- WHEN cloud suffix is used with local endpoint
- THEN MUST return error
- Test: `cloud_suffix_with_local_endpoint_errors` in `src/providers/ollama.rs`

#### Scenario: Cloud suffix without API key errors

- WHEN cloud suffix is used without API key
- THEN MUST return error
- Test: `cloud_suffix_without_api_key_errors` in `src/providers/ollama.rs`

#### Scenario: Remote endpoint auth enabled when key present

- WHEN remote endpoint has API key
- THEN MUST enable auth
- Test: `remote_endpoint_auth_enabled_when_key_present` in `src/providers/ollama.rs`

#### Scenario: Remote endpoint with api suffix still allows cloud models

- WHEN remote endpoint has /api suffix
- THEN MUST still allow cloud models
- Test: `remote_endpoint_with_api_suffix_still_allows_cloud_models` in `src/providers/ollama.rs`

#### Scenario: Local endpoint auth disabled even with key

- WHEN local endpoint has API key
- THEN MUST disable auth
- Test: `local_endpoint_auth_disabled_even_with_key` in `src/providers/ollama.rs`

#### Scenario: Request omits think when reasoning not configured

- WHEN reasoning is not configured
- THEN MUST omit think parameter
- Test: `request_omits_think_when_reasoning_not_configured` in `src/providers/ollama.rs`

#### Scenario: Request includes think when reasoning configured

- WHEN reasoning is configured
- THEN MUST include think parameter
- Test: `request_includes_think_when_reasoning_configured` in `src/providers/ollama.rs`

#### Scenario: Response deserializes

- WHEN API response is received
- THEN MUST deserialize correctly
- Test: `response_deserializes` in `src/providers/ollama.rs`

#### Scenario: Response with empty content

- WHEN response has empty content
- THEN MUST handle gracefully
- Test: `response_with_empty_content` in `src/providers/ollama.rs`

#### Scenario: Normalize response text rejects whitespace only content

- WHEN response text is whitespace only
- THEN MUST reject as empty
- Test: `normalize_response_text_rejects_whitespace_only_content` in `src/providers/ollama.rs`

#### Scenario: Fallback text for empty content without thinking is generic

- WHEN content is empty and no thinking content
- THEN MUST produce generic fallback text
- Test: `fallback_text_for_empty_content_without_thinking_is_generic` in `src/providers/ollama.rs`

#### Scenario: Response with missing content defaults to empty

- WHEN response has missing content field
- THEN MUST default to empty
- Test: `response_with_missing_content_defaults_to_empty` in `src/providers/ollama.rs`

#### Scenario: Response with thinking field extracts content

- WHEN response has thinking field
- THEN MUST extract thinking content
- Test: `response_with_thinking_field_extracts_content` in `src/providers/ollama.rs`

#### Scenario: Response with tool calls parses correctly

- WHEN response has tool_calls
- THEN MUST parse tool calls
- Test: `response_with_tool_calls_parses_correctly` in `src/providers/ollama.rs`

#### Scenario: Extract tool name handles nested tool call

- WHEN tool call has nested structure
- THEN MUST extract tool name correctly
- Test: `extract_tool_name_handles_nested_tool_call` in `src/providers/ollama.rs`

#### Scenario: Extract tool name handles prefixed name

- WHEN tool name is prefixed
- THEN MUST handle prefix correctly
- Test: `extract_tool_name_handles_prefixed_name` in `src/providers/ollama.rs`

#### Scenario: Extract tool name handles normal call

- WHEN tool call is normal format
- THEN MUST extract name directly
- Test: `extract_tool_name_handles_normal_call` in `src/providers/ollama.rs`

#### Scenario: Format tool calls produces valid JSON

- WHEN tool calls are formatted for tool loop
- THEN MUST produce valid JSON
- Test: `format_tool_calls_produces_valid_json` in `src/providers/ollama.rs`

#### Scenario: Convert messages parses native assistant tool calls

- WHEN assistant messages with tool calls are converted
- THEN MUST parse native tool call format
- Test: `convert_messages_parses_native_assistant_tool_calls` in `src/providers/ollama.rs`

#### Scenario: Convert messages maps tool result call ID to tool name

- WHEN tool result messages are converted
- THEN MUST map call_id to tool name
- Test: `convert_messages_maps_tool_result_call_id_to_tool_name` in `src/providers/ollama.rs`

#### Scenario: Convert messages extracts images from user marker

- WHEN user messages contain image markers
- THEN MUST extract images
- Test: `convert_messages_extracts_images_from_user_marker` in `src/providers/ollama.rs`

#### Scenario: Capabilities include native tools and vision

- WHEN capabilities are queried
- THEN MUST include native tools and vision
- Test: `capabilities_include_native_tools_and_vision` in `src/providers/ollama.rs`

#### Scenario: API response parses eval counts

- WHEN API response includes eval counts
- THEN MUST parse token usage
- Test: `api_response_parses_eval_counts` in `src/providers/ollama.rs`

#### Scenario: API response parses without eval counts

- WHEN API response lacks eval counts
- THEN MUST return None usage
- Test: `api_response_parses_without_eval_counts` in `src/providers/ollama.rs`

### REQ-PROV-007: OpenAI-Compatible Provider

`OpenAiCompatibleProvider` MUST support any OpenAI-compatible API with configurable endpoints and auth styles.

#### Scenario: Creates with key

- WHEN OpenAiCompatibleProvider is created with key
- THEN MUST store key and base URL
- Test: `creates_with_key` in `src/providers/compatible.rs`

#### Scenario: Creates without key

- WHEN OpenAiCompatibleProvider is created without key
- THEN MUST create successfully
- Test: `creates_without_key` in `src/providers/compatible.rs`

#### Scenario: Strips trailing slash

- WHEN base URL has trailing slash
- THEN MUST strip it
- Test: `strips_trailing_slash` in `src/providers/compatible.rs`

#### Scenario: Chat fails without key

- WHEN chat is called without API key
- THEN MUST return error
- Test: `chat_fails_without_key` in `src/providers/compatible.rs`

#### Scenario: Request serializes correctly

- WHEN chat request is built
- THEN MUST serialize in OpenAI-compatible format
- Test: `request_serializes_correctly` in `src/providers/compatible.rs`

#### Scenario: Response deserializes

- WHEN API response is received
- THEN MUST deserialize correctly
- Test: `response_deserializes` in `src/providers/compatible.rs`

#### Scenario: Response empty choices

- WHEN response has empty choices
- THEN MUST handle gracefully
- Test: `response_empty_choices` in `src/providers/compatible.rs`

#### Scenario: Parse chat response body reports sanitized snippet

- WHEN chat response body fails to parse
- THEN MUST report sanitized snippet in error
- Test: `parse_chat_response_body_reports_sanitized_snippet` in `src/providers/compatible.rs`

#### Scenario: Parse responses response body reports sanitized snippet

- WHEN responses body fails to parse
- THEN MUST report sanitized snippet in error
- Test: `parse_responses_response_body_reports_sanitized_snippet` in `src/providers/compatible.rs`

#### Scenario: X-API-Key auth style

- WHEN x-api-key auth style is configured
- THEN MUST use x-api-key header
- Test: `x_api_key_auth_style` in `src/providers/compatible.rs`

#### Scenario: Custom auth style

- WHEN custom auth style is configured
- THEN MUST use custom header
- Test: `custom_auth_style` in `src/providers/compatible.rs`

#### Scenario: Custom constructor applies responses mode and max tokens override

- WHEN provider is constructed with custom options
- THEN MUST apply responses mode and max tokens override
- Test: `custom_constructor_applies_responses_mode_and_max_tokens_override` in `src/providers/compatible.rs`

#### Scenario: All compatible providers fail without key

- WHEN any compatible provider calls chat without key
- THEN MUST return error
- Test: `all_compatible_providers_fail_without_key` in `src/providers/compatible.rs`

#### Scenario: Responses extracts top-level output text

- WHEN responses API returns top-level output_text
- THEN MUST extract it
- Test: `responses_extracts_top_level_output_text` in `src/providers/compatible.rs`

#### Scenario: Responses extracts nested output text

- WHEN responses API returns nested output text
- THEN MUST extract it
- Test: `responses_extracts_nested_output_text` in `src/providers/compatible.rs`

#### Scenario: Responses extracts any text as fallback

- WHEN responses API returns text in non-standard location
- THEN MUST extract as fallback
- Test: `responses_extracts_any_text_as_fallback` in `src/providers/compatible.rs`

#### Scenario: Responses extracts function call as tool call

- WHEN responses API returns function_call output
- THEN MUST extract as tool call
- Test: `responses_extracts_function_call_as_tool_call` in `src/providers/compatible.rs`

#### Scenario: WebSocket URL converts scheme and adds model query

- WHEN WebSocket URL is constructed
- THEN MUST convert scheme and add model query
- Test: `websocket_url_converts_scheme_and_adds_model_query` in `src/providers/compatible.rs`

#### Scenario: WebSocket URL preserves existing model query

- WHEN WebSocket URL already has model query
- THEN MUST preserve it
- Test: `websocket_url_preserves_existing_model_query` in `src/providers/compatible.rs`

#### Scenario: WebSocket accumulator parses delta and completed event

- WHEN WebSocket events are accumulated
- THEN MUST parse delta and completed events
- Test: `websocket_accumulator_parses_delta_and_completed_event` in `src/providers/compatible.rs`

#### Scenario: WebSocket accumulator falls back to output items

- WHEN WebSocket completed event lacks text
- THEN MUST fall back to output items
- Test: `websocket_accumulator_falls_back_to_output_items` in `src/providers/compatible.rs`

#### Scenario: WebSocket accumulator reports stream error

- WHEN WebSocket stream has error event
- THEN MUST report error
- Test: `websocket_accumulator_reports_stream_error` in `src/providers/compatible.rs`

#### Scenario: Build responses prompt preserves multi-turn history

- WHEN responses prompt is built from multi-turn history
- THEN MUST preserve all turns
- Test: `build_responses_prompt_preserves_multi_turn_history` in `src/providers/compatible.rs`

#### Scenario: Chat via responses requires non-system message

- WHEN chat via responses has only system message
- THEN MUST require non-system message
- Test: `chat_via_responses_requires_non_system_message` in `src/providers/compatible.rs`

#### Scenario: Tool call function name falls back to top-level name

- WHEN tool call lacks nested function name
- THEN MUST fall back to top-level name
- Test: `tool_call_function_name_falls_back_to_top_level_name` in `src/providers/compatible.rs`

#### Scenario: Tool call function arguments falls back to parameters object

- WHEN tool call lacks nested function arguments
- THEN MUST fall back to parameters object
- Test: `tool_call_function_arguments_falls_back_to_parameters_object` in `src/providers/compatible.rs`

#### Scenario: Tool call function arguments prefers nested function field

- WHEN tool call has nested function arguments
- THEN MUST prefer nested function field
- Test: `tool_call_function_arguments_prefers_nested_function_field` in `src/providers/compatible.rs`

#### Scenario: Chat completions URL standard OpenAI

- WHEN standard OpenAI base URL is used
- THEN MUST produce correct chat completions URL
- Test: `chat_completions_url_standard_openai` in `src/providers/compatible.rs`

#### Scenario: Chat completions URL trailing slash

- WHEN base URL has trailing slash
- THEN MUST produce correct URL
- Test: `chat_completions_url_trailing_slash` in `src/providers/compatible.rs`

#### Scenario: Chat completions URL Volcengine ARK

- WHEN Volcengine ARK URL is used
- THEN MUST produce correct URL
- Test: `chat_completions_url_volcengine_ark` in `src/providers/compatible.rs`

#### Scenario: Chat completions URL custom full endpoint

- WHEN custom full endpoint is provided
- THEN MUST use as-is
- Test: `chat_completions_url_custom_full_endpoint` in `src/providers/compatible.rs`

#### Scenario: Chat completions URL requires exact suffix match

- WHEN URL has partial suffix match
- THEN MUST require exact match
- Test: `chat_completions_url_requires_exact_suffix_match` in `src/providers/compatible.rs`

#### Scenario: Responses URL standard

- WHEN standard base URL is used for responses
- THEN MUST produce correct responses URL
- Test: `responses_url_standard` in `src/providers/compatible.rs`

#### Scenario: Responses URL custom full endpoint

- WHEN custom full endpoint is provided for responses
- THEN MUST use as-is
- Test: `responses_url_custom_full_endpoint` in `src/providers/compatible.rs`

#### Scenario: Responses URL requires exact suffix match

- WHEN URL has partial suffix match for responses
- THEN MUST require exact match
- Test: `responses_url_requires_exact_suffix_match` in `src/providers/compatible.rs`

#### Scenario: Responses URL derives from chat endpoint

- WHEN responses URL is derived from chat endpoint
- THEN MUST derive correctly
- Test: `responses_url_derives_from_chat_endpoint` in `src/providers/compatible.rs`

#### Scenario: Responses URL base with v1 no duplicate

- WHEN base URL already has /v1
- THEN MUST not duplicate
- Test: `responses_url_base_with_v1_no_duplicate` in `src/providers/compatible.rs`

#### Scenario: Responses URL non-v1 API path uses raw suffix

- WHEN URL has non-v1 API path
- THEN MUST use raw suffix
- Test: `responses_url_non_v1_api_path_uses_raw_suffix` in `src/providers/compatible.rs`

#### Scenario: Chat completions URL without v1

- WHEN base URL lacks /v1
- THEN MUST add /v1 prefix
- Test: `chat_completions_url_without_v1` in `src/providers/compatible.rs`

#### Scenario: Chat completions URL base with v1

- WHEN base URL already has /v1
- THEN MUST not duplicate
- Test: `chat_completions_url_base_with_v1` in `src/providers/compatible.rs`

#### Scenario: Chat completions URL ZAI

- WHEN ZAI provider URL is used
- THEN MUST produce correct URL
- Test: `chat_completions_url_zai` in `src/providers/compatible.rs`

#### Scenario: Chat completions URL MiniMax

- WHEN MiniMax provider URL is used
- THEN MUST produce correct URL
- Test: `chat_completions_url_minimax` in `src/providers/compatible.rs`

#### Scenario: Chat completions URL GLM

- WHEN GLM provider URL is used
- THEN MUST produce correct URL
- Test: `chat_completions_url_glm` in `src/providers/compatible.rs`

#### Scenario: Chat completions URL OpenCode

- WHEN OpenCode provider URL is used
- THEN MUST produce correct URL
- Test: `chat_completions_url_opencode` in `src/providers/compatible.rs`

#### Scenario: Parse native response preserves tool call ID

- WHEN native response has tool call with ID
- THEN MUST preserve tool call ID
- Test: `parse_native_response_preserves_tool_call_id` in `src/providers/compatible.rs`

#### Scenario: Convert messages for native maps tool result payload

- WHEN messages with tool results are converted
- THEN MUST map tool result payload
- Test: `convert_messages_for_native_maps_tool_result_payload` in `src/providers/compatible.rs`

#### Scenario: Convert messages for native keeps user image markers as text when disabled

- WHEN image parts are disabled and user has image markers
- THEN MUST keep markers as text
- Test: `convert_messages_for_native_keeps_user_image_markers_as_text_when_disabled` in `src/providers/compatible.rs`

#### Scenario: Flatten system messages merges into first user

- WHEN system messages are present with user message
- THEN MUST merge into first user message
- Test: `flatten_system_messages_merges_into_first_user` in `src/providers/compatible.rs`

#### Scenario: Flatten system messages inserts user when missing

- WHEN system messages are present without user message
- THEN MUST insert synthetic user message
- Test: `flatten_system_messages_inserts_user_when_missing` in `src/providers/compatible.rs`

#### Scenario: Strip think tags drops unclosed block suffix

- WHEN content has unclosed think tag
- THEN MUST drop suffix
- Test: `strip_think_tags_drops_unclosed_block_suffix` in `src/providers/compatible.rs`

#### Scenario: Native tool schema unsupported detection is precise

- WHEN HTTP 516 error occurs with specific hint
- THEN MUST detect native tool schema rejection precisely
- Test: `native_tool_schema_unsupported_detection_is_precise` in `src/providers/compatible.rs`

#### Scenario: Prompt-guided tool fallback injects system instruction

- WHEN prompt-guided tool fallback is used
- THEN MUST inject system instruction
- Test: `prompt_guided_tool_fallback_injects_system_instruction` in `src/providers/compatible.rs`

#### Scenario: Warmup without key is noop

- WHEN warmup is called without key
- THEN MUST succeed as noop
- Test: `warmup_without_key_is_noop` in `src/providers/compatible.rs`

#### Scenario: Capabilities reports native tool calling

- WHEN capabilities are queried
- THEN MUST report native tool calling
- Test: `capabilities_reports_native_tool_calling` in `src/providers/compatible.rs`

#### Scenario: Capabilities reports vision for Qwen compatible provider

- WHEN Qwen compatible provider capabilities are queried
- THEN MUST report vision support
- Test: `capabilities_reports_vision_for_qwen_compatible_provider` in `src/providers/compatible.rs`

#### Scenario: MiniMax provider disables native tool calling

- WHEN MiniMax compatible provider is created
- THEN MUST disable native tool calling
- Test: `minimax_provider_disables_native_tool_calling` in `src/providers/compatible.rs`

#### Scenario: User agent constructor keeps native tool calling enabled

- WHEN user agent constructor is used
- THEN MUST keep native tool calling enabled
- Test: `user_agent_constructor_keeps_native_tool_calling_enabled` in `src/providers/compatible.rs`

#### Scenario: User agent and vision constructor preserves capability flags

- WHEN user agent and vision constructor is used
- THEN MUST preserve both capability flags
- Test: `user_agent_and_vision_constructor_preserves_capability_flags` in `src/providers/compatible.rs`

#### Scenario: No responses fallback constructor keeps native tool calling enabled

- WHEN no-responses-fallback constructor is used
- THEN MUST keep native tool calling enabled
- Test: `no_responses_fallback_constructor_keeps_native_tool_calling_enabled` in `src/providers/compatible.rs`

#### Scenario: To message content converts image markers to OpenAI parts

- WHEN user content has image markers
- THEN MUST convert to OpenAI image parts
- Test: `to_message_content_converts_image_markers_to_openai_parts` in `src/providers/compatible.rs`

#### Scenario: To message content keeps markers as text when user image parts disabled

- WHEN user image parts are disabled
- THEN MUST keep image markers as text
- Test: `to_message_content_keeps_markers_as_text_when_user_image_parts_disabled` in `src/providers/compatible.rs`

#### Scenario: To message content keeps plain text for non-user roles

- WHEN content is from non-user role
- THEN MUST keep as plain text
- Test: `to_message_content_keeps_plain_text_for_non_user_roles` in `src/providers/compatible.rs`

#### Scenario: Tool specs convert to OpenAI format

- WHEN tool specs are converted
- THEN MUST produce OpenAI format
- Test: `tool_specs_convert_to_openai_format` in `src/providers/compatible.rs`

#### Scenario: OpenAI tools convert back to tool specs for prompt fallback

- WHEN OpenAI tools are converted back
- THEN MUST produce valid tool specs for prompt fallback
- Test: `openai_tools_convert_back_to_tool_specs_for_prompt_fallback` in `src/providers/compatible.rs`

#### Scenario: Request serializes with tools

- WHEN request is built with tools
- THEN MUST serialize tools in request
- Test: `request_serializes_with_tools` in `src/providers/compatible.rs`

#### Scenario: Response with tool calls deserializes

- WHEN response has tool calls
- THEN MUST deserialize tool calls
- Test: `response_with_tool_calls_deserializes` in `src/providers/compatible.rs`

#### Scenario: Response with multiple tool calls

- WHEN response has multiple tool calls
- THEN MUST deserialize all tool calls
- Test: `response_with_multiple_tool_calls` in `src/providers/compatible.rs`

#### Scenario: Chat with tools fails without key

- WHEN chat_with_tools is called without key
- THEN MUST return error
- Test: `chat_with_tools_fails_without_key` in `src/providers/compatible.rs`

#### Scenario: Chat with tools falls back on HTTP 516 tool schema error

- WHEN chat_with_tools gets HTTP 516 with schema hint
- THEN MUST fall back to prompt-guided tools
- Test: `chat_with_tools_falls_back_on_http_516_tool_schema_error` in `src/providers/compatible.rs`

#### Scenario: Chat falls back on HTTP 516 tool schema error

- WHEN chat gets HTTP 516 with schema hint
- THEN MUST fall back to prompt-guided tools
- Test: `chat_falls_back_on_http_516_tool_schema_error` in `src/providers/compatible.rs`

#### Scenario: Chat with tools does not fallback on generic 516

- WHEN chat_with_tools gets generic HTTP 516 without schema hint
- THEN MUST NOT fall back
- Test: `chat_with_tools_does_not_fallback_on_generic_516` in `src/providers/compatible.rs`

#### Scenario: Chat does not fallback on generic 516

- WHEN chat gets generic HTTP 516 without schema hint
- THEN MUST NOT fall back
- Test: `chat_does_not_fallback_on_generic_516` in `src/providers/compatible.rs`

#### Scenario: Response with no tool calls has empty vec

- WHEN response has no tool calls
- THEN MUST return empty vector
- Test: `response_with_no_tool_calls_has_empty_vec` in `src/providers/compatible.rs`

#### Scenario: Flatten system messages merges into first user and removes system roles

- WHEN system messages are flattened
- THEN MUST merge and remove system roles
- Test: `flatten_system_messages_merges_into_first_user_and_removes_system_roles` in `src/providers/compatible.rs`

#### Scenario: Flatten system messages inserts synthetic user when no user exists

- WHEN system messages exist without user
- THEN MUST insert synthetic user
- Test: `flatten_system_messages_inserts_synthetic_user_when_no_user_exists` in `src/providers/compatible.rs`

#### Scenario: Strip think tags removes multiple blocks with surrounding text

- WHEN content has multiple think blocks
- THEN MUST remove all think blocks
- Test: `strip_think_tags_removes_multiple_blocks_with_surrounding_text` in `src/providers/compatible.rs`

#### Scenario: Strip think tags drops tail for unclosed block

- WHEN content has unclosed think block at tail
- THEN MUST drop tail
- Test: `strip_think_tags_drops_tail_for_unclosed_block` in `src/providers/compatible.rs`

#### Scenario: Reasoning content fallback when content empty

- WHEN content is empty but reasoning_content exists
- THEN MUST fall back to reasoning_content
- Test: `reasoning_content_fallback_when_content_empty` in `src/providers/compatible.rs`

#### Scenario: Reasoning content fallback when content null

- WHEN content is null but reasoning_content exists
- THEN MUST fall back to reasoning_content
- Test: `reasoning_content_fallback_when_content_null` in `src/providers/compatible.rs`

#### Scenario: Reasoning content fallback when content missing

- WHEN content is missing but reasoning_content exists
- THEN MUST fall back to reasoning_content
- Test: `reasoning_content_fallback_when_content_missing` in `src/providers/compatible.rs`

#### Scenario: Reasoning content not used when content present

- WHEN both content and reasoning_content exist
- THEN MUST prefer content
- Test: `reasoning_content_not_used_when_content_present` in `src/providers/compatible.rs`

#### Scenario: Reasoning content used when content is only think tags

- WHEN content has only think tags and reasoning_content exists
- THEN MUST use reasoning_content
- Test: `reasoning_content_used_when_content_only_think_tags` in `src/providers/compatible.rs`

#### Scenario: Reasoning content both absent returns empty

- WHEN both content and reasoning_content are absent
- THEN MUST return empty
- Test: `reasoning_content_both_absent_returns_empty` in `src/providers/compatible.rs`

#### Scenario: Reasoning content ignored by normal models

- WHEN normal model has reasoning_content
- THEN MUST ignore it
- Test: `reasoning_content_ignored_by_normal_models` in `src/providers/compatible.rs`

#### Scenario: Parse SSE line with content

- WHEN SSE line has content field
- THEN MUST extract content
- Test: `parse_sse_line_with_content` in `src/providers/compatible.rs`

#### Scenario: Parse SSE line with reasoning content

- WHEN SSE line has reasoning_content field
- THEN MUST extract reasoning content
- Test: `parse_sse_line_with_reasoning_content` in `src/providers/compatible.rs`

#### Scenario: Parse SSE line with both prefers content

- WHEN SSE line has both content and reasoning_content
- THEN MUST prefer content
- Test: `parse_sse_line_with_both_prefers_content` in `src/providers/compatible.rs`

#### Scenario: Parse SSE line with empty content falls back to reasoning content

- WHEN SSE line has empty content but reasoning_content
- THEN MUST fall back to reasoning_content
- Test: `parse_sse_line_with_empty_content_falls_back_to_reasoning_content` in `src/providers/compatible.rs`

#### Scenario: Parse SSE line done sentinel

- WHEN SSE line is done sentinel
- THEN MUST signal done
- Test: `parse_sse_line_done_sentinel` in `src/providers/compatible.rs`

#### Scenario: API response parses usage

- WHEN API response includes usage
- THEN MUST parse token counts
- Test: `api_response_parses_usage` in `src/providers/compatible.rs`

#### Scenario: API response parses without usage

- WHEN API response lacks usage
- THEN MUST return None
- Test: `api_response_parses_without_usage` in `src/providers/compatible.rs`

#### Scenario: Parse native response captures reasoning content

- WHEN native response has reasoning content
- THEN MUST capture it
- Test: `parse_native_response_captures_reasoning_content` in `src/providers/compatible.rs`

#### Scenario: Parse native response none reasoning content for normal model

- WHEN normal model native response is parsed
- THEN MUST not extract reasoning content
- Test: `parse_native_response_none_reasoning_content_for_normal_model` in `src/providers/compatible.rs`

#### Scenario: Convert messages for native round-trips reasoning content

- WHEN messages with reasoning content are converted for native
- THEN MUST round-trip reasoning content
- Test: `convert_messages_for_native_round_trips_reasoning_content` in `src/providers/compatible.rs`

#### Scenario: Convert messages for native no reasoning content when absent

- WHEN messages without reasoning content are converted for native
- THEN MUST not inject reasoning content
- Test: `convert_messages_for_native_no_reasoning_content_when_absent` in `src/providers/compatible.rs`

#### Scenario: Convert messages for native reasoning content serialized only when present

- WHEN messages are converted for native format
- THEN reasoning_content MUST be serialized only when present
- Test: `convert_messages_for_native_reasoning_content_serialized_only_when_present` in `src/providers/compatible.rs`

### REQ-PROV-008: OpenRouter Provider

`OpenRouterProvider` MUST support OpenRouter API with multi-modal vision handling.

#### Scenario: Capabilities report vision support

- WHEN capabilities are queried
- THEN MUST report vision support
- Test: `capabilities_report_vision_support` in `src/providers/openrouter.rs`

#### Scenario: Creates with key

- WHEN OpenRouterProvider is created with key
- THEN MUST store key
- Test: `creates_with_key` in `src/providers/openrouter.rs`

#### Scenario: Creates without key

- WHEN OpenRouterProvider is created without key
- THEN MUST create successfully
- Test: `creates_without_key` in `src/providers/openrouter.rs`

#### Scenario: Warmup without key is noop

- WHEN warmup is called without key
- THEN MUST succeed as noop
- Test: `warmup_without_key_is_noop` in `src/providers/openrouter.rs`

#### Scenario: Chat with system fails without key

- WHEN chat_with_system is called without key
- THEN MUST return error
- Test: `chat_with_system_fails_without_key` in `src/providers/openrouter.rs`

#### Scenario: Chat with history fails without key

- WHEN chat_with_history is called without key
- THEN MUST return error
- Test: `chat_with_history_fails_without_key` in `src/providers/openrouter.rs`

#### Scenario: Chat request serializes with system and user

- WHEN request is built with system and user
- THEN MUST serialize correctly
- Test: `chat_request_serializes_with_system_and_user` in `src/providers/openrouter.rs`

#### Scenario: Chat request serializes history messages

- WHEN request is built with history
- THEN MUST serialize all messages
- Test: `chat_request_serializes_history_messages` in `src/providers/openrouter.rs`

#### Scenario: Chat request serializes max tokens when present

- WHEN max_tokens is set
- THEN MUST include in request
- Test: `chat_request_serializes_max_tokens_when_present` in `src/providers/openrouter.rs`

#### Scenario: Response deserializes single choice

- WHEN API response has single choice
- THEN MUST deserialize correctly
- Test: `response_deserializes_single_choice` in `src/providers/openrouter.rs`

#### Scenario: Response deserializes empty choices

- WHEN API response has empty choices
- THEN MUST handle gracefully
- Test: `response_deserializes_empty_choices` in `src/providers/openrouter.rs`

#### Scenario: Chat with tools fails without key

- WHEN chat_with_tools is called without key
- THEN MUST return error
- Test: `chat_with_tools_fails_without_key` in `src/providers/openrouter.rs`

#### Scenario: Native response deserializes with tool calls

- WHEN native response has tool calls
- THEN MUST deserialize tool calls
- Test: `native_response_deserializes_with_tool_calls` in `src/providers/openrouter.rs`

#### Scenario: Native response deserializes with text and tool calls

- WHEN native response has both text and tool calls
- THEN MUST deserialize both
- Test: `native_response_deserializes_with_text_and_tool_calls` in `src/providers/openrouter.rs`

#### Scenario: Parse native response converts to chat response

- WHEN native response is parsed
- THEN MUST convert to ProviderChatResponse
- Test: `parse_native_response_converts_to_chat_response` in `src/providers/openrouter.rs`

#### Scenario: Convert messages parses assistant tool call payload

- WHEN assistant messages with tool call payload are converted
- THEN MUST parse tool calls
- Test: `convert_messages_parses_assistant_tool_call_payload` in `src/providers/openrouter.rs`

#### Scenario: Convert messages parses tool result payload

- WHEN tool result messages are converted
- THEN MUST parse tool results
- Test: `convert_messages_parses_tool_result_payload` in `src/providers/openrouter.rs`

#### Scenario: To message content converts image markers to OpenAI parts

- WHEN user content has image markers
- THEN MUST convert to OpenAI image URL parts
- Test: `to_message_content_converts_image_markers_to_openai_parts` in `src/providers/openrouter.rs`

#### Scenario: Native response parses usage

- WHEN native response includes usage
- THEN MUST parse token counts
- Test: `native_response_parses_usage` in `src/providers/openrouter.rs`

#### Scenario: Native response parses without usage

- WHEN native response lacks usage
- THEN MUST return None
- Test: `native_response_parses_without_usage` in `src/providers/openrouter.rs`

#### Scenario: Parse native response captures reasoning content

- WHEN native response has reasoning content
- THEN MUST capture it
- Test: `parse_native_response_captures_reasoning_content` in `src/providers/openrouter.rs`

#### Scenario: Parse native response none reasoning content for normal model

- WHEN normal model response is parsed
- THEN MUST not extract reasoning content
- Test: `parse_native_response_none_reasoning_content_for_normal_model` in `src/providers/openrouter.rs`

#### Scenario: Native response deserializes reasoning content

- WHEN native response JSON has reasoning_content
- THEN MUST deserialize it
- Test: `native_response_deserializes_reasoning_content` in `src/providers/openrouter.rs`

#### Scenario: Convert messages round-trips reasoning content

- WHEN messages with reasoning content are converted
- THEN MUST preserve through round-trip
- Test: `convert_messages_round_trips_reasoning_content` in `src/providers/openrouter.rs`

#### Scenario: Convert messages no reasoning content when absent

- WHEN messages without reasoning content are converted
- THEN MUST not inject it
- Test: `convert_messages_no_reasoning_content_when_absent` in `src/providers/openrouter.rs`

#### Scenario: Native message omits reasoning content when none

- WHEN native message has no reasoning content
- THEN MUST omit field
- Test: `native_message_omits_reasoning_content_when_none` in `src/providers/openrouter.rs`

#### Scenario: Native message includes reasoning content when some

- WHEN native message has reasoning content
- THEN MUST include field
- Test: `native_message_includes_reasoning_content_when_some` in `src/providers/openrouter.rs`

### REQ-PROV-009: GLM Provider

`GlmProvider` MUST support Zhipu GLM models with JWT authentication.

#### Scenario: Parses API key

- WHEN valid API key is provided
- THEN MUST parse key and secret
- Test: `parses_api_key` in `src/providers/glm.rs`

#### Scenario: Handles no key

- WHEN no API key is provided
- THEN MUST handle gracefully
- Test: `handles_no_key` in `src/providers/glm.rs`

#### Scenario: Handles invalid key format

- WHEN API key has invalid format
- THEN MUST return error
- Test: `handles_invalid_key_format` in `src/providers/glm.rs`

#### Scenario: Generates JWT token

- WHEN JWT token is generated
- THEN MUST produce valid JWT
- Test: `generates_jwt_token` in `src/providers/glm.rs`

#### Scenario: Caches token

- WHEN JWT token is requested multiple times
- THEN MUST cache and reuse
- Test: `caches_token` in `src/providers/glm.rs`

#### Scenario: Fails without key

- WHEN provider is used without key
- THEN MUST fail
- Test: `fails_without_key` in `src/providers/glm.rs`

#### Scenario: Chat fails without key

- WHEN chat is called without key
- THEN MUST return error
- Test: `chat_fails_without_key` in `src/providers/glm.rs`

#### Scenario: Chat with history fails without key

- WHEN chat_with_history is called without key
- THEN MUST return error
- Test: `chat_with_history_fails_without_key` in `src/providers/glm.rs`

#### Scenario: Base64url no padding

- WHEN base64url encoding is used
- THEN MUST not include padding
- Test: `base64url_no_padding` in `src/providers/glm.rs`

#### Scenario: Warmup without key is noop

- WHEN warmup is called without key
- THEN MUST succeed as noop
- Test: `warmup_without_key_is_noop` in `src/providers/glm.rs`

### REQ-PROV-010: Telnyx Provider

`TelnyxProvider` MUST support Telnyx inference API.

#### Scenario: Creates provider with key

- WHEN TelnyxProvider is created with key
- THEN MUST store key
- Test: `creates_provider_with_key` in `src/providers/telnyx.rs`

#### Scenario: Creates provider without key

- WHEN TelnyxProvider is created without key
- THEN MUST create successfully
- Test: `creates_provider_without_key` in `src/providers/telnyx.rs`

#### Scenario: Model constants are valid

- WHEN model constants are checked
- THEN MUST be valid strings
- Test: `model_constants_are_valid` in `src/providers/telnyx.rs`

#### Scenario: Resolve key from parameter

- WHEN key is provided as parameter
- THEN MUST use parameter key
- Test: `resolve_key_from_parameter` in `src/providers/telnyx.rs`

#### Scenario: Resolve key trims whitespace

- WHEN key has surrounding whitespace
- THEN MUST trim whitespace
- Test: `resolve_key_trims_whitespace` in `src/providers/telnyx.rs`

#### Scenario: Models response deserializes

- WHEN models API response is received
- THEN MUST deserialize correctly
- Test: `models_response_deserializes` in `src/providers/telnyx.rs`

#### Scenario: Chat request serializes

- WHEN chat request is built
- THEN MUST serialize correctly
- Test: `chat_request_serializes` in `src/providers/telnyx.rs`

#### Scenario: Chat response deserializes

- WHEN chat response is received
- THEN MUST deserialize correctly
- Test: `chat_response_deserializes` in `src/providers/telnyx.rs`

### REQ-PROV-011: Cursor Provider

`CursorProvider` MUST support Cursor headless CLI as a provider backend.

#### Scenario: New uses env override

- WHEN CURSOR_BINARY env is set
- THEN MUST use env override
- Test: `new_uses_env_override` in `src/providers/cursor.rs`

#### Scenario: New defaults to cursor

- WHEN CURSOR_BINARY env is not set
- THEN MUST default to "cursor"
- Test: `new_defaults_to_cursor` in `src/providers/cursor.rs`

#### Scenario: New ignores blank env override

- WHEN CURSOR_BINARY env is blank
- THEN MUST ignore and use default
- Test: `new_ignores_blank_env_override` in `src/providers/cursor.rs`

#### Scenario: Should forward model standard

- WHEN standard model name is used
- THEN MUST forward model to CLI
- Test: `should_forward_model_standard` in `src/providers/cursor.rs`

#### Scenario: Should not forward default model

- WHEN default model is used
- THEN MUST NOT forward model to CLI
- Test: `should_not_forward_default_model` in `src/providers/cursor.rs`

#### Scenario: Validate temperature allows defaults

- WHEN default temperature is used
- THEN MUST validate successfully
- Test: `validate_temperature_allows_defaults` in `src/providers/cursor.rs`

#### Scenario: Validate temperature rejects custom value

- WHEN custom temperature is used
- THEN MUST reject with error
- Test: `validate_temperature_rejects_custom_value` in `src/providers/cursor.rs`

#### Scenario: Invoke missing binary returns error

- WHEN cursor binary is missing
- THEN MUST return error
- Test: `invoke_missing_binary_returns_error` in `src/providers/cursor.rs`

### REQ-PROV-012: Copilot Provider

`CopilotProvider` MUST support GitHub Copilot API with OAuth device flow.

#### Scenario: New without token

- WHEN CopilotProvider is created without token
- THEN MUST create with None token
- Test: `new_without_token` in `src/providers/copilot.rs`

#### Scenario: New with token

- WHEN CopilotProvider is created with token
- THEN MUST store token
- Test: `new_with_token` in `src/providers/copilot.rs`

#### Scenario: Empty token treated as none

- WHEN CopilotProvider is created with empty token
- THEN MUST treat as None
- Test: `empty_token_treated_as_none` in `src/providers/copilot.rs`

#### Scenario: Cache starts empty

- WHEN provider is first created
- THEN API key cache MUST be empty
- Test: `cache_starts_empty` in `src/providers/copilot.rs`

#### Scenario: Copilot headers include required fields

- WHEN request headers are built
- THEN MUST include required Copilot fields
- Test: `copilot_headers_include_required_fields` in `src/providers/copilot.rs`

#### Scenario: Default interval and expiry

- WHEN default values are used
- THEN MUST match expected defaults
- Test: `default_interval_and_expiry` in `src/providers/copilot.rs`

#### Scenario: Supports native tools

- WHEN supports_native_tools is checked
- THEN MUST return true
- Test: `supports_native_tools` in `src/providers/copilot.rs`

#### Scenario: API response parses usage

- WHEN API response includes usage
- THEN MUST parse token counts
- Test: `api_response_parses_usage` in `src/providers/copilot.rs`

#### Scenario: API response parses without usage

- WHEN API response lacks usage
- THEN MUST return None
- Test: `api_response_parses_without_usage` in `src/providers/copilot.rs`

#### Scenario: Merge response choices merges tool calls across choices

- WHEN response has multiple choices with tool calls
- THEN MUST merge tool calls across choices
- Test: `merge_response_choices_merges_tool_calls_across_choices` in `src/providers/copilot.rs`

#### Scenario: Merge response choices prefers first non-empty text

- WHEN response has multiple choices with text
- THEN MUST prefer first non-empty text
- Test: `merge_response_choices_prefers_first_non_empty_text` in `src/providers/copilot.rs`

#### Scenario: Merge response choices rejects empty choice list

- WHEN response has empty choice list
- THEN MUST reject with error
- Test: `merge_response_choices_rejects_empty_choice_list` in `src/providers/copilot.rs`

### REQ-PROV-013: OpenAI Codex Provider

`OpenAiCodexProvider` MUST support Codex responses API via WebSocket transport.

#### Scenario: Extracts output text first

- WHEN response has output_text at top level
- THEN MUST extract it first
- Test: `extracts_output_text_first` in `src/providers/openai_codex.rs`

#### Scenario: Extracts nested output text

- WHEN response has nested output text
- THEN MUST extract it
- Test: `extracts_nested_output_text` in `src/providers/openai_codex.rs`

#### Scenario: Default state dir is non-empty

- WHEN default state dir is computed
- THEN MUST return non-empty path
- Test: `default_state_dir_is_non_empty` in `src/providers/openai_codex.rs`

#### Scenario: Build responses URL appends suffix for base URL

- WHEN base URL is provided
- THEN MUST append responses suffix
- Test: `build_responses_url_appends_suffix_for_base_url` in `src/providers/openai_codex.rs`

#### Scenario: Build responses URL keeps existing responses endpoint

- WHEN URL already has responses endpoint
- THEN MUST keep as-is
- Test: `build_responses_url_keeps_existing_responses_endpoint` in `src/providers/openai_codex.rs`

#### Scenario: Resolve responses URL prefers explicit endpoint env

- WHEN explicit endpoint env is set
- THEN MUST prefer explicit endpoint
- Test: `resolve_responses_url_prefers_explicit_endpoint_env` in `src/providers/openai_codex.rs`

#### Scenario: Resolve responses URL uses provider API URL override

- WHEN provider API URL override is set
- THEN MUST use provider URL override
- Test: `resolve_responses_url_uses_provider_api_url_override` in `src/providers/openai_codex.rs`

#### Scenario: Resolve transport mode defaults to auto

- WHEN transport mode is not configured
- THEN MUST default to auto
- Test: `resolve_transport_mode_defaults_to_auto` in `src/providers/openai_codex.rs`

#### Scenario: Resolve transport mode accepts runtime override

- WHEN transport mode is overridden at runtime
- THEN MUST accept override
- Test: `resolve_transport_mode_accepts_runtime_override` in `src/providers/openai_codex.rs`

#### Scenario: Resolve transport mode legacy bool env is supported

- WHEN legacy boolean env for WebSocket is set
- THEN MUST support legacy format
- Test: `resolve_transport_mode_legacy_bool_env_is_supported` in `src/providers/openai_codex.rs`

#### Scenario: Resolve transport mode rejects invalid runtime override

- WHEN invalid transport mode is configured
- THEN MUST reject with error
- Test: `resolve_transport_mode_rejects_invalid_runtime_override` in `src/providers/openai_codex.rs`

#### Scenario: WebSocket URL uses ws scheme and model query

- WHEN WebSocket URL is constructed
- THEN MUST use ws scheme and add model query
- Test: `websocket_url_uses_ws_scheme_and_model_query` in `src/providers/openai_codex.rs`

#### Scenario: Default responses URL detector handles equivalent URLs

- WHEN URLs are checked for default responses endpoint
- THEN MUST detect equivalent URLs
- Test: `default_responses_url_detector_handles_equivalent_urls` in `src/providers/openai_codex.rs`

#### Scenario: Constructor enables custom endpoint key mode

- WHEN custom endpoint is configured
- THEN MUST enable custom endpoint key mode
- Test: `constructor_enables_custom_endpoint_key_mode` in `src/providers/openai_codex.rs`

#### Scenario: Resolve instructions uses default when missing

- WHEN system prompt is missing
- THEN MUST use default instructions
- Test: `resolve_instructions_uses_default_when_missing` in `src/providers/openai_codex.rs`

#### Scenario: Resolve instructions uses default when blank

- WHEN system prompt is blank
- THEN MUST use default instructions
- Test: `resolve_instructions_uses_default_when_blank` in `src/providers/openai_codex.rs`

#### Scenario: Resolve instructions uses system prompt when present

- WHEN system prompt is present
- THEN MUST use system prompt as instructions
- Test: `resolve_instructions_uses_system_prompt_when_present` in `src/providers/openai_codex.rs`

#### Scenario: Clamp reasoning effort adjusts known models

- WHEN reasoning effort is set for known models
- THEN MUST clamp to valid range
- Test: `clamp_reasoning_effort_adjusts_known_models` in `src/providers/openai_codex.rs`

#### Scenario: Resolve reasoning effort prefers config override

- WHEN config override is set
- THEN MUST prefer config override
- Test: `resolve_reasoning_effort_prefers_config_override` in `src/providers/openai_codex.rs`

#### Scenario: Resolve reasoning effort falls back to env when override invalid

- WHEN config override is invalid
- THEN MUST fall back to env
- Test: `resolve_reasoning_effort_falls_back_to_env_when_override_invalid` in `src/providers/openai_codex.rs`

#### Scenario: Parse SSE text reads output text delta

- WHEN SSE text contains output_text delta
- THEN MUST read delta text
- Test: `parse_sse_text_reads_output_text_delta` in `src/providers/openai_codex.rs`

#### Scenario: Parse SSE text falls back to completed response

- WHEN SSE text has completed response but no delta
- THEN MUST fall back to completed response
- Test: `parse_sse_text_falls_back_to_completed_response` in `src/providers/openai_codex.rs`

#### Scenario: Build responses input maps content types by role

- WHEN messages with different roles are converted
- THEN MUST map content types by role
- Test: `build_responses_input_maps_content_types_by_role` in `src/providers/openai_codex.rs`

#### Scenario: Build responses input uses default instructions without system

- WHEN no system message is present
- THEN MUST use default instructions
- Test: `build_responses_input_uses_default_instructions_without_system` in `src/providers/openai_codex.rs`

#### Scenario: Build responses input ignores unknown roles

- WHEN messages have unknown roles
- THEN MUST ignore them
- Test: `build_responses_input_ignores_unknown_roles` in `src/providers/openai_codex.rs`

#### Scenario: Build responses input handles image markers

- WHEN messages contain image markers
- THEN MUST handle image markers
- Test: `build_responses_input_handles_image_markers` in `src/providers/openai_codex.rs`

#### Scenario: Build responses input preserves text-only messages

- WHEN messages are text-only
- THEN MUST preserve as text
- Test: `build_responses_input_preserves_text_only_messages` in `src/providers/openai_codex.rs`

#### Scenario: Build responses input handles multiple images

- WHEN messages contain multiple images
- THEN MUST handle all images
- Test: `build_responses_input_handles_multiple_images` in `src/providers/openai_codex.rs`

#### Scenario: Capabilities includes vision

- WHEN capabilities are queried
- THEN MUST include vision support
- Test: `capabilities_includes_vision` in `src/providers/openai_codex.rs`

### REQ-PROV-014: Reliable Provider

`ReliableProvider` MUST wrap providers with circuit breaking, fallback chains, and health tracking.

#### Scenario: Succeeds without retry

- WHEN primary provider succeeds on first attempt
- THEN MUST return response without retry
- Test: `succeeds_without_retry` in `src/providers/reliable.rs`

#### Scenario: Retries then recovers

- WHEN provider fails then succeeds on retry
- THEN MUST retry and return success
- Test: `retries_then_recovers` in `src/providers/reliable.rs`

#### Scenario: Falls back after retries exhausted

- WHEN primary provider retries are exhausted
- THEN MUST fall back to next provider
- Test: `falls_back_after_retries_exhausted` in `src/providers/reliable.rs`

#### Scenario: Returns aggregated error when all providers fail

- WHEN all providers fail
- THEN MUST return aggregated error
- Test: `returns_aggregated_error_when_all_providers_fail` in `src/providers/reliable.rs`

#### Scenario: Non-retryable detects common patterns

- WHEN error matches non-retryable patterns (401, 403, 404)
- THEN MUST detect as non-retryable
- Test: `non_retryable_detects_common_patterns` in `src/providers/reliable.rs`

#### Scenario: Context window error aborts retries and model fallbacks

- WHEN context window exceeded error occurs
- THEN MUST abort retries and model fallbacks
- Test: `context_window_error_aborts_retries_and_model_fallbacks` in `src/providers/reliable.rs`

#### Scenario: Aggregated error marks non-retryable model mismatch with details

- WHEN aggregated error includes model mismatch
- THEN MUST mark as non-retryable with details
- Test: `aggregated_error_marks_non_retryable_model_mismatch_with_details` in `src/providers/reliable.rs`

#### Scenario: Skips retries on non-retryable error

- WHEN error is non-retryable
- THEN MUST skip retries
- Test: `skips_retries_on_non_retryable_error` in `src/providers/reliable.rs`

#### Scenario: Chat with history retries then recovers

- WHEN chat_with_history fails then succeeds on retry
- THEN MUST retry and recover
- Test: `chat_with_history_retries_then_recovers` in `src/providers/reliable.rs`

#### Scenario: Chat with history falls back

- WHEN chat_with_history retries are exhausted
- THEN MUST fall back to next provider
- Test: `chat_with_history_falls_back` in `src/providers/reliable.rs`

#### Scenario: Model failover tries fallback model

- WHEN primary model fails
- THEN MUST try fallback model
- Test: `model_failover_tries_fallback_model` in `src/providers/reliable.rs`

#### Scenario: Model failover all models fail

- WHEN all models fail
- THEN MUST return aggregated error
- Test: `model_failover_all_models_fail` in `src/providers/reliable.rs`

#### Scenario: No model fallbacks behaves like before

- WHEN no model fallbacks are configured
- THEN MUST behave like simple retry
- Test: `no_model_fallbacks_behaves_like_before` in `src/providers/reliable.rs`

#### Scenario: Provider-keyed model fallbacks remap fallback provider models

- WHEN provider-keyed model fallbacks are configured
- THEN MUST remap model names for fallback providers
- Test: `provider_keyed_model_fallbacks_remap_fallback_provider_models` in `src/providers/reliable.rs`

#### Scenario: Auth rotation cycles keys

- WHEN multiple auth keys are available
- THEN MUST cycle through keys on failure
- Test: `auth_rotation_cycles_keys` in `src/providers/reliable.rs`

#### Scenario: Auth rotation returns none when empty

- WHEN no auth keys are available
- THEN MUST return None
- Test: `auth_rotation_returns_none_when_empty` in `src/providers/reliable.rs`

#### Scenario: Parse retry-after integer

- WHEN retry-after header is integer
- THEN MUST parse as milliseconds
- Test: `parse_retry_after_integer` in `src/providers/reliable.rs`

#### Scenario: Parse retry-after float

- WHEN retry-after header is float
- THEN MUST parse as milliseconds
- Test: `parse_retry_after_float` in `src/providers/reliable.rs`

#### Scenario: Parse retry-after missing

- WHEN retry-after header is missing
- THEN MUST return None
- Test: `parse_retry_after_missing` in `src/providers/reliable.rs`

#### Scenario: Rate limited detection

- WHEN error is rate limited (429)
- THEN MUST detect as rate limited
- Test: `rate_limited_detection` in `src/providers/reliable.rs`

#### Scenario: Non-retryable rate limit detects plan-restricted model

- WHEN rate limit error mentions plan restriction
- THEN MUST detect as non-retryable rate limit
- Test: `non_retryable_rate_limit_detects_plan_restricted_model` in `src/providers/reliable.rs`

#### Scenario: Non-retryable rate limit detects insufficient balance

- WHEN rate limit error mentions insufficient balance
- THEN MUST detect as non-retryable rate limit
- Test: `non_retryable_rate_limit_detects_insufficient_balance` in `src/providers/reliable.rs`

#### Scenario: Non-retryable rate limit does not flag generic 429

- WHEN rate limit error is generic 429
- THEN MUST NOT flag as non-retryable
- Test: `non_retryable_rate_limit_does_not_flag_generic_429` in `src/providers/reliable.rs`

#### Scenario: Compute backoff uses retry-after

- WHEN retry-after header is available
- THEN MUST use retry-after for backoff
- Test: `compute_backoff_uses_retry_after` in `src/providers/reliable.rs`

#### Scenario: Compute backoff caps at 30s

- WHEN computed backoff exceeds 30s
- THEN MUST cap at 30 seconds
- Test: `compute_backoff_caps_at_30s` in `src/providers/reliable.rs`

#### Scenario: Compute backoff falls back to base

- WHEN no retry-after header
- THEN MUST fall back to base backoff
- Test: `compute_backoff_falls_back_to_base` in `src/providers/reliable.rs`

#### Scenario: Non-retryable detects 401

- WHEN error is 401 Unauthorized
- THEN MUST detect as non-retryable
- Test: `non_retryable_detects_401` in `src/providers/reliable.rs`

#### Scenario: Non-retryable detects 403

- WHEN error is 403 Forbidden
- THEN MUST detect as non-retryable
- Test: `non_retryable_detects_403` in `src/providers/reliable.rs`

#### Scenario: Non-retryable detects 404

- WHEN error is 404 Not Found
- THEN MUST detect as non-retryable
- Test: `non_retryable_detects_404` in `src/providers/reliable.rs`

#### Scenario: Non-retryable does not flag 429

- WHEN error is 429 Too Many Requests
- THEN MUST NOT flag as non-retryable
- Test: `non_retryable_does_not_flag_429` in `src/providers/reliable.rs`

#### Scenario: Non-retryable does not flag 408

- WHEN error is 408 Request Timeout
- THEN MUST NOT flag as non-retryable
- Test: `non_retryable_does_not_flag_408` in `src/providers/reliable.rs`

#### Scenario: Non-retryable does not flag 500

- WHEN error is 500 Internal Server Error
- THEN MUST NOT flag as non-retryable
- Test: `non_retryable_does_not_flag_500` in `src/providers/reliable.rs`

#### Scenario: Non-retryable does not flag 502

- WHEN error is 502 Bad Gateway
- THEN MUST NOT flag as non-retryable
- Test: `non_retryable_does_not_flag_502` in `src/providers/reliable.rs`

#### Scenario: Parse retry-after zero

- WHEN retry-after is zero
- THEN MUST handle zero value
- Test: `parse_retry_after_zero` in `src/providers/reliable.rs`

#### Scenario: Parse retry-after with underscore separator

- WHEN retry-after uses underscore separator
- THEN MUST parse correctly
- Test: `parse_retry_after_with_underscore_separator` in `src/providers/reliable.rs`

#### Scenario: Parse retry-after space separator

- WHEN retry-after uses space separator
- THEN MUST parse correctly
- Test: `parse_retry_after_space_separator` in `src/providers/reliable.rs`

#### Scenario: Rate limited false for generic error

- WHEN error is generic (not 429)
- THEN MUST return false for rate limited
- Test: `rate_limited_false_for_generic_error` in `src/providers/reliable.rs`

#### Scenario: Non-retryable skips retries for 401

- WHEN 401 error occurs
- THEN MUST skip retries
- Test: `non_retryable_skips_retries_for_401` in `src/providers/reliable.rs`

#### Scenario: Non-retryable rate limit skips retries for plan errors

- WHEN plan-restricted rate limit error occurs
- THEN MUST skip retries
- Test: `non_retryable_rate_limit_skips_retries_for_plan_errors` in `src/providers/reliable.rs`

#### Scenario: Native tool schema rejection skips retries for 516

- WHEN HTTP 516 with schema hint occurs
- THEN MUST skip retries
- Test: `native_tool_schema_rejection_skips_retries_for_516` in `src/providers/reliable.rs`

#### Scenario: Generic 516 without schema hint remains retryable

- WHEN HTTP 516 without schema hint occurs
- THEN MUST remain retryable
- Test: `generic_516_without_schema_hint_remains_retryable` in `src/providers/reliable.rs`

#### Scenario: Chat delegates to inner provider

- WHEN chat is called on reliable provider
- THEN MUST delegate to inner provider
- Test: `chat_delegates_to_inner_provider` in `src/providers/reliable.rs`

#### Scenario: Chat retries and recovers

- WHEN chat call fails then succeeds
- THEN MUST retry and recover
- Test: `chat_retries_and_recovers` in `src/providers/reliable.rs`

#### Scenario: Chat preserves native tools support

- WHEN inner provider supports native tools
- THEN MUST preserve native tools support flag
- Test: `chat_preserves_native_tools_support` in `src/providers/reliable.rs`

#### Scenario: Chat returns aggregated error when all providers fail

- WHEN all providers fail for chat
- THEN MUST return aggregated error
- Test: `chat_returns_aggregated_error_when_all_providers_fail` in `src/providers/reliable.rs`

#### Scenario: Chat tries model failover on failure

- WHEN chat fails with primary model
- THEN MUST try model failover
- Test: `chat_tries_model_failover_on_failure` in `src/providers/reliable.rs`

#### Scenario: Chat skips non-retryable errors

- WHEN chat encounters non-retryable error
- THEN MUST skip retries
- Test: `chat_skips_non_retryable_errors` in `src/providers/reliable.rs`

#### Scenario: Vision override forces true

- WHEN vision override is set to true
- THEN MUST force vision support to true
- Test: `vision_override_forces_true` in `src/providers/reliable.rs`

#### Scenario: Vision override forces false

- WHEN vision override is set to false
- THEN MUST force vision support to false
- Test: `vision_override_forces_false` in `src/providers/reliable.rs`

#### Scenario: Vision override none defers to provider

- WHEN vision override is None
- THEN MUST defer to inner provider
- Test: `vision_override_none_defers_to_provider` in `src/providers/reliable.rs`

### REQ-PROV-015: Health Tracking

`ProviderHealthTracker` MUST implement circuit breaker pattern for provider availability.

#### Scenario: Allows provider initially

- WHEN provider is first tracked
- THEN MUST allow requests
- Test: `allows_provider_initially` in `src/providers/health.rs`

#### Scenario: Tracks failures below threshold

- WHEN failures are below threshold
- THEN MUST still allow requests
- Test: `tracks_failures_below_threshold` in `src/providers/health.rs`

#### Scenario: Opens circuit at threshold

- WHEN failures reach threshold
- THEN MUST open circuit and block requests
- Test: `opens_circuit_at_threshold` in `src/providers/health.rs`

#### Scenario: Circuit closes after cooldown

- WHEN cooldown period expires
- THEN MUST close circuit and allow requests
- Test: `circuit_closes_after_cooldown` in `src/providers/health.rs`

#### Scenario: Repeated failures while circuit open do not extend cooldown

- WHEN failures occur while circuit is open
- THEN MUST NOT extend cooldown
- Test: `repeated_failures_while_circuit_open_do_not_extend_cooldown` in `src/providers/health.rs`

#### Scenario: New rejects zero failure threshold

- WHEN failure threshold is set to zero
- THEN MUST reject with error
- Test: `new_rejects_zero_failure_threshold` in `src/providers/health.rs`

#### Scenario: New rejects zero cooldown

- WHEN cooldown is set to zero
- THEN MUST reject with error
- Test: `new_rejects_zero_cooldown` in `src/providers/health.rs`

#### Scenario: Success resets failure count

- WHEN success occurs after failures
- THEN MUST reset failure count
- Test: `success_resets_failure_count` in `src/providers/health.rs`

#### Scenario: Success clears circuit breaker

- WHEN success occurs after circuit opens
- THEN MUST clear circuit breaker
- Test: `success_clears_circuit_breaker` in `src/providers/health.rs`

#### Scenario: Tracks multiple providers independently

- WHEN multiple providers are tracked
- THEN MUST track each independently
- Test: `tracks_multiple_providers_independently` in `src/providers/health.rs`

#### Scenario: Get all states returns all tracked providers

- WHEN get_all_states is called
- THEN MUST return all tracked providers
- Test: `get_all_states_returns_all_tracked_providers` in `src/providers/health.rs`

### REQ-PROV-016: Provider Router

`RouterProvider` MUST route model requests to configured providers based on routing rules.

#### Scenario: Routes hint to correct provider

- WHEN model has hint prefix matching a route
- THEN MUST route to correct provider
- Test: `routes_hint_to_correct_provider` in `src/providers/router.rs`

#### Scenario: Routes fast hint

- WHEN model has fast hint
- THEN MUST route to fast provider
- Test: `routes_fast_hint` in `src/providers/router.rs`

#### Scenario: Unknown hint falls back to default

- WHEN model has unknown hint
- THEN MUST fall back to default provider
- Test: `unknown_hint_falls_back_to_default` in `src/providers/router.rs`

#### Scenario: Non-hint model uses default provider

- WHEN model has no hint
- THEN MUST use default provider
- Test: `non_hint_model_uses_default_provider` in `src/providers/router.rs`

#### Scenario: Resolve preserves model for non-hints

- WHEN resolve is called with non-hint model
- THEN MUST preserve original model name
- Test: `resolve_preserves_model_for_non_hints` in `src/providers/router.rs`

#### Scenario: Resolve strips hint prefix

- WHEN resolve is called with hint model
- THEN MUST strip hint prefix
- Test: `resolve_strips_hint_prefix` in `src/providers/router.rs`

#### Scenario: Resolve trims whitespace in hint reference

- WHEN hint reference has whitespace
- THEN MUST trim whitespace
- Test: `resolve_trims_whitespace_in_hint_reference` in `src/providers/router.rs`

#### Scenario: Resolve matches routes with whitespace hint config

- WHEN route config has whitespace in hint
- THEN MUST match after trimming
- Test: `resolve_matches_routes_with_whitespace_hint_config` in `src/providers/router.rs`

#### Scenario: Skips routes with unknown provider

- WHEN route references unknown provider
- THEN MUST skip route
- Test: `skips_routes_with_unknown_provider` in `src/providers/router.rs`

#### Scenario: Warmup calls all providers

- WHEN warmup is called
- THEN MUST call warmup on all providers
- Test: `warmup_calls_all_providers` in `src/providers/router.rs`

#### Scenario: Chat with system passes system prompt

- WHEN chat_with_system is called via router
- THEN MUST pass system prompt to resolved provider
- Test: `chat_with_system_passes_system_prompt` in `src/providers/router.rs`

#### Scenario: Chat with tools delegates to resolved provider

- WHEN chat_with_tools is called via router
- THEN MUST delegate to resolved provider
- Test: `chat_with_tools_delegates_to_resolved_provider` in `src/providers/router.rs`

#### Scenario: Chat with tools routes hint correctly

- WHEN chat_with_tools is called with hint model
- THEN MUST route to correct provider
- Test: `chat_with_tools_routes_hint_correctly` in `src/providers/router.rs`

### REQ-PROV-017: Backoff Store

`BackoffStore` MUST provide TTL-based exponential backoff tracking per provider.

#### Scenario: Backoff stores and retrieves entry

- WHEN a backoff entry is stored
- THEN MUST be retrievable
- Test: `backoff_stores_and_retrieves_entry` in `src/providers/backoff.rs`

#### Scenario: Backoff expires after duration

- WHEN backoff duration expires
- THEN MUST no longer be active
- Test: `backoff_expires_after_duration` in `src/providers/backoff.rs`

#### Scenario: Backoff clears on demand

- WHEN backoff is cleared
- THEN MUST no longer be active
- Test: `backoff_clears_on_demand` in `src/providers/backoff.rs`

#### Scenario: Backoff min deadline eviction at capacity

- WHEN backoff store reaches capacity
- THEN MUST evict entry with earliest deadline
- Test: `backoff_min_deadline_eviction_at_capacity` in `src/providers/backoff.rs`

#### Scenario: Backoff max entries clamped to one

- WHEN max entries is set below one
- THEN MUST clamp to one
- Test: `backoff_max_entries_clamped_to_one` in `src/providers/backoff.rs`

### REQ-PROV-018: Quota Types

Quota types MUST provide data structures for tracking provider quota status and filtering.

#### Scenario: Available providers filter

- WHEN `available_providers()` is called on QuotaSummary
- THEN MUST return only providers with Ok status
- Test: `quota_summary_available_providers` in `src/providers/quota_types.rs`

#### Scenario: Rate limited providers filter

- WHEN `rate_limited_providers()` is called
- THEN MUST return providers with RateLimited or QuotaExhausted status
- Test: `quota_summary_rate_limited_providers` in `src/providers/quota_types.rs`

#### Scenario: Circuit open providers filter

- WHEN `circuit_open_providers()` is called
- THEN MUST return providers with CircuitOpen status
- Test: `quota_summary_circuit_open_providers` in `src/providers/quota_types.rs`

#### Scenario: Empty providers returns empty collections

- WHEN QuotaSummary has no providers
- THEN MUST return empty collections
- Test: `quota_summary_empty_providers` in `src/providers/quota_types.rs`

#### Scenario: All OK providers

- WHEN all providers have Ok status
- THEN MUST return all as available
- Test: `quota_summary_all_ok` in `src/providers/quota_types.rs`

#### Scenario: Provider usage metrics new

- WHEN ProviderUsageMetrics is constructed
- THEN MUST initialize correctly
- Test: `provider_usage_metrics_new` in `src/providers/quota_types.rs`

#### Scenario: Provider usage metrics default

- WHEN ProviderUsageMetrics is default-constructed
- THEN MUST have zero values
- Test: `provider_usage_metrics_default` in `src/providers/quota_types.rs`

#### Scenario: Quota status serde roundtrip

- WHEN QuotaStatus is serialized and deserialized
- THEN MUST round-trip correctly
- Test: `quota_status_serde_roundtrip` in `src/providers/quota_types.rs`

#### Scenario: Quota metadata construction

- WHEN QuotaMetadata is constructed
- THEN MUST store all fields
- Test: `quota_metadata_construction` in `src/providers/quota_types.rs`

#### Scenario: Provider quota info with profiles

- WHEN ProviderQuotaInfo includes profiles
- THEN MUST store profile data
- Test: `provider_quota_info_with_profiles` in `src/providers/quota_types.rs`

### REQ-PROV-019: Quota Adapter

`QuotaExtractor` trait and adapters MUST extract quota information from HTTP response headers per provider format.

#### Scenario: OpenAI extractor headers

- WHEN OpenAI response headers are inspected
- THEN MUST extract rate limit metadata
- Test: `test_openai_extractor_headers` in `src/providers/quota_adapter.rs`

#### Scenario: Anthropic extractor headers

- WHEN Anthropic response headers are inspected
- THEN MUST extract rate limit metadata
- Test: `test_anthropic_extractor_headers` in `src/providers/quota_adapter.rs`

#### Scenario: Gemini extractor headers

- WHEN Gemini response headers are inspected
- THEN MUST extract rate limit metadata
- Test: `test_gemini_extractor_headers` in `src/providers/quota_adapter.rs`

#### Scenario: Gemini extractor error

- WHEN Gemini error is inspected
- THEN MUST extract quota info from error
- Test: `test_gemini_extractor_error` in `src/providers/quota_adapter.rs`

#### Scenario: Universal extractor provider-specific

- WHEN provider-specific headers are inspected by universal extractor
- THEN MUST delegate to correct provider extractor
- Test: `test_universal_extractor_provider_specific` in `src/providers/quota_adapter.rs`

#### Scenario: Universal extractor fallback

- WHEN unknown provider headers are inspected
- THEN MUST use fallback extraction
- Test: `test_universal_extractor_fallback` in `src/providers/quota_adapter.rs`

#### Scenario: Universal extractor error fallback

- WHEN error is inspected by universal extractor
- THEN MUST use error fallback extraction
- Test: `test_universal_extractor_error_fallback` in `src/providers/quota_adapter.rs`

#### Scenario: Universal extractor no match

- WHEN headers have no rate limit info
- THEN MUST return None
- Test: `test_universal_extractor_no_match` in `src/providers/quota_adapter.rs`

#### Scenario: Qwen extractor headers

- WHEN Qwen response headers are inspected
- THEN MUST extract rate limit metadata
- Test: `test_qwen_extractor_headers` in `src/providers/quota_adapter.rs`

#### Scenario: Qwen extractor error

- WHEN Qwen error is inspected
- THEN MUST extract quota info from error
- Test: `test_qwen_extractor_error` in `src/providers/quota_adapter.rs`

#### Scenario: Universal extractor Qwen error

- WHEN Qwen error is inspected by universal extractor
- THEN MUST delegate to Qwen extractor
- Test: `test_universal_extractor_qwen_error` in `src/providers/quota_adapter.rs`

### REQ-PROV-020: Quota CLI

Quota CLI MUST display provider quota status in text and JSON formats.

#### Scenario: Format relative time future

- WHEN time is in the future
- THEN MUST format as relative future time
- Test: `test_format_relative_time_future` in `src/providers/quota_cli.rs`

#### Scenario: Format relative time past

- WHEN time is in the past
- THEN MUST format as relative past time
- Test: `test_format_relative_time_past` in `src/providers/quota_cli.rs`

#### Scenario: Truncate string

- WHEN string exceeds max length
- THEN MUST truncate with ellipsis
- Test: `test_truncate` in `src/providers/quota_cli.rs`

#### Scenario: Parse retry-after from error

- WHEN error message contains retry-after
- THEN MUST parse retry-after value
- Test: `test_parse_retry_after` in `src/providers/quota_cli.rs`

### REQ-PROV-021: Provider Factory

Factory functions MUST construct the correct provider from configuration with API key resolution and model validation.

#### Scenario: Resolve provider credential prefers explicit argument

- WHEN explicit credential argument is provided
- THEN MUST prefer explicit argument over env
- Test: `resolve_provider_credential_prefers_explicit_argument` in `src/providers/mod.rs`

#### Scenario: Resolve provider credential uses MiniMax OAuth env for placeholder

- WHEN credential is MiniMax OAuth placeholder
- THEN MUST use OAuth env key
- Test: `resolve_provider_credential_uses_minimax_oauth_env_for_placeholder` in `src/providers/mod.rs`

#### Scenario: Resolve provider credential falls back to MiniMax API key for placeholder

- WHEN credential is MiniMax placeholder and OAuth env missing
- THEN MUST fall back to MiniMax API key env
- Test: `resolve_provider_credential_falls_back_to_minimax_api_key_for_placeholder` in `src/providers/mod.rs`

#### Scenario: Resolve provider credential placeholder ignores generic API key fallback

- WHEN credential is placeholder
- THEN MUST NOT fall back to generic API key env
- Test: `resolve_provider_credential_placeholder_ignores_generic_api_key_fallback` in `src/providers/mod.rs`

#### Scenario: Resolve provider credential Bedrock uses internal credential path

- WHEN Bedrock credential is resolved
- THEN MUST use internal credential path
- Test: `resolve_provider_credential_bedrock_uses_internal_credential_path` in `src/providers/mod.rs`

#### Scenario: Resolve provider credential prefers step primary env key

- WHEN StepFun primary env key is set
- THEN MUST prefer primary key
- Test: `resolve_provider_credential_prefers_step_primary_env_key` in `src/providers/mod.rs`

#### Scenario: Resolve provider credential uses StepFun fallback env key

- WHEN StepFun primary key is missing
- THEN MUST use fallback env key
- Test: `resolve_provider_credential_uses_stepfun_fallback_env_key` in `src/providers/mod.rs`

#### Scenario: Resolve Qwen OAuth context prefers explicit override

- WHEN explicit Qwen OAuth override is provided
- THEN MUST prefer override
- Test: `resolve_qwen_oauth_context_prefers_explicit_override` in `src/providers/mod.rs`

#### Scenario: Resolve Qwen OAuth context uses env token and resource URL

- WHEN Qwen OAuth env token is set
- THEN MUST use env token and resource URL
- Test: `resolve_qwen_oauth_context_uses_env_token_and_resource_url` in `src/providers/mod.rs`

#### Scenario: Resolve Qwen OAuth context reads cached credentials file

- WHEN cached credentials file exists
- THEN MUST read cached credentials
- Test: `resolve_qwen_oauth_context_reads_cached_credentials_file` in `src/providers/mod.rs`

#### Scenario: Resolve Qwen OAuth context placeholder does not use DashScope fallback

- WHEN Qwen OAuth placeholder is used
- THEN MUST NOT use DashScope fallback
- Test: `resolve_qwen_oauth_context_placeholder_does_not_use_dashscope_fallback` in `src/providers/mod.rs`

#### Scenario: Provider credential available Qwen OAuth accepts refresh token without live refresh

- WHEN Qwen OAuth has refresh token
- THEN MUST accept without live refresh
- Test: `provider_credential_available_qwen_oauth_accepts_refresh_token_without_live_refresh` in `src/providers/mod.rs`

#### Scenario: Provider credential available Qwen OAuth rejects placeholder without sources

- WHEN Qwen OAuth has placeholder without sources
- THEN MUST reject
- Test: `provider_credential_available_qwen_oauth_rejects_placeholder_without_sources` in `src/providers/mod.rs`

#### Scenario: Regional alias predicates cover expected variants

- WHEN regional alias predicates are checked
- THEN MUST cover all expected variants
- Test: `regional_alias_predicates_cover_expected_variants` in `src/providers/mod.rs`

#### Scenario: Canonical China provider name maps regional aliases

- WHEN regional aliases are mapped
- THEN MUST map to canonical China provider names
- Test: `canonical_china_provider_name_maps_regional_aliases` in `src/providers/mod.rs`

#### Scenario: Regional endpoint aliases map to expected URLs

- WHEN regional endpoint aliases are resolved
- THEN MUST map to expected URLs
- Test: `regional_endpoint_aliases_map_to_expected_urls` in `src/providers/mod.rs`

#### Scenario: Factory OpenRouter

- WHEN openrouter provider is requested
- THEN MUST construct OpenRouterProvider
- Test: `factory_openrouter` in `src/providers/mod.rs`

#### Scenario: Factory Anthropic

- WHEN anthropic provider is requested
- THEN MUST construct AnthropicProvider
- Test: `factory_anthropic` in `src/providers/mod.rs`

#### Scenario: Factory OpenAI

- WHEN openai provider is requested
- THEN MUST construct OpenAiProvider
- Test: `factory_openai` in `src/providers/mod.rs`

#### Scenario: Factory OpenAI Codex

- WHEN openai-codex provider is requested
- THEN MUST construct OpenAiCodexProvider
- Test: `factory_openai_codex` in `src/providers/mod.rs`

#### Scenario: Factory Ollama

- WHEN ollama provider is requested
- THEN MUST construct OllamaProvider
- Test: `factory_ollama` in `src/providers/mod.rs`

#### Scenario: Factory Gemini

- WHEN gemini provider is requested
- THEN MUST construct GeminiProvider
- Test: `factory_gemini` in `src/providers/mod.rs`

#### Scenario: Factory Telnyx

- WHEN telnyx provider is requested
- THEN MUST construct TelnyxProvider
- Test: `factory_telnyx` in `src/providers/mod.rs`

#### Scenario: Factory Venice

- WHEN venice provider is requested
- THEN MUST construct compatible provider
- Test: `factory_venice` in `src/providers/mod.rs`

#### Scenario: Factory Vercel

- WHEN vercel provider is requested
- THEN MUST construct compatible provider
- Test: `factory_vercel` in `src/providers/mod.rs`

#### Scenario: Vercel gateway base URL matches public gateway endpoint

- WHEN Vercel gateway URL is constructed
- THEN MUST match public gateway endpoint
- Test: `vercel_gateway_base_url_matches_public_gateway_endpoint` in `src/providers/mod.rs`

#### Scenario: Factory Cloudflare

- WHEN cloudflare provider is requested
- THEN MUST construct compatible provider
- Test: `factory_cloudflare` in `src/providers/mod.rs`

#### Scenario: Factory Moonshot

- WHEN moonshot provider is requested
- THEN MUST construct compatible provider
- Test: `factory_moonshot` in `src/providers/mod.rs`

#### Scenario: Factory StepFun

- WHEN stepfun provider is requested
- THEN MUST construct compatible provider
- Test: `factory_stepfun` in `src/providers/mod.rs`

#### Scenario: Factory Kimi Code

- WHEN kimi-code provider is requested
- THEN MUST construct compatible provider
- Test: `factory_kimi_code` in `src/providers/mod.rs`

#### Scenario: Factory Synthetic

- WHEN synthetic provider is requested
- THEN MUST construct compatible provider
- Test: `factory_synthetic` in `src/providers/mod.rs`

#### Scenario: Factory OpenCode

- WHEN opencode provider is requested
- THEN MUST construct compatible provider
- Test: `factory_opencode` in `src/providers/mod.rs`

#### Scenario: Factory ZAI

- WHEN zai provider is requested
- THEN MUST construct compatible provider
- Test: `factory_zai` in `src/providers/mod.rs`

#### Scenario: Factory GLM

- WHEN glm provider is requested
- THEN MUST construct GlmProvider
- Test: `factory_glm` in `src/providers/mod.rs`

#### Scenario: Factory MiniMax

- WHEN minimax provider is requested
- THEN MUST construct compatible provider
- Test: `factory_minimax` in `src/providers/mod.rs`

#### Scenario: Factory MiniMax disables native tool calling

- WHEN minimax provider is constructed
- THEN MUST disable native tool calling
- Test: `factory_minimax_disables_native_tool_calling` in `src/providers/mod.rs`

#### Scenario: Factory Bedrock

- WHEN bedrock provider is requested
- THEN MUST construct AwsBedrockProvider
- Test: `factory_bedrock` in `src/providers/mod.rs`

#### Scenario: Factory Hunyuan

- WHEN hunyuan provider is requested
- THEN MUST construct compatible provider
- Test: `factory_hunyuan` in `src/providers/mod.rs`

#### Scenario: Factory Qianfan

- WHEN qianfan provider is requested
- THEN MUST construct compatible provider
- Test: `factory_qianfan` in `src/providers/mod.rs`

#### Scenario: Factory Doubao

- WHEN doubao provider is requested
- THEN MUST construct compatible provider
- Test: `factory_doubao` in `src/providers/mod.rs`

#### Scenario: Factory SiliconFlow

- WHEN siliconflow provider is requested
- THEN MUST construct compatible provider
- Test: `factory_siliconflow` in `src/providers/mod.rs`

#### Scenario: Factory Qwen

- WHEN qwen provider is requested
- THEN MUST construct compatible provider
- Test: `factory_qwen` in `src/providers/mod.rs`

#### Scenario: Qwen provider supports vision

- WHEN qwen provider capabilities are queried
- THEN MUST support vision
- Test: `qwen_provider_supports_vision` in `src/providers/mod.rs`

#### Scenario: Factory LM Studio

- WHEN lmstudio provider is requested
- THEN MUST construct compatible provider
- Test: `factory_lmstudio` in `src/providers/mod.rs`

#### Scenario: Factory LlamaCpp

- WHEN llamacpp provider is requested
- THEN MUST construct compatible provider
- Test: `factory_llamacpp` in `src/providers/mod.rs`

#### Scenario: Factory SGLang

- WHEN sglang provider is requested
- THEN MUST construct compatible provider
- Test: `factory_sglang` in `src/providers/mod.rs`

#### Scenario: Factory VLLM

- WHEN vllm provider is requested
- THEN MUST construct compatible provider
- Test: `factory_vllm` in `src/providers/mod.rs`

#### Scenario: Factory Osaurus

- WHEN osaurus provider is requested
- THEN MUST construct compatible provider
- Test: `factory_osaurus` in `src/providers/mod.rs`

#### Scenario: Factory Osaurus uses default key when none

- WHEN osaurus provider has no key
- THEN MUST use default key
- Test: `factory_osaurus_uses_default_key_when_none` in `src/providers/mod.rs`

#### Scenario: Factory Osaurus custom URL

- WHEN osaurus provider has custom URL
- THEN MUST use custom URL
- Test: `factory_osaurus_custom_url` in `src/providers/mod.rs`

#### Scenario: Resolve provider credential Osaurus env

- WHEN osaurus credential is resolved from env
- THEN MUST use osaurus env key
- Test: `resolve_provider_credential_osaurus_env` in `src/providers/mod.rs`

#### Scenario: Factory Groq

- WHEN groq provider is requested
- THEN MUST construct compatible provider
- Test: `factory_groq` in `src/providers/mod.rs`

#### Scenario: Factory Mistral

- WHEN mistral provider is requested
- THEN MUST construct compatible provider
- Test: `factory_mistral` in `src/providers/mod.rs`

#### Scenario: Factory XAI

- WHEN xai provider is requested
- THEN MUST construct compatible provider
- Test: `factory_xai` in `src/providers/mod.rs`

#### Scenario: Factory DeepSeek

- WHEN deepseek provider is requested
- THEN MUST construct compatible provider
- Test: `factory_deepseek` in `src/providers/mod.rs`

#### Scenario: DeepSeek provider keeps vision disabled

- WHEN deepseek provider capabilities are queried
- THEN MUST keep vision disabled
- Test: `deepseek_provider_keeps_vision_disabled` in `src/providers/mod.rs`

#### Scenario: Factory Together

- WHEN together provider is requested
- THEN MUST construct compatible provider
- Test: `factory_together` in `src/providers/mod.rs`

#### Scenario: Factory Fireworks

- WHEN fireworks provider is requested
- THEN MUST construct compatible provider
- Test: `factory_fireworks` in `src/providers/mod.rs`

#### Scenario: Factory Perplexity

- WHEN perplexity provider is requested
- THEN MUST construct compatible provider
- Test: `factory_perplexity` in `src/providers/mod.rs`

#### Scenario: Factory Cohere

- WHEN cohere provider is requested
- THEN MUST construct compatible provider
- Test: `factory_cohere` in `src/providers/mod.rs`

#### Scenario: Factory Copilot

- WHEN copilot provider is requested
- THEN MUST construct CopilotProvider
- Test: `factory_copilot` in `src/providers/mod.rs`

#### Scenario: Factory Cursor

- WHEN cursor provider is requested
- THEN MUST construct CursorProvider
- Test: `factory_cursor` in `src/providers/mod.rs`

#### Scenario: Factory NVIDIA

- WHEN nvidia provider is requested
- THEN MUST construct compatible provider
- Test: `factory_nvidia` in `src/providers/mod.rs`

#### Scenario: Factory AstrAI

- WHEN astrai provider is requested
- THEN MUST construct compatible provider
- Test: `factory_astrai` in `src/providers/mod.rs`

#### Scenario: Factory custom URL

- WHEN custom URL provider is requested
- THEN MUST construct compatible provider with custom URL
- Test: `factory_custom_url` in `src/providers/mod.rs`

#### Scenario: Factory custom localhost

- WHEN custom localhost provider is requested
- THEN MUST construct compatible provider
- Test: `factory_custom_localhost` in `src/providers/mod.rs`

#### Scenario: Factory custom no key

- WHEN custom provider is requested without key
- THEN MUST create without key
- Test: `factory_custom_no_key` in `src/providers/mod.rs`

#### Scenario: Factory custom empty URL errors

- WHEN custom provider has empty URL
- THEN MUST return error
- Test: `factory_custom_empty_url_errors` in `src/providers/mod.rs`

#### Scenario: Factory custom invalid URL errors

- WHEN custom provider has invalid URL
- THEN MUST return error
- Test: `factory_custom_invalid_url_errors` in `src/providers/mod.rs`

#### Scenario: Factory custom unsupported scheme errors

- WHEN custom provider has unsupported URL scheme
- THEN MUST return error
- Test: `factory_custom_unsupported_scheme_errors` in `src/providers/mod.rs`

#### Scenario: Factory custom trims whitespace

- WHEN custom provider URL has whitespace
- THEN MUST trim whitespace
- Test: `factory_custom_trims_whitespace` in `src/providers/mod.rs`

#### Scenario: Factory Anthropic custom URL

- WHEN anthropic-custom provider is requested with URL
- THEN MUST construct AnthropicProvider with custom URL
- Test: `factory_anthropic_custom_url` in `src/providers/mod.rs`

#### Scenario: Factory Anthropic custom trailing slash

- WHEN anthropic-custom URL has trailing slash
- THEN MUST handle trailing slash
- Test: `factory_anthropic_custom_trailing_slash` in `src/providers/mod.rs`

#### Scenario: Factory Anthropic custom no key

- WHEN anthropic-custom is requested without key
- THEN MUST create without key
- Test: `factory_anthropic_custom_no_key` in `src/providers/mod.rs`

#### Scenario: Factory Anthropic custom empty URL errors

- WHEN anthropic-custom has empty URL
- THEN MUST return error
- Test: `factory_anthropic_custom_empty_url_errors` in `src/providers/mod.rs`

#### Scenario: Factory Anthropic custom invalid URL errors

- WHEN anthropic-custom has invalid URL
- THEN MUST return error
- Test: `factory_anthropic_custom_invalid_url_errors` in `src/providers/mod.rs`

#### Scenario: Factory Anthropic custom unsupported scheme errors

- WHEN anthropic-custom has unsupported URL scheme
- THEN MUST return error
- Test: `factory_anthropic_custom_unsupported_scheme_errors` in `src/providers/mod.rs`

#### Scenario: Factory unknown provider errors

- WHEN unknown provider name is requested
- THEN MUST return error
- Test: `factory_unknown_provider_errors` in `src/providers/mod.rs`

#### Scenario: Factory empty name errors

- WHEN empty provider name is requested
- THEN MUST return error
- Test: `factory_empty_name_errors` in `src/providers/mod.rs`

#### Scenario: Resilient provider ignores duplicate and invalid fallbacks

- WHEN resilient provider has duplicate or invalid fallback names
- THEN MUST ignore duplicates and invalid entries
- Test: `resilient_provider_ignores_duplicate_and_invalid_fallbacks` in `src/providers/mod.rs`

#### Scenario: Resilient provider errors for invalid primary

- WHEN resilient provider has invalid primary provider
- THEN MUST return error
- Test: `resilient_provider_errors_for_invalid_primary` in `src/providers/mod.rs`

#### Scenario: Resilient fallback resolves own credential

- WHEN resilient fallback provider needs credential
- THEN MUST resolve its own credential
- Test: `resilient_fallback_resolves_own_credential` in `src/providers/mod.rs`

#### Scenario: Resilient fallback supports custom URL

- WHEN resilient fallback has custom URL
- THEN MUST support custom URL
- Test: `resilient_fallback_supports_custom_url` in `src/providers/mod.rs`

#### Scenario: Resilient fallback mixed chain

- WHEN resilient provider has mixed fallback chain
- THEN MUST support mixed provider types
- Test: `resilient_fallback_mixed_chain` in `src/providers/mod.rs`

#### Scenario: Ollama with custom URL

- WHEN ollama factory is called with custom URL
- THEN MUST use custom URL
- Test: `ollama_with_custom_url` in `src/providers/mod.rs`

#### Scenario: Ollama cloud with custom URL

- WHEN ollama cloud factory is called with custom URL
- THEN MUST use custom URL
- Test: `ollama_cloud_with_custom_url` in `src/providers/mod.rs`

#### Scenario: Resilient fallback includes Osaurus

- WHEN resilient fallback chain includes osaurus
- THEN MUST include osaurus provider
- Test: `resilient_fallback_includes_osaurus` in `src/providers/mod.rs`

#### Scenario: Factory all providers create successfully

- WHEN all known provider names are iterated
- THEN MUST create all providers successfully
- Test: `factory_all_providers_create_successfully` in `src/providers/mod.rs`

#### Scenario: Listed providers have unique IDs and aliases

- WHEN providers are listed
- THEN MUST have unique IDs and aliases
- Test: `listed_providers_have_unique_ids_and_aliases` in `src/providers/mod.rs`

#### Scenario: Listed providers and aliases are constructible

- WHEN providers and aliases are listed
- THEN MUST all be constructible
- Test: `listed_providers_and_aliases_are_constructible` in `src/providers/mod.rs`

#### Scenario: Native tool schema rejection status covers vendor 516

- WHEN HTTP 516 status is received
- THEN MUST detect as native tool schema rejection
- Test: `native_tool_schema_rejection_status_covers_vendor_516` in `src/providers/mod.rs`

#### Scenario: Native tool schema rejection hint is precise

- WHEN native tool schema rejection hint is checked
- THEN MUST match precise hint patterns
- Test: `native_tool_schema_rejection_hint_is_precise` in `src/providers/mod.rs`

#### Scenario: Native tool schema rejection combines status and hint

- WHEN HTTP 516 with hint is received
- THEN MUST combine status and hint for detection
- Test: `native_tool_schema_rejection_combines_status_and_hint` in `src/providers/mod.rs`

#### Scenario: Sanitize scrubs sk prefix

- WHEN error message contains sk- prefixed secret
- THEN MUST scrub the secret
- Test: `sanitize_scrubs_sk_prefix` in `src/providers/mod.rs`

#### Scenario: Sanitize scrubs multiple prefixes

- WHEN error message contains multiple secret prefixes
- THEN MUST scrub all secrets
- Test: `sanitize_scrubs_multiple_prefixes` in `src/providers/mod.rs`

#### Scenario: Sanitize short prefix then real key

- WHEN error has short prefix followed by real key
- THEN MUST scrub real key
- Test: `sanitize_short_prefix_then_real_key` in `src/providers/mod.rs`

#### Scenario: Sanitize sk-proj comment then real key

- WHEN error has sk-proj comment then real key
- THEN MUST scrub real key
- Test: `sanitize_sk_proj_comment_then_real_key` in `src/providers/mod.rs`

#### Scenario: Sanitize keeps bare prefix

- WHEN error has only bare prefix without key
- THEN MUST keep bare prefix
- Test: `sanitize_keeps_bare_prefix` in `src/providers/mod.rs`

#### Scenario: Sanitize handles JSON wrapped key

- WHEN error has JSON-wrapped API key
- THEN MUST sanitize key in JSON
- Test: `sanitize_handles_json_wrapped_key` in `src/providers/mod.rs`

#### Scenario: Sanitize handles delimiter boundaries

- WHEN error has secrets at delimiter boundaries
- THEN MUST handle boundary scrubbing
- Test: `sanitize_handles_delimiter_boundaries` in `src/providers/mod.rs`

#### Scenario: Sanitize truncates long error

- WHEN error message is very long
- THEN MUST truncate
- Test: `sanitize_truncates_long_error` in `src/providers/mod.rs`

#### Scenario: Sanitize truncates after scrub

- WHEN error message is long after scrubbing
- THEN MUST truncate after scrub
- Test: `sanitize_truncates_after_scrub` in `src/providers/mod.rs`

#### Scenario: Sanitize preserves unicode boundaries

- WHEN error message has unicode content
- THEN MUST preserve unicode boundaries when truncating
- Test: `sanitize_preserves_unicode_boundaries` in `src/providers/mod.rs`

#### Scenario: Sanitize no secret no change

- WHEN error message has no secrets
- THEN MUST not modify message
- Test: `sanitize_no_secret_no_change` in `src/providers/mod.rs`

#### Scenario: Scrub GitHub personal access token

- WHEN error contains GitHub PAT
- THEN MUST scrub token
- Test: `scrub_github_personal_access_token` in `src/providers/mod.rs`

#### Scenario: Scrub GitHub OAuth token

- WHEN error contains GitHub OAuth token
- THEN MUST scrub token
- Test: `scrub_github_oauth_token` in `src/providers/mod.rs`

#### Scenario: Scrub GitHub user token

- WHEN error contains GitHub user token
- THEN MUST scrub token
- Test: `scrub_github_user_token` in `src/providers/mod.rs`

#### Scenario: Scrub GitHub fine-grained PAT

- WHEN error contains GitHub fine-grained PAT
- THEN MUST scrub token
- Test: `scrub_github_fine_grained_pat` in `src/providers/mod.rs`

#### Scenario: Scrub Google API key prefix

- WHEN error contains Google API key
- THEN MUST scrub key
- Test: `scrub_google_api_key_prefix` in `src/providers/mod.rs`

#### Scenario: Scrub AWS access key prefix

- WHEN error contains AWS access key
- THEN MUST scrub key
- Test: `scrub_aws_access_key_prefix` in `src/providers/mod.rs`

#### Scenario: Sanitize redacts JSON access_token field

- WHEN error has JSON access_token field
- THEN MUST redact token value
- Test: `sanitize_redacts_json_access_token_field` in `src/providers/mod.rs`

#### Scenario: Sanitize redacts query client_secret field

- WHEN error has query client_secret field
- THEN MUST redact secret value
- Test: `sanitize_redacts_query_client_secret_field` in `src/providers/mod.rs`

#### Scenario: Sanitize redacts JSON token field

- WHEN error has JSON token field
- THEN MUST redact token value
- Test: `sanitize_redacts_json_token_field` in `src/providers/mod.rs`

#### Scenario: Sanitize redacts query token field

- WHEN error has query token field
- THEN MUST redact token value
- Test: `sanitize_redacts_query_token_field` in `src/providers/mod.rs`

#### Scenario: Sanitize redacts Bearer token sequence

- WHEN error has Bearer token sequence
- THEN MUST redact token
- Test: `sanitize_redacts_bearer_token_sequence` in `src/providers/mod.rs`

#### Scenario: Sanitize preserves short Bearer phrase without secret

- WHEN error has short Bearer phrase without secret
- THEN MUST preserve phrase
- Test: `sanitize_preserves_short_bearer_phrase_without_secret` in `src/providers/mod.rs`

#### Scenario: Routed provider accepts per-route max tokens

- WHEN routed provider has per-route max tokens
- THEN MUST accept and apply max tokens
- Test: `routed_provider_accepts_per_route_max_tokens` in `src/providers/mod.rs`

#### Scenario: Routed provider supports hint default when primary init fails

- WHEN routed provider primary init fails
- THEN MUST support hint default fallback
- Test: `routed_provider_supports_hint_default_when_primary_init_fails` in `src/providers/mod.rs`

#### Scenario: Routed provider normalizes whitespace in hint routes

- WHEN hint routes have whitespace
- THEN MUST normalize whitespace
- Test: `routed_provider_normalizes_whitespace_in_hint_routes` in `src/providers/mod.rs`

#### Scenario: Routed provider rejects unresolved hint default

- WHEN hint default cannot be resolved
- THEN MUST reject configuration
- Test: `routed_provider_rejects_unresolved_hint_default` in `src/providers/mod.rs`

#### Scenario: Parse provider profile plain name

- WHEN plain provider name is parsed
- THEN MUST return name without profile
- Test: `parse_provider_profile_plain_name` in `src/providers/mod.rs`

#### Scenario: Parse provider profile with profile

- WHEN provider name with profile is parsed
- THEN MUST return name and profile
- Test: `parse_provider_profile_with_profile` in `src/providers/mod.rs`

#### Scenario: Parse provider profile custom URL not split

- WHEN custom URL provider is parsed
- THEN MUST NOT split on colon
- Test: `parse_provider_profile_custom_url_not_split` in `src/providers/mod.rs`

#### Scenario: Parse provider profile anthropic-custom not split

- WHEN anthropic-custom provider is parsed
- THEN MUST NOT split on colon
- Test: `parse_provider_profile_anthropic_custom_not_split` in `src/providers/mod.rs`

#### Scenario: Parse provider profile empty profile ignored

- WHEN provider name has empty profile
- THEN MUST ignore empty profile
- Test: `parse_provider_profile_empty_profile_ignored` in `src/providers/mod.rs`

#### Scenario: Parse provider profile extra colons kept

- WHEN provider name has extra colons
- THEN MUST keep extra colons in profile
- Test: `parse_provider_profile_extra_colons_kept` in `src/providers/mod.rs`

#### Scenario: Resilient fallback with profile syntax

- WHEN resilient fallback uses profile syntax
- THEN MUST resolve profile correctly
- Test: `resilient_fallback_with_profile_syntax` in `src/providers/mod.rs`

#### Scenario: Resilient fallback mixed profiles and custom

- WHEN resilient fallback mixes profiles and custom URLs
- THEN MUST resolve all correctly
- Test: `resilient_fallback_mixed_profiles_and_custom` in `src/providers/mod.rs`

## Mock Strategy

- HTTP APIs: `wiremock::MockServer` for provider API simulation
- Provider trait: Mock providers implementing `Provider` with configurable responses
- Health tracking: Direct construction and state testing
- Quota: Direct struct construction with test data
- Backoff: Time-based testing with short TTLs
- Factory: Partial construction with test config (API keys not required for config parsing tests)

## Coverage Notes

| Source File | Test Count |
|---|---|
| `src/providers/traits.rs` | 27 |
| `src/providers/mod.rs` | 128 |
| `src/providers/anthropic.rs` | 50 |
| `src/providers/openai.rs` | 29 |
| `src/providers/gemini.rs` | 47 |
| `src/providers/bedrock.rs` | 55 |
| `src/providers/ollama.rs` | 30 |
| `src/providers/openrouter.rs` | 27 |
| `src/providers/compatible.rs` | 96 |
| `src/providers/glm.rs` | 10 |
| `src/providers/telnyx.rs` | 8 |
| `src/providers/cursor.rs` | 8 |
| `src/providers/copilot.rs` | 12 |
| `src/providers/openai_codex.rs` | 29 |
| `src/providers/reliable.rs` | 50 |
| `src/providers/health.rs` | 11 |
| `src/providers/router.rs` | 13 |
| `src/providers/backoff.rs` | 5 |
| `src/providers/quota_types.rs` | 10 |
| `src/providers/quota_adapter.rs` | 11 |
| `src/providers/quota_cli.rs` | 4 |
| **Total** | **660** |
