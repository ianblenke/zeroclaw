# Tools Scheduling Specification

## Purpose

Define requirements for the scheduling tool implementations: cron job management (add, list, remove, run, runs, update) and the schedule tool.

## Scope

- Files: `src/tools/cron_add.rs` (11 tests), `src/tools/cron_list.rs` (2 tests), `src/tools/cron_remove.rs` (4 tests), `src/tools/cron_run.rs` (5 tests), `src/tools/cron_runs.rs` (2 tests), `src/tools/cron_update.rs` (5 tests), `src/tools/schedule.rs` (11 tests)
- Total tests: 40
- Risk tier: MEDIUM (cron jobs execute scheduled commands; schedule tool creates delayed actions)

## Requirements

---

### Cron Add (`src/tools/cron_add.rs`)

### REQ-CRON-001: CronAddTool MUST create cron jobs with validated cron expressions

#### Scenario: Valid shell job creation
- WHEN a valid cron expression and shell command are provided
- THEN MUST create the job and return its ID
- Test: `adds_shell_job` in `src/tools/cron_add.rs`

#### Scenario: Invalid schedule rejected
- WHEN an invalid cron/at/every expression is provided
- THEN MUST return an error
- Test: `rejects_invalid_schedule` in `src/tools/cron_add.rs`

### REQ-CRON-001A: CronAddTool MUST enforce security policies

#### Scenario: Disallowed command blocked
- WHEN a shell command is not in the allowlist
- THEN MUST return an error
- Test: `blocks_disallowed_shell_command` in `src/tools/cron_add.rs`

#### Scenario: Read-only mode blocks mutation
- WHEN read-only mode is active
- THEN MUST block the add operation
- Test: `blocks_mutation_in_read_only_mode` in `src/tools/cron_add.rs`

#### Scenario: Rate limit blocks mutation
- WHEN rate limit is exceeded
- THEN MUST block the add operation
- Test: `blocks_add_when_rate_limited` in `src/tools/cron_add.rs`

#### Scenario: Medium-risk command requires approval
- WHEN a medium-risk shell command is provided
- THEN MUST require approval before creating
- Test: `medium_risk_shell_command_requires_approval` in `src/tools/cron_add.rs`

### REQ-CRON-001B: CronAddTool MUST validate agent job configuration

#### Scenario: Agent job requires prompt
- WHEN an agent job is submitted without a prompt
- THEN MUST return an error
- Test: `agent_job_requires_prompt` in `src/tools/cron_add.rs`

#### Scenario: Agent every schedule requires recurring confirmation
- WHEN an agent job with every-style schedule lacks confirmation
- THEN MUST require explicit confirmation
- Test: `agent_every_requires_recurring_confirmation` in `src/tools/cron_add.rs`

#### Scenario: Agent cron schedule requires recurring confirmation
- WHEN an agent job with cron-style schedule lacks confirmation
- THEN MUST require explicit confirmation
- Test: `agent_cron_requires_recurring_confirmation` in `src/tools/cron_add.rs`

#### Scenario: Agent every rejects high-frequency intervals
- WHEN an agent job has a very short every-interval
- THEN MUST reject it as too frequent
- Test: `agent_every_rejects_high_frequency_intervals` in `src/tools/cron_add.rs`

#### Scenario: Agent every with explicit confirmation succeeds
- WHEN an agent job with every-style schedule includes explicit confirmation
- THEN MUST create the job successfully
- Test: `agent_every_with_explicit_confirmation_succeeds` in `src/tools/cron_add.rs`

---

### Cron List (`src/tools/cron_list.rs`)

### REQ-CRON-002: CronListTool MUST list all cron jobs

#### Scenario: Empty list
- WHEN no cron jobs exist
- THEN MUST return empty list message
- Test: `returns_empty_list_when_no_jobs` in `src/tools/cron_list.rs`

#### Scenario: Disabled cron returns error
- WHEN cron is disabled in config
- THEN MUST return error
- Test: `errors_when_cron_disabled` in `src/tools/cron_list.rs`

---

### Cron Remove (`src/tools/cron_remove.rs`)

### REQ-CRON-003: CronRemoveTool MUST remove a cron job by ID

#### Scenario: Valid removal
- WHEN a valid job ID is provided
- THEN MUST remove the job and confirm
- Test: `removes_existing_job` in `src/tools/cron_remove.rs`

#### Scenario: Missing job ID
- WHEN no job ID is provided
- THEN MUST return error
- Test: `errors_when_job_id_missing` in `src/tools/cron_remove.rs`

### REQ-CRON-003A: CronRemoveTool MUST enforce security policies

#### Scenario: Read-only mode blocks removal
- WHEN read-only mode is active
- THEN MUST block the remove operation
- Test: `blocks_remove_in_read_only_mode` in `src/tools/cron_remove.rs`

#### Scenario: Rate limit blocks removal
- WHEN rate limit is exceeded
- THEN MUST block the remove operation
- Test: `blocks_remove_when_rate_limited` in `src/tools/cron_remove.rs`

---

### Cron Run (`src/tools/cron_run.rs`)

### REQ-CRON-004: CronRunTool MUST manually trigger a cron job execution

#### Scenario: Force run and record history
- WHEN a valid job ID is provided
- THEN MUST execute the job immediately and record the run
- Test: `force_runs_job_and_records_history` in `src/tools/cron_run.rs`

#### Scenario: Missing job returns error
- WHEN an unknown job ID is provided
- THEN MUST return error
- Test: `errors_for_missing_job` in `src/tools/cron_run.rs`

### REQ-CRON-004A: CronRunTool MUST enforce security policies

#### Scenario: Read-only mode blocks run
- WHEN read-only mode is active
- THEN MUST block the run operation
- Test: `blocks_run_in_read_only_mode` in `src/tools/cron_run.rs`

#### Scenario: Medium-risk shell run requires approval
- WHEN a medium-risk shell command is to be run
- THEN MUST require approval
- Test: `shell_run_requires_approval_for_medium_risk` in `src/tools/cron_run.rs`

#### Scenario: Rate limit blocks run
- WHEN rate limit is exceeded
- THEN MUST block the run operation
- Test: `blocks_run_when_rate_limited` in `src/tools/cron_run.rs`

---

### Cron Runs (`src/tools/cron_runs.rs`)

### REQ-CRON-005: CronRunsTool MUST list recent execution history

#### Scenario: List runs with truncation
- WHEN a valid job ID is provided
- THEN MUST return recent run results with truncated output
- Test: `lists_runs_with_truncation` in `src/tools/cron_runs.rs`

#### Scenario: Missing job ID
- WHEN no job ID is provided
- THEN MUST return error
- Test: `errors_when_job_id_missing` in `src/tools/cron_runs.rs`

---

### Cron Update (`src/tools/cron_update.rs`)

### REQ-CRON-006: CronUpdateTool MUST update cron job properties

#### Scenario: Update enabled flag
- WHEN valid update parameters are provided (e.g. enabled=false)
- THEN MUST update the job and confirm changes
- Test: `updates_enabled_flag` in `src/tools/cron_update.rs`

### REQ-CRON-006A: CronUpdateTool MUST enforce security policies

#### Scenario: Disallowed command update blocked
- WHEN an updated command is not in the allowlist
- THEN MUST block the update
- Test: `blocks_disallowed_command_updates` in `src/tools/cron_update.rs`

#### Scenario: Read-only mode blocks update
- WHEN read-only mode is active
- THEN MUST block the update operation
- Test: `blocks_mutation_in_read_only_mode` in `src/tools/cron_update.rs`

#### Scenario: Medium-risk shell update requires approval
- WHEN a medium-risk shell command update is provided
- THEN MUST require approval
- Test: `medium_risk_shell_update_requires_approval` in `src/tools/cron_update.rs`

#### Scenario: Rate limit blocks update
- WHEN rate limit is exceeded
- THEN MUST block the update operation
- Test: `blocks_update_when_rate_limited` in `src/tools/cron_update.rs`

---

### Schedule Tool (`src/tools/schedule.rs`)

### REQ-SCHED-001: ScheduleTool MUST provide correct identity

#### Scenario: Tool name and schema
- WHEN name() and parameters_schema() are called
- THEN MUST return "schedule" and valid schema
- Test: `tool_name_and_schema` in `src/tools/schedule.rs`

### REQ-SCHED-002: ScheduleTool MUST support list action

#### Scenario: Empty list
- WHEN no scheduled tasks exist
- THEN MUST return empty list
- Test: `list_empty` in `src/tools/schedule.rs`

### REQ-SCHED-003: ScheduleTool MUST support create/get/cancel lifecycle

#### Scenario: Full lifecycle
- WHEN a task is created, retrieved, and cancelled
- THEN MUST handle each step correctly with proper IDs and state transitions
- Test: `create_get_and_cancel_roundtrip` in `src/tools/schedule.rs`

### REQ-SCHED-004: ScheduleTool MUST support action aliases (once, pause, resume)

#### Scenario: Aliases work
- WHEN "once" (alias for create) and "pause"/"resume" actions are used
- THEN MUST behave identically to their canonical equivalents
- Test: `once_and_pause_resume_aliases_work` in `src/tools/schedule.rs`

### REQ-SCHED-005: ScheduleTool MUST enforce security policies

#### Scenario: Read-only blocks mutating actions
- WHEN read-only mode is active
- THEN MUST block create/cancel/pause/resume actions
- Test: `readonly_blocks_mutating_actions` in `src/tools/schedule.rs`

#### Scenario: Rate limit blocks create
- WHEN rate limit is exceeded
- THEN MUST block the create action
- Test: `rate_limit_blocks_create_action` in `src/tools/schedule.rs`

#### Scenario: Rate limit blocks cancel and preserves job
- WHEN rate limit is exceeded during cancel
- THEN MUST block the cancel and keep the job intact
- Test: `rate_limit_blocks_cancel_and_keeps_job` in `src/tools/schedule.rs`

### REQ-SCHED-006: ScheduleTool MUST handle edge cases

#### Scenario: Unknown action returns failure
- WHEN an unknown action is provided
- THEN MUST return failure result
- Test: `unknown_action_returns_failure` in `src/tools/schedule.rs`

#### Scenario: Mutating actions fail when cron disabled
- WHEN cron is disabled in config and a mutating action is attempted
- THEN MUST return failure
- Test: `mutating_actions_fail_when_cron_disabled` in `src/tools/schedule.rs`

### REQ-SCHED-007: ScheduleTool MUST validate shell commands

#### Scenario: Disallowed command blocked
- WHEN a disallowed shell command is scheduled
- THEN MUST block the creation
- Test: `create_blocks_disallowed_command` in `src/tools/schedule.rs`

#### Scenario: Medium-risk command requires approval
- WHEN a medium-risk shell command is scheduled
- THEN MUST require approval
- Test: `medium_risk_create_requires_approval` in `src/tools/schedule.rs`

## Mock Strategy

- Cron engine: SQLite via `tempfile::TempDir`
- SecurityPolicy: Constructed with test-specific settings (read-only, rate-limited, etc.)
- Config: Built with test defaults including `cron.enabled = true` and temp db_path

## Coverage Notes
- `src/tools/cron_add.rs`: 11 tests (creation, validation, security enforcement)
- `src/tools/cron_list.rs`: 2 tests (empty list, disabled cron)
- `src/tools/cron_remove.rs`: 4 tests (removal, missing ID, security)
- `src/tools/cron_run.rs`: 5 tests (force run, missing job, security)
- `src/tools/cron_runs.rs`: 2 tests (list runs, missing ID)
- `src/tools/cron_update.rs`: 5 tests (update, validation, security)
- `src/tools/schedule.rs`: 11 tests (lifecycle, aliases, security, edge cases)
- Total: 40 tests across 7 files
