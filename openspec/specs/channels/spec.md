# Channels Specification

## Purpose

Define requirements for the channel trait contract and all platform channel implementations.

## Scope

- Files: 30 files in `src/channels/` (~44,100 LOC total)
- Risk tier: MEDIUM-HIGH (channels handle user messages, auth tokens, and webhook payloads)
- Verified tests: 1003 across 30 files

| Source File | Test Count |
|---|---|
| `src/channels/mod.rs` | 120 |
| `src/channels/traits.rs` | 10 |
| `src/channels/telegram.rs` | 172 |
| `src/channels/discord.rs` | 70 |
| `src/channels/slack.rs` | 40 |
| `src/channels/whatsapp.rs` | 47 |
| `src/channels/whatsapp_web.rs` | 17 |
| `src/channels/wati.rs` | 19 |
| `src/channels/whatsapp_storage.rs` | 3 |
| `src/channels/matrix.rs` | 38 |
| `src/channels/irc.rs` | 37 |
| `src/channels/email_channel.rs` | 40 |
| `src/channels/signal.rs` | 32 |
| `src/channels/nostr.rs` | 11 |
| `src/channels/lark.rs` | 49 |
| `src/channels/imessage.rs` | 58 |
| `src/channels/bluebubbles.rs` | 41 |
| `src/channels/mattermost.rs` | 35 |
| `src/channels/mqtt.rs` | 12 |
| `src/channels/github.rs` | 9 |
| `src/channels/nextcloud_talk.rs` | 11 |
| `src/channels/dingtalk.rs` | 14 |
| `src/channels/qq.rs` | 21 |
| `src/channels/linq.rs` | 25 |
| `src/channels/napcat.rs` | 22 |
| `src/channels/acp.rs` | 19 |
| `src/channels/cli.rs` | 6 |
| `src/channels/clawdtalk.rs` | 5 |
| `src/channels/ack_reaction.rs` | 10 |
| `src/channels/transcription.rs` | 10 |

## Requirements

### REQ-CHAN-001: Channel Trait Contract

All channels MUST implement the `Channel` trait with `name`, `send`, and `listen` methods.

#### Scenario: Trait method signatures verified via DummyChannel

- WHEN a type implements `Channel`
- THEN MUST provide `name() -> &str`, `send(&SendMessage)`, and `listen(tx: mpsc::Sender)`
- Test: `default_trait_methods_return_success` in `src/channels/traits.rs`

#### Scenario: Listen delivers messages through mpsc channel

- WHEN `listen()` is called with a sender
- THEN MUST deliver `ChannelMessage` values through the sender
- Test: `listen_sends_message_to_channel` in `src/channels/traits.rs`

### REQ-CHAN-002: Channel Message Types

`ChannelMessage` MUST carry id, sender, reply_target, content, channel name, timestamp, and optional thread_ts.

#### Scenario: Clone preserves all fields

- WHEN a `ChannelMessage` is cloned
- THEN all fields (id, sender, reply_target, content, channel, timestamp) MUST be preserved
- Test: `channel_message_clone_preserves_fields` in `src/channels/traits.rs`

### REQ-CHAN-003: SendMessage Helpers

`SendMessage` MUST provide `new()`, `with_subject()`, and `in_thread()` builder methods.

#### Scenario: new() sets content and recipient

- WHEN `SendMessage::new()` is called
- THEN content and recipient MUST be set, subject and thread_ts MUST be None
- Test: `send_message_new_sets_fields` in `src/channels/traits.rs`

#### Scenario: with_subject() sets all fields including subject

- WHEN `SendMessage::with_subject()` is called
- THEN content, recipient, and subject MUST all be set
- Test: `send_message_with_subject_sets_all_fields` in `src/channels/traits.rs`

#### Scenario: in_thread() sets thread_ts

- WHEN `in_thread()` is chained
- THEN thread_ts MUST be set to the provided value or None
- Test: `send_message_in_thread_sets_thread_ts` in `src/channels/traits.rs`

### REQ-CHAN-004: Optional Trait Methods

Channels MAY implement optional methods (health_check, typing indicators, draft updates, approval prompts, reactions) with safe no-op defaults.

#### Scenario: Default health_check returns true

- WHEN `health_check()` is not overridden
- THEN MUST return `true`
- Test: `default_trait_methods_return_success` in `src/channels/traits.rs`

#### Scenario: Default typing indicators return Ok

- WHEN `start_typing()` / `stop_typing()` are not overridden
- THEN MUST return `Ok(())`
- Test: `default_trait_methods_return_success` in `src/channels/traits.rs`

#### Scenario: Default reaction methods return Ok

- WHEN `add_reaction()` / `remove_reaction()` are not overridden
- THEN MUST return `Ok(())`
- Test: `default_reaction_methods_return_success` in `src/channels/traits.rs`

#### Scenario: Default draft methods return safe no-ops

- WHEN draft methods are not overridden
- THEN `supports_draft_updates()` returns false, `send_draft()` returns None, others return Ok
- Test: `default_draft_methods_return_success` in `src/channels/traits.rs`

#### Scenario: Approval prompt truncates multibyte UTF-8 safely

- WHEN approval prompt receives arguments exceeding 220 bytes with multibyte chars
- THEN MUST truncate at a valid UTF-8 char boundary
- Test: `approval_prompt_truncates_safely_for_multibyte_utf8` in `src/channels/traits.rs`

#### Scenario: Approval prompt passes short args without truncation

- WHEN approval prompt receives short arguments
- THEN MUST not truncate
- Test: `approval_prompt_short_args_not_truncated` in `src/channels/traits.rs`

### REQ-CHAN-005: Factory Registration and Orchestration

All channels MUST be registered in the factory (`src/channels/mod.rs`) with stable, lowercase keys. The orchestration layer MUST handle message dispatch, runtime commands, prompt building, memory context, history management, progress streaming, tool execution, approval flows, typing indicators, reactions, health classification, and supervised listeners.

#### Scenario: Channel message timeout clamps to minimum

- WHEN a configured timeout is below the minimum
- THEN MUST clamp to the minimum value
- Test: `effective_channel_message_timeout_secs_clamps_to_minimum` in `src/channels/mod.rs`

#### Scenario: Timeout budget scales with tool iterations

- WHEN tool iterations increase
- THEN timeout budget MUST scale proportionally
- Test: `channel_message_timeout_budget_scales_with_tool_iterations` in `src/channels/mod.rs`

#### Scenario: Timeout budget uses safe defaults and cap

- WHEN no explicit configuration is provided
- THEN MUST use safe defaults with an upper cap
- Test: `channel_message_timeout_budget_uses_safe_defaults_and_cap` in `src/channels/mod.rs`

#### Scenario: Runtime commands parse on non-model channels

- WHEN approval commands are sent on non-model channels
- THEN MUST parse correctly
- Test: `parse_runtime_command_allows_approval_commands_on_non_model_channels` in `src/channels/mod.rs`

#### Scenario: Runtime commands support natural language approval intents

- WHEN natural language approval phrases are used
- THEN MUST parse as approval commands
- Test: `parse_runtime_command_supports_natural_language_approval_intents` in `src/channels/mod.rs`

#### Scenario: Context window overflow error detection

- WHEN known overflow error messages are encountered
- THEN MUST be detected as context window overflow
- Test: `context_window_overflow_error_detector_matches_known_messages` in `src/channels/mod.rs`

#### Scenario: Memory context skip rules exclude history blobs

- WHEN memory context entries are history blobs
- THEN MUST be excluded from context
- Test: `memory_context_skip_rules_exclude_history_blobs` in `src/channels/mod.rs`

#### Scenario: Normalize merges consecutive user turns

- WHEN cached channel turns have consecutive user turns
- THEN MUST merge them into one
- Test: `normalize_cached_channel_turns_merges_consecutive_user_turns` in `src/channels/mod.rs`

#### Scenario: Normalize merges consecutive assistant turns

- WHEN cached channel turns have consecutive assistant turns
- THEN MUST merge them into one
- Test: `normalize_cached_channel_turns_merges_consecutive_assistant_turns` in `src/channels/mod.rs`

#### Scenario: Normalize preserves failure marker after orphan user turn

- WHEN a failure marker follows an orphan user turn
- THEN MUST preserve the failure marker
- Test: `normalize_preserves_failure_marker_after_orphan_user_turn` in `src/channels/mod.rs`

#### Scenario: Normalize preserves timeout marker after orphan user turn

- WHEN a timeout marker follows an orphan user turn
- THEN MUST preserve the timeout marker
- Test: `normalize_preserves_timeout_marker_after_orphan_user_turn` in `src/channels/mod.rs`

#### Scenario: Compact sender history keeps recent truncated messages

- WHEN sender history is compacted
- THEN MUST keep recent messages in truncated form
- Test: `compact_sender_history_keeps_recent_truncated_messages` in `src/channels/mod.rs`

#### Scenario: Append sender turn stores single turn per call

- WHEN a sender turn is appended
- THEN MUST store exactly one turn per call
- Test: `append_sender_turn_stores_single_turn_per_call` in `src/channels/mod.rs`

#### Scenario: Rollback orphan user turn removes only latest matching

- WHEN rollback is triggered for an orphan user turn
- THEN MUST remove only the latest matching user turn
- Test: `rollback_orphan_user_turn_removes_only_latest_matching_user_turn` in `src/channels/mod.rs`

#### Scenario: Tool visibility prompt respects excluded snapshot

- WHEN tools are excluded via runtime snapshot
- THEN tool visibility prompt MUST reflect the exclusion
- Test: `build_runtime_tool_visibility_prompt_respects_excluded_snapshot` in `src/channels/mod.rs`

#### Scenario: Process message injects runtime tool visibility prompt

- WHEN a message is processed with tool exclusions
- THEN MUST inject the tool visibility prompt into the system prompt
- Test: `process_channel_message_injects_runtime_tool_visibility_prompt` in `src/channels/mod.rs`

#### Scenario: Process message executes tool calls instead of sending raw JSON

- WHEN the LLM response contains tool call tags
- THEN MUST execute the tool calls rather than sending raw JSON to the user
- Test: `process_channel_message_executes_tool_calls_instead_of_sending_raw_json` in `src/channels/mod.rs`

#### Scenario: Telegram does not persist tool summary prefix

- WHEN processing a Telegram message with tool calls
- THEN MUST not persist the tool summary prefix in history
- Test: `process_channel_message_telegram_does_not_persist_tool_summary_prefix` in `src/channels/mod.rs`

#### Scenario: Streaming hides internal progress by default

- WHEN streaming mode is active without explicit request
- THEN MUST hide internal progress lines
- Test: `process_channel_message_streaming_hides_internal_progress_by_default` in `src/channels/mod.rs`

#### Scenario: Streaming shows internal progress on explicit request

- WHEN user explicitly requests verbose progress
- THEN MUST show internal progress lines
- Test: `process_channel_message_streaming_shows_internal_progress_on_explicit_request` in `src/channels/mod.rs`

#### Scenario: Strips unexecuted tool JSON artifacts from reply

- WHEN the reply contains unexecuted tool JSON artifacts
- THEN MUST strip them before sending to the user
- Test: `process_channel_message_strips_unexecuted_tool_json_artifacts_from_reply` in `src/channels/mod.rs`

#### Scenario: Executes tool calls with alias tags

- WHEN the LLM response contains alias-style tool call tags
- THEN MUST execute those tool calls
- Test: `process_channel_message_executes_tool_calls_with_alias_tags` in `src/channels/mod.rs`

#### Scenario: Handles models command without LLM call

- WHEN user sends /models command
- THEN MUST respond without making an LLM call
- Test: `process_channel_message_handles_models_command_without_llm_call` in `src/channels/mod.rs`

#### Scenario: Handles approve command without LLM call

- WHEN user sends /approve command
- THEN MUST handle approval without making an LLM call
- Test: `process_channel_message_handles_approve_command_without_llm_call` in `src/channels/mod.rs`

#### Scenario: Handles approve-allow command without LLM call

- WHEN user sends /approve-allow command
- THEN MUST grant approval without making an LLM call
- Test: `process_channel_message_handles_approve_allow_command_without_llm_call` in `src/channels/mod.rs`

#### Scenario: Handles approve-deny command without LLM call

- WHEN user sends /approve-deny command
- THEN MUST deny approval without making an LLM call
- Test: `process_channel_message_handles_approve_deny_command_without_llm_call` in `src/channels/mod.rs`

#### Scenario: Prompts and waits for non-CLI always-ask approval

- WHEN non-CLI always-ask approval mode is active
- THEN MUST prompt and wait for explicit approval
- Test: `process_channel_message_prompts_and_waits_for_non_cli_always_ask_approval` in `src/channels/mod.rs`

#### Scenario: Denies approval management for unlisted sender

- WHEN an unlisted sender attempts approval management
- THEN MUST deny the operation
- Test: `process_channel_message_denies_approval_management_for_unlisted_sender` in `src/channels/mod.rs`

#### Scenario: Handles unapprove command without LLM call

- WHEN user sends /unapprove command
- THEN MUST revoke approval without making an LLM call
- Test: `process_channel_message_handles_unapprove_command_without_llm_call` in `src/channels/mod.rs`

#### Scenario: Handles approvals listing command without LLM call

- WHEN user sends /approvals command
- THEN MUST list approvals without making an LLM call
- Test: `process_channel_message_handles_approvals_command_without_llm_call` in `src/channels/mod.rs`

#### Scenario: Approve-allow resolves pending request with yes

- WHEN approve-allow is sent for a pending request
- THEN MUST resolve the request as approved
- Test: `process_channel_message_approve_allow_resolves_pending_request_yes` in `src/channels/mod.rs`

#### Scenario: Approve-deny resolves pending request with no

- WHEN approve-deny is sent for a pending request
- THEN MUST resolve the request as denied
- Test: `process_channel_message_approve_deny_resolves_pending_request_no` in `src/channels/mod.rs`

#### Scenario: Natural language request then confirm approval

- WHEN user requests approval via natural language then confirms
- THEN MUST process the approval flow end-to-end
- Test: `process_channel_message_natural_request_then_confirm_approval` in `src/channels/mod.rs`

#### Scenario: Blocks GCG-like suffix when perplexity filter enabled

- WHEN perplexity filter is enabled and a GCG-like suffix is detected
- THEN MUST block the message
- Test: `process_channel_message_blocks_gcg_like_suffix_when_perplexity_filter_enabled` in `src/channels/mod.rs`

#### Scenario: All-tools-once requires confirm and stays runtime only

- WHEN all-tools-once approval mode is active
- THEN MUST require confirmation and stay in runtime-only scope
- Test: `process_channel_message_all_tools_once_requires_confirm_and_stays_runtime_only` in `src/channels/mod.rs`

#### Scenario: Natural approval direct mode grants immediately

- WHEN natural approval is in direct mode
- THEN MUST grant approval immediately without waiting
- Test: `process_channel_message_natural_approval_direct_mode_grants_immediately` in `src/channels/mod.rs`

#### Scenario: Natural approval honors channel mode override

- WHEN a channel-specific approval mode override is configured
- THEN MUST honor the override
- Test: `process_channel_message_natural_approval_honors_channel_mode_override` in `src/channels/mod.rs`

#### Scenario: Natural approval can be disabled but slash still works

- WHEN natural language approval is disabled
- THEN slash command approval MUST still work
- Test: `process_channel_message_natural_approval_can_be_disabled_but_slash_still_works` in `src/channels/mod.rs`

#### Scenario: Confirm rejects sender mismatch

- WHEN a different sender tries to confirm another sender's approval
- THEN MUST reject the confirmation
- Test: `process_channel_message_confirm_rejects_sender_mismatch` in `src/channels/mod.rs`

#### Scenario: Uses route override provider and model

- WHEN a route override is configured
- THEN MUST use the override provider and model
- Test: `process_channel_message_uses_route_override_provider_and_model` in `src/channels/mod.rs`

#### Scenario: Prefers cached default provider instance

- WHEN a default provider instance is already cached
- THEN MUST reuse the cached instance
- Test: `process_channel_message_prefers_cached_default_provider_instance` in `src/channels/mod.rs`

#### Scenario: Uses runtime default model from store

- WHEN a runtime default model is set in the store
- THEN MUST use it for message processing
- Test: `process_channel_message_uses_runtime_default_model_from_store` in `src/channels/mod.rs`

#### Scenario: Runtime defaults from config include autonomy policy

- WHEN runtime defaults are loaded from config file
- THEN MUST include the autonomy policy
- Test: `load_runtime_defaults_from_config_file_includes_autonomy_policy` in `src/channels/mod.rs`

#### Scenario: Runtime defaults use provider fallback when model missing

- WHEN model is missing from config but provider is present
- THEN MUST use provider fallback
- Test: `load_runtime_defaults_from_config_file_uses_provider_fallback_when_model_missing` in `src/channels/mod.rs`

#### Scenario: Runtime config update refreshes autonomy policy and excluded tools

- WHEN the runtime config file changes
- THEN MUST refresh autonomy policy and excluded tools
- Test: `maybe_apply_runtime_config_update_refreshes_autonomy_policy_and_excluded_tools` in `src/channels/mod.rs`

#### Scenario: Respects configured max tool iterations above default

- WHEN max_tool_iterations is configured above the default
- THEN MUST respect the configured value
- Test: `process_channel_message_respects_configured_max_tool_iterations_above_default` in `src/channels/mod.rs`

#### Scenario: Reports configured max tool iterations limit

- WHEN the tool iteration limit is reached
- THEN MUST report the limit to the user
- Test: `process_channel_message_reports_configured_max_tool_iterations_limit` in `src/channels/mod.rs`

#### Scenario: Message dispatch processes messages in parallel

- WHEN multiple messages arrive concurrently
- THEN MUST dispatch them in parallel
- Test: `message_dispatch_processes_messages_in_parallel` in `src/channels/mod.rs`

#### Scenario: Message dispatch interrupts in-flight Telegram request

- WHEN a new message interrupts an in-flight Telegram request
- THEN MUST cancel the old request and preserve context
- Test: `message_dispatch_interrupts_in_flight_telegram_request_and_preserves_context` in `src/channels/mod.rs`

#### Scenario: Interrupt scope is same sender same chat

- WHEN interruption occurs
- THEN scope MUST be limited to the same sender in the same chat
- Test: `message_dispatch_interrupt_scope_is_same_sender_same_chat` in `src/channels/mod.rs`

#### Scenario: Cancels scoped typing task on message completion

- WHEN message processing completes
- THEN MUST cancel the scoped typing task
- Test: `process_channel_message_cancels_scoped_typing_task` in `src/channels/mod.rs`

#### Scenario: Adds and swaps reactions during message processing

- WHEN message processing progresses through stages
- THEN MUST add and swap reactions accordingly
- Test: `process_channel_message_adds_and_swaps_reactions` in `src/channels/mod.rs`

#### Scenario: System prompt contains all required sections

- WHEN the system prompt is built
- THEN MUST contain all required sections
- Test: `prompt_contains_all_sections` in `src/channels/mod.rs`

#### Scenario: System prompt injects tools

- WHEN tools are available
- THEN MUST inject tool definitions into the prompt
- Test: `prompt_injects_tools` in `src/channels/mod.rs`

#### Scenario: Single tool protocol block after append

- WHEN tool protocol is appended to prompt
- THEN MUST produce a single tool protocol block
- Test: `prompt_includes_single_tool_protocol_block_after_append` in `src/channels/mod.rs`

#### Scenario: System prompt injects safety instructions

- WHEN the system prompt is built
- THEN MUST include safety instructions
- Test: `prompt_injects_safety` in `src/channels/mod.rs`

#### Scenario: System prompt injects workspace files

- WHEN workspace files are available
- THEN MUST inject their contents into the prompt
- Test: `prompt_injects_workspace_files` in `src/channels/mod.rs`

#### Scenario: Missing file markers in prompt

- WHEN referenced files are missing
- THEN MUST include appropriate markers
- Test: `prompt_missing_file_markers` in `src/channels/mod.rs`

#### Scenario: Bootstrap only included if file exists

- WHEN bootstrap file exists
- THEN MUST include it; otherwise MUST skip
- Test: `prompt_bootstrap_only_if_exists` in `src/channels/mod.rs`

#### Scenario: No daily memory injection into prompt

- WHEN daily memory is available
- THEN MUST not inject it into the prompt
- Test: `prompt_no_daily_memory_injection` in `src/channels/mod.rs`

#### Scenario: Prompt includes runtime metadata

- WHEN the prompt is built
- THEN MUST include runtime metadata
- Test: `prompt_runtime_metadata` in `src/channels/mod.rs`

#### Scenario: Skills include instructions and tools

- WHEN skills are configured
- THEN MUST include their instructions and tool definitions
- Test: `prompt_skills_include_instructions_and_tools` in `src/channels/mod.rs`

#### Scenario: Skills compact mode omits instructions and tools

- WHEN skills are in compact mode
- THEN MUST omit instructions and tools
- Test: `prompt_skills_compact_mode_omits_instructions_and_tools` in `src/channels/mod.rs`

#### Scenario: Skills escape reserved XML chars

- WHEN skill content contains reserved XML characters
- THEN MUST escape them
- Test: `prompt_skills_escape_reserved_xml_chars` in `src/channels/mod.rs`

#### Scenario: Prompt truncation for oversized content

- WHEN prompt content exceeds limits
- THEN MUST truncate safely
- Test: `prompt_truncation` in `src/channels/mod.rs`

#### Scenario: Empty workspace files skipped in prompt

- WHEN workspace files are empty
- THEN MUST skip them
- Test: `prompt_empty_files_skipped` in `src/channels/mod.rs`

#### Scenario: Channel log truncation is UTF-8 safe

- WHEN log text contains multibyte characters
- THEN MUST truncate at valid UTF-8 char boundaries
- Test: `channel_log_truncation_is_utf8_safe_for_multibyte_text` in `src/channels/mod.rs`

#### Scenario: Prompt contains channel capabilities

- WHEN the prompt is built for a channel
- THEN MUST include channel-specific capabilities
- Test: `prompt_contains_channel_capabilities` in `src/channels/mod.rs`

#### Scenario: Prompt includes workspace path

- WHEN a workspace path is configured
- THEN MUST include it in the prompt
- Test: `prompt_workspace_path` in `src/channels/mod.rs`

#### Scenario: Conversation memory key uses message ID

- WHEN a conversation memory key is generated
- THEN MUST use the message ID
- Test: `conversation_memory_key_uses_message_id` in `src/channels/mod.rs`

#### Scenario: Conversation memory key is unique per message

- WHEN multiple messages produce memory keys
- THEN keys MUST be unique per message
- Test: `conversation_memory_key_is_unique_per_message` in `src/channels/mod.rs`

#### Scenario: Assistant memory key is namespaced from user key

- WHEN assistant memory key is generated
- THEN MUST be namespaced separately from the user key
- Test: `assistant_memory_key_is_namespaced_from_user_key` in `src/channels/mod.rs`

#### Scenario: Conversation history key ignores QQ message ID thread

- WHEN QQ channel message has a message_id-based thread
- THEN conversation history key MUST ignore it
- Test: `conversation_history_key_ignores_qq_message_id_thread` in `src/channels/mod.rs`

#### Scenario: Conversation history key ignores NapCat message ID thread

- WHEN NapCat channel message has a message_id-based thread
- THEN conversation history key MUST ignore it
- Test: `conversation_history_key_ignores_napcat_message_id_thread` in `src/channels/mod.rs`

#### Scenario: Autosave keys preserve multiple conversation facts

- WHEN multiple facts are saved in a conversation
- THEN MUST preserve all of them with unique keys
- Test: `autosave_keys_preserve_multiple_conversation_facts` in `src/channels/mod.rs`

#### Scenario: Build memory context includes recalled entries

- WHEN memory entries are recalled
- THEN MUST include them in the context
- Test: `build_memory_context_includes_recalled_entries` in `src/channels/mod.rs`

#### Scenario: Build memory context respects session scope

- WHEN session scope is set
- THEN memory context MUST respect the scope boundary
- Test: `build_memory_context_respects_session_scope` in `src/channels/mod.rs`

#### Scenario: Restores per-sender history on follow-ups

- WHEN a follow-up message arrives from the same sender
- THEN MUST restore per-sender history
- Test: `process_channel_message_restores_per_sender_history_on_follow_ups` in `src/channels/mod.rs`

#### Scenario: QQ keeps history across distinct message IDs

- WHEN QQ messages arrive with distinct message IDs
- THEN MUST keep history across them
- Test: `process_channel_message_qq_keeps_history_across_distinct_message_ids` in `src/channels/mod.rs`

#### Scenario: Enriches current turn without persisting context

- WHEN the current turn is enriched with context
- THEN MUST not persist the enrichment to history
- Test: `process_channel_message_enriches_current_turn_without_persisting_context` in `src/channels/mod.rs`

#### Scenario: Telegram keeps system instruction at top only

- WHEN Telegram message processing builds the prompt
- THEN MUST keep system instruction at the top only
- Test: `process_channel_message_telegram_keeps_system_instruction_at_top_only` in `src/channels/mod.rs`

#### Scenario: Extract tool context summary collects alias and native tool calls

- WHEN history contains tool calls with alias and native tags
- THEN MUST collect all tool names
- Test: `extract_tool_context_summary_collects_alias_and_native_tool_calls` in `src/channels/mod.rs`

#### Scenario: Extract tool context summary collects prompt-mode tool result names

- WHEN history contains prompt-mode tool results
- THEN MUST collect those tool names
- Test: `extract_tool_context_summary_collects_prompt_mode_tool_result_names` in `src/channels/mod.rs`

#### Scenario: Extract tool context summary respects start index

- WHEN a start index is provided
- THEN MUST only collect from that index onward
- Test: `extract_tool_context_summary_respects_start_index` in `src/channels/mod.rs`

#### Scenario: Strip isolated tool JSON artifacts removes tool calls and results

- WHEN tool JSON artifacts are present in the message
- THEN MUST strip tool call and result artifacts
- Test: `strip_isolated_tool_json_artifacts_removes_tool_calls_and_results` in `src/channels/mod.rs`

#### Scenario: Should expose internal tool details matches explicit requests

- WHEN user explicitly requests tool details
- THEN MUST detect and expose them
- Test: `should_expose_internal_tool_details_matches_explicit_requests` in `src/channels/mod.rs`

#### Scenario: Should expose internal tool details respects negative requests

- WHEN user explicitly requests hiding tool details
- THEN MUST respect the negative request
- Test: `should_expose_internal_tool_details_respects_negative_requests` in `src/channels/mod.rs`

#### Scenario: Split internal progress delta detects sentinel prefix

- WHEN a progress delta contains a sentinel prefix
- THEN MUST split it correctly
- Test: `split_internal_progress_delta_detects_sentinel_prefix` in `src/channels/mod.rs`

#### Scenario: Effective progress mode defaults non-Telegram to off

- WHEN the channel is not Telegram
- THEN progress mode MUST default to off
- Test: `effective_progress_mode_defaults_non_telegram_to_off` in `src/channels/mod.rs`

#### Scenario: Effective progress mode uses Telegram runtime setting

- WHEN the channel is Telegram
- THEN MUST use the Telegram runtime progress mode setting
- Test: `effective_progress_mode_uses_telegram_runtime_setting` in `src/channels/mod.rs`

#### Scenario: Upsert progress section replaces existing block

- WHEN a progress section already exists
- THEN MUST replace the existing block
- Test: `upsert_progress_section_replaces_existing_block` in `src/channels/mod.rs`

#### Scenario: Channel system prompt includes visibility policy

- WHEN the channel system prompt is built
- THEN MUST include the visibility policy
- Test: `build_channel_system_prompt_includes_visibility_policy` in `src/channels/mod.rs`

#### Scenario: Strip preserves non-tool JSON

- WHEN JSON in the message is not a tool artifact
- THEN MUST preserve it
- Test: `strip_isolated_tool_json_artifacts_preserves_non_tool_json` in `src/channels/mod.rs`

#### Scenario: Sanitize removes tool call tags and tool JSON artifacts

- WHEN channel response contains tool call tags and JSON artifacts
- THEN MUST remove both
- Test: `sanitize_channel_response_removes_tool_call_tags_and_tool_json_artifacts` in `src/channels/mod.rs`

#### Scenario: Sanitize redacts detected credentials

- WHEN channel response contains credential-like strings
- THEN MUST redact them
- Test: `sanitize_channel_response_redacts_detected_credentials` in `src/channels/mod.rs`

#### Scenario: Sanitize skips leak scan when disabled

- WHEN outbound leak guard is disabled
- THEN MUST skip the leak scan
- Test: `sanitize_channel_response_skips_leak_scan_when_disabled` in `src/channels/mod.rs`

#### Scenario: Sanitize blocks detected credentials when configured

- WHEN blocking mode is configured for credential detection
- THEN MUST block the entire response
- Test: `sanitize_channel_response_blocks_detected_credentials_when_configured` in `src/channels/mod.rs`

#### Scenario: AIEOS identity from file

- WHEN AIEOS identity is configured from a file
- THEN MUST load and parse the identity
- Test: `aieos_identity_from_file` in `src/channels/mod.rs`

#### Scenario: AIEOS identity from inline config

- WHEN AIEOS identity is configured inline
- THEN MUST parse the inline identity
- Test: `aieos_identity_from_inline` in `src/channels/mod.rs`

#### Scenario: AIEOS fallback to OpenClaw on parse error

- WHEN AIEOS identity parsing fails
- THEN MUST fall back to OpenClaw identity
- Test: `aieos_fallback_to_openclaw_on_parse_error` in `src/channels/mod.rs`

#### Scenario: AIEOS empty uses OpenClaw

- WHEN AIEOS identity is empty
- THEN MUST use OpenClaw identity
- Test: `aieos_empty_uses_openclaw` in `src/channels/mod.rs`

#### Scenario: OpenClaw format uses bootstrap files

- WHEN OpenClaw identity is used
- THEN MUST incorporate bootstrap files
- Test: `openclaw_format_uses_bootstrap_files` in `src/channels/mod.rs`

#### Scenario: OpenClaw extra files are injected

- WHEN extra identity files are configured
- THEN MUST inject them
- Test: `openclaw_extra_files_are_injected` in `src/channels/mod.rs`

#### Scenario: OpenClaw extra files reject unsafe paths

- WHEN extra identity files contain unsafe paths (traversal)
- THEN MUST reject them
- Test: `openclaw_extra_files_reject_unsafe_paths` in `src/channels/mod.rs`

#### Scenario: None identity config uses OpenClaw

- WHEN identity config is None
- THEN MUST use OpenClaw identity
- Test: `none_identity_config_uses_openclaw` in `src/channels/mod.rs`

#### Scenario: Health classification OK true

- WHEN health check succeeds
- THEN MUST classify as OK
- Test: `classify_health_ok_true` in `src/channels/mod.rs`

#### Scenario: Health classification OK false

- WHEN health check fails
- THEN MUST classify as not OK
- Test: `classify_health_ok_false` in `src/channels/mod.rs`

#### Scenario: Health classification timeout

- WHEN health check times out
- THEN MUST classify as timeout
- Test: `classify_health_timeout` in `src/channels/mod.rs`

#### Scenario: Collect configured channels includes Mattermost

- WHEN Mattermost is configured
- THEN MUST include it in collected channels
- Test: `collect_configured_channels_includes_mattermost_when_configured` in `src/channels/mod.rs`

#### Scenario: Collect configured channels includes DingTalk

- WHEN DingTalk is configured
- THEN MUST include it in collected channels
- Test: `collect_configured_channels_includes_dingtalk_when_configured` in `src/channels/mod.rs`

#### Scenario: Supervised listener marks error and restarts on failures

- WHEN a supervised listener encounters an error
- THEN MUST mark the error and restart
- Test: `supervised_listener_marks_error_and_restarts_on_failures` in `src/channels/mod.rs`

#### Scenario: Supervised listener refreshes health while running

- WHEN a supervised listener is running
- THEN MUST periodically refresh health status
- Test: `supervised_listener_refreshes_health_while_running` in `src/channels/mod.rs`

#### Scenario: Daemon restart systemd args regression

- WHEN systemd daemon restart is triggered
- THEN MUST use correct systemd arguments
- Test: `maybe_restart_daemon_systemd_args_regression` in `src/channels/mod.rs`

#### Scenario: Daemon restart openrc args regression

- WHEN openrc daemon restart is triggered
- THEN MUST use correct openrc arguments
- Test: `maybe_restart_daemon_openrc_args_regression` in `src/channels/mod.rs`

#### Scenario: Normalize merges consecutive user turns (standalone)

- WHEN consecutive user turns appear in history
- THEN MUST merge them
- Test: `normalize_merges_consecutive_user_turns` in `src/channels/mod.rs`

#### Scenario: Normalize preserves strict alternation

- WHEN history already has strict alternation
- THEN MUST preserve it unchanged
- Test: `normalize_preserves_strict_alternation` in `src/channels/mod.rs`

#### Scenario: Normalize merges multiple consecutive user turns

- WHEN three or more consecutive user turns appear
- THEN MUST merge all into one
- Test: `normalize_merges_multiple_consecutive_user_turns` in `src/channels/mod.rs`

#### Scenario: Normalize handles empty input

- WHEN history is empty
- THEN MUST return empty
- Test: `normalize_empty_input` in `src/channels/mod.rs`

#### Scenario: E2E photo attachment rejected by non-vision provider

- WHEN a photo attachment is sent to a non-vision provider
- THEN MUST reject with appropriate error
- Test: `e2e_photo_attachment_rejected_by_non_vision_provider` in `src/channels/mod.rs`

#### Scenario: E2E failed vision turn does not poison follow-up text turn

- WHEN a vision turn fails
- THEN follow-up text turns MUST not be poisoned
- Test: `e2e_failed_vision_turn_does_not_poison_follow_up_text_turn` in `src/channels/mod.rs`

### REQ-CHAN-006: Telegram Channel

`TelegramChannel` MUST support message sending, webhook listening, typing indicators, reactions, approval prompts, markdown formatting, attachment handling, voice transcription, mention-only mode, and message splitting.

#### Scenario: Channel name is "telegram"

- WHEN name() is called
- THEN MUST return "telegram"
- Test: `telegram_channel_name` in `src/channels/telegram.rs`

#### Scenario: Ack reaction is from configured pool

- WHEN a random ack reaction is selected
- THEN MUST be from the configured pool
- Test: `random_telegram_ack_reaction_is_from_pool` in `src/channels/telegram.rs`

#### Scenario: Ack reaction request shape

- WHEN an ack reaction request is built
- THEN MUST have the correct shape
- Test: `telegram_ack_reaction_request_shape` in `src/channels/telegram.rs`

#### Scenario: Extract update message target parses IDs

- WHEN a Telegram update is parsed
- THEN MUST extract the message target IDs
- Test: `telegram_extract_update_message_target_parses_ids` in `src/channels/telegram.rs`

#### Scenario: Typing handle starts as None

- WHEN a channel is created
- THEN typing handle MUST start as None
- Test: `typing_handle_starts_as_none` in `src/channels/telegram.rs`

#### Scenario: Stop typing clears handle

- WHEN stop_typing is called
- THEN MUST clear the typing handle
- Test: `stop_typing_clears_handle` in `src/channels/telegram.rs`

#### Scenario: Start typing replaces previous handle

- WHEN start_typing is called while already typing
- THEN MUST replace the previous handle
- Test: `start_typing_replaces_previous_handle` in `src/channels/telegram.rs`

#### Scenario: Draft updates respect stream mode

- WHEN stream mode is configured
- THEN supports_draft_updates MUST respect the setting
- Test: `supports_draft_updates_respects_stream_mode` in `src/channels/telegram.rs`

#### Scenario: Send draft returns None when stream mode off

- WHEN stream mode is off
- THEN send_draft MUST return None
- Test: `send_draft_returns_none_when_stream_mode_off` in `src/channels/telegram.rs`

#### Scenario: Update draft rate limits short-circuit network

- WHEN update_draft is called within the rate limit window
- THEN MUST short-circuit without making a network call
- Test: `update_draft_rate_limit_short_circuits_network` in `src/channels/telegram.rs`

#### Scenario: Update draft UTF-8 truncation safe for multibyte

- WHEN draft text is truncated
- THEN MUST truncate at a valid UTF-8 boundary
- Test: `update_draft_utf8_truncation_is_safe_for_multibyte_text` in `src/channels/telegram.rs`

#### Scenario: Finalize draft falls back on invalid message ID

- WHEN finalize_draft receives an invalid message ID
- THEN MUST fall back to chunk send
- Test: `finalize_draft_invalid_message_id_falls_back_to_chunk_send` in `src/channels/telegram.rs`

#### Scenario: API URL construction

- WHEN API URL is built
- THEN MUST include the bot token
- Test: `telegram_api_url` in `src/channels/telegram.rs`

#### Scenario: Custom base URL

- WHEN a custom base URL is configured
- THEN MUST use it for API calls
- Test: `telegram_custom_base_url` in `src/channels/telegram.rs`

#### Scenario: Sanitize error redacts bot token in URL

- WHEN an error message contains a bot token URL
- THEN MUST redact the token
- Test: `sanitize_telegram_error_redacts_bot_token_in_url` in `src/channels/telegram.rs`

#### Scenario: Sanitize error does not redact non-token bot path

- WHEN an error message contains a non-token bot path
- THEN MUST not redact
- Test: `sanitize_telegram_error_does_not_redact_non_token_bot_path` in `src/channels/telegram.rs`

#### Scenario: Markdown-to-HTML escapes quotes in link href

- WHEN markdown contains links with quotes
- THEN MUST escape quotes in HTML href
- Test: `telegram_markdown_to_html_escapes_quotes_in_link_href` in `src/channels/telegram.rs`

#### Scenario: Markdown-to-HTML escapes quotes in plain text

- WHEN markdown contains plain text with quotes
- THEN MUST escape quotes in HTML output
- Test: `telegram_markdown_to_html_escapes_quotes_in_plain_text` in `src/channels/telegram.rs`

#### Scenario: Markdown-to-HTML code block drops language attribute

- WHEN markdown contains code blocks with language hints
- THEN MUST drop the language attribute in HTML
- Test: `telegram_markdown_to_html_code_block_drops_language_attribute` in `src/channels/telegram.rs`

#### Scenario: User allowed with wildcard

- WHEN allowlist contains wildcard
- THEN all users MUST be allowed
- Test: `telegram_user_allowed_wildcard` in `src/channels/telegram.rs`

#### Scenario: User allowed with specific entry

- WHEN allowlist contains a specific user
- THEN only that user MUST be allowed
- Test: `telegram_user_allowed_specific` in `src/channels/telegram.rs`

#### Scenario: User allowed with @ prefix in config

- WHEN allowlist entry has @ prefix
- THEN MUST match without the prefix
- Test: `telegram_user_allowed_with_at_prefix_in_config` in `src/channels/telegram.rs`

#### Scenario: User denied with empty allowlist

- WHEN allowlist is empty
- THEN all users MUST be denied
- Test: `telegram_user_denied_empty` in `src/channels/telegram.rs`

#### Scenario: User match is exact not substring

- WHEN a username is a substring of an allowlisted user
- THEN MUST not match
- Test: `telegram_user_exact_match_not_substring` in `src/channels/telegram.rs`

#### Scenario: Empty string user denied

- WHEN user ID is empty string
- THEN MUST be denied
- Test: `telegram_user_empty_string_denied` in `src/channels/telegram.rs`

#### Scenario: User matching is case sensitive

- WHEN username case differs from allowlist
- THEN MUST not match
- Test: `telegram_user_case_sensitive` in `src/channels/telegram.rs`

#### Scenario: Wildcard with specific users

- WHEN allowlist contains wildcard and specific users
- THEN wildcard MUST take precedence
- Test: `telegram_wildcard_with_specific_users` in `src/channels/telegram.rs`

#### Scenario: User allowed by numeric ID identity

- WHEN allowlist contains a numeric user ID
- THEN MUST match by numeric ID
- Test: `telegram_user_allowed_by_numeric_id_identity` in `src/channels/telegram.rs`

#### Scenario: User denied when none of identities match

- WHEN no identities match the allowlist
- THEN MUST deny
- Test: `telegram_user_denied_when_none_of_identities_match` in `src/channels/telegram.rs`

#### Scenario: Pairing enabled with empty allowlist

- WHEN allowlist is empty
- THEN pairing MUST be enabled
- Test: `telegram_pairing_enabled_with_empty_allowlist` in `src/channels/telegram.rs`

#### Scenario: Pairing disabled with nonempty allowlist

- WHEN allowlist is nonempty
- THEN pairing MUST be disabled
- Test: `telegram_pairing_disabled_with_nonempty_allowlist` in `src/channels/telegram.rs`

#### Scenario: Extract bind code from plain command

- WHEN a plain bind command is sent
- THEN MUST extract the bind code
- Test: `telegram_extract_bind_code_plain_command` in `src/channels/telegram.rs`

#### Scenario: Extract bind code supports bot mention

- WHEN a bind command includes a bot mention
- THEN MUST extract the bind code
- Test: `telegram_extract_bind_code_supports_bot_mention` in `src/channels/telegram.rs`

#### Scenario: Extract bind code rejects invalid forms

- WHEN an invalid bind command is sent
- THEN MUST reject it
- Test: `telegram_extract_bind_code_rejects_invalid_forms` in `src/channels/telegram.rs`

#### Scenario: Parse attachment markers extracts multiple types

- WHEN message contains multiple attachment markers
- THEN MUST extract all types
- Test: `parse_attachment_markers_extracts_multiple_types` in `src/channels/telegram.rs`

#### Scenario: Parse attachment markers deduplicates

- WHEN message contains duplicate attachment targets
- THEN MUST deduplicate them
- Test: `parse_attachment_markers_deduplicates_duplicate_targets` in `src/channels/telegram.rs`

#### Scenario: Parse attachment markers keeps invalid markers in text

- WHEN message contains invalid attachment markers
- THEN MUST keep them as plain text
- Test: `parse_attachment_markers_keeps_invalid_markers_in_text` in `src/channels/telegram.rs`

#### Scenario: Parse attachment markers handles brackets in filename

- WHEN attachment filename contains brackets
- THEN MUST handle them correctly
- Test: `parse_attachment_markers_handles_brackets_in_filename` in `src/channels/telegram.rs`

#### Scenario: Unclosed bracket falls back to text

- WHEN an attachment marker has an unclosed bracket
- THEN MUST fall back to plain text
- Test: `parse_attachment_markers_unclosed_bracket_falls_back_to_text` in `src/channels/telegram.rs`

#### Scenario: Path-only attachment detects existing file

- WHEN message is just a file path
- THEN MUST detect it as an attachment
- Test: `parse_path_only_attachment_detects_existing_file` in `src/channels/telegram.rs`

#### Scenario: Path-only attachment rejects sentence text

- WHEN message is a sentence (not a file path)
- THEN MUST not detect it as an attachment
- Test: `parse_path_only_attachment_rejects_sentence_text` in `src/channels/telegram.rs`

#### Scenario: Sanitize attachment filename strips path traversal

- WHEN attachment filename contains path traversal
- THEN MUST strip it
- Test: `sanitize_attachment_filename_strips_path_traversal` in `src/channels/telegram.rs`

#### Scenario: Resolve workspace attachment rejects escape

- WHEN attachment path tries to escape workspace
- THEN MUST reject; workspace files MUST be accepted
- Test: `resolve_workspace_attachment_path_rejects_escape_and_accepts_workspace_file` in `src/channels/telegram.rs`

#### Scenario: Resolve workspace attachment accepts workspace prefix mapping

- WHEN attachment path uses workspace prefix
- THEN MUST resolve correctly
- Test: `resolve_workspace_attachment_path_accepts_workspace_prefix_mapping` in `src/channels/telegram.rs`

#### Scenario: Resolve workspace attachment output rejects symlinked save dir

- WHEN save directory is a symlink
- THEN MUST reject
- Test: `resolve_workspace_attachment_output_path_rejects_symlinked_save_dir` in `src/channels/telegram.rs`

#### Scenario: Resolve workspace attachment output rejects symlink target file

- WHEN target file is a symlink
- THEN MUST reject
- Test: `resolve_workspace_attachment_output_path_rejects_symlink_target_file` in `src/channels/telegram.rs`

#### Scenario: Infer attachment kind from target detects document extension

- WHEN target has a document extension
- THEN MUST infer document kind
- Test: `infer_attachment_kind_from_target_detects_document_extension` in `src/channels/telegram.rs`

#### Scenario: Parse update message uses chat_id as reply_target

- WHEN parsing an update message
- THEN MUST use chat_id as reply_target
- Test: `parse_update_message_uses_chat_id_as_reply_target` in `src/channels/telegram.rs`

#### Scenario: Parse update allows numeric ID without username

- WHEN update has numeric ID but no username
- THEN MUST allow it
- Test: `parse_update_message_allows_numeric_id_without_username` in `src/channels/telegram.rs`

#### Scenario: Parse update extracts thread ID for forum topic

- WHEN update is in a forum topic
- THEN MUST extract the thread ID
- Test: `parse_update_message_extracts_thread_id_for_forum_topic` in `src/channels/telegram.rs`

#### Scenario: Parse approval callback maps approve and deny

- WHEN approval callback is received
- THEN MUST map approve and deny actions
- Test: `parse_approval_callback_command_maps_approve_and_deny` in `src/channels/telegram.rs`

#### Scenario: Parse approval callback trims and rejects empty IDs

- WHEN approval callback has empty IDs
- THEN MUST reject them
- Test: `parse_approval_callback_command_trims_and_rejects_empty_ids` in `src/channels/telegram.rs`

#### Scenario: Build typing action body uses plain chat_id

- WHEN typing action body is built
- THEN MUST use plain chat_id and optional thread_id
- Test: `build_typing_action_body_uses_plain_chat_id_and_optional_thread_id` in `src/channels/telegram.rs`

#### Scenario: Build typing action body without thread

- WHEN typing action body is built without thread
- THEN MUST not emit thread_id
- Test: `build_typing_action_body_without_thread_does_not_emit_thread_id` in `src/channels/telegram.rs`

#### Scenario: Parse approval callback query builds runtime command

- WHEN an approval callback query is received
- THEN MUST build a runtime command message
- Test: `try_parse_approval_callback_query_builds_runtime_command_message` in `src/channels/telegram.rs`

#### Scenario: API URL for send_document

- WHEN sending a document
- THEN MUST use the sendDocument API URL
- Test: `telegram_api_url_send_document` in `src/channels/telegram.rs`

#### Scenario: API URL for send_photo

- WHEN sending a photo
- THEN MUST use the sendPhoto API URL
- Test: `telegram_api_url_send_photo` in `src/channels/telegram.rs`

#### Scenario: API URL for send_video

- WHEN sending a video
- THEN MUST use the sendVideo API URL
- Test: `telegram_api_url_send_video` in `src/channels/telegram.rs`

#### Scenario: API URL for send_audio

- WHEN sending audio
- THEN MUST use the sendAudio API URL
- Test: `telegram_api_url_send_audio` in `src/channels/telegram.rs`

#### Scenario: API URL for send_voice

- WHEN sending a voice message
- THEN MUST use the sendVoice API URL
- Test: `telegram_api_url_send_voice` in `src/channels/telegram.rs`

#### Scenario: Send document bytes builds correct form

- WHEN sending document bytes
- THEN MUST build the correct multipart form
- Test: `telegram_send_document_bytes_builds_correct_form` in `src/channels/telegram.rs`

#### Scenario: Send photo bytes builds correct form

- WHEN sending photo bytes
- THEN MUST build the correct multipart form
- Test: `telegram_send_photo_bytes_builds_correct_form` in `src/channels/telegram.rs`

#### Scenario: Send document by URL builds correct JSON

- WHEN sending a document by URL
- THEN MUST build the correct JSON payload
- Test: `telegram_send_document_by_url_builds_correct_json` in `src/channels/telegram.rs`

#### Scenario: Send photo by URL builds correct JSON

- WHEN sending a photo by URL
- THEN MUST build the correct JSON payload
- Test: `telegram_send_photo_by_url_builds_correct_json` in `src/channels/telegram.rs`

#### Scenario: Send document for nonexistent file

- WHEN sending a nonexistent document file
- THEN MUST handle gracefully
- Test: `telegram_send_document_nonexistent_file` in `src/channels/telegram.rs`

#### Scenario: Send photo for nonexistent file

- WHEN sending a nonexistent photo file
- THEN MUST handle gracefully
- Test: `telegram_send_photo_nonexistent_file` in `src/channels/telegram.rs`

#### Scenario: Send video for nonexistent file

- WHEN sending a nonexistent video file
- THEN MUST handle gracefully
- Test: `telegram_send_video_nonexistent_file` in `src/channels/telegram.rs`

#### Scenario: Send audio for nonexistent file

- WHEN sending a nonexistent audio file
- THEN MUST handle gracefully
- Test: `telegram_send_audio_nonexistent_file` in `src/channels/telegram.rs`

#### Scenario: Send voice for nonexistent file

- WHEN sending a nonexistent voice file
- THEN MUST handle gracefully
- Test: `telegram_send_voice_nonexistent_file` in `src/channels/telegram.rs`

#### Scenario: Split short message stays intact

- WHEN message is under the limit
- THEN MUST not split
- Test: `telegram_split_short_message` in `src/channels/telegram.rs`

#### Scenario: Split at exact limit

- WHEN message is exactly at the limit
- THEN MUST not split
- Test: `telegram_split_exact_limit` in `src/channels/telegram.rs`

#### Scenario: Split over limit

- WHEN message exceeds the limit
- THEN MUST split into chunks
- Test: `telegram_split_over_limit` in `src/channels/telegram.rs`

#### Scenario: Split at word boundary

- WHEN splitting is needed
- THEN MUST prefer word boundaries
- Test: `telegram_split_at_word_boundary` in `src/channels/telegram.rs`

#### Scenario: Split at newline

- WHEN splitting is needed and newlines are present
- THEN MUST prefer newline boundaries
- Test: `telegram_split_at_newline` in `src/channels/telegram.rs`

#### Scenario: Split preserves all content

- WHEN message is split
- THEN reassembled chunks MUST equal the original
- Test: `telegram_split_preserves_content` in `src/channels/telegram.rs`

#### Scenario: Split empty message

- WHEN message is empty
- THEN MUST return empty or single empty chunk
- Test: `telegram_split_empty_message` in `src/channels/telegram.rs`

#### Scenario: Split very long message

- WHEN message is very long
- THEN MUST split into multiple chunks
- Test: `telegram_split_very_long_message` in `src/channels/telegram.rs`

#### Scenario: Send document bytes with caption

- WHEN sending document bytes with a caption
- THEN MUST include caption in form
- Test: `telegram_send_document_bytes_with_caption` in `src/channels/telegram.rs`

#### Scenario: Send photo bytes with caption

- WHEN sending photo bytes with a caption
- THEN MUST include caption in form
- Test: `telegram_send_photo_bytes_with_caption` in `src/channels/telegram.rs`

#### Scenario: Send document bytes with empty file

- WHEN sending empty document bytes
- THEN MUST handle gracefully
- Test: `telegram_send_document_bytes_empty_file` in `src/channels/telegram.rs`

#### Scenario: Send document bytes with empty filename

- WHEN sending document bytes with empty filename
- THEN MUST handle gracefully
- Test: `telegram_send_document_bytes_empty_filename` in `src/channels/telegram.rs`

#### Scenario: Send document bytes with empty chat_id

- WHEN sending document bytes with empty chat_id
- THEN MUST handle gracefully
- Test: `telegram_send_document_bytes_empty_chat_id` in `src/channels/telegram.rs`

#### Scenario: Message ID format includes chat and message ID

- WHEN a Telegram message ID is generated
- THEN MUST include both chat and message ID
- Test: `telegram_message_id_format_includes_chat_and_message_id` in `src/channels/telegram.rs`

#### Scenario: Message ID is deterministic

- WHEN the same inputs are used
- THEN MUST produce the same ID
- Test: `telegram_message_id_is_deterministic` in `src/channels/telegram.rs`

#### Scenario: Different message produces different ID

- WHEN different message IDs are used
- THEN MUST produce different IDs
- Test: `telegram_message_id_different_message_different_id` in `src/channels/telegram.rs`

#### Scenario: Different chat produces different ID

- WHEN different chat IDs are used
- THEN MUST produce different IDs
- Test: `telegram_message_id_different_chat_different_id` in `src/channels/telegram.rs`

#### Scenario: Message ID has no UUID randomness

- WHEN a message ID is generated
- THEN MUST not contain UUID randomness
- Test: `telegram_message_id_no_uuid_randomness` in `src/channels/telegram.rs`

#### Scenario: Message ID handles zero message ID

- WHEN message ID is zero
- THEN MUST handle it correctly
- Test: `telegram_message_id_handles_zero_message_id` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags removes standard tags

- WHEN message contains standard tool call tags
- THEN MUST remove them
- Test: `strip_tool_call_tags_removes_standard_tags` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags removes alias tags

- WHEN message contains alias tool call tags
- THEN MUST remove them
- Test: `strip_tool_call_tags_removes_alias_tags` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags removes dash tags

- WHEN message contains dash-style tool call tags
- THEN MUST remove them
- Test: `strip_tool_call_tags_removes_dash_tags` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags removes tool_call tags

- WHEN message contains tool_call tags
- THEN MUST remove them
- Test: `strip_tool_call_tags_removes_tool_call_tags` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags removes invoke tags

- WHEN message contains invoke tags
- THEN MUST remove them
- Test: `strip_tool_call_tags_removes_invoke_tags` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags handles multiple tags

- WHEN message contains multiple tool call tags
- THEN MUST remove all of them
- Test: `strip_tool_call_tags_handles_multiple_tags` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags handles mixed tags

- WHEN message contains mixed types of tool call tags
- THEN MUST remove all of them
- Test: `strip_tool_call_tags_handles_mixed_tags` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags preserves normal text

- WHEN message contains no tool call tags
- THEN MUST preserve the text unchanged
- Test: `strip_tool_call_tags_preserves_normal_text` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags handles unclosed tags

- WHEN message contains unclosed tool call tags
- THEN MUST handle gracefully
- Test: `strip_tool_call_tags_handles_unclosed_tags` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags handles unclosed with JSON

- WHEN message contains unclosed tool call tags with JSON content
- THEN MUST handle gracefully
- Test: `strip_tool_call_tags_handles_unclosed_tool_call_with_json` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags handles mismatched close tag

- WHEN message contains mismatched close tags
- THEN MUST handle gracefully
- Test: `strip_tool_call_tags_handles_mismatched_close_tag` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags cleans extra newlines

- WHEN stripping leaves extra newlines
- THEN MUST clean them
- Test: `strip_tool_call_tags_cleans_extra_newlines` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags handles empty input

- WHEN input is empty
- THEN MUST return empty
- Test: `strip_tool_call_tags_handles_empty_input` in `src/channels/telegram.rs`

#### Scenario: Strip tool call tags handles only tags

- WHEN input consists only of tool call tags
- THEN MUST return empty or whitespace
- Test: `strip_tool_call_tags_handles_only_tags` in `src/channels/telegram.rs`

#### Scenario: Contains bot mention finds mention

- WHEN message contains a bot mention
- THEN MUST detect it
- Test: `telegram_contains_bot_mention_finds_mention` in `src/channels/telegram.rs`

#### Scenario: Contains bot mention no false positives

- WHEN message does not contain a bot mention
- THEN MUST not detect one
- Test: `telegram_contains_bot_mention_no_false_positives` in `src/channels/telegram.rs`

#### Scenario: Normalize incoming strips mention

- WHEN incoming content contains a mention
- THEN MUST strip it
- Test: `telegram_normalize_incoming_content_strips_mention` in `src/channels/telegram.rs`

#### Scenario: Normalize incoming handles multiple mentions

- WHEN incoming content contains multiple mentions
- THEN MUST strip all of them
- Test: `telegram_normalize_incoming_content_handles_multiple_mentions` in `src/channels/telegram.rs`

#### Scenario: Normalize incoming returns None for empty

- WHEN incoming content is empty after stripping
- THEN MUST return None
- Test: `telegram_normalize_incoming_content_returns_none_for_empty` in `src/channels/telegram.rs`

#### Scenario: Mention-only group requires exact mention

- WHEN mention-only mode is enabled in a group
- THEN MUST require exact bot mention
- Test: `parse_update_message_mention_only_group_requires_exact_mention` in `src/channels/telegram.rs`

#### Scenario: Mention-only strips mention and drops empty

- WHEN mention is stripped and result is empty
- THEN MUST drop the message
- Test: `parse_update_message_mention_only_group_strips_mention_and_drops_empty` in `src/channels/telegram.rs`

#### Scenario: Mention-only allows configured sender without mention

- WHEN a configured sender sends without mention in mention-only mode
- THEN MUST allow the message
- Test: `parse_update_message_mention_only_group_allows_configured_sender_without_mention` in `src/channels/telegram.rs`

#### Scenario: Mention-only gate allows configured sender for non-text messages

- WHEN a configured sender sends non-text in mention-only mode
- THEN MUST allow it
- Test: `passes_mention_only_gate_allows_configured_sender_for_non_text_messages` in `src/channels/telegram.rs`

#### Scenario: Mention-only gate rejects non-mentioned non-bypassed non-text

- WHEN a non-bypassed sender sends non-text without mention
- THEN MUST reject it
- Test: `passes_mention_only_gate_rejects_non_mentioned_non_bypassed_non_text_messages` in `src/channels/telegram.rs`

#### Scenario: Is group message detects groups

- WHEN message is from a group chat
- THEN MUST detect it as a group message
- Test: `telegram_is_group_message_detects_groups` in `src/channels/telegram.rs`

#### Scenario: Mention-only enabled by config

- WHEN mention-only is configured
- THEN MUST be enabled
- Test: `telegram_mention_only_enabled_by_config` in `src/channels/telegram.rs`

#### Scenario: Skip unauthorized prompt for non-mentioned group message

- WHEN a non-mentioned message arrives in mention-only group
- THEN MUST skip the unauthorized prompt
- Test: `should_skip_unauthorized_prompt_for_non_mentioned_group_message` in `src/channels/telegram.rs`

#### Scenario: Do not skip unauthorized prompt for mentioned group message

- WHEN a mentioned message arrives in mention-only group
- THEN MUST not skip the unauthorized prompt
- Test: `should_not_skip_unauthorized_prompt_for_mentioned_group_message` in `src/channels/telegram.rs`

#### Scenario: Do not skip unauthorized prompt outside group mention-only

- WHEN outside of group mention-only context
- THEN MUST not skip the unauthorized prompt
- Test: `should_not_skip_unauthorized_prompt_outside_group_mention_only` in `src/channels/telegram.rs`

#### Scenario: Do not skip unauthorized prompt for group sender trigger override

- WHEN sender has a trigger override in mention-only group
- THEN MUST not skip the unauthorized prompt
- Test: `should_not_skip_unauthorized_prompt_for_group_sender_trigger_override` in `src/channels/telegram.rs`

#### Scenario: Mention-only group photo without caption is ignored

- WHEN a photo without caption arrives in mention-only group
- THEN MUST ignore it
- Test: `telegram_mention_only_group_photo_without_caption_is_ignored` in `src/channels/telegram.rs`

#### Scenario: Mention-only group photo with caption without mention is ignored

- WHEN a photo with caption but no mention arrives in mention-only group
- THEN MUST ignore it
- Test: `telegram_mention_only_group_photo_with_caption_without_mention_is_ignored` in `src/channels/telegram.rs`

#### Scenario: Mention-only private chat photo still works

- WHEN a photo arrives in private chat (not group)
- THEN MUST still work regardless of mention-only setting
- Test: `telegram_mention_only_private_chat_photo_still_works` in `src/channels/telegram.rs`

#### Scenario: Split code block at boundary

- WHEN splitting happens at a code block boundary
- THEN MUST handle code block continuation correctly
- Test: `telegram_split_code_block_at_boundary` in `src/channels/telegram.rs`

#### Scenario: Split single long word

- WHEN message is a single very long word
- THEN MUST hard-split it
- Test: `telegram_split_single_long_word` in `src/channels/telegram.rs`

#### Scenario: Split exactly at limit no split

- WHEN message is exactly at the limit
- THEN MUST not split
- Test: `telegram_split_exactly_at_limit_no_split` in `src/channels/telegram.rs`

#### Scenario: Split one over limit

- WHEN message is one byte over the limit
- THEN MUST split
- Test: `telegram_split_one_over_limit` in `src/channels/telegram.rs`

#### Scenario: Split many short lines

- WHEN message has many short lines
- THEN MUST split at appropriate newline boundaries
- Test: `telegram_split_many_short_lines` in `src/channels/telegram.rs`

#### Scenario: Split only whitespace

- WHEN message is only whitespace
- THEN MUST handle correctly
- Test: `telegram_split_only_whitespace` in `src/channels/telegram.rs`

#### Scenario: Split emoji at boundary

- WHEN an emoji falls at the split boundary
- THEN MUST not split the emoji
- Test: `telegram_split_emoji_at_boundary` in `src/channels/telegram.rs`

#### Scenario: Split consecutive newlines

- WHEN message has consecutive newlines at boundary
- THEN MUST handle correctly
- Test: `telegram_split_consecutive_newlines` in `src/channels/telegram.rs`

#### Scenario: Parse voice metadata extracts voice

- WHEN update contains voice data
- THEN MUST extract voice metadata
- Test: `parse_voice_metadata_extracts_voice` in `src/channels/telegram.rs`

#### Scenario: Parse voice metadata extracts audio

- WHEN update contains audio data
- THEN MUST extract audio metadata
- Test: `parse_voice_metadata_extracts_audio` in `src/channels/telegram.rs`

#### Scenario: Parse voice metadata returns None for text

- WHEN update is a text message
- THEN MUST return None for voice metadata
- Test: `parse_voice_metadata_returns_none_for_text` in `src/channels/telegram.rs`

#### Scenario: Parse voice metadata defaults duration to zero

- WHEN voice duration is not present
- THEN MUST default to zero
- Test: `parse_voice_metadata_defaults_duration_to_zero` in `src/channels/telegram.rs`

#### Scenario: Infer voice filename prefers hint with extension

- WHEN a filename hint has an extension
- THEN MUST use it
- Test: `infer_voice_filename_prefers_hint_with_extension` in `src/channels/telegram.rs`

#### Scenario: Infer voice filename uses MIME extension when path has none

- WHEN path has no extension but MIME type is known
- THEN MUST use the MIME type extension
- Test: `infer_voice_filename_uses_mime_extension_when_path_has_none` in `src/channels/telegram.rs`

#### Scenario: Infer voice filename falls back for audio without hints

- WHEN no filename hints are available
- THEN MUST fall back to a default
- Test: `infer_voice_filename_falls_back_for_audio_without_hints` in `src/channels/telegram.rs`

#### Scenario: Extract sender info with username

- WHEN update has a username
- THEN MUST extract it as sender info
- Test: `extract_sender_info_with_username` in `src/channels/telegram.rs`

#### Scenario: Extract sender info without username

- WHEN update has no username
- THEN MUST fall back to first name or ID
- Test: `extract_sender_info_without_username` in `src/channels/telegram.rs`

#### Scenario: Extract reply context for text message

- WHEN a text message replies to another
- THEN MUST extract reply context
- Test: `extract_reply_context_text_message` in `src/channels/telegram.rs`

#### Scenario: Extract reply context for voice message

- WHEN a voice message replies to another
- THEN MUST extract reply context
- Test: `extract_reply_context_voice_message` in `src/channels/telegram.rs`

#### Scenario: Extract reply context for no reply

- WHEN message is not a reply
- THEN MUST return None
- Test: `extract_reply_context_no_reply` in `src/channels/telegram.rs`

#### Scenario: Extract reply context no username uses first name

- WHEN reply target has no username
- THEN MUST use first name
- Test: `extract_reply_context_no_username_uses_first_name` in `src/channels/telegram.rs`

#### Scenario: Extract reply context voice with cached transcription

- WHEN reply is to a voice message with cached transcription
- THEN MUST include the transcription
- Test: `extract_reply_context_voice_with_cached_transcription` in `src/channels/telegram.rs`

#### Scenario: Parse update includes reply context

- WHEN an update message is parsed and it replies to another
- THEN MUST include reply context in the parsed result
- Test: `parse_update_message_includes_reply_context` in `src/channels/telegram.rs`

#### Scenario: With transcription sets config when enabled

- WHEN transcription is enabled
- THEN MUST set the transcription config
- Test: `with_transcription_sets_config_when_enabled` in `src/channels/telegram.rs`

#### Scenario: With transcription skips when disabled

- WHEN transcription is disabled
- THEN MUST skip transcription config
- Test: `with_transcription_skips_when_disabled` in `src/channels/telegram.rs`

#### Scenario: Try parse voice returns None when transcription disabled

- WHEN transcription is disabled and a voice message arrives
- THEN MUST return None
- Test: `try_parse_voice_message_returns_none_when_transcription_disabled` in `src/channels/telegram.rs`

#### Scenario: Try parse voice skips when duration exceeds limit

- WHEN voice duration exceeds the configured limit
- THEN MUST skip transcription
- Test: `try_parse_voice_message_skips_when_duration_exceeds_limit` in `src/channels/telegram.rs`

#### Scenario: Try parse voice rejects unauthorized sender before download

- WHEN an unauthorized sender sends a voice message
- THEN MUST reject before downloading
- Test: `try_parse_voice_message_rejects_unauthorized_sender_before_download` in `src/channels/telegram.rs`

#### Scenario: E2E live voice transcription and reply cache

- WHEN a voice message is transcribed end-to-end
- THEN MUST cache the transcription for reply context
- Test: `e2e_live_voice_transcription_and_reply_cache` in `src/channels/telegram.rs`

#### Scenario: Parse attachment metadata detects document

- WHEN update contains a document attachment
- THEN MUST detect it
- Test: `parse_attachment_metadata_detects_document` in `src/channels/telegram.rs`

#### Scenario: Parse attachment metadata detects photo

- WHEN update contains a photo attachment
- THEN MUST detect it
- Test: `parse_attachment_metadata_detects_photo` in `src/channels/telegram.rs`

#### Scenario: Parse attachment metadata extracts caption

- WHEN attachment has a caption
- THEN MUST extract it
- Test: `parse_attachment_metadata_extracts_caption` in `src/channels/telegram.rs`

#### Scenario: Parse attachment metadata document without optional fields

- WHEN document attachment lacks optional fields
- THEN MUST handle gracefully
- Test: `parse_attachment_metadata_document_without_optional_fields` in `src/channels/telegram.rs`

#### Scenario: Parse attachment metadata returns None for text

- WHEN update is a text message
- THEN MUST return None for attachment metadata
- Test: `parse_attachment_metadata_returns_none_for_text` in `src/channels/telegram.rs`

#### Scenario: Parse attachment metadata returns None for voice

- WHEN update is a voice message
- THEN MUST return None for attachment metadata (voice is separate)
- Test: `parse_attachment_metadata_returns_none_for_voice` in `src/channels/telegram.rs`

#### Scenario: Parse attachment metadata handles empty photo array

- WHEN photo array is empty
- THEN MUST return None
- Test: `parse_attachment_metadata_empty_photo_array` in `src/channels/telegram.rs`

#### Scenario: With workspace dir sets field

- WHEN workspace dir is configured
- THEN MUST set the field
- Test: `with_workspace_dir_sets_field` in `src/channels/telegram.rs`

#### Scenario: Max file download bytes is 20MB

- WHEN checking max file download limit
- THEN MUST be 20MB
- Test: `telegram_max_file_download_bytes_is_20mb` in `src/channels/telegram.rs`

#### Scenario: Attachment photo content uses image marker

- WHEN a photo attachment is formatted
- THEN MUST use image marker
- Test: `attachment_photo_content_uses_image_marker` in `src/channels/telegram.rs`

#### Scenario: Attachment document content uses document label

- WHEN a document attachment is formatted
- THEN MUST use document label
- Test: `attachment_document_content_uses_document_label` in `src/channels/telegram.rs`

#### Scenario: Markdown file never produces image marker

- WHEN a markdown file is sent as attachment
- THEN MUST not produce an image marker
- Test: `markdown_file_never_produces_image_marker` in `src/channels/telegram.rs`

#### Scenario: Non-image photo falls back to document format

- WHEN a photo has a non-image extension
- THEN MUST fall back to document format
- Test: `non_image_photo_falls_back_to_document_format` in `src/channels/telegram.rs`

#### Scenario: Image extensions produce image marker

- WHEN attachment has an image extension
- THEN MUST produce image marker
- Test: `image_extensions_produce_image_marker` in `src/channels/telegram.rs`

#### Scenario: Markdown attachment not detected by multimodal image markers

- WHEN a markdown attachment is present
- THEN MUST not be detected as multimodal image
- Test: `markdown_attachment_not_detected_by_multimodal_image_markers` in `src/channels/telegram.rs`

#### Scenario: Is image extension recognizes images

- WHEN checking file extensions
- THEN MUST recognize standard image extensions
- Test: `is_image_extension_recognizes_images` in `src/channels/telegram.rs`

#### Scenario: Photo image marker detected by multimodal

- WHEN a photo with image marker is present
- THEN MUST be detected as multimodal image
- Test: `photo_image_marker_detected_by_multimodal` in `src/channels/telegram.rs`

#### Scenario: Photo image marker with caption

- WHEN a photo with image marker has a caption
- THEN MUST include caption in multimodal detection
- Test: `photo_image_marker_with_caption` in `src/channels/telegram.rs`

#### Scenario: E2E attachment saves file and formats content

- WHEN an attachment is processed end-to-end
- THEN MUST save the file and format content
- Test: `e2e_attachment_saves_file_and_formats_content` in `src/channels/telegram.rs`

#### Scenario: Groq provider rejects photo with vision error

- WHEN a photo is sent to a Groq provider without vision
- THEN MUST reject with vision error
- Test: `groq_provider_rejects_photo_with_vision_error` in `src/channels/telegram.rs`

#### Scenario: Document with image extension routes to image marker

- WHEN a document has an image extension
- THEN MUST route to image marker format
- Test: `document_with_image_extension_routes_to_image_marker` in `src/channels/telegram.rs`

#### Scenario: Document with non-image extension routes to document format

- WHEN a document has a non-image extension
- THEN MUST route to document format
- Test: `document_with_non_image_extension_routes_to_document_format` in `src/channels/telegram.rs`

### REQ-CHAN-007: Discord Channel

`DiscordChannel` MUST support message sending, webhook/gateway listening, reactions, channel routing, attachment handling, mention-only mode, message splitting, and typing indicators.

#### Scenario: Channel name is "discord"

- WHEN name() is called
- THEN MUST return "discord"
- Test: `discord_channel_name` in `src/channels/discord.rs`

#### Scenario: Base64 decode bot ID

- WHEN bot token is base64-decoded
- THEN MUST extract the bot user ID
- Test: `base64_decode_bot_id` in `src/channels/discord.rs`

#### Scenario: Bot user ID extraction

- WHEN extracting bot user ID from token
- THEN MUST extract correctly
- Test: `bot_user_id_extraction` in `src/channels/discord.rs`

#### Scenario: Empty allowlist denies everyone

- WHEN allowlist is empty
- THEN MUST deny all users
- Test: `empty_allowlist_denies_everyone` in `src/channels/discord.rs`

#### Scenario: Wildcard allows everyone

- WHEN allowlist contains wildcard
- THEN MUST allow all users
- Test: `wildcard_allows_everyone` in `src/channels/discord.rs`

#### Scenario: Specific allowlist filters

- WHEN allowlist contains specific users
- THEN MUST filter accordingly
- Test: `specific_allowlist_filters` in `src/channels/discord.rs`

#### Scenario: Allowlist is exact match not substring

- WHEN user ID is a substring of an allowlisted ID
- THEN MUST not match
- Test: `allowlist_is_exact_match_not_substring` in `src/channels/discord.rs`

#### Scenario: Allowlist empty string user ID

- WHEN user ID is empty string
- THEN MUST deny
- Test: `allowlist_empty_string_user_id` in `src/channels/discord.rs`

#### Scenario: Allowlist with wildcard and specific

- WHEN allowlist contains both wildcard and specific entries
- THEN wildcard MUST take precedence
- Test: `allowlist_with_wildcard_and_specific` in `src/channels/discord.rs`

#### Scenario: Allowlist is case sensitive

- WHEN user ID case differs from allowlist
- THEN MUST not match
- Test: `allowlist_case_sensitive` in `src/channels/discord.rs`

#### Scenario: Base64 decode empty string

- WHEN empty string is base64-decoded
- THEN MUST handle gracefully
- Test: `base64_decode_empty_string` in `src/channels/discord.rs`

#### Scenario: Base64 decode invalid chars

- WHEN invalid base64 is decoded
- THEN MUST handle gracefully
- Test: `base64_decode_invalid_chars` in `src/channels/discord.rs`

#### Scenario: Bot user ID from empty token

- WHEN token is empty
- THEN MUST return None
- Test: `bot_user_id_from_empty_token` in `src/channels/discord.rs`

#### Scenario: Contains bot mention supports plain and nick forms

- WHEN message contains bot mention in plain or nick form
- THEN MUST detect both forms
- Test: `contains_bot_mention_supports_plain_and_nick_forms` in `src/channels/discord.rs`

#### Scenario: Normalize incoming requires mention when enabled

- WHEN mention-only mode is enabled
- THEN MUST require bot mention in incoming content
- Test: `normalize_incoming_content_requires_mention_when_enabled` in `src/channels/discord.rs`

#### Scenario: Normalize incoming strips mentions and trims

- WHEN incoming content contains mentions
- THEN MUST strip them and trim whitespace
- Test: `normalize_incoming_content_strips_mentions_and_trims` in `src/channels/discord.rs`

#### Scenario: Normalize incoming rejects empty after strip

- WHEN content is empty after mention stripping
- THEN MUST reject
- Test: `normalize_incoming_content_rejects_empty_after_strip` in `src/channels/discord.rs`

#### Scenario: Group reply allowed sender IDs trims and deduplicates

- WHEN group reply sender IDs contain duplicates and whitespace
- THEN MUST trim and deduplicate
- Test: `normalize_group_reply_allowed_sender_ids_trims_and_deduplicates` in `src/channels/discord.rs`

#### Scenario: Group reply sender override matches exact and wildcard

- WHEN group reply sender is configured with exact and wildcard entries
- THEN MUST match accordingly
- Test: `group_reply_sender_override_matches_exact_and_wildcard` in `src/channels/discord.rs`

#### Scenario: Split empty message

- WHEN message is empty
- THEN MUST return empty or single chunk
- Test: `split_empty_message` in `src/channels/discord.rs`

#### Scenario: Split short message under limit

- WHEN message is under Discord's limit
- THEN MUST not split
- Test: `split_short_message_under_limit` in `src/channels/discord.rs`

#### Scenario: Split message exactly 2000 chars

- WHEN message is exactly 2000 characters
- THEN MUST not split
- Test: `split_message_exactly_2000_chars` in `src/channels/discord.rs`

#### Scenario: Split message just over limit

- WHEN message is just over 2000 characters
- THEN MUST split
- Test: `split_message_just_over_limit` in `src/channels/discord.rs`

#### Scenario: Split very long message

- WHEN message is very long
- THEN MUST split into multiple chunks
- Test: `split_very_long_message` in `src/channels/discord.rs`

#### Scenario: Split prefers newline break

- WHEN splitting is needed and newlines are present
- THEN MUST prefer newline as break point
- Test: `split_prefer_newline_break` in `src/channels/discord.rs`

#### Scenario: Split prefers space break

- WHEN splitting is needed and spaces are present
- THEN MUST prefer space as break point
- Test: `split_prefer_space_break` in `src/channels/discord.rs`

#### Scenario: Split without good break points hard splits

- WHEN no good break points exist
- THEN MUST hard-split at the limit
- Test: `split_without_good_break_points_hard_split` in `src/channels/discord.rs`

#### Scenario: Split handles multiple breaks

- WHEN message requires multiple splits
- THEN MUST handle all break points
- Test: `split_multiple_breaks` in `src/channels/discord.rs`

#### Scenario: Split preserves all content

- WHEN message is split
- THEN reassembled MUST equal original
- Test: `split_preserves_content` in `src/channels/discord.rs`

#### Scenario: Split handles unicode content

- WHEN message contains unicode characters
- THEN MUST split at valid character boundaries
- Test: `split_unicode_content` in `src/channels/discord.rs`

#### Scenario: Split newline too close to end

- WHEN a newline is very close to the chunk end
- THEN MUST handle it properly
- Test: `split_newline_too_close_to_end` in `src/channels/discord.rs`

#### Scenario: Split multibyte-only content without panics

- WHEN message contains only multibyte characters
- THEN MUST split without panicking
- Test: `split_multibyte_only_content_without_panics` in `src/channels/discord.rs`

#### Scenario: Split chunks always within Discord limit

- WHEN message is split
- THEN all chunks MUST be within Discord's 2000-char limit
- Test: `split_chunks_always_within_discord_limit` in `src/channels/discord.rs`

#### Scenario: Split message with multiple newlines

- WHEN message has multiple consecutive newlines
- THEN MUST handle correctly
- Test: `split_message_with_multiple_newlines` in `src/channels/discord.rs`

#### Scenario: Typing handles start empty

- WHEN channel is created
- THEN typing handles MUST start empty
- Test: `typing_handles_start_empty` in `src/channels/discord.rs`

#### Scenario: Start typing sets handle

- WHEN start_typing is called
- THEN MUST set the typing handle
- Test: `start_typing_sets_handle` in `src/channels/discord.rs`

#### Scenario: Stop typing clears handle

- WHEN stop_typing is called
- THEN MUST clear the typing handle
- Test: `stop_typing_clears_handle` in `src/channels/discord.rs`

#### Scenario: Stop typing is idempotent

- WHEN stop_typing is called without active typing
- THEN MUST succeed without error
- Test: `stop_typing_is_idempotent` in `src/channels/discord.rs`

#### Scenario: Concurrent typing handles are independent

- WHEN multiple recipients have typing indicators
- THEN handles MUST be independent per recipient
- Test: `concurrent_typing_handles_are_independent` in `src/channels/discord.rs`

#### Scenario: Encode emoji unicode percent encodes

- WHEN encoding a unicode emoji for Discord API
- THEN MUST percent-encode it
- Test: `encode_emoji_unicode_percent_encodes` in `src/channels/discord.rs`

#### Scenario: Encode emoji checkmark

- WHEN encoding a checkmark emoji
- THEN MUST encode correctly
- Test: `encode_emoji_checkmark` in `src/channels/discord.rs`

#### Scenario: Encode emoji custom guild emoji passthrough

- WHEN encoding a custom guild emoji
- THEN MUST pass it through unchanged
- Test: `encode_emoji_custom_guild_emoji_passthrough` in `src/channels/discord.rs`

#### Scenario: Encode emoji simple ASCII char

- WHEN encoding a simple ASCII character
- THEN MUST encode correctly
- Test: `encode_emoji_simple_ascii_char` in `src/channels/discord.rs`

#### Scenario: Random Discord ack reaction is from pool

- WHEN a random ack reaction is selected
- THEN MUST be from the configured pool
- Test: `random_discord_ack_reaction_is_from_pool` in `src/channels/discord.rs`

#### Scenario: Reaction URL encodes emoji and strips prefix

- WHEN building a reaction URL
- THEN MUST encode the emoji and strip any prefix
- Test: `discord_reaction_url_encodes_emoji_and_strips_prefix` in `src/channels/discord.rs`

#### Scenario: Discord message ID format includes prefix

- WHEN a Discord message ID is generated
- THEN MUST include the "discord_" prefix
- Test: `discord_message_id_format_includes_discord_prefix` in `src/channels/discord.rs`

#### Scenario: Discord message ID is deterministic

- WHEN the same inputs are used
- THEN MUST produce the same ID
- Test: `discord_message_id_is_deterministic` in `src/channels/discord.rs`

#### Scenario: Different message produces different Discord ID

- WHEN different message IDs are used
- THEN MUST produce different Discord IDs
- Test: `discord_message_id_different_message_different_id` in `src/channels/discord.rs`

#### Scenario: Discord message ID uses snowflake ID

- WHEN a Discord message ID is generated
- THEN MUST use the snowflake ID
- Test: `discord_message_id_uses_snowflake_id` in `src/channels/discord.rs`

#### Scenario: Discord message ID fallback to UUID on empty

- WHEN snowflake ID is empty
- THEN MUST fall back to UUID
- Test: `discord_message_id_fallback_to_uuid_on_empty` in `src/channels/discord.rs`

#### Scenario: Split code block at boundary

- WHEN splitting happens at a code block boundary
- THEN MUST handle code block continuation
- Test: `split_message_code_block_at_boundary` in `src/channels/discord.rs`

#### Scenario: Split single long word exceeds limit

- WHEN a single word exceeds the limit
- THEN MUST hard-split it
- Test: `split_message_single_long_word_exceeds_limit` in `src/channels/discord.rs`

#### Scenario: Split exactly at limit no split

- WHEN message is exactly at the limit
- THEN MUST not split
- Test: `split_message_exactly_at_limit_no_split` in `src/channels/discord.rs`

#### Scenario: Split one over limit splits

- WHEN message is one over the limit
- THEN MUST split
- Test: `split_message_one_over_limit_splits` in `src/channels/discord.rs`

#### Scenario: Split many short lines

- WHEN message has many short lines
- THEN MUST combine into chunks within limit
- Test: `split_message_many_short_lines` in `src/channels/discord.rs`

#### Scenario: Split only whitespace

- WHEN message is only whitespace
- THEN MUST handle correctly
- Test: `split_message_only_whitespace` in `src/channels/discord.rs`

#### Scenario: Split emoji at boundary

- WHEN emoji falls at split boundary
- THEN MUST not split the emoji
- Test: `split_message_emoji_at_boundary` in `src/channels/discord.rs`

#### Scenario: Split consecutive newlines at boundary

- WHEN consecutive newlines fall at boundary
- THEN MUST handle correctly
- Test: `split_message_consecutive_newlines_at_boundary` in `src/channels/discord.rs`

#### Scenario: Process attachments empty list returns empty

- WHEN attachment list is empty
- THEN MUST return empty result
- Test: `process_attachments_empty_list_returns_empty` in `src/channels/discord.rs`

#### Scenario: Process attachments skips unsupported types

- WHEN attachments have unsupported content types
- THEN MUST skip them
- Test: `process_attachments_skips_unsupported_types` in `src/channels/discord.rs`

#### Scenario: Process attachments emits image marker for image content type

- WHEN attachment has image content type
- THEN MUST emit an image marker
- Test: `process_attachments_emits_image_marker_for_image_content_type` in `src/channels/discord.rs`

#### Scenario: Process attachments emits multiple image markers

- WHEN multiple image attachments are present
- THEN MUST emit markers for each
- Test: `process_attachments_emits_multiple_image_markers` in `src/channels/discord.rs`

#### Scenario: Process attachments emits image marker from filename

- WHEN attachment has no content type but image filename
- THEN MUST infer and emit image marker
- Test: `process_attachments_emits_image_marker_from_filename_without_content_type` in `src/channels/discord.rs`

#### Scenario: Is image attachment prefers content type over extension

- WHEN content type is non-image but extension is image
- THEN MUST prefer the content type (not image)
- Test: `is_image_attachment_prefers_non_image_content_type_over_extension` in `src/channels/discord.rs`

#### Scenario: Is image attachment allows octet-stream extension fallback

- WHEN content type is octet-stream and extension is image
- THEN MUST fall back to extension
- Test: `is_image_attachment_allows_octet_stream_extension_fallback` in `src/channels/discord.rs`

#### Scenario: Parse attachment markers extracts supported markers

- WHEN message contains supported attachment markers
- THEN MUST extract them
- Test: `parse_attachment_markers_extracts_supported_markers` in `src/channels/discord.rs`

#### Scenario: Parse attachment markers keeps invalid marker text

- WHEN message contains invalid attachment markers
- THEN MUST keep them as text
- Test: `parse_attachment_markers_keeps_invalid_marker_text` in `src/channels/discord.rs`

#### Scenario: Classify outgoing attachments splits local remote and unresolved

- WHEN outgoing attachments are classified
- THEN MUST split into local, remote, and unresolved
- Test: `classify_outgoing_attachments_splits_local_remote_and_unresolved` in `src/channels/discord.rs`

#### Scenario: With inline attachment URLs appends URLs

- WHEN inline attachment URLs are added
- THEN MUST append URLs and unresolved markers
- Test: `with_inline_attachment_urls_appends_urls_and_unresolved_markers` in `src/channels/discord.rs`

#### Scenario: With workspace dir sets field

- WHEN workspace dir is configured
- THEN MUST set the field
- Test: `with_workspace_dir_sets_field` in `src/channels/discord.rs`

#### Scenario: Resolve local attachment path blocks workspace escape

- WHEN attachment path tries to escape workspace
- THEN MUST block it
- Test: `resolve_local_attachment_path_blocks_workspace_escape` in `src/channels/discord.rs`

### REQ-CHAN-008: Slack Channel

`SlackChannel` MUST support message sending, webhook listening, thread_ts threading, approval prompts, mention-only mode, display name caching, retry logic, and socket mode.

#### Scenario: Channel name is "slack"

- WHEN name() is called
- THEN MUST return "slack"
- Test: `slack_channel_name` in `src/channels/slack.rs`

#### Scenario: Channel with explicit channel_id

- WHEN channel_id is configured
- THEN MUST use the configured channel_id
- Test: `slack_channel_with_channel_id` in `src/channels/slack.rs`

#### Scenario: Group reply policy defaults to all messages

- WHEN no group reply policy override is set
- THEN MUST default to processing all messages
- Test: `slack_group_reply_policy_defaults_to_all_messages` in `src/channels/slack.rs`

#### Scenario: Group reply policy applies sender overrides

- WHEN sender-specific overrides are configured
- THEN MUST apply them
- Test: `slack_group_reply_policy_applies_sender_overrides` in `src/channels/slack.rs`

#### Scenario: Normalized channel_id respects wildcard and blank

- WHEN channel_id is wildcard or blank
- THEN MUST normalize accordingly
- Test: `normalized_channel_id_respects_wildcard_and_blank` in `src/channels/slack.rs`

#### Scenario: Configured app token ignores blank values

- WHEN app_token is blank
- THEN MUST return None
- Test: `configured_app_token_ignores_blank_values` in `src/channels/slack.rs`

#### Scenario: Configured app token trims value

- WHEN app_token has leading/trailing whitespace
- THEN MUST trim it
- Test: `configured_app_token_trims_value` in `src/channels/slack.rs`

#### Scenario: Scoped channel IDs prefers explicit list

- WHEN an explicit channel ID list is configured
- THEN MUST use the explicit list
- Test: `scoped_channel_ids_prefers_explicit_list` in `src/channels/slack.rs`

#### Scenario: Scoped channel IDs falls back to single channel_id

- WHEN no explicit list but single channel_id is set
- THEN MUST fall back to the single channel_id
- Test: `scoped_channel_ids_falls_back_to_single_channel_id` in `src/channels/slack.rs`

#### Scenario: Scoped channel IDs returns None for wildcard mode

- WHEN channel_id is wildcard
- THEN scoped_channel_ids MUST return None
- Test: `scoped_channel_ids_returns_none_for_wildcard_mode` in `src/channels/slack.rs`

#### Scenario: Is group channel_id detects channel prefixes

- WHEN channel_id starts with C or G prefix
- THEN MUST detect as group channel
- Test: `is_group_channel_id_detects_channel_prefixes` in `src/channels/slack.rs`

#### Scenario: Extract channel IDs filters archived and non-member

- WHEN channel list contains archived or non-member entries
- THEN MUST filter them out
- Test: `extract_channel_ids_filters_archived_and_non_member_entries` in `src/channels/slack.rs`

#### Scenario: Empty allowlist denies everyone

- WHEN allowlist is empty
- THEN MUST deny all users
- Test: `empty_allowlist_denies_everyone` in `src/channels/slack.rs`

#### Scenario: Wildcard allows everyone

- WHEN allowlist contains wildcard
- THEN MUST allow all users
- Test: `wildcard_allows_everyone` in `src/channels/slack.rs`

#### Scenario: Extract user display name prefers profile display name

- WHEN user profile has a display name
- THEN MUST prefer it over other fields
- Test: `extract_user_display_name_prefers_profile_display_name` in `src/channels/slack.rs`

#### Scenario: Extract user display name falls back to username

- WHEN user profile has no display name
- THEN MUST fall back to username
- Test: `extract_user_display_name_falls_back_to_username` in `src/channels/slack.rs`

#### Scenario: Cached sender display name returns None when expired

- WHEN cached display name has expired
- THEN MUST return None
- Test: `cached_sender_display_name_returns_none_when_expired` in `src/channels/slack.rs`

#### Scenario: Cached sender display name returns value when valid

- WHEN cached display name is still valid
- THEN MUST return the cached value
- Test: `cached_sender_display_name_returns_cached_value_when_valid` in `src/channels/slack.rs`

#### Scenario: Normalize incoming requires mention when enabled

- WHEN mention-only mode is enabled
- THEN MUST require bot mention
- Test: `normalize_incoming_content_requires_mention_when_enabled` in `src/channels/slack.rs`

#### Scenario: Normalize incoming without mention mode keeps message

- WHEN mention-only mode is disabled
- THEN MUST keep the message as-is
- Test: `normalize_incoming_content_without_mention_mode_keeps_message` in `src/channels/slack.rs`

#### Scenario: Specific allowlist filters

- WHEN allowlist contains specific users
- THEN MUST filter accordingly
- Test: `specific_allowlist_filters` in `src/channels/slack.rs`

#### Scenario: Allowlist exact match not substring

- WHEN user ID is a substring of an allowlisted ID
- THEN MUST not match
- Test: `allowlist_exact_match_not_substring` in `src/channels/slack.rs`

#### Scenario: Allowlist empty user ID

- WHEN user ID is empty
- THEN MUST deny
- Test: `allowlist_empty_user_id` in `src/channels/slack.rs`

#### Scenario: Allowlist case sensitive

- WHEN user ID case differs
- THEN MUST not match
- Test: `allowlist_case_sensitive` in `src/channels/slack.rs`

#### Scenario: Allowlist wildcard and specific

- WHEN allowlist has wildcard and specific entries
- THEN wildcard MUST take precedence
- Test: `allowlist_wildcard_and_specific` in `src/channels/slack.rs`

#### Scenario: Slack message ID format includes channel and ts

- WHEN a Slack message ID is generated
- THEN MUST include channel and timestamp
- Test: `slack_message_id_format_includes_channel_and_ts` in `src/channels/slack.rs`

#### Scenario: Slack message ID is deterministic

- WHEN same inputs are used
- THEN MUST produce same ID
- Test: `slack_message_id_is_deterministic` in `src/channels/slack.rs`

#### Scenario: Different ts produces different Slack ID

- WHEN different timestamps are used
- THEN MUST produce different IDs
- Test: `slack_message_id_different_ts_different_id` in `src/channels/slack.rs`

#### Scenario: Different channel produces different Slack ID

- WHEN different channels are used
- THEN MUST produce different IDs
- Test: `slack_message_id_different_channel_different_id` in `src/channels/slack.rs`

#### Scenario: Slack message ID has no UUID randomness

- WHEN a Slack message ID is generated
- THEN MUST not contain UUID randomness
- Test: `slack_message_id_no_uuid_randomness` in `src/channels/slack.rs`

#### Scenario: Inbound thread_ts prefers explicit thread_ts

- WHEN both thread_ts and ts are present
- THEN MUST prefer explicit thread_ts
- Test: `inbound_thread_ts_prefers_explicit_thread_ts` in `src/channels/slack.rs`

#### Scenario: Inbound thread_ts falls back to ts

- WHEN thread_ts is not present but ts is
- THEN MUST fall back to ts
- Test: `inbound_thread_ts_falls_back_to_ts` in `src/channels/slack.rs`

#### Scenario: Inbound thread_ts None when ts missing

- WHEN neither thread_ts nor ts is present
- THEN MUST return None
- Test: `inbound_thread_ts_none_when_ts_missing` in `src/channels/slack.rs`

#### Scenario: Ensure poll cursor bootstraps new channel

- WHEN no poll cursor exists for a channel
- THEN MUST bootstrap a new one
- Test: `ensure_poll_cursor_bootstraps_new_channel` in `src/channels/slack.rs`

#### Scenario: Ensure poll cursor keeps existing cursor

- WHEN a poll cursor already exists
- THEN MUST keep the existing one
- Test: `ensure_poll_cursor_keeps_existing_cursor` in `src/channels/slack.rs`

#### Scenario: Parse retry-after value accepts integer seconds

- WHEN retry-after header has integer seconds
- THEN MUST parse correctly
- Test: `parse_retry_after_value_accepts_integer_seconds` in `src/channels/slack.rs`

#### Scenario: Parse retry-after value accepts decimal seconds

- WHEN retry-after header has decimal seconds
- THEN MUST parse correctly
- Test: `parse_retry_after_value_accepts_decimal_seconds` in `src/channels/slack.rs`

#### Scenario: Parse retry-after value rejects non-numeric

- WHEN retry-after header has non-numeric value
- THEN MUST reject
- Test: `parse_retry_after_value_rejects_non_numeric_values` in `src/channels/slack.rs`

#### Scenario: Parse retry-after secs reads header

- WHEN retry-after header is present
- THEN MUST read the value
- Test: `parse_retry_after_secs_reads_header_value` in `src/channels/slack.rs`

#### Scenario: Compute retry delay applies backoff and jitter with cap

- WHEN computing retry delay
- THEN MUST apply exponential backoff with jitter and cap
- Test: `compute_retry_delay_applies_backoff_and_jitter_with_cap` in `src/channels/slack.rs`

### REQ-CHAN-009: WhatsApp Channels

WhatsApp channels (whatsapp.rs, whatsapp_web.rs, wati.rs) MUST support message sending with platform-specific auth, number allowlisting, and message parsing.

#### Scenario: WhatsApp channel name

- WHEN name() is called on WhatsApp channel
- THEN MUST return "whatsapp"
- Test: `whatsapp_channel_name` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp number allowed exact match

- WHEN allowlist contains an exact number
- THEN MUST allow that number
- Test: `whatsapp_number_allowed_exact` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp number allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all numbers
- Test: `whatsapp_number_allowed_wildcard` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp number denied empty allowlist

- WHEN allowlist is empty
- THEN MUST deny all numbers
- Test: `whatsapp_number_denied_empty` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp number denied mismatch

- WHEN number does not match allowlist
- THEN MUST deny
- Test: `whatsapp_number_denied_mismatch` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp exact match not substring

- WHEN number is a substring of an allowlisted number
- THEN MUST not match
- Test: `whatsapp_number_exact_match_not_substring` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse valid payload

- WHEN a valid WhatsApp webhook payload is received
- THEN MUST parse it correctly
- Test: `whatsapp_parse_valid_payload` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse skip status-only payload

- WHEN payload contains only status updates
- THEN MUST skip it
- Test: `whatsapp_parse_skip_status_only` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse skip empty changes

- WHEN payload has empty changes array
- THEN MUST skip it
- Test: `whatsapp_parse_skip_empty_changes` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse skip empty value

- WHEN payload has empty value object
- THEN MUST skip it
- Test: `whatsapp_parse_skip_empty_value` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse extracts sender phone

- WHEN a valid message is parsed
- THEN MUST extract the sender phone number
- Test: `whatsapp_parse_extracts_sender_phone` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse extracts text body

- WHEN a text message is parsed
- THEN MUST extract the text body
- Test: `whatsapp_parse_extracts_text_body` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse handles reaction message

- WHEN a reaction message is received
- THEN MUST handle it
- Test: `whatsapp_parse_handles_reaction` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse handles interactive reply

- WHEN an interactive reply is received
- THEN MUST parse the reply content
- Test: `whatsapp_parse_handles_interactive_reply` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse handles button reply

- WHEN a button reply is received
- THEN MUST parse the button payload
- Test: `whatsapp_parse_handles_button_reply` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp verify token matches

- WHEN verify token matches configuration
- THEN MUST accept the verification
- Test: `whatsapp_verify_token_matches` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp verify token rejects mismatch

- WHEN verify token does not match
- THEN MUST reject
- Test: `whatsapp_verify_token_rejects_mismatch` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse image message

- WHEN an image message is received
- THEN MUST parse image metadata
- Test: `whatsapp_parse_image_message` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse audio message

- WHEN an audio message is received
- THEN MUST parse audio metadata
- Test: `whatsapp_parse_audio_message` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse video message

- WHEN a video message is received
- THEN MUST parse video metadata
- Test: `whatsapp_parse_video_message` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse document message

- WHEN a document message is received
- THEN MUST parse document metadata
- Test: `whatsapp_parse_document_message` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse sticker message

- WHEN a sticker message is received
- THEN MUST parse sticker metadata
- Test: `whatsapp_parse_sticker_message` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse location message

- WHEN a location message is received
- THEN MUST parse location coordinates
- Test: `whatsapp_parse_location_message` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse contacts message

- WHEN a contacts message is received
- THEN MUST parse contact information
- Test: `whatsapp_parse_contacts_message` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp health check returns false when disconnected

- WHEN the channel is not connected
- THEN health_check MUST return false
- Test: `whatsapp_health_check_returns_false` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp normalize phone strips non-digits

- WHEN phone number contains non-digit characters
- THEN MUST normalize by stripping them
- Test: `whatsapp_normalize_phone` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp message ID is deterministic

- WHEN same inputs are used
- THEN MUST produce same message ID
- Test: `whatsapp_message_id_deterministic` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse skips own messages

- WHEN message is from the bot itself
- THEN MUST skip it
- Test: `whatsapp_parse_skips_own_messages` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse handles missing text field

- WHEN text field is missing
- THEN MUST handle gracefully
- Test: `whatsapp_parse_handles_missing_text` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp number with plus prefix

- WHEN number has + prefix
- THEN MUST handle it correctly
- Test: `whatsapp_number_with_plus_prefix` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse extracts wamid

- WHEN message has a wamid
- THEN MUST extract it as the message ID
- Test: `whatsapp_parse_extracts_wamid` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse extracts timestamp

- WHEN message has a timestamp
- THEN MUST extract it
- Test: `whatsapp_parse_extracts_timestamp` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse handles context with quoted message

- WHEN message has a quoted reply context
- THEN MUST parse the context
- Test: `whatsapp_parse_handles_context` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp duplicate message detection

- WHEN the same message ID is seen twice
- THEN MUST detect the duplicate
- Test: `whatsapp_duplicate_detection` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp config serde round trip

- WHEN config is serialized and deserialized
- THEN MUST preserve all fields
- Test: `whatsapp_config_serde` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp config defaults

- WHEN config uses defaults
- THEN MUST have sensible defaults
- Test: `whatsapp_config_defaults` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp Cloud API send message shape

- WHEN building a send message request
- THEN MUST produce correct Cloud API shape
- Test: `whatsapp_send_message_shape` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse handles ephemeral message

- WHEN an ephemeral message is received
- THEN MUST handle it
- Test: `whatsapp_parse_handles_ephemeral` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp parse handles forwarded message

- WHEN a forwarded message is received
- THEN MUST handle it
- Test: `whatsapp_parse_handles_forwarded` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp with transcription sets config

- WHEN transcription is enabled
- THEN MUST set transcription config
- Test: `with_transcription_sets_config_when_enabled` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp with transcription skips when disabled

- WHEN transcription is disabled
- THEN MUST skip transcription config
- Test: `with_transcription_skips_when_disabled` in `src/channels/whatsapp.rs`

#### Scenario: WhatsApp Web channel name

- WHEN name() is called on WhatsApp Web channel
- THEN MUST return "whatsapp_web"
- Test: `whatsapp_web_channel_name` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web number allowed exact

- WHEN allowlist contains exact number
- THEN MUST allow it
- Test: `whatsapp_web_number_allowed_exact` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web number allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all
- Test: `whatsapp_web_number_allowed_wildcard` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web number denied empty

- WHEN allowlist is empty
- THEN MUST deny all
- Test: `whatsapp_web_number_denied_empty` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web normalize phone adds plus

- WHEN phone number lacks + prefix
- THEN MUST add it
- Test: `whatsapp_web_normalize_phone_adds_plus` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web normalize phone preserves plus

- WHEN phone number already has + prefix
- THEN MUST preserve it
- Test: `whatsapp_web_normalize_phone_preserves_plus` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web normalize phone from JID

- WHEN phone comes from a JID format
- THEN MUST normalize correctly
- Test: `whatsapp_web_normalize_phone_from_jid` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web render pairing QR rejects empty payload

- WHEN QR payload is empty
- THEN MUST reject
- Test: `whatsapp_web_render_pairing_qr_rejects_empty_payload` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web render pairing QR outputs multiline text

- WHEN QR code is rendered
- THEN MUST output multiline text representation
- Test: `whatsapp_web_render_pairing_qr_outputs_multiline_text` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web health check disconnected

- WHEN channel is disconnected
- THEN health_check MUST return false
- Test: `whatsapp_web_health_check_disconnected` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web parse WA markers image

- WHEN message contains WA image markers
- THEN MUST parse them
- Test: `parse_wa_markers_image` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web parse WA markers multiple

- WHEN message contains multiple WA markers
- THEN MUST parse all of them
- Test: `parse_wa_markers_multiple` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web parse WA markers none

- WHEN message contains no WA markers
- THEN MUST return none
- Test: `parse_wa_markers_no_markers` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web unknown marker kind preserved

- WHEN message contains unknown marker kind
- THEN MUST preserve it in text
- Test: `parse_wa_markers_unknown_kind_preserved` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web with transcription sets config

- WHEN transcription is enabled
- THEN MUST set config
- Test: `with_transcription_sets_config_when_enabled` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web with transcription skips when disabled

- WHEN transcription is disabled
- THEN MUST skip config
- Test: `with_transcription_skips_when_disabled` in `src/channels/whatsapp_web.rs`

#### Scenario: WhatsApp Web audio MIME to filename maps voice note

- WHEN a WhatsApp voice note MIME type is received
- THEN MUST map to appropriate filename
- Test: `audio_mime_to_filename_maps_whatsapp_voice_note` in `src/channels/whatsapp_web.rs`

#### Scenario: WATI channel name

- WHEN name() is called on WATI channel
- THEN MUST return "wati"
- Test: `wati_channel_name` in `src/channels/wati.rs`

#### Scenario: WATI number allowed exact

- WHEN allowlist contains exact number
- THEN MUST allow it
- Test: `wati_number_allowed_exact` in `src/channels/wati.rs`

#### Scenario: WATI number allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all
- Test: `wati_number_allowed_wildcard` in `src/channels/wati.rs`

#### Scenario: WATI number allowed empty

- WHEN allowlist is empty
- THEN MUST deny all
- Test: `wati_number_allowed_empty` in `src/channels/wati.rs`

#### Scenario: WATI build target with tenant

- WHEN tenant is configured
- THEN MUST build target with tenant prefix
- Test: `wati_build_target_with_tenant` in `src/channels/wati.rs`

#### Scenario: WATI build target without tenant

- WHEN no tenant is configured
- THEN MUST build target without prefix
- Test: `wati_build_target_without_tenant` in `src/channels/wati.rs`

#### Scenario: WATI build target already prefixed

- WHEN target already has tenant prefix
- THEN MUST not double-prefix
- Test: `wati_build_target_already_prefixed` in `src/channels/wati.rs`

#### Scenario: WATI parse valid message

- WHEN a valid WATI webhook message is received
- THEN MUST parse it correctly
- Test: `wati_parse_valid_message` in `src/channels/wati.rs`

#### Scenario: WATI parse skip from_me messages

- WHEN message is from the bot
- THEN MUST skip it
- Test: `wati_parse_skip_from_me` in `src/channels/wati.rs`

#### Scenario: WATI parse skip no text

- WHEN message has no text
- THEN MUST skip it
- Test: `wati_parse_skip_no_text` in `src/channels/wati.rs`

#### Scenario: WATI parse alternative field names

- WHEN payload uses alternative field names
- THEN MUST parse correctly
- Test: `wati_parse_alternative_field_names` in `src/channels/wati.rs`

#### Scenario: WATI parse timestamp seconds

- WHEN timestamp is in seconds
- THEN MUST parse correctly
- Test: `wati_parse_timestamp_seconds` in `src/channels/wati.rs`

#### Scenario: WATI parse timestamp milliseconds

- WHEN timestamp is in milliseconds
- THEN MUST convert to seconds
- Test: `wati_parse_timestamp_milliseconds` in `src/channels/wati.rs`

#### Scenario: WATI parse timestamp ISO

- WHEN timestamp is in ISO format
- THEN MUST parse correctly
- Test: `wati_parse_timestamp_iso` in `src/channels/wati.rs`

#### Scenario: WATI parse normalizes phone

- WHEN phone number needs normalization
- THEN MUST normalize it
- Test: `wati_parse_normalizes_phone` in `src/channels/wati.rs`

#### Scenario: WATI parse empty payload

- WHEN payload is empty
- THEN MUST handle gracefully
- Test: `wati_parse_empty_payload` in `src/channels/wati.rs`

#### Scenario: WATI parse from field fallback

- WHEN primary sender field is missing
- THEN MUST fall back to alternative field
- Test: `wati_parse_from_field_fallback` in `src/channels/wati.rs`

#### Scenario: WATI parse message text fallback

- WHEN primary text field is missing
- THEN MUST fall back to alternative field
- Test: `wati_parse_message_text_fallback` in `src/channels/wati.rs`

#### Scenario: WATI parse owner field as from_me

- WHEN owner field indicates self-sent
- THEN MUST treat as from_me
- Test: `wati_parse_owner_field_as_from_me` in `src/channels/wati.rs`

### REQ-CHAN-010: WhatsApp Storage

`RusqliteStore` MUST implement Signal protocol storage traits for WhatsApp Web encryption.

#### Scenario: Store creates database with schema

- WHEN store is created
- THEN MUST initialize SQLite schema for all protocol stores
- Test: `rusqlite_store_creates_database` in `src/channels/whatsapp_storage.rs`

#### Scenario: LID mapping round trip preserves data

- WHEN a LID mapping is stored and retrieved
- THEN MUST preserve learning_source and updated_at
- Test: `lid_mapping_round_trip_preserves_learning_source_and_updated_at` in `src/channels/whatsapp_storage.rs`

#### Scenario: Delete expired TC tokens returns deleted count

- WHEN expired TC tokens are deleted
- THEN MUST return the count of deleted rows
- Test: `delete_expired_tc_tokens_returns_deleted_row_count` in `src/channels/whatsapp_storage.rs`

### REQ-CHAN-011: Matrix Channel

`MatrixChannel` MUST support message sending, room syncing, end-to-end encryption support, mention-only mode, and event caching.

#### Scenario: Matrix channel name

- WHEN name() is called
- THEN MUST return "matrix"
- Test: `matrix_channel_name` in `src/channels/matrix.rs`

#### Scenario: Matrix user allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all users
- Test: `matrix_user_allowed_wildcard` in `src/channels/matrix.rs`

#### Scenario: Matrix user allowed specific

- WHEN allowlist contains specific user
- THEN MUST allow that user
- Test: `matrix_user_allowed_specific` in `src/channels/matrix.rs`

#### Scenario: Matrix user denied empty

- WHEN allowlist is empty
- THEN MUST deny all users
- Test: `matrix_user_denied_empty` in `src/channels/matrix.rs`

#### Scenario: Matrix user exact match not substring

- WHEN user is a substring of allowlisted user
- THEN MUST not match
- Test: `matrix_user_exact_match_not_substring` in `src/channels/matrix.rs`

#### Scenario: Matrix allowlist case sensitive

- WHEN user case differs from allowlist
- THEN MUST not match
- Test: `matrix_allowlist_case_sensitive` in `src/channels/matrix.rs`

#### Scenario: Matrix wildcard with specific users

- WHEN allowlist has wildcard and specific entries
- THEN wildcard MUST take precedence
- Test: `matrix_wildcard_with_specific` in `src/channels/matrix.rs`

#### Scenario: Matrix room ID from config

- WHEN room_id is configured
- THEN MUST use configured room_id
- Test: `matrix_room_id_from_config` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync response

- WHEN a Matrix sync response is received
- THEN MUST parse room events
- Test: `matrix_parse_sync_response` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync skips own messages

- WHEN sync response contains own messages
- THEN MUST skip them
- Test: `matrix_parse_sync_skips_own_messages` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync skips denied users

- WHEN sync response contains messages from denied users
- THEN MUST skip them
- Test: `matrix_parse_sync_skips_denied` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync extracts sender

- WHEN parsing a sync event
- THEN MUST extract the sender
- Test: `matrix_parse_sync_extracts_sender` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync extracts body

- WHEN parsing a sync event
- THEN MUST extract the message body
- Test: `matrix_parse_sync_extracts_body` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync extracts event ID

- WHEN parsing a sync event
- THEN MUST extract the event ID
- Test: `matrix_parse_sync_extracts_event_id` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync extracts timestamp

- WHEN parsing a sync event
- THEN MUST extract the timestamp
- Test: `matrix_parse_sync_extracts_timestamp` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync handles empty rooms

- WHEN sync response has empty rooms
- THEN MUST handle gracefully
- Test: `matrix_parse_sync_handles_empty_rooms` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync handles missing timeline

- WHEN sync response has missing timeline
- THEN MUST handle gracefully
- Test: `matrix_parse_sync_handles_missing_timeline` in `src/channels/matrix.rs`

#### Scenario: Matrix event cache tracks seen events

- WHEN events are processed
- THEN MUST track seen event IDs
- Test: `matrix_event_cache_tracks_seen` in `src/channels/matrix.rs`

#### Scenario: Matrix event cache rejects duplicates

- WHEN a duplicate event is received
- THEN MUST reject it
- Test: `matrix_event_cache_rejects_duplicates` in `src/channels/matrix.rs`

#### Scenario: Matrix event cache evicts old entries

- WHEN cache exceeds capacity
- THEN MUST evict old entries
- Test: `matrix_event_cache_evicts_old` in `src/channels/matrix.rs`

#### Scenario: Matrix mention-only mode requires mention

- WHEN mention-only mode is enabled
- THEN MUST require bot mention
- Test: `matrix_mention_only_requires_mention` in `src/channels/matrix.rs`

#### Scenario: Matrix mention-only strips mention

- WHEN mention is detected
- THEN MUST strip it from content
- Test: `matrix_mention_only_strips_mention` in `src/channels/matrix.rs`

#### Scenario: Matrix mention-only allows configured sender

- WHEN configured sender sends without mention
- THEN MUST allow
- Test: `matrix_mention_only_allows_configured_sender` in `src/channels/matrix.rs`

#### Scenario: Matrix health check disconnected

- WHEN channel is not connected
- THEN health_check MUST return false
- Test: `matrix_health_check_disconnected` in `src/channels/matrix.rs`

#### Scenario: Matrix config serde

- WHEN config is serialized/deserialized
- THEN MUST round-trip correctly
- Test: `matrix_config_serde` in `src/channels/matrix.rs`

#### Scenario: Matrix config defaults

- WHEN config uses defaults
- THEN MUST have sensible defaults
- Test: `matrix_config_defaults` in `src/channels/matrix.rs`

#### Scenario: Matrix message ID format

- WHEN a Matrix message ID is generated
- THEN MUST use the event ID
- Test: `matrix_message_id_format` in `src/channels/matrix.rs`

#### Scenario: Matrix message ID is deterministic

- WHEN same event ID is used
- THEN MUST produce same message ID
- Test: `matrix_message_id_deterministic` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync multiple rooms

- WHEN sync response has events from multiple rooms
- THEN MUST parse events from all rooms
- Test: `matrix_parse_sync_multiple_rooms` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync handles redacted events

- WHEN sync response contains redacted events
- THEN MUST handle gracefully
- Test: `matrix_parse_sync_handles_redacted` in `src/channels/matrix.rs`

#### Scenario: Matrix parse formatted body

- WHEN event has formatted body
- THEN MUST parse it
- Test: `matrix_parse_formatted_body` in `src/channels/matrix.rs`

#### Scenario: Matrix scoped room IDs from config

- WHEN multiple room IDs are configured
- THEN MUST scope to those rooms
- Test: `matrix_scoped_room_ids` in `src/channels/matrix.rs`

#### Scenario: Matrix parse sync skips non-text events

- WHEN sync contains non-text events
- THEN MUST skip them
- Test: `matrix_parse_sync_skips_non_text` in `src/channels/matrix.rs`

#### Scenario: Matrix normalize incoming strips mention

- WHEN incoming content has mention
- THEN MUST strip it
- Test: `matrix_normalize_incoming_strips_mention` in `src/channels/matrix.rs`

#### Scenario: Matrix normalize incoming keeps without mention mode

- WHEN mention mode is disabled
- THEN MUST keep content as-is
- Test: `matrix_normalize_incoming_keeps_without_mention_mode` in `src/channels/matrix.rs`

### REQ-CHAN-012: IRC Channel

`IrcChannel` MUST support message sending, channel joining, IRC protocol handling, SASL authentication, message splitting, and allowlist filtering.

#### Scenario: IRC channel name

- WHEN name() is called
- THEN MUST return "irc"
- Test: `irc_channel_name` in `src/channels/irc.rs`

#### Scenario: IRC user allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all users
- Test: `irc_user_allowed_wildcard` in `src/channels/irc.rs`

#### Scenario: IRC user allowed specific

- WHEN allowlist contains specific user
- THEN MUST allow that user
- Test: `irc_user_allowed_specific` in `src/channels/irc.rs`

#### Scenario: IRC user denied empty

- WHEN allowlist is empty
- THEN MUST deny all users
- Test: `irc_user_denied_empty` in `src/channels/irc.rs`

#### Scenario: IRC user exact match not substring

- WHEN user is a substring of allowlisted user
- THEN MUST not match
- Test: `irc_user_exact_match_not_substring` in `src/channels/irc.rs`

#### Scenario: IRC parse PRIVMSG

- WHEN a PRIVMSG is received
- THEN MUST parse sender, target, and content
- Test: `irc_parse_privmsg` in `src/channels/irc.rs`

#### Scenario: IRC parse PRIVMSG with host

- WHEN PRIVMSG has a full hostmask
- THEN MUST extract the nick
- Test: `irc_parse_privmsg_with_host` in `src/channels/irc.rs`

#### Scenario: IRC parse ignores non-PRIVMSG

- WHEN a non-PRIVMSG line is received
- THEN MUST ignore it
- Test: `irc_parse_ignores_non_privmsg` in `src/channels/irc.rs`

#### Scenario: IRC parse handles malformed lines

- WHEN a malformed IRC line is received
- THEN MUST handle gracefully
- Test: `irc_parse_handles_malformed` in `src/channels/irc.rs`

#### Scenario: IRC split short message

- WHEN message is under the IRC line limit
- THEN MUST not split
- Test: `irc_split_short_message` in `src/channels/irc.rs`

#### Scenario: IRC split long message

- WHEN message exceeds IRC line limit
- THEN MUST split into multiple lines
- Test: `irc_split_long_message` in `src/channels/irc.rs`

#### Scenario: IRC split at word boundary

- WHEN splitting is needed
- THEN MUST prefer word boundaries
- Test: `irc_split_at_word_boundary` in `src/channels/irc.rs`

#### Scenario: IRC split preserves content

- WHEN message is split
- THEN reassembled MUST equal original
- Test: `irc_split_preserves_content` in `src/channels/irc.rs`

#### Scenario: IRC SASL plain encoding

- WHEN SASL PLAIN credentials are encoded
- THEN MUST produce correct base64 encoding
- Test: `irc_sasl_plain_encoding` in `src/channels/irc.rs`

#### Scenario: IRC SASL plain empty password

- WHEN SASL PLAIN password is empty
- THEN MUST handle gracefully
- Test: `irc_sasl_plain_empty_password` in `src/channels/irc.rs`

#### Scenario: IRC config serde

- WHEN config is serialized/deserialized
- THEN MUST round-trip correctly
- Test: `irc_config_serde` in `src/channels/irc.rs`

#### Scenario: IRC config defaults

- WHEN config uses defaults
- THEN MUST have sensible defaults
- Test: `irc_config_defaults` in `src/channels/irc.rs`

#### Scenario: IRC channel join command

- WHEN joining a channel
- THEN MUST produce correct JOIN command
- Test: `irc_channel_join_command` in `src/channels/irc.rs`

#### Scenario: IRC health check disconnected

- WHEN channel is not connected
- THEN health_check MUST return false
- Test: `irc_health_check_disconnected` in `src/channels/irc.rs`

#### Scenario: IRC message ID format

- WHEN an IRC message ID is generated
- THEN MUST have correct format
- Test: `irc_message_id_format` in `src/channels/irc.rs`

#### Scenario: IRC parse CTCP ACTION

- WHEN a CTCP ACTION is received
- THEN MUST parse it as an action
- Test: `irc_parse_ctcp_action` in `src/channels/irc.rs`

#### Scenario: IRC parse NOTICE

- WHEN a NOTICE is received
- THEN MUST parse it
- Test: `irc_parse_notice` in `src/channels/irc.rs`

#### Scenario: IRC allowlist case sensitive

- WHEN username case differs from allowlist
- THEN MUST not match
- Test: `irc_allowlist_case_sensitive` in `src/channels/irc.rs`

#### Scenario: IRC wildcard with specific users

- WHEN allowlist has wildcard and specific entries
- THEN wildcard MUST take precedence
- Test: `irc_wildcard_with_specific` in `src/channels/irc.rs`

#### Scenario: IRC normalize nick from hostmask

- WHEN a hostmask is received
- THEN MUST normalize to just the nick
- Test: `irc_normalize_nick_from_hostmask` in `src/channels/irc.rs`

#### Scenario: IRC split empty message

- WHEN message is empty
- THEN MUST handle correctly
- Test: `irc_split_empty_message` in `src/channels/irc.rs`

#### Scenario: IRC split multibyte safe

- WHEN message contains multibyte characters at boundary
- THEN MUST split at valid UTF-8 boundary
- Test: `irc_split_multibyte_safe` in `src/channels/irc.rs`

#### Scenario: IRC parse strips formatting codes

- WHEN message contains IRC formatting codes
- THEN MUST strip them
- Test: `irc_parse_strips_formatting` in `src/channels/irc.rs`

#### Scenario: IRC config TLS defaults

- WHEN TLS config uses defaults
- THEN MUST have sensible TLS defaults
- Test: `irc_config_tls_defaults` in `src/channels/irc.rs`

#### Scenario: IRC mention-only mode

- WHEN mention-only mode is enabled
- THEN MUST require bot nick mention
- Test: `irc_mention_only_mode` in `src/channels/irc.rs`

#### Scenario: IRC mention-only strips mention

- WHEN mention is present in mention-only mode
- THEN MUST strip the mention
- Test: `irc_mention_only_strips_mention` in `src/channels/irc.rs`

#### Scenario: IRC split very long word

- WHEN a single word exceeds the line limit
- THEN MUST hard-split it
- Test: `irc_split_very_long_word` in `src/channels/irc.rs`

#### Scenario: IRC parse multiple channels

- WHEN messages arrive from multiple channels
- THEN MUST route correctly
- Test: `irc_parse_multiple_channels` in `src/channels/irc.rs`

#### Scenario: IRC message ID deterministic

- WHEN same inputs are used
- THEN MUST produce same ID
- Test: `irc_message_id_deterministic` in `src/channels/irc.rs`

#### Scenario: IRC config channel list

- WHEN multiple channels are configured
- THEN MUST join all of them
- Test: `irc_config_channel_list` in `src/channels/irc.rs`

### REQ-CHAN-013: Email Channel

`EmailChannel` MUST support message sending via SMTP, receiving via IMAP, subject lines, sender allowlisting, HTML stripping, and configuration serialization.

#### Scenario: Default SMTP port uses TLS port

- WHEN no explicit SMTP port is set
- THEN MUST default to TLS port (465)
- Test: `default_smtp_port_uses_tls_port` in `src/channels/email_channel.rs`

#### Scenario: Email config default uses TLS SMTP defaults

- WHEN config uses defaults
- THEN MUST use TLS SMTP defaults
- Test: `email_config_default_uses_tls_smtp_defaults` in `src/channels/email_channel.rs`

#### Scenario: Default idle timeout is 29 minutes

- WHEN no explicit idle timeout is set
- THEN MUST default to 29 minutes
- Test: `default_idle_timeout_is_29_minutes` in `src/channels/email_channel.rs`

#### Scenario: Seen messages starts empty

- WHEN channel is created
- THEN seen messages set MUST start empty
- Test: `seen_messages_starts_empty` in `src/channels/email_channel.rs`

#### Scenario: Seen messages tracks unique IDs

- WHEN messages are processed
- THEN MUST track unique message IDs
- Test: `seen_messages_tracks_unique_ids` in `src/channels/email_channel.rs`

#### Scenario: Email config default

- WHEN config is created with defaults
- THEN MUST have sensible defaults
- Test: `email_config_default` in `src/channels/email_channel.rs`

#### Scenario: Email config custom

- WHEN config is created with custom values
- THEN MUST use the custom values
- Test: `email_config_custom` in `src/channels/email_channel.rs`

#### Scenario: Email config clone

- WHEN config is cloned
- THEN MUST preserve all fields
- Test: `email_config_clone` in `src/channels/email_channel.rs`

#### Scenario: Email channel new

- WHEN email channel is created
- THEN MUST initialize correctly
- Test: `email_channel_new` in `src/channels/email_channel.rs`

#### Scenario: Email channel name

- WHEN name() is called
- THEN MUST return "email"
- Test: `email_channel_name` in `src/channels/email_channel.rs`

#### Scenario: Sender allowed empty list denies all

- WHEN sender allowlist is empty
- THEN MUST deny all senders
- Test: `is_sender_allowed_empty_list_denies_all` in `src/channels/email_channel.rs`

#### Scenario: Sender allowed wildcard allows all

- WHEN sender allowlist contains wildcard
- THEN MUST allow all senders
- Test: `is_sender_allowed_wildcard_allows_all` in `src/channels/email_channel.rs`

#### Scenario: Sender allowed specific email

- WHEN allowlist contains a specific email
- THEN MUST allow that sender
- Test: `is_sender_allowed_specific_email` in `src/channels/email_channel.rs`

#### Scenario: Sender allowed domain with @ prefix

- WHEN allowlist contains a domain with @ prefix
- THEN MUST allow all senders from that domain
- Test: `is_sender_allowed_domain_with_at_prefix` in `src/channels/email_channel.rs`

#### Scenario: Sender allowed domain without @ prefix

- WHEN allowlist contains a domain without @ prefix
- THEN MUST allow all senders from that domain
- Test: `is_sender_allowed_domain_without_at_prefix` in `src/channels/email_channel.rs`

#### Scenario: Sender allowed case insensitive

- WHEN sender case differs from allowlist
- THEN MUST still match (case insensitive)
- Test: `is_sender_allowed_case_insensitive` in `src/channels/email_channel.rs`

#### Scenario: Sender allowed multiple senders

- WHEN allowlist has multiple entries
- THEN MUST allow all listed senders
- Test: `is_sender_allowed_multiple_senders` in `src/channels/email_channel.rs`

#### Scenario: Sender allowed wildcard with specific

- WHEN allowlist has wildcard and specific entries
- THEN wildcard MUST take precedence
- Test: `is_sender_allowed_wildcard_with_specific` in `src/channels/email_channel.rs`

#### Scenario: Sender allowed empty sender string

- WHEN sender is empty string
- THEN MUST deny
- Test: `is_sender_allowed_empty_sender` in `src/channels/email_channel.rs`

#### Scenario: Strip HTML basic

- WHEN HTML content has basic tags
- THEN MUST strip them
- Test: `strip_html_basic` in `src/channels/email_channel.rs`

#### Scenario: Strip HTML nested tags

- WHEN HTML has nested tags
- THEN MUST strip all levels
- Test: `strip_html_nested_tags` in `src/channels/email_channel.rs`

#### Scenario: Strip HTML multiple lines

- WHEN HTML spans multiple lines
- THEN MUST strip across lines
- Test: `strip_html_multiple_lines` in `src/channels/email_channel.rs`

#### Scenario: Strip HTML preserves text

- WHEN stripping HTML
- THEN MUST preserve the text content
- Test: `strip_html_preserves_text` in `src/channels/email_channel.rs`

#### Scenario: Strip HTML handles malformed

- WHEN HTML is malformed
- THEN MUST handle gracefully
- Test: `strip_html_handles_malformed` in `src/channels/email_channel.rs`

#### Scenario: Strip HTML self-closing tags

- WHEN HTML has self-closing tags
- THEN MUST strip them
- Test: `strip_html_self_closing_tags` in `src/channels/email_channel.rs`

#### Scenario: Strip HTML attributes preserved

- WHEN HTML has attributes
- THEN MUST strip tags including attributes
- Test: `strip_html_attributes_preserved` in `src/channels/email_channel.rs`

#### Scenario: Strip HTML multiple spaces collapsed

- WHEN stripped result has multiple spaces
- THEN MUST collapse them
- Test: `strip_html_multiple_spaces_collapsed` in `src/channels/email_channel.rs`

#### Scenario: Strip HTML special characters

- WHEN HTML has special/entity characters
- THEN MUST handle them
- Test: `strip_html_special_characters` in `src/channels/email_channel.rs`

#### Scenario: Default IMAP port returns 993

- WHEN no explicit IMAP port is set
- THEN MUST default to 993
- Test: `default_imap_port_returns_993` in `src/channels/email_channel.rs`

#### Scenario: Default SMTP port returns 465

- WHEN no explicit SMTP port is set
- THEN MUST default to 465
- Test: `default_smtp_port_returns_465` in `src/channels/email_channel.rs`

#### Scenario: Default IMAP folder returns INBOX

- WHEN no explicit IMAP folder is set
- THEN MUST default to "INBOX"
- Test: `default_imap_folder_returns_inbox` in `src/channels/email_channel.rs`

#### Scenario: Default true helper returns true

- WHEN default_true() is called
- THEN MUST return true
- Test: `default_true_returns_true` in `src/channels/email_channel.rs`

#### Scenario: Email config serialize/deserialize

- WHEN config is serialized then deserialized
- THEN MUST round-trip correctly
- Test: `email_config_serialize_deserialize` in `src/channels/email_channel.rs`

#### Scenario: Email config deserialize with defaults

- WHEN config is deserialized with missing fields
- THEN MUST use defaults for missing fields
- Test: `email_config_deserialize_with_defaults` in `src/channels/email_channel.rs`

#### Scenario: Idle timeout deserializes explicit value

- WHEN idle_timeout is explicitly set
- THEN MUST use the explicit value
- Test: `idle_timeout_deserializes_explicit_value` in `src/channels/email_channel.rs`

#### Scenario: Idle timeout deserializes legacy poll_interval alias

- WHEN legacy poll_interval key is used
- THEN MUST deserialize as idle_timeout
- Test: `idle_timeout_deserializes_legacy_poll_interval_alias` in `src/channels/email_channel.rs`

#### Scenario: Idle timeout propagates to channel

- WHEN idle_timeout is configured
- THEN MUST propagate to the channel instance
- Test: `idle_timeout_propagates_to_channel` in `src/channels/email_channel.rs`

#### Scenario: IMAP ID defaults deserialize when omitted

- WHEN IMAP ID fields are omitted from config
- THEN MUST use defaults
- Test: `imap_id_defaults_deserialize_when_omitted` in `src/channels/email_channel.rs`

#### Scenario: IMAP ID custom values deserialize

- WHEN IMAP ID fields have custom values
- THEN MUST deserialize them
- Test: `imap_id_custom_values_deserialize` in `src/channels/email_channel.rs`

#### Scenario: Email config debug output

- WHEN debug formatting email config
- THEN MUST produce valid debug output
- Test: `email_config_debug_output` in `src/channels/email_channel.rs`

### REQ-CHAN-014: Signal Channel

`SignalChannel` MUST support message sending and receiving via Signal protocol, envelope processing, recipient target parsing, and SSE listener support.

#### Scenario: Signal channel name

- WHEN name() is called
- THEN MUST return "signal"
- Test: `signal_channel_name` in `src/channels/signal.rs`

#### Scenario: Signal user allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all users
- Test: `signal_user_allowed_wildcard` in `src/channels/signal.rs`

#### Scenario: Signal user allowed specific

- WHEN allowlist contains specific user
- THEN MUST allow that user
- Test: `signal_user_allowed_specific` in `src/channels/signal.rs`

#### Scenario: Signal user denied empty

- WHEN allowlist is empty
- THEN MUST deny all users
- Test: `signal_user_denied_empty` in `src/channels/signal.rs`

#### Scenario: Signal user exact match

- WHEN user is a substring of allowlisted user
- THEN MUST not match
- Test: `signal_user_exact_match` in `src/channels/signal.rs`

#### Scenario: Signal process envelope text message

- WHEN a text envelope is received
- THEN MUST process it into a ChannelMessage
- Test: `signal_process_envelope_text` in `src/channels/signal.rs`

#### Scenario: Signal process envelope skips own messages

- WHEN envelope is from the bot itself
- THEN MUST skip it
- Test: `signal_process_envelope_skips_own` in `src/channels/signal.rs`

#### Scenario: Signal process envelope skips empty

- WHEN envelope has no text content
- THEN MUST skip it
- Test: `signal_process_envelope_skips_empty` in `src/channels/signal.rs`

#### Scenario: Signal process envelope skips denied

- WHEN envelope is from a denied sender
- THEN MUST skip it
- Test: `signal_process_envelope_skips_denied` in `src/channels/signal.rs`

#### Scenario: Signal process envelope group message

- WHEN a group message envelope is received
- THEN MUST extract group ID as reply target
- Test: `signal_process_envelope_group` in `src/channels/signal.rs`

#### Scenario: Signal parse recipient target phone

- WHEN recipient is a phone number
- THEN MUST parse as phone target
- Test: `signal_parse_recipient_target_phone` in `src/channels/signal.rs`

#### Scenario: Signal parse recipient target UUID

- WHEN recipient is a UUID
- THEN MUST parse as UUID target
- Test: `signal_parse_recipient_target_uuid` in `src/channels/signal.rs`

#### Scenario: Signal parse recipient target group

- WHEN recipient is a group ID
- THEN MUST parse as group target
- Test: `signal_parse_recipient_target_group` in `src/channels/signal.rs`

#### Scenario: Signal config serde

- WHEN config is serialized/deserialized
- THEN MUST round-trip correctly
- Test: `signal_config_serde` in `src/channels/signal.rs`

#### Scenario: Signal config defaults

- WHEN config uses defaults
- THEN MUST have sensible defaults
- Test: `signal_config_defaults` in `src/channels/signal.rs`

#### Scenario: Signal health check disconnected

- WHEN channel is not connected
- THEN health_check MUST return false
- Test: `signal_health_check_disconnected` in `src/channels/signal.rs`

#### Scenario: Signal message ID format

- WHEN a Signal message ID is generated
- THEN MUST have correct format
- Test: `signal_message_id_format` in `src/channels/signal.rs`

#### Scenario: Signal message ID deterministic

- WHEN same inputs are used
- THEN MUST produce same ID
- Test: `signal_message_id_deterministic` in `src/channels/signal.rs`

#### Scenario: Signal process envelope extracts timestamp

- WHEN processing an envelope
- THEN MUST extract the timestamp
- Test: `signal_process_envelope_extracts_timestamp` in `src/channels/signal.rs`

#### Scenario: Signal SSE listener parses event

- WHEN an SSE event is received
- THEN MUST parse it correctly
- Test: `signal_sse_parse_event` in `src/channels/signal.rs`

#### Scenario: Signal SSE listener skips non-message events

- WHEN SSE event is not a message
- THEN MUST skip it
- Test: `signal_sse_skips_non_message` in `src/channels/signal.rs`

#### Scenario: Signal SSE handles malformed data

- WHEN SSE data is malformed
- THEN MUST handle gracefully
- Test: `signal_sse_handles_malformed` in `src/channels/signal.rs`

#### Scenario: Signal normalize phone

- WHEN phone number needs normalization
- THEN MUST normalize it
- Test: `signal_normalize_phone` in `src/channels/signal.rs`

#### Scenario: Signal process envelope reaction

- WHEN a reaction envelope is received
- THEN MUST handle it
- Test: `signal_process_envelope_reaction` in `src/channels/signal.rs`

#### Scenario: Signal process envelope typing

- WHEN a typing indicator envelope is received
- THEN MUST handle it
- Test: `signal_process_envelope_typing` in `src/channels/signal.rs`

#### Scenario: Signal process envelope receipt

- WHEN a receipt envelope is received
- THEN MUST handle it
- Test: `signal_process_envelope_receipt` in `src/channels/signal.rs`

#### Scenario: Signal allowlist case sensitive

- WHEN user case differs from allowlist
- THEN MUST not match
- Test: `signal_allowlist_case_sensitive` in `src/channels/signal.rs`

#### Scenario: Signal wildcard with specific

- WHEN allowlist has wildcard and specific entries
- THEN wildcard MUST take precedence
- Test: `signal_wildcard_with_specific` in `src/channels/signal.rs`

#### Scenario: Signal duplicate message detection

- WHEN same message is received twice
- THEN MUST detect duplicate
- Test: `signal_duplicate_detection` in `src/channels/signal.rs`

#### Scenario: Signal process envelope with attachment

- WHEN envelope contains an attachment
- THEN MUST handle the attachment
- Test: `signal_process_envelope_attachment` in `src/channels/signal.rs`

#### Scenario: Signal process envelope mention only

- WHEN mention-only mode is enabled
- THEN MUST require mention
- Test: `signal_mention_only` in `src/channels/signal.rs`

### REQ-CHAN-015: Nostr Channel

`NostrChannel` MUST support NIP-04 and NIP-17 encrypted messaging protocols, key validation, and relay health checks.

#### Scenario: Allow list empty denies all

- WHEN pubkey allowlist is empty
- THEN MUST deny all pubkeys
- Test: `allow_list_empty_denies_all` in `src/channels/nostr.rs`

#### Scenario: Allow list wildcard allows all

- WHEN pubkey allowlist contains wildcard
- THEN MUST allow all pubkeys
- Test: `allow_list_wildcard_allows_all` in `src/channels/nostr.rs`

#### Scenario: Allow list specific pubkeys

- WHEN allowlist contains specific pubkeys
- THEN MUST allow only those pubkeys
- Test: `allow_list_specific_pubkeys` in `src/channels/nostr.rs`

#### Scenario: Allow list rejects invalid key

- WHEN an invalid pubkey is checked
- THEN MUST reject it
- Test: `allow_list_rejects_invalid_key` in `src/channels/nostr.rs`

#### Scenario: Nostr channel name is "nostr"

- WHEN name() is called
- THEN MUST return "nostr"
- Test: `nostr_channel_name_is_nostr` in `src/channels/nostr.rs`

#### Scenario: Nostr channel stores parsed keys

- WHEN channel is created with keys
- THEN MUST store the parsed keys
- Test: `nostr_channel_stores_parsed_keys` in `src/channels/nostr.rs`

#### Scenario: New rejects invalid key

- WHEN channel is created with invalid key
- THEN MUST reject
- Test: `new_rejects_invalid_key` in `src/channels/nostr.rs`

#### Scenario: New rejects invalid allowed pubkey

- WHEN channel is created with invalid allowed pubkey
- THEN MUST reject
- Test: `new_rejects_invalid_allowed_pubkey` in `src/channels/nostr.rs`

#### Scenario: Health check false with no relays

- WHEN no relays are configured
- THEN health_check MUST return false
- Test: `health_check_false_with_no_relays` in `src/channels/nostr.rs`

#### Scenario: Default protocol is NIP-17

- WHEN no protocol is explicitly set
- THEN MUST default to NIP-17
- Test: `default_protocol_is_nip17` in `src/channels/nostr.rs`

#### Scenario: Sender protocol tracks updates

- WHEN sender protocol preference is updated
- THEN MUST track the update
- Test: `sender_protocol_tracks_updates` in `src/channels/nostr.rs`

### REQ-CHAN-016: Lark Channel

`LarkChannel` MUST support Feishu/Lark messaging with bot API, challenge verification, mention-only group mode, token refresh, locale detection, WebSocket activity tracking, and reaction support.

#### Scenario: Lark channel name

- WHEN name() is called
- THEN MUST return "lark"
- Test: `lark_channel_name` in `src/channels/lark.rs`

#### Scenario: WS activity refreshes heartbeat watchdog

- WHEN a WebSocket activity frame is received
- THEN MUST refresh the heartbeat watchdog
- Test: `lark_ws_activity_refreshes_heartbeat_watchdog` in `src/channels/lark.rs`

#### Scenario: WS non-activity frames do not refresh heartbeat

- WHEN a non-activity WebSocket frame is received
- THEN MUST not refresh the heartbeat watchdog
- Test: `lark_ws_non_activity_frames_do_not_refresh_heartbeat_watchdog` in `src/channels/lark.rs`

#### Scenario: Group response requires matching bot mention

- WHEN mention-only mode is enabled with bot IDs
- THEN MUST require matching bot mention in group messages
- Test: `lark_group_response_requires_matching_bot_mention_when_ids_available` in `src/channels/lark.rs`

#### Scenario: Group response requires resolved open_id

- WHEN mention-only is enabled
- THEN MUST require resolved open_id for matching
- Test: `lark_group_response_requires_resolved_open_id_when_mention_only_enabled` in `src/channels/lark.rs`

#### Scenario: Group response allows post mentions for bot open_id

- WHEN message has post-style mention of bot open_id
- THEN MUST allow it
- Test: `lark_group_response_allows_post_mentions_for_bot_open_id` in `src/channels/lark.rs`

#### Scenario: Group response allows sender override without mention

- WHEN sender has a trigger override
- THEN MUST allow without mention
- Test: `lark_group_response_allows_sender_override_without_mention` in `src/channels/lark.rs`

#### Scenario: Should refresh token on HTTP 401

- WHEN API returns HTTP 401
- THEN MUST trigger token refresh
- Test: `lark_should_refresh_token_on_http_401` in `src/channels/lark.rs`

#### Scenario: Should refresh token on body code 99991663

- WHEN API returns error code 99991663
- THEN MUST trigger token refresh
- Test: `lark_should_refresh_token_on_body_code_99991663` in `src/channels/lark.rs`

#### Scenario: Should not refresh token on success body

- WHEN API returns success
- THEN MUST not trigger token refresh
- Test: `lark_should_not_refresh_token_on_success_body` in `src/channels/lark.rs`

#### Scenario: Extract token TTL supports expire and expires_in

- WHEN token response has expire or expires_in fields
- THEN MUST extract TTL from either
- Test: `lark_extract_token_ttl_seconds_supports_expire_and_expires_in` in `src/channels/lark.rs`

#### Scenario: Next token refresh deadline reserves refresh skew

- WHEN computing next token refresh deadline
- THEN MUST reserve skew buffer
- Test: `lark_next_token_refresh_deadline_reserves_refresh_skew` in `src/channels/lark.rs`

#### Scenario: Ensure send success rejects non-zero code

- WHEN send response has non-zero code
- THEN MUST treat as failure
- Test: `lark_ensure_send_success_rejects_non_zero_code` in `src/channels/lark.rs`

#### Scenario: User allowed exact match

- WHEN allowlist contains exact user
- THEN MUST allow
- Test: `lark_user_allowed_exact` in `src/channels/lark.rs`

#### Scenario: User allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all users
- Test: `lark_user_allowed_wildcard` in `src/channels/lark.rs`

#### Scenario: User denied empty allowlist

- WHEN allowlist is empty
- THEN MUST deny all users
- Test: `lark_user_denied_empty` in `src/channels/lark.rs`

#### Scenario: Parse challenge response

- WHEN a challenge verification request is received
- THEN MUST parse and respond correctly
- Test: `lark_parse_challenge` in `src/channels/lark.rs`

#### Scenario: Parse valid text message

- WHEN a valid text message event is received
- THEN MUST parse it correctly
- Test: `lark_parse_valid_text_message` in `src/channels/lark.rs`

#### Scenario: Parse valid text message with object content

- WHEN text message has object-type content
- THEN MUST parse the text from the object
- Test: `lark_parse_valid_text_message_with_object_content` in `src/channels/lark.rs`

#### Scenario: WS payload deserializes object content

- WHEN WebSocket payload has object content
- THEN MUST deserialize correctly
- Test: `lark_ws_payload_deserializes_object_content` in `src/channels/lark.rs`

#### Scenario: Parse unauthorized user

- WHEN message is from unauthorized user
- THEN MUST skip it
- Test: `lark_parse_unauthorized_user` in `src/channels/lark.rs`

#### Scenario: Parse image message uses fallback text

- WHEN an image message is received without text
- THEN MUST use fallback text
- Test: `lark_parse_image_message_uses_fallback_text` in `src/channels/lark.rs`

#### Scenario: Parse event async image missing key uses fallback

- WHEN async image event is missing key
- THEN MUST use fallback text
- Test: `lark_parse_event_payload_async_image_missing_key_uses_fallback_text` in `src/channels/lark.rs`

#### Scenario: Parse event async deduplicates repeated event ID

- WHEN the same event ID arrives twice
- THEN MUST deduplicate
- Test: `lark_parse_event_payload_async_dedupes_repeated_event_id` in `src/channels/lark.rs`

#### Scenario: Parse event async deduplicates by message_id

- WHEN event has no event_id but repeated message_id
- THEN MUST deduplicate by message_id
- Test: `lark_parse_event_payload_async_dedupes_by_message_id_without_event_id` in `src/channels/lark.rs`

#### Scenario: Event key seen cleans up expired keys periodically

- WHEN seen keys accumulate
- THEN MUST periodically clean up expired keys
- Test: `try_mark_event_key_seen_cleans_up_expired_keys_periodically` in `src/channels/lark.rs`

#### Scenario: Parse empty text is skipped

- WHEN text content is empty
- THEN MUST skip the message
- Test: `lark_parse_empty_text_skipped` in `src/channels/lark.rs`

#### Scenario: Parse wrong event type

- WHEN event type is not a message
- THEN MUST skip it
- Test: `lark_parse_wrong_event_type` in `src/channels/lark.rs`

#### Scenario: Parse missing sender

- WHEN message has no sender field
- THEN MUST handle gracefully
- Test: `lark_parse_missing_sender` in `src/channels/lark.rs`

#### Scenario: Parse unicode message

- WHEN message contains unicode text
- THEN MUST parse correctly
- Test: `lark_parse_unicode_message` in `src/channels/lark.rs`

#### Scenario: Parse missing event

- WHEN payload has no event field
- THEN MUST handle gracefully
- Test: `lark_parse_missing_event` in `src/channels/lark.rs`

#### Scenario: Parse invalid content JSON

- WHEN content field has invalid JSON
- THEN MUST handle gracefully
- Test: `lark_parse_invalid_content_json` in `src/channels/lark.rs`

#### Scenario: Config serde round trip

- WHEN config is serialized/deserialized as JSON
- THEN MUST round-trip correctly
- Test: `lark_config_serde` in `src/channels/lark.rs`

#### Scenario: Config TOML round trip

- WHEN config is serialized/deserialized as TOML
- THEN MUST round-trip correctly
- Test: `lark_config_toml_roundtrip` in `src/channels/lark.rs`

#### Scenario: Config defaults for optional fields

- WHEN optional config fields are omitted
- THEN MUST use defaults
- Test: `lark_config_defaults_optional_fields` in `src/channels/lark.rs`

#### Scenario: From config preserves mode and region

- WHEN constructing from config
- THEN MUST preserve mode and region settings
- Test: `lark_from_config_preserves_mode_and_region` in `src/channels/lark.rs`

#### Scenario: From lark config ignores legacy feishu flag

- WHEN constructing from lark config with legacy feishu flag
- THEN MUST ignore the legacy flag
- Test: `lark_from_lark_config_ignores_legacy_feishu_flag` in `src/channels/lark.rs`

#### Scenario: From feishu config sets feishu platform

- WHEN constructing from feishu config
- THEN MUST set feishu as the platform
- Test: `lark_from_feishu_config_sets_feishu_platform` in `src/channels/lark.rs`

#### Scenario: Parse fallback sender to open_id

- WHEN sender field is missing but open_id is present
- THEN MUST fall back to open_id
- Test: `lark_parse_fallback_sender_to_open_id` in `src/channels/lark.rs`

#### Scenario: Parse group message requires bot mention when enabled

- WHEN mention-only is enabled in group
- THEN MUST require bot mention
- Test: `lark_parse_group_message_requires_bot_mention_when_enabled` in `src/channels/lark.rs`

#### Scenario: Parse group post message accepts at when top-level mentions empty

- WHEN group post message has @ mention but top-level mentions array is empty
- THEN MUST accept the message
- Test: `lark_parse_group_post_message_accepts_at_when_top_level_mentions_empty` in `src/channels/lark.rs`

#### Scenario: Parse group message allows without mention when disabled

- WHEN mention-only is disabled
- THEN MUST allow group messages without mention
- Test: `lark_parse_group_message_allows_without_mention_when_disabled` in `src/channels/lark.rs`

#### Scenario: Reaction URL matches region

- WHEN building reaction URL
- THEN MUST match the configured region
- Test: `lark_reaction_url_matches_region` in `src/channels/lark.rs`

#### Scenario: Reaction locale explicit language tags

- WHEN explicit language tags are configured
- THEN MUST use them for reactions
- Test: `lark_reaction_locale_explicit_language_tags` in `src/channels/lark.rs`

#### Scenario: Reaction locale prefers explicit payload locale

- WHEN payload has an explicit locale
- THEN MUST prefer it over detection
- Test: `lark_reaction_locale_prefers_explicit_payload_locale` in `src/channels/lark.rs`

#### Scenario: Reaction locale unsupported payload falls back to text script

- WHEN payload locale is unsupported
- THEN MUST fall back to text script detection
- Test: `lark_reaction_locale_unsupported_payload_falls_back_to_text_script` in `src/channels/lark.rs`

#### Scenario: Reaction locale detects simplified and traditional text

- WHEN text contains simplified or traditional Chinese
- THEN MUST detect the correct locale
- Test: `lark_reaction_locale_detects_simplified_and_traditional_text` in `src/channels/lark.rs`

#### Scenario: Reaction locale defaults to English for unsupported text

- WHEN text language cannot be determined
- THEN MUST default to English
- Test: `lark_reaction_locale_defaults_to_english_for_unsupported_text` in `src/channels/lark.rs`

#### Scenario: Random ack reaction respects detected locale pool

- WHEN selecting a random ack reaction
- THEN MUST use the locale-appropriate pool
- Test: `random_lark_ack_reaction_respects_detected_locale_pool` in `src/channels/lark.rs`

### REQ-CHAN-017: Additional Platform Channels

Platform channels (GitHub, Mattermost, NextcloudTalk, MQTT, QQ, LinQ, iMessage, BlueBubbles, DingTalk, CLI, ACP, NapCat, ClawdTalk) MUST implement the Channel trait with platform-specific message handling.

#### REQ-CHAN-017-IMESSAGE: iMessage Channel (58 tests)

##### Scenario: Creates with contacts

- WHEN channel is created with contacts
- THEN MUST store the contacts
- Test: `creates_with_contacts` in `src/channels/imessage.rs`

##### Scenario: Creates with empty contacts

- WHEN channel is created with empty contacts
- THEN MUST handle gracefully
- Test: `creates_with_empty_contacts` in `src/channels/imessage.rs`

##### Scenario: Wildcard allows anyone

- WHEN allowlist contains wildcard
- THEN MUST allow all contacts
- Test: `wildcard_allows_anyone` in `src/channels/imessage.rs`

##### Scenario: Specific contact allowed

- WHEN allowlist contains specific contact
- THEN MUST allow that contact
- Test: `specific_contact_allowed` in `src/channels/imessage.rs`

##### Scenario: Unknown contact denied

- WHEN contact is not in allowlist
- THEN MUST deny
- Test: `unknown_contact_denied` in `src/channels/imessage.rs`

##### Scenario: Contact matching is case insensitive

- WHEN contact case differs from allowlist
- THEN MUST still match
- Test: `contact_case_insensitive` in `src/channels/imessage.rs`

##### Scenario: Empty allowlist denies all

- WHEN allowlist is empty
- THEN MUST deny all contacts
- Test: `empty_allowlist_denies_all` in `src/channels/imessage.rs`

##### Scenario: Name returns "imessage"

- WHEN name() is called
- THEN MUST return "imessage"
- Test: `name_returns_imessage` in `src/channels/imessage.rs`

##### Scenario: Wildcard among others still allows all

- WHEN wildcard is mixed with specific entries
- THEN wildcard MUST take precedence
- Test: `wildcard_among_others_still_allows_all` in `src/channels/imessage.rs`

##### Scenario: Contact with spaces requires exact match

- WHEN contact has spaces
- THEN MUST require exact match
- Test: `contact_with_spaces_exact_match` in `src/channels/imessage.rs`

##### Scenario: Escape AppleScript double quotes

- WHEN content has double quotes
- THEN MUST escape for AppleScript
- Test: `escape_applescript_double_quotes` in `src/channels/imessage.rs`

##### Scenario: Escape AppleScript backslashes

- WHEN content has backslashes
- THEN MUST escape for AppleScript
- Test: `escape_applescript_backslashes` in `src/channels/imessage.rs`

##### Scenario: Escape AppleScript mixed characters

- WHEN content has mixed special characters
- THEN MUST escape all of them
- Test: `escape_applescript_mixed` in `src/channels/imessage.rs`

##### Scenario: Escape AppleScript injection attempt

- WHEN content contains injection attempt
- THEN MUST escape to prevent injection
- Test: `escape_applescript_injection_attempt` in `src/channels/imessage.rs`

##### Scenario: Escape AppleScript empty string

- WHEN content is empty
- THEN MUST handle gracefully
- Test: `escape_applescript_empty_string` in `src/channels/imessage.rs`

##### Scenario: Escape AppleScript no special chars

- WHEN content has no special characters
- THEN MUST return unchanged
- Test: `escape_applescript_no_special_chars` in `src/channels/imessage.rs`

##### Scenario: Escape AppleScript unicode

- WHEN content has unicode characters
- THEN MUST preserve them
- Test: `escape_applescript_unicode` in `src/channels/imessage.rs`

##### Scenario: Escape AppleScript newlines

- WHEN content has newlines
- THEN MUST escape them
- Test: `escape_applescript_newlines_escaped` in `src/channels/imessage.rs`

##### Scenario: Valid phone number simple

- WHEN target is a simple phone number
- THEN MUST validate as valid
- Test: `valid_phone_number_simple` in `src/channels/imessage.rs`

##### Scenario: Valid phone number with country code

- WHEN target has country code prefix
- THEN MUST validate as valid
- Test: `valid_phone_number_with_country_code` in `src/channels/imessage.rs`

##### Scenario: Valid phone number with spaces

- WHEN target has spaces in phone number
- THEN MUST validate as valid
- Test: `valid_phone_number_with_spaces` in `src/channels/imessage.rs`

##### Scenario: Valid phone number with dashes

- WHEN target has dashes in phone number
- THEN MUST validate as valid
- Test: `valid_phone_number_with_dashes` in `src/channels/imessage.rs`

##### Scenario: Valid phone number international

- WHEN target is an international phone number
- THEN MUST validate as valid
- Test: `valid_phone_number_international` in `src/channels/imessage.rs`

##### Scenario: Valid email simple

- WHEN target is a simple email
- THEN MUST validate as valid
- Test: `valid_email_simple` in `src/channels/imessage.rs`

##### Scenario: Valid email with subdomain

- WHEN target is an email with subdomain
- THEN MUST validate as valid
- Test: `valid_email_with_subdomain` in `src/channels/imessage.rs`

##### Scenario: Valid email with plus

- WHEN target is an email with + alias
- THEN MUST validate as valid
- Test: `valid_email_with_plus` in `src/channels/imessage.rs`

##### Scenario: Valid email with dots

- WHEN target is an email with dots in local part
- THEN MUST validate as valid
- Test: `valid_email_with_dots` in `src/channels/imessage.rs`

##### Scenario: Valid email iCloud

- WHEN target is an iCloud email
- THEN MUST validate as valid
- Test: `valid_email_icloud` in `src/channels/imessage.rs`

##### Scenario: Invalid target empty

- WHEN target is empty
- THEN MUST reject
- Test: `invalid_target_empty` in `src/channels/imessage.rs`

##### Scenario: Invalid target no plus prefix for phone

- WHEN phone number lacks + prefix
- THEN MUST reject
- Test: `invalid_target_no_plus_prefix` in `src/channels/imessage.rs`

##### Scenario: Invalid target too short phone

- WHEN phone number is too short
- THEN MUST reject
- Test: `invalid_target_too_short_phone` in `src/channels/imessage.rs`

##### Scenario: Invalid target too long phone

- WHEN phone number is too long
- THEN MUST reject
- Test: `invalid_target_too_long_phone` in `src/channels/imessage.rs`

##### Scenario: Invalid target email no @

- WHEN email has no @ symbol
- THEN MUST reject
- Test: `invalid_target_email_no_at` in `src/channels/imessage.rs`

##### Scenario: Invalid target email no domain

- WHEN email has no domain
- THEN MUST reject
- Test: `invalid_target_email_no_domain` in `src/channels/imessage.rs`

##### Scenario: Invalid target email no local part

- WHEN email has no local part
- THEN MUST reject
- Test: `invalid_target_email_no_local` in `src/channels/imessage.rs`

##### Scenario: Invalid target email no dot in domain

- WHEN email domain has no dot
- THEN MUST reject
- Test: `invalid_target_email_no_dot_in_domain` in `src/channels/imessage.rs`

##### Scenario: Invalid target injection attempt

- WHEN target contains shell injection attempt
- THEN MUST reject
- Test: `invalid_target_injection_attempt` in `src/channels/imessage.rs`

##### Scenario: Invalid target AppleScript injection

- WHEN target contains AppleScript injection
- THEN MUST reject
- Test: `invalid_target_applescript_injection` in `src/channels/imessage.rs`

##### Scenario: Invalid target special chars

- WHEN target contains special characters
- THEN MUST reject
- Test: `invalid_target_special_chars` in `src/channels/imessage.rs`

##### Scenario: Invalid target null byte

- WHEN target contains null byte
- THEN MUST reject
- Test: `invalid_target_null_byte` in `src/channels/imessage.rs`

##### Scenario: Invalid target newline

- WHEN target contains newline
- THEN MUST reject
- Test: `invalid_target_newline` in `src/channels/imessage.rs`

##### Scenario: Target with whitespace is trimmed

- WHEN target has leading/trailing whitespace
- THEN MUST trim before validation
- Test: `target_with_leading_trailing_whitespace_trimmed` in `src/channels/imessage.rs`

##### Scenario: Get max rowid empty database

- WHEN database is empty
- THEN MUST return 0
- Test: `get_max_rowid_empty_database` in `src/channels/imessage.rs`

##### Scenario: Get max rowid with messages

- WHEN database has messages
- THEN MUST return the max rowid
- Test: `get_max_rowid_with_messages` in `src/channels/imessage.rs`

##### Scenario: Get max rowid nonexistent database

- WHEN database does not exist
- THEN MUST handle gracefully
- Test: `get_max_rowid_nonexistent_database` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages empty database

- WHEN database is empty
- THEN MUST return empty list
- Test: `fetch_new_messages_empty_database` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages returns correct data

- WHEN new messages exist
- THEN MUST return them with correct data
- Test: `fetch_new_messages_returns_correct_data` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages filters by rowid

- WHEN filtering by rowid
- THEN MUST only return messages after that rowid
- Test: `fetch_new_messages_filters_by_rowid` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages excludes sent messages

- WHEN messages include sent (is_from_me) messages
- THEN MUST exclude them
- Test: `fetch_new_messages_excludes_sent_messages` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages excludes null text

- WHEN messages have null text
- THEN MUST exclude them
- Test: `fetch_new_messages_excludes_null_text` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages respects limit

- WHEN a limit is set
- THEN MUST return at most that many messages
- Test: `fetch_new_messages_respects_limit` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages ordered by rowid ascending

- WHEN fetching new messages
- THEN MUST order by rowid ascending
- Test: `fetch_new_messages_ordered_by_rowid_asc` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages nonexistent database

- WHEN database does not exist
- THEN MUST handle gracefully
- Test: `fetch_new_messages_nonexistent_database` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages handles special characters

- WHEN messages contain special characters
- THEN MUST handle them correctly
- Test: `fetch_new_messages_handles_special_characters` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages handles unicode

- WHEN messages contain unicode
- THEN MUST handle correctly
- Test: `fetch_new_messages_handles_unicode` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages handles empty text

- WHEN messages have empty text (not null)
- THEN MUST handle correctly
- Test: `fetch_new_messages_handles_empty_text` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages negative rowid edge case

- WHEN rowid is negative
- THEN MUST handle edge case
- Test: `fetch_new_messages_negative_rowid_edge_case` in `src/channels/imessage.rs`

##### Scenario: Fetch new messages large rowid edge case

- WHEN rowid is very large
- THEN MUST handle edge case
- Test: `fetch_new_messages_large_rowid_edge_case` in `src/channels/imessage.rs`

#### REQ-CHAN-017-BLUEBUBBLES: BlueBubbles Channel (41 tests)

##### Scenario: Channel name is "bluebubbles"

- WHEN name() is called
- THEN MUST return "bluebubbles"
- Test: `bluebubbles_channel_name` in `src/channels/bluebubbles.rs`

##### Scenario: Sender allowed exact match

- WHEN allowlist contains exact sender
- THEN MUST allow
- Test: `bluebubbles_sender_allowed_exact` in `src/channels/bluebubbles.rs`

##### Scenario: Sender allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all
- Test: `bluebubbles_sender_allowed_wildcard` in `src/channels/bluebubbles.rs`

##### Scenario: Sender allowed empty list allows all

- WHEN allowlist is empty
- THEN MUST allow all (BlueBubbles-specific behavior)
- Test: `bluebubbles_sender_allowed_empty_list_allows_all` in `src/channels/bluebubbles.rs`

##### Scenario: Server URL trailing slash trimmed

- WHEN server URL has trailing slash
- THEN MUST trim it
- Test: `bluebubbles_server_url_trailing_slash_trimmed` in `src/channels/bluebubbles.rs`

##### Scenario: Normalize handle strips service prefix

- WHEN handle has service prefix
- THEN MUST strip it
- Test: `bluebubbles_normalize_handle_strips_service_prefix` in `src/channels/bluebubbles.rs`

##### Scenario: Normalize handle email lowercased

- WHEN handle is an email
- THEN MUST lowercase it
- Test: `bluebubbles_normalize_handle_email_lowercased` in `src/channels/bluebubbles.rs`

##### Scenario: Parse valid DM message

- WHEN a valid DM webhook is received
- THEN MUST parse it correctly
- Test: `bluebubbles_parse_valid_dm_message` in `src/channels/bluebubbles.rs`

##### Scenario: Parse group chat message

- WHEN a group chat webhook is received
- THEN MUST parse it correctly
- Test: `bluebubbles_parse_group_chat_message` in `src/channels/bluebubbles.rs`

##### Scenario: Parse skip is_from_me

- WHEN message is from the bot
- THEN MUST skip it
- Test: `bluebubbles_parse_skip_is_from_me` in `src/channels/bluebubbles.rs`

##### Scenario: Parse skip non-message event

- WHEN event is not a message
- THEN MUST skip it
- Test: `bluebubbles_parse_skip_non_message_event` in `src/channels/bluebubbles.rs`

##### Scenario: Parse skip unauthorized sender

- WHEN sender is unauthorized
- THEN MUST skip it
- Test: `bluebubbles_parse_skip_unauthorized_sender` in `src/channels/bluebubbles.rs`

##### Scenario: Parse skip empty text no attachments

- WHEN message has empty text and no attachments
- THEN MUST skip it
- Test: `bluebubbles_parse_skip_empty_text_no_attachments` in `src/channels/bluebubbles.rs`

##### Scenario: Parse image attachment

- WHEN message has image attachment
- THEN MUST parse and format as image marker
- Test: `bluebubbles_parse_image_attachment` in `src/channels/bluebubbles.rs`

##### Scenario: Parse non-image attachment

- WHEN message has non-image attachment
- THEN MUST parse as document
- Test: `bluebubbles_parse_non_image_attachment` in `src/channels/bluebubbles.rs`

##### Scenario: Parse text with attachment

- WHEN message has both text and attachment
- THEN MUST combine them
- Test: `bluebubbles_parse_text_with_attachment` in `src/channels/bluebubbles.rs`

##### Scenario: Parse fallback reply target when no chats

- WHEN message has no chats field
- THEN MUST use fallback reply target
- Test: `bluebubbles_parse_fallback_reply_target_when_no_chats` in `src/channels/bluebubbles.rs`

##### Scenario: Parse missing data field

- WHEN webhook has missing data field
- THEN MUST handle gracefully
- Test: `bluebubbles_parse_missing_data_field` in `src/channels/bluebubbles.rs`

##### Scenario: Parse email handle

- WHEN sender uses email handle
- THEN MUST parse correctly
- Test: `bluebubbles_parse_email_handle` in `src/channels/bluebubbles.rs`

##### Scenario: Parse direct chat guid field

- WHEN message has chatGuid field
- THEN MUST use it for routing
- Test: `bluebubbles_parse_direct_chat_guid_field` in `src/channels/bluebubbles.rs`

##### Scenario: Parse timestamp not double divided

- WHEN timestamp is in seconds
- THEN MUST not double-divide
- Test: `bluebubbles_parse_timestamp_seconds_not_double_divided` in `src/channels/bluebubbles.rs`

##### Scenario: Parse video attachment

- WHEN message has video attachment
- THEN MUST parse as video
- Test: `bluebubbles_parse_video_attachment` in `src/channels/bluebubbles.rs`

##### Scenario: Parse multiple images

- WHEN message has multiple image attachments
- THEN MUST parse all of them
- Test: `bluebubbles_parse_multiple_images` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body plain text no markers

- WHEN plain text has no formatting
- THEN MUST return unchanged
- Test: `attributed_body_plain_text_no_markers` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body bold

- WHEN text has bold formatting
- THEN MUST produce bold markers
- Test: `attributed_body_bold` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body italic

- WHEN text has italic formatting
- THEN MUST produce italic markers
- Test: `attributed_body_italic` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body strikethrough

- WHEN text has strikethrough formatting
- THEN MUST produce strikethrough markers
- Test: `attributed_body_strikethrough` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body underline

- WHEN text has underline formatting
- THEN MUST produce underline markers
- Test: `attributed_body_underline` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body mixed three segments

- WHEN text has three different formatted segments
- THEN MUST produce correct markers for each
- Test: `attributed_body_mixed_three_segments` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body nested bold italic

- WHEN text has nested bold+italic formatting
- THEN MUST produce nested markers
- Test: `attributed_body_nested_bold_italic` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body empty string

- WHEN text is empty
- THEN MUST return empty
- Test: `attributed_body_empty_string` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body preserved in send message field

- WHEN plain text is in a send message
- THEN MUST preserve it
- Test: `attributed_body_plain_text_preserved_in_send_message_field` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body inline code renders as bold

- WHEN text has inline code
- THEN MUST render as bold (platform limitation)
- Test: `attributed_body_inline_code_renders_as_bold` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body inline code in sentence

- WHEN inline code is within a sentence
- THEN MUST render correctly
- Test: `attributed_body_inline_code_in_sentence` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body header bold

- WHEN text has header formatting
- THEN MUST render as bold
- Test: `attributed_body_header_bold` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body header resets after newline

- WHEN header formatting is followed by a newline
- THEN MUST reset formatting
- Test: `attributed_body_header_resets_after_newline` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body code block fences stripped

- WHEN text has plain code block fences
- THEN MUST strip fences
- Test: `attributed_body_code_block_plain_fences_stripped` in `src/channels/bluebubbles.rs`

##### Scenario: Attributed body code block with language hint

- WHEN code block has a language hint
- THEN MUST strip the hint
- Test: `attributed_body_code_block_with_language_hint` in `src/channels/bluebubbles.rs`

##### Scenario: Ignore sender exact match

- WHEN ignore list contains exact sender
- THEN MUST ignore that sender
- Test: `bluebubbles_ignore_sender_exact` in `src/channels/bluebubbles.rs`

##### Scenario: Ignore sender takes precedence over allowlist

- WHEN sender is in both ignore and allow lists
- THEN ignore MUST take precedence
- Test: `bluebubbles_ignore_sender_takes_precedence_over_allowlist` in `src/channels/bluebubbles.rs`

##### Scenario: Ignore sender empty list ignores nothing

- WHEN ignore list is empty
- THEN MUST not ignore any sender
- Test: `bluebubbles_ignore_sender_empty_list_ignores_nothing` in `src/channels/bluebubbles.rs`

#### REQ-CHAN-017-MATTERMOST: Mattermost Channel (35 tests)

##### Scenario: URL trimming

- WHEN server URL has extra whitespace or slashes
- THEN MUST trim them
- Test: `mattermost_url_trimming` in `src/channels/mattermost.rs`

##### Scenario: Allowlist wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all
- Test: `mattermost_allowlist_wildcard` in `src/channels/mattermost.rs`

##### Scenario: Parse post basic

- WHEN a basic post is received
- THEN MUST parse it correctly
- Test: `mattermost_parse_post_basic` in `src/channels/mattermost.rs`

##### Scenario: Parse post thread replies enabled

- WHEN thread replies are enabled
- THEN MUST extract thread root ID
- Test: `mattermost_parse_post_thread_replies_enabled` in `src/channels/mattermost.rs`

##### Scenario: Parse post thread

- WHEN post is in a thread
- THEN MUST extract thread_ts
- Test: `mattermost_parse_post_thread` in `src/channels/mattermost.rs`

##### Scenario: Parse post ignore self

- WHEN post is from the bot
- THEN MUST ignore it
- Test: `mattermost_parse_post_ignore_self` in `src/channels/mattermost.rs`

##### Scenario: Parse post ignore old

- WHEN post is too old
- THEN MUST ignore it
- Test: `mattermost_parse_post_ignore_old` in `src/channels/mattermost.rs`

##### Scenario: Parse post no thread when disabled

- WHEN thread replies are disabled
- THEN MUST not extract thread_ts
- Test: `mattermost_parse_post_no_thread_when_disabled` in `src/channels/mattermost.rs`

##### Scenario: Existing thread always threads

- WHEN post is already in a thread
- THEN MUST always thread regardless of config
- Test: `mattermost_existing_thread_always_threads` in `src/channels/mattermost.rs`

##### Scenario: Mention-only skips message without mention

- WHEN mention-only is enabled and message lacks mention
- THEN MUST skip
- Test: `mention_only_skips_message_without_mention` in `src/channels/mattermost.rs`

##### Scenario: Mention-only accepts message with @mention

- WHEN mention-only is enabled and message has @mention
- THEN MUST accept
- Test: `mention_only_accepts_message_with_at_mention` in `src/channels/mattermost.rs`

##### Scenario: Mention-only strips mention and trims

- WHEN mention is stripped
- THEN MUST trim remaining whitespace
- Test: `mention_only_strips_mention_and_trims` in `src/channels/mattermost.rs`

##### Scenario: Mention-only rejects empty after stripping

- WHEN message is empty after mention stripping
- THEN MUST reject
- Test: `mention_only_rejects_empty_after_stripping` in `src/channels/mattermost.rs`

##### Scenario: Mention-only case insensitive

- WHEN mention case differs
- THEN MUST still match
- Test: `mention_only_case_insensitive` in `src/channels/mattermost.rs`

##### Scenario: Mention-only detects metadata mentions

- WHEN mention is in metadata
- THEN MUST detect it
- Test: `mention_only_detects_metadata_mentions` in `src/channels/mattermost.rs`

##### Scenario: Mention-only word boundary prevents partial match

- WHEN mention-like text is part of another word
- THEN MUST not match
- Test: `mention_only_word_boundary_prevents_partial_match` in `src/channels/mattermost.rs`

##### Scenario: Mention-only mention in middle of text

- WHEN mention is in the middle of text
- THEN MUST detect it
- Test: `mention_only_mention_in_middle_of_text` in `src/channels/mattermost.rs`

##### Scenario: Mention-only disabled passes all messages

- WHEN mention-only is disabled
- THEN MUST pass all messages
- Test: `mention_only_disabled_passes_all_messages` in `src/channels/mattermost.rs`

##### Scenario: Mention-only sender override allows without mention

- WHEN sender has override configured
- THEN MUST allow without mention
- Test: `mention_only_sender_override_allows_without_mention` in `src/channels/mattermost.rs`

##### Scenario: Contains mention text at end

- WHEN mention is at end of text
- THEN MUST detect it
- Test: `contains_mention_text_at_end` in `src/channels/mattermost.rs`

##### Scenario: Contains mention text at start

- WHEN mention is at start of text
- THEN MUST detect it
- Test: `contains_mention_text_at_start` in `src/channels/mattermost.rs`

##### Scenario: Contains mention text alone

- WHEN message is just the mention
- THEN MUST detect it
- Test: `contains_mention_text_alone` in `src/channels/mattermost.rs`

##### Scenario: No mention different username

- WHEN message mentions a different username
- THEN MUST not detect
- Test: `no_mention_different_username` in `src/channels/mattermost.rs`

##### Scenario: No mention partial username

- WHEN message has partial username match
- THEN MUST not detect
- Test: `no_mention_partial_username` in `src/channels/mattermost.rs`

##### Scenario: Mention detects later valid mention after partial prefix

- WHEN text has partial prefix then valid mention
- THEN MUST detect the valid mention
- Test: `mention_detects_later_valid_mention_after_partial_prefix` in `src/channels/mattermost.rs`

##### Scenario: Mention followed by punctuation

- WHEN mention is followed by punctuation
- THEN MUST still detect it
- Test: `mention_followed_by_punctuation` in `src/channels/mattermost.rs`

##### Scenario: Mention via metadata only

- WHEN mention is only in metadata
- THEN MUST detect it
- Test: `mention_via_metadata_only` in `src/channels/mattermost.rs`

##### Scenario: No mention empty username no metadata

- WHEN username is empty and no metadata
- THEN MUST not detect mention
- Test: `no_mention_empty_username_no_metadata` in `src/channels/mattermost.rs`

##### Scenario: Normalize strips and trims

- WHEN normalizing content with mention
- THEN MUST strip mention and trim
- Test: `normalize_strips_and_trims` in `src/channels/mattermost.rs`

##### Scenario: Normalize returns None for no mention

- WHEN content has no mention in mention-only mode
- THEN MUST return None
- Test: `normalize_returns_none_for_no_mention` in `src/channels/mattermost.rs`

##### Scenario: Normalize returns None when only mention

- WHEN content is just the mention
- THEN MUST return None (empty after strip)
- Test: `normalize_returns_none_when_only_mention` in `src/channels/mattermost.rs`

##### Scenario: Normalize preserves text for metadata mention

- WHEN mention is detected via metadata
- THEN MUST preserve full text
- Test: `normalize_preserves_text_for_metadata_mention` in `src/channels/mattermost.rs`

##### Scenario: Normalize strips multiple mentions

- WHEN text has multiple mentions
- THEN MUST strip all of them
- Test: `normalize_strips_multiple_mentions` in `src/channels/mattermost.rs`

##### Scenario: Normalize keeps partial username mentions

- WHEN text has partial username that looks like mention
- THEN MUST keep it (not a real mention)
- Test: `normalize_keeps_partial_username_mentions` in `src/channels/mattermost.rs`

##### Scenario: Normalize group reply allowed sender IDs deduplicates

- WHEN sender IDs have duplicates
- THEN MUST deduplicate
- Test: `normalize_group_reply_allowed_sender_ids_deduplicates` in `src/channels/mattermost.rs`

#### REQ-CHAN-017-GITHUB: GitHub Channel (9 tests)

##### Scenario: GitHub channel name

- WHEN name() is called
- THEN MUST return "github"
- Test: `github_channel_name` in `src/channels/github.rs`

##### Scenario: Verify GitHub signature valid

- WHEN webhook signature is valid
- THEN MUST accept
- Test: `verify_github_signature_valid` in `src/channels/github.rs`

##### Scenario: Verify GitHub signature rejects invalid

- WHEN webhook signature is invalid
- THEN MUST reject
- Test: `verify_github_signature_rejects_invalid` in `src/channels/github.rs`

##### Scenario: Parse issue comment event created

- WHEN issue comment creation event is received
- THEN MUST parse it
- Test: `parse_issue_comment_event_created` in `src/channels/github.rs`

##### Scenario: Parse issue comment event skips bot actor

- WHEN issue comment is from a bot
- THEN MUST skip it
- Test: `parse_issue_comment_event_skips_bot_actor` in `src/channels/github.rs`

##### Scenario: Parse issue comment event blocks unallowed repo

- WHEN event is from an unallowed repo
- THEN MUST block it
- Test: `parse_issue_comment_event_blocks_unallowed_repo` in `src/channels/github.rs`

##### Scenario: Parse PR review comment event created

- WHEN PR review comment creation event is received
- THEN MUST parse it
- Test: `parse_pr_review_comment_event_created` in `src/channels/github.rs`

##### Scenario: Parse issue recipient format

- WHEN building issue recipient
- THEN MUST use correct format
- Test: `parse_issue_recipient_format` in `src/channels/github.rs`

##### Scenario: Allowlist supports wildcards

- WHEN repo allowlist has wildcards
- THEN MUST match repos by wildcard
- Test: `allowlist_supports_wildcards` in `src/channels/github.rs`

#### REQ-CHAN-017-NEXTCLOUD: Nextcloud Talk Channel (11 tests)

##### Scenario: Nextcloud Talk channel name

- WHEN name() is called
- THEN MUST return "nextcloud_talk"
- Test: `nextcloud_talk_channel_name` in `src/channels/nextcloud_talk.rs`

##### Scenario: User allowlist exact and wildcard

- WHEN allowlist has exact and wildcard entries
- THEN MUST match accordingly
- Test: `nextcloud_talk_user_allowlist_exact_and_wildcard` in `src/channels/nextcloud_talk.rs`

##### Scenario: Parse valid message payload

- WHEN a valid message webhook is received
- THEN MUST parse it correctly
- Test: `nextcloud_talk_parse_valid_message_payload` in `src/channels/nextcloud_talk.rs`

##### Scenario: Parse skips non-message events

- WHEN event is not a message
- THEN MUST skip
- Test: `nextcloud_talk_parse_skips_non_message_events` in `src/channels/nextcloud_talk.rs`

##### Scenario: Parse skips bot messages

- WHEN message is from a bot
- THEN MUST skip
- Test: `nextcloud_talk_parse_skips_bot_messages` in `src/channels/nextcloud_talk.rs`

##### Scenario: Parse skips unauthorized sender

- WHEN sender is unauthorized
- THEN MUST skip
- Test: `nextcloud_talk_parse_skips_unauthorized_sender` in `src/channels/nextcloud_talk.rs`

##### Scenario: Parse skips system message

- WHEN event is a system message
- THEN MUST skip
- Test: `nextcloud_talk_parse_skips_system_message` in `src/channels/nextcloud_talk.rs`

##### Scenario: Parse timestamp millis to seconds

- WHEN timestamp is in milliseconds
- THEN MUST convert to seconds
- Test: `nextcloud_talk_parse_timestamp_millis_to_seconds` in `src/channels/nextcloud_talk.rs`

##### Scenario: Signature verification valid

- WHEN webhook signature is valid
- THEN MUST accept
- Test: `nextcloud_talk_signature_verification_valid` in `src/channels/nextcloud_talk.rs`

##### Scenario: Signature verification invalid

- WHEN webhook signature is invalid
- THEN MUST reject
- Test: `nextcloud_talk_signature_verification_invalid` in `src/channels/nextcloud_talk.rs`

##### Scenario: Signature verification accepts sha256 prefix

- WHEN signature uses sha256= prefix
- THEN MUST accept
- Test: `nextcloud_talk_signature_verification_accepts_sha256_prefix` in `src/channels/nextcloud_talk.rs`

#### REQ-CHAN-017-MQTT: MQTT Channel (12 tests)

##### Scenario: Config validation rejects bad QoS

- WHEN QoS value is invalid
- THEN MUST reject
- Test: `mqtt_config_validation_rejects_bad_qos` in `src/channels/mqtt.rs`

##### Scenario: Config validation rejects bad URL

- WHEN broker URL is invalid
- THEN MUST reject
- Test: `mqtt_config_validation_rejects_bad_url` in `src/channels/mqtt.rs`

##### Scenario: Config validation rejects empty topics

- WHEN topic list is empty
- THEN MUST reject
- Test: `mqtt_config_validation_rejects_empty_topics` in `src/channels/mqtt.rs`

##### Scenario: Config validation rejects empty client ID

- WHEN client_id is empty
- THEN MUST reject
- Test: `mqtt_config_validation_rejects_empty_client_id` in `src/channels/mqtt.rs`

##### Scenario: Config validation accepts valid config

- WHEN config is valid
- THEN MUST accept
- Test: `mqtt_config_validation_accepts_valid` in `src/channels/mqtt.rs`

##### Scenario: TLS flag rejects mqtt scheme with use_tls

- WHEN mqtt:// is used with use_tls=true
- THEN MUST reject
- Test: `mqtt_tls_flag_rejects_mqtt_scheme_with_use_tls` in `src/channels/mqtt.rs`

##### Scenario: TLS flag rejects mqtts scheme without use_tls

- WHEN mqtts:// is used with use_tls=false
- THEN MUST reject
- Test: `mqtt_tls_flag_rejects_mqtts_scheme_without_use_tls` in `src/channels/mqtt.rs`

##### Scenario: TLS flag accepts mqtts with use_tls

- WHEN mqtts:// is used with use_tls=true
- THEN MUST accept
- Test: `mqtt_tls_flag_accepts_mqtts_with_use_tls` in `src/channels/mqtt.rs`

##### Scenario: Broker host extraction

- WHEN extracting host from broker URL
- THEN MUST extract correctly
- Test: `broker_host_extracts_host` in `src/channels/mqtt.rs`

##### Scenario: Broker port extraction

- WHEN extracting port from broker URL
- THEN MUST extract correctly
- Test: `broker_port_extracts_port` in `src/channels/mqtt.rs`

##### Scenario: Broker port defaults 1883 for mqtt

- WHEN port is not specified for mqtt://
- THEN MUST default to 1883
- Test: `broker_port_defaults_1883_for_mqtt` in `src/channels/mqtt.rs`

##### Scenario: Broker port defaults 8883 for mqtts

- WHEN port is not specified for mqtts://
- THEN MUST default to 8883
- Test: `broker_port_defaults_8883_for_mqtts` in `src/channels/mqtt.rs`

#### REQ-CHAN-017-DINGTALK: DingTalk Channel (14 tests)

##### Scenario: DingTalk channel name

- WHEN name() is called
- THEN MUST return "dingtalk"
- Test: `test_name` in `src/channels/dingtalk.rs`

##### Scenario: User allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all
- Test: `test_user_allowed_wildcard` in `src/channels/dingtalk.rs`

##### Scenario: User allowed specific

- WHEN allowlist contains specific user
- THEN MUST allow
- Test: `test_user_allowed_specific` in `src/channels/dingtalk.rs`

##### Scenario: User denied empty

- WHEN allowlist is empty
- THEN MUST deny all
- Test: `test_user_denied_empty` in `src/channels/dingtalk.rs`

##### Scenario: Config serde

- WHEN config is serialized/deserialized
- THEN MUST round-trip correctly
- Test: `test_config_serde` in `src/channels/dingtalk.rs`

##### Scenario: Config serde defaults

- WHEN config uses defaults
- THEN MUST have sensible defaults
- Test: `test_config_serde_defaults` in `src/channels/dingtalk.rs`

##### Scenario: Parse stream data supports string payload

- WHEN stream data is a string
- THEN MUST parse correctly
- Test: `parse_stream_data_supports_string_payload` in `src/channels/dingtalk.rs`

##### Scenario: Parse stream data supports object payload

- WHEN stream data is an object
- THEN MUST parse correctly
- Test: `parse_stream_data_supports_object_payload` in `src/channels/dingtalk.rs`

##### Scenario: Resolve chat_id handles numeric group conversation type

- WHEN conversation type is numeric group
- THEN MUST resolve chat_id
- Test: `resolve_chat_id_handles_numeric_group_conversation_type` in `src/channels/dingtalk.rs`

##### Scenario: Extract text content prefers nested text content

- WHEN text field has nested content object
- THEN MUST prefer nested text.content
- Test: `extract_text_content_prefers_nested_text_content` in `src/channels/dingtalk.rs`

##### Scenario: Extract text content supports JSON-encoded text string

- WHEN text field is a JSON-encoded string
- THEN MUST decode and extract
- Test: `extract_text_content_supports_json_encoded_text_string` in `src/channels/dingtalk.rs`

##### Scenario: Extract text content falls back to content and markdown

- WHEN primary text fields are missing
- THEN MUST fall back to content/markdown fields
- Test: `extract_text_content_falls_back_to_content_and_markdown` in `src/channels/dingtalk.rs`

##### Scenario: Extract text content supports rich text payload

- WHEN rich text payload is present
- THEN MUST extract from rich text
- Test: `extract_text_content_supports_rich_text_payload` in `src/channels/dingtalk.rs`

##### Scenario: Extract text content bounds rich text recursion depth

- WHEN rich text has deep nesting
- THEN MUST bound recursion depth
- Test: `extract_text_content_bounds_rich_text_recursion_depth` in `src/channels/dingtalk.rs`

#### REQ-CHAN-017-NAPCAT: NapCat Channel (22 tests)

##### Scenario: Derive API base converts ws to http

- WHEN WebSocket URL uses ws://
- THEN MUST derive http:// API base
- Test: `derive_api_base_converts_ws_to_http` in `src/channels/napcat.rs`

##### Scenario: Derive API base converts wss to https

- WHEN WebSocket URL uses wss://
- THEN MUST derive https:// API base
- Test: `derive_api_base_converts_wss_to_https` in `src/channels/napcat.rs`

##### Scenario: Derive API base invalid scheme returns None

- WHEN WebSocket URL has invalid scheme
- THEN MUST return None
- Test: `derive_api_base_invalid_scheme_returns_none` in `src/channels/napcat.rs`

##### Scenario: Normalize token trims and filters empty

- WHEN token has whitespace or is empty
- THEN MUST trim and filter
- Test: `normalize_token_trims_and_filters_empty` in `src/channels/napcat.rs`

##### Scenario: Compose OneBot content includes reply and image markers

- WHEN content has reply and image markers
- THEN MUST compose OneBot content with them
- Test: `compose_onebot_content_includes_reply_and_image_markers` in `src/channels/napcat.rs`

##### Scenario: Compose OneBot content no reply no image

- WHEN content has no reply or image
- THEN MUST compose plain text OneBot content
- Test: `compose_onebot_content_no_reply_no_image` in `src/channels/napcat.rs`

##### Scenario: Compose OneBot content skips empty reply ID

- WHEN reply ID is empty
- THEN MUST skip the reply segment
- Test: `compose_onebot_content_skips_empty_reply_id` in `src/channels/napcat.rs`

##### Scenario: Parse message segments plain text

- WHEN message is plain text
- THEN MUST parse as text segment
- Test: `parse_message_segments_plain_text` in `src/channels/napcat.rs`

##### Scenario: Parse message segments array with text and image

- WHEN message is array with text and image
- THEN MUST parse both segments
- Test: `parse_message_segments_array_with_text_and_image` in `src/channels/napcat.rs`

##### Scenario: Parse message segments empty array

- WHEN message array is empty
- THEN MUST return empty
- Test: `parse_message_segments_empty_array` in `src/channels/napcat.rs`

##### Scenario: Parse message segments non-string non-array

- WHEN message is neither string nor array
- THEN MUST handle gracefully
- Test: `parse_message_segments_non_string_non_array` in `src/channels/napcat.rs`

##### Scenario: Extract message ID integer

- WHEN message_id is an integer
- THEN MUST extract it
- Test: `extract_message_id_integer` in `src/channels/napcat.rs`

##### Scenario: Extract message ID string

- WHEN message_id is a string
- THEN MUST extract it
- Test: `extract_message_id_string` in `src/channels/napcat.rs`

##### Scenario: Extract message ID missing generates UUID

- WHEN message_id is missing
- THEN MUST generate a UUID
- Test: `extract_message_id_missing_generates_uuid` in `src/channels/napcat.rs`

##### Scenario: From config empty WebSocket URL fails

- WHEN WebSocket URL is empty
- THEN MUST fail
- Test: `from_config_empty_websocket_url_fails` in `src/channels/napcat.rs`

##### Scenario: From config derives API base from WebSocket

- WHEN constructing from config
- THEN MUST derive API base from WebSocket URL
- Test: `from_config_derives_api_base_from_websocket` in `src/channels/napcat.rs`

##### Scenario: Is user allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all
- Test: `is_user_allowed_wildcard` in `src/channels/napcat.rs`

##### Scenario: Is user allowed specific

- WHEN allowlist contains specific user
- THEN MUST allow only that user
- Test: `is_user_allowed_specific` in `src/channels/napcat.rs`

##### Scenario: Is duplicate tracks message IDs

- WHEN same message ID is seen twice
- THEN MUST detect duplicate
- Test: `is_duplicate_tracks_message_ids` in `src/channels/napcat.rs`

##### Scenario: Is duplicate empty ID not tracked

- WHEN message ID is empty
- THEN MUST not track it
- Test: `is_duplicate_empty_id_not_tracked` in `src/channels/napcat.rs`

##### Scenario: Parse private event maps to channel message

- WHEN a private message event is received
- THEN MUST map to ChannelMessage
- Test: `parse_private_event_maps_to_channel_message` in `src/channels/napcat.rs`

##### Scenario: Parse group event with image segment

- WHEN a group message has an image segment
- THEN MUST parse the image marker
- Test: `parse_group_event_with_image_segment` in `src/channels/napcat.rs`

#### REQ-CHAN-017-QQ: QQ Channel (21 tests)

##### Scenario: QQ channel name

- WHEN name() is called
- THEN MUST return "qq"
- Test: `qq_channel_name` in `src/channels/qq.rs`

##### Scenario: QQ user allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all
- Test: `qq_user_allowed_wildcard` in `src/channels/qq.rs`

##### Scenario: QQ user allowed specific

- WHEN allowlist contains specific user
- THEN MUST allow
- Test: `qq_user_allowed_specific` in `src/channels/qq.rs`

##### Scenario: QQ user denied empty

- WHEN allowlist is empty
- THEN MUST deny all
- Test: `qq_user_denied_empty` in `src/channels/qq.rs`

##### Scenario: QQ webhook signature valid

- WHEN webhook signature is valid
- THEN MUST accept
- Test: `qq_webhook_signature_valid` in `src/channels/qq.rs`

##### Scenario: QQ webhook signature invalid

- WHEN webhook signature is invalid
- THEN MUST reject
- Test: `qq_webhook_signature_invalid` in `src/channels/qq.rs`

##### Scenario: QQ webhook validation request

- WHEN a validation request is received
- THEN MUST respond correctly
- Test: `qq_webhook_validation_request` in `src/channels/qq.rs`

##### Scenario: QQ parse C2C message

- WHEN a C2C (private) message is received
- THEN MUST parse it
- Test: `qq_parse_c2c_message` in `src/channels/qq.rs`

##### Scenario: QQ parse group message

- WHEN a group message is received
- THEN MUST parse it
- Test: `qq_parse_group_message` in `src/channels/qq.rs`

##### Scenario: QQ parse skips unauthorized

- WHEN sender is unauthorized
- THEN MUST skip
- Test: `qq_parse_skips_unauthorized` in `src/channels/qq.rs`

##### Scenario: QQ compose reply segments

- WHEN composing a reply
- THEN MUST build correct message segments
- Test: `qq_compose_reply_segments` in `src/channels/qq.rs`

##### Scenario: QQ compose message with image markers

- WHEN message contains image markers
- THEN MUST compose with image segments
- Test: `qq_compose_message_with_image_markers` in `src/channels/qq.rs`

##### Scenario: QQ parse message strips at mention

- WHEN message contains @mention
- THEN MUST strip it
- Test: `qq_parse_message_strips_at_mention` in `src/channels/qq.rs`

##### Scenario: QQ config serde

- WHEN config is serialized/deserialized
- THEN MUST round-trip correctly
- Test: `qq_config_serde` in `src/channels/qq.rs`

##### Scenario: QQ config defaults

- WHEN config uses defaults
- THEN MUST have sensible defaults
- Test: `qq_config_defaults` in `src/channels/qq.rs`

##### Scenario: QQ health check disconnected

- WHEN channel is not connected
- THEN health_check MUST return false
- Test: `qq_health_check_disconnected` in `src/channels/qq.rs`

##### Scenario: QQ message ID format

- WHEN a QQ message ID is generated
- THEN MUST have correct format
- Test: `qq_message_id_format` in `src/channels/qq.rs`

##### Scenario: QQ duplicate detection

- WHEN same message is received twice
- THEN MUST detect duplicate
- Test: `qq_duplicate_detection` in `src/channels/qq.rs`

##### Scenario: QQ parse empty text skipped

- WHEN message has empty text
- THEN MUST skip
- Test: `qq_parse_empty_text_skipped` in `src/channels/qq.rs`

##### Scenario: QQ webhook challenge response

- WHEN a challenge request is received
- THEN MUST respond with correct challenge
- Test: `qq_webhook_challenge_response` in `src/channels/qq.rs`

##### Scenario: QQ parse at mention in group

- WHEN group message has @mention of bot
- THEN MUST detect and handle
- Test: `qq_parse_at_mention_in_group` in `src/channels/qq.rs`

#### REQ-CHAN-017-LINQ: LinQ Channel (25 tests)

##### Scenario: LinQ channel name

- WHEN name() is called
- THEN MUST return "linq"
- Test: `linq_channel_name` in `src/channels/linq.rs`

##### Scenario: LinQ user allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all
- Test: `linq_user_allowed_wildcard` in `src/channels/linq.rs`

##### Scenario: LinQ user allowed specific

- WHEN allowlist contains specific user
- THEN MUST allow
- Test: `linq_user_allowed_specific` in `src/channels/linq.rs`

##### Scenario: LinQ user denied empty

- WHEN allowlist is empty
- THEN MUST deny all
- Test: `linq_user_denied_empty` in `src/channels/linq.rs`

##### Scenario: LinQ signature verification valid

- WHEN webhook signature is valid
- THEN MUST accept
- Test: `linq_signature_valid` in `src/channels/linq.rs`

##### Scenario: LinQ signature verification invalid

- WHEN webhook signature is invalid
- THEN MUST reject
- Test: `linq_signature_invalid` in `src/channels/linq.rs`

##### Scenario: LinQ parse valid message

- WHEN a valid webhook message is received
- THEN MUST parse it
- Test: `linq_parse_valid_message` in `src/channels/linq.rs`

##### Scenario: LinQ parse skips unauthorized

- WHEN sender is unauthorized
- THEN MUST skip
- Test: `linq_parse_skips_unauthorized` in `src/channels/linq.rs`

##### Scenario: LinQ parse skips empty text

- WHEN message has empty text
- THEN MUST skip
- Test: `linq_parse_skips_empty_text` in `src/channels/linq.rs`

##### Scenario: LinQ parse extracts sender

- WHEN parsing a message
- THEN MUST extract the sender
- Test: `linq_parse_extracts_sender` in `src/channels/linq.rs`

##### Scenario: LinQ parse extracts timestamp

- WHEN parsing a message
- THEN MUST extract the timestamp
- Test: `linq_parse_extracts_timestamp` in `src/channels/linq.rs`

##### Scenario: LinQ config serde

- WHEN config is serialized/deserialized
- THEN MUST round-trip correctly
- Test: `linq_config_serde` in `src/channels/linq.rs`

##### Scenario: LinQ config defaults

- WHEN config uses defaults
- THEN MUST have sensible defaults
- Test: `linq_config_defaults` in `src/channels/linq.rs`

##### Scenario: LinQ health check disconnected

- WHEN channel is not connected
- THEN health_check MUST return false
- Test: `linq_health_check_disconnected` in `src/channels/linq.rs`

##### Scenario: LinQ message ID format

- WHEN a LinQ message ID is generated
- THEN MUST have correct format
- Test: `linq_message_id_format` in `src/channels/linq.rs`

##### Scenario: LinQ message ID deterministic

- WHEN same inputs are used
- THEN MUST produce same ID
- Test: `linq_message_id_deterministic` in `src/channels/linq.rs`

##### Scenario: LinQ parse handles missing fields

- WHEN webhook has missing fields
- THEN MUST handle gracefully
- Test: `linq_parse_handles_missing_fields` in `src/channels/linq.rs`

##### Scenario: LinQ parse group message

- WHEN a group message is received
- THEN MUST parse correctly
- Test: `linq_parse_group_message` in `src/channels/linq.rs`

##### Scenario: LinQ duplicate detection

- WHEN same message is received twice
- THEN MUST detect duplicate
- Test: `linq_duplicate_detection` in `src/channels/linq.rs`

##### Scenario: LinQ allowlist exact match not substring

- WHEN user is a substring of allowlisted user
- THEN MUST not match
- Test: `linq_exact_match_not_substring` in `src/channels/linq.rs`

##### Scenario: LinQ allowlist case sensitive

- WHEN user case differs
- THEN MUST not match
- Test: `linq_allowlist_case_sensitive` in `src/channels/linq.rs`

##### Scenario: LinQ wildcard with specific

- WHEN allowlist has wildcard and specific
- THEN wildcard MUST take precedence
- Test: `linq_wildcard_with_specific` in `src/channels/linq.rs`

##### Scenario: LinQ parse image message

- WHEN image message is received
- THEN MUST parse with image marker
- Test: `linq_parse_image_message` in `src/channels/linq.rs`

##### Scenario: LinQ parse audio message

- WHEN audio message is received
- THEN MUST parse correctly
- Test: `linq_parse_audio_message` in `src/channels/linq.rs`

##### Scenario: LinQ normalize incoming content

- WHEN incoming content needs normalization
- THEN MUST normalize
- Test: `linq_normalize_incoming_content` in `src/channels/linq.rs`

#### REQ-CHAN-017-ACP: ACP Channel (19 tests)

##### Scenario: ACP channel name

- WHEN name() is called
- THEN MUST return "acp"
- Test: `acp_channel_name` in `src/channels/acp.rs`

##### Scenario: ACP user allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all
- Test: `acp_user_allowed_wildcard` in `src/channels/acp.rs`

##### Scenario: ACP user allowed specific

- WHEN allowlist contains specific user
- THEN MUST allow
- Test: `acp_user_allowed_specific` in `src/channels/acp.rs`

##### Scenario: ACP user denied empty

- WHEN allowlist is empty
- THEN MUST deny all
- Test: `acp_user_denied_empty` in `src/channels/acp.rs`

##### Scenario: ACP JSON-RPC parse valid request

- WHEN a valid JSON-RPC request is received
- THEN MUST parse it
- Test: `acp_jsonrpc_parse_valid` in `src/channels/acp.rs`

##### Scenario: ACP JSON-RPC parse invalid request

- WHEN an invalid JSON-RPC request is received
- THEN MUST reject
- Test: `acp_jsonrpc_parse_invalid` in `src/channels/acp.rs`

##### Scenario: ACP JSON-RPC response success

- WHEN building a success response
- THEN MUST have correct JSON-RPC shape
- Test: `acp_jsonrpc_response_success` in `src/channels/acp.rs`

##### Scenario: ACP JSON-RPC response error

- WHEN building an error response
- THEN MUST have correct JSON-RPC error shape
- Test: `acp_jsonrpc_response_error` in `src/channels/acp.rs`

##### Scenario: ACP config serde

- WHEN config is serialized/deserialized
- THEN MUST round-trip correctly
- Test: `acp_config_serde` in `src/channels/acp.rs`

##### Scenario: ACP config defaults

- WHEN config uses defaults
- THEN MUST have sensible defaults
- Test: `acp_config_defaults` in `src/channels/acp.rs`

##### Scenario: ACP health check

- WHEN health check is called
- THEN MUST return correct status
- Test: `acp_health_check` in `src/channels/acp.rs`

##### Scenario: ACP message ID format

- WHEN an ACP message ID is generated
- THEN MUST have correct format
- Test: `acp_message_id_format` in `src/channels/acp.rs`

##### Scenario: ACP parse extracts sender

- WHEN parsing an ACP message
- THEN MUST extract sender
- Test: `acp_parse_extracts_sender` in `src/channels/acp.rs`

##### Scenario: ACP parse extracts content

- WHEN parsing an ACP message
- THEN MUST extract content
- Test: `acp_parse_extracts_content` in `src/channels/acp.rs`

##### Scenario: ACP parse handles missing fields

- WHEN ACP message has missing fields
- THEN MUST handle gracefully
- Test: `acp_parse_handles_missing_fields` in `src/channels/acp.rs`

##### Scenario: ACP allowlist exact match

- WHEN user matches exactly
- THEN MUST allow
- Test: `acp_allowlist_exact_match` in `src/channels/acp.rs`

##### Scenario: ACP allowlist case sensitive

- WHEN user case differs
- THEN MUST not match
- Test: `acp_allowlist_case_sensitive` in `src/channels/acp.rs`

##### Scenario: ACP duplicate detection

- WHEN same message is received twice
- THEN MUST detect duplicate
- Test: `acp_duplicate_detection` in `src/channels/acp.rs`

##### Scenario: ACP JSON-RPC batch not supported

- WHEN a batch JSON-RPC request is received
- THEN MUST reject (not supported)
- Test: `acp_jsonrpc_batch_not_supported` in `src/channels/acp.rs`

#### REQ-CHAN-017-CLI: CLI Channel (6 tests)

##### Scenario: CLI channel name

- WHEN name() is called
- THEN MUST return "cli"
- Test: `cli_channel_name` in `src/channels/cli.rs`

##### Scenario: CLI send does not panic

- WHEN send() is called
- THEN MUST not panic
- Test: `cli_channel_send_does_not_panic` in `src/channels/cli.rs`

##### Scenario: CLI send empty message

- WHEN sending an empty message
- THEN MUST handle gracefully
- Test: `cli_channel_send_empty_message` in `src/channels/cli.rs`

##### Scenario: CLI health check

- WHEN health_check is called
- THEN MUST return true
- Test: `cli_channel_health_check` in `src/channels/cli.rs`

##### Scenario: Channel message struct construction

- WHEN ChannelMessage is constructed for CLI
- THEN MUST have correct fields
- Test: `channel_message_struct` in `src/channels/cli.rs`

##### Scenario: Channel message clone

- WHEN ChannelMessage is cloned
- THEN MUST preserve all fields
- Test: `channel_message_clone` in `src/channels/cli.rs`

#### REQ-CHAN-017-CLAWDTALK: ClawdTalk Channel (5 tests)

##### Scenario: Creates channel

- WHEN ClawdTalk channel is created
- THEN MUST initialize correctly
- Test: `creates_channel` in `src/channels/clawdtalk.rs`

##### Scenario: Destination allowed exact match

- WHEN allowlist contains exact destination
- THEN MUST allow
- Test: `destination_allowed_exact_match` in `src/channels/clawdtalk.rs`

##### Scenario: Destination allowed wildcard

- WHEN allowlist contains wildcard
- THEN MUST allow all
- Test: `destination_allowed_wildcard` in `src/channels/clawdtalk.rs`

##### Scenario: Destination allowed empty means all

- WHEN allowlist is empty
- THEN MUST allow all (ClawdTalk-specific)
- Test: `destination_allowed_empty_means_all` in `src/channels/clawdtalk.rs`

##### Scenario: Webhook event deserializes

- WHEN a webhook event is received
- THEN MUST deserialize correctly
- Test: `webhook_event_deserializes` in `src/channels/clawdtalk.rs`

### REQ-CHAN-018: Acknowledgement Reactions

`AckReactionContext` MUST track message acknowledgement state with emoji reactions, support policy-based rules, and respect sampling rates.

#### Scenario: Disabled policy returns None

- WHEN ack reaction policy is disabled
- THEN MUST return None
- Test: `disabled_policy_returns_none` in `src/channels/ack_reaction.rs`

#### Scenario: Falls back to defaults when no override

- WHEN no rule override matches
- THEN MUST fall back to defaults
- Test: `falls_back_to_defaults_when_no_override` in `src/channels/ack_reaction.rs`

#### Scenario: First strategy uses first emoji

- WHEN "first" strategy is active
- THEN MUST use the first emoji in the pool
- Test: `first_strategy_uses_first_emoji` in `src/channels/ack_reaction.rs`

#### Scenario: Rule matches chat type and keyword

- WHEN a rule matches by chat type and keyword
- THEN MUST apply the rule
- Test: `rule_matches_chat_type_and_keyword` in `src/channels/ack_reaction.rs`

#### Scenario: Rule respects sender and locale filters

- WHEN a rule has sender and locale filters
- THEN MUST respect both
- Test: `rule_respects_sender_and_locale_filters` in `src/channels/ack_reaction.rs`

#### Scenario: Rule respects chat_id filter

- WHEN a rule has a chat_id filter
- THEN MUST only apply to matching chat
- Test: `rule_respects_chat_id_filter` in `src/channels/ack_reaction.rs`

#### Scenario: Rule can suppress reaction

- WHEN a rule suppresses reactions
- THEN MUST return None
- Test: `rule_can_suppress_reaction` in `src/channels/ack_reaction.rs`

#### Scenario: Contains-none blocks keyword match

- WHEN contains_none filter blocks a keyword
- THEN MUST not match
- Test: `contains_none_blocks_keyword_match` in `src/channels/ack_reaction.rs`

#### Scenario: Regex filters are supported

- WHEN a rule uses regex filters
- THEN MUST match by regex
- Test: `regex_filters_are_supported` in `src/channels/ack_reaction.rs`

#### Scenario: Sample rate zero disables fallback reaction

- WHEN sample_rate is zero
- THEN MUST disable fallback reaction
- Test: `sample_rate_zero_disables_fallback_reaction` in `src/channels/ack_reaction.rs`

### REQ-CHAN-019: Transcription Support

Transcription functions MUST support Groq Whisper API for audio transcription, validate inputs, and normalize filenames.

#### Scenario: Rejects oversized audio

- WHEN audio data exceeds size limit
- THEN MUST reject
- Test: `rejects_oversized_audio` in `src/channels/transcription.rs`

#### Scenario: Rejects missing API key

- WHEN Groq API key is missing
- THEN MUST reject
- Test: `rejects_missing_api_key` in `src/channels/transcription.rs`

#### Scenario: Uses config API key without GROQ env

- WHEN config has API key but GROQ env is unset
- THEN MUST use config key
- Test: `uses_config_api_key_without_groq_env` in `src/channels/transcription.rs`

#### Scenario: MIME for audio maps accepted formats

- WHEN audio format is accepted
- THEN MUST map to correct MIME type
- Test: `mime_for_audio_maps_accepted_formats` in `src/channels/transcription.rs`

#### Scenario: MIME for audio is case insensitive

- WHEN audio format has mixed case
- THEN MUST still map correctly
- Test: `mime_for_audio_case_insensitive` in `src/channels/transcription.rs`

#### Scenario: MIME for audio rejects unknown

- WHEN audio format is unknown
- THEN MUST reject
- Test: `mime_for_audio_rejects_unknown` in `src/channels/transcription.rs`

#### Scenario: Normalize audio filename rewrites oga

- WHEN filename has .oga extension
- THEN MUST rewrite to accepted extension
- Test: `normalize_audio_filename_rewrites_oga` in `src/channels/transcription.rs`

#### Scenario: Normalize audio filename preserves accepted

- WHEN filename has accepted extension
- THEN MUST preserve it
- Test: `normalize_audio_filename_preserves_accepted` in `src/channels/transcription.rs`

#### Scenario: Normalize audio filename no extension

- WHEN filename has no extension
- THEN MUST handle gracefully
- Test: `normalize_audio_filename_no_extension` in `src/channels/transcription.rs`

#### Scenario: Rejects unsupported audio format

- WHEN audio format is unsupported
- THEN MUST reject
- Test: `rejects_unsupported_audio_format` in `src/channels/transcription.rs`

## Mock Strategy

- HTTP APIs: `wiremock::MockServer` for platform API simulation
- WebSocket channels: Test config parsing and message formatting; skip live connection tests
- SQLite stores: `tempfile::TempDir` for database isolation
- Channel trait: Direct construction with test config defaults

## Coverage Notes

| Source File | Test Count | Coverage Status |
|---|---|---|
| `src/channels/mod.rs` | 120 | Comprehensive: orchestration, dispatch, prompt, memory, approval, progress, identity, health |
| `src/channels/traits.rs` | 10 | Full: trait contract, builders, defaults, reactions, drafts, approval prompt |
| `src/channels/telegram.rs` | 172 | Comprehensive: send, listen, typing, reactions, attachments, voice, mention-only, splitting |
| `src/channels/discord.rs` | 70 | Comprehensive: send, listen, typing, reactions, attachments, mention-only, splitting |
| `src/channels/slack.rs` | 40 | Good: send, listen, threading, mention-only, display name caching, retry logic |
| `src/channels/whatsapp.rs` | 47 | Good: parse, allowlist, send shape, config, attachments, transcription |
| `src/channels/whatsapp_web.rs` | 17 | Good: parse, allowlist, normalize, QR, transcription |
| `src/channels/wati.rs` | 19 | Good: parse, allowlist, tenant routing, timestamps |
| `src/channels/whatsapp_storage.rs` | 3 | Minimal: schema creation, LID mapping, token cleanup |
| `src/channels/matrix.rs` | 38 | Good: parse sync, allowlist, event cache, mention-only, config |
| `src/channels/irc.rs` | 37 | Good: parse, allowlist, SASL, splitting, config, mention-only |
| `src/channels/email_channel.rs` | 40 | Good: sender allowlist, HTML stripping, config serde, IMAP/SMTP defaults |
| `src/channels/signal.rs` | 32 | Good: envelope processing, recipient parsing, SSE, config, allowlist |
| `src/channels/nostr.rs` | 11 | Good: key validation, allowlist, protocol routing, health |
| `src/channels/lark.rs` | 49 | Comprehensive: parse, mention-only, token refresh, locale detection, config |
| `src/channels/imessage.rs` | 58 | Comprehensive: target validation, AppleScript escaping, SQLite fetch, allowlist |
| `src/channels/bluebubbles.rs` | 41 | Good: parse, allowlist, attributed body formatting, ignore sender |
| `src/channels/mattermost.rs` | 35 | Good: parse, mention-only, normalize, thread handling |
| `src/channels/mqtt.rs` | 12 | Good: config validation, TLS, broker URL parsing |
| `src/channels/github.rs` | 9 | Good: signature verification, event parsing, allowlist |
| `src/channels/nextcloud_talk.rs` | 11 | Good: parse, signature verification, allowlist |
| `src/channels/dingtalk.rs` | 14 | Good: config, text extraction, stream data parsing |
| `src/channels/qq.rs` | 21 | Good: webhook, parse, compose, config, allowlist |
| `src/channels/linq.rs` | 25 | Good: signature, parse, config, allowlist, media |
| `src/channels/napcat.rs` | 22 | Good: API derivation, OneBot composition, parse, allowlist |
| `src/channels/acp.rs` | 19 | Good: JSON-RPC, config, allowlist, health |
| `src/channels/cli.rs` | 6 | Adequate: name, send, health, message struct |
| `src/channels/clawdtalk.rs` | 5 | Adequate: creation, destination allowlist, webhook |
| `src/channels/ack_reaction.rs` | 10 | Full: policy, rules, strategy, sampling, filters |
| `src/channels/transcription.rs` | 10 | Full: validation, MIME mapping, filename normalization |
| **Total** | **1003** | |
