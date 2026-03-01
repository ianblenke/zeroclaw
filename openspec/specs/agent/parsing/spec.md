# Agent Loop Parsing Specification

## Purpose

Define requirements for parsing LLM responses into structured tool calls across multiple provider-specific formats (OpenAI JSON, XML tags, MiniMax invoke, GLM-style, Perl-style, FunctionCall-style, markdown code blocks).

## Scope

- Files: `src/agent/loop_/parsing.rs`
- Risk tier: HIGH (tool execution depends on correct parsing; incorrect parsing could enable prompt injection)

## Requirements

### REQ-PARSE-001: Argument Parsing

`parse_arguments_value` MUST parse tool call arguments from multiple representations.

#### Scenario: JSON string argument is deserialized

- WHEN the raw argument is a JSON string containing `{"command":"ls"}`
- THEN `parse_arguments_value` MUST deserialize it into a JSON object
- Test: `parse_arguments_value_json_string` in `src/agent/loop_/parsing.rs`

#### Scenario: JSON object argument is passed through

- WHEN the raw argument is already a JSON object
- THEN `parse_arguments_value` MUST return it unchanged
- Test: `parse_arguments_value_object_passthrough` in `src/agent/loop_/parsing.rs`

#### Scenario: None argument returns empty object

- WHEN the raw argument is `None`
- THEN `parse_arguments_value` MUST return an empty JSON object
- Test: `parse_arguments_value_none_returns_empty` in `src/agent/loop_/parsing.rs`

#### Scenario: Invalid JSON string returns empty object

- WHEN the raw argument is a string that is not valid JSON
- THEN `parse_arguments_value` MUST return an empty JSON object
- Test: `parse_arguments_value_invalid_json_returns_empty` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-002: Shell Command Normalization

`normalize_shell_command_from_raw` MUST extract shell commands from various raw formats.

#### Scenario: Plain command passes through

- WHEN the raw string is a plain shell command like `ls -la`
- THEN `normalize_shell_command_from_raw` MUST return it as-is
- Test: `shell_normalize_plain_command` in `src/agent/loop_/parsing.rs`

#### Scenario: Quoted strings are unwrapped

- WHEN the raw string is wrapped in single or double quotes
- THEN `normalize_shell_command_from_raw` MUST strip the quotes
- Test: `shell_normalize_strips_quotes` in `src/agent/loop_/parsing.rs`

#### Scenario: URLs are converted to curl

- WHEN the raw string is an HTTP/HTTPS URL
- THEN `normalize_shell_command_from_raw` MUST convert it to a `curl -s` command
- Test: `shell_normalize_url_to_curl` in `src/agent/loop_/parsing.rs`

#### Scenario: Empty strings return None

- WHEN the raw string is empty or whitespace-only
- THEN `normalize_shell_command_from_raw` MUST return `None`
- Test: `shell_normalize_empty_returns_none` in `src/agent/loop_/parsing.rs`

#### Scenario: JSON-like strings return None

- WHEN the raw string looks like a JSON object `{...}` or array `[...]`
- THEN `normalize_shell_command_from_raw` MUST return `None`
- Test: `shell_normalize_json_returns_none` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-003: Shell Arguments Normalization

`normalize_shell_arguments` MUST resolve a command from multiple argument representations including alias keys, URL fields, and raw string hints.

#### Scenario: Command key is preserved

- WHEN arguments contain a non-empty `command` key
- THEN `normalize_shell_arguments` MUST return arguments unchanged
- Test: `shell_args_command_key_preserved` in `src/agent/loop_/parsing.rs`

#### Scenario: Alias keys are mapped to command

- WHEN arguments contain alias keys like `cmd`, `script`, `bash`, `sh`
- THEN `normalize_shell_arguments` MUST move the value to `command`
- Test: `shell_args_alias_mapped_to_command` in `src/agent/loop_/parsing.rs`

#### Scenario: URL keys are converted to curl

- WHEN arguments contain a `url` or `http_url` key with a URL value
- THEN `normalize_shell_arguments` MUST convert to a curl command
- Test: `shell_args_url_to_curl` in `src/agent/loop_/parsing.rs`

#### Scenario: String argument becomes command

- WHEN arguments are a plain JSON string
- THEN `normalize_shell_arguments` MUST wrap it as `{"command": value}`
- Test: `shell_args_string_to_command` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-004: Tool Name Aliasing

`map_tool_name_alias` MUST map all known LLM tool name variants to canonical ZeroClaw names.

#### Scenario: Shell aliases

- WHEN the tool name is `bash`, `sh`, `exec`, `command`, `cmd`, `browser_open`, `browser`, or `web_search`
- THEN `map_tool_name_alias` MUST return `shell`
- Test: `alias_shell_variants` in `src/agent/loop_/parsing.rs`

#### Scenario: File tool aliases

- WHEN the tool name is `fileread`, `readfile`, `read_file`, or `file`
- THEN `map_tool_name_alias` MUST return `file_read`
- Test: `alias_file_read_variants` in `src/agent/loop_/parsing.rs`

#### Scenario: Memory tool aliases

- WHEN the tool name is `memoryrecall`, `recall`, or `memrecall`
- THEN `map_tool_name_alias` MUST return `memory_recall`
- Test: `alias_memory_variants` in `src/agent/loop_/parsing.rs`

#### Scenario: Unknown names pass through

- WHEN the tool name is not a known alias
- THEN `map_tool_name_alias` MUST return it unchanged
- Test: `alias_unknown_passthrough` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-005: XML Pair Extraction

`extract_xml_pairs` MUST extract all `<tag>…</tag>` pairs from input text.

#### Scenario: Single pair extraction

- WHEN input contains `<shell>pwd</shell>`
- THEN `extract_xml_pairs` MUST return `[("shell", "pwd")]`
- Test: `xml_pairs_single` in `src/agent/loop_/parsing.rs`

#### Scenario: Multiple pairs extraction

- WHEN input contains multiple XML tag pairs
- THEN `extract_xml_pairs` MUST return all pairs in order
- Test: `xml_pairs_multiple` in `src/agent/loop_/parsing.rs`

#### Scenario: Unclosed tags are skipped

- WHEN input contains `<shell>pwd` without a closing tag
- THEN `extract_xml_pairs` MUST skip the unclosed tag
- Test: `xml_pairs_unclosed_skipped` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-006: XML Meta Tag Detection

`is_xml_meta_tag` MUST identify structural/meta XML tags that are not tool names.

#### Scenario: Known meta tags

- WHEN the tag is `tool_call`, `toolcall`, `tool-call`, `invoke`, `thinking`, `thought`, `analysis`, `reasoning`, or `reflection`
- THEN `is_xml_meta_tag` MUST return `true`
- Test: `meta_tags_detected` in `src/agent/loop_/parsing.rs`

#### Scenario: Tool names are not meta

- WHEN the tag is `shell`, `file_read`, or any actual tool name
- THEN `is_xml_meta_tag` MUST return `false`
- Test: `tool_names_not_meta` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-007: XML Tool Call Parsing

`parse_xml_tool_calls` MUST parse tool calls from XML content with nested argument tags or JSON payloads.

#### Scenario: Nested argument tags

- WHEN content is `<memory_recall><query>test</query></memory_recall>`
- THEN `parse_xml_tool_calls` MUST return a tool call with `name=memory_recall` and `arguments.query=test`
- Test: `xml_tool_calls_nested_args` in `src/agent/loop_/parsing.rs`

#### Scenario: JSON payload in tag body

- WHEN content is `<shell>{"command":"ls"}</shell>`
- THEN `parse_xml_tool_calls` MUST return a tool call with parsed JSON arguments
- Test: `xml_tool_calls_json_body` in `src/agent/loop_/parsing.rs`

#### Scenario: Meta tags are skipped

- WHEN content wraps a tool call in `<tool_call>` meta tags
- THEN `parse_xml_tool_calls` MUST skip the meta wrapper
- Test: `xml_tool_calls_meta_skipped` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-008: JSON Value Extraction

`extract_json_values` MUST extract JSON objects/arrays from mixed text content.

#### Scenario: Standalone JSON

- WHEN input is a valid JSON object
- THEN `extract_json_values` MUST return it as a single-element vector
- Test: `json_extract_standalone` in `src/agent/loop_/parsing.rs`

#### Scenario: JSON embedded in text

- WHEN input contains JSON objects mixed with non-JSON text
- THEN `extract_json_values` MUST extract only the JSON objects
- Test: `json_extract_embedded` in `src/agent/loop_/parsing.rs`

#### Scenario: Empty input

- WHEN input is empty or whitespace
- THEN `extract_json_values` MUST return an empty vector
- Test: `json_extract_empty` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-009: JSON End Finder

`find_json_end` MUST find the end position of a brace-balanced JSON object.

#### Scenario: Simple object

- WHEN input starts with `{"key":"value"}`
- THEN `find_json_end` MUST return the position after the closing `}`
- Test: `json_end_simple` in `src/agent/loop_/parsing.rs`

#### Scenario: Nested objects

- WHEN input contains nested `{` and `}` with proper balancing
- THEN `find_json_end` MUST return the correct end position
- Test: `json_end_nested` in `src/agent/loop_/parsing.rs`

#### Scenario: Escaped braces in strings

- WHEN input contains `{` or `}` inside quoted strings
- THEN `find_json_end` MUST ignore them and track only unquoted braces
- Test: `json_end_escaped_braces` in `src/agent/loop_/parsing.rs`

#### Scenario: Non-object input

- WHEN input does not start with `{`
- THEN `find_json_end` MUST return `None`
- Test: `json_end_non_object` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-010: Build Curl Command

`build_curl_command` MUST convert HTTP/HTTPS URLs to curl commands.

#### Scenario: Valid URL

- WHEN input is a valid HTTP or HTTPS URL
- THEN `build_curl_command` MUST return `Some("curl -s '<url>'")`
- Test: `curl_valid_url` in `src/agent/loop_/parsing.rs`

#### Scenario: Non-URL input

- WHEN input does not start with `http://` or `https://`
- THEN `build_curl_command` MUST return `None`
- Test: `curl_non_url` in `src/agent/loop_/parsing.rs`

#### Scenario: URL with whitespace

- WHEN input URL contains whitespace
- THEN `build_curl_command` MUST return `None`
- Test: `curl_url_with_whitespace` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-011: Default Parameter for Tool

`default_param_for_tool` MUST map tool names to their canonical default parameter names.

#### Scenario: Shell tools default to command

- WHEN tool name is `shell`, `bash`, `sh`, `exec`, `command`, or `cmd`
- THEN `default_param_for_tool` MUST return `"command"`
- Test: `default_param_shell` in `src/agent/loop_/parsing.rs`

#### Scenario: File tools default to path

- WHEN tool name is any file tool variant
- THEN `default_param_for_tool` MUST return `"path"`
- Test: `default_param_file` in `src/agent/loop_/parsing.rs`

#### Scenario: Memory recall defaults to query

- WHEN tool name is any memory recall variant
- THEN `default_param_for_tool` MUST return `"query"`
- Test: `default_param_memory` in `src/agent/loop_/parsing.rs`

#### Scenario: Unknown tools default to input

- WHEN tool name is not recognized
- THEN `default_param_for_tool` MUST return `"input"`
- Test: `default_param_unknown` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-012: Tool Call Signature

`canonicalize_json_for_tool_signature` and `tool_call_signature` MUST produce deterministic dedup keys.

#### Scenario: Keys are sorted

- WHEN a JSON object has keys in arbitrary order
- THEN `canonicalize_json_for_tool_signature` MUST sort keys alphabetically
- Test: `signature_sorts_keys` in `src/agent/loop_/parsing.rs`

#### Scenario: Name is lowercased

- WHEN `tool_call_signature` is called with a mixed-case name
- THEN the returned name component MUST be lowercase
- Test: `signature_lowercase_name` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-013: GLM Shortened Body Parsing

`parse_glm_shortened_body` MUST parse GLM-style shortened tool call bodies.

#### Scenario: Single value format

- WHEN body is `shell>ls -la`
- THEN MUST parse as tool=shell, command=ls -la
- Test: `glm_shortened_single_value` in `src/agent/loop_/parsing.rs`

#### Scenario: YAML-like multi-line

- WHEN body has multiple `key: value` lines
- THEN MUST parse each line as a parameter
- Test: `glm_shortened_yaml_multiline` in `src/agent/loop_/parsing.rs`

#### Scenario: Attribute style

- WHEN body is `shell command="ls" description="list files"`
- THEN MUST parse XML-like attributes as parameters
- Test: `glm_shortened_attribute_style` in `src/agent/loop_/parsing.rs`

#### Scenario: URL value for shell

- WHEN body is `shell>https://example.com`
- THEN MUST convert to curl command
- Test: `glm_shortened_url_to_curl` in `src/agent/loop_/parsing.rs`

#### Scenario: Empty body

- WHEN body is empty or whitespace
- THEN MUST return `None`
- Test: `glm_shortened_empty` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-014: Main Parse Entry Point

`parse_tool_calls` MUST parse tool calls following a priority chain: OpenAI JSON → MiniMax invoke → XML tags → markdown code blocks → direct XML → attribute XML → Perl style → FunctionCall → GLM style.

#### Scenario: OpenAI JSON format

- WHEN response is a JSON object with `tool_calls` array
- THEN `parse_tool_calls` MUST parse as OpenAI format
- Test: `parse_openai_json_format` in `src/agent/loop_/parsing.rs`

#### Scenario: XML tool_call tags

- WHEN response contains `<tool_call>{"name":"shell","arguments":{"command":"ls"}}</tool_call>`
- THEN `parse_tool_calls` MUST parse the XML-wrapped JSON
- Test: `parse_xml_tag_format` in `src/agent/loop_/parsing.rs`

#### Scenario: Markdown code blocks

- WHEN response contains ````tool_call\n{"name":"shell","arguments":{"command":"ls"}}\n````
- THEN `parse_tool_calls` MUST parse the markdown block
- Test: `parse_markdown_tool_call` in `src/agent/loop_/parsing.rs`

#### Scenario: Text before/after tags is preserved

- WHEN response contains text around tool call tags
- THEN `parse_tool_calls` MUST return the text parts joined by newline
- Test: `parse_preserves_surrounding_text` in `src/agent/loop_/parsing.rs`

#### Scenario: Plain text without tool calls

- WHEN response is plain text with no tool call patterns
- THEN `parse_tool_calls` MUST return the text and empty calls
- Test: `parse_plain_text_no_calls` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-015: Tool Call Parse Issue Detection

`detect_tool_call_parse_issue` MUST detect when a response looks like it contains tool calls but none were parsed.

#### Scenario: Unparsed tool call detected

- WHEN response contains `<tool_call>` or `"tool_calls"` but parsed_calls is empty
- THEN `detect_tool_call_parse_issue` MUST return `Some(...)` with a warning message
- Test: `detect_issue_unparsed_tool_call` in `src/agent/loop_/parsing.rs`

#### Scenario: Clean text has no issue

- WHEN response is plain text and parsed_calls is empty
- THEN `detect_tool_call_parse_issue` MUST return `None`
- Test: `detect_issue_clean_text` in `src/agent/loop_/parsing.rs`

#### Scenario: Successfully parsed has no issue

- WHEN parsed_calls is non-empty
- THEN `detect_tool_call_parse_issue` MUST return `None`
- Test: `detect_issue_parsed_ok` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-016: Structured Tool Call Parsing

`parse_structured_tool_calls` MUST convert provider `ToolCall` structs to `ParsedToolCall` with normalization.

#### Scenario: Valid structured tool call

- WHEN a ToolCall has valid name and JSON arguments
- THEN `parse_structured_tool_calls` MUST return a normalized ParsedToolCall
- Test: `structured_valid_call` in `src/agent/loop_/parsing.rs`

#### Scenario: Invalid arguments fall back to empty

- WHEN a ToolCall has invalid JSON in arguments
- THEN `parse_structured_tool_calls` MUST fall back to empty object
- Test: `structured_invalid_args` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-017: MiniMax Invoke Parsing

`parse_minimax_invoke_calls` MUST parse MiniMax-style attributed `<invoke>` tags with `<parameter>` children.

#### Scenario: Standard invoke format

- WHEN response contains `<invoke name="shell"><parameter name="command">pwd</parameter></invoke>`
- THEN MUST return a tool call with name=shell and arguments.command=pwd
- Test: `minimax_standard_invoke` in `src/agent/loop_/parsing.rs`

#### Scenario: No invoke tags

- WHEN response has no `<invoke>` tags
- THEN MUST return `None`
- Test: `minimax_no_invoke` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-018: Perl-Style Tool Call Parsing

`parse_perl_style_tool_calls` MUST parse hash-ref style TOOL_CALL blocks.

#### Scenario: Standard Perl format

- WHEN response contains `TOOL_CALL\n{tool => "shell", args => {\n  --command "ls"\n}}\n/TOOL_CALL`
- THEN MUST return a tool call with name=shell and arguments.command=ls
- Test: `perl_style_standard` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-019: FunctionCall-Style Parsing

`parse_function_call_tool_calls` MUST parse `<FunctionCall>` blocks with `<code>` argument syntax.

#### Scenario: Standard FunctionCall format

- WHEN response contains `<FunctionCall>\nfile_read\n<code>path>/tmp/file.txt</code>\n</FunctionCall>`
- THEN MUST return a tool call with name=file_read and arguments.path=/tmp/file.txt
- Test: `function_call_standard` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-020: XML Attribute Tool Call Parsing

`parse_xml_attribute_tool_calls` MUST parse `<invoke name="...">` with `<parameter name="...">` children.

#### Scenario: Standard attribute format

- WHEN response contains `<invoke name="shell"><parameter name="command">ls</parameter></invoke>`
- THEN MUST return a tool call with name=shell and arguments.command=ls
- Test: `xml_attribute_standard` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-021: Tool Call ID Parsing

`parse_tool_call_id` MUST extract tool call IDs from multiple JSON locations.

#### Scenario: ID from function object

- WHEN the function sub-object contains `"id"` field
- THEN MUST return that ID
- Test: `tool_call_id_from_function` in `src/agent/loop_/parsing.rs`

#### Scenario: ID from root object

- WHEN the root object contains `"id"`, `"tool_call_id"`, or `"call_id"`
- THEN MUST return the first found ID
- Test: `tool_call_id_from_root` in `src/agent/loop_/parsing.rs`

#### Scenario: No ID found

- WHEN neither root nor function have an ID
- THEN MUST return `None`
- Test: `tool_call_id_none` in `src/agent/loop_/parsing.rs`

### REQ-PARSE-022: Close Tag Matching

`matching_tool_call_close_tag` MUST return the correct close tag for each open tag variant.

#### Scenario: All known pairs

- WHEN open tag is any of the 6 known open tags
- THEN MUST return the corresponding close tag
- Test: `close_tag_matching` in `src/agent/loop_/parsing.rs`

#### Scenario: Unknown tag

- WHEN open tag is not recognized
- THEN MUST return `None`
- Test: `close_tag_unknown` in `src/agent/loop_/parsing.rs`

## Mock Strategy

All functions are pure/deterministic — no mocks needed. Tests use literal string inputs and assert on parsed output structures. The `ToolCall` struct from `crate::providers` is constructed directly for `parse_structured_tool_calls` tests.
