# Agent Loop Execution, Context, and History Specification

## Purpose

Define requirements for the agent tool execution pipeline, memory context building, hardware RAG context, and conversation history management (trimming, compaction).

## Scope

- Files: `src/agent/loop_/execution.rs` (7 tests), `src/agent/loop_/context.rs` (3 tests), `src/agent/loop_/history.rs` (8 tests), `src/agent/memory_loader.rs` (3 tests)
- Total tests: 21
- Risk tier: HIGH (execution.rs orchestrates actual tool invocations; history compaction affects conversation fidelity)

## Requirements

---

### Tool Execution (`src/agent/loop_/execution.rs`)

### REQ-EXEC-001: Tool Lookup

`find_tool` MUST find a tool by name from the tool registry.

#### Scenario: Known tool found

- WHEN the registry contains a tool with the requested name
- THEN `find_tool` MUST return a reference to that tool
- Test: `find_tool_returns_matching` in `src/agent/loop_/execution.rs`

#### Scenario: Unknown tool not found

- WHEN the registry does not contain the requested name
- THEN `find_tool` MUST return `None`
- Test: `find_tool_returns_none_for_unknown` in `src/agent/loop_/execution.rs`

### REQ-EXEC-002: Tool Execution Outcome

`execute_one_tool` MUST return a structured `ToolExecutionOutcome` for every outcome path.

#### Scenario: Successful execution

- WHEN a tool executes successfully
- THEN outcome MUST have success=true and scrubbed output
- Test: `execute_one_tool_success` in `src/agent/loop_/execution.rs`

#### Scenario: Unknown tool

- WHEN the tool is not found in the registry
- THEN outcome MUST have success=false and error containing "Unknown tool"
- Test: `execute_one_tool_unknown_tool` in `src/agent/loop_/execution.rs`

#### Scenario: Tool returns error

- WHEN a tool returns ToolResult with success=false
- THEN outcome MUST have success=false with "Error:" prefix
- Test: `execute_one_tool_tool_error` in `src/agent/loop_/execution.rs`

### REQ-EXEC-003: Parallel Execution Decision

`should_execute_tools_in_parallel` MUST decide whether multiple tool calls can run concurrently.

#### Scenario: Single call is never parallel

- WHEN there is 0 or 1 tool call
- THEN MUST return false
- Test: `parallel_single_call_false` in `src/agent/loop_/execution.rs`

#### Scenario: Multiple calls without approval

- WHEN there are 2+ calls and no approval gating
- THEN MUST return true
- Test: `parallel_multiple_no_approval` in `src/agent/loop_/execution.rs`

---

### Conversation History (`src/agent/loop_/history.rs`)

### REQ-HIST-001: History Trimming

`trim_history` MUST trim conversation history while preserving system prompt and avoiding orphan tool messages.

#### Scenario: Within limit unchanged

- WHEN history count is within max_history
- THEN `trim_history` MUST not modify the history
- Test: `trim_history_within_limit` in `src/agent/loop_/history.rs`

#### Scenario: System prompt preserved

- WHEN first message is role=system
- THEN `trim_history` MUST preserve it
- Test: `trim_history_preserves_system` in `src/agent/loop_/history.rs`

#### Scenario: Tool messages not orphaned

- WHEN trim boundary falls before a tool message
- THEN `trim_history` MUST advance to avoid orphan tool results
- Test: `trim_history_avoids_orphan_tool_at_boundary` in `src/agent/loop_/history.rs`

### REQ-HIST-002: Compaction Transcript

`build_compaction_transcript` MUST build a text transcript from messages for summarization.

#### Scenario: Messages are formatted

- WHEN given a list of ChatMessages
- THEN MUST produce uppercase-role prefixed lines
- Test: `compaction_transcript_format` in `src/agent/loop_/history.rs`

#### Scenario: Long transcripts are truncated

- WHEN the transcript exceeds COMPACTION_MAX_SOURCE_CHARS
- THEN MUST truncate with ellipsis
- Test: `compaction_transcript_truncated` in `src/agent/loop_/history.rs`

### REQ-HIST-003: Compaction Summary Application

`apply_compaction_summary` MUST replace a range of messages with a compaction summary.

#### Scenario: Messages are replaced

- WHEN a compaction range is specified
- THEN MUST splice the range and insert a single assistant summary message
- Test: `apply_compaction_replaces_range` in `src/agent/loop_/history.rs`

### REQ-HIST-004: Auto Compaction

`auto_compact_history` MUST automatically compact old messages when history exceeds max_history.

#### Scenario: Within limit returns false

- WHEN history count is within max_history
- THEN MUST return Ok(false) without modification
- Test: `auto_compact_within_limit` in `src/agent/loop_/history.rs`

#### Scenario: Compaction preserves tool-run boundaries

- WHEN history exceeds max_history
- THEN MUST compact old messages while preserving tool-run boundaries and the most recent messages
- Test: `auto_compact_history_does_not_split_tool_run_boundary` in `src/agent/loop_/history.rs`

---

### Memory Context (`src/agent/loop_/context.rs`)

### REQ-CTX-001: Memory Context Building

`build_context` MUST build context from memory entries filtered by relevance score.

#### Scenario: Relevant entries included

- WHEN memory returns entries above min_relevance_score
- THEN MUST include them in `[Memory context]` block
- Test: `build_context_includes_relevant` in `src/agent/loop_/context.rs`

#### Scenario: Low-score entries filtered

- WHEN entries have scores below min_relevance_score
- THEN MUST exclude them
- Test: `build_context_filters_low_score` in `src/agent/loop_/context.rs`

#### Scenario: No entries returns empty

- WHEN memory returns no entries
- THEN MUST return an empty string
- Test: `build_context_empty_memory` in `src/agent/loop_/context.rs`

---

### Memory Loader (`src/agent/memory_loader.rs`)

### REQ-LOADER-001: Memory Loader

`DefaultMemoryLoader` MUST load and format memory context with relevance filtering.

#### Scenario: Formats context

- WHEN memory has entries
- THEN MUST format as `[Memory context]\n- key: value` block
- Test: `default_loader_formats_context` in `src/agent/memory_loader.rs`

#### Scenario: Filters autosave entries

- WHEN memory has assistant autosave entries
- THEN MUST skip them
- Test: `default_loader_skips_legacy_assistant_autosave_entries` in `src/agent/memory_loader.rs`

#### Scenario: Filters low-score entries

- WHEN all entries are below min_relevance_score
- THEN MUST return empty string
- Test: `default_loader_filters_low_score` in `src/agent/memory_loader.rs`

## Mock Strategy

- **Tool trait**: Lightweight struct implementing `Tool` with controlled `execute()` return values
- **Observer trait**: NoOp observer implementation
- **Memory trait**: In-memory mock returning pre-configured entries
- **Provider trait**: Static-response provider for compaction tests (`StaticSummaryProvider`)
- **HardwareRag**: Not mocked in unit tests (requires construction with empty state)

## Coverage Notes
- `src/agent/loop_/execution.rs`: 7 tests (find_tool, execute_one_tool, parallel decision)
- `src/agent/loop_/history.rs`: 8 tests (trim, compaction transcript, apply summary, auto compact)
- `src/agent/loop_/context.rs`: 3 tests (build_context with relevance filtering)
- `src/agent/memory_loader.rs`: 3 tests (format, autosave filter, low-score filter)
- Total: 21 tests across 4 files
