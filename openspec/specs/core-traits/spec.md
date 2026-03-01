# Core Traits Specification

## Purpose
Define the behavioral contracts for all extension-point traits in ZeroClaw: Provider, Channel, Tool, Memory, Observer, RuntimeAdapter, Peripheral, Sandbox, HookHandler, and Plugin.

## Scope
- Files: `src/providers/traits.rs`, `src/channels/traits.rs`, `src/tools/traits.rs`, `src/memory/traits.rs`, `src/observability/traits.rs`, `src/runtime/traits.rs`, `src/peripherals/traits.rs`, `src/security/traits.rs`, `src/hooks/traits.rs`, `src/plugins/traits.rs`
- Risk tier: HIGH (these are the foundation all implementations depend on)

## Requirements

---

### Provider Trait (`src/providers/traits.rs`)

### REQ-PROV-001: ChatMessage constructors MUST produce correct roles
ChatMessage::system(), ::user(), ::assistant(), ::tool() MUST set the correct role string.

#### Scenario: All four role constructors
- WHEN each constructor is called with content
- THEN the role field matches the corresponding ROLE_* constant and content is preserved
- Test: `chat_message_constructors` in `src/providers/traits.rs`

### REQ-PROV-002: ChatResponse helpers MUST report tool call presence and text content
has_tool_calls() MUST return true iff tool_calls is non-empty. text_or_empty() MUST return text or empty string.

#### Scenario: Empty response
- WHEN ChatResponse has no text and no tool_calls
- THEN has_tool_calls() returns false and text_or_empty() returns ""
- Test: `chat_response_helpers` in `src/providers/traits.rs`

#### Scenario: Response with tool calls
- WHEN ChatResponse has text and tool_calls
- THEN has_tool_calls() returns true and text_or_empty() returns the text
- Test: `chat_response_helpers` in `src/providers/traits.rs`

### REQ-PROV-003: TokenUsage default MUST be all-None
Default TokenUsage MUST have input_tokens and output_tokens as None.

#### Scenario: Default construction
- WHEN TokenUsage::default() is called
- THEN both token fields are None
- Test: `token_usage_default_is_none` in `src/providers/traits.rs`

### REQ-PROV-004: ProviderCapabilities default MUST be all-false
Default capabilities MUST have native_tool_calling and vision as false.

#### Scenario: Default capabilities
- WHEN ProviderCapabilities::default() is called
- THEN native_tool_calling is false and vision is false
- Test: `provider_capabilities_default` in `src/providers/traits.rs`

#### Scenario: Capabilities equality
- WHEN two ProviderCapabilities with same fields are compared
- THEN they are equal; different fields are not equal
- Test: `provider_capabilities_equality` in `src/providers/traits.rs`

### REQ-PROV-005: Provider default convert_tools MUST return PromptGuided
When a provider does not override convert_tools, it MUST return ToolsPayload::PromptGuided.

#### Scenario: Default convert_tools
- WHEN convert_tools is called on a provider that doesn't override it
- THEN it returns PromptGuided with tool instructions text
- Test: `provider_convert_tools_default` in `src/providers/traits.rs`

### REQ-PROV-006: Provider.chat() MUST inject tools into system prompt for non-native providers
When tools are provided but the provider doesn't support native tools, chat() MUST inject tool instructions into the system prompt.

#### Scenario: Prompt-guided fallback with existing system message
- WHEN chat() is called with tools on a non-native provider that has a system message
- THEN tool instructions are appended to the existing system message
- Test: `provider_chat_prompt_guided_preserves_existing_system_not_first` in `src/providers/traits.rs`

#### Scenario: Prompt-guided fallback without system message
- WHEN chat() is called with tools on a non-native provider with no system message
- THEN a new system message is prepended with tool instructions
- Test: `provider_chat_prompt_guided_fallback` in `src/providers/traits.rs`

#### Scenario: Non-prompt-guided payload rejected
- WHEN convert_tools returns non-PromptGuided but supports_native_tools is false
- THEN chat() MUST return an error
- Test: `provider_chat_prompt_guided_rejects_non_prompt_payload` in `src/providers/traits.rs`

### REQ-PROV-007: build_tool_instructions_text MUST format tool protocol and listings
The function MUST include the XML tool call protocol description and list all tools with their parameters.

#### Scenario: Multiple tools
- WHEN called with two tools
- THEN output contains protocol, both tool names, descriptions, and parameters
- Test: `build_tool_instructions_text_format` in `src/providers/traits.rs`

#### Scenario: Empty tools
- WHEN called with empty slice
- THEN output still contains protocol header and "Available Tools" section
- Test: `build_tool_instructions_text_empty` in `src/providers/traits.rs`

### REQ-PROV-008: ToolCall and ConversationMessage MUST serialize/deserialize correctly

#### Scenario: ToolCall serialization
- WHEN a ToolCall is serialized to JSON
- THEN id, name, and arguments are present
- Test: `tool_call_serialization` in `src/providers/traits.rs`

#### Scenario: ConversationMessage variants
- WHEN Chat and ToolResults variants are serialized
- THEN they include the correct "type" tag
- Test: `conversation_message_variants` in `src/providers/traits.rs`

### REQ-PROV-009: ToolsPayload variants MUST be constructible and matchable

#### Scenario: All four variants
- WHEN Gemini, Anthropic, OpenAI, and PromptGuided variants are constructed
- THEN they match their respective patterns
- Test: `tools_payload_variants` in `src/providers/traits.rs`

### REQ-PROV-010: StreamChunk constructors MUST produce correct fields

#### Scenario: delta() creates non-final chunk
- WHEN StreamChunk::delta("hello") is called
- THEN delta is "hello", is_final is false, token_count is 0
- Test: `stream_chunk_delta_creates_non_final` in `src/providers/traits.rs` (NEW)

#### Scenario: final_chunk() creates empty final
- WHEN StreamChunk::final_chunk() is called
- THEN delta is empty, is_final is true
- Test: `stream_chunk_final_creates_final` in `src/providers/traits.rs` (NEW)

#### Scenario: error() creates final with message
- WHEN StreamChunk::error("boom") is called
- THEN delta is "boom", is_final is true
- Test: `stream_chunk_error_creates_final_with_message` in `src/providers/traits.rs` (NEW)

#### Scenario: with_token_estimate calculates tokens
- WHEN with_token_estimate() is called on a chunk with 8-char delta
- THEN token_count is 2 (~4 chars per token)
- Test: `stream_chunk_with_token_estimate` in `src/providers/traits.rs` (NEW)

### REQ-PROV-011: StreamOptions MUST support builder pattern

#### Scenario: Default and builder
- WHEN StreamOptions::new(true).with_token_count() is called
- THEN enabled is true and count_tokens is true
- Test: `stream_options_builder` in `src/providers/traits.rs` (NEW)

### REQ-PROV-012: is_user_or_assistant_role MUST correctly classify roles

#### Scenario: Role classification
- WHEN called with "user", "assistant", "system", "tool"
- THEN returns true for user/assistant, false for system/tool
- Test: `is_user_or_assistant_role_classifies_correctly` in `src/providers/traits.rs` (NEW)

### REQ-PROV-013: Provider default warmup MUST succeed

#### Scenario: Default warmup is no-op
- WHEN warmup() is called on default implementation
- THEN it returns Ok(())
- Test: `provider_default_warmup_succeeds` in `src/providers/traits.rs` (NEW)

### REQ-PROV-014: Provider default supports_streaming MUST return false

#### Scenario: Default streaming support
- WHEN supports_streaming() is called on default implementation
- THEN it returns false
- Test: `provider_default_supports_streaming_is_false` in `src/providers/traits.rs` (NEW)

---

### Channel Trait (`src/channels/traits.rs`)

### REQ-CHAN-001: ChannelMessage clone MUST preserve all fields

#### Scenario: Clone preserves fields
- WHEN a ChannelMessage is cloned
- THEN all fields match the original
- Test: `channel_message_clone_preserves_fields` in `src/channels/traits.rs`

### REQ-CHAN-002: Channel default methods MUST return success

#### Scenario: health_check, typing, send
- WHEN default implementations are called
- THEN health_check returns true, start_typing/stop_typing return Ok
- Test: `default_trait_methods_return_success` in `src/channels/traits.rs`

### REQ-CHAN-003: Channel default reaction methods MUST return Ok

#### Scenario: add_reaction, remove_reaction
- WHEN default implementations are called
- THEN they return Ok(())
- Test: `default_reaction_methods_return_success` in `src/channels/traits.rs`

### REQ-CHAN-004: Channel default draft methods MUST return expected defaults

#### Scenario: Draft support defaults
- WHEN default implementations are called
- THEN supports_draft_updates returns false, send_draft returns Ok(None), etc.
- Test: `default_draft_methods_return_success` in `src/channels/traits.rs`

### REQ-CHAN-005: Channel listen MUST send messages through the provided sender

#### Scenario: Listen delivers message
- WHEN listen() is called with a sender
- THEN a message is received on the channel
- Test: `listen_sends_message_to_channel` in `src/channels/traits.rs`

### REQ-CHAN-006: Approval prompt MUST handle UTF-8 truncation safely

#### Scenario: Long multibyte args truncated safely
- WHEN args exceed 220 bytes with multibyte UTF-8
- THEN truncation happens at a valid char boundary
- Test: `approval_prompt_truncates_safely_for_multibyte_utf8` in `src/channels/traits.rs`

#### Scenario: Short args not truncated
- WHEN args are under 220 bytes
- THEN they are included verbatim
- Test: `approval_prompt_short_args_not_truncated` in `src/channels/traits.rs`

### REQ-CHAN-007: SendMessage constructors MUST build correctly

#### Scenario: SendMessage::new
- WHEN SendMessage::new("hello", "bob") is called
- THEN content, recipient are set, subject and thread_ts are None
- Test: `send_message_new_sets_fields` in `src/channels/traits.rs` (NEW)

#### Scenario: SendMessage::with_subject
- WHEN SendMessage::with_subject("hello", "bob", "re: test") is called
- THEN content, recipient, subject are all set
- Test: `send_message_with_subject_sets_all_fields` in `src/channels/traits.rs` (NEW)

#### Scenario: SendMessage::in_thread
- WHEN .in_thread(Some("ts123")) is called
- THEN thread_ts is set
- Test: `send_message_in_thread_sets_thread_ts` in `src/channels/traits.rs` (NEW)

---

### Tool Trait (`src/tools/traits.rs`)

### REQ-TOOL-001: Tool.spec() MUST compose from name, description, parameters_schema

#### Scenario: Spec composition
- WHEN spec() is called on a tool
- THEN name, description, parameters match the tool's methods
- Test: `spec_uses_tool_metadata_and_schema` in `src/tools/traits.rs`

### REQ-TOOL-002: Tool.execute() MUST return ToolResult

#### Scenario: Successful execution
- WHEN execute is called with valid args
- THEN returns ToolResult with success=true
- Test: `execute_returns_expected_output` in `src/tools/traits.rs`

### REQ-TOOL-003: ToolResult MUST serialize/deserialize correctly

#### Scenario: Roundtrip with error
- WHEN a ToolResult with error is serialized and deserialized
- THEN all fields are preserved
- Test: `tool_result_serialization_roundtrip` in `src/tools/traits.rs`

### REQ-TOOL-004: ToolSpec MUST serialize/deserialize correctly

#### Scenario: ToolSpec roundtrip
- WHEN a ToolSpec is serialized and deserialized
- THEN name, description, parameters are preserved
- Test: `tool_spec_serialization_roundtrip` in `src/tools/traits.rs` (NEW)

### REQ-TOOL-005: ToolResult success field MUST be independent of error field

#### Scenario: Success with no error
- WHEN ToolResult has success=true and error=None
- THEN serialization reflects both fields
- Test: `tool_result_success_without_error` in `src/tools/traits.rs` (NEW)

---

### Memory Trait (`src/memory/traits.rs`)

### REQ-MEM-001: MemoryCategory display MUST output snake_case

#### Scenario: Display variants
- WHEN Core, Daily, Conversation, Custom("x") are displayed
- THEN they output "core", "daily", "conversation", "x"
- Test: `memory_category_display_outputs_expected_values` in `src/memory/traits.rs`

### REQ-MEM-002: MemoryCategory serde MUST use snake_case

#### Scenario: JSON serialization
- WHEN categories are serialized
- THEN they produce snake_case strings
- Test: `memory_category_serde_uses_snake_case` in `src/memory/traits.rs`

### REQ-MEM-003: MemoryEntry MUST roundtrip through JSON preserving optional fields

#### Scenario: Full entry roundtrip
- WHEN entry with session_id and score is serialized then deserialized
- THEN all fields including optionals are preserved
- Test: `memory_entry_roundtrip_preserves_optional_fields` in `src/memory/traits.rs`

### REQ-MEM-004: Memory.reindex() default MUST bail with "not supported"

#### Scenario: Default reindex
- WHEN reindex() is called on a backend that doesn't override it
- THEN it returns an error containing "not supported"
- Test: `memory_reindex_default_bails` in `src/memory/traits.rs` (NEW)

### REQ-MEM-005: MemoryCategory Custom variant MUST preserve custom name

#### Scenario: Custom category roundtrip
- WHEN Custom("project_notes") is serialized and deserialized
- THEN it produces Custom("project_notes")
- Test: `memory_category_custom_serde_roundtrip` in `src/memory/traits.rs` (NEW)

---

### Observer Trait (`src/observability/traits.rs`)

### REQ-OBS-001: Observer MUST record events and metrics through its methods

#### Scenario: Recording events and metrics
- WHEN events and metrics are recorded
- THEN the observer's counters increment
- Test: `observer_records_events_and_metrics` in `src/observability/traits.rs`

### REQ-OBS-002: Observer default flush MUST be a no-op, and as_any MUST downcast

#### Scenario: Default flush and as_any
- WHEN flush() is called and as_any() is downcast
- THEN flush completes without error and downcast succeeds
- Test: `observer_default_flush_and_as_any_work` in `src/observability/traits.rs`

### REQ-OBS-003: ObserverEvent and ObserverMetric MUST be Clone

#### Scenario: Clone
- WHEN event and metric variants are cloned
- THEN clones match original pattern
- Test: `observer_event_and_metric_are_cloneable` in `src/observability/traits.rs`

---

### RuntimeAdapter Trait (`src/runtime/traits.rs`)

### REQ-RT-001: RuntimeAdapter default memory_budget MUST return 0

#### Scenario: Default memory budget
- WHEN memory_budget() is called
- THEN it returns 0
- Test: `default_memory_budget_is_zero` in `src/runtime/traits.rs`

### REQ-RT-002: RuntimeAdapter MUST report capabilities correctly

#### Scenario: Capability queries
- WHEN name, has_shell_access, has_filesystem_access, supports_long_running, storage_path are queried
- THEN they return the implementor's declared values
- Test: `runtime_reports_capabilities` in `src/runtime/traits.rs`

### REQ-RT-003: build_shell_command MUST produce an executable command

#### Scenario: Command execution
- WHEN build_shell_command is called and the command is run
- THEN it executes successfully and output contains the command string
- Test: `build_shell_command_executes` in `src/runtime/traits.rs`

---

### Peripheral Trait (`src/peripherals/traits.rs`)

### REQ-PERIPH-001: Peripheral MUST implement full lifecycle (connect, disconnect, health_check, tools)
This is a marker requirement — each implementation must satisfy the trait contract.

#### Scenario: Mock peripheral lifecycle
- WHEN a mock peripheral connects, checks health, lists tools, then disconnects
- THEN connect succeeds, health_check returns true when connected, tools() returns registered tools, disconnect succeeds, health_check returns false after disconnect
- Test: `peripheral_lifecycle_connect_check_disconnect` in `src/peripherals/traits.rs` (NEW)

### REQ-PERIPH-002: Peripheral MUST expose name and board_type

#### Scenario: Peripheral identity
- WHEN name() and board_type() are called
- THEN they return the configured values
- Test: `peripheral_name_and_board_type` in `src/peripherals/traits.rs` (NEW)

---

### Sandbox Trait (`src/security/traits.rs`)

### REQ-SAND-001: NoopSandbox MUST always be available

#### Scenario: Availability
- WHEN is_available() is called
- THEN it returns true
- Test: `noop_sandbox_is_always_available` in `src/security/traits.rs`

### REQ-SAND-002: NoopSandbox.wrap_command MUST not modify the command

#### Scenario: Pass-through
- WHEN wrap_command is called with a command
- THEN program and args are unchanged
- Test: `noop_sandbox_wrap_command_is_noop` in `src/security/traits.rs`

### REQ-SAND-003: NoopSandbox name MUST be "none"

#### Scenario: Name
- WHEN name() is called
- THEN it returns "none"
- Test: `noop_sandbox_name` in `src/security/traits.rs`

### REQ-SAND-004: NoopSandbox description MUST describe no sandboxing

#### Scenario: Description
- WHEN description() is called
- THEN it returns a string indicating no sandboxing
- Test: `noop_sandbox_description` in `src/security/traits.rs` (NEW)

---

### HookHandler Trait (`src/hooks/traits.rs`)

### REQ-HOOK-001: HookResult::is_cancel MUST correctly classify variants

#### Scenario: Continue vs Cancel
- WHEN Continue and Cancel variants are checked
- THEN is_cancel returns false for Continue and true for Cancel
- Test: `hook_result_is_cancel` in `src/hooks/traits.rs`

### REQ-HOOK-002: Default priority MUST be 0

#### Scenario: Default priority
- WHEN priority() is called on a minimal implementation
- THEN it returns 0
- Test: `default_priority_is_zero` in `src/hooks/traits.rs`

### REQ-HOOK-003: Default modifying hooks MUST pass through unchanged

#### Scenario: before_tool_call pass-through
- WHEN before_tool_call is called on default implementation
- THEN it returns Continue with original values
- Test: `default_modifying_hooks_pass_through` in `src/hooks/traits.rs`

### REQ-HOOK-004: Default capabilities MUST be empty

#### Scenario: Empty capabilities
- WHEN capabilities() is called on default implementation
- THEN it returns an empty slice
- Test: `default_hook_capabilities_empty` in `src/hooks/traits.rs` (NEW)

### REQ-HOOK-005: Default void hooks MUST complete without error

#### Scenario: Void hooks no-op
- WHEN on_gateway_start, on_session_start, on_heartbeat_tick are called
- THEN they complete without error
- Test: `default_void_hooks_complete_without_error` in `src/hooks/traits.rs` (NEW)

---

### Plugin Trait (`src/plugins/traits.rs`)

### REQ-PLUG-001: PluginApi MUST collect tools and hooks

#### Scenario: Empty registration
- WHEN a plugin registers without adding tools or hooks
- THEN api.tools and api.hooks remain empty
- Test: `plugin_api_collects_nothing_by_default` in `src/plugins/traits.rs`

### REQ-PLUG-002: PluginApi MUST accumulate registered tools and hooks

#### Scenario: Tool and hook registration
- WHEN register_tool and register_hook are called
- THEN api.tools and api.hooks grow by one each
- Test: `plugin_api_accumulates_tools_and_hooks` in `src/plugins/traits.rs` (NEW)

### REQ-PLUG-003: PluginCapability variants MUST serialize/deserialize correctly

#### Scenario: Capability serde
- WHEN PluginCapability variants are serialized and deserialized
- THEN they roundtrip correctly
- Test: `plugin_capability_serde_roundtrip` in `src/plugins/traits.rs` (NEW)

### REQ-PLUG-004: PluginLogger MUST format with plugin prefix

#### Scenario: Logger prefix
- WHEN PluginLogger::new("my-plugin") is created
- THEN its prefix contains "[plugin:my-plugin]"
- Test: `plugin_logger_has_correct_prefix` in `src/plugins/traits.rs` (NEW)

---

## Mock Strategy
All traits are tested using manual mock structs implementing the trait. No external dependencies needed — all tests are pure unit tests using in-memory data.
