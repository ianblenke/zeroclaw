# Security Policy Specification

## Purpose
Define the behavioral contract for ZeroClaw's security policy enforcement: autonomy levels, command risk classification, allowlist validation, path guards, rate limiting, role-based access control, emergency stop, and the redact helper.

## Scope
- Files: `src/security/policy.rs` (112 tests), `src/security/mod.rs` (3 tests), `src/security/roles.rs` (7 tests), `src/security/estop.rs` (7 tests)
- Risk tier: HIGH (security-critical enforcement boundary)
- Total tests: 129

## Requirements

---

### Autonomy Level (`src/security/policy.rs`)

### REQ-SEC-POL-001: AutonomyLevel MUST default to Supervised
The default autonomy level is Supervised — deny-by-default for state-changing commands.

#### Scenario: Default autonomy
- WHEN `AutonomyLevel::default()` is called
- THEN it returns `AutonomyLevel::Supervised`
- Test: `autonomy_default_is_supervised` in `src/security/policy.rs`

#### Scenario: Default policy has sane values
- WHEN `SecurityPolicy::default()` is constructed
- THEN autonomy is Supervised, workspace_only is true, and rate limits are positive
- Test: `default_policy_has_sane_values` in `src/security/policy.rs`

### REQ-SEC-POL-002: AutonomyLevel MUST serialize/deserialize correctly
Serde roundtrip MUST preserve autonomy level identity for all variants.

#### Scenario: Serde roundtrip
- WHEN AutonomyLevel variants are serialized then deserialized
- THEN each variant is preserved exactly
- Test: `autonomy_serde_roundtrip` in `src/security/policy.rs`

### REQ-SEC-POL-003: can_act() MUST reflect autonomy level
ReadOnly MUST return false; Supervised and Full MUST return true.

#### Scenario: ReadOnly cannot act
- WHEN `can_act()` is called on ReadOnly policy
- THEN it returns false
- Test: `can_act_readonly_false` in `src/security/policy.rs`

#### Scenario: Supervised can act
- WHEN `can_act()` is called on Supervised policy
- THEN it returns true
- Test: `can_act_supervised_true` in `src/security/policy.rs`

#### Scenario: Full can act
- WHEN `can_act()` is called on Full policy
- THEN it returns true
- Test: `can_act_full_true` in `src/security/policy.rs`

---

### Tool Operation Enforcement (`src/security/policy.rs`)

### REQ-SEC-POL-004: enforce_tool_operation MUST gate by autonomy and rate budget
Read operations MUST be allowed in ReadOnly; act operations MUST be blocked in ReadOnly; act operations MUST consume rate budget.

#### Scenario: Read allowed in ReadOnly
- WHEN `enforce_tool_operation(Read)` is called on ReadOnly policy
- THEN it returns Ok
- Test: `enforce_tool_operation_read_allowed_in_readonly_mode` in `src/security/policy.rs`

#### Scenario: Act blocked in ReadOnly
- WHEN `enforce_tool_operation(Act)` is called on ReadOnly policy
- THEN it returns Err
- Test: `enforce_tool_operation_act_blocked_in_readonly_mode` in `src/security/policy.rs`

#### Scenario: Act consumes rate budget
- WHEN `enforce_tool_operation(Act)` is called repeatedly up to rate limit
- THEN it succeeds within limit and fails when exceeded
- Test: `enforce_tool_operation_act_uses_rate_budget` in `src/security/policy.rs`

---

### Command Allowlist (`src/security/policy.rs`)

### REQ-SEC-POL-005: is_command_allowed MUST enforce allowlist with autonomy levels
Commands MUST be validated against the allowlist. ReadOnly blocks all. Full still uses allowlist. Supervised allows listed commands.

#### Scenario: Basic allowed commands pass
- WHEN listed commands (ls, cat, grep, etc.) are checked
- THEN `is_command_allowed()` returns true
- Test: `allowed_commands_basic` in `src/security/policy.rs`

#### Scenario: Unlisted commands blocked
- WHEN unlisted commands (curl, wget, etc.) are checked
- THEN `is_command_allowed()` returns false
- Test: `blocked_commands_basic` in `src/security/policy.rs`

#### Scenario: ReadOnly blocks all commands
- WHEN autonomy is ReadOnly
- THEN all commands are blocked including safe ones
- Tests:
  - `readonly_blocks_all_commands` in `src/security/policy.rs`
  - `readonly_blocks_even_safe_commands` in `src/security/policy.rs`

#### Scenario: Full autonomy still uses allowlist
- WHEN autonomy is Full
- THEN commands not in allowlist are still blocked
- Test: `full_autonomy_still_uses_allowlist` in `src/security/policy.rs`

#### Scenario: Supervised allows listed commands
- WHEN autonomy is Supervised and command is in allowlist
- THEN command is allowed
- Test: `supervised_allows_listed_commands` in `src/security/policy.rs`

#### Scenario: Absolute path commands extract basename
- WHEN command uses absolute path like `/usr/bin/ls`
- THEN basename `ls` is matched against allowlist
- Test: `command_with_absolute_path_extracts_basename` in `src/security/policy.rs`

#### Scenario: Allowlist supports explicit executable paths
- WHEN allowlist contains `/usr/bin/python3`
- THEN that exact path is allowed
- Test: `allowlist_supports_explicit_executable_paths` in `src/security/policy.rs`

#### Scenario: Allowlist supports wildcard
- WHEN allowlist contains `*`
- THEN all commands are allowed (subject to other restrictions)
- Test: `allowlist_supports_wildcard_entry` in `src/security/policy.rs`

#### Scenario: Empty command blocked
- WHEN an empty string is passed as command
- THEN it is blocked
- Test: `empty_command_blocked` in `src/security/policy.rs`

#### Scenario: Piped commands validate all segments
- WHEN command contains pipes (`|`)
- THEN each segment is validated independently
- Test: `command_with_pipes_validates_all_segments` in `src/security/policy.rs`

#### Scenario: Custom allowlist
- WHEN a non-default allowlist is configured
- THEN only listed commands are allowed
- Test: `custom_allowlist` in `src/security/policy.rs`

#### Scenario: Empty allowlist blocks everything
- WHEN allowlist is empty
- THEN all commands are blocked
- Test: `empty_allowlist_blocks_everything` in `src/security/policy.rs`

---

### Command Risk Classification (`src/security/policy.rs`)

### REQ-SEC-POL-006: Destructive commands MUST be classified as High risk
Commands like rm, sudo, curl, ssh, dd, shutdown MUST return `CommandRiskLevel::High`.

#### Scenario: High-risk command detection
- WHEN `command_risk_level()` is called with rm, sudo, curl, ssh, etc.
- THEN it returns `CommandRiskLevel::High`
- Test: `command_risk_high_for_dangerous_commands` in `src/security/policy.rs`

### REQ-SEC-POL-007: State-changing development commands MUST be classified as Medium risk
Commands like git commit, npm install, cargo publish, touch, mkdir MUST return `CommandRiskLevel::Medium`.

#### Scenario: Medium-risk command detection
- WHEN `command_risk_level()` is called with git commit, npm install, etc.
- THEN it returns `CommandRiskLevel::Medium`
- Test: `command_risk_medium_for_mutating_commands` in `src/security/policy.rs`

### REQ-SEC-POL-008: Read-only commands MUST be classified as Low risk
Commands like ls, cat, echo, pwd MUST return `CommandRiskLevel::Low`.

#### Scenario: Low-risk command detection
- WHEN `command_risk_level()` is called with ls, cat, echo, etc.
- THEN it returns `CommandRiskLevel::Low`
- Test: `command_risk_low_for_read_commands` in `src/security/policy.rs`

---

### Command Execution Gate (`src/security/policy.rs`)

### REQ-SEC-POL-009: validate_command_execution MUST enforce layered policy
Validation MUST follow: allowlist → risk classification → policy flags → autonomy level × approval.

#### Scenario: Medium-risk requires approval in Supervised
- WHEN a medium-risk command is validated in Supervised mode
- THEN it requires approval
- Test: `validate_command_requires_approval_for_medium_risk` in `src/security/policy.rs`

#### Scenario: High-risk blocked by default
- WHEN a high-risk command is validated with `block_high_risk_commands=true`
- THEN it is blocked with descriptive error
- Test: `validate_command_blocks_high_risk_by_default` in `src/security/policy.rs`

#### Scenario: Full mode skips medium-risk approval
- WHEN a medium-risk command is validated in Full mode
- THEN approval is not required
- Test: `validate_command_full_mode_skips_medium_risk_approval_gate` in `src/security/policy.rs`

#### Scenario: Background chain bypass rejected
- WHEN command contains background operator to bypass allowlist
- THEN it is rejected
- Test: `validate_command_rejects_background_chain_bypass` in `src/security/policy.rs`

#### Scenario: Forbidden paths rejected in execution gate
- WHEN command targets a forbidden path
- THEN `validate_command_execution` returns Err
- Test: `validate_command_execution_rejects_forbidden_paths` in `src/security/policy.rs`

---

### Command Injection Prevention (`src/security/policy.rs`)

### REQ-SEC-POL-010: Subshell operators MUST be blocked
Backticks, `$()`, `${}`, process substitution MUST be blocked. Quoted literals inside single quotes MUST be allowed.

#### Scenario: Backtick blocked
- WHEN command contains backtick subshell
- THEN it is blocked
- Test: `command_injection_backtick_blocked` in `src/security/policy.rs`

#### Scenario: Dollar-paren blocked
- WHEN command contains `$(...)` subshell
- THEN it is blocked
- Test: `command_injection_dollar_paren_blocked` in `src/security/policy.rs`

#### Scenario: Dollar-paren literal in single quotes allowed
- WHEN `$(...)` appears inside single quotes
- THEN it is allowed (literal, not executed)
- Test: `command_injection_dollar_paren_literal_inside_single_quotes_allowed` in `src/security/policy.rs`

#### Scenario: Dollar-brace literal in single quotes allowed
- WHEN `${...}` appears inside single quotes
- THEN it is allowed (literal, not executed)
- Test: `command_injection_dollar_brace_literal_inside_single_quotes_allowed` in `src/security/policy.rs`

#### Scenario: Dollar-brace unquoted blocked
- WHEN `${...}` appears unquoted
- THEN it is blocked
- Tests:
  - `command_injection_dollar_brace_unquoted_blocked` in `src/security/policy.rs`
  - `command_injection_dollar_brace_blocked` in `src/security/policy.rs`

#### Scenario: Plain dollar-variable blocked
- WHEN command contains `$VAR` expansion
- THEN it is blocked
- Test: `command_injection_plain_dollar_var_blocked` in `src/security/policy.rs`

#### Scenario: Process substitution blocked
- WHEN command contains `<(...)` or `>(...)`
- THEN it is blocked
- Test: `command_injection_process_substitution_blocked` in `src/security/policy.rs`

### REQ-SEC-POL-011: Shell redirections MUST be blocked
Unquoted `<` and `>` MUST be blocked to prevent path bypass. Quoted literals MUST be allowed.

#### Scenario: Redirect blocked
- WHEN command contains unquoted redirection
- THEN it is blocked
- Test: `command_injection_redirect_blocked` in `src/security/policy.rs`

#### Scenario: Quoted operators not treated as redirections
- WHEN `&` and `>` appear inside quotes
- THEN they are allowed as literals
- Test: `quoted_ampersand_and_redirect_literals_are_not_treated_as_operators` in `src/security/policy.rs`

### REQ-SEC-POL-012: Background operator (&) MUST be blocked
Single unquoted `&` MUST be blocked; `&&` MUST remain allowed.

#### Scenario: Background chain blocked
- WHEN command contains `&` (not `&&`)
- THEN it is blocked
- Test: `command_injection_background_chain_blocked` in `src/security/policy.rs`

### REQ-SEC-POL-013: Semicolons MUST split commands for validation
Unquoted semicolons MUST split commands. Quoted semicolons (e.g. in SQL) MUST NOT split.

#### Scenario: Semicolon splits blocked segments
- WHEN command contains unquoted semicolon before a dangerous command
- THEN it is blocked
- Tests:
  - `command_injection_semicolon_blocked` in `src/security/policy.rs`
  - `command_injection_semicolon_no_space` in `src/security/policy.rs`

#### Scenario: Quoted semicolons in SQL preserved
- WHEN semicolons appear inside quoted SQL strings
- THEN they do not cause splitting
- Test: `quoted_semicolons_do_not_split_sqlite_command` in `src/security/policy.rs`

#### Scenario: Unquoted semicolon after quoted SQL still splits
- WHEN an unquoted semicolon follows a quoted SQL block
- THEN the subsequent segment is validated separately
- Test: `unquoted_semicolon_after_quoted_sql_still_splits_commands` in `src/security/policy.rs`

### REQ-SEC-POL-014: Chain operators MUST validate all segments
Commands chained with `&&`, `||` MUST validate each segment independently.

#### Scenario: AND-chain blocked when any segment fails
- WHEN command uses `&&` with a blocked segment
- THEN it is blocked
- Test: `command_injection_and_chain_blocked` in `src/security/policy.rs`

#### Scenario: OR-chain blocked when any segment fails
- WHEN command uses `||` with a blocked segment
- THEN it is blocked
- Test: `command_injection_or_chain_blocked` in `src/security/policy.rs`

### REQ-SEC-POL-015: Newline injection MUST be blocked
Commands containing unquoted newlines MUST be blocked.

#### Scenario: Newline injection blocked
- WHEN command contains embedded newline
- THEN it is blocked
- Test: `command_newline_injection_blocked` in `src/security/policy.rs`

### REQ-SEC-POL-016: Environment variable prefixes MUST be handled
Commands with `ENV=val cmd` syntax MUST extract the actual command correctly.

#### Scenario: Env var prefix with command
- WHEN command has environment variable prefix
- THEN the actual command is extracted and validated
- Tests:
  - `command_with_env_var_prefix` in `src/security/policy.rs`
  - `command_env_var_prefix_with_allowed_cmd` in `src/security/policy.rs`

---

### Dangerous Arguments (`src/security/policy.rs`)

### REQ-SEC-POL-017: Dangerous arguments MUST be blocked
`find -exec`, `git config` writes, `git alias`, `tee` MUST be blocked. Git config reads MUST be allowed.

#### Scenario: Generic argument injection blocked
- WHEN command contains dangerous argument patterns
- THEN it is blocked
- Test: `command_argument_injection_blocked` in `src/security/policy.rs`

#### Scenario: Git config read operations allowed
- WHEN `git config` is called with read-only flags (`--get`, `--list`, etc.)
- THEN it is allowed
- Test: `git_config_readonly_operations_allowed` in `src/security/policy.rs`

#### Scenario: Git config write operations blocked
- WHEN `git config` is called with write flags
- THEN it is blocked
- Test: `git_config_write_operations_blocked` in `src/security/policy.rs`

#### Scenario: Git config mixed read-write flags blocked
- WHEN `git config` has both read and write flags
- THEN it is blocked (write takes precedence)
- Test: `git_config_mixed_read_write_flags_blocked` in `src/security/policy.rs`

#### Scenario: Git config global/injection flags blocked
- WHEN `git config` uses `--global`, `--system`, or `core.` injection
- THEN it is blocked
- Test: `git_config_global_injection_flags_blocked` in `src/security/policy.rs`

#### Scenario: Tee command blocked
- WHEN `tee` is used (can write to arbitrary files)
- THEN it is blocked
- Test: `command_injection_tee_blocked` in `src/security/policy.rs`

---

### Forbidden Path Arguments (`src/security/policy.rs`)

### REQ-SEC-POL-018: forbidden_path_argument MUST detect sensitive paths in arguments
Commands targeting forbidden paths (system dirs, sensitive dotfiles) MUST be detected.

#### Scenario: Absolute path to sensitive location
- WHEN command argument contains `/etc/passwd` or similar
- THEN `forbidden_path_argument()` returns `Some(path)`
- Test: `forbidden_path_argument_detects_absolute_path` in `src/security/policy.rs`

#### Scenario: Parent directory traversal reference
- WHEN command argument contains `../` traversal to sensitive path
- THEN it is detected
- Test: `forbidden_path_argument_detects_parent_dir_reference` in `src/security/policy.rs`

#### Scenario: Workspace-relative paths allowed
- WHEN command arguments reference workspace-relative paths
- THEN they are allowed
- Test: `forbidden_path_argument_allows_workspace_relative_paths` in `src/security/policy.rs`

#### Scenario: Option assignment paths (--file=/etc/passwd)
- WHEN `--option=/sensitive/path` pattern is detected
- THEN it is blocked
- Test: `forbidden_path_argument_detects_option_assignment_paths` in `src/security/policy.rs`

#### Scenario: Safe option assignment paths allowed
- WHEN `--option=/safe/path` pattern is detected
- THEN it is allowed
- Test: `forbidden_path_argument_allows_safe_option_assignment_paths` in `src/security/policy.rs`

#### Scenario: Short option attached paths (-f/etc/passwd)
- WHEN `-f/sensitive/path` pattern is detected
- THEN it is blocked
- Test: `forbidden_path_argument_detects_short_option_attached_paths` in `src/security/policy.rs`

#### Scenario: Safe short option attached paths allowed
- WHEN `-f/safe/path` pattern is detected
- THEN it is allowed
- Test: `forbidden_path_argument_allows_safe_short_option_attached_paths` in `src/security/policy.rs`

#### Scenario: Tilde user paths (~root/.bashrc)
- WHEN argument references `~user/` path to sensitive location
- THEN it is detected
- Test: `forbidden_path_argument_detects_tilde_user_paths` in `src/security/policy.rs`

#### Scenario: Input redirection paths
- WHEN `< /sensitive/path` pattern is detected
- THEN it is blocked
- Test: `forbidden_path_argument_detects_input_redirection_paths` in `src/security/policy.rs`

#### Scenario: Forbidden system paths blocked
- WHEN commands target system directories
- THEN they are blocked
- Test: `forbidden_paths_blocked` in `src/security/policy.rs`

---

### Path Security (`src/security/policy.rs`)

### REQ-SEC-POL-019: is_path_allowed MUST enforce workspace boundaries
Workspace-only mode blocks absolute paths and traversal. Relative and dotfile-in-workspace MUST be allowed.

#### Scenario: Relative paths allowed
- WHEN a relative path is checked
- THEN it is allowed
- Test: `relative_paths_allowed` in `src/security/policy.rs`

#### Scenario: Empty path allowed
- WHEN empty path is checked
- THEN it is allowed
- Test: `empty_path_allowed` in `src/security/policy.rs`

#### Scenario: Dotfile in workspace allowed
- WHEN `.gitignore` in workspace is checked
- THEN it is allowed
- Test: `dotfile_in_workspace_allowed` in `src/security/policy.rs`

#### Scenario: Path traversal blocked
- WHEN `../` traversal is detected
- THEN path is blocked
- Tests:
  - `path_traversal_blocked` in `src/security/policy.rs`
  - `path_traversal_encoded_dots` in `src/security/policy.rs`
  - `path_traversal_double_dot_in_filename` in `src/security/policy.rs`

#### Scenario: Absolute paths blocked in workspace-only
- WHEN workspace_only=true and absolute path is checked
- THEN it is blocked
- Test: `absolute_paths_blocked_when_workspace_only` in `src/security/policy.rs`

#### Scenario: Absolute paths allowed when not workspace-only
- WHEN workspace_only=false and absolute path is checked
- THEN it is allowed
- Test: `absolute_paths_allowed_when_not_workspace_only` in `src/security/policy.rs`

#### Scenario: Null byte injection blocked
- WHEN path contains null byte
- THEN it is blocked
- Tests:
  - `path_with_null_byte_blocked` in `src/security/policy.rs`
  - `is_path_allowed_blocks_null_bytes` in `src/security/policy.rs`

#### Scenario: Symlink-style absolute path blocked
- WHEN path appears to be symlink to system location
- THEN it is blocked
- Test: `path_symlink_style_absolute` in `src/security/policy.rs`

#### Scenario: Home tilde SSH path blocked
- WHEN `~/.ssh/` path is checked
- THEN it is blocked
- Test: `path_home_tilde_ssh` in `src/security/policy.rs`

#### Scenario: /var/run path blocked
- WHEN `/var/run/` path is checked
- THEN it is blocked
- Test: `path_var_run_blocked` in `src/security/policy.rs`

#### Scenario: URL-encoded traversal blocked
- WHEN path contains URL-encoded `..` sequences
- THEN it is blocked
- Test: `is_path_allowed_blocks_url_encoded_traversal` in `src/security/policy.rs`

### REQ-SEC-POL-020: is_resolved_path_allowed MUST enforce workspace + allowed_roots
Resolved paths MUST be within workspace or allowed_roots. Symlink escape, root escape MUST be blocked.

#### Scenario: Workspace-only allows resolved inside workspace
- WHEN workspace_only=false and resolved path is outside workspace
- THEN it is allowed
- Test: `workspace_only_false_allows_resolved_outside_workspace` in `src/security/policy.rs`

#### Scenario: Workspace-only blocks resolved outside workspace
- WHEN workspace_only=true and resolved path is outside workspace
- THEN it is blocked
- Test: `workspace_only_true_blocks_resolved_outside_workspace` in `src/security/policy.rs`

#### Scenario: Resolved path blocks outside workspace
- WHEN resolved path escapes workspace directory
- THEN it is blocked
- Test: `resolved_path_blocks_outside_workspace` in `src/security/policy.rs`

#### Scenario: Resolved path blocks root escape
- WHEN resolved path attempts root-level access
- THEN it is blocked
- Test: `resolved_path_blocks_root_escape` in `src/security/policy.rs`

#### Scenario: Resolved path blocks symlink escape
- WHEN symlink resolves to a path outside workspace
- THEN it is blocked
- Test: `resolved_path_blocks_symlink_escape` in `src/security/policy.rs`

#### Scenario: allowed_roots permits paths outside workspace
- WHEN path is outside workspace but within allowed_roots
- THEN it is permitted
- Test: `allowed_roots_permits_paths_outside_workspace` in `src/security/policy.rs`

#### Scenario: Violation message includes allowed roots guidance
- WHEN resolved path is blocked
- THEN error message includes allowed_roots for guidance
- Test: `resolved_path_violation_message_includes_allowed_roots_guidance` in `src/security/policy.rs`

#### Scenario: Full autonomy still respects forbidden paths
- WHEN autonomy is Full but path is forbidden
- THEN it is still blocked
- Test: `full_autonomy_still_respects_forbidden_paths` in `src/security/policy.rs`

---

### Security Checklist (`src/security/policy.rs`)

### REQ-SEC-POL-021: Comprehensive path security checklist MUST pass
A systematic checklist of path security invariants MUST hold.

#### Scenario: Root path blocked
- WHEN `/` is checked as path argument
- THEN it is blocked
- Test: `checklist_root_path_blocked` in `src/security/policy.rs`

#### Scenario: All system directories blocked
- WHEN `/etc`, `/usr`, `/bin`, `/sbin`, `/var`, `/proc`, `/sys`, `/dev` are checked
- THEN all are blocked
- Test: `checklist_all_system_dirs_blocked` in `src/security/policy.rs`

#### Scenario: Sensitive dotfiles blocked
- WHEN `.ssh`, `.gnupg`, `.aws`, `.env`, `.npmrc` paths are checked
- THEN all are blocked
- Test: `checklist_sensitive_dotfiles_blocked` in `src/security/policy.rs`

#### Scenario: Null byte injection blocked in checklist
- WHEN path contains null byte injection attempt
- THEN it is blocked
- Test: `checklist_null_byte_injection_blocked` in `src/security/policy.rs`

#### Scenario: Workspace-only blocks all absolute paths in checklist
- WHEN workspace_only=true and any absolute path is checked
- THEN it is blocked
- Test: `checklist_workspace_only_blocks_all_absolute` in `src/security/policy.rs`

#### Scenario: Resolved path must be in workspace
- WHEN resolved path is outside workspace
- THEN it is rejected
- Test: `checklist_resolved_path_must_be_in_workspace` in `src/security/policy.rs`

#### Scenario: Default policy is workspace-only
- WHEN default policy is constructed
- THEN workspace_only is true
- Test: `checklist_default_policy_is_workspace_only` in `src/security/policy.rs`

#### Scenario: Default forbidden paths comprehensive
- WHEN default policy forbidden_paths are checked
- THEN they cover all critical system paths
- Test: `checklist_default_forbidden_paths_comprehensive` in `src/security/policy.rs`

---

### Rate Limiting (`src/security/policy.rs`)

### REQ-SEC-POL-022: ActionTracker MUST use sliding window rate limiting
ActionTracker MUST count actions within a 1-hour window and prune older entries.

#### Scenario: Tracker starts at zero
- WHEN a new ActionTracker is created
- THEN count is zero
- Test: `action_tracker_starts_at_zero` in `src/security/policy.rs`

#### Scenario: Records increment count
- WHEN actions are recorded
- THEN count reflects the number of recorded actions
- Test: `action_tracker_records_actions` in `src/security/policy.rs`

#### Scenario: Within limit allowed
- WHEN actions recorded are within max_actions_per_hour
- THEN `record_action()` returns true
- Test: `record_action_allows_within_limit` in `src/security/policy.rs`

#### Scenario: Over limit blocked
- WHEN actions exceed max_actions_per_hour
- THEN `record_action()` returns false
- Test: `record_action_blocks_over_limit` in `src/security/policy.rs`

#### Scenario: is_rate_limited reflects count
- WHEN count is checked relative to limit
- THEN `is_rate_limited()` returns correct boolean
- Test: `is_rate_limited_reflects_count` in `src/security/policy.rs`

#### Scenario: Clone is independent
- WHEN ActionTracker is cloned
- THEN cloned tracker operates independently
- Test: `action_tracker_clone_is_independent` in `src/security/policy.rs`

#### Scenario: Exactly at boundary
- WHEN action count equals max_actions_per_hour exactly
- THEN next record is blocked (boundary is exclusive)
- Test: `rate_limit_exactly_at_boundary` in `src/security/policy.rs`

#### Scenario: Zero limit blocks everything
- WHEN max_actions_per_hour is 0
- THEN all actions are blocked
- Test: `rate_limit_zero_blocks_everything` in `src/security/policy.rs`

#### Scenario: High limit allows many
- WHEN max_actions_per_hour is very high
- THEN many actions are allowed
- Test: `rate_limit_high_allows_many` in `src/security/policy.rs`

---

### Config Construction (`src/security/policy.rs`)

### REQ-SEC-POL-023: from_config MUST map all config fields correctly
SecurityPolicy MUST be constructible from config with all fields mapped, roots normalized, and fresh tracker.

#### Scenario: All config fields mapped
- WHEN `SecurityPolicy::from_config()` is called with all fields set
- THEN each field is correctly populated
- Test: `from_config_maps_all_fields` in `src/security/policy.rs`

#### Scenario: Allowed roots normalized
- WHEN config contains allowed_roots paths
- THEN paths are normalized (trailing slashes, canonicalized)
- Test: `from_config_normalizes_allowed_roots` in `src/security/policy.rs`

#### Scenario: Fresh tracker created
- WHEN `from_config()` constructs a policy
- THEN a fresh ActionTracker with zero count is created
- Test: `from_config_creates_fresh_tracker` in `src/security/policy.rs`

---

### Heartbeat Summary (`src/security/policy.rs`)

### REQ-SEC-POL-024: summary_for_heartbeat MUST report key policy state
Heartbeat summary MUST include autonomy level, key policy flags, rate limit state.

#### Scenario: Summary contains key fields
- WHEN `summary_for_heartbeat()` is called
- THEN output contains autonomy level, workspace, rate limit info
- Test: `summary_for_heartbeat_contains_key_fields` in `src/security/policy.rs`

#### Scenario: Summary truncates long lists
- WHEN policy has many allowed commands
- THEN summary truncates the list
- Test: `summary_for_heartbeat_truncates_long_lists` in `src/security/policy.rs`

#### Scenario: Summary for Full autonomy
- WHEN autonomy is Full
- THEN summary reflects "full" level
- Test: `summary_for_heartbeat_full_autonomy` in `src/security/policy.rs`

#### Scenario: Summary for ReadOnly autonomy
- WHEN autonomy is ReadOnly
- THEN summary reflects "read_only" level
- Test: `summary_for_heartbeat_readonly_autonomy` in `src/security/policy.rs`

---

### Security Module Re-exports (`src/security/mod.rs`)

### REQ-SEC-MOD-001: SecurityPolicy and PairingGuard MUST be reachable via security module
Re-exported types MUST be constructible through the module namespace.

#### Scenario: Reexported types
- WHEN `SecurityPolicy::default()` and `PairingGuard::new()` are called via security module
- THEN they construct without error
- Test: `reexported_policy_and_pairing_types_are_usable` in `src/security/mod.rs`

### REQ-SEC-MOD-002: SecretStore encrypt/decrypt roundtrip MUST work
The reexported SecretStore MUST support encrypt→decrypt roundtrip.

#### Scenario: Secret store roundtrip
- WHEN `store.encrypt(value)` then `store.decrypt(cipher)` is called
- THEN original value is recovered
- Test: `reexported_secret_store_encrypt_decrypt_roundtrip` in `src/security/mod.rs`

### REQ-SEC-MOD-003: redact() MUST hide sensitive values
`redact()` MUST show first 4 chars + `***` for values > 4 chars, or just `***` for shorter values.

#### Scenario: Redact long and short values
- WHEN `redact("abcdefgh")` and `redact("ab")` are called
- THEN they return `"abcd***"` and `"***"` respectively
- Test: `redact_hides_most_of_value` in `src/security/mod.rs`

---

### Role-Based Access Control (`src/security/roles.rs`)

### REQ-SEC-ROLE-001: Built-in roles MUST enforce correct permissions
Owner/admin get wildcard access, operator denies memory_forget/users_manage/roles_manage and gates shell/browser/file_write with TOTP, viewer is read-only, guest has no access.

#### Scenario: Operator permissions gate shell with TOTP
- WHEN `resolve_tool_access("operator", "shell")` is called
- THEN allowed=true and requires_totp=true
- Test: `built_in_operator_permissions_gate_shell` in `src/security/roles.rs`

#### Scenario: Viewer is read-only
- WHEN `resolve_tool_access("viewer", "shell")` is called
- THEN allowed=false
- Test: `built_in_viewer_is_read_only` in `src/security/roles.rs`

### REQ-SEC-ROLE-002: Custom roles MUST support inheritance
Custom roles can inherit from built-in or other custom roles. TOTP gating and denied tools propagate through inheritance.

#### Scenario: Custom role inherits parent allowlist and TOTP
- WHEN a custom "developer" role inherits "operator"
- THEN inherited permissions and TOTP gates apply
- Test: `custom_role_inherits_parent_allowlist_and_totp` in `src/security/roles.rs`

### REQ-SEC-ROLE-003: Inheritance cycles MUST be rejected
If role A inherits B and B inherits A, `from_config` MUST return an error.

#### Scenario: Cycle detection
- WHEN two roles form a cycle
- THEN `from_config` returns Err containing "cycle"
- Test: `inheritance_cycle_is_rejected` in `src/security/roles.rs`

### REQ-SEC-ROLE-004: Owner MUST use global gated actions for TOTP
When `global_gated_actions` are configured, owner role MUST gate those tools with TOTP.

#### Scenario: Global TOTP gating
- WHEN `resolve_tool_access("owner", "shell")` is called with `global_gated_actions=["shell"]`
- THEN requires_totp=true
- Test: `owner_uses_global_gated_actions_for_totp` in `src/security/roles.rs`

### REQ-SEC-ROLE-005: Unknown role MUST deny access
Resolving access for an unregistered role MUST return allowed=false.

#### Scenario: Unknown role denied
- WHEN `resolve_tool_access("nonexistent", "shell")` is called
- THEN allowed=false
- Test: `unknown_role_denies_access` in `src/security/roles.rs`

### REQ-SEC-ROLE-006: Empty role/tool names MUST deny access
Empty or whitespace-only role or tool names MUST return allowed=false.

#### Scenario: Empty names denied
- WHEN `resolve_tool_access("", "shell")` or `resolve_tool_access("owner", "")` is called
- THEN allowed=false
- Test: `empty_role_or_tool_denies_access` in `src/security/roles.rs`

---

### Emergency Stop (`src/security/estop.rs`)

### REQ-SEC-ESTOP-001: EstopManager MUST support multi-level engagement
KillAll, NetworkKill, DomainBlock, and ToolFreeze levels MUST compose independently.

#### Scenario: Multi-level compose and resume
- WHEN multiple levels are engaged then selectively resumed
- THEN each level operates independently
- Test: `estop_levels_compose_and_resume` in `src/security/estop.rs`

### REQ-SEC-ESTOP-002: Estop state MUST persist and reload across sessions
State MUST be persisted to JSON file and correctly reloaded.

#### Scenario: State persistence
- WHEN estop is engaged, then EstopManager is recreated from same path
- THEN reloaded state matches engaged state
- Test: `estop_state_survives_reload` in `src/security/estop.rs`

### REQ-SEC-ESTOP-003: Corrupted state MUST trigger fail-closed mode
If state file is corrupted/unparseable, estop MUST enter kill_all=true mode.

#### Scenario: Corrupted state
- WHEN state file contains invalid JSON
- THEN EstopManager enters fail-closed (kill_all=true)
- Test: `corrupted_state_defaults_to_fail_closed_kill_all` in `src/security/estop.rs`

### REQ-SEC-ESTOP-004: Resume with OTP-required MUST validate OTP code
When `require_otp_to_resume` is true, resume MUST require a valid OTP code.

#### Scenario: Resume requires OTP
- WHEN resume is called without OTP and `require_otp_to_resume=true`
- THEN it returns Err
- Test: `resume_requires_valid_otp_when_enabled` in `src/security/estop.rs`

#### Scenario: Resume with valid OTP
- WHEN resume is called with valid OTP code
- THEN it succeeds
- Test: `resume_accepts_valid_otp_code` in `src/security/estop.rs`

### REQ-SEC-ESTOP-005: EstopState MUST correctly report engagement status
`is_engaged()` MUST return true when any level is active (kill_all, network_kill, blocked_domains, frozen_tools).

#### Scenario: Engaged when any level active
- WHEN EstopState has any active level
- THEN `is_engaged()` returns true
- Test: `estop_state_is_engaged_when_any_level_active` in `src/security/estop.rs`

#### Scenario: Default state is not engaged
- WHEN `EstopState::default()` is created
- THEN `is_engaged()` returns false
- Test: `estop_state_default_is_not_engaged` in `src/security/estop.rs`

---

## Mock Strategy
Policy tests use direct struct construction with `SecurityPolicy::default()` and field overrides. Estop tests use `tempfile::tempdir()` for state file isolation. Roles use `SecurityRoleConfig` structs. No external service dependencies.

## Coverage Notes
- `src/security/policy.rs`: 112 tests — all command parsing, risk, allowlist, path, injection, rate limit paths covered
- `src/security/mod.rs`: 3 tests — re-export verification
- `src/security/roles.rs`: 7 tests (5 existing + 2 gap) — built-in roles, inheritance, cycles, edge cases
- `src/security/estop.rs`: 7 tests (5 existing + 2 gap) — levels, persistence, fail-closed, OTP, engagement
