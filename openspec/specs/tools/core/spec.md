# Tools Core Specification

## Purpose

Define requirements for the core tool implementations: shell, file_read, file_write, file_edit, glob_search, content_search, process, bg_run, and the Tool trait contract.

## Scope

- Files: `src/tools/traits.rs`, `src/tools/mod.rs`, `src/tools/shell.rs`, `src/tools/file_read.rs`, `src/tools/file_write.rs`, `src/tools/file_edit.rs`, `src/tools/glob_search.rs`, `src/tools/content_search.rs`, `src/tools/process.rs`, `src/tools/bg_run.rs`
- Risk tier: HIGH (tools execute with real-world side effects; shell has command injection risk)
- Existing tests: 185 across 10 files (traits: 5, mod: 18, shell: 31, file_read: 29, file_write: 18, file_edit: 21, glob_search: 15, content_search: 22, process: 19, bg_run: 7)

---

## Requirements

### REQ-TRAIT-001: Tool Trait Contract

All tools MUST implement the `Tool` trait with valid name, description, parameters_schema, and execute.

#### Scenario: Spec uses tool metadata and schema

- WHEN `spec()` is called on a `DummyTool`
- THEN MUST return a `ToolSpec` whose name, description, and parameters match the trait methods
- Test: `spec_uses_tool_metadata_and_schema` in `src/tools/traits.rs`

#### Scenario: Execute returns expected output

- WHEN `execute` is called on a `DummyTool` with a `name` argument
- THEN MUST return a `ToolResult` with success=true and expected output
- Test: `execute_returns_expected_output` in `src/tools/traits.rs`

#### Scenario: ToolSpec serialization roundtrip

- WHEN a `ToolSpec` is serialized to JSON and deserialized back
- THEN the round-tripped value MUST equal the original
- Test: `tool_spec_serialization_roundtrip` in `src/tools/traits.rs`

#### Scenario: ToolResult success without error

- WHEN a `ToolResult` is constructed with success=true and no error
- THEN `error` MUST be `None` and `success` MUST be true
- Test: `tool_result_success_without_error` in `src/tools/traits.rs`

#### Scenario: ToolResult serialization roundtrip

- WHEN a `ToolResult` is serialized to JSON and deserialized back
- THEN the round-tripped value MUST equal the original
- Test: `tool_result_serialization_roundtrip` in `src/tools/traits.rs`

---

### REQ-REGISTRY-001: Tool Registry and Factory

The tool registry MUST compose the correct set of tools based on configuration and runtime.

#### Scenario: Default tools have expected count

- WHEN `default_tools` is called with default security
- THEN MUST return exactly 7 tools (including apply_patch)
- Test: `default_tools_has_expected_count` in `src/tools/mod.rs`

#### Scenario: Default tools contain expected names

- WHEN `default_tools` is called
- THEN MUST include shell, file_read, file_write, file_edit, glob_search, content_search
- Test: `default_tools_names` in `src/tools/mod.rs`

#### Scenario: All default tools have descriptions

- WHEN `default_tools` is called
- THEN every tool MUST have a non-empty description
- Test: `default_tools_all_have_descriptions` in `src/tools/mod.rs`

#### Scenario: All default tools have valid schemas

- WHEN `default_tools` is called
- THEN every tool schema MUST be an object with a `properties` sub-object
- Test: `default_tools_all_have_schemas` in `src/tools/mod.rs`

#### Scenario: Tool spec generation via registry

- WHEN `spec()` is called on each default tool
- THEN MUST return matching name, description, and object-typed parameters
- Test: `tool_spec_generation` in `src/tools/mod.rs`

#### Scenario: ToolResult success serde

- WHEN a successful `ToolResult` is serialized and deserialized
- THEN fields MUST round-trip correctly
- Test: `tool_result_serde` in `src/tools/mod.rs`

#### Scenario: ToolResult error serde

- WHEN a failed `ToolResult` with error is serialized and deserialized
- THEN error message MUST round-trip correctly
- Test: `tool_result_with_error_serde` in `src/tools/mod.rs`

#### Scenario: ToolSpec serde

- WHEN a `ToolSpec` is serialized and deserialized
- THEN name and description MUST round-trip correctly
- Test: `tool_spec_serde` in `src/tools/mod.rs`

---

### REQ-REGISTRY-002: Runtime-Aware Tool Composition

The registry MUST adapt tool sets based on runtime adapter type.

#### Scenario: Wasm runtime includes wasm_module in default_tools_with_runtime

- WHEN `default_tools_with_runtime` is called with a WasmRuntime
- THEN MUST include wasm_module
- Test: `default_tools_with_runtime_includes_wasm_module_for_wasm_runtime` in `src/tools/mod.rs`

#### Scenario: Wasm runtime excludes shell and filesystem tools

- WHEN `default_tools_with_runtime` is called with a WasmRuntime
- THEN MUST NOT include shell, file_read, file_write, file_edit, apply_patch, glob_search, content_search
- Test: `default_tools_with_runtime_excludes_shell_and_fs_for_wasm_runtime` in `src/tools/mod.rs`

#### Scenario: Wasm runtime includes wasm_module in all_tools_with_runtime

- WHEN `all_tools_with_runtime` is called with a WasmRuntime
- THEN MUST include wasm_module and exclude shell, process, git_operations, file tools, openclaw_migration
- Test: `all_tools_with_runtime_includes_wasm_module_for_wasm_runtime` in `src/tools/mod.rs`

#### Scenario: Native runtime includes background tools

- WHEN `all_tools_with_runtime` is called with a NativeRuntime
- THEN MUST include bg_run and bg_status
- Test: `all_tools_with_runtime_includes_background_tools` in `src/tools/mod.rs`

---

### REQ-REGISTRY-003: Conditional Tool Inclusion

The registry MUST include or exclude tools based on feature configuration.

#### Scenario: Browser excluded when disabled

- WHEN `all_tools` is called with browser disabled
- THEN MUST NOT include browser_open, MUST include schedule, model_routing_config, pushover, proxy_config, web_access_config, web_search_config, openclaw_migration
- Test: `all_tools_excludes_browser_when_disabled` in `src/tools/mod.rs`

#### Scenario: Browser included when enabled

- WHEN `all_tools` is called with browser enabled
- THEN MUST include browser_open, content_search, model_routing_config, pushover, proxy_config, web_access_config, web_search_config, openclaw_migration
- Test: `all_tools_includes_browser_when_enabled` in `src/tools/mod.rs`

#### Scenario: Docx and PDF read tools included

- WHEN `all_tools` is called
- THEN MUST include docx_read and pdf_read
- Test: `all_tools_includes_docx_read_tool` in `src/tools/mod.rs`

#### Scenario: Delegate tools included when agents configured

- WHEN `all_tools` is called with agent configs present
- THEN MUST include delegate and delegate_coordination_status
- Test: `all_tools_includes_delegate_when_agents_configured` in `src/tools/mod.rs`

#### Scenario: Delegate tools excluded when no agents

- WHEN `all_tools` is called with empty agent configs
- THEN MUST NOT include delegate or delegate_coordination_status
- Test: `all_tools_excludes_delegate_when_no_agents` in `src/tools/mod.rs`

#### Scenario: Coordination tool disabled when coordination config is disabled

- WHEN `all_tools` is called with coordination.enabled=false
- THEN MUST NOT include delegate_coordination_status even when agents are configured
- Test: `all_tools_disables_coordination_tool_when_coordination_is_disabled` in `src/tools/mod.rs`

---

### REQ-SHELL-001: Shell Tool Identity

The shell tool MUST have a stable name, description, and schema.

#### Scenario: Shell tool name

- WHEN `name()` is called
- THEN MUST return "shell"
- Test: `shell_tool_name` in `src/tools/shell.rs`

#### Scenario: Shell tool description

- WHEN `description()` is called
- THEN MUST return a non-empty description
- Test: `shell_tool_description` in `src/tools/shell.rs`

#### Scenario: Shell tool schema has command parameter

- WHEN `parameters_schema()` is called
- THEN MUST include a "command" property of type string and it MUST be required
- Test: `shell_tool_schema_has_command` in `src/tools/shell.rs`

---

### REQ-SHELL-002: Shell Command Execution

The shell tool MUST execute allowed commands and capture output.

#### Scenario: Execute allowed command

- WHEN a valid allowed command is provided
- THEN MUST execute and return successful result with stdout
- Test: `shell_executes_allowed_command` in `src/tools/shell.rs`

#### Scenario: Execute command from cmd alias

- WHEN a command is provided via the "cmd" alias parameter
- THEN MUST execute the command the same as with "command"
- Test: `shell_executes_command_from_cmd_alias` in `src/tools/shell.rs`

#### Scenario: Extract command argument supports aliases

- WHEN args contain "command", "cmd", or "script" keys
- THEN `extract_command_argument` MUST extract the value from any alias
- Test: `extract_command_argument_supports_aliases` in `src/tools/shell.rs`

#### Scenario: Capture exit code

- WHEN a command exits with a non-zero code
- THEN MUST include exit code in output
- Test: `shell_captures_exit_code` in `src/tools/shell.rs`

#### Scenario: Capture stderr output

- WHEN a command writes to stderr
- THEN MUST capture and include stderr in output
- Test: `shell_captures_stderr_output` in `src/tools/shell.rs`

#### Scenario: Handle nonexistent command

- WHEN a nonexistent command is executed
- THEN MUST return an error result without panic
- Test: `shell_handles_nonexistent_command` in `src/tools/shell.rs`

#### Scenario: Missing command parameter

- WHEN no command parameter is provided
- THEN MUST return an error
- Test: `shell_missing_command_param` in `src/tools/shell.rs`

#### Scenario: Wrong type parameter

- WHEN command parameter is not a string
- THEN MUST return an error
- Test: `shell_wrong_type_param` in `src/tools/shell.rs`

---

### REQ-SHELL-003: Shell Security Policy Enforcement

The shell tool MUST enforce security policy restrictions.

#### Scenario: Block disallowed command

- WHEN a command is not in the allowed list
- THEN MUST reject execution
- Test: `shell_blocks_disallowed_command` in `src/tools/shell.rs`

#### Scenario: Block in readonly mode

- WHEN security policy is in readonly mode
- THEN MUST block shell execution
- Test: `shell_blocks_readonly` in `src/tools/shell.rs`

#### Scenario: Block when rate limited

- WHEN rate limit budget is exhausted
- THEN MUST reject the command
- Test: `shell_blocks_rate_limited` in `src/tools/shell.rs`

#### Scenario: Require approval for medium risk command

- WHEN a command requires medium risk approval
- THEN MUST require explicit approval before execution
- Test: `shell_requires_approval_for_medium_risk_command` in `src/tools/shell.rs`

#### Scenario: Record action budget exhaustion

- WHEN action budget is exhausted
- THEN MUST block execution and record the exhaustion event
- Test: `shell_record_action_budget_exhaustion` in `src/tools/shell.rs`

---

### REQ-SHELL-004: Shell Path Injection Prevention

The shell tool MUST block path injection attempts in arguments.

#### Scenario: Block absolute path argument

- WHEN a command argument contains an absolute path
- THEN MUST block execution
- Test: `shell_blocks_absolute_path_argument` in `src/tools/shell.rs`

#### Scenario: Block option assignment path argument

- WHEN a command argument uses option=path format with absolute path
- THEN MUST block execution
- Test: `shell_blocks_option_assignment_path_argument` in `src/tools/shell.rs`

#### Scenario: Block short option attached path argument

- WHEN a command argument uses short option with attached absolute path
- THEN MUST block execution
- Test: `shell_blocks_short_option_attached_path_argument` in `src/tools/shell.rs`

#### Scenario: Block tilde user path argument

- WHEN a command argument uses ~user path expansion
- THEN MUST block execution
- Test: `shell_blocks_tilde_user_path_argument` in `src/tools/shell.rs`

#### Scenario: Block input redirection path bypass

- WHEN a command uses input redirection with path
- THEN MUST block execution
- Test: `shell_blocks_input_redirection_path_bypass` in `src/tools/shell.rs`

---

### REQ-SHELL-005: Shell Environment Variable Safety

The shell tool MUST sanitize environment variables to prevent secret leakage.

#### Scenario: Does not leak API key

- WHEN an API key is set in the environment
- THEN MUST NOT pass it to the child process
- Test: `shell_does_not_leak_api_key` in `src/tools/shell.rs`

#### Scenario: Preserves PATH and HOME for env command

- WHEN the shell tool is configured to allow env command
- THEN MUST preserve PATH and HOME environment variables
- Test: `shell_preserves_path_and_home_for_env_command` in `src/tools/shell.rs`

#### Scenario: Blocks plain variable expansion

- WHEN a command attempts to expand environment variables
- THEN MUST block the expansion of sensitive variables
- Test: `shell_blocks_plain_variable_expansion` in `src/tools/shell.rs`

#### Scenario: Allows configured env passthrough

- WHEN specific environment variables are configured for passthrough
- THEN MUST pass only those configured variables
- Test: `shell_allows_configured_env_passthrough` in `src/tools/shell.rs`

#### Scenario: Invalid env passthrough names are filtered

- WHEN env passthrough configuration contains invalid variable names
- THEN MUST filter out invalid names
- Test: `invalid_shell_env_passthrough_names_are_filtered` in `src/tools/shell.rs`

#### Scenario: Safe env vars excludes secrets

- WHEN safe environment variable list is checked
- THEN MUST NOT include any secret-bearing variable names
- Test: `shell_safe_env_vars_excludes_secrets` in `src/tools/shell.rs`

#### Scenario: Safe env vars includes essentials

- WHEN safe environment variable list is checked
- THEN MUST include essential variables like PATH and HOME
- Test: `shell_safe_env_vars_includes_essentials` in `src/tools/shell.rs`

---

### REQ-SHELL-006: Shell Constants and Limits

The shell tool MUST enforce reasonable timeout and output limits.

#### Scenario: Timeout constant is reasonable

- WHEN the shell timeout constant is checked
- THEN MUST be within a reasonable range
- Test: `shell_timeout_constant_is_reasonable` in `src/tools/shell.rs`

#### Scenario: Output limit is 1MB

- WHEN the shell output limit constant is checked
- THEN MUST be 1MB
- Test: `shell_output_limit_is_1mb` in `src/tools/shell.rs`

---

### REQ-SHELL-007: Shell Syscall Anomaly Detection

The shell tool MUST integrate with the syscall anomaly detector.

#### Scenario: Syscall detector writes anomaly log

- WHEN a shell command triggers syscall anomaly detection
- THEN MUST write anomaly data to the detector log
- Test: `shell_syscall_detector_writes_anomaly_log` in `src/tools/shell.rs`

---

### REQ-FILE-READ-001: File Read Tool Identity

The file_read tool MUST have a stable name and schema.

#### Scenario: File read name

- WHEN `name()` is called
- THEN MUST return "file_read"
- Test: `file_read_name` in `src/tools/file_read.rs`

#### Scenario: File read schema has path

- WHEN `parameters_schema()` is called
- THEN MUST include a "path" property and it MUST be required
- Test: `file_read_schema_has_path` in `src/tools/file_read.rs`

---

### REQ-FILE-READ-002: File Read Operations

The file_read tool MUST read files correctly with various options.

#### Scenario: Read existing file

- WHEN a valid path to an existing file is provided
- THEN MUST return the file contents
- Test: `file_read_existing_file` in `src/tools/file_read.rs`

#### Scenario: Read nonexistent file

- WHEN a path to a nonexistent file is provided
- THEN MUST return an error result
- Test: `file_read_nonexistent_file` in `src/tools/file_read.rs`

#### Scenario: Read empty file

- WHEN a path to an empty file is provided
- THEN MUST return successful result with empty content
- Test: `file_read_empty_file` in `src/tools/file_read.rs`

#### Scenario: Read nested path

- WHEN a path to a file in nested directories is provided
- THEN MUST return the file contents
- Test: `file_read_nested_path` in `src/tools/file_read.rs`

#### Scenario: Read with offset and limit

- WHEN offset and limit parameters are provided
- THEN MUST return only the specified range of lines
- Test: `file_read_with_offset_and_limit` in `src/tools/file_read.rs`

#### Scenario: Offset beyond end of file

- WHEN offset is beyond the end of the file
- THEN MUST return empty or appropriate result
- Test: `file_read_offset_beyond_end` in `src/tools/file_read.rs`

#### Scenario: Missing path parameter

- WHEN no path parameter is provided
- THEN MUST return an error
- Test: `file_read_missing_path_param` in `src/tools/file_read.rs`

#### Scenario: Allows readonly mode

- WHEN security policy is in readonly mode
- THEN MUST allow file reading
- Test: `file_read_allows_readonly_mode` in `src/tools/file_read.rs`

---

### REQ-FILE-READ-003: File Read Security

The file_read tool MUST enforce path security restrictions.

#### Scenario: Block path traversal

- WHEN path contains `..` traversal
- THEN MUST reject the request
- Test: `file_read_blocks_path_traversal` in `src/tools/file_read.rs`

#### Scenario: Block absolute path

- WHEN an absolute path is provided
- THEN MUST reject the request
- Test: `file_read_blocks_absolute_path` in `src/tools/file_read.rs`

#### Scenario: Block sensitive .env file by default

- WHEN path targets a `.env` file
- THEN MUST reject the request by default
- Test: `file_read_blocks_sensitive_env_file_by_default` in `src/tools/file_read.rs`

#### Scenario: Block sensitive dotenv variant by default

- WHEN path targets a dotenv variant file
- THEN MUST reject the request by default
- Test: `file_read_blocks_sensitive_dotenv_variant_by_default` in `src/tools/file_read.rs`

#### Scenario: Block sensitive directory credentials by default

- WHEN path targets credential files in sensitive directories
- THEN MUST reject the request by default
- Test: `file_read_blocks_sensitive_directory_credentials_by_default` in `src/tools/file_read.rs`

#### Scenario: Allow sensitive file when policy enabled

- WHEN sensitive file access is explicitly enabled in policy
- THEN MUST allow reading the sensitive file
- Test: `file_read_allows_sensitive_file_when_policy_enabled` in `src/tools/file_read.rs`

#### Scenario: Allow sensitive nested path when policy enabled

- WHEN sensitive file access is enabled and path is nested
- THEN MUST allow reading the sensitive nested file
- Test: `file_read_allows_sensitive_nested_path_when_policy_enabled` in `src/tools/file_read.rs`

#### Scenario: Block symlink escape

- WHEN path resolves through a symlink that escapes workspace
- THEN MUST reject the request
- Test: `file_read_blocks_symlink_escape` in `src/tools/file_read.rs`

#### Scenario: Block hardlink escape

- WHEN path is a hardlink pointing outside workspace
- THEN MUST reject the request
- Test: `file_read_blocks_hardlink_escape` in `src/tools/file_read.rs`

#### Scenario: Block null byte in path

- WHEN path contains a null byte
- THEN MUST reject the request
- Test: `file_read_blocks_null_byte_in_path` in `src/tools/file_read.rs`

#### Scenario: Block when rate limited

- WHEN rate limit budget is exhausted
- THEN MUST reject the request
- Test: `file_read_blocks_when_rate_limited` in `src/tools/file_read.rs`

#### Scenario: Allow outside workspace when workspace_only disabled

- WHEN workspace_only is disabled in security policy
- THEN MUST allow reading files outside the workspace
- Test: `file_read_outside_workspace_allowed_when_workspace_only_disabled` in `src/tools/file_read.rs`

#### Scenario: Nonexistent file consumes rate limit budget

- WHEN a nonexistent file path is requested
- THEN MUST still consume rate limit budget
- Test: `file_read_nonexistent_consumes_rate_limit_budget` in `src/tools/file_read.rs`

---

### REQ-FILE-READ-004: File Read Size and Format Limits

The file_read tool MUST enforce file size limits and handle binary/PDF content.

#### Scenario: Reject oversized file

- WHEN file exceeds the maximum allowed size
- THEN MUST reject the read with appropriate error
- Test: `file_read_rejects_oversized_file` in `src/tools/file_read.rs`

#### Scenario: Extract PDF text

- WHEN file is a PDF
- THEN MUST extract text content from the PDF
- Test: `file_read_extracts_pdf_text` in `src/tools/file_read.rs`

#### Scenario: Lossy read of binary file

- WHEN file is binary
- THEN MUST perform lossy UTF-8 conversion instead of failing
- Test: `file_read_lossy_reads_binary_file` in `src/tools/file_read.rs`

---

### REQ-FILE-READ-005: File Read End-to-End Agent Integration

End-to-end tests MUST verify file_read works through the full agent loop.

#### Scenario: E2E agent file read PDF extraction

- WHEN the agent loop processes a file_read tool call for a PDF file
- THEN MUST extract text content via the tool and return it to the agent
- Test: `e2e_agent_file_read_pdf_extraction` in `src/tools/file_read.rs`

#### Scenario: E2E agent file read lossy binary

- WHEN the agent loop processes a file_read tool call for a binary file
- THEN MUST perform lossy read and return content to the agent
- Test: `e2e_agent_file_read_lossy_binary` in `src/tools/file_read.rs`

---

### REQ-FILE-WRITE-001: File Write Tool Identity

The file_write tool MUST have a stable name and schema.

#### Scenario: File write name

- WHEN `name()` is called
- THEN MUST return "file_write"
- Test: `file_write_name` in `src/tools/file_write.rs`

#### Scenario: File write schema has path and content

- WHEN `parameters_schema()` is called
- THEN MUST include "path" and "content" properties
- Test: `file_write_schema_has_path_and_content` in `src/tools/file_write.rs`

---

### REQ-FILE-WRITE-002: File Write Operations

The file_write tool MUST create, overwrite, and manage files.

#### Scenario: Create new file

- WHEN a path to a non-existing file with content is provided
- THEN MUST create the file with the given content
- Test: `file_write_creates_file` in `src/tools/file_write.rs`

#### Scenario: Create parent directories

- WHEN a path contains non-existing parent directories
- THEN MUST create parent directories before writing
- Test: `file_write_creates_parent_dirs` in `src/tools/file_write.rs`

#### Scenario: Overwrite existing file

- WHEN a path to an existing file is provided
- THEN MUST overwrite with new content
- Test: `file_write_overwrites_existing` in `src/tools/file_write.rs`

#### Scenario: Write empty content

- WHEN content is an empty string
- THEN MUST create an empty file
- Test: `file_write_empty_content` in `src/tools/file_write.rs`

#### Scenario: Missing path parameter

- WHEN no path parameter is provided
- THEN MUST return an error
- Test: `file_write_missing_path_param` in `src/tools/file_write.rs`

#### Scenario: Missing content parameter

- WHEN no content parameter is provided
- THEN MUST return an error
- Test: `file_write_missing_content_param` in `src/tools/file_write.rs`

---

### REQ-FILE-WRITE-003: File Write Security

The file_write tool MUST enforce path and mode security restrictions.

#### Scenario: Block path traversal

- WHEN path contains `..` traversal
- THEN MUST reject the request
- Test: `file_write_blocks_path_traversal` in `src/tools/file_write.rs`

#### Scenario: Block absolute path

- WHEN an absolute path is provided
- THEN MUST reject the request
- Test: `file_write_blocks_absolute_path` in `src/tools/file_write.rs`

#### Scenario: Block sensitive file by default

- WHEN path targets a sensitive file (e.g. .env)
- THEN MUST reject the request by default
- Test: `file_write_blocks_sensitive_file_by_default` in `src/tools/file_write.rs`

#### Scenario: Allow sensitive file when configured

- WHEN sensitive file access is explicitly enabled in policy
- THEN MUST allow writing the sensitive file
- Test: `file_write_allows_sensitive_file_when_configured` in `src/tools/file_write.rs`

#### Scenario: Block symlink escape

- WHEN path resolves through a symlink that escapes workspace
- THEN MUST reject the request
- Test: `file_write_blocks_symlink_escape` in `src/tools/file_write.rs`

#### Scenario: Block symlink target file

- WHEN path is a symlink pointing to a file outside workspace
- THEN MUST reject the write
- Test: `file_write_blocks_symlink_target_file` in `src/tools/file_write.rs`

#### Scenario: Block hardlink target file

- WHEN path is a hardlink pointing to a file outside workspace
- THEN MUST reject the write
- Test: `file_write_blocks_hardlink_target_file` in `src/tools/file_write.rs`

#### Scenario: Block null byte in path

- WHEN path contains a null byte
- THEN MUST reject the request
- Test: `file_write_blocks_null_byte_in_path` in `src/tools/file_write.rs`

#### Scenario: Block in readonly mode

- WHEN security policy is in readonly mode
- THEN MUST block file writing
- Test: `file_write_blocks_readonly_mode` in `src/tools/file_write.rs`

#### Scenario: Block when rate limited

- WHEN rate limit budget is exhausted
- THEN MUST reject the request
- Test: `file_write_blocks_when_rate_limited` in `src/tools/file_write.rs`

---

### REQ-FILE-EDIT-001: File Edit Tool Identity

The file_edit tool MUST have a stable name and schema.

#### Scenario: File edit name

- WHEN `name()` is called
- THEN MUST return "file_edit"
- Test: `file_edit_name` in `src/tools/file_edit.rs`

#### Scenario: File edit schema has required params

- WHEN `parameters_schema()` is called
- THEN MUST include path, old_string, and new_string properties
- Test: `file_edit_schema_has_required_params` in `src/tools/file_edit.rs`

---

### REQ-FILE-EDIT-002: File Edit Operations

The file_edit tool MUST perform string replacements in files.

#### Scenario: Replace single match

- WHEN old_string matches exactly once in the file
- THEN MUST replace it with new_string
- Test: `file_edit_replaces_single_match` in `src/tools/file_edit.rs`

#### Scenario: Old string not found

- WHEN old_string does not match any content in the file
- THEN MUST return an error indicating no match
- Test: `file_edit_not_found` in `src/tools/file_edit.rs`

#### Scenario: Multiple matches

- WHEN old_string matches more than once
- THEN MUST return an error indicating ambiguity (or replace all, depending on mode)
- Test: `file_edit_multiple_matches` in `src/tools/file_edit.rs`

#### Scenario: Delete via empty new_string

- WHEN new_string is empty
- THEN MUST delete the matched old_string
- Test: `file_edit_delete_via_empty_new_string` in `src/tools/file_edit.rs`

#### Scenario: Nonexistent file

- WHEN path points to a file that does not exist
- THEN MUST return an error
- Test: `file_edit_nonexistent_file` in `src/tools/file_edit.rs`

#### Scenario: Reject empty old_string

- WHEN old_string is empty
- THEN MUST return an error
- Test: `file_edit_rejects_empty_old_string` in `src/tools/file_edit.rs`

#### Scenario: Missing path parameter

- WHEN no path parameter is provided
- THEN MUST return an error
- Test: `file_edit_missing_path_param` in `src/tools/file_edit.rs`

#### Scenario: Missing old_string parameter

- WHEN no old_string parameter is provided
- THEN MUST return an error
- Test: `file_edit_missing_old_string_param` in `src/tools/file_edit.rs`

#### Scenario: Missing new_string parameter

- WHEN no new_string parameter is provided
- THEN MUST return an error
- Test: `file_edit_missing_new_string_param` in `src/tools/file_edit.rs`

---

### REQ-FILE-EDIT-003: File Edit Security

The file_edit tool MUST enforce path and mode security restrictions.

#### Scenario: Block sensitive file by default

- WHEN path targets a sensitive file (e.g. .env)
- THEN MUST reject the request by default
- Test: `file_edit_blocks_sensitive_file_by_default` in `src/tools/file_edit.rs`

#### Scenario: Allow sensitive file when configured

- WHEN sensitive file access is explicitly enabled in policy
- THEN MUST allow editing the sensitive file
- Test: `file_edit_allows_sensitive_file_when_configured` in `src/tools/file_edit.rs`

#### Scenario: Block path traversal

- WHEN path contains `..` traversal
- THEN MUST reject the request
- Test: `file_edit_blocks_path_traversal` in `src/tools/file_edit.rs`

#### Scenario: Block absolute path

- WHEN an absolute path is provided
- THEN MUST reject the request
- Test: `file_edit_blocks_absolute_path` in `src/tools/file_edit.rs`

#### Scenario: Block symlink escape

- WHEN path resolves through a symlink that escapes workspace
- THEN MUST reject the request
- Test: `file_edit_blocks_symlink_escape` in `src/tools/file_edit.rs`

#### Scenario: Block symlink target file

- WHEN path is a symlink pointing to a file outside workspace
- THEN MUST reject the edit
- Test: `file_edit_blocks_symlink_target_file` in `src/tools/file_edit.rs`

#### Scenario: Block hardlink target file

- WHEN path is a hardlink pointing to a file outside workspace
- THEN MUST reject the edit
- Test: `file_edit_blocks_hardlink_target_file` in `src/tools/file_edit.rs`

#### Scenario: Block null byte in path

- WHEN path contains a null byte
- THEN MUST reject the request
- Test: `file_edit_blocks_null_byte_in_path` in `src/tools/file_edit.rs`

#### Scenario: Block in readonly mode

- WHEN security policy is in readonly mode
- THEN MUST block file editing
- Test: `file_edit_blocks_readonly_mode` in `src/tools/file_edit.rs`

#### Scenario: Block when rate limited

- WHEN rate limit budget is exhausted
- THEN MUST reject the request
- Test: `file_edit_blocks_when_rate_limited` in `src/tools/file_edit.rs`

---

### REQ-GLOB-001: Glob Search Tool Identity and Schema

The glob_search tool MUST have a stable name and schema.

#### Scenario: Glob search name and schema

- WHEN `name()` and `parameters_schema()` are called
- THEN MUST return "glob_search" and a schema with pattern property
- Test: `glob_search_name_and_schema` in `src/tools/glob_search.rs`

---

### REQ-GLOB-002: Glob Search Operations

The glob_search tool MUST find files matching glob patterns.

#### Scenario: Single file match

- WHEN a pattern matches exactly one file
- THEN MUST return that file path
- Test: `glob_search_single_file` in `src/tools/glob_search.rs`

#### Scenario: Multiple file matches

- WHEN a pattern matches multiple files
- THEN MUST return all matching file paths
- Test: `glob_search_multiple_files` in `src/tools/glob_search.rs`

#### Scenario: Recursive search

- WHEN a recursive glob pattern is used (e.g. `**/*.rs`)
- THEN MUST find files in subdirectories
- Test: `glob_search_recursive` in `src/tools/glob_search.rs`

#### Scenario: No matches

- WHEN a pattern matches no files
- THEN MUST return an appropriate empty result
- Test: `glob_search_no_matches` in `src/tools/glob_search.rs`

#### Scenario: Missing pattern parameter

- WHEN no pattern parameter is provided
- THEN MUST return an error
- Test: `glob_search_missing_param` in `src/tools/glob_search.rs`

#### Scenario: Results are sorted

- WHEN multiple files match
- THEN results MUST be returned in sorted order
- Test: `glob_search_results_sorted` in `src/tools/glob_search.rs`

#### Scenario: Excludes directories

- WHEN a pattern matches directories
- THEN MUST exclude them from results (files only)
- Test: `glob_search_excludes_directories` in `src/tools/glob_search.rs`

#### Scenario: Invalid pattern

- WHEN an invalid glob pattern is provided
- THEN MUST return an error
- Test: `glob_search_invalid_pattern` in `src/tools/glob_search.rs`

---

### REQ-GLOB-003: Glob Search Security

The glob_search tool MUST enforce path security restrictions.

#### Scenario: Reject absolute path

- WHEN an absolute path is provided
- THEN MUST reject the request
- Test: `glob_search_rejects_absolute_path` in `src/tools/glob_search.rs`

#### Scenario: Reject path traversal

- WHEN path contains `..` traversal
- THEN MUST reject the request
- Test: `glob_search_rejects_path_traversal` in `src/tools/glob_search.rs`

#### Scenario: Reject dotdot only

- WHEN path is `..` alone
- THEN MUST reject the request
- Test: `glob_search_rejects_dotdot_only` in `src/tools/glob_search.rs`

#### Scenario: Filter symlink escape

- WHEN results include symlinks pointing outside workspace
- THEN MUST filter them out
- Test: `glob_search_filters_symlink_escape` in `src/tools/glob_search.rs`

#### Scenario: Readonly mode allowed

- WHEN security policy is in readonly mode
- THEN MUST allow glob search (read-only operation)
- Test: `glob_search_readonly_mode` in `src/tools/glob_search.rs`

#### Scenario: Block when rate limited

- WHEN rate limit budget is exhausted
- THEN MUST reject the request
- Test: `glob_search_rate_limited` in `src/tools/glob_search.rs`

---

### REQ-SEARCH-001: Content Search Tool Identity and Schema

The content_search tool MUST have a stable name and schema.

#### Scenario: Content search name and schema

- WHEN `name()` and `parameters_schema()` are called
- THEN MUST return "content_search" and a schema with pattern property
- Test: `content_search_name_and_schema` in `src/tools/content_search.rs`

---

### REQ-SEARCH-002: Content Search Operations

The content_search tool MUST find content matching patterns in files.

#### Scenario: Basic match

- WHEN a pattern matches content in files
- THEN MUST return matching lines with file paths
- Test: `content_search_basic_match` in `src/tools/content_search.rs`

#### Scenario: Files with matches mode

- WHEN output mode is "files_with_matches"
- THEN MUST return only file paths, not matching lines
- Test: `content_search_files_with_matches_mode` in `src/tools/content_search.rs`

#### Scenario: Count mode

- WHEN output mode is "count"
- THEN MUST return match counts per file
- Test: `content_search_count_mode` in `src/tools/content_search.rs`

#### Scenario: Case insensitive search

- WHEN case_insensitive flag is set
- THEN MUST match regardless of case
- Test: `content_search_case_insensitive` in `src/tools/content_search.rs`

#### Scenario: Include filter

- WHEN include parameter restricts file types
- THEN MUST only search matching files
- Test: `content_search_include_filter` in `src/tools/content_search.rs`

#### Scenario: Context lines

- WHEN context lines are requested
- THEN MUST include surrounding lines in output
- Test: `content_search_context_lines` in `src/tools/content_search.rs`

#### Scenario: No matches

- WHEN pattern matches nothing
- THEN MUST return appropriate empty result
- Test: `content_search_no_matches` in `src/tools/content_search.rs`

#### Scenario: Subdirectory search

- WHEN a subdirectory path is specified
- THEN MUST search only within that subdirectory
- Test: `content_search_subdirectory` in `src/tools/content_search.rs`

#### Scenario: Multiline without rg

- WHEN multiline mode is used and rg is not available
- THEN MUST fall back to grep-based multiline search
- Test: `content_search_multiline_without_rg` in `src/tools/content_search.rs`

---

### REQ-SEARCH-003: Content Search Input Validation

The content_search tool MUST validate inputs.

#### Scenario: Empty pattern rejected

- WHEN an empty pattern is provided
- THEN MUST reject the request
- Test: `content_search_empty_pattern_rejected` in `src/tools/content_search.rs`

#### Scenario: Missing pattern

- WHEN no pattern parameter is provided
- THEN MUST return an error
- Test: `content_search_missing_pattern` in `src/tools/content_search.rs`

#### Scenario: Invalid output mode rejected

- WHEN an invalid output mode is provided
- THEN MUST reject the request
- Test: `content_search_invalid_output_mode_rejected` in `src/tools/content_search.rs`

---

### REQ-SEARCH-004: Content Search Security

The content_search tool MUST enforce path security restrictions.

#### Scenario: Reject absolute path

- WHEN an absolute path is provided
- THEN MUST reject the request
- Test: `content_search_rejects_absolute_path` in `src/tools/content_search.rs`

#### Scenario: Reject path traversal

- WHEN path contains `..` traversal
- THEN MUST reject the request
- Test: `content_search_rejects_path_traversal` in `src/tools/content_search.rs`

#### Scenario: Block when rate limited

- WHEN rate limit budget is exhausted
- THEN MUST reject the request
- Test: `content_search_rate_limited` in `src/tools/content_search.rs`

#### Scenario: Symlink escape blocked

- WHEN search results include content from symlinked files outside workspace
- THEN MUST block those results
- Test: `content_search_symlink_escape_blocked` in `src/tools/content_search.rs`

---

### REQ-SEARCH-005: Content Search Internal Utilities

Internal utility functions MUST behave correctly.

#### Scenario: Relativize path strips prefix

- WHEN a path line has the workspace prefix
- THEN `relativize_path` MUST strip it
- Test: `relativize_path_strips_prefix` in `src/tools/content_search.rs`

#### Scenario: Relativize path without prefix

- WHEN a path line does not have the workspace prefix
- THEN `relativize_path` MUST return it unchanged
- Test: `relativize_path_no_prefix` in `src/tools/content_search.rs`

#### Scenario: Format line output counts match lines only

- WHEN formatting content output lines
- THEN `format_line_output` MUST count only actual match lines, not context separators
- Test: `format_line_output_content_counts_match_lines_only` in `src/tools/content_search.rs`

#### Scenario: Parse count line supports colons in path

- WHEN a count output line has colons in the file path
- THEN `parse_count_line` MUST correctly separate path from count
- Test: `parse_count_line_supports_colons_in_path` in `src/tools/content_search.rs`

#### Scenario: Truncate UTF-8 keeps char boundary

- WHEN truncating a UTF-8 string to a byte limit
- THEN `truncate_utf8` MUST not split a multi-byte character
- Test: `truncate_utf8_keeps_char_boundary` in `src/tools/content_search.rs`

---

### REQ-PROCESS-001: Process Tool Identity

The process tool MUST have a stable name, description, and schema.

#### Scenario: Process tool name

- WHEN `name()` is called
- THEN MUST return "process"
- Test: `process_tool_name` in `src/tools/process.rs`

#### Scenario: Process tool description not empty

- WHEN `description()` is called
- THEN MUST return a non-empty description
- Test: `process_tool_description_not_empty` in `src/tools/process.rs`

#### Scenario: Process tool schema has action

- WHEN `parameters_schema()` is called
- THEN MUST include an "action" property with enum values
- Test: `process_tool_schema_has_action` in `src/tools/process.rs`

#### Scenario: Constants are correct

- WHEN process tool constants are checked
- THEN MUST have expected values for limits
- Test: `constants_are_correct` in `src/tools/process.rs`

---

### REQ-PROCESS-002: Process Lifecycle Management

The process tool MUST support spawn, list, output, and kill actions.

#### Scenario: Spawn starts background process

- WHEN spawn action is called with a valid command
- THEN MUST start a background process and return its ID
- Test: `spawn_starts_background_process` in `src/tools/process.rs`

#### Scenario: List shows spawned process

- WHEN list action is called after spawning a process
- THEN MUST show the spawned process in the list
- Test: `list_shows_spawned_process` in `src/tools/process.rs`

#### Scenario: Output returns stdout

- WHEN output action is called for a running/completed process
- THEN MUST return the process stdout
- Test: `output_returns_stdout` in `src/tools/process.rs`

#### Scenario: Kill terminates process

- WHEN kill action is called for a running process
- THEN MUST terminate the process
- Test: `kill_terminates_process` in `src/tools/process.rs`

#### Scenario: Unknown action returns error

- WHEN an unrecognized action is provided
- THEN MUST return an error
- Test: `unknown_action_returns_error` in `src/tools/process.rs`

---

### REQ-PROCESS-003: Process Security and Policy

The process tool MUST enforce security restrictions.

#### Scenario: Spawn blocks disallowed command

- WHEN a command is not in the allowed list
- THEN MUST reject spawn
- Test: `spawn_blocks_disallowed_command` in `src/tools/process.rs`

#### Scenario: Spawn blocks forbidden path

- WHEN a command contains a forbidden path
- THEN MUST reject spawn
- Test: `spawn_blocks_forbidden_path` in `src/tools/process.rs`

#### Scenario: Kill blocks in readonly mode

- WHEN security policy is in readonly mode
- THEN MUST block kill action
- Test: `kill_blocks_readonly` in `src/tools/process.rs`

#### Scenario: Spawn blocks when rate limited

- WHEN rate limit budget is exhausted
- THEN MUST reject spawn
- Test: `spawn_blocks_rate_limited` in `src/tools/process.rs`

#### Scenario: Spawn rejects when runtime unsupported

- WHEN runtime adapter does not support long-running processes
- THEN MUST reject spawn with appropriate error
- Test: `spawn_rejects_when_runtime_unsupported` in `src/tools/process.rs`

---

### REQ-PROCESS-004: Process Output and Error Handling

The process tool MUST handle edge cases in output retrieval.

#### Scenario: Output missing id returns error

- WHEN output action is called without an id parameter
- THEN MUST return an error
- Test: `output_missing_id_returns_error` in `src/tools/process.rs`

#### Scenario: Output nonexistent id returns error

- WHEN output action is called with an id that does not exist
- THEN MUST return an error
- Test: `output_nonexistent_id_returns_error` in `src/tools/process.rs`

---

### REQ-PROCESS-005: Process Internal Utilities

Internal utilities MUST handle buffering and output tracking correctly.

#### Scenario: Append bounded truncates old data

- WHEN data appended exceeds buffer limit
- THEN `append_bounded` MUST discard oldest data
- Test: `append_bounded_truncates_old_data` in `src/tools/process.rs`

#### Scenario: Slice unseen output tracks new tail after rollover

- WHEN output buffer rolls over
- THEN `slice_unseen_output` MUST correctly track the new unseen portion
- Test: `slice_unseen_output_tracks_new_tail_after_rollover` in `src/tools/process.rs`

---

### REQ-PROCESS-006: Process Syscall Anomaly Detection

The process tool MUST integrate with the syscall anomaly detector.

#### Scenario: Process output runs syscall detector incrementally

- WHEN process output is retrieved
- THEN MUST run syscall anomaly detection on the new output incrementally
- Test: `process_output_runs_syscall_detector_incrementally` in `src/tools/process.rs`

---

### REQ-BG-001: Background Job Store

`BgJobStore` MUST manage concurrent background jobs with lifecycle tracking.

#### Scenario: Job ID format

- WHEN a job ID is generated
- THEN MUST be a valid hex string
- Test: `job_id_format` in `src/tools/bg_run.rs`

#### Scenario: Insert and get job

- WHEN a job is inserted into the store
- THEN MUST be retrievable by ID
- Test: `job_store_insert_and_get` in `src/tools/bg_run.rs`

#### Scenario: Update job status

- WHEN a job status is updated
- THEN MUST reflect the new status on retrieval
- Test: `job_store_update` in `src/tools/bg_run.rs`

#### Scenario: Drain completed jobs

- WHEN `drain_completed` is called
- THEN MUST return all completed jobs and remove them from the store
- Test: `job_store_drain_completed` in `src/tools/bg_run.rs`

#### Scenario: Sender filtering on drain

- WHEN `drain_completed` is called with a sender filter
- THEN MUST only return jobs matching that sender
- Test: `job_store_drain_filters_by_sender` in `src/tools/bg_run.rs`

---

### REQ-BG-002: XML Escaping

`escape_xml` MUST escape all XML special characters to prevent injection.

#### Scenario: All special characters escaped

- WHEN input contains `&`, `<`, `>`, `"`, `'`
- THEN MUST replace with XML entities
- Test: `escape_xml_special_chars` in `src/tools/bg_run.rs`

---

### REQ-BG-003: Result Formatting

`format_bg_result` MUST produce valid formatted output with escaped content.

#### Scenario: Result with XML-like content

- WHEN tool output contains XML-like strings
- THEN formatted result MUST have escaped content
- Test: `format_bg_result` in `src/tools/bg_run.rs`

---

## Coverage Notes

- Total test functions across 10 core tool files: **185**
  - `src/tools/traits.rs`: 5 tests
  - `src/tools/mod.rs`: 18 tests
  - `src/tools/shell.rs`: 31 tests
  - `src/tools/file_read.rs`: 29 tests
  - `src/tools/file_write.rs`: 18 tests
  - `src/tools/file_edit.rs`: 21 tests
  - `src/tools/glob_search.rs`: 15 tests
  - `src/tools/content_search.rs`: 22 tests
  - `src/tools/process.rs`: 19 tests
  - `src/tools/bg_run.rs`: 7 tests
- Every test function is mapped to at least one scenario
- 1 test (`e2e_live_file_read_pdf` in `src/tools/file_read.rs`) is `#[ignore]` (requires live credentials) and is not included in scenario mapping

## Mock Strategy

- Core tools: Use `tempfile::TempDir` for filesystem isolation
- Shell: Tested with safe commands (`echo`, `true`, `false`)
- Process: Tested with safe long-running commands (`sleep`, `yes`)
- BgJobStore: Direct construction and async operations in `#[tokio::test]`
- Content search: Uses `new_with_backend` to test both rg and grep fallback paths
- File read E2E: Uses `RecordingProvider` mock to verify agent loop integration
