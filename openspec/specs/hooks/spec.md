# Hooks Specification

## Purpose

Define requirements for the hook system: trait contract, hook runner, and built-in hooks (boot_script, command_logger, session_memory).

## Scope

- Files: 7 files in `src/hooks/` (~1,081 LOC total)
- Risk tier: LOW (hooks execute auxiliary actions; no security-critical paths)
- Total tests: 15 across 5 files

## Requirements

---

### HookHandler Trait (`src/hooks/traits.rs`)

### REQ-HOOK-001: HookResult::is_cancel MUST correctly classify variants

#### Scenario: Continue vs Cancel
- WHEN `HookResult::Continue` and `HookResult::Cancel` variants are checked
- THEN `is_cancel()` returns false for Continue and true for Cancel
- Test: `hook_result_is_cancel` in `src/hooks/traits.rs`

### REQ-HOOK-002: Default priority MUST be 0

#### Scenario: Default priority
- WHEN `priority()` is called on a minimal HookHandler implementation
- THEN it returns 0
- Test: `default_priority_is_zero` in `src/hooks/traits.rs`

### REQ-HOOK-003: Default capabilities MUST be empty

#### Scenario: Empty capabilities
- WHEN `capabilities()` is called on a default HookHandler implementation
- THEN it returns an empty slice
- Test: `default_hook_capabilities_empty` in `src/hooks/traits.rs`

### REQ-HOOK-004: Default void hooks MUST complete without error

#### Scenario: Void hooks no-op
- WHEN `on_gateway_start`, `on_session_start`, `on_heartbeat_tick` are called on default implementation
- THEN they complete without error
- Test: `default_void_hooks_complete_without_error` in `src/hooks/traits.rs`

### REQ-HOOK-005: Default modifying hooks MUST pass through unchanged

#### Scenario: before_tool_call pass-through
- WHEN `before_tool_call`, `before_prompt_build`, etc. are called on default implementation
- THEN they return Continue with original values unmodified
- Test: `default_modifying_hooks_pass_through` in `src/hooks/traits.rs`

---

### Hook Runner (`src/hooks/runner.rs`)

### REQ-HOOK-006: HookRunner MUST register and sort hooks by priority

#### Scenario: Registration and sorting
- WHEN hooks with different priorities are registered
- THEN they are stored sorted by priority (ascending)
- Test: `register_and_sort_by_priority` in `src/hooks/runner.rs`

### REQ-HOOK-007: HookRunner MUST fire void hooks to all handlers

#### Scenario: Void hooks fire all
- WHEN a void event (e.g. heartbeat_tick) is fired
- THEN all registered handlers receive the event
- Test: `void_hooks_fire_all_handlers` in `src/hooks/runner.rs`

### REQ-HOOK-008: Modifying hooks MUST support cancellation

#### Scenario: Hook cancels pipeline
- WHEN a modifying hook returns `HookResult::Cancel`
- THEN the pipeline stops and returns Cancel
- Test: `modifying_hook_can_cancel` in `src/hooks/runner.rs`

### REQ-HOOK-009: Modifying hooks MUST pipeline data through chain

#### Scenario: Hook pipelines data
- WHEN multiple modifying hooks are registered
- THEN each receives the output of the previous hook
- Test: `modifying_hook_pipelines_data` in `src/hooks/runner.rs`

### REQ-HOOK-010: tool_result_persist MUST enforce capability gating

#### Scenario: Modification blocked without capability
- WHEN a hook without `ModifyToolResults` capability tries to modify a tool result
- THEN the modification is silently blocked
- Test: `tool_result_persist_blocks_modification_without_capability` in `src/hooks/runner.rs`

#### Scenario: Modification allowed with capability
- WHEN a hook with `ModifyToolResults` capability modifies a tool result
- THEN the modification is applied
- Test: `tool_result_persist_allows_modification_with_capability` in `src/hooks/runner.rs`

#### Scenario: Cancel blocked without capability
- WHEN a hook without `ModifyToolResults` capability tries to cancel a tool result
- THEN the cancellation is blocked
- Test: `tool_result_persist_blocks_cancel_without_capability` in `src/hooks/runner.rs`

---

### Built-in Hooks

### REQ-HOOK-011: BootScript hook MUST pass prompt through

#### Scenario: Boot script execution
- WHEN `before_prompt_build` is called on BootScript hook
- THEN the prompt is passed through unchanged
- Test: `boot_script_hook_passes_prompt_through` in `src/hooks/builtin/boot_script.rs`

### REQ-HOOK-012: CommandLogger hook MUST log tool calls

#### Scenario: Command logging
- WHEN `on_after_tool_call` is called with tool name and result
- THEN the tool call is recorded in the logger's entries
- Test: `logs_tool_calls` in `src/hooks/builtin/command_logger.rs`

### REQ-HOOK-013: SessionMemory hook MUST pass messages through

#### Scenario: Session memory compaction
- WHEN `before_compaction` is called with messages
- THEN messages are passed through unchanged (memory extraction is separate)
- Test: `session_memory_hook_passes_messages_through` in `src/hooks/builtin/session_memory.rs`

---

## Mock Strategy

- Hook execution: Direct construction with test configs
- Mock hooks: Manual mock structs implementing HookHandler with counters/flags
- File system: `tempfile::TempDir` for log/script isolation

## Coverage Notes
- `src/hooks/traits.rs`: 5 tests — trait defaults and HookResult classification
- `src/hooks/runner.rs`: 7 tests — registration, sorting, void/modifying pipelines, capability gating
- `src/hooks/builtin/boot_script.rs`: 1 test — prompt pass-through
- `src/hooks/builtin/command_logger.rs`: 1 test — tool call logging
- `src/hooks/builtin/session_memory.rs`: 1 test — message pass-through
