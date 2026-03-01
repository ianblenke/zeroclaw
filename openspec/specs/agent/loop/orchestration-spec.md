# Agent Orchestration Loop Specification

## Purpose

Define requirements for the main agent loop orchestration: provider vision detection, cost estimation/enforcement, budget messaging, error classification, and progress utilities.

## Scope

- Files: `src/agent/loop_.rs` (6,085 LOC, 154 existing tests)
- Risk tier: HIGH (controls tool execution, cost enforcement, and error recovery)

## Requirements

### REQ-ORCH-001: Vision Capability Detection

`should_treat_provider_as_vision_capable` MUST determine if a provider supports vision, with special handling for anthropic routes.

#### Scenario: Anthropic treated as vision capable

- WHEN provider name is "anthropic" or starts with "anthropic-custom:"
- THEN MUST return true even if supports_vision() returns false
- Test: `vision_capable_anthropic_always_true` in `src/agent/loop_.rs`

#### Scenario: Non-anthropic depends on trait

- WHEN provider name is not anthropic
- THEN MUST return the value of `provider.supports_vision()`
- Test: `vision_capable_non_anthropic_delegates` in `src/agent/loop_.rs`

### REQ-ORCH-002: Token Estimation

`estimate_prompt_tokens` MUST produce a reasonable token estimate from message text and tool specs.

#### Scenario: Empty messages

- WHEN messages is empty and no tools
- THEN MUST return a small overhead value (framing only)
- Test: `estimate_tokens_empty` in `src/agent/loop_.rs`

#### Scenario: Message content counted

- WHEN messages have content
- THEN estimate MUST grow roughly proportionally to content length
- Test: `estimate_tokens_proportional` in `src/agent/loop_.rs`

### REQ-ORCH-003: Model Pricing Lookup

`lookup_model_pricing` MUST resolve pricing via exact match, model-only match, prefix match, or default.

#### Scenario: Exact provider/model match

- WHEN prices contain "provider/model" key
- THEN MUST return that pricing
- Test: `pricing_exact_match` in `src/agent/loop_.rs`

#### Scenario: Model-only match

- WHEN prices contain model name without provider prefix
- THEN MUST return that pricing
- Test: `pricing_model_only` in `src/agent/loop_.rs`

#### Scenario: Default fallback

- WHEN no match found
- THEN MUST return (3.0, 15.0) default
- Test: `pricing_default_fallback` in `src/agent/loop_.rs`

### REQ-ORCH-004: Usage Period Labels

`usage_period_label` MUST map period enums to human-readable strings.

#### Scenario: All variants

- WHEN each variant is provided
- THEN MUST return "session", "daily", "monthly" respectively
- Test: `usage_period_labels` in `src/agent/loop_.rs`

### REQ-ORCH-005: Budget Exceeded Message

`budget_exceeded_message` MUST format a clear enforcement message.

#### Scenario: Message format

- WHEN called with cost/limit parameters
- THEN MUST include model name, projected cost, period label, and limit
- Test: `budget_message_format` in `src/agent/loop_.rs`

### REQ-ORCH-006: Error Classification

Error classification functions MUST correctly identify specific error types.

#### Scenario: ToolLoopCancelled detection

- WHEN error is ToolLoopCancelled
- THEN `is_tool_loop_cancelled` MUST return true
- Test: `error_classify_tool_loop_cancelled` in `src/agent/loop_.rs`

#### Scenario: Iteration limit detection

- WHEN error message contains "Agent exceeded maximum tool iterations"
- THEN `is_tool_iteration_limit_error` MUST return true
- Test: `error_classify_iteration_limit` in `src/agent/loop_.rs`

#### Scenario: Loop detection detection

- WHEN error message contains "Agent stopped early due to detected loop pattern"
- THEN `is_loop_detection_error` MUST return true
- Test: `error_classify_loop_detection` in `src/agent/loop_.rs`

#### Scenario: Non-matching errors return false

- WHEN error does not match the specific type
- THEN classification functions MUST return false
- Test: `error_classify_non_matching` in `src/agent/loop_.rs`

### REQ-ORCH-007: Truncate Tool Args for Progress

`truncate_tool_args_for_progress` MUST extract a human-readable hint from tool arguments.

#### Scenario: Shell tool extracts command

- WHEN tool name is "shell" and arguments have "command"
- THEN MUST return the command value (truncated if needed)
- Test: `progress_tracker_renders_in_place_block` in `src/agent/loop_.rs`

## Mock Strategy

- **Provider trait**: Simple struct with controlled `supports_vision()` return value for vision tests
- **ModelPricing**: Constructed directly as HashMap entries
- **CostEnforcementContext**: Not tested directly (requires CostTracker with filesystem); tested indirectly via `estimate_request_cost_usd` parameter inputs
- **Error types**: Constructed via `anyhow::Error::new()` and `anyhow::anyhow!()`
