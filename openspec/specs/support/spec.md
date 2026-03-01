# Support Modules Specification

## Purpose

Define requirements for the remaining support modules: SOP engine, skills, cron engine, economic tracking, coordination, auth, goals, tunnel, runtime, observability, plugins, approval, daemon, service management, and utilities.

## Scope

- Files: ~90 files across 20+ directories
- Risk tier: MIXED (LOW for utilities; MEDIUM for SOP/economic/coordination/auth; HIGH for runtime/tunnel)
- Existing tests: 712 across support modules (SOP: 164, Skills: 106, Cron: 60, Economic: 26, Coordination: 25, Auth: 33, Goals: 33, Tunnel: 48, Runtime: 82, Observability: 55, Plugins: 29, Approval: 34, Daemon: 25, Service: 17, Util: 14)

## Requirements

### REQ-SOP-001: SOP Engine Core

`SopEngine` MUST execute Standard Operating Procedures with step orchestration, condition evaluation, and gate approvals.

#### Scenario: Manual and MQTT trigger matching

- WHEN an SOP event is dispatched against registered SOPs
- THEN MUST match only SOPs whose trigger type and parameters correspond to the event
- Tests:
  - `match_manual_trigger` in `src/sop/engine.rs`
  - `no_match_for_wrong_source` in `src/sop/engine.rs`
  - `match_mqtt_trigger_exact` in `src/sop/engine.rs`
  - `match_mqtt_wildcard_plus` in `src/sop/engine.rs`
  - `match_mqtt_wildcard_hash` in `src/sop/engine.rs`
  - `mqtt_topic_matching_edge_cases` in `src/sop/engine.rs`

#### Scenario: Webhook and cron trigger matching

- WHEN webhook or cron events arrive
- THEN MUST match only SOPs with the correct path or cron expression
- Tests:
  - `webhook_trigger_matches_exact_path` in `src/sop/engine.rs`
  - `webhook_trigger_rejects_different_path` in `src/sop/engine.rs`
  - `cron_trigger_matches_only_matching_expression` in `src/sop/engine.rs`

#### Scenario: MQTT and peripheral condition filtering

- WHEN an SOP trigger includes a condition
- THEN MUST evaluate conditions against event payload and accept or reject accordingly
- Tests:
  - `mqtt_condition_filters_by_payload` in `src/sop/engine.rs`
  - `mqtt_no_condition_matches_any_payload` in `src/sop/engine.rs`
  - `mqtt_condition_no_payload_fails_closed` in `src/sop/engine.rs`
  - `peripheral_condition_filters_by_payload` in `src/sop/engine.rs`
  - `peripheral_no_condition_matches_any` in `src/sop/engine.rs`

#### Scenario: SOP run lifecycle (start, advance, complete, cancel)

- WHEN a run is started for a matched SOP
- THEN MUST create a run, advance through steps, and complete or cancel correctly
- Tests:
  - `start_run_returns_first_step` in `src/sop/engine.rs`
  - `start_run_unknown_sop_fails` in `src/sop/engine.rs`
  - `advance_step_to_completion` in `src/sop/engine.rs`
  - `step_failure_ends_run` in `src/sop/engine.rs`
  - `cancel_run` in `src/sop/engine.rs`
  - `cancel_unknown_run_fails` in `src/sop/engine.rs`
  - `get_run_finds_active_and_finished` in `src/sop/engine.rs`

#### Scenario: Concurrency and cooldown limits

- WHEN multiple runs are requested
- THEN MUST enforce per-SOP and global concurrency limits and cooldown periods
- Tests:
  - `per_sop_concurrency_limit` in `src/sop/engine.rs`
  - `global_concurrency_limit` in `src/sop/engine.rs`
  - `cooldown_blocks_immediate_restart` in `src/sop/engine.rs`

#### Scenario: Execution modes (auto, supervised, step-by-step, priority-based)

- WHEN an SOP is configured with a specific execution mode
- THEN MUST execute steps accordingly (auto, wait for approval, or require per-step confirmation)
- Tests:
  - `auto_mode_executes_immediately` in `src/sop/engine.rs`
  - `supervised_mode_waits_on_first_step` in `src/sop/engine.rs`
  - `step_by_step_waits_on_every_step` in `src/sop/engine.rs`
  - `priority_based_critical_auto` in `src/sop/engine.rs`
  - `priority_based_normal_supervised` in `src/sop/engine.rs`
  - `requires_confirmation_overrides_auto` in `src/sop/engine.rs`

#### Scenario: Approval workflow

- WHEN a step requires approval
- THEN MUST transition to waiting state and advance on approval
- Tests:
  - `approve_transitions_to_execute` in `src/sop/engine.rs`
  - `approve_non_waiting_fails` in `src/sop/engine.rs`
  - `waiting_since_set_on_wait_approval` in `src/sop/engine.rs`
  - `waiting_since_cleared_on_approve` in `src/sop/engine.rs`

#### Scenario: Approval timeout auto-approval

- WHEN approval timeout elapses
- THEN MUST auto-approve critical SOPs and leave normal SOPs waiting
- Tests:
  - `timeout_auto_approves_critical` in `src/sop/engine.rs`
  - `timeout_does_not_auto_approve_normal` in `src/sop/engine.rs`
  - `timeout_zero_disables_check` in `src/sop/engine.rs`

#### Scenario: Step context and finished run management

- WHEN step context is generated or runs finish
- THEN MUST include SOP name and step info, and evict oldest finished runs when over limit
- Tests:
  - `step_context_includes_sop_name_and_step` in `src/sop/engine.rs`
  - `max_finished_runs_evicts_oldest` in `src/sop/engine.rs`
  - `max_finished_runs_zero_means_unlimited` in `src/sop/engine.rs`

#### Scenario: ISO 8601 timestamp handling

- WHEN timestamps are generated or parsed
- THEN MUST produce valid ISO 8601 and round-trip correctly
- Tests:
  - `iso8601_roundtrip` in `src/sop/engine.rs`
  - `parse_known_timestamp` in `src/sop/engine.rs`

### REQ-SOP-002: SOP Conditions

Condition evaluation MUST support boolean logic, value comparison, and context-aware branching.

#### Scenario: Empty and missing payload conditions

- WHEN conditions are empty or payload is missing
- THEN MUST match on empty condition and fail closed on missing payload
- Tests:
  - `empty_condition_matches` in `src/sop/condition.rs`
  - `missing_payload_fails_closed` in `src/sop/condition.rs`

#### Scenario: JSON path comparison operators

- WHEN conditions use JSON path with comparison operators (gt, gte, lt, lte, eq, neq)
- THEN MUST evaluate correctly against extracted values
- Tests:
  - `json_path_gt` in `src/sop/condition.rs`
  - `json_path_gte` in `src/sop/condition.rs`
  - `json_path_lt` in `src/sop/condition.rs`
  - `json_path_lte` in `src/sop/condition.rs`
  - `json_path_eq` in `src/sop/condition.rs`
  - `json_path_neq` in `src/sop/condition.rs`
  - `json_path_numeric_eq` in `src/sop/condition.rs`

#### Scenario: JSON path traversal (nested, array, bool, missing key, invalid payload)

- WHEN conditions traverse nested JSON, arrays, booleans, or encounter missing keys
- THEN MUST resolve paths correctly or return false on failure
- Tests:
  - `json_nested_path` in `src/sop/condition.rs`
  - `json_path_missing_key` in `src/sop/condition.rs`
  - `json_invalid_payload` in `src/sop/condition.rs`
  - `json_path_array_index` in `src/sop/condition.rs`
  - `json_path_bool_value` in `src/sop/condition.rs`

#### Scenario: Direct (non-path) conditions

- WHEN conditions compare raw payload values directly
- THEN MUST evaluate numeric and string comparisons correctly
- Tests:
  - `direct_gt` in `src/sop/condition.rs`
  - `direct_gte` in `src/sop/condition.rs`
  - `direct_lt` in `src/sop/condition.rs`
  - `direct_eq` in `src/sop/condition.rs`
  - `direct_neq` in `src/sop/condition.rs`
  - `direct_non_numeric_payload` in `src/sop/condition.rs`
  - `direct_float_comparison` in `src/sop/condition.rs`

#### Scenario: Operator and path parsing internals

- WHEN parsing operator-value pairs and path segments
- THEN MUST extract correct operators, values, and path segments
- Tests:
  - `parse_op_value_basic` in `src/sop/condition.rs`
  - `parse_op_value_gte_not_gt` in `src/sop/condition.rs`
  - `parse_op_value_no_value` in `src/sop/condition.rs`
  - `parse_path_op_value_basic` in `src/sop/condition.rs`
  - `parse_path_op_value_nested` in `src/sop/condition.rs`
  - `parse_path_op_value_string_comparand` in `src/sop/condition.rs`

#### Scenario: JSON path resolution

- WHEN resolving JSON paths against a value
- THEN MUST resolve simple, nested, and missing paths correctly
- Tests:
  - `resolve_path_simple` in `src/sop/condition.rs`
  - `resolve_path_nested` in `src/sop/condition.rs`
  - `resolve_path_missing` in `src/sop/condition.rs`

### REQ-SOP-003: SOP Metrics

`SopMetricsCollector` MUST track execution metrics and performance data.

#### Scenario: Baseline and counter state

- WHEN the metrics collector is freshly initialized
- THEN MUST report zero-state baseline for all metrics
- Tests:
  - `zero_state_baseline` in `src/sop/metrics.rs`
  - `counter_arithmetic` in `src/sop/metrics.rs`

#### Scenario: Windowed and derived metric filtering

- WHEN metrics are queried with a time window
- THEN MUST filter runs by timestamp and compute derived rates correctly
- Tests:
  - `windowed_filtering` in `src/sop/metrics.rs`
  - `windowed_excludes_old_runs` in `src/sop/metrics.rs`
  - `derived_rate_metrics` in `src/sop/metrics.rs`
  - `get_metric_windowed_7d_matches_suffix` in `src/sop/metrics.rs`
  - `get_metric_windowed_custom_duration` in `src/sop/metrics.rs`
  - `get_metric_provider_window_propagation` in `src/sop/metrics.rs`

#### Scenario: Protocol adherence and deviation rates

- WHEN SOP runs complete with various step outcomes
- THEN MUST compute protocol adherence and deviation rates accurately
- Tests:
  - `deviation_rate_zero_steps` in `src/sop/metrics.rs`
  - `protocol_adherence_rate_partial_run` in `src/sop/metrics.rs`
  - `protocol_adherence_rate_full_run` in `src/sop/metrics.rs`
  - `protocol_adherence_rate_failed_run` in `src/sop/metrics.rs`

#### Scenario: Per-SOP lookup and metric disambiguation

- WHEN metrics are queried by SOP name
- THEN MUST resolve the correct per-SOP counters and disambiguate longest match
- Tests:
  - `per_sop_lookup` in `src/sop/metrics.rs`
  - `longest_match_disambiguation` in `src/sop/metrics.rs`
  - `not_found_for_unknown_metric` in `src/sop/metrics.rs`
  - `sop_name_matching_metric_suffix_resolves_global` in `src/sop/metrics.rs`

#### Scenario: Approval and pending approval tracking

- WHEN approvals or timeout auto-approvals occur
- THEN MUST propagate flags, track pending state, and evict stale entries
- Tests:
  - `approval_flag_propagation` in `src/sop/metrics.rs`
  - `pending_approval_stale_eviction` in `src/sop/metrics.rs`
  - `multiple_approvals_per_run_consistent` in `src/sop/metrics.rs`

#### Scenario: Diagnostic snapshot and ring buffer

- WHEN a diagnostic snapshot is requested or the ring buffer overflows
- THEN MUST produce valid output and cap buffer size
- Tests:
  - `snapshot_diagnostic_output` in `src/sop/metrics.rs`
  - `runs_cancelled_tracking` in `src/sop/metrics.rs`
  - `ring_buffer_overflow_cap` in `src/sop/metrics.rs`

#### Scenario: MetricsProvider trait implementation

- WHEN the collector is used as a MetricsProvider
- THEN MUST respond to metric queries correctly
- Tests:
  - `metrics_provider_get_metric` in `src/sop/metrics.rs`

#### Scenario: Warm start from memory

- WHEN the metrics collector rebuilds from persisted memory
- THEN MUST restore counters and handle edge cases (running runs, empty memory, approval matching)
- Tests:
  - `warm_start_roundtrip` in `src/sop/metrics.rs`
  - `warm_start_skips_running_runs` in `src/sop/metrics.rs`
  - `warm_start_empty_memory` in `src/sop/metrics.rs`
  - `warm_start_approval_matching` in `src/sop/metrics.rs`
  - `warm_start_preserves_pending_for_nonterminal_runs` in `src/sop/metrics.rs`

### REQ-SOP-004: SOP Types

SOP types MUST serialize/deserialize correctly and display meaningful representations.

#### Scenario: Type display and serde roundtrips

- WHEN SOP types are displayed or serialized
- THEN MUST produce correct string representations and survive roundtrips
- Tests:
  - `priority_display` in `src/sop/types.rs`
  - `execution_mode_display` in `src/sop/types.rs`
  - `trigger_display` in `src/sop/types.rs`
  - `priority_serde_roundtrip` in `src/sop/types.rs`
  - `execution_mode_serde_roundtrip` in `src/sop/types.rs`
  - `trigger_toml_roundtrip` in `src/sop/types.rs`
  - `trigger_manual_toml` in `src/sop/types.rs`
  - `run_status_display` in `src/sop/types.rs`
  - `step_defaults` in `src/sop/types.rs`
  - `manifest_parse` in `src/sop/types.rs`
  - `trigger_source_display` in `src/sop/types.rs`
  - `step_status_display` in `src/sop/types.rs`
  - `sop_event_serde_roundtrip` in `src/sop/types.rs`
  - `sop_run_serde_roundtrip` in `src/sop/types.rs`

### REQ-SOP-005: SOP Loading and Parsing

SOP loading MUST discover, parse, and validate SOPs from the filesystem.

#### Scenario: Step parsing from markdown

- WHEN markdown content is parsed for SOP steps
- THEN MUST extract step definitions with titles, bodies, and conditions
- Tests:
  - `parse_steps_basic` in `src/sop/mod.rs`
  - `parse_steps_empty_md` in `src/sop/mod.rs`
  - `parse_steps_no_bold_title` in `src/sop/mod.rs`
  - `parse_steps_multiline_body` in `src/sop/mod.rs`

#### Scenario: SOP directory loading

- WHEN SOP directories are scanned
- THEN MUST load valid SOPs, handle missing/empty directories, and apply config defaults
- Tests:
  - `load_sop_from_directory` in `src/sop/mod.rs`
  - `load_sops_empty_dir` in `src/sop/mod.rs`
  - `load_sops_nonexistent_dir` in `src/sop/mod.rs`
  - `load_sop_toml_only_no_md` in `src/sop/mod.rs`
  - `load_sop_uses_config_default_execution_mode_when_omitted` in `src/sop/mod.rs`

#### Scenario: SOP validation and directory resolution

- WHEN SOPs are validated or directory paths resolved
- THEN MUST report warnings for invalid SOPs and resolve paths correctly
- Tests:
  - `validate_sop_warnings` in `src/sop/mod.rs`
  - `validate_sop_clean` in `src/sop/mod.rs`
  - `resolve_sops_dir_default` in `src/sop/mod.rs`
  - `resolve_sops_dir_override` in `src/sop/mod.rs`

#### Scenario: Markdown title extraction and trigger parsing

- WHEN bold titles and trigger types are extracted from SOP content
- THEN MUST parse correctly or return None for invalid input
- Tests:
  - `extract_bold_title_with_dash` in `src/sop/mod.rs`
  - `extract_bold_title_no_separator` in `src/sop/mod.rs`
  - `extract_bold_title_none` in `src/sop/mod.rs`
  - `parse_all_trigger_types` in `src/sop/mod.rs`

### REQ-SOP-006: SOP Gates

`GateEvaluator` MUST evaluate phase gates with metric-based conditions and persist state.

#### Scenario: Gate tick evaluation

- WHEN gates are ticked with current metrics
- THEN MUST return correct decisions (pass, pending, no-op) and advance phase state
- Tests:
  - `tick_no_gates_returns_none` in `src/sop/gates.rs`
  - `tick_with_passing_gate_returns_decision` in `src/sop/gates.rs`
  - `tick_transition_advances_phase` in `src/sop/gates.rs`
  - `tick_observed_no_state_change` in `src/sop/gates.rs`
  - `tick_pending_human_sets_pending` in `src/sop/gates.rs`
  - `tick_respects_interval` in `src/sop/gates.rs`

#### Scenario: Gate loading from file

- WHEN gate definitions are loaded from persona files
- THEN MUST parse valid gates and handle missing/invalid files gracefully
- Tests:
  - `load_gates_missing_file_returns_empty` in `src/sop/gates.rs`
  - `load_gates_valid_persona` in `src/sop/gates.rs`
  - `load_gates_no_gates_key_returns_empty` in `src/sop/gates.rs`
  - `load_gates_invalid_json_returns_empty` in `src/sop/gates.rs`

#### Scenario: Gate warm start from memory

- WHEN the gate evaluator restores state from memory
- THEN MUST reconstruct state correctly and handle empty memory
- Tests:
  - `warm_start_roundtrip` in `src/sop/gates.rs`
  - `warm_start_empty_memory` in `src/sop/gates.rs`

#### Scenario: Gate priority and idempotency

- WHEN demote and promote gates conflict or ticks repeat
- THEN MUST prioritize demote and produce idempotent results
- Tests:
  - `demote_priority_over_promote` in `src/sop/gates.rs`
  - `idempotent_tick_after_apply` in `src/sop/gates.rs`

#### Scenario: Gate integration with real metrics collector

- WHEN gates are ticked with a real `SopMetricsCollector`
- THEN MUST evaluate conditions against live metric data
- Tests:
  - `gate_tick_with_real_collector` in `src/sop/gates.rs`

#### Scenario: Ampersona decision string stability

- WHEN decision records are serialized
- THEN MUST produce stable string representations
- Tests:
  - `ampersona_decision_strings_stable` in `src/sop/gates.rs`

### REQ-SOP-007: SOP Dispatch

SOP dispatch MUST route events to matching SOPs and handle batch execution.

#### Scenario: Event dispatch lifecycle

- WHEN SOP events are dispatched
- THEN MUST start matching SOPs, skip cooldown-blocked SOPs, and handle unknown events
- Tests:
  - `dispatch_starts_matching_sop` in `src/sop/dispatch.rs`
  - `dispatch_skips_when_cooldown_active` in `src/sop/dispatch.rs`
  - `dispatch_returns_no_match_for_unknown_event` in `src/sop/dispatch.rs`
  - `dispatch_batch_lock_starts_multiple_sops` in `src/sop/dispatch.rs`

#### Scenario: Dispatch action capture

- WHEN dispatch produces wait-approval or execute-step actions
- THEN MUST capture the correct action type in the result
- Tests:
  - `dispatch_captures_action_for_wait_approval` in `src/sop/dispatch.rs`
  - `dispatch_captures_action_for_execute_step` in `src/sop/dispatch.rs`

#### Scenario: Peripheral signal dispatch

- WHEN peripheral signals are received
- THEN MUST dispatch to matching SOPs or return empty for non-matches
- Tests:
  - `peripheral_signal_dispatches_to_matching_sop` in `src/sop/dispatch.rs`
  - `peripheral_signal_no_match_returns_empty` in `src/sop/dispatch.rs`

#### Scenario: Cron-based SOP scheduling

- WHEN cron expressions are cached and evaluated
- THEN MUST parse valid expressions, skip invalid ones, and fire triggers on schedule
- Tests:
  - `cron_cache_skips_invalid_expression` in `src/sop/dispatch.rs`
  - `cron_cache_parses_valid_expression` in `src/sop/dispatch.rs`
  - `cron_sop_trigger_fires_on_schedule` in `src/sop/dispatch.rs`
  - `cron_sop_only_matching_expression_fires` in `src/sop/dispatch.rs`
  - `cron_sop_window_check_does_not_miss_tick` in `src/sop/dispatch.rs`

### REQ-SOP-008: SOP Audit

SOP audit MUST log execution history with timestamps and outcomes.

#### Scenario: Audit log persistence and retrieval

- WHEN SOP runs, steps, approvals, and timeout auto-approvals are logged
- THEN MUST persist to memory and retrieve correctly
- Tests:
  - `audit_roundtrip` in `src/sop/audit.rs`
  - `log_approval_persists_entry` in `src/sop/audit.rs`
  - `log_timeout_auto_approve_persists_entry` in `src/sop/audit.rs`
  - `get_nonexistent_run_returns_none` in `src/sop/audit.rs`

### REQ-SKILL-001: Skill System

`Skill` and `SkillTool` MUST load skill manifests and execute skill-defined tools.

#### Scenario: Skill loading from directories

- WHEN skill directories and manifests are scanned
- THEN MUST load skills from TOML and markdown, handle empty/nonexistent directories, and prioritize TOML over markdown
- Tests:
  - `load_empty_skills_dir` in `src/skills/mod.rs`
  - `load_skill_from_toml` in `src/skills/mod.rs`
  - `load_skill_from_md` in `src/skills/mod.rs`
  - `load_nonexistent_dir` in `src/skills/mod.rs`
  - `load_ignores_files_in_skills_dir` in `src/skills/mod.rs`
  - `load_ignores_dir_without_manifest` in `src/skills/mod.rs`
  - `load_multiple_skills` in `src/skills/mod.rs`
  - `toml_prefers_over_md` in `src/skills/mod.rs`

#### Scenario: TOML skill parsing

- WHEN TOML skill manifests are parsed
- THEN MUST handle multiple tools, minimal manifests, and invalid syntax
- Tests:
  - `toml_skill_with_multiple_tools` in `src/skills/mod.rs`
  - `toml_skill_minimal` in `src/skills/mod.rs`
  - `toml_skill_invalid_syntax_skipped` in `src/skills/mod.rs`

#### Scenario: Markdown skill parsing

- WHEN markdown skill files are parsed
- THEN MUST extract metadata from front matter and heading-only content
- Tests:
  - `md_skill_heading_only` in `src/skills/mod.rs`
  - `load_skill_md_front_matter_overrides_metadata_and_description` in `src/skills/mod.rs`

#### Scenario: Skills prompt generation

- WHEN skills are rendered to prompt text
- THEN MUST include tools, escape XML, and support compact mode
- Tests:
  - `skills_to_prompt_empty` in `src/skills/mod.rs`
  - `skills_to_prompt_with_skills` in `src/skills/mod.rs`
  - `skills_to_prompt_compact_mode_omits_instructions_and_tools` in `src/skills/mod.rs`
  - `skills_to_prompt_compact_mode_includes_always_skill_instructions_and_tools` in `src/skills/mod.rs`
  - `skills_to_prompt_includes_tools` in `src/skills/mod.rs`
  - `skills_to_prompt_escapes_xml_content` in `src/skills/mod.rs`

#### Scenario: Skills directory initialization

- WHEN the skills directory is initialized
- THEN MUST create the readme and be idempotent
- Tests:
  - `init_skills_creates_readme` in `src/skills/mod.rs`
  - `init_skills_idempotent` in `src/skills/mod.rs`
  - `skills_dir_path` in `src/skills/mod.rs`

#### Scenario: Git source detection

- WHEN skill sources are evaluated for git protocol
- THEN MUST accept remote protocols/SCP-style and reject local paths
- Tests:
  - `git_source_detection_accepts_remote_protocols_and_scp_style` in `src/skills/mod.rs`
  - `git_source_detection_rejects_local_paths_and_invalid_inputs` in `src/skills/mod.rs`

#### Scenario: Open Skills configuration

- WHEN open skills are configured via env or config
- THEN MUST resolve enabled state and directory correctly
- Tests:
  - `open_skills_enabled_resolution_prefers_env_then_config_then_default_false` in `src/skills/mod.rs`
  - `resolve_open_skills_dir_resolution_prefers_env_then_config_then_home` in `src/skills/mod.rs`
  - `load_skills_with_config_reads_open_skills_dir_without_network` in `src/skills/mod.rs`

#### Scenario: Registry source detection

- WHEN skill sources are evaluated for registry format
- THEN MUST accept valid namespace/name and reject local paths, git URLs, and invalid formats
- Tests:
  - `registry_install_dir_name_is_package_name_only` in `src/skills/mod.rs`
  - `is_registry_source_accepts_valid_namespace_name` in `src/skills/mod.rs`
  - `is_registry_source_rejects_local_path_prefixes` in `src/skills/mod.rs`
  - `is_registry_source_rejects_git_urls_and_http_schemes` in `src/skills/mod.rs`
  - `is_registry_source_rejects_invalid_formats` in `src/skills/mod.rs`

#### Scenario: Skill scaffolding

- WHEN a new skill is scaffolded
- THEN MUST validate name, create required files for each template, and substitute placeholders
- Tests:
  - `scaffold_skill_rejects_traversal_in_name` in `src/skills/mod.rs`
  - `scaffold_skill_rejects_slash_in_name` in `src/skills/mod.rs`
  - `scaffold_skill_rejects_space_in_name` in `src/skills/mod.rs`
  - `scaffold_skill_rejects_empty_name` in `src/skills/mod.rs`
  - `scaffold_skill_rejects_unknown_template` in `src/skills/mod.rs`
  - `scaffold_skill_rejects_existing_directory` in `src/skills/mod.rs`
  - `scaffold_skill_typescript_creates_required_files` in `src/skills/mod.rs`
  - `scaffold_skill_rust_creates_required_files` in `src/skills/mod.rs`
  - `scaffold_skill_substitutes_name_placeholder` in `src/skills/mod.rs`
  - `scaffold_skill_go_creates_required_files` in `src/skills/mod.rs`
  - `scaffold_skill_gitignore_always_created` in `src/skills/mod.rs`
  - `scaffold_skill_skill_md_contains_name` in `src/skills/mod.rs`

#### Scenario: ClawHub and zip URL source detection

- WHEN skill sources are evaluated for ClawHub or zip URL format
- THEN MUST accept valid formats and reject invalid ones
- Tests:
  - `is_clawhub_source_accepts_profile_url` in `src/skills/mod.rs`
  - `is_clawhub_source_accepts_short_prefix` in `src/skills/mod.rs`
  - `is_clawhub_source_rejects_other_domains` in `src/skills/mod.rs`
  - `clawhub_download_url_from_profile_url` in `src/skills/mod.rs`
  - `clawhub_download_url_from_single_path_url` in `src/skills/mod.rs`
  - `clawhub_download_url_from_short_prefix` in `src/skills/mod.rs`
  - `clawhub_download_url_rejects_slash_in_prefix_slug` in `src/skills/mod.rs`
  - `is_zip_url_source_accepts_explicit_prefix` in `src/skills/mod.rs`
  - `is_zip_url_source_accepts_direct_zip_url` in `src/skills/mod.rs`
  - `is_zip_url_source_rejects_non_zip_https` in `src/skills/mod.rs`
  - `is_zip_url_source_rejects_non_https` in `src/skills/mod.rs`
  - `is_zip_url_source_rejects_other_formats` in `src/skills/mod.rs`

#### Scenario: Skill name normalization

- WHEN skill names are normalized
- THEN MUST convert hyphens, lowercase, and strip non-alnum
- Tests:
  - `normalize_skill_name_converts_hyphens_and_lowercases` in `src/skills/mod.rs`
  - `normalize_skill_name_strips_non_alnum` in `src/skills/mod.rs`

### REQ-SKILL-002: Skill Tool Handler

`SkillToolHandler` MUST execute shell, HTTP, and script tools from skill definitions.

#### Scenario: Placeholder extraction and parameter inference

- WHEN tool commands contain placeholders
- THEN MUST extract, deduplicate, and infer parameter types
- Tests:
  - `extract_placeholders_from_command` in `src/skills/tool_handler.rs`
  - `extract_placeholders_deduplicates` in `src/skills/tool_handler.rs`
  - `infer_integer_type` in `src/skills/tool_handler.rs`
  - `infer_boolean_type` in `src/skills/tool_handler.rs`
  - `infer_string_type_default` in `src/skills/tool_handler.rs`

#### Scenario: Schema generation and command rendering

- WHEN tool parameters are defined
- THEN MUST generate correct JSON schema and render commands with argument substitution
- Tests:
  - `generate_schema_with_parameters` in `src/skills/tool_handler.rs`
  - `render_command_with_all_args` in `src/skills/tool_handler.rs`
  - `render_command_with_optional_params_omitted` in `src/skills/tool_handler.rs`
  - `render_command_removes_optional_flags_with_dashes` in `src/skills/tool_handler.rs`
  - `render_command_quotes_numeric_strings` in `src/skills/tool_handler.rs`

#### Scenario: Shell escape injection prevention

- WHEN user-provided arguments are substituted into commands
- THEN MUST escape shell metacharacters to prevent injection
- Tests:
  - `shell_escape_prevents_injection` in `src/skills/tool_handler.rs`

### REQ-SKILL-003: Skill Audit

Skill audit MUST validate skill directories and zip archives for safety.

#### Scenario: Safe and unsafe skill directory validation

- WHEN skill directories are audited
- THEN MUST accept safe skills and reject shell scripts, escape links, high-risk patterns, and injection patterns
- Tests:
  - `audit_accepts_safe_skill` in `src/skills/audit.rs`
  - `audit_rejects_shell_script_files` in `src/skills/audit.rs`
  - `audit_allows_shell_script_files_when_enabled` in `src/skills/audit.rs`
  - `audit_rejects_markdown_escape_links` in `src/skills/audit.rs`
  - `audit_rejects_high_risk_patterns` in `src/skills/audit.rs`
  - `audit_rejects_prompt_injection_override_patterns` in `src/skills/audit.rs`
  - `audit_rejects_phishing_secret_harvest_patterns` in `src/skills/audit.rs`
  - `audit_rejects_obfuscated_backdoor_patterns` in `src/skills/audit.rs`
  - `audit_rejects_chained_commands_in_manifest` in `src/skills/audit.rs`

#### Scenario: Cross-skill reference handling

- WHEN markdown links reference files across skill boundaries
- THEN MUST allow cross-skill references and reject missing local files
- Tests:
  - `audit_allows_missing_cross_skill_reference_with_parent_dir` in `src/skills/audit.rs`
  - `audit_allows_missing_cross_skill_reference_with_bare_filename` in `src/skills/audit.rs`
  - `audit_allows_missing_cross_skill_reference_with_dot_slash` in `src/skills/audit.rs`
  - `audit_rejects_missing_local_markdown_file` in `src/skills/audit.rs`
  - `audit_allows_existing_cross_skill_reference` in `src/skills/audit.rs`
  - `is_cross_skill_reference_detection` in `src/skills/audit.rs`

#### Scenario: Zip archive skill audit

- WHEN zip skill archives are audited
- THEN MUST accept clean markdown, reject path traversal, absolute paths, native binaries, and high-risk content
- Tests:
  - `zip_audit_accepts_clean_skill_md` in `src/skills/audit.rs`
  - `zip_audit_rejects_path_traversal` in `src/skills/audit.rs`
  - `zip_audit_rejects_absolute_unix_path` in `src/skills/audit.rs`
  - `zip_audit_rejects_native_binary_exe` in `src/skills/audit.rs`
  - `zip_audit_rejects_native_binary_dll` in `src/skills/audit.rs`
  - `zip_audit_allows_wasm_file` in `src/skills/audit.rs`
  - `zip_audit_rejects_high_risk_shell_in_md` in `src/skills/audit.rs`
  - `zip_audit_rejects_high_risk_shell_in_js` in `src/skills/audit.rs`
  - `zip_audit_accepts_meta_json` in `src/skills/audit.rs`

### REQ-SKILL-004: Skill Templates

Skill templates MUST provide scaffolding for multiple languages.

#### Scenario: Template lookup and application

- WHEN templates are searched by name or language alias
- THEN MUST find correct template or return None for unknowns, and apply substitutions
- Tests:
  - `find_by_exact_name` in `src/skills/templates.rs`
  - `find_by_language_alias` in `src/skills/templates.rs`
  - `find_typescript_aliases` in `src/skills/templates.rs`
  - `find_python_aliases` in `src/skills/templates.rs`
  - `find_unknown_returns_none` in `src/skills/templates.rs`
  - `all_templates_have_files` in `src/skills/templates.rs`
  - `apply_substitutions` in `src/skills/templates.rs`
  - `apply_no_substitutions` in `src/skills/templates.rs`
  - `all_templates_count` in `src/skills/templates.rs`
  - `find_go_template` in `src/skills/templates.rs`

### REQ-SKILL-005: Skill Symlink Security

Skill loading MUST enforce trusted symlink roots for workspace skills.

#### Scenario: Symlink edge cases and trust boundaries

- WHEN workspace skills contain symlinks
- THEN MUST enforce trusted root policies and reject untrusted symlinks
- Tests:
  - `test_skills_symlink_unix_edge_cases` in `src/skills/symlink_tests.rs`
  - `test_workspace_symlink_loading_requires_trusted_roots` in `src/skills/symlink_tests.rs`
  - `test_skills_audit_respects_trusted_symlink_roots` in `src/skills/symlink_tests.rs`

### REQ-CRON-001: Cron Scheduler

`CronScheduler` MUST manage job execution with cron expression scheduling, retry logic, and delivery.

#### Scenario: Job command execution

- WHEN shell jobs are executed
- THEN MUST run successfully, report failures, enforce timeouts, and block disallowed commands
- Tests:
  - `run_job_command_success` in `src/cron/scheduler.rs`
  - `run_job_command_failure` in `src/cron/scheduler.rs`
  - `run_job_command_times_out` in `src/cron/scheduler.rs`
  - `run_job_command_blocks_disallowed_command` in `src/cron/scheduler.rs`
  - `run_job_command_blocks_forbidden_path_argument` in `src/cron/scheduler.rs`
  - `run_job_command_blocks_forbidden_option_assignment_path_argument` in `src/cron/scheduler.rs`
  - `run_job_command_blocks_forbidden_short_option_attached_path_argument` in `src/cron/scheduler.rs`
  - `run_job_command_blocks_tilde_user_path_argument` in `src/cron/scheduler.rs`
  - `run_job_command_blocks_input_redirection_path_bypass` in `src/cron/scheduler.rs`
  - `run_job_command_blocks_readonly_mode` in `src/cron/scheduler.rs`
  - `run_job_command_blocks_rate_limited` in `src/cron/scheduler.rs`

#### Scenario: Job retry logic

- WHEN a job fails on first attempt
- THEN MUST retry and recover, or exhaust retry attempts
- Tests:
  - `execute_job_with_retry_recovers_after_first_failure` in `src/cron/scheduler.rs`
  - `execute_job_with_retry_exhausts_attempts` in `src/cron/scheduler.rs`

#### Scenario: Agent job execution

- WHEN agent-type jobs are executed
- THEN MUST require provider key, block readonly mode, and block rate-limited state
- Tests:
  - `run_agent_job_returns_error_without_provider_key` in `src/cron/scheduler.rs`
  - `run_agent_job_blocks_readonly_mode` in `src/cron/scheduler.rs`
  - `run_agent_job_blocks_rate_limited` in `src/cron/scheduler.rs`

#### Scenario: Job processing and state management

- WHEN due jobs are processed
- THEN MUST mark component health, persist results, reschedule, and handle one-shot jobs
- Tests:
  - `process_due_jobs_marks_component_ok_even_when_idle` in `src/cron/scheduler.rs`
  - `process_due_jobs_failure_does_not_mark_component_unhealthy` in `src/cron/scheduler.rs`
  - `persist_job_result_records_run_and_reschedules_shell_job` in `src/cron/scheduler.rs`
  - `persist_job_result_success_deletes_one_shot` in `src/cron/scheduler.rs`
  - `persist_job_result_failure_disables_one_shot` in `src/cron/scheduler.rs`
  - `persist_job_result_success_deletes_one_shot_shell_job` in `src/cron/scheduler.rs`
  - `persist_job_result_failure_disables_one_shot_shell_job` in `src/cron/scheduler.rs`
  - `persist_job_result_at_schedule_without_delete_after_run_is_not_deleted` in `src/cron/scheduler.rs`

#### Scenario: Delivery handling

- WHEN job output is delivered
- THEN MUST handle missing/invalid channels, skip no-reply sentinel, and validate channel requirements
- Tests:
  - `persist_job_result_delivery_failure_non_best_effort_marks_error` in `src/cron/scheduler.rs`
  - `persist_job_result_delivery_failure_best_effort_keeps_success` in `src/cron/scheduler.rs`
  - `deliver_if_configured_handles_none_and_invalid_channel` in `src/cron/scheduler.rs`
  - `deliver_if_configured_skips_no_reply_sentinel` in `src/cron/scheduler.rs`
  - `no_reply_sentinel_matching_is_trimmed_and_case_insensitive` in `src/cron/scheduler.rs`
  - `deliver_if_configured_whatsapp_web_requires_live_session_in_web_mode` in `src/cron/scheduler.rs`

### REQ-CRON-002: Cron Store

`CronStore` MUST persist job definitions and execution history.

#### Scenario: Job CRUD operations

- WHEN jobs are created, listed, and removed
- THEN MUST persist and retrieve correctly with proper expression parsing
- Tests:
  - `add_job_accepts_five_field_expression` in `src/cron/store.rs`
  - `add_shell_job_marks_at_schedule_for_auto_delete` in `src/cron/store.rs`
  - `add_list_remove_roundtrip` in `src/cron/store.rs`

#### Scenario: Due job filtering and scheduling

- WHEN due jobs are queried
- THEN MUST filter by timestamp, enabled state, and respect max tasks limit
- Tests:
  - `due_jobs_filters_by_timestamp_and_enabled` in `src/cron/store.rs`
  - `due_jobs_respects_scheduler_max_tasks_limit` in `src/cron/store.rs`

#### Scenario: Rescheduling and run history

- WHEN jobs are rescheduled after execution
- THEN MUST persist last status, last run, and truncate large output
- Tests:
  - `reschedule_after_run_persists_last_status_and_last_run` in `src/cron/store.rs`
  - `record_and_prune_runs` in `src/cron/store.rs`
  - `remove_job_cascades_run_history` in `src/cron/store.rs`
  - `record_run_truncates_large_output` in `src/cron/store.rs`
  - `reschedule_after_run_truncates_last_output` in `src/cron/store.rs`

#### Scenario: Job type SQL conversion and migration

- WHEN job types are read from SQL
- THEN MUST parse valid values, reject invalid values, and handle legacy migration
- Tests:
  - `job_type_from_sql_reads_valid_value` in `src/cron/store.rs`
  - `job_type_from_sql_rejects_invalid_value` in `src/cron/store.rs`
  - `migration_falls_back_to_legacy_expression` in `src/cron/store.rs`

### REQ-CRON-003: Cron Schedule Utilities

Schedule utilities MUST validate and normalize cron expressions.

#### Scenario: Schedule resolution

- WHEN schedule objects are queried for next run time
- THEN MUST support every/at patterns and timezones
- Tests:
  - `next_run_for_schedule_supports_every_and_at` in `src/cron/schedule.rs`
  - `next_run_for_schedule_supports_timezone` in `src/cron/schedule.rs`

### REQ-CRON-004: Cron Job Types

Cron job types MUST parse and validate correctly.

#### Scenario: Job type parsing

- WHEN job type strings are parsed
- THEN MUST accept known values case-insensitively and reject invalid values
- Tests:
  - `job_type_try_from_accepts_known_values_case_insensitive` in `src/cron/types.rs`
  - `job_type_try_from_rejects_invalid_values` in `src/cron/types.rs`

### REQ-CRON-005: Cron Job Update

Cron jobs MUST support partial updates to command, expression, name, and timezone.

#### Scenario: Job field updates

- WHEN jobs are updated via handler
- THEN MUST change specified fields, preserve unchanged fields, and reject invalid updates
- Tests:
  - `update_changes_command_via_handler` in `src/cron/mod.rs`
  - `update_changes_expression_via_handler` in `src/cron/mod.rs`
  - `update_changes_name_via_handler` in `src/cron/mod.rs`
  - `update_tz_alone_sets_timezone` in `src/cron/mod.rs`
  - `update_expression_preserves_existing_tz` in `src/cron/mod.rs`
  - `update_preserves_unchanged_fields` in `src/cron/mod.rs`
  - `update_no_flags_fails` in `src/cron/mod.rs`
  - `update_nonexistent_job_fails` in `src/cron/mod.rs`
  - `update_security_allows_safe_command` in `src/cron/mod.rs`

### REQ-CRON-006: Memory Consolidation Job

Consolidation jobs MUST produce valid cron jobs with correct schedule and prompt.

#### Scenario: Consolidation job creation

- WHEN consolidation jobs are created
- THEN MUST produce valid jobs with correct schedule, timezone, and prompt content
- Tests:
  - `create_consolidation_job_produces_valid_job` in `src/cron/consolidation.rs`
  - `create_consolidation_job_uses_correct_schedule` in `src/cron/consolidation.rs`
  - `create_consolidation_job_prompt_contains_key_instructions` in `src/cron/consolidation.rs`
  - `create_consolidation_job_with_custom_schedule_applies_tz` in `src/cron/consolidation.rs`

### REQ-ECON-001: Economic Tracker

`EconomicTracker` MUST track agent balance, income, and survival status.

#### Scenario: Tracker initialization and balance tracking

- WHEN the tracker is initialized and tokens are tracked
- THEN MUST set initial balance and reduce it for token usage
- Tests:
  - `tracker_initialization` in `src/economic/tracker.rs`
  - `track_tokens_reduces_balance` in `src/economic/tracker.rs`

#### Scenario: Work income with threshold

- WHEN work income is added
- THEN MUST apply threshold logic correctly
- Tests:
  - `work_income_with_threshold` in `src/economic/tracker.rs`

#### Scenario: Survival status transitions

- WHEN balance changes
- THEN MUST update survival status through all lifecycle stages
- Tests:
  - `survival_status_changes` in `src/economic/tracker.rs`

#### Scenario: State persistence

- WHEN economic state is saved and loaded
- THEN MUST persist and restore correctly
- Tests:
  - `state_persistence` in `src/economic/tracker.rs`

#### Scenario: API call categorization

- WHEN API calls are tracked
- THEN MUST categorize costs correctly
- Tests:
  - `api_call_categorization` in `src/economic/tracker.rs`

### REQ-ECON-002: Task Classifier

`TaskClassifier` MUST classify tasks into occupation categories for billing.

#### Scenario: Task classification by keyword

- WHEN task descriptions are classified
- THEN MUST match software, finance, and fallback categories correctly
- Tests:
  - `test_classifier_new` in `src/economic/classifier.rs`
  - `test_classify_software` in `src/economic/classifier.rs`
  - `test_classify_finance` in `src/economic/classifier.rs`
  - `test_classify_fallback` in `src/economic/classifier.rs`

#### Scenario: Hours estimation

- WHEN task complexity is assessed
- THEN MUST estimate hours for complex and simple tasks
- Tests:
  - `test_estimate_hours_complex` in `src/economic/classifier.rs`
  - `test_estimate_hours_simple` in `src/economic/classifier.rs`

#### Scenario: Fuzzy matching and category filtering

- WHEN occupation names are fuzzy-matched or filtered by category
- THEN MUST return correct results
- Tests:
  - `test_fuzzy_match` in `src/economic/classifier.rs`
  - `test_occupations_by_category` in `src/economic/classifier.rs`

### REQ-ECON-003: Cost Breakdown

`CostBreakdown` MUST track and aggregate token costs.

#### Scenario: Cost arithmetic

- WHEN costs are totaled and added
- THEN MUST compute totals correctly
- Tests:
  - `cost_breakdown_total` in `src/economic/costs.rs`
  - `cost_breakdown_add` in `src/economic/costs.rs`

#### Scenario: Token pricing calculation

- WHEN token pricing is applied
- THEN MUST calculate costs and provide defaults
- Tests:
  - `token_pricing_calculation` in `src/economic/costs.rs`
  - `default_token_pricing` in `src/economic/costs.rs`

### REQ-ECON-004: Survival Status

`SurvivalStatus` MUST reflect economic health based on balance ratios.

#### Scenario: Status thresholds

- WHEN balance ratios change
- THEN MUST assign correct status (thriving, stable, struggling, critical, bankrupt)
- Tests:
  - `thriving_above_80_percent` in `src/economic/status.rs`
  - `stable_between_40_and_80_percent` in `src/economic/status.rs`
  - `struggling_between_10_and_40_percent` in `src/economic/status.rs`
  - `critical_between_0_and_10_percent` in `src/economic/status.rs`
  - `bankrupt_at_zero_or_negative` in `src/economic/status.rs`

#### Scenario: Status utility methods

- WHEN status is queried for operational and intervention flags
- THEN MUST return correct boolean values and display format
- Tests:
  - `is_operational` in `src/economic/status.rs`
  - `needs_intervention` in `src/economic/status.rs`
  - `display_format` in `src/economic/status.rs`

### REQ-COORD-001: Coordination Protocol

Coordination MUST support multi-agent message passing with delivery scopes and envelope sequencing.

#### Scenario: Envelope validation

- WHEN coordination envelopes are constructed
- THEN MUST validate delegate tasks require direct target and task results require correlation ID
- Tests:
  - `delegate_task_requires_direct_target` in `src/coordination/mod.rs`
  - `task_result_requires_correlation_id` in `src/coordination/mod.rs`
  - `json_roundtrip_keeps_payload_shape` in `src/coordination/mod.rs`

#### Scenario: Message deduplication

- WHEN duplicate message IDs are published
- THEN MUST reject duplicates, dead-letter them, and allow reuse after eviction
- Tests:
  - `duplicate_message_ids_are_rejected_and_dead_lettered` in `src/coordination/mod.rs`
  - `dedupe_window_evicts_old_ids_and_allows_reuse_after_eviction` in `src/coordination/mod.rs`

#### Scenario: Context patching and conflict handling

- WHEN context patches conflict
- THEN MUST dead-letter conflicts and validate delegate context requirements
- Tests:
  - `context_patch_conflict_goes_to_dead_letter` in `src/coordination/mod.rs`
  - `delegate_context_patch_requires_correlation_id` in `src/coordination/mod.rs`
  - `delegate_context_patch_rejects_mismatched_correlation_id` in `src/coordination/mod.rs`
  - `delegate_context_patch_rejects_invalid_delegate_key_shape` in `src/coordination/mod.rs`
  - `delegate_context_patch_rejects_empty_tail_segment` in `src/coordination/mod.rs`

#### Scenario: Concurrent publishing and delegation flow

- WHEN multiple agents publish concurrently
- THEN MUST maintain inbox order and support full delegation flow
- Tests:
  - `concurrent_publish_keeps_inbox_order` in `src/coordination/mod.rs`
  - `multi_agent_delegation_flow_updates_context_and_returns_result` in `src/coordination/mod.rs`

#### Scenario: Inbox peek and correlation tracking

- WHEN messages are peeked or queried by correlation ID
- THEN MUST not consume messages, support paging, and track correlation counts
- Tests:
  - `peek_does_not_consume_messages` in `src/coordination/mod.rs`
  - `correlation_pending_and_peek_paging_follow_inbox_lifecycle` in `src/coordination/mod.rs`
  - `inbox_correlation_counts_stay_consistent_with_overflow_evictions` in `src/coordination/mod.rs`
  - `correlation_peek_normalizes_whitespace_in_message_correlation_id` in `src/coordination/mod.rs`

#### Scenario: Agent registration and context snapshot

- WHEN agents register and share context
- THEN MUST track registered agents and provide context snapshots
- Tests:
  - `registered_agents_and_context_snapshot_are_available` in `src/coordination/mod.rs`

#### Scenario: Inbox and dead letter limits

- WHEN inbox or dead letter limits are reached
- THEN MUST drop oldest entries, record dead letters, and cap totals
- Tests:
  - `inbox_limit_drops_oldest_and_records_dead_letter` in `src/coordination/mod.rs`
  - `dead_letter_limit_is_capped` in `src/coordination/mod.rs`

#### Scenario: Context limits and eviction

- WHEN context entries exceed limits
- THEN MUST evict oldest entries while preserving hot keys
- Tests:
  - `context_limit_evicts_oldest_entries_and_tracks_stats` in `src/coordination/mod.rs`
  - `context_limit_uses_write_recency_and_preserves_hot_keys` in `src/coordination/mod.rs`

#### Scenario: Context, dead letter, and delegate paging and indexes

- WHEN context entries or dead letters are queried with offset/paging
- THEN MUST return newest-first pages with correct indexing
- Tests:
  - `context_entries_recent_with_offset_returns_newest_first_pages` in `src/coordination/mod.rs`
  - `dead_letters_recent_returns_newest_first_pages` in `src/coordination/mod.rs`
  - `context_entries_recent_for_correlation_support_paging_and_count` in `src/coordination/mod.rs`
  - `delegate_context_indexes_exclude_non_delegate_keys_and_support_paging` in `src/coordination/mod.rs`
  - `dead_letter_correlation_index_tracks_evictions_and_paging` in `src/coordination/mod.rs`

### REQ-AUTH-001: Auth Service

`AuthService` MUST provide unified authentication for OAuth and token-based providers.

#### Scenario: Provider normalization and profile selection

- WHEN auth is configured for a provider
- THEN MUST normalize provider aliases and select the correct profile
- Tests:
  - `normalize_provider_aliases` in `src/auth/mod.rs`
  - `select_profile_prefers_override_then_active_then_default` in `src/auth/mod.rs`

### REQ-AUTH-002: OAuth Common Utilities

OAuth common utilities MUST generate PKCE state, encode/decode URLs, and parse query params.

#### Scenario: PKCE generation

- WHEN PKCE state is generated
- THEN MUST produce valid verifier and SHA-256 challenge
- Tests:
  - `pkce_generation_is_valid` in `src/auth/oauth_common.rs`
  - `pkce_challenge_is_sha256_of_verifier` in `src/auth/oauth_common.rs`

#### Scenario: URL encoding/decoding

- WHEN URLs are encoded and decoded
- THEN MUST handle basic cases and round-trip correctly
- Tests:
  - `url_encode_basic` in `src/auth/oauth_common.rs`
  - `url_decode_basic` in `src/auth/oauth_common.rs`
  - `url_encode_decode_roundtrip` in `src/auth/oauth_common.rs`

#### Scenario: Query parameter parsing

- WHEN query strings are parsed
- THEN MUST extract params, handle encoded values, and return empty for empty input
- Tests:
  - `parse_query_params_basic` in `src/auth/oauth_common.rs`
  - `parse_query_params_encoded` in `src/auth/oauth_common.rs`
  - `parse_query_params_empty` in `src/auth/oauth_common.rs`

#### Scenario: Miscellaneous utilities

- WHEN base64url is generated or URL truncation is detected
- THEN MUST produce correct lengths and detect truncation
- Tests:
  - `random_base64url_length` in `src/auth/oauth_common.rs`
  - `detect_url_truncation_incomplete_url` in `src/auth/oauth_common.rs`
  - `detect_url_truncation_short_state` in `src/auth/oauth_common.rs`
  - `detect_url_truncation_valid_url` in `src/auth/oauth_common.rs`

### REQ-AUTH-003: OpenAI OAuth

OpenAI OAuth MUST support PKCE authorization, redirect parsing, and JWT account extraction.

#### Scenario: PKCE and redirect parsing

- WHEN OAuth redirects are received
- THEN MUST extract authorization codes and handle errors
- Tests:
  - `pkce_generation_is_valid` in `src/auth/openai_oauth.rs`
  - `parse_redirect_url_extracts_code` in `src/auth/openai_oauth.rs`
  - `parse_redirect_accepts_raw_code` in `src/auth/openai_oauth.rs`
  - `parse_redirect_rejects_state_mismatch` in `src/auth/openai_oauth.rs`
  - `parse_redirect_rejects_error_without_code` in `src/auth/openai_oauth.rs`

#### Scenario: JWT account ID extraction

- WHEN a JWT token is provided
- THEN MUST extract the account ID from the payload
- Tests:
  - `extract_account_id_from_jwt_payload` in `src/auth/openai_oauth.rs`

### REQ-AUTH-004: Gemini OAuth

Gemini OAuth MUST support PKCE authorization, redirect parsing, and email extraction from ID tokens.

#### Scenario: PKCE and redirect parsing

- WHEN Gemini OAuth flows are initiated
- THEN MUST generate valid state and parse authorization responses
- Tests:
  - `pkce_generates_valid_state` in `src/auth/gemini_oauth.rs`
  - `authorize_url_contains_required_params` in `src/auth/gemini_oauth.rs`
  - `parse_code_from_url` in `src/auth/gemini_oauth.rs`
  - `parse_code_from_raw` in `src/auth/gemini_oauth.rs`

#### Scenario: ID token email extraction

- WHEN a Gemini ID token is provided
- THEN MUST extract the email address
- Tests:
  - `extract_email_from_id_token` in `src/auth/gemini_oauth.rs`

### REQ-AUTH-005: Anthropic Token Detection

Anthropic auth kind MUST be detected from token format and explicit override.

#### Scenario: Auth kind detection

- WHEN tokens are inspected
- THEN MUST detect correct auth kind based on metadata, override, JWT pattern, and API prefix
- Tests:
  - `parse_kind_from_metadata` in `src/auth/anthropic_token.rs`
  - `detect_prefers_override` in `src/auth/anthropic_token.rs`
  - `detect_jwt_like_as_authorization` in `src/auth/anthropic_token.rs`
  - `detect_default_for_api_prefix` in `src/auth/anthropic_token.rs`

### REQ-AUTH-006: Auth Profiles

Auth profiles MUST persist, encrypt, and manage per-provider authentication profiles.

#### Scenario: Profile ID and token expiry

- WHEN profile IDs are constructed and token expiry is checked
- THEN MUST format correctly and compute expiry math
- Tests:
  - `profile_id_format` in `src/auth/profiles.rs`
  - `token_expiry_math` in `src/auth/profiles.rs`

#### Scenario: Encrypted storage roundtrip

- WHEN profiles are stored with encryption
- THEN MUST roundtrip correctly and atomically replace files
- Tests:
  - `store_roundtrip_with_encryption` in `src/auth/profiles.rs`
  - `atomic_write_replaces_file` in `src/auth/profiles.rs`

### REQ-GOAL-001: Goal Engine

`Goal` state machine MUST track goal lifecycle (pending, active, paused, completed, failed).

#### Scenario: Goal state serde and config

- WHEN goal state and config are serialized
- THEN MUST roundtrip correctly with defaults
- Tests:
  - `goal_loop_config_serde_roundtrip` in `src/goals/engine.rs`
  - `goal_loop_config_defaults` in `src/goals/engine.rs`
  - `goal_state_serde_roundtrip` in `src/goals/engine.rs`

#### Scenario: Actionable goal selection

- WHEN the engine selects the next actionable goal
- THEN MUST pick highest priority, skip exhausted/non-in-progress goals, and return None when nothing is actionable
- Tests:
  - `select_next_actionable_picks_highest_priority` in `src/goals/engine.rs`
  - `select_next_actionable_skips_exhausted_steps` in `src/goals/engine.rs`
  - `select_next_actionable_skips_non_in_progress_goals` in `src/goals/engine.rs`
  - `select_next_actionable_returns_none_when_nothing_actionable` in `src/goals/engine.rs`

#### Scenario: Step prompt generation

- WHEN step prompts are built
- THEN MUST include goal and step info, and retry warnings
- Tests:
  - `build_step_prompt_includes_goal_and_step` in `src/goals/engine.rs`
  - `build_step_prompt_includes_retry_warning` in `src/goals/engine.rs`

#### Scenario: Result interpretation

- WHEN step results are interpreted
- THEN MUST distinguish success from failure patterns
- Tests:
  - `interpret_result_success` in `src/goals/engine.rs`
  - `interpret_result_failure` in `src/goals/engine.rs`

#### Scenario: State persistence

- WHEN goal state is saved and loaded
- THEN MUST roundtrip through filesystem
- Tests:
  - `load_save_state_roundtrip` in `src/goals/engine.rs`

#### Scenario: Priority ordering and defaults

- WHEN priorities are compared and statuses are constructed
- THEN MUST order correctly and default to pending
- Tests:
  - `priority_ordering` in `src/goals/engine.rs`
  - `goal_status_default_is_pending` in `src/goals/engine.rs`
  - `step_status_default_is_pending` in `src/goals/engine.rs`

#### Scenario: Stalled goal detection

- WHEN goals have exhausted steps, empty steps, completed steps, or mixed states
- THEN MUST detect stalled goals correctly
- Tests:
  - `find_stalled_goals_detects_exhausted_steps` in `src/goals/engine.rs`
  - `find_stalled_goals_ignores_actionable_goals` in `src/goals/engine.rs`
  - `find_stalled_goals_ignores_completed_goals` in `src/goals/engine.rs`
  - `find_stalled_goals_empty_steps_not_stalled` in `src/goals/engine.rs`
  - `find_stalled_goals_multiple_stalled` in `src/goals/engine.rs`
  - `find_stalled_goals_all_steps_completed_is_stalled` in `src/goals/engine.rs`
  - `find_stalled_goals_mix_completed_and_blocked_steps` in `src/goals/engine.rs`

#### Scenario: Reflection prompt generation

- WHEN reflection prompts are built
- THEN MUST include step summary, handle empty context, missing errors, and all tags
- Tests:
  - `build_reflection_prompt_includes_step_summary` in `src/goals/engine.rs`
  - `build_reflection_prompt_empty_context_omits_section` in `src/goals/engine.rs`
  - `build_reflection_prompt_no_last_error_omits_section` in `src/goals/engine.rs`
  - `build_reflection_prompt_all_done_tags` in `src/goals/engine.rs`

#### Scenario: Status deserialization and self-healing

- WHEN goal/step statuses are deserialized from known or unknown variants
- THEN MUST parse all valid variants and self-heal unknown ones
- Tests:
  - `goal_status_deserializes_all_valid_variants` in `src/goals/engine.rs`
  - `goal_status_self_healing_unknown_variants` in `src/goals/engine.rs`
  - `step_status_deserializes_all_valid_variants` in `src/goals/engine.rs`
  - `step_status_self_healing_unknown_variants` in `src/goals/engine.rs`
  - `goal_status_self_healing_in_full_goal_json` in `src/goals/engine.rs`

#### Scenario: Priority serde completeness

- WHEN all priority variants are serialized/deserialized
- THEN MUST roundtrip all comparisons correctly
- Tests:
  - `priority_all_comparisons` in `src/goals/engine.rs`
  - `priority_serde_roundtrip_all_variants` in `src/goals/engine.rs`

### REQ-TUNNEL-001: Tunnel Factory

`create_tunnel` MUST construct the correct tunnel implementation based on configuration.

#### Scenario: Factory routing

- WHEN tunnel configuration specifies a provider
- THEN MUST construct the correct implementation or error for unknown/missing config
- Tests:
  - `factory_none_returns_none` in `src/tunnel/mod.rs`
  - `factory_empty_string_returns_none` in `src/tunnel/mod.rs`
  - `factory_unknown_provider_errors` in `src/tunnel/mod.rs`
  - `factory_cloudflare_missing_config_errors` in `src/tunnel/mod.rs`
  - `factory_cloudflare_with_config_ok` in `src/tunnel/mod.rs`
  - `factory_tailscale_defaults_ok` in `src/tunnel/mod.rs`
  - `factory_ngrok_missing_config_errors` in `src/tunnel/mod.rs`
  - `factory_ngrok_with_config_ok` in `src/tunnel/mod.rs`
  - `factory_custom_missing_config_errors` in `src/tunnel/mod.rs`
  - `factory_custom_with_config_ok` in `src/tunnel/mod.rs`

#### Scenario: Tunnel trait behavior via factory

- WHEN tunnel instances are used via the trait interface
- THEN MUST report correct name, public URL state, and health status
- Tests:
  - `none_tunnel_name` in `src/tunnel/mod.rs`
  - `none_tunnel_public_url_is_none` in `src/tunnel/mod.rs`
  - `none_tunnel_health_always_true` in `src/tunnel/mod.rs`
  - `none_tunnel_start_returns_local` in `src/tunnel/mod.rs`
  - `cloudflare_tunnel_name` in `src/tunnel/mod.rs`
  - `tailscale_tunnel_name` in `src/tunnel/mod.rs`
  - `tailscale_funnel_mode` in `src/tunnel/mod.rs`
  - `ngrok_tunnel_name` in `src/tunnel/mod.rs`
  - `ngrok_with_domain` in `src/tunnel/mod.rs`
  - `custom_tunnel_name` in `src/tunnel/mod.rs`

#### Scenario: Shared process lifecycle

- WHEN shared tunnel processes are managed
- THEN MUST handle no-process case and terminate running processes
- Tests:
  - `kill_shared_no_process_is_ok` in `src/tunnel/mod.rs`
  - `kill_shared_terminates_and_clears_child` in `src/tunnel/mod.rs`

#### Scenario: Health check before start

- WHEN tunnels are health-checked before starting
- THEN MUST report false for all providers
- Tests:
  - `cloudflare_health_false_before_start` in `src/tunnel/mod.rs`
  - `ngrok_health_false_before_start` in `src/tunnel/mod.rs`
  - `tailscale_health_false_before_start` in `src/tunnel/mod.rs`
  - `custom_health_false_before_start_without_health_url` in `src/tunnel/mod.rs`

### REQ-TUNNEL-002: None Tunnel

`NoneTunnel` MUST provide a no-op tunnel with local URL passthrough.

#### Scenario: None tunnel behavior

- WHEN the None tunnel is used
- THEN MUST return local URL, always pass health, and have no public URL
- Tests:
  - `name_is_none` in `src/tunnel/none.rs`
  - `start_returns_local_url` in `src/tunnel/none.rs`
  - `stop_is_noop_success` in `src/tunnel/none.rs`
  - `health_check_is_always_true` in `src/tunnel/none.rs`
  - `public_url_is_always_none` in `src/tunnel/none.rs`

### REQ-TUNNEL-003: Ngrok Tunnel

`NgrokTunnel` MUST manage ngrok tunnel processes with domain support.

#### Scenario: Ngrok pre-start state

- WHEN ngrok tunnel is constructed but not started
- THEN MUST store domain, have no public URL, and report unhealthy
- Tests:
  - `constructor_stores_domain` in `src/tunnel/ngrok.rs`
  - `public_url_is_none_before_start` in `src/tunnel/ngrok.rs`
  - `stop_without_started_process_is_ok` in `src/tunnel/ngrok.rs`
  - `health_check_is_false_before_start` in `src/tunnel/ngrok.rs`

### REQ-TUNNEL-004: Custom Tunnel

`CustomTunnel` MUST execute user-provided tunnel commands with placeholder substitution.

#### Scenario: Custom tunnel command execution

- WHEN custom tunnel commands are started
- THEN MUST handle empty commands, placeholder substitution, and URL extraction
- Tests:
  - `start_with_empty_command_returns_error` in `src/tunnel/custom.rs`
  - `start_without_pattern_returns_local_url` in `src/tunnel/custom.rs`
  - `start_with_pattern_extracts_url` in `src/tunnel/custom.rs`
  - `start_replaces_host_and_port_placeholders` in `src/tunnel/custom.rs`
  - `health_check_with_unreachable_health_url_returns_false` in `src/tunnel/custom.rs`

### REQ-TUNNEL-005: Cloudflare Tunnel

`CloudflareTunnel` MUST manage Cloudflare tunnel processes.

#### Scenario: Cloudflare pre-start state

- WHEN Cloudflare tunnel is constructed but not started
- THEN MUST store token, have no public URL, and report unhealthy
- Tests:
  - `constructor_stores_token` in `src/tunnel/cloudflare.rs`
  - `public_url_is_none_before_start` in `src/tunnel/cloudflare.rs`
  - `stop_without_started_process_is_ok` in `src/tunnel/cloudflare.rs`
  - `health_check_is_false_before_start` in `src/tunnel/cloudflare.rs`

### REQ-TUNNEL-006: Tailscale Tunnel

`TailscaleTunnel` MUST manage Tailscale tunnel/funnel processes.

#### Scenario: Tailscale pre-start state

- WHEN Tailscale tunnel is constructed but not started
- THEN MUST store hostname and mode, have no public URL, and report unhealthy
- Tests:
  - `constructor_stores_hostname_and_mode` in `src/tunnel/tailscale.rs`
  - `public_url_is_none_before_start` in `src/tunnel/tailscale.rs`
  - `health_check_is_false_before_start` in `src/tunnel/tailscale.rs`
  - `stop_without_started_process_is_ok` in `src/tunnel/tailscale.rs`

### REQ-RT-001: Runtime Factory

`create_runtime` MUST construct the correct runtime adapter based on configuration.

#### Scenario: Runtime factory routing

- WHEN runtime configuration specifies a backend
- THEN MUST construct native, Docker, WASM, or error for unknown backends
- Tests:
  - `factory_native` in `src/runtime/mod.rs`
  - `factory_docker` in `src/runtime/mod.rs`
  - `factory_wasm` in `src/runtime/mod.rs`
  - `factory_cloudflare_errors` in `src/runtime/mod.rs`
  - `factory_unknown_errors` in `src/runtime/mod.rs`
  - `factory_empty_errors` in `src/runtime/mod.rs`

### REQ-RT-002: Native Runtime

`NativeRuntime` MUST provide full shell and filesystem access with local storage.

#### Scenario: Native runtime capabilities

- WHEN the native runtime is queried
- THEN MUST report full access, unlimited memory, and correct name
- Tests:
  - `native_name` in `src/runtime/native.rs`
  - `native_has_shell_access` in `src/runtime/native.rs`
  - `native_has_filesystem_access` in `src/runtime/native.rs`
  - `native_supports_long_running` in `src/runtime/native.rs`
  - `native_memory_budget_unlimited` in `src/runtime/native.rs`
  - `native_storage_path_contains_zeroclaw` in `src/runtime/native.rs`
  - `native_builds_shell_command` in `src/runtime/native.rs`

### REQ-RT-003: Docker Runtime

`DockerRuntime` MUST provide containerized execution with workspace mounting and security flags.

#### Scenario: Docker runtime capabilities and command building

- WHEN the Docker runtime builds shell commands
- THEN MUST include runtime flags, network flags, read-only flags, and block unsafe mounts
- Tests:
  - `docker_runtime_name` in `src/runtime/docker.rs`
  - `docker_runtime_memory_budget` in `src/runtime/docker.rs`
  - `docker_build_shell_command_includes_runtime_flags` in `src/runtime/docker.rs`
  - `docker_workspace_allowlist_blocks_outside_paths` in `src/runtime/docker.rs`
  - `docker_build_shell_command_includes_network_flag` in `src/runtime/docker.rs`
  - `docker_build_shell_command_includes_read_only_flag` in `src/runtime/docker.rs`
  - `docker_refuses_root_mount` in `src/runtime/docker.rs`
  - `docker_no_memory_flag_when_not_configured` in `src/runtime/docker.rs`

### REQ-RT-004: WASM Runtime

`WasmRuntime` MUST provide sandboxed WebAssembly execution with capability limits.

#### Scenario: WASM runtime basic capabilities

- WHEN the WASM runtime is queried for capabilities
- THEN MUST report no shell access, configurable filesystem, no long-running, and correct name
- Tests:
  - `wasm_runtime_name` in `src/runtime/wasm.rs`
  - `wasm_no_shell_access` in `src/runtime/wasm.rs`
  - `wasm_no_filesystem_by_default` in `src/runtime/wasm.rs`
  - `wasm_filesystem_when_read_enabled` in `src/runtime/wasm.rs`
  - `wasm_filesystem_when_write_enabled` in `src/runtime/wasm.rs`
  - `wasm_no_long_running` in `src/runtime/wasm.rs`
  - `wasm_memory_budget` in `src/runtime/wasm.rs`
  - `wasm_shell_command_errors` in `src/runtime/wasm.rs`
  - `wasm_storage_path_default` in `src/runtime/wasm.rs`
  - `wasm_storage_path_with_workspace` in `src/runtime/wasm.rs`

#### Scenario: WASM config validation

- WHEN WASM configuration is validated
- THEN MUST reject invalid settings (zero memory, excessive memory, zero fuel, empty/absolute/traversal tools dir) and accept valid settings
- Tests:
  - `validate_rejects_zero_memory` in `src/runtime/wasm.rs`
  - `validate_rejects_excessive_memory` in `src/runtime/wasm.rs`
  - `validate_rejects_zero_fuel` in `src/runtime/wasm.rs`
  - `validate_rejects_zero_max_module_size` in `src/runtime/wasm.rs`
  - `validate_rejects_empty_tools_dir` in `src/runtime/wasm.rs`
  - `validate_rejects_absolute_tools_dir` in `src/runtime/wasm.rs`
  - `validate_rejects_path_traversal` in `src/runtime/wasm.rs`
  - `validate_allows_absolute_tools_dir_when_configured` in `src/runtime/wasm.rs`
  - `validate_allows_path_traversal_when_configured` in `src/runtime/wasm.rs`
  - `validate_rejects_wildcard_host_entries` in `src/runtime/wasm.rs`
  - `validate_ignores_invalid_host_entries_when_non_strict` in `src/runtime/wasm.rs`
  - `validate_accepts_valid_config` in `src/runtime/wasm.rs`
  - `validate_rejects_invalid_module_sha256_pin_format` in `src/runtime/wasm.rs`
  - `validate_rejects_invalid_module_sha256_pin_name` in `src/runtime/wasm.rs`
  - `validate_rejects_enforce_hash_policy_without_pins` in `src/runtime/wasm.rs`
  - `validate_accepts_max_memory` in `src/runtime/wasm.rs`
  - `validate_rejects_memory_just_above_limit` in `src/runtime/wasm.rs`

#### Scenario: WASM fuel and memory limits

- WHEN fuel and memory are configured with defaults and overrides
- THEN MUST use config defaults, respect overrides, and clamp to limits
- Tests:
  - `effective_fuel_uses_config_default` in `src/runtime/wasm.rs`
  - `effective_fuel_respects_override` in `src/runtime/wasm.rs`
  - `effective_fuel_clamps_override_to_config_limit` in `src/runtime/wasm.rs`
  - `effective_memory_uses_config_default` in `src/runtime/wasm.rs`
  - `effective_memory_respects_override` in `src/runtime/wasm.rs`
  - `effective_memory_clamps_override_to_config_limit` in `src/runtime/wasm.rs`
  - `effective_memory_saturating` in `src/runtime/wasm.rs`
  - `wasm_fuel_limit_enforced_in_config` in `src/runtime/wasm.rs`
  - `wasm_memory_limit_enforced_in_config` in `src/runtime/wasm.rs`
  - `wasm_zero_fuel_override_uses_default` in `src/runtime/wasm.rs`

#### Scenario: WASM capabilities validation

- WHEN capabilities are validated against config limits
- THEN MUST reject fuel/memory/host escalation and accept subsets
- Tests:
  - `default_capabilities_match_config` in `src/runtime/wasm.rs`
  - `validate_capabilities_rejects_fuel_escalation` in `src/runtime/wasm.rs`
  - `validate_capabilities_rejects_memory_escalation` in `src/runtime/wasm.rs`
  - `validate_capabilities_rejects_host_escalation` in `src/runtime/wasm.rs`
  - `validate_capabilities_accepts_host_subset` in `src/runtime/wasm.rs`
  - `validate_capabilities_clamps_escalation_when_configured` in `src/runtime/wasm.rs`
  - `capabilities_default_is_locked_down` in `src/runtime/wasm.rs`

#### Scenario: WASM module operations

- WHEN modules are listed, executed, or integrity-checked
- THEN MUST handle missing files, invalid WASM, oversized files, hash mismatches, and symlinks
- Tests:
  - `tools_dir_resolves_relative_to_workspace` in `src/runtime/wasm.rs`
  - `list_modules_empty_when_dir_missing` in `src/runtime/wasm.rs`
  - `list_modules_finds_wasm_files` in `src/runtime/wasm.rs`
  - `validate_module_name_rejects_traversal_like_input` in `src/runtime/wasm.rs`
  - `execute_module_missing_file` in `src/runtime/wasm.rs`
  - `execute_module_invalid_wasm` in `src/runtime/wasm.rs`
  - `execute_module_oversized_file` in `src/runtime/wasm.rs`
  - `execute_module_enforce_hash_policy_rejects_mismatch` in `src/runtime/wasm.rs`
  - `execute_module_warn_hash_policy_allows_execution_path` in `src/runtime/wasm.rs`
  - `execute_module_rejects_symlink_tools_dir_when_enabled` in `src/runtime/wasm.rs`
  - `execute_module_allows_symlink_tools_dir_when_disabled` in `src/runtime/wasm.rs`
  - `execute_module_stub_returns_error_without_feature` in `src/runtime/wasm.rs`

#### Scenario: WASM availability and limits

- WHEN WASM feature availability and memory limits are checked
- THEN MUST match feature flag and handle overflow
- Tests:
  - `is_available_matches_feature_flag` in `src/runtime/wasm.rs`
  - `memory_budget_no_overflow` in `src/runtime/wasm.rs`

### REQ-RT-005: Runtime Traits

`RuntimeAdapter` trait MUST define the adapter interface with default behaviors.

#### Scenario: Trait default behavior

- WHEN the default trait implementation is used
- THEN MUST report zero memory budget and support basic shell command execution
- Tests:
  - `default_memory_budget_is_zero` in `src/runtime/traits.rs`
  - `runtime_reports_capabilities` in `src/runtime/traits.rs`
  - `build_shell_command_executes` in `src/runtime/traits.rs`

### REQ-OBS-001: Observer Factory

`create_observer` MUST construct the correct observer backend from configuration.

#### Scenario: Observer factory routing

- WHEN observer configuration specifies a backend
- THEN MUST construct noop, log, prometheus, otel (with aliases), or fall back to noop for unknown
- Tests:
  - `factory_none_returns_noop` in `src/observability/mod.rs`
  - `factory_noop_returns_noop` in `src/observability/mod.rs`
  - `factory_log_returns_log` in `src/observability/mod.rs`
  - `factory_prometheus_returns_prometheus` in `src/observability/mod.rs`
  - `factory_otel_returns_otel` in `src/observability/mod.rs`
  - `factory_opentelemetry_alias` in `src/observability/mod.rs`
  - `factory_otlp_alias` in `src/observability/mod.rs`
  - `factory_unknown_falls_back_to_noop` in `src/observability/mod.rs`
  - `factory_empty_string_falls_back_to_noop` in `src/observability/mod.rs`
  - `factory_garbage_falls_back_to_noop` in `src/observability/mod.rs`

### REQ-OBS-002: Noop Observer

`NoopObserver` MUST accept all events and metrics without side effects.

#### Scenario: Noop observer behavior

- WHEN events and metrics are recorded on the noop observer
- THEN MUST not panic and report correct name
- Tests:
  - `noop_name` in `src/observability/noop.rs`
  - `noop_record_event_does_not_panic` in `src/observability/noop.rs`
  - `noop_record_metric_does_not_panic` in `src/observability/noop.rs`
  - `noop_flush_does_not_panic` in `src/observability/noop.rs`

### REQ-OBS-003: Log Observer

`LogObserver` MUST log events and metrics to the standard logging framework.

#### Scenario: Log observer behavior

- WHEN events and metrics are recorded on the log observer
- THEN MUST not panic and report correct name
- Tests:
  - `log_observer_name` in `src/observability/log.rs`
  - `log_observer_all_events_no_panic` in `src/observability/log.rs`
  - `log_observer_all_metrics_no_panic` in `src/observability/log.rs`

### REQ-OBS-004: Verbose Observer

`VerboseObserver` MUST output detailed event information to stderr.

#### Scenario: Verbose observer behavior

- WHEN events are recorded on the verbose observer
- THEN MUST not panic and report correct name
- Tests:
  - `verbose_name` in `src/observability/verbose.rs`
  - `verbose_events_do_not_panic` in `src/observability/verbose.rs`

### REQ-OBS-005: Prometheus Observer

`PrometheusObserver` MUST track counters, gauges, and histograms in Prometheus text format.

#### Scenario: Prometheus event and metric recording

- WHEN events and metrics are recorded
- THEN MUST not panic and encode valid Prometheus text format
- Tests:
  - `prometheus_observer_name` in `src/observability/prometheus.rs`
  - `records_all_events_without_panic` in `src/observability/prometheus.rs`
  - `records_all_metrics_without_panic` in `src/observability/prometheus.rs`
  - `encode_produces_prometheus_text_format` in `src/observability/prometheus.rs`

#### Scenario: Counter and gauge behavior

- WHEN counters are incremented and gauges are set
- THEN MUST track values correctly including per-tool and per-component breakdowns
- Tests:
  - `counters_increment_correctly` in `src/observability/prometheus.rs`
  - `tool_calls_track_success_and_failure_separately` in `src/observability/prometheus.rs`
  - `errors_track_by_component` in `src/observability/prometheus.rs`
  - `gauge_reflects_latest_value` in `src/observability/prometheus.rs`

#### Scenario: LLM response tracking

- WHEN LLM responses are recorded
- THEN MUST track request count, tokens, and handle missing token counts
- Tests:
  - `llm_response_tracks_request_count_and_tokens` in `src/observability/prometheus.rs`
  - `llm_response_without_tokens_increments_request_only` in `src/observability/prometheus.rs`

### REQ-OBS-006: OpenTelemetry Observer

`OtelObserver` MUST export events and metrics via OpenTelemetry protocol.

#### Scenario: OTel event and metric recording

- WHEN events and metrics are recorded on the OTel observer
- THEN MUST not panic, handle error events, and flush idempotently
- Tests:
  - `otel_observer_name` in `src/observability/otel.rs`
  - `records_all_events_without_panic` in `src/observability/otel.rs`
  - `records_all_metrics_without_panic` in `src/observability/otel.rs`
  - `flush_does_not_panic` in `src/observability/otel.rs`
  - `otel_records_error_event_without_panic` in `src/observability/otel.rs`
  - `otel_records_llm_failure_without_panic` in `src/observability/otel.rs`
  - `otel_flush_idempotent_with_unreachable_endpoint` in `src/observability/otel.rs`
  - `otel_records_zero_duration_metrics` in `src/observability/otel.rs`
  - `otel_observer_creation_with_valid_endpoint_succeeds` in `src/observability/otel.rs`

### REQ-OBS-007: Multi Observer

`MultiObserver` MUST fan out events, metrics, and flush to all child observers.

#### Scenario: Multi observer fan-out

- WHEN events, metrics, and flush are called on multi observer
- THEN MUST forward to all child observers
- Tests:
  - `multi_name` in `src/observability/multi.rs`
  - `multi_empty_no_panic` in `src/observability/multi.rs`
  - `multi_fans_out_events` in `src/observability/multi.rs`
  - `multi_fans_out_metrics` in `src/observability/multi.rs`
  - `multi_fans_out_flush` in `src/observability/multi.rs`

### REQ-OBS-008: Cost Observer

`CostObserver` MUST track LLM usage costs per response using model pricing.

#### Scenario: Cost tracking from LLM events

- WHEN LLM response events are received
- THEN MUST record costs, ignore failures/zero tokens, and match model families
- Tests:
  - `cost_observer_records_llm_response_usage` in `src/observability/cost.rs`
  - `cost_observer_ignores_failed_responses` in `src/observability/cost.rs`
  - `cost_observer_ignores_zero_token_responses` in `src/observability/cost.rs`
  - `cost_observer_uses_default_pricing_for_unknown_models` in `src/observability/cost.rs`
  - `cost_observer_matches_model_family` in `src/observability/cost.rs`

### REQ-OBS-009: Runtime Trace

Runtime trace MUST record and query trace events with configurable storage.

#### Scenario: Trace path resolution and storage modes

- WHEN trace storage is configured
- THEN MUST resolve paths correctly and parse storage modes
- Tests:
  - `resolve_trace_path_relative_joins_workspace` in `src/observability/runtime_trace.rs`
  - `storage_mode_parses_known_values` in `src/observability/runtime_trace.rs`

#### Scenario: Rolling mode and event lookup

- WHEN rolling storage mode is used
- THEN MUST keep latest entries and find events by ID
- Tests:
  - `rolling_mode_keeps_latest_entries` in `src/observability/runtime_trace.rs`
  - `find_event_by_id_returns_match` in `src/observability/runtime_trace.rs`

### REQ-OBS-010: Observer Traits

`Observer` trait MUST define the event/metric recording interface with default flush behavior.

#### Scenario: Trait behavior

- WHEN the Observer trait is implemented
- THEN MUST support events, metrics, flush, and cloneable event/metric types
- Tests:
  - `observer_records_events_and_metrics` in `src/observability/traits.rs`
  - `observer_default_flush_and_as_any_work` in `src/observability/traits.rs`
  - `observer_event_and_metric_are_cloneable` in `src/observability/traits.rs`

### REQ-PLUGIN-001: Plugin Loader

Plugin loader MUST discover, filter, and isolate plugins.

#### Scenario: Plugin loading and isolation

- WHEN plugins are loaded
- THEN MUST activate OK plugins, isolate panicking/erroring plugins, and respect deny/allow lists
- Tests:
  - `disabled_system_returns_empty_registry` in `src/plugins/loader.rs`
  - `ok_plugin_is_active` in `src/plugins/loader.rs`
  - `panic_plugin_is_isolated` in `src/plugins/loader.rs`
  - `error_plugin_is_isolated` in `src/plugins/loader.rs`
  - `denylist_disables_plugin` in `src/plugins/loader.rs`
  - `allowlist_filters_plugins` in `src/plugins/loader.rs`

### REQ-PLUGIN-002: Plugin Runtime

Plugin runtime MUST initialize from config and manage the global registry.

#### Scenario: Runtime initialization

- WHEN the runtime is initialized
- THEN MUST reject invalid manifests, load valid ones, and apply config updates
- Tests:
  - `runtime_rejects_invalid_manifest` in `src/plugins/runtime.rs`
  - `runtime_loads_plugin_manifest_files` in `src/plugins/runtime.rs`
  - `initialize_from_config_applies_updated_plugin_dirs` in `src/plugins/runtime.rs`

### REQ-PLUGIN-003: Plugin Registry

Plugin registry MUST track active plugins with tool and provider indexes.

#### Scenario: Registry operations

- WHEN plugins are registered
- THEN MUST track active count, rebuild indexes on re-register, and report empty state
- Tests:
  - `empty_registry` in `src/plugins/registry.rs`
  - `active_count_filters_correctly` in `src/plugins/registry.rs`
  - `manifest_indexes_replace_on_reregister` in `src/plugins/registry.rs`

### REQ-PLUGIN-004: Plugin Discovery

Plugin discovery MUST scan workspace and extra paths for plugin manifests.

#### Scenario: Plugin directory scanning

- WHEN plugin directories are scanned
- THEN MUST discover from workspace, extra paths, skip hidden dirs, and record bad manifests
- Tests:
  - `discover_from_workspace` in `src/plugins/discovery.rs`
  - `discover_from_extra_paths` in `src/plugins/discovery.rs`
  - `discover_skips_hidden_dirs` in `src/plugins/discovery.rs`
  - `discover_records_bad_manifest` in `src/plugins/discovery.rs`

### REQ-PLUGIN-005: Plugin Manifest

Plugin manifest MUST validate ID, module path, and WIT package requirements.

#### Scenario: Manifest loading and validation

- WHEN manifests are loaded from directories
- THEN MUST parse valid manifests, reject missing/empty IDs, and validate runtime requirements
- Tests:
  - `load_valid_manifest` in `src/plugins/manifest.rs`
  - `load_missing_manifest` in `src/plugins/manifest.rs`
  - `load_manifest_missing_id` in `src/plugins/manifest.rs`
  - `load_manifest_empty_id` in `src/plugins/manifest.rs`
  - `manifest_requires_id_and_module_path_for_runtime_validation` in `src/plugins/manifest.rs`
  - `manifest_rejects_unknown_wit_package` in `src/plugins/manifest.rs`

### REQ-PLUGIN-006: Plugin Traits

Plugin API MUST accumulate tools and hooks, and logger MUST prefix messages with plugin ID.

#### Scenario: Plugin API and logger behavior

- WHEN plugins register tools/hooks and use the logger
- THEN MUST accumulate correctly and prefix log messages
- Tests:
  - `plugin_capability_serde_roundtrip` in `src/plugins/traits.rs`
  - `plugin_logger_has_correct_prefix` in `src/plugins/traits.rs`
  - `plugin_api_accumulates_tools_and_hooks` in `src/plugins/traits.rs`
  - `plugin_api_collects_nothing_by_default` in `src/plugins/traits.rs`

### REQ-PLUGIN-007: Plugin Hot Reload

Hot reload MUST respect configuration for enable/disable state.

#### Scenario: Hot reload configuration

- WHEN hot reload config is checked
- THEN MUST default to disabled
- Tests:
  - `hot_reload_disabled_by_default` in `src/plugins/hot_reload.rs`

### REQ-PLUGIN-008: Plugin Module Reexports

Plugin module MUST reexport core types accessibly.

#### Scenario: Module reexport accessibility

- WHEN plugin types are imported
- THEN MUST be accessible through module reexports
- Tests:
  - `module_reexports_are_accessible` in `src/plugins/mod.rs`

### REQ-PLUGIN-009: Plugin Observer Bridge

Plugin observer bridge MUST forward observer calls to inner implementation.

#### Scenario: Observer bridge forwarding

- WHEN events are recorded on the bridge
- THEN MUST forward to the inner observer
- Tests:
  - `bridge_forwards_events` in `src/plugins/bridge/observer.rs`

### REQ-APPROVAL-001: Approval System

Approval workflow MUST support interactive pre-execution approval with configurable policies.

#### Scenario: Auto-approve and always-ask tools

- WHEN tools are checked against approval policies
- THEN MUST skip prompt for auto-approved tools and always prompt for always-ask tools
- Tests:
  - `auto_approve_tools_skip_prompt` in `src/approval/mod.rs`
  - `always_ask_tools_always_prompt` in `src/approval/mod.rs`
  - `unknown_tool_needs_approval_in_supervised` in `src/approval/mod.rs`

#### Scenario: Autonomy modes

- WHEN full autonomy or readonly mode is active
- THEN MUST never prompt
- Tests:
  - `full_autonomy_never_prompts` in `src/approval/mod.rs`
  - `readonly_never_prompts` in `src/approval/mod.rs`

#### Scenario: Session allowlist management

- WHEN "always" responses are recorded
- THEN MUST add to session allowlist, but always-ask tools must override
- Tests:
  - `always_response_adds_to_session_allowlist` in `src/approval/mod.rs`
  - `always_ask_overrides_session_allowlist` in `src/approval/mod.rs`
  - `yes_response_does_not_add_to_allowlist` in `src/approval/mod.rs`

#### Scenario: Non-CLI session approval

- WHEN non-CLI approvals are granted or revoked
- THEN MUST persist, list, and revoke correctly
- Tests:
  - `non_cli_session_approval_persists_across_checks` in `src/approval/mod.rs`
  - `non_cli_session_approval_can_be_revoked` in `src/approval/mod.rs`
  - `non_cli_session_allowlist_snapshot_lists_granted_tools` in `src/approval/mod.rs`

#### Scenario: Non-CLI allow-all-once tokens

- WHEN allow-all-once tokens are granted
- THEN MUST be counted and consumed correctly
- Tests:
  - `non_cli_allow_all_once_tokens_are_counted_and_consumed` in `src/approval/mod.rs`

#### Scenario: Persistent runtime grants and revokes

- WHEN persistent runtime grants/revokes are applied
- THEN MUST update policy immediately
- Tests:
  - `persistent_runtime_grant_updates_policy_immediately` in `src/approval/mod.rs`
  - `persistent_runtime_revoke_updates_policy_immediately` in `src/approval/mod.rs`

#### Scenario: Non-CLI pending approval requests

- WHEN non-CLI approval requests are created, confirmed, or rejected
- THEN MUST track pending state, require same sender/channel, and handle expiry
- Tests:
  - `create_and_confirm_pending_non_cli_approval_request` in `src/approval/mod.rs`
  - `create_and_reject_pending_non_cli_approval_request` in `src/approval/mod.rs`
  - `pending_non_cli_resolution_is_recorded_and_consumed` in `src/approval/mod.rs`
  - `pending_non_cli_approval_requires_same_sender_and_channel` in `src/approval/mod.rs`
  - `list_pending_non_cli_approvals_filters_scope` in `src/approval/mod.rs`
  - `pending_non_cli_approval_expiry_is_pruned` in `src/approval/mod.rs`

#### Scenario: Non-CLI approval actor and mode configuration

- WHEN non-CLI approval actors and natural language modes are configured
- THEN MUST respect defaults, allowlists, wildcards, per-channel overrides, and runtime policy replacement
- Tests:
  - `non_cli_approval_actor_defaults_to_allow_when_not_configured` in `src/approval/mod.rs`
  - `non_cli_natural_language_approval_mode_defaults_to_direct` in `src/approval/mod.rs`
  - `non_cli_approval_actor_allowlist_supports_exact_and_wildcards` in `src/approval/mod.rs`
  - `non_cli_natural_language_approval_mode_honors_config_override` in `src/approval/mod.rs`
  - `non_cli_natural_language_approval_mode_supports_per_channel_override` in `src/approval/mod.rs`
  - `replace_runtime_non_cli_policy_updates_modes_and_approvers` in `src/approval/mod.rs`

#### Scenario: Audit log and serialization

- WHEN approval decisions are recorded
- THEN MUST log with timestamps, channels, and support serde roundtrip
- Tests:
  - `audit_log_records_decisions` in `src/approval/mod.rs`
  - `audit_log_contains_timestamp_and_channel` in `src/approval/mod.rs`
  - `approval_response_serde_roundtrip` in `src/approval/mod.rs`
  - `approval_request_serde` in `src/approval/mod.rs`

#### Scenario: Argument summarization

- WHEN tool arguments are summarized for display
- THEN MUST format objects, truncate long values safely, and handle non-objects
- Tests:
  - `summarize_args_object` in `src/approval/mod.rs`
  - `summarize_args_truncates_long_values` in `src/approval/mod.rs`
  - `summarize_args_unicode_safe_truncation` in `src/approval/mod.rs`
  - `summarize_args_non_object` in `src/approval/mod.rs`

### REQ-DAEMON-001: Daemon Process

Daemon MUST manage component supervision with port binding and state writing.

#### Scenario: State file path

- WHEN the daemon state file path is resolved
- THEN MUST use the config directory
- Tests:
  - `state_file_path_uses_config_directory` in `src/daemon/mod.rs`

#### Scenario: Component supervisor lifecycle

- WHEN supervised components fail or exit unexpectedly
- THEN MUST mark errors and restart
- Tests:
  - `supervisor_marks_error_and_restart_on_failure` in `src/daemon/mod.rs`
  - `supervisor_marks_unexpected_exit_as_error` in `src/daemon/mod.rs`

#### Scenario: Supervised channel detection

- WHEN config is checked for supervised channels
- THEN MUST detect presence/absence of all supported channel types
- Tests:
  - `detects_no_supervised_channels` in `src/daemon/mod.rs`
  - `detects_supervised_channels_present` in `src/daemon/mod.rs`
  - `detects_dingtalk_as_supervised_channel` in `src/daemon/mod.rs`
  - `detects_mattermost_as_supervised_channel` in `src/daemon/mod.rs`
  - `detects_qq_as_supervised_channel` in `src/daemon/mod.rs`
  - `detects_nextcloud_talk_as_supervised_channel` in `src/daemon/mod.rs`

#### Scenario: Heartbeat tasks and announcements

- WHEN heartbeat tasks are resolved and output is processed
- THEN MUST use file tasks, fall back to config, skip sentinels, and format announcements
- Tests:
  - `heartbeat_tasks_use_file_tasks_when_available` in `src/daemon/mod.rs`
  - `heartbeat_tasks_fall_back_to_config_message` in `src/daemon/mod.rs`
  - `heartbeat_tasks_ignore_empty_fallback_message` in `src/daemon/mod.rs`
  - `heartbeat_announcement_text_skips_no_reply_sentinel` in `src/daemon/mod.rs`
  - `heartbeat_announcement_text_skips_heartbeat_ok_sentinel` in `src/daemon/mod.rs`
  - `heartbeat_announcement_text_skips_heartbeat_ok_prefix_case_insensitive` in `src/daemon/mod.rs`
  - `heartbeat_announcement_text_uses_default_for_empty_output` in `src/daemon/mod.rs`
  - `heartbeat_announcement_text_keeps_regular_output` in `src/daemon/mod.rs`

#### Scenario: Heartbeat delivery target resolution

- WHEN heartbeat delivery targets are configured
- THEN MUST validate channel configuration and reject unsupported/missing setups
- Tests:
  - `heartbeat_delivery_target_none_when_unset` in `src/daemon/mod.rs`
  - `heartbeat_delivery_target_requires_to_field` in `src/daemon/mod.rs`
  - `heartbeat_delivery_target_requires_target_field` in `src/daemon/mod.rs`
  - `heartbeat_delivery_target_rejects_unsupported_channel` in `src/daemon/mod.rs`
  - `heartbeat_delivery_target_requires_channel_configuration` in `src/daemon/mod.rs`
  - `heartbeat_delivery_target_accepts_telegram_configuration` in `src/daemon/mod.rs`
  - `heartbeat_delivery_target_accepts_whatsapp_web_target_in_web_mode` in `src/daemon/mod.rs`
  - `heartbeat_delivery_target_rejects_whatsapp_web_target_in_cloud_mode` in `src/daemon/mod.rs`

### REQ-SERVICE-001: Service Management

`InitSystem` MUST support systemd, OpenRC, and Windows Task Scheduler for service management.

#### Scenario: Init system parsing

- WHEN init system strings are parsed
- THEN MUST accept known values, reject unknown, and default to auto
- Tests:
  - `init_system_from_str_parses_valid_values` in `src/service/mod.rs`
  - `init_system_from_str_rejects_unknown` in `src/service/mod.rs`
  - `init_system_default_is_auto` in `src/service/mod.rs`

#### Scenario: Service file generation

- WHEN service files are generated
- THEN MUST use correct suffix and contain required directives
- Tests:
  - `linux_service_file_has_expected_suffix` in `src/service/mod.rs`
  - `windows_task_name_is_constant` in `src/service/mod.rs`
  - `generate_openrc_script_contains_required_directives` in `src/service/mod.rs`

#### Scenario: Command execution utilities

- WHEN system commands are captured or checked
- THEN MUST read stdout, fall back to stderr, and error on non-zero status
- Tests:
  - `run_capture_reads_stdout` in `src/service/mod.rs`
  - `run_capture_falls_back_to_stderr` in `src/service/mod.rs`
  - `run_checked_errors_on_non_zero_status` in `src/service/mod.rs`
  - `run_capture_reads_stdout_windows` in `src/service/mod.rs`
  - `run_checked_errors_on_non_zero_status_windows` in `src/service/mod.rs`

#### Scenario: XML escaping

- WHEN XML content is escaped for service configurations
- THEN MUST escape reserved characters
- Tests:
  - `xml_escape_escapes_reserved_chars` in `src/service/mod.rs`

#### Scenario: System user and path utilities

- WHEN system UID or binary path is checked
- THEN MUST match system UID and detect home path
- Tests:
  - `is_root_matches_system_uid` in `src/service/mod.rs`
  - `warn_if_binary_in_home_detects_home_path` in `src/service/mod.rs`

#### Scenario: Shell quoting and OpenRC writability probe

- WHEN shell arguments are quoted or OpenRC writability probes are built
- THEN MUST escape quotes and prefer runuser over su
- Tests:
  - `shell_single_quote_escapes_single_quotes` in `src/service/mod.rs`
  - `openrc_writability_probe_prefers_runuser_when_available` in `src/service/mod.rs`
  - `openrc_writability_probe_falls_back_to_su` in `src/service/mod.rs`

### REQ-UTIL-001: Utilities

Utility functions MUST provide safe string truncation respecting UTF-8 boundaries.

#### Scenario: ASCII truncation

- WHEN ASCII strings are truncated
- THEN MUST truncate at character boundary with ellipsis, or return full string if within limit
- Tests:
  - `test_truncate_ascii_no_truncation` in `src/util.rs`
  - `test_truncate_ascii_with_truncation` in `src/util.rs`

#### Scenario: Edge case truncation

- WHEN empty strings, exact boundaries, or zero max chars are truncated
- THEN MUST handle correctly
- Tests:
  - `test_truncate_empty_string` in `src/util.rs`
  - `test_truncate_at_exact_boundary` in `src/util.rs`
  - `test_truncate_zero_max_chars` in `src/util.rs`

#### Scenario: Multi-byte and emoji truncation

- WHEN multi-byte characters (emoji, CJK, accented, Unicode) are truncated
- THEN MUST not split multi-byte characters
- Tests:
  - `test_truncate_emoji_single` in `src/util.rs`
  - `test_truncate_emoji_multiple` in `src/util.rs`
  - `test_truncate_mixed_ascii_emoji` in `src/util.rs`
  - `test_truncate_cjk_characters` in `src/util.rs`
  - `test_truncate_accented_characters` in `src/util.rs`
  - `test_truncate_unicode_edge_case` in `src/util.rs`
  - `test_truncate_long_string` in `src/util.rs`

#### Scenario: Floor UTF-8 char boundary

- WHEN byte index floor is computed for UTF-8 strings
- THEN MUST find correct char boundary for ASCII and multibyte
- Tests:
  - `test_floor_utf8_char_boundary_ascii` in `src/util.rs`
  - `test_floor_utf8_char_boundary_multibyte` in `src/util.rs`

## Coverage Notes

Total explicit tests: 712

| Module | File | Test Count |
|--------|------|-----------|
| SOP engine | `src/sop/engine.rs` | 42 |
| SOP conditions | `src/sop/condition.rs` | 30 |
| SOP metrics | `src/sop/metrics.rs` | 28 |
| SOP types | `src/sop/types.rs` | 14 |
| SOP loading/parsing | `src/sop/mod.rs` | 17 |
| SOP gates | `src/sop/gates.rs` | 16 |
| SOP dispatch | `src/sop/dispatch.rs` | 13 |
| SOP audit | `src/sop/audit.rs` | 4 |
| Skills core | `src/skills/mod.rs` | 58 |
| Skills audit | `src/skills/audit.rs` | 24 |
| Skills tool handler | `src/skills/tool_handler.rs` | 11 |
| Skills templates | `src/skills/templates.rs` | 10 |
| Skills symlink | `src/skills/symlink_tests.rs` | 3 |
| Cron scheduler | `src/cron/scheduler.rs` | 30 |
| Cron store | `src/cron/store.rs` | 13 |
| Cron commands | `src/cron/mod.rs` | 9 |
| Cron consolidation | `src/cron/consolidation.rs` | 4 |
| Cron schedule | `src/cron/schedule.rs` | 2 |
| Cron types | `src/cron/types.rs` | 2 |
| Economic tracker | `src/economic/tracker.rs` | 6 |
| Economic classifier | `src/economic/classifier.rs` | 8 |
| Economic costs | `src/economic/costs.rs` | 4 |
| Economic status | `src/economic/status.rs` | 8 |
| Coordination | `src/coordination/mod.rs` | 25 |
| Auth service | `src/auth/mod.rs` | 2 |
| Auth OAuth common | `src/auth/oauth_common.rs` | 12 |
| Auth OpenAI OAuth | `src/auth/openai_oauth.rs` | 6 |
| Auth Gemini OAuth | `src/auth/gemini_oauth.rs` | 5 |
| Auth Anthropic token | `src/auth/anthropic_token.rs` | 4 |
| Auth profiles | `src/auth/profiles.rs` | 4 |
| Goals engine | `src/goals/engine.rs` | 33 |
| Tunnel factory | `src/tunnel/mod.rs` | 26 |
| Tunnel none | `src/tunnel/none.rs` | 5 |
| Tunnel ngrok | `src/tunnel/ngrok.rs` | 4 |
| Tunnel custom | `src/tunnel/custom.rs` | 5 |
| Tunnel cloudflare | `src/tunnel/cloudflare.rs` | 4 |
| Tunnel tailscale | `src/tunnel/tailscale.rs` | 4 |
| Runtime factory | `src/runtime/mod.rs` | 6 |
| Runtime native | `src/runtime/native.rs` | 7 |
| Runtime docker | `src/runtime/docker.rs` | 8 |
| Runtime WASM | `src/runtime/wasm.rs` | 58 |
| Runtime traits | `src/runtime/traits.rs` | 3 |
| Observability factory | `src/observability/mod.rs` | 10 |
| Observability noop | `src/observability/noop.rs` | 4 |
| Observability log | `src/observability/log.rs` | 3 |
| Observability verbose | `src/observability/verbose.rs` | 2 |
| Observability prometheus | `src/observability/prometheus.rs` | 10 |
| Observability OTel | `src/observability/otel.rs` | 9 |
| Observability multi | `src/observability/multi.rs` | 5 |
| Observability cost | `src/observability/cost.rs` | 5 |
| Observability runtime trace | `src/observability/runtime_trace.rs` | 4 |
| Observability traits | `src/observability/traits.rs` | 3 |
| Plugins loader | `src/plugins/loader.rs` | 6 |
| Plugins runtime | `src/plugins/runtime.rs` | 3 |
| Plugins registry | `src/plugins/registry.rs` | 3 |
| Plugins discovery | `src/plugins/discovery.rs` | 4 |
| Plugins manifest | `src/plugins/manifest.rs` | 6 |
| Plugins traits | `src/plugins/traits.rs` | 4 |
| Plugins hot reload | `src/plugins/hot_reload.rs` | 1 |
| Plugins mod | `src/plugins/mod.rs` | 1 |
| Plugins bridge observer | `src/plugins/bridge/observer.rs` | 1 |
| Approval | `src/approval/mod.rs` | 34 |
| Daemon | `src/daemon/mod.rs` | 25 |
| Service | `src/service/mod.rs` | 17 |
| Utilities | `src/util.rs` | 14 |

## Mock Strategy

- SOP: Direct engine construction with test step definitions
- Skills: `tempfile::TempDir` for skill workspace isolation
- Cron: JSONL fixtures in temp directory; SQLite in-memory for store tests
- Economic: Direct tracker construction with test config
- Coordination: Direct bus construction with mock envelopes
- Auth: Mock OAuth token endpoints; PKCE generation with in-memory profiles
- Goals: Direct engine construction with sample goal state
- Tunnels: Test config parsing; skip live tunnel tests; pre-start state assertions
- Runtime: Direct adapter construction with config variations
- Observers: Direct construction with event/metric assertions; mock inner observers for multi/bridge
- Plugins: Mock manifest files in temp directory; stub Plugin trait implementations
- Approval: Direct ApprovalService construction with various autonomy configs
- Daemon: Config-driven state path resolution; mock component supervisors
- Service: Command execution mocking; init system string parsing
