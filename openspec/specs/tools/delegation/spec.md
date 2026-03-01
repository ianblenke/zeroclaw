# Tools Delegation Specification

## Purpose

Define requirements for the delegation and sub-agent tool implementations: delegate, delegate coordination status, sub-agent spawn/manage/list/registry, and MCP tool wrapper.

## Scope

- Files: `src/tools/delegate.rs` (27 tests), `src/tools/delegate_coordination_status.rs` (6 tests), `src/tools/subagent_spawn.rs` (13 tests), `src/tools/subagent_manage.rs` (17 tests), `src/tools/subagent_list.rs` (9 tests), `src/tools/subagent_registry.rs` (18 tests), `src/tools/mcp_tool.rs` (5 tests)
- Total tests: 95
- Risk tier: HIGH (delegates execute with agent authority; MCP tools dispatch external tool calls)

## Requirements

---

### Delegate Tool (`src/tools/delegate.rs`)

### REQ-DELEG-001: Delegate Tool Identity

`DelegateTool` MUST implement the `Tool` trait with name `"delegate"`, non-empty description, and a valid JSON parameter schema listing configured agent names.

#### Scenario: Name and schema
- WHEN `name()`, `description()`, and `parameters_schema()` are called
- THEN MUST return `"delegate"`, a non-empty description, and a schema listing agent names
- Test: `name_and_schema` in `src/tools/delegate.rs`
- Test: `description_not_empty` in `src/tools/delegate.rs`
- Test: `schema_lists_agent_names` in `src/tools/delegate.rs`

#### Scenario: Empty agents schema
- WHEN no agents are configured
- THEN MUST return a schema with empty agent enum
- Test: `empty_agents_schema` in `src/tools/delegate.rs`

### REQ-DELEG-002: Delegate Parameter Validation

`DelegateTool` MUST validate required parameters (agent, prompt) and reject blank/missing values.

#### Scenario: Missing agent parameter
- WHEN the agent parameter is missing
- THEN MUST return error
- Test: `missing_agent_param` in `src/tools/delegate.rs`

#### Scenario: Missing prompt parameter
- WHEN the prompt parameter is missing
- THEN MUST return error
- Test: `missing_prompt_param` in `src/tools/delegate.rs`

#### Scenario: Blank agent rejected
- WHEN the agent parameter is blank/whitespace
- THEN MUST reject after trimming
- Test: `blank_agent_rejected` in `src/tools/delegate.rs`

#### Scenario: Blank prompt rejected
- WHEN the prompt parameter is blank/whitespace
- THEN MUST reject after trimming
- Test: `blank_prompt_rejected` in `src/tools/delegate.rs`

#### Scenario: Whitespace agent name trimmed and found
- WHEN the agent parameter has leading/trailing whitespace
- THEN MUST trim and resolve the agent
- Test: `whitespace_agent_name_trimmed_and_found` in `src/tools/delegate.rs`

### REQ-DELEG-003: Delegate Agent Resolution

`DelegateTool` MUST resolve agent names to configured agents and error on unknown agents or invalid providers.

#### Scenario: Unknown agent
- WHEN an unknown agent name is provided
- THEN MUST return an error
- Test: `unknown_agent_returns_error` in `src/tools/delegate.rs`

#### Scenario: Invalid provider
- WHEN agent config references an invalid provider
- THEN MUST return an error
- Test: `invalid_provider_returns_error` in `src/tools/delegate.rs`

#### Scenario: No agents configured
- WHEN DelegateTool has no agents and execute is called
- THEN MUST return an error indicating no agents available
- Test: `delegate_no_agents_configured` in `src/tools/delegate.rs`

### REQ-DELEG-004: Delegate Context Handling

`DelegateTool` MUST prepend context to the prompt when provided, and omit the prefix when context is empty.

#### Scenario: Context prepended
- WHEN context parameter is provided
- THEN MUST prepend context to the prompt
- Test: `delegate_context_is_prepended_to_prompt` in `src/tools/delegate.rs`

#### Scenario: Empty context omitted
- WHEN context is empty or absent
- THEN MUST omit context prefix
- Test: `delegate_empty_context_omits_prefix` in `src/tools/delegate.rs`

### REQ-DELEG-005: Delegate Security Enforcement

`DelegateTool` MUST respect read-only mode, rate limiting, and depth limits.

#### Scenario: Read-only mode
- WHEN read-only mode is active
- THEN MUST block delegation
- Test: `delegation_blocked_in_readonly_mode` in `src/tools/delegate.rs`

#### Scenario: Rate limiting
- WHEN rate limit is exceeded
- THEN MUST block delegation
- Test: `delegation_blocked_when_rate_limited` in `src/tools/delegate.rs`

#### Scenario: Depth limit enforcement (global)
- WHEN delegation depth exceeds the global configured limit
- THEN MUST reject further delegation
- Test: `depth_limit_enforced` in `src/tools/delegate.rs`

#### Scenario: Depth limit per agent
- WHEN delegation depth exceeds the per-agent max_depth limit
- THEN MUST reject further delegation for that agent
- Test: `depth_limit_per_agent` in `src/tools/delegate.rs`

#### Scenario: Depth construction
- WHEN DelegateTool is constructed with a specific depth
- THEN the depth field MUST be set correctly
- Test: `delegate_depth_construction` in `src/tools/delegate.rs`

### REQ-DELEG-006: Delegate Agentic Mode

`DelegateTool` MUST support agentic mode with tool-call loop, tool filtering, and iteration limits.

#### Scenario: Empty allowed tools rejected
- WHEN agentic mode is requested with empty allowed_tools
- THEN MUST reject
- Test: `agentic_mode_rejects_empty_allowed_tools` in `src/tools/delegate.rs`

#### Scenario: Unmatched tools rejected
- WHEN allowed_tools references non-existent tools
- THEN MUST reject
- Test: `agentic_mode_rejects_unmatched_allowed_tools` in `src/tools/delegate.rs`

#### Scenario: Tool call loop with filtered tools
- WHEN agentic mode executes with allowed_tools
- THEN MUST run the tool-call loop with only the allowed tools
- Test: `execute_agentic_runs_tool_call_loop_with_filtered_tools` in `src/tools/delegate.rs`

#### Scenario: Self-exclusion
- WHEN delegate is in allowed_tools list
- THEN MUST exclude delegate from filtered tools
- Test: `execute_agentic_excludes_delegate_even_if_allowlisted` in `src/tools/delegate.rs`

#### Scenario: Max iterations
- WHEN agentic loop reaches max iterations
- THEN MUST stop and return partial result
- Test: `execute_agentic_respects_max_iterations` in `src/tools/delegate.rs`

#### Scenario: Provider error propagation
- WHEN the provider returns an error during agentic execution
- THEN MUST propagate the error
- Test: `execute_agentic_propagates_provider_errors` in `src/tools/delegate.rs`

### REQ-DELEG-007: Delegate Coordination Tracing

`DelegateTool` MUST record coordination events (start, completion, failure) to the coordination bus when enabled.

#### Scenario: Failure events
- WHEN delegation fails
- THEN MUST record failure events in coordination bus
- Test: `execute_records_failure_events_in_coordination_bus` in `src/tools/delegate.rs`

#### Scenario: Completion transition
- WHEN delegation completes
- THEN MUST transition coordination state to completed
- Test: `coordination_trace_transitions_state_to_completed` in `src/tools/delegate.rs`

---

### Delegate Coordination Status Tool (`src/tools/delegate_coordination_status.rs`)

### REQ-DELEG-008: Delegate Coordination Status Tool

`DelegateCoordinationStatusTool` MUST provide read-only observability into delegation coordination state with filtering and pagination.

#### Scenario: Context and inbox reporting
- WHEN status is queried
- THEN MUST report context and inboxes with optional agent/correlation_id filters
- Test: `status_tool_reports_context_and_inboxes` in `src/tools/delegate_coordination_status.rs`

#### Scenario: Dead letter pagination
- WHEN dead_letter_limit is set
- THEN MUST apply pagination to dead letter results
- Test: `status_tool_applies_dead_letter_limit` in `src/tools/delegate_coordination_status.rs`

#### Scenario: Context pagination
- WHEN context_limit is set
- THEN MUST apply pagination in recent-first order
- Test: `status_tool_applies_context_limit_in_recent_order` in `src/tools/delegate_coordination_status.rs`

#### Scenario: Context paging with correlation filter
- WHEN correlation_id filter is combined with context paging
- THEN MUST apply both filters correctly
- Test: `status_tool_applies_context_paging_with_correlation_filter` in `src/tools/delegate_coordination_status.rs`

#### Scenario: Dead letter paging with correlation filter
- WHEN correlation_id filter is combined with dead letter paging
- THEN MUST apply both filters correctly
- Test: `status_tool_applies_dead_letter_paging_with_correlation_filter` in `src/tools/delegate_coordination_status.rs`

#### Scenario: Message paging with correlation filter
- WHEN correlation_id filter is combined with message paging
- THEN MUST apply both filters correctly
- Test: `status_tool_applies_message_paging_with_correlation_filter` in `src/tools/delegate_coordination_status.rs`

---

### SubAgent Spawn Tool (`src/tools/subagent_spawn.rs`)

### REQ-DELEG-009: SubAgent Spawn Tool Identity

`SubAgentSpawnTool` MUST implement the Tool trait with correct name and schema.

#### Scenario: Name and schema
- WHEN `name()` and `parameters_schema()` are called
- THEN MUST return `"subagent_spawn"` and valid schema listing agent names
- Test: `name_and_schema` in `src/tools/subagent_spawn.rs`
- Test: `schema_lists_agent_names` in `src/tools/subagent_spawn.rs`

#### Scenario: Description not empty
- WHEN `description()` is called
- THEN MUST return a non-empty description
- Test: `description_not_empty` in `src/tools/subagent_spawn.rs`

### REQ-DELEG-009A: SubAgent Spawn Parameter Validation

#### Scenario: Missing agent parameter
- WHEN the agent parameter is missing
- THEN MUST return error
- Test: `missing_agent_param` in `src/tools/subagent_spawn.rs`

#### Scenario: Missing task parameter
- WHEN the task parameter is missing
- THEN MUST return error
- Test: `missing_task_param` in `src/tools/subagent_spawn.rs`

#### Scenario: Blank agent rejected
- WHEN the agent parameter is blank
- THEN MUST return error
- Test: `blank_agent_rejected` in `src/tools/subagent_spawn.rs`

#### Scenario: Blank task rejected
- WHEN the task parameter is blank
- THEN MUST return error
- Test: `blank_task_rejected` in `src/tools/subagent_spawn.rs`

#### Scenario: Unknown agent
- WHEN an unknown agent name is provided
- THEN MUST return error
- Test: `unknown_agent_returns_error` in `src/tools/subagent_spawn.rs`

### REQ-DELEG-009B: SubAgent Spawn Execution

#### Scenario: Session ID returned
- WHEN spawn succeeds
- THEN MUST return session ID
- Test: `spawn_returns_session_id` in `src/tools/subagent_spawn.rs`

#### Scenario: No agents configured
- WHEN no agents are configured and spawn is attempted
- THEN MUST return error
- Test: `spawn_no_agents_configured` in `src/tools/subagent_spawn.rs`

#### Scenario: Concurrent limit
- WHEN concurrent session limit is reached
- THEN MUST reject new spawns
- Test: `spawn_respects_concurrent_limit` in `src/tools/subagent_spawn.rs`

### REQ-DELEG-009C: SubAgent Spawn Security

#### Scenario: Read-only mode blocks spawn
- WHEN read-only mode is active
- THEN MUST block spawning
- Test: `spawn_blocked_in_readonly_mode` in `src/tools/subagent_spawn.rs`

#### Scenario: Rate limit blocks spawn
- WHEN rate limit is exceeded
- THEN MUST block spawning
- Test: `spawn_blocked_when_rate_limited` in `src/tools/subagent_spawn.rs`

---

### SubAgent Manage Tool (`src/tools/subagent_manage.rs`)

### REQ-DELEG-010: SubAgent Manage Tool Identity

#### Scenario: Name and schema
- WHEN `name()` and `parameters_schema()` are called
- THEN MUST return `"subagent_manage"` and valid schema
- Test: `name_and_schema` in `src/tools/subagent_manage.rs`

#### Scenario: Description not empty
- WHEN `description()` is called
- THEN MUST return a non-empty description
- Test: `description_not_empty` in `src/tools/subagent_manage.rs`

### REQ-DELEG-010A: SubAgent Manage Parameter Validation

#### Scenario: Missing session_id
- WHEN session_id is missing
- THEN MUST return error
- Test: `missing_session_id` in `src/tools/subagent_manage.rs`

#### Scenario: Missing action
- WHEN action is missing
- THEN MUST return error
- Test: `missing_action` in `src/tools/subagent_manage.rs`

#### Scenario: Blank session_id rejected
- WHEN session_id is blank
- THEN MUST return error
- Test: `blank_session_id_rejected` in `src/tools/subagent_manage.rs`

#### Scenario: Blank action rejected
- WHEN action is blank
- THEN MUST return error
- Test: `blank_action_rejected` in `src/tools/subagent_manage.rs`

#### Scenario: Unknown action rejected
- WHEN an unknown action is provided
- THEN MUST return error
- Test: `unknown_action_rejected` in `src/tools/subagent_manage.rs`

### REQ-DELEG-010B: SubAgent Manage Status Queries

#### Scenario: Status of unknown session
- WHEN status is queried for a non-existent session
- THEN MUST return error indicating session not found
- Test: `status_unknown_session` in `src/tools/subagent_manage.rs`

#### Scenario: Status of running session
- WHEN status is queried for a running session
- THEN MUST return running status with agent and task info
- Test: `status_running_session` in `src/tools/subagent_manage.rs`

#### Scenario: Status of completed session
- WHEN status is queried for a completed session
- THEN MUST return completed status with result
- Test: `status_completed_session` in `src/tools/subagent_manage.rs`

#### Scenario: Status truncates long output
- WHEN a completed session has very long output
- THEN MUST truncate the output in the status report
- Test: `status_truncates_long_output` in `src/tools/subagent_manage.rs`

### REQ-DELEG-010C: SubAgent Manage Kill Operations

#### Scenario: Kill running session
- WHEN kill action targets a running session
- THEN MUST abort the session
- Test: `kill_running_session` in `src/tools/subagent_manage.rs`

#### Scenario: Kill completed session fails
- WHEN kill action targets a completed session
- THEN MUST return error (cannot kill completed)
- Test: `kill_completed_session_fails` in `src/tools/subagent_manage.rs`

#### Scenario: Kill unknown session
- WHEN kill action targets an unknown session
- THEN MUST return error
- Test: `kill_unknown_session` in `src/tools/subagent_manage.rs`

### REQ-DELEG-010D: SubAgent Manage Security

#### Scenario: Kill blocked in read-only mode
- WHEN read-only mode is active and kill is attempted
- THEN MUST block the operation
- Test: `kill_blocked_in_readonly_mode` in `src/tools/subagent_manage.rs`

#### Scenario: Kill blocked when rate limited
- WHEN rate limit is exceeded and kill is attempted
- THEN MUST block the operation
- Test: `kill_blocked_when_rate_limited` in `src/tools/subagent_manage.rs`

#### Scenario: Status allowed in read-only mode
- WHEN read-only mode is active and status is queried
- THEN MUST allow the read operation
- Test: `status_allowed_in_readonly_mode` in `src/tools/subagent_manage.rs`

---

### SubAgent List Tool (`src/tools/subagent_list.rs`)

### REQ-DELEG-011: SubAgent List Tool Identity

#### Scenario: Name and schema
- WHEN `name()` and `parameters_schema()` are called
- THEN MUST return `"subagent_list"` and valid schema
- Test: `name_and_schema` in `src/tools/subagent_list.rs`

#### Scenario: Description not empty
- WHEN `description()` is called
- THEN MUST return a non-empty description
- Test: `description_not_empty` in `src/tools/subagent_list.rs`

### REQ-DELEG-011A: SubAgent List Operations

#### Scenario: Empty registry
- WHEN no sessions exist
- THEN MUST return empty list message
- Test: `list_empty_registry` in `src/tools/subagent_list.rs`

#### Scenario: List all sessions
- WHEN sessions exist and no filter is provided
- THEN MUST return all sessions
- Test: `list_all_sessions` in `src/tools/subagent_list.rs`

#### Scenario: Filter running sessions
- WHEN status filter is "running"
- THEN MUST return only running sessions
- Test: `list_filters_running` in `src/tools/subagent_list.rs`

#### Scenario: Filter completed sessions
- WHEN status filter is "completed"
- THEN MUST return only completed sessions
- Test: `list_filters_completed` in `src/tools/subagent_list.rs`

#### Scenario: Filter failed sessions
- WHEN status filter is "failed"
- THEN MUST return only failed sessions
- Test: `list_filters_failed` in `src/tools/subagent_list.rs`

#### Scenario: Default shows all
- WHEN no status filter is provided (default)
- THEN MUST show all sessions regardless of status
- Test: `list_default_shows_all` in `src/tools/subagent_list.rs`

#### Scenario: Invalid status filter
- WHEN an invalid status filter is provided
- THEN MUST return error or empty result
- Test: `invalid_status_filter` in `src/tools/subagent_list.rs`

---

### SubAgent Registry (`src/tools/subagent_registry.rs`)

### REQ-DELEG-012: SubAgent Registry Lifecycle

`SubAgentRegistry` MUST manage concurrent sub-agent sessions with lifecycle tracking, concurrent limits, and session cleanup.

#### Scenario: Insert and list
- WHEN a session is inserted
- THEN MUST appear in list results
- Test: `registry_insert_and_list` in `src/tools/subagent_registry.rs`

#### Scenario: Complete session
- WHEN a session is completed with a result
- THEN MUST transition to completed status with result stored
- Test: `registry_complete_session` in `src/tools/subagent_registry.rs`

#### Scenario: Fail session
- WHEN a session fails with an error
- THEN MUST transition to failed status with error stored
- Test: `registry_fail_session` in `src/tools/subagent_registry.rs`

#### Scenario: Kill running session
- WHEN a running session is killed
- THEN MUST transition to killed status and return true
- Test: `registry_kill_running_session` in `src/tools/subagent_registry.rs`

#### Scenario: Kill non-running returns false
- WHEN a non-running session is killed
- THEN MUST return false
- Test: `registry_kill_non_running_returns_false` in `src/tools/subagent_registry.rs`

#### Scenario: Kill unknown returns false
- WHEN an unknown session ID is killed
- THEN MUST return false
- Test: `registry_kill_unknown_returns_false` in `src/tools/subagent_registry.rs`

#### Scenario: List filters by status
- WHEN list is called with a status filter
- THEN MUST return only matching sessions
- Test: `registry_list_filters_by_status` in `src/tools/subagent_registry.rs`

#### Scenario: Get status of unknown session
- WHEN get_status is called for an unknown session
- THEN MUST return None
- Test: `registry_get_status_unknown` in `src/tools/subagent_registry.rs`

#### Scenario: Session exists check
- WHEN exists is called for a known/unknown session
- THEN MUST return true/false appropriately
- Test: `registry_exists` in `src/tools/subagent_registry.rs`

#### Scenario: Running count
- WHEN running_count is called
- THEN MUST return the number of currently running sessions
- Test: `registry_running_count` in `src/tools/subagent_registry.rs`

#### Scenario: Old session cleanup
- WHEN completed/failed sessions expire beyond max age
- THEN MUST clean them up on subsequent list operations
- Test: `registry_cleanup_old_sessions` in `src/tools/subagent_registry.rs`

### REQ-DELEG-012A: SubAgent Registry Utilities

#### Scenario: Task truncation (short)
- WHEN truncate_task is called on a short task
- THEN MUST return it unchanged
- Test: `truncate_task_short` in `src/tools/subagent_registry.rs`

#### Scenario: Task truncation (long)
- WHEN truncate_task is called on a long task
- THEN MUST truncate with ellipsis
- Test: `truncate_task_long` in `src/tools/subagent_registry.rs`

#### Scenario: Task truncation (multibyte safe)
- WHEN truncate_task is called on multibyte content
- THEN MUST truncate on char boundary
- Test: `truncate_task_multibyte_safe` in `src/tools/subagent_registry.rs`

#### Scenario: Status display
- WHEN SubAgentStatus variants are displayed
- THEN MUST return correct string representations
- Test: `status_display` in `src/tools/subagent_registry.rs`

#### Scenario: Default construction
- WHEN SubAgentRegistry::default() is called
- THEN MUST create an empty registry
- Test: `registry_default` in `src/tools/subagent_registry.rs`

### REQ-DELEG-012B: SubAgent Registry Concurrency

#### Scenario: Concurrent insert and list
- WHEN multiple threads insert and list concurrently
- THEN MUST handle safely without data races
- Test: `concurrent_insert_and_list` in `src/tools/subagent_registry.rs`

#### Scenario: Session info serialization
- WHEN SubAgentSessionInfo is serialized
- THEN MUST produce valid JSON
- Test: `session_info_serialization` in `src/tools/subagent_registry.rs`

---

### MCP Tool Wrapper (`src/tools/mcp_tool.rs`)

### REQ-DELEG-013: MCP Tool Wrapper Identity

`McpToolWrapper` MUST implement the `Tool` trait wrapping an MCP tool definition with prefixed naming.

#### Scenario: Prefixed name
- WHEN `name()` is called
- THEN MUST return the prefixed tool name (e.g. `server__tool_name`)
- Test: `mcp_tool_wrapper_name` in `src/tools/mcp_tool.rs`

#### Scenario: Description forwarding
- WHEN `description()` is called on a tool with a description
- THEN MUST return the MCP tool definition description
- Test: `mcp_tool_wrapper_description` in `src/tools/mcp_tool.rs`

#### Scenario: Default description
- WHEN `description()` is called on a tool without a description
- THEN MUST return the default "MCP tool" description
- Test: `mcp_tool_wrapper_default_description` in `src/tools/mcp_tool.rs`

#### Scenario: Schema forwarding
- WHEN `parameters_schema()` is called
- THEN MUST return the MCP tool input schema
- Test: `mcp_tool_wrapper_schema` in `src/tools/mcp_tool.rs`

### REQ-DELEG-013A: MCP Tool Wrapper Execution

#### Scenario: Execute with unknown tool returns error
- WHEN execute is called for a tool not registered in the MCP registry
- THEN MUST return an error
- Test: `mcp_tool_wrapper_execute_unknown_tool` in `src/tools/mcp_tool.rs`

## Mock Strategy

- Provider: Mock provider implementing `Provider` trait with configurable responses (`OneToolThenFinalProvider`, `InfiniteToolCallProvider`, `FailingProvider`)
- Agent configs: Constructed with test defaults (provider name, model, system prompt)
- SecurityPolicy: Default construction with test-safe settings (read-only and rate-limited variants for enforcement tests)
- SubAgentRegistry: Direct construction with `Arc<SubAgentRegistry>`
- CoordinationBus: `InMemoryMessageBus` for coordination tracing tests
- McpRegistry: `Arc<McpRegistry>` with empty server list for wrapper tests
- EchoTool: Lightweight mock tool that echoes `value` argument

## Coverage Notes
- `src/tools/delegate.rs`: 27 tests (identity, validation, resolution, context, security, agentic, coordination)
- `src/tools/delegate_coordination_status.rs`: 6 tests (context/inbox reporting, pagination, correlation filtering)
- `src/tools/subagent_spawn.rs`: 13 tests (identity, validation, execution, security)
- `src/tools/subagent_manage.rs`: 17 tests (identity, validation, status queries, kill ops, security)
- `src/tools/subagent_list.rs`: 9 tests (identity, listing, filtering)
- `src/tools/subagent_registry.rs`: 18 tests (lifecycle, utilities, concurrency)
- `src/tools/mcp_tool.rs`: 5 tests (identity, default description, execution)
- Total: 95 tests across 7 files
