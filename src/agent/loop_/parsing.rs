use crate::providers::ToolCall;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub(super) struct ParsedToolCall {
    pub(super) name: String,
    pub(super) arguments: serde_json::Value,
    pub(super) tool_call_id: Option<String>,
}

pub(super) fn parse_arguments_value(raw: Option<&serde_json::Value>) -> serde_json::Value {
    match raw {
        Some(serde_json::Value::String(s)) => serde_json::from_str::<serde_json::Value>(s)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
        Some(value) => value.clone(),
        None => serde_json::Value::Object(serde_json::Map::new()),
    }
}

pub(super) fn raw_string_argument_hint(raw: Option<&serde_json::Value>) -> Option<&str> {
    raw.and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

pub(super) fn normalize_shell_command_from_raw(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let unwrapped = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| {
            trimmed
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
        })
        .unwrap_or(trimmed)
        .trim();

    if unwrapped.is_empty() {
        return None;
    }

    if (unwrapped.starts_with('{') && unwrapped.ends_with('}'))
        || (unwrapped.starts_with('[') && unwrapped.ends_with(']'))
    {
        return None;
    }

    if unwrapped.starts_with("http://") || unwrapped.starts_with("https://") {
        return build_curl_command(unwrapped).or_else(|| Some(unwrapped.to_string()));
    }

    Some(unwrapped.to_string())
}

pub(super) fn normalize_shell_arguments(
    arguments: serde_json::Value,
    raw_string_hint: Option<&str>,
) -> serde_json::Value {
    match arguments {
        serde_json::Value::Object(mut map) => {
            if map
                .get("command")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .is_some_and(|cmd| !cmd.is_empty())
            {
                return serde_json::Value::Object(map);
            }

            for alias in [
                "cmd",
                "script",
                "shell_command",
                "command_line",
                "bash",
                "sh",
                "input",
            ] {
                if let Some(value) = map.get(alias).and_then(|v| v.as_str()) {
                    if let Some(command) = normalize_shell_command_from_raw(value) {
                        map.insert("command".to_string(), serde_json::Value::String(command));
                        return serde_json::Value::Object(map);
                    }
                }
            }

            if let Some(url) = map
                .get("url")
                .or_else(|| map.get("http_url"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|url| !url.is_empty())
            {
                if let Some(command) = normalize_shell_command_from_raw(url) {
                    map.insert("command".to_string(), serde_json::Value::String(command));
                    return serde_json::Value::Object(map);
                }
            }

            if let Some(raw) = raw_string_hint.and_then(normalize_shell_command_from_raw) {
                map.insert("command".to_string(), serde_json::Value::String(raw));
            }

            serde_json::Value::Object(map)
        }
        serde_json::Value::String(raw) => normalize_shell_command_from_raw(&raw)
            .map(|command| serde_json::json!({ "command": command }))
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
        _ => raw_string_hint
            .and_then(normalize_shell_command_from_raw)
            .map(|command| serde_json::json!({ "command": command }))
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
    }
}

pub(super) fn normalize_tool_arguments(
    tool_name: &str,
    arguments: serde_json::Value,
    raw_string_hint: Option<&str>,
) -> serde_json::Value {
    match map_tool_name_alias(tool_name) {
        "shell" => normalize_shell_arguments(arguments, raw_string_hint),
        _ => arguments,
    }
}

pub(super) fn parse_tool_call_id(
    root: &serde_json::Value,
    function: Option<&serde_json::Value>,
) -> Option<String> {
    function
        .and_then(|func| func.get("id"))
        .or_else(|| root.get("id"))
        .or_else(|| root.get("tool_call_id"))
        .or_else(|| root.get("call_id"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(ToString::to_string)
}

pub(super) fn canonicalize_json_for_tool_signature(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort_unstable();
            let mut ordered = serde_json::Map::new();
            for key in keys {
                if let Some(child) = map.get(&key) {
                    ordered.insert(key, canonicalize_json_for_tool_signature(child));
                }
            }
            serde_json::Value::Object(ordered)
        }
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .iter()
                .map(canonicalize_json_for_tool_signature)
                .collect(),
        ),
        _ => value.clone(),
    }
}

pub(super) fn tool_call_signature(name: &str, arguments: &serde_json::Value) -> (String, String) {
    let canonical_args = canonicalize_json_for_tool_signature(arguments);
    let args_json = serde_json::to_string(&canonical_args).unwrap_or_else(|_| "{}".to_string());
    (name.trim().to_ascii_lowercase(), args_json)
}

pub(super) fn parse_tool_call_value(value: &serde_json::Value) -> Option<ParsedToolCall> {
    if let Some(function) = value.get("function") {
        let tool_call_id = parse_tool_call_id(value, Some(function));
        let name = function
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if !name.is_empty() {
            let raw_arguments = function
                .get("arguments")
                .or_else(|| function.get("parameters"));
            let arguments = normalize_tool_arguments(
                &name,
                parse_arguments_value(raw_arguments),
                raw_string_argument_hint(raw_arguments),
            );
            return Some(ParsedToolCall {
                name,
                arguments,
                tool_call_id,
            });
        }
    }

    let tool_call_id = parse_tool_call_id(value, None);
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if name.is_empty() {
        return None;
    }

    let raw_arguments = value.get("arguments").or_else(|| value.get("parameters"));
    let arguments = normalize_tool_arguments(
        &name,
        parse_arguments_value(raw_arguments),
        raw_string_argument_hint(raw_arguments),
    );
    Some(ParsedToolCall {
        name,
        arguments,
        tool_call_id,
    })
}

pub(super) fn parse_tool_calls_from_json_value(value: &serde_json::Value) -> Vec<ParsedToolCall> {
    let mut calls = Vec::new();

    if let Some(tool_calls) = value.get("tool_calls").and_then(|v| v.as_array()) {
        for call in tool_calls {
            if let Some(parsed) = parse_tool_call_value(call) {
                calls.push(parsed);
            }
        }

        if !calls.is_empty() {
            return calls;
        }
    }

    if let Some(message) = value.get("message") {
        let nested = parse_tool_calls_from_json_value(message);
        if !nested.is_empty() {
            return nested;
        }
    }

    if let Some(choices) = value.get("choices").and_then(|v| v.as_array()) {
        for choice in choices {
            if let Some(message) = choice.get("message") {
                let nested = parse_tool_calls_from_json_value(message);
                if !nested.is_empty() {
                    calls.extend(nested);
                }
            }
        }
        if !calls.is_empty() {
            return calls;
        }
    }

    if let Some(array) = value.as_array() {
        for item in array {
            if let Some(parsed) = parse_tool_call_value(item) {
                calls.push(parsed);
            }
        }
        return calls;
    }

    if let Some(parsed) = parse_tool_call_value(value) {
        calls.push(parsed);
    }

    calls
}

fn extract_tool_text_from_json_value(value: &serde_json::Value) -> Option<String> {
    if let Some(content) = value
        .get("content")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Some(content.to_string());
    }

    if let Some(message) = value.get("message") {
        if let Some(content) = extract_tool_text_from_json_value(message) {
            return Some(content);
        }
    }

    if let Some(choices) = value.get("choices").and_then(|v| v.as_array()) {
        for choice in choices {
            if let Some(content) = extract_tool_text_from_json_value(choice) {
                return Some(content);
            }
        }
    }

    None
}

pub(super) fn is_xml_meta_tag(tag: &str) -> bool {
    let normalized = tag.to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "tool_call"
            | "toolcall"
            | "tool-call"
            | "invoke"
            | "thinking"
            | "thought"
            | "analysis"
            | "reasoning"
            | "reflection"
    )
}

/// Match opening XML tags: `<tag_name>`.  Does NOT use backreferences.
static XML_OPEN_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<([a-zA-Z_][a-zA-Z0-9_-]*)>").unwrap());

/// MiniMax XML invoke format:
/// `<invoke name="shell"><parameter name="command">pwd</parameter></invoke>`
static MINIMAX_INVOKE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<invoke\b[^>]*\bname\s*=\s*(?:"([^"]+)"|'([^']+)')[^>]*>(.*?)</invoke>"#)
        .unwrap()
});

static MINIMAX_PARAMETER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<parameter\b[^>]*\bname\s*=\s*(?:"([^"]+)"|'([^']+)')[^>]*>(.*?)</parameter>"#,
    )
    .unwrap()
});

/// Extracts all `<tag>…</tag>` pairs from `input`, returning `(tag_name, inner_content)`.
/// Handles matching closing tags without regex backreferences.
pub(super) fn extract_xml_pairs(input: &str) -> Vec<(&str, &str)> {
    let mut results = Vec::new();
    let mut search_start = 0;
    while let Some(open_cap) = XML_OPEN_TAG_RE.captures(&input[search_start..]) {
        let full_open = open_cap.get(0).unwrap();
        let tag_name = open_cap.get(1).unwrap().as_str();
        let open_end = search_start + full_open.end();

        let closing_tag = format!("</{tag_name}>");
        if let Some(close_pos) = input[open_end..].find(&closing_tag) {
            let inner = &input[open_end..open_end + close_pos];
            results.push((tag_name, inner.trim()));
            search_start = open_end + close_pos + closing_tag.len();
        } else {
            search_start = open_end;
        }
    }
    results
}

/// Parse XML-style tool calls in `<tool_call>` bodies.
/// Supports both nested argument tags and JSON argument payloads:
/// - `<memory_recall><query>...</query></memory_recall>`
/// - `<shell>{"command":"pwd"}</shell>`
pub(super) fn parse_xml_tool_calls(xml_content: &str) -> Option<Vec<ParsedToolCall>> {
    let mut calls = Vec::new();
    let trimmed = xml_content.trim();

    if !trimmed.contains('<') || !trimmed.contains('>') {
        return None;
    }

    for (tool_name_str, inner_content) in extract_xml_pairs(trimmed) {
        if is_xml_meta_tag(tool_name_str) {
            continue;
        }
        let tool_name = map_tool_name_alias(tool_name_str).to_string();

        if inner_content.is_empty() {
            continue;
        }

        let mut args = serde_json::Map::new();

        if let Some(first_json) = extract_json_values(inner_content).into_iter().next() {
            match first_json {
                serde_json::Value::Object(object_args) => {
                    args = object_args;
                }
                other => {
                    args.insert("value".to_string(), other);
                }
            }
        } else {
            for (key_str, value) in extract_xml_pairs(inner_content) {
                let key = key_str.to_string();
                if is_xml_meta_tag(&key) {
                    continue;
                }
                if !value.is_empty() {
                    args.insert(key, serde_json::Value::String(value.to_string()));
                }
            }

            if args.is_empty() {
                args.insert(
                    "content".to_string(),
                    serde_json::Value::String(inner_content.to_string()),
                );
            }
        }

        let arguments = normalize_tool_arguments(
            &tool_name,
            serde_json::Value::Object(args),
            Some(inner_content),
        );

        calls.push(ParsedToolCall {
            name: tool_name,
            arguments,
            tool_call_id: None,
        });
    }

    if calls.is_empty() {
        None
    } else {
        Some(calls)
    }
}

/// Parse MiniMax-style XML tool calls with attributed invoke/parameter tags.
pub(super) fn parse_minimax_invoke_calls(response: &str) -> Option<(String, Vec<ParsedToolCall>)> {
    let mut calls = Vec::new();
    let mut text_parts = Vec::new();
    let mut last_end = 0usize;

    for cap in MINIMAX_INVOKE_RE.captures_iter(response) {
        let Some(full_match) = cap.get(0) else {
            continue;
        };

        let before = response[last_end..full_match.start()].trim();
        if !before.is_empty() {
            text_parts.push(before.to_string());
        }

        let name = cap
            .get(1)
            .or_else(|| cap.get(2))
            .map(|m| m.as_str().trim())
            .filter(|v| !v.is_empty());
        let body = cap.get(3).map(|m| m.as_str()).unwrap_or("").trim();
        last_end = full_match.end();

        let Some(name) = name else {
            continue;
        };

        let mut args = serde_json::Map::new();
        for param_cap in MINIMAX_PARAMETER_RE.captures_iter(body) {
            let key = param_cap
                .get(1)
                .or_else(|| param_cap.get(2))
                .map(|m| m.as_str().trim())
                .unwrap_or_default();
            if key.is_empty() {
                continue;
            }
            let value = param_cap
                .get(3)
                .map(|m| m.as_str().trim())
                .unwrap_or_default();
            if value.is_empty() {
                continue;
            }

            let parsed = extract_json_values(value).into_iter().next();
            args.insert(
                key.to_string(),
                parsed.unwrap_or_else(|| serde_json::Value::String(value.to_string())),
            );
        }

        if args.is_empty() {
            if let Some(first_json) = extract_json_values(body).into_iter().next() {
                match first_json {
                    serde_json::Value::Object(obj) => args = obj,
                    other => {
                        args.insert("value".to_string(), other);
                    }
                }
            } else if !body.is_empty() {
                args.insert(
                    "content".to_string(),
                    serde_json::Value::String(body.to_string()),
                );
            }
        }

        calls.push(ParsedToolCall {
            name: name.to_string(),
            arguments: serde_json::Value::Object(args),
            tool_call_id: None,
        });
    }

    if calls.is_empty() {
        return None;
    }

    let after = response[last_end..].trim();
    if !after.is_empty() {
        text_parts.push(after.to_string());
    }

    let text = text_parts
        .join("\n")
        .replace("<minimax:tool_call>", "")
        .replace("</minimax:tool_call>", "")
        .replace("<minimax:toolcall>", "")
        .replace("</minimax:toolcall>", "")
        .trim()
        .to_string();

    Some((text, calls))
}

const TOOL_CALL_OPEN_TAGS: [&str; 6] = [
    "<tool_call>",
    "<toolcall>",
    "<tool-call>",
    "<invoke>",
    "<minimax:tool_call>",
    "<minimax:toolcall>",
];

const TOOL_CALL_CLOSE_TAGS: [&str; 6] = [
    "</tool_call>",
    "</toolcall>",
    "</tool-call>",
    "</invoke>",
    "</minimax:tool_call>",
    "</minimax:toolcall>",
];

pub(super) fn find_first_tag<'a>(haystack: &str, tags: &'a [&'a str]) -> Option<(usize, &'a str)> {
    tags.iter()
        .filter_map(|tag| haystack.find(tag).map(|idx| (idx, *tag)))
        .min_by_key(|(idx, _)| *idx)
}

pub(super) fn matching_tool_call_close_tag(open_tag: &str) -> Option<&'static str> {
    match open_tag {
        "<tool_call>" => Some("</tool_call>"),
        "<toolcall>" => Some("</toolcall>"),
        "<tool-call>" => Some("</tool-call>"),
        "<invoke>" => Some("</invoke>"),
        "<minimax:tool_call>" => Some("</minimax:tool_call>"),
        "<minimax:toolcall>" => Some("</minimax:toolcall>"),
        _ => None,
    }
}

pub(super) fn extract_first_json_value_with_end(input: &str) -> Option<(serde_json::Value, usize)> {
    let trimmed = input.trim_start();
    let trim_offset = input.len().saturating_sub(trimmed.len());

    for (byte_idx, ch) in trimmed.char_indices() {
        if ch != '{' && ch != '[' {
            continue;
        }

        let slice = &trimmed[byte_idx..];
        let mut stream = serde_json::Deserializer::from_str(slice).into_iter::<serde_json::Value>();
        if let Some(Ok(value)) = stream.next() {
            let consumed = stream.byte_offset();
            if consumed > 0 {
                return Some((value, trim_offset + byte_idx + consumed));
            }
        }
    }

    None
}

pub(super) fn strip_leading_close_tags(mut input: &str) -> &str {
    loop {
        let trimmed = input.trim_start();
        if !trimmed.starts_with("</") {
            return trimmed;
        }

        let Some(close_end) = trimmed.find('>') else {
            return "";
        };
        input = &trimmed[close_end + 1..];
    }
}

/// Extract JSON values from a string.
///
/// # Security Warning
///
/// This function extracts ANY JSON objects/arrays from the input. It MUST only
/// be used on content that is already trusted to be from the LLM, such as
/// content inside `<invoke>` tags where the LLM has explicitly indicated intent
/// to make a tool call. Do NOT use this on raw user input or content that
/// could contain prompt injection payloads.
pub(super) fn extract_json_values(input: &str) -> Vec<serde_json::Value> {
    let mut values = Vec::new();
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return values;
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        values.push(value);
        return values;
    }

    let char_positions: Vec<(usize, char)> = trimmed.char_indices().collect();
    let mut idx = 0;
    while idx < char_positions.len() {
        let (byte_idx, ch) = char_positions[idx];
        if ch == '{' || ch == '[' {
            let slice = &trimmed[byte_idx..];
            let mut stream =
                serde_json::Deserializer::from_str(slice).into_iter::<serde_json::Value>();
            if let Some(Ok(value)) = stream.next() {
                let consumed = stream.byte_offset();
                if consumed > 0 {
                    values.push(value);
                    let next_byte = byte_idx + consumed;
                    while idx < char_positions.len() && char_positions[idx].0 < next_byte {
                        idx += 1;
                    }
                    continue;
                }
            }
        }
        idx += 1;
    }

    values
}

/// Find the end position of a JSON object by tracking balanced braces.
pub(super) fn find_json_end(input: &str) -> Option<usize> {
    let trimmed = input.trim_start();
    let offset = input.len() - trimmed.len();

    if !trimmed.starts_with('{') {
        return None;
    }

    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in trimmed.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(offset + i + ch.len_utf8());
                }
            }
            _ => {}
        }
    }

    None
}

/// Parse XML attribute-style tool calls from response text.
/// This handles MiniMax and similar providers that output:
/// ```xml
/// <minimax:toolcall>
/// <invoke name="shell">
/// <parameter name="command">ls</parameter>
/// </invoke>
/// </minimax:toolcall>
/// ```
pub(super) fn parse_xml_attribute_tool_calls(response: &str) -> Vec<ParsedToolCall> {
    let mut calls = Vec::new();

    // Regex to find <invoke name="toolname">...</invoke> blocks
    static INVOKE_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?s)<invoke\s+name="([^"]+)"[^>]*>(.*?)</invoke>"#).unwrap()
    });

    // Regex to find <parameter name="paramname">value</parameter>
    static PARAM_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"<parameter\s+name="([^"]+)"[^>]*>([^<]*)</parameter>"#).unwrap()
    });

    for cap in INVOKE_RE.captures_iter(response) {
        let tool_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let inner = cap.get(2).map(|m| m.as_str()).unwrap_or("");

        if tool_name.is_empty() {
            continue;
        }

        let mut arguments = serde_json::Map::new();

        for param_cap in PARAM_RE.captures_iter(inner) {
            let param_name = param_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let param_value = param_cap.get(2).map(|m| m.as_str()).unwrap_or("");

            if !param_name.is_empty() {
                arguments.insert(
                    param_name.to_string(),
                    serde_json::Value::String(param_value.to_string()),
                );
            }
        }

        if !arguments.is_empty() {
            calls.push(ParsedToolCall {
                name: map_tool_name_alias(tool_name).to_string(),
                arguments: serde_json::Value::Object(arguments),
                tool_call_id: None,
            });
        }
    }

    calls
}

/// Parse Perl/hash-ref style tool calls from response text.
/// This handles formats like:
/// ```text
/// TOOL_CALL
/// {tool => "shell", args => {
///   --command "ls -la"
///   --description "List current directory contents"
/// }}
/// /TOOL_CALL
/// ```
pub(super) fn parse_perl_style_tool_calls(response: &str) -> Vec<ParsedToolCall> {
    let mut calls = Vec::new();

    // Regex to find TOOL_CALL blocks - handle double closing braces }}
    static PERL_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)TOOL_CALL\s*\{(.+?)\}\}\s*/TOOL_CALL").unwrap());

    // Regex to find tool => "name" in the content
    static TOOL_NAME_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"tool\s*=>\s*"([^"]+)""#).unwrap());

    // Regex to find args => { ... } block
    static ARGS_BLOCK_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)args\s*=>\s*\{(.+?)\}").unwrap());

    // Regex to find --key "value" pairs
    static ARGS_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"--(\w+)\s+"([^"]+)""#).unwrap());

    for cap in PERL_RE.captures_iter(response) {
        let content = cap.get(1).map(|m| m.as_str()).unwrap_or("");

        // Extract tool name
        let tool_name = TOOL_NAME_RE
            .captures(content)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .unwrap_or("");

        if tool_name.is_empty() {
            continue;
        }

        // Extract args block
        let args_block = ARGS_BLOCK_RE
            .captures(content)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .unwrap_or("");

        let mut arguments = serde_json::Map::new();

        for arg_cap in ARGS_RE.captures_iter(args_block) {
            let key = arg_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = arg_cap.get(2).map(|m| m.as_str()).unwrap_or("");

            if !key.is_empty() {
                arguments.insert(
                    key.to_string(),
                    serde_json::Value::String(value.to_string()),
                );
            }
        }

        if !arguments.is_empty() {
            calls.push(ParsedToolCall {
                name: map_tool_name_alias(tool_name).to_string(),
                arguments: serde_json::Value::Object(arguments),
                tool_call_id: None,
            });
        }
    }

    calls
}

/// Parse FunctionCall-style tool calls from response text.
/// This handles formats like:
/// ```text
/// <FunctionCall>
/// file_read
/// <code>path>/Users/kylelampa/Documents/zeroclaw/README.md</code>
/// </FunctionCall>
/// ```
pub(super) fn parse_function_call_tool_calls(response: &str) -> Vec<ParsedToolCall> {
    let mut calls = Vec::new();

    // Regex to find <FunctionCall> blocks
    static FUNC_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?s)<FunctionCall>\s*(\w+)\s*<code>([^<]+)</code>\s*</FunctionCall>").unwrap()
    });

    for cap in FUNC_RE.captures_iter(response) {
        let tool_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let args_text = cap.get(2).map(|m| m.as_str()).unwrap_or("");

        if tool_name.is_empty() {
            continue;
        }

        // Parse key>value pairs (e.g., path>/Users/.../file.txt)
        let mut arguments = serde_json::Map::new();
        for line in args_text.lines() {
            let line = line.trim();
            if let Some(pos) = line.find('>') {
                let key = line[..pos].trim();
                let value = line[pos + 1..].trim();
                if !key.is_empty() && !value.is_empty() {
                    arguments.insert(
                        key.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
            }
        }

        if !arguments.is_empty() {
            calls.push(ParsedToolCall {
                name: map_tool_name_alias(tool_name).to_string(),
                arguments: serde_json::Value::Object(arguments),
                tool_call_id: None,
            });
        }
    }

    calls
}

/// Parse GLM-style tool calls from response text.
/// Map tool name aliases from various LLM providers to ZeroClaw tool names.
/// This handles variations like "fileread" -> "file_read", "bash" -> "shell", etc.
pub(super) fn map_tool_name_alias(tool_name: &str) -> &str {
    match tool_name {
        // Shell variations (including GLM aliases that map to shell)
        "shell" | "bash" | "sh" | "exec" | "command" | "cmd" | "browser_open" | "browser"
        | "web_search" => "shell",
        // Messaging variations
        "send_message" | "sendmessage" => "message_send",
        // File tool variations
        "fileread" | "file_read" | "readfile" | "read_file" | "file" => "file_read",
        "filewrite" | "file_write" | "writefile" | "write_file" => "file_write",
        "filelist" | "file_list" | "listfiles" | "list_files" => "file_list",
        // Memory variations
        "memoryrecall" | "memory_recall" | "recall" | "memrecall" => "memory_recall",
        "memorystore" | "memory_store" | "store" | "memstore" => "memory_store",
        "memoryobserve" | "memory_observe" | "observe" | "memobserve" => "memory_observe",
        "memoryforget" | "memory_forget" | "forget" | "memforget" => "memory_forget",
        // HTTP variations
        "http_request" | "http" | "fetch" | "curl" | "wget" => "http_request",
        _ => tool_name,
    }
}

fn is_probable_direct_xml_tool_name(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    if normalized.is_empty() || is_xml_meta_tag(&normalized) {
        return false;
    }

    if map_tool_name_alias(&normalized) != normalized {
        return true;
    }

    normalized.contains('_')
        || matches!(
            normalized.as_str(),
            "shell"
                | "http"
                | "curl"
                | "wget"
                | "fetch"
                | "browser"
                | "message"
                | "notify"
                | "recall"
                | "store"
                | "forget"
                | "search"
                | "read"
                | "write"
                | "edit"
                | "list"
        )
}

pub(super) fn build_curl_command(url: &str) -> Option<String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return None;
    }

    if url.chars().any(char::is_whitespace) {
        return None;
    }

    let escaped = url.replace('\'', r#"'\\''"#);
    Some(format!("curl -s '{}'", escaped))
}

pub(super) fn parse_glm_style_tool_calls(
    text: &str,
) -> Vec<(String, serde_json::Value, Option<String>)> {
    let mut calls = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Format: tool_name/param>value or tool_name/{json}
        if let Some(pos) = line.find('/') {
            let tool_part = &line[..pos];
            let rest = &line[pos + 1..];

            if tool_part.chars().all(|c| c.is_alphanumeric() || c == '_') {
                let tool_name = map_tool_name_alias(tool_part);

                if let Some(gt_pos) = rest.find('>') {
                    let param_name = rest[..gt_pos].trim();
                    let value = rest[gt_pos + 1..].trim();

                    let arguments = match tool_name {
                        "shell" => {
                            if param_name == "url" {
                                let Some(command) = build_curl_command(value) else {
                                    continue;
                                };
                                serde_json::json!({ "command": command })
                            } else if value.starts_with("http://") || value.starts_with("https://")
                            {
                                if let Some(command) = build_curl_command(value) {
                                    serde_json::json!({ "command": command })
                                } else {
                                    serde_json::json!({ "command": value })
                                }
                            } else {
                                serde_json::json!({ "command": value })
                            }
                        }
                        "http_request" => {
                            serde_json::json!({"url": value, "method": "GET"})
                        }
                        _ => serde_json::json!({ param_name: value }),
                    };

                    calls.push((tool_name.to_string(), arguments, Some(line.to_string())));
                    continue;
                }

                if rest.starts_with('{') {
                    if let Ok(json_args) = serde_json::from_str::<serde_json::Value>(rest) {
                        calls.push((tool_name.to_string(), json_args, Some(line.to_string())));
                    }
                }
            }
        }

        // Plain URL
        if let Some(command) = build_curl_command(line) {
            calls.push((
                "shell".to_string(),
                serde_json::json!({ "command": command }),
                Some(line.to_string()),
            ));
        }
    }

    calls
}

/// Return the canonical default parameter name for a tool.
///
/// When a model emits a shortened call like `shell>uname -a` (without an
/// explicit `/param_name`), we need to infer which parameter the value maps
/// to. This function encodes the mapping for known ZeroClaw tools.
pub(super) fn default_param_for_tool(tool: &str) -> &'static str {
    match tool {
        "shell" | "bash" | "sh" | "exec" | "command" | "cmd" => "command",
        // All file tools default to "path"
        "file_read" | "fileread" | "readfile" | "read_file" | "file" | "file_write"
        | "filewrite" | "writefile" | "write_file" | "file_edit" | "fileedit" | "editfile"
        | "edit_file" | "file_list" | "filelist" | "listfiles" | "list_files" => "path",
        // Memory recall and forget both default to "query"
        "memory_recall" | "memoryrecall" | "recall" | "memrecall" | "memory_forget"
        | "memoryforget" | "forget" | "memforget" => "query",
        "memory_store" | "memorystore" | "store" | "memstore" => "content",
        "memory_observe" | "memoryobserve" | "observe" | "memobserve" => "observation",
        // HTTP and browser tools default to "url"
        "http_request" | "http" | "fetch" | "curl" | "wget" | "browser_open" | "browser"
        | "web_search" => "url",
        _ => "input",
    }
}

/// Parse GLM-style shortened tool call bodies found inside `<tool_call>` tags.
///
/// Handles three sub-formats that GLM-4.7 emits:
///
/// 1. **Shortened**: `tool_name>value` — single value mapped via
///    [`default_param_for_tool`].
/// 2. **YAML-like multi-line**: `tool_name>\nkey: value\nkey: value` — each
///    subsequent `key: value` line becomes a parameter.
/// 3. **Attribute-style**: `tool_name key="value" [/]>` — XML-like attributes.
///
/// Returns `None` if the body does not match any of these formats.
pub(super) fn parse_glm_shortened_body(body: &str) -> Option<ParsedToolCall> {
    let body = body.trim();
    if body.is_empty() {
        return None;
    }

    let function_style = body.find('(').and_then(|open| {
        if body.ends_with(')') && open > 0 {
            Some((body[..open].trim(), body[open + 1..body.len() - 1].trim()))
        } else {
            None
        }
    });

    // Check attribute-style FIRST: `tool_name key="value" />`
    // Must come before `>` check because `/>` contains `>` and would
    // misparse the tool name in the first branch.
    let (tool_raw, value_part) = if let Some((tool, args)) = function_style {
        (tool, args)
    } else if body.contains("=\"") {
        // Attribute-style: split at first whitespace to get tool name
        let split_pos = body.find(|c: char| c.is_whitespace()).unwrap_or(body.len());
        let tool = body[..split_pos].trim();
        let attrs = body[split_pos..]
            .trim()
            .trim_end_matches("/>")
            .trim_end_matches('>')
            .trim_end_matches('/')
            .trim();
        (tool, attrs)
    } else if let Some(gt_pos) = body.find('>') {
        // GLM shortened: `tool_name>value`
        let tool = body[..gt_pos].trim();
        let value = body[gt_pos + 1..].trim();
        // Strip trailing self-close markers that some models emit
        let value = value.trim_end_matches("/>").trim_end_matches('/').trim();
        (tool, value)
    } else {
        return None;
    };

    // Validate tool name: must be alphanumeric + underscore only
    let tool_raw = tool_raw.trim_end_matches(|c: char| c.is_whitespace());
    if tool_raw.is_empty() || !tool_raw.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }

    let tool_name = map_tool_name_alias(tool_raw);

    // Try attribute-style: `key="value" key2="value2"`
    if value_part.contains("=\"") {
        let mut args = serde_json::Map::new();
        // Simple attribute parser: key="value" pairs
        let mut rest = value_part;
        while let Some(eq_pos) = rest.find("=\"") {
            let key_start = rest[..eq_pos]
                .rfind(|c: char| c.is_whitespace())
                .map(|p| p + 1)
                .unwrap_or(0);
            let key = rest[key_start..eq_pos]
                .trim()
                .trim_matches(|c: char| c == ',' || c == ';');
            let after_quote = &rest[eq_pos + 2..];
            if let Some(end_quote) = after_quote.find('"') {
                let value = &after_quote[..end_quote];
                if !key.is_empty() {
                    args.insert(
                        key.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
                rest = &after_quote[end_quote + 1..];
            } else {
                break;
            }
        }
        if !args.is_empty() {
            return Some(ParsedToolCall {
                name: tool_name.to_string(),
                arguments: serde_json::Value::Object(args),
                tool_call_id: None,
            });
        }
    }

    // Try YAML-style multi-line: each line is `key: value`
    if value_part.contains('\n') {
        let mut args = serde_json::Map::new();
        for line in value_part.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim();
                if !key.is_empty() && !value.is_empty() {
                    // Normalize boolean-like values
                    let json_value = match value {
                        "true" | "yes" => serde_json::Value::Bool(true),
                        "false" | "no" => serde_json::Value::Bool(false),
                        _ => serde_json::Value::String(value.to_string()),
                    };
                    args.insert(key.to_string(), json_value);
                }
            }
        }
        if !args.is_empty() {
            return Some(ParsedToolCall {
                name: tool_name.to_string(),
                arguments: serde_json::Value::Object(args),
                tool_call_id: None,
            });
        }
    }

    // Single-value shortened: `tool>value`
    if !value_part.is_empty() {
        let param = default_param_for_tool(tool_raw);
        let arguments = match tool_name {
            "shell" => {
                if value_part.starts_with("http://") || value_part.starts_with("https://") {
                    if let Some(cmd) = build_curl_command(value_part) {
                        serde_json::json!({ "command": cmd })
                    } else {
                        serde_json::json!({ "command": value_part })
                    }
                } else {
                    serde_json::json!({ "command": value_part })
                }
            }
            "http_request" => serde_json::json!({"url": value_part, "method": "GET"}),
            _ => serde_json::json!({ param: value_part }),
        };
        return Some(ParsedToolCall {
            name: tool_name.to_string(),
            arguments,
            tool_call_id: None,
        });
    }

    None
}

// ── Tool-Call Parsing ─────────────────────────────────────────────────────
// LLM responses may contain tool calls in multiple formats depending on
// the provider. Parsing follows a priority chain:
//   1. OpenAI-style JSON with `tool_calls` array (native API)
//   2. XML tags: <tool_call>, <toolcall>, <tool-call>, <invoke>
//   3. Markdown code blocks with `tool_call` language
//   4. GLM-style line-based format (e.g. `shell/command>ls`)
// SECURITY: We never fall back to extracting arbitrary JSON from the
// response body, because that would enable prompt-injection attacks where
// malicious content in emails/files/web pages mimics a tool call.

/// Parse tool calls from an LLM response that uses XML-style function calling.
///
/// Expected format (common with system-prompt-guided tool use):
/// ```text
/// <tool_call>
/// {"name": "shell", "arguments": {"command": "ls"}}
/// </tool_call>
/// ```
///
/// Also accepts common tag variants (`<toolcall>`, `<tool-call>`) for model
/// compatibility.
///
/// Also supports JSON with `tool_calls` array from OpenAI-format responses.
pub(super) fn parse_tool_calls(response: &str) -> (String, Vec<ParsedToolCall>) {
    let mut text_parts = Vec::new();
    let mut calls = Vec::new();
    let mut remaining = response;

    // First, try to parse as OpenAI-style JSON response with tool_calls array
    // This handles providers like Minimax that return tool_calls in native JSON format
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response.trim()) {
        calls = parse_tool_calls_from_json_value(&json_value);
        if !calls.is_empty() {
            // If we found tool_calls, extract any content field as text.
            // Some providers wrap tool calls under `message` or `choices[*].message`.
            if let Some(content) = extract_tool_text_from_json_value(&json_value) {
                text_parts.push(content);
            }
            return (text_parts.join("\n"), calls);
        }
    }

    if let Some((minimax_text, minimax_calls)) = parse_minimax_invoke_calls(response) {
        if !minimax_calls.is_empty() {
            return (minimax_text, minimax_calls);
        }
    }

    // Fall back to XML-style tool-call tag parsing.
    while let Some((start, open_tag)) = find_first_tag(remaining, &TOOL_CALL_OPEN_TAGS) {
        // Everything before the tag is text
        let before = &remaining[..start];
        if !before.trim().is_empty() {
            text_parts.push(before.trim().to_string());
        }

        let Some(close_tag) = matching_tool_call_close_tag(open_tag) else {
            break;
        };

        let after_open = &remaining[start + open_tag.len()..];
        if let Some(close_idx) = after_open.find(close_tag) {
            let inner = &after_open[..close_idx];
            let mut parsed_any = false;

            // Try JSON format first
            let json_values = extract_json_values(inner);
            for value in json_values {
                let parsed_calls = parse_tool_calls_from_json_value(&value);
                if !parsed_calls.is_empty() {
                    parsed_any = true;
                    calls.extend(parsed_calls);
                }
            }

            // If JSON parsing failed, try XML format (DeepSeek/GLM style)
            if !parsed_any {
                if let Some(xml_calls) = parse_xml_tool_calls(inner) {
                    calls.extend(xml_calls);
                    parsed_any = true;
                }
            }

            if !parsed_any {
                // GLM-style shortened body: `shell>uname -a` or `shell\ncommand: date`
                if let Some(glm_call) = parse_glm_shortened_body(inner) {
                    calls.push(glm_call);
                    parsed_any = true;
                }
            }

            if !parsed_any {
                tracing::warn!(
                    "Malformed <tool_call>: expected tool-call object in tag body (JSON/XML/GLM)"
                );
            }

            remaining = &after_open[close_idx + close_tag.len()..];
        } else {
            // Matching close tag not found — try cross-alias close tags first.
            // Models sometimes mix open/close tag aliases (e.g. <tool_call>...</invoke>).
            let mut resolved = false;
            if let Some((cross_idx, cross_tag)) = find_first_tag(after_open, &TOOL_CALL_CLOSE_TAGS)
            {
                let inner = &after_open[..cross_idx];
                let mut parsed_any = false;

                // Try JSON
                let json_values = extract_json_values(inner);
                for value in json_values {
                    let parsed_calls = parse_tool_calls_from_json_value(&value);
                    if !parsed_calls.is_empty() {
                        parsed_any = true;
                        calls.extend(parsed_calls);
                    }
                }

                // Try XML
                if !parsed_any {
                    if let Some(xml_calls) = parse_xml_tool_calls(inner) {
                        calls.extend(xml_calls);
                        parsed_any = true;
                    }
                }

                // Try GLM shortened body
                if !parsed_any {
                    if let Some(glm_call) = parse_glm_shortened_body(inner) {
                        calls.push(glm_call);
                        parsed_any = true;
                    }
                }

                if parsed_any {
                    remaining = &after_open[cross_idx + cross_tag.len()..];
                    resolved = true;
                }
            }

            if resolved {
                continue;
            }

            // No cross-alias close tag resolved — fall back to JSON recovery
            // from unclosed tags (brace-balancing).
            if let Some(json_end) = find_json_end(after_open) {
                if let Ok(value) =
                    serde_json::from_str::<serde_json::Value>(&after_open[..json_end])
                {
                    let parsed_calls = parse_tool_calls_from_json_value(&value);
                    if !parsed_calls.is_empty() {
                        calls.extend(parsed_calls);
                        remaining = strip_leading_close_tags(&after_open[json_end..]);
                        continue;
                    }
                }
            }

            if let Some((value, consumed_end)) = extract_first_json_value_with_end(after_open) {
                let parsed_calls = parse_tool_calls_from_json_value(&value);
                if !parsed_calls.is_empty() {
                    calls.extend(parsed_calls);
                    remaining = strip_leading_close_tags(&after_open[consumed_end..]);
                    continue;
                }
            }

            // Last resort: try GLM shortened body on everything after the open tag.
            // The model may have emitted `<tool_call>shell>ls` with no close tag at all.
            let glm_input = after_open.trim();
            if let Some(glm_call) = parse_glm_shortened_body(glm_input) {
                calls.push(glm_call);
                remaining = "";
                continue;
            }

            remaining = &remaining[start..];
            break;
        }
    }

    // If XML tags found nothing, try markdown code blocks with tool_call language.
    // Models behind OpenRouter sometimes output ```tool_call ... ``` or hybrid
    // ```tool_call ... </tool_call> instead of structured API calls or XML tags.
    if calls.is_empty() {
        static MD_TOOL_CALL_RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r"(?s)```(?:tool[_-]?call|invoke)\s*\n(.*?)(?:```|</tool[_-]?call>|</toolcall>|</invoke>|</minimax:toolcall>)",
            )
            .unwrap()
        });
        let mut md_text_parts: Vec<String> = Vec::new();
        let mut last_end = 0;

        for cap in MD_TOOL_CALL_RE.captures_iter(response) {
            let full_match = cap.get(0).unwrap();
            let before = &response[last_end..full_match.start()];
            if !before.trim().is_empty() {
                md_text_parts.push(before.trim().to_string());
            }
            let inner = &cap[1];
            let json_values = extract_json_values(inner);
            for value in json_values {
                let parsed_calls = parse_tool_calls_from_json_value(&value);
                calls.extend(parsed_calls);
            }
            last_end = full_match.end();
        }

        if !calls.is_empty() {
            let after = &response[last_end..];
            if !after.trim().is_empty() {
                md_text_parts.push(after.trim().to_string());
            }
            text_parts = md_text_parts;
            remaining = "";
        }
    }

    // Try ```tool <name> format used by some providers (e.g., xAI grok)
    // Example: ```tool file_write\n{"path": "...", "content": "..."}\n```
    if calls.is_empty() {
        static MD_TOOL_NAME_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(?s)```tool\s+(\w+)\s*\n(.*?)(?:```|$)").unwrap());
        let mut md_text_parts: Vec<String> = Vec::new();
        let mut last_end = 0;

        for cap in MD_TOOL_NAME_RE.captures_iter(response) {
            let full_match = cap.get(0).unwrap();
            let before = &response[last_end..full_match.start()];
            if !before.trim().is_empty() {
                md_text_parts.push(before.trim().to_string());
            }
            let tool_name = &cap[1];
            let inner = &cap[2];

            // Try to parse the inner content as JSON arguments
            let json_values = extract_json_values(inner);
            if json_values.is_empty() {
                // Log a warning if we found a tool block but couldn't parse arguments
                tracing::warn!(
                    tool_name = %tool_name,
                    inner = %inner.chars().take(100).collect::<String>(),
                    "Found ```tool <name> block but could not parse JSON arguments"
                );
            } else {
                for value in json_values {
                    let arguments = if value.is_object() {
                        value
                    } else {
                        serde_json::Value::Object(serde_json::Map::new())
                    };
                    calls.push(ParsedToolCall {
                        name: tool_name.to_string(),
                        arguments,
                        tool_call_id: None,
                    });
                }
            }
            last_end = full_match.end();
        }

        if !calls.is_empty() {
            let after = &response[last_end..];
            if !after.trim().is_empty() {
                md_text_parts.push(after.trim().to_string());
            }
            text_parts = md_text_parts;
            remaining = "";
        }
    }

    // Direct XML tool tags (without <tool_call> wrapper), e.g.:
    // <shell>pwd</shell>
    // <file_write><path>...</path><content>...</content></file_write>
    if calls.is_empty() {
        if let Some(xml_calls) = parse_xml_tool_calls(remaining) {
            let direct_calls: Vec<ParsedToolCall> = xml_calls
                .into_iter()
                .filter(|call| is_probable_direct_xml_tool_name(&call.name))
                .collect();
            if !direct_calls.is_empty() {
                let mut cleaned_text = remaining.to_string();
                let parsed_names: HashSet<&str> =
                    direct_calls.iter().map(|call| call.name.as_str()).collect();

                for (tag_name, _) in extract_xml_pairs(remaining) {
                    let canonical_tag = map_tool_name_alias(tag_name);
                    if !parsed_names.contains(tag_name) && !parsed_names.contains(canonical_tag) {
                        continue;
                    }

                    let open = format!("<{tag_name}>");
                    let close = format!("</{tag_name}>");
                    while let Some(start) = cleaned_text.find(&open) {
                        let search_from = start + open.len();
                        let Some(end_rel) = cleaned_text[search_from..].find(&close) else {
                            break;
                        };
                        let end = search_from + end_rel + close.len();
                        cleaned_text.replace_range(start..end, "");
                    }
                }

                calls.extend(direct_calls);
                if !cleaned_text.trim().is_empty() {
                    text_parts.push(cleaned_text.trim().to_string());
                }
                remaining = "";
            }
        }
    }

    // XML attribute-style tool calls:
    // <minimax:toolcall>
    // <invoke name="shell">
    // <parameter name="command">ls</parameter>
    // </invoke>
    // </minimax:toolcall>
    if calls.is_empty() {
        let xml_calls = parse_xml_attribute_tool_calls(remaining);
        if !xml_calls.is_empty() {
            let mut cleaned_text = remaining.to_string();
            for call in xml_calls {
                calls.push(call);
                // Try to remove the XML from text
                if let Some(start) = cleaned_text.find("<minimax:toolcall>") {
                    if let Some(end) = cleaned_text.find("</minimax:toolcall>") {
                        let end_pos = end + "</minimax:toolcall>".len();
                        if end_pos <= cleaned_text.len() {
                            cleaned_text =
                                format!("{}{}", &cleaned_text[..start], &cleaned_text[end_pos..]);
                        }
                    }
                }
            }
            if !cleaned_text.trim().is_empty() {
                text_parts.push(cleaned_text.trim().to_string());
            }
            remaining = "";
        }
    }

    // Perl/hash-ref style tool calls:
    // TOOL_CALL
    // {tool => "shell", args => {
    //   --command "ls -la"
    //   --description "List current directory contents"
    // }}
    // /TOOL_CALL
    if calls.is_empty() {
        let perl_calls = parse_perl_style_tool_calls(remaining);
        if !perl_calls.is_empty() {
            let mut cleaned_text = remaining.to_string();
            for call in perl_calls {
                calls.push(call);
                // Try to remove the TOOL_CALL block from text
                while let Some(start) = cleaned_text.find("TOOL_CALL") {
                    if let Some(end) = cleaned_text.find("/TOOL_CALL") {
                        let end_pos = end + "/TOOL_CALL".len();
                        if end_pos <= cleaned_text.len() {
                            cleaned_text =
                                format!("{}{}", &cleaned_text[..start], &cleaned_text[end_pos..]);
                        }
                    } else {
                        break;
                    }
                }
            }
            if !cleaned_text.trim().is_empty() {
                text_parts.push(cleaned_text.trim().to_string());
            }
            remaining = "";
        }
    }

    // <FunctionCall>
    // file_read
    // <code>path>/Users/...</code>
    // </FunctionCall>
    if calls.is_empty() {
        let func_calls = parse_function_call_tool_calls(remaining);
        if !func_calls.is_empty() {
            let mut cleaned_text = remaining.to_string();
            for call in func_calls {
                calls.push(call);
                // Try to remove the FunctionCall block from text
                while let Some(start) = cleaned_text.find("<FunctionCall>") {
                    if let Some(end) = cleaned_text.find("</FunctionCall>") {
                        let end_pos = end + "</FunctionCall>".len();
                        if end_pos <= cleaned_text.len() {
                            cleaned_text =
                                format!("{}{}", &cleaned_text[..start], &cleaned_text[end_pos..]);
                        }
                    } else {
                        break;
                    }
                }
            }
            if !cleaned_text.trim().is_empty() {
                text_parts.push(cleaned_text.trim().to_string());
            }
            remaining = "";
        }
    }

    // GLM-style tool calls (browser_open/url>https://..., shell/command>ls, etc.)
    if calls.is_empty() {
        let glm_calls = parse_glm_style_tool_calls(remaining);
        if !glm_calls.is_empty() {
            let mut cleaned_text = remaining.to_string();
            for (name, args, raw) in &glm_calls {
                calls.push(ParsedToolCall {
                    name: name.clone(),
                    arguments: args.clone(),
                    tool_call_id: None,
                });
                if let Some(r) = raw {
                    cleaned_text = cleaned_text.replace(r, "");
                }
            }
            if !cleaned_text.trim().is_empty() {
                text_parts.push(cleaned_text.trim().to_string());
            }
            remaining = "";
        }
    }

    // SECURITY: We do NOT fall back to extracting arbitrary JSON from the response
    // here. That would enable prompt injection attacks where malicious content
    // (e.g., in emails, files, or web pages) could include JSON that mimics a
    // tool call. Tool calls MUST be explicitly wrapped in either:
    // 1. OpenAI-style JSON with a "tool_calls" array
    // 2. ZeroClaw tool-call tags (<tool_call>, <toolcall>, <tool-call>)
    // 3. Markdown code blocks with tool_call/toolcall/tool-call language
    // 4. Explicit GLM line-based call formats (e.g. `shell/command>...`)
    // This ensures only the LLM's intentional tool calls are executed.

    // Remaining text after last tool call
    if !remaining.trim().is_empty() {
        text_parts.push(remaining.trim().to_string());
    }

    (text_parts.join("\n"), calls)
}

pub(super) fn detect_tool_call_parse_issue(
    response: &str,
    parsed_calls: &[ParsedToolCall],
) -> Option<String> {
    if !parsed_calls.is_empty() {
        return None;
    }

    let trimmed = response.trim();
    if trimmed.is_empty() {
        return None;
    }

    let looks_like_tool_payload = trimmed.contains("<tool_call")
        || trimmed.contains("<toolcall")
        || trimmed.contains("<tool-call")
        || trimmed.contains("<shell>")
        || trimmed.contains("<file_write>")
        || trimmed.contains("<file_read>")
        || trimmed.contains("<memory_recall>")
        || trimmed.contains("```tool_call")
        || trimmed.contains("```toolcall")
        || trimmed.contains("```tool-call")
        || trimmed.contains("```tool file_")
        || trimmed.contains("```tool shell")
        || trimmed.contains("```tool web_")
        || trimmed.contains("```tool memory_")
        || trimmed.contains("```tool ") // Generic ```tool <name> pattern
        || trimmed.contains("\"tool_calls\"")
        || trimmed.contains("TOOL_CALL")
        || trimmed.contains("<FunctionCall>");

    if looks_like_tool_payload {
        Some("response resembled a tool-call payload but no valid tool call could be parsed".into())
    } else {
        None
    }
}

pub(super) fn parse_structured_tool_calls(tool_calls: &[ToolCall]) -> Vec<ParsedToolCall> {
    tool_calls
        .iter()
        .map(|call| {
            let name = call.name.clone();
            let parsed = serde_json::from_str::<serde_json::Value>(&call.arguments)
                .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
            ParsedToolCall {
                name: name.clone(),
                arguments: normalize_tool_arguments(&name, parsed, Some(call.arguments.as_str())),
                tool_call_id: Some(call.id.clone()),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── REQ-PARSE-001: Argument Parsing ──────────────────────────────

    /// REQ-PARSE-001-SC01
    #[test]
    fn parse_arguments_value_json_string() {
        let raw = json!(r#"{"command":"ls"}"#);
        let result = parse_arguments_value(Some(&raw));
        assert_eq!(result, json!({"command": "ls"}));
    }

    /// REQ-PARSE-001-SC02
    #[test]
    fn parse_arguments_value_object_passthrough() {
        let raw = json!({"command": "ls"});
        let result = parse_arguments_value(Some(&raw));
        assert_eq!(result, json!({"command": "ls"}));
    }

    /// REQ-PARSE-001-SC03
    #[test]
    fn parse_arguments_value_none_returns_empty() {
        let result = parse_arguments_value(None);
        assert_eq!(result, json!({}));
    }

    /// REQ-PARSE-001-SC04
    #[test]
    fn parse_arguments_value_invalid_json_returns_empty() {
        let raw = json!("this is not json");
        let result = parse_arguments_value(Some(&raw));
        assert_eq!(result, json!({}));
    }

    // ── REQ-PARSE-002: Shell Command Normalization ───────────────────

    /// REQ-PARSE-002-SC01
    #[test]
    fn shell_normalize_plain_command() {
        assert_eq!(
            normalize_shell_command_from_raw("ls -la"),
            Some("ls -la".to_string())
        );
    }

    /// REQ-PARSE-002-SC02
    #[test]
    fn shell_normalize_strips_quotes() {
        assert_eq!(
            normalize_shell_command_from_raw("\"echo hello\""),
            Some("echo hello".to_string())
        );
        assert_eq!(
            normalize_shell_command_from_raw("'echo hello'"),
            Some("echo hello".to_string())
        );
    }

    /// REQ-PARSE-002-SC03
    #[test]
    fn shell_normalize_url_to_curl() {
        let result = normalize_shell_command_from_raw("https://example.com/api");
        assert!(result.is_some());
        let cmd = result.unwrap();
        assert!(cmd.starts_with("curl -s"));
        assert!(cmd.contains("example.com"));
    }

    /// REQ-PARSE-002-SC04
    #[test]
    fn shell_normalize_empty_returns_none() {
        assert_eq!(normalize_shell_command_from_raw(""), None);
        assert_eq!(normalize_shell_command_from_raw("   "), None);
        assert_eq!(normalize_shell_command_from_raw("\"\""), None);
    }

    /// REQ-PARSE-002-SC05
    #[test]
    fn shell_normalize_json_returns_none() {
        assert_eq!(
            normalize_shell_command_from_raw(r#"{"key": "value"}"#),
            None
        );
        assert_eq!(normalize_shell_command_from_raw("[1,2,3]"), None);
    }

    // ── REQ-PARSE-003: Shell Arguments Normalization ─────────────────

    /// REQ-PARSE-003-SC01
    #[test]
    fn shell_args_command_key_preserved() {
        let args = json!({"command": "echo hi"});
        let result = normalize_shell_arguments(args.clone(), None);
        assert_eq!(result, args);
    }

    /// REQ-PARSE-003-SC02
    #[test]
    fn shell_args_alias_mapped_to_command() {
        for alias in ["cmd", "script", "bash", "sh", "input"] {
            let args = json!({alias: "echo test"});
            let result = normalize_shell_arguments(args, None);
            assert_eq!(
                result.get("command").and_then(|v| v.as_str()),
                Some("echo test"),
                "alias {alias} should map to command"
            );
        }
    }

    /// REQ-PARSE-003-SC03
    #[test]
    fn shell_args_url_to_curl() {
        let args = json!({"url": "https://example.com"});
        let result = normalize_shell_arguments(args, None);
        let cmd = result.get("command").and_then(|v| v.as_str()).unwrap();
        assert!(cmd.starts_with("curl -s"));
    }

    /// REQ-PARSE-003-SC04
    #[test]
    fn shell_args_string_to_command() {
        let args = json!("echo hello");
        let result = normalize_shell_arguments(args, None);
        assert_eq!(
            result.get("command").and_then(|v| v.as_str()),
            Some("echo hello")
        );
    }

    // ── REQ-PARSE-004: Tool Name Aliasing ────────────────────────────

    /// REQ-PARSE-004-SC01
    #[test]
    fn alias_shell_variants() {
        for name in [
            "bash",
            "sh",
            "exec",
            "command",
            "cmd",
            "browser_open",
            "browser",
            "web_search",
        ] {
            assert_eq!(map_tool_name_alias(name), "shell", "alias for {name}");
        }
    }

    /// REQ-PARSE-004-SC02
    #[test]
    fn alias_file_read_variants() {
        for name in ["fileread", "readfile", "read_file", "file"] {
            assert_eq!(map_tool_name_alias(name), "file_read", "alias for {name}");
        }
    }

    /// REQ-PARSE-004-SC03
    #[test]
    fn alias_memory_variants() {
        for name in ["memoryrecall", "recall", "memrecall"] {
            assert_eq!(
                map_tool_name_alias(name),
                "memory_recall",
                "alias for {name}"
            );
        }
    }

    /// REQ-PARSE-004-SC04
    #[test]
    fn alias_unknown_passthrough() {
        assert_eq!(map_tool_name_alias("my_custom_tool"), "my_custom_tool");
        assert_eq!(map_tool_name_alias("foobar"), "foobar");
    }

    // ── REQ-PARSE-005: XML Pair Extraction ───────────────────────────

    /// REQ-PARSE-005-SC01
    #[test]
    fn xml_pairs_single() {
        let pairs = extract_xml_pairs("<shell>pwd</shell>");
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("shell", "pwd"));
    }

    /// REQ-PARSE-005-SC02
    #[test]
    fn xml_pairs_multiple() {
        let input = "<query>search term</query><path>/tmp/file.txt</path>";
        let pairs = extract_xml_pairs(input);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0, "query");
        assert_eq!(pairs[1].0, "path");
    }

    /// REQ-PARSE-005-SC03
    #[test]
    fn xml_pairs_unclosed_skipped() {
        let pairs = extract_xml_pairs("<shell>pwd");
        assert!(pairs.is_empty());
    }

    // ── REQ-PARSE-006: XML Meta Tag Detection ────────────────────────

    /// REQ-PARSE-006-SC01
    #[test]
    fn meta_tags_detected() {
        for tag in [
            "tool_call",
            "toolcall",
            "tool-call",
            "invoke",
            "thinking",
            "thought",
            "analysis",
            "reasoning",
            "reflection",
        ] {
            assert!(is_xml_meta_tag(tag), "{tag} should be meta");
        }
        // Case-insensitive
        assert!(is_xml_meta_tag("TOOL_CALL"));
        assert!(is_xml_meta_tag("Thinking"));
    }

    /// REQ-PARSE-006-SC02
    #[test]
    fn tool_names_not_meta() {
        for tag in ["shell", "file_read", "memory_recall", "http_request"] {
            assert!(!is_xml_meta_tag(tag), "{tag} should NOT be meta");
        }
    }

    // ── REQ-PARSE-007: XML Tool Call Parsing ─────────────────────────

    /// REQ-PARSE-007-SC01
    #[test]
    fn xml_tool_calls_nested_args() {
        let input = "<memory_recall><query>test query</query></memory_recall>";
        let calls = parse_xml_tool_calls(input).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "memory_recall");
        assert_eq!(
            calls[0].arguments.get("query").and_then(|v| v.as_str()),
            Some("test query")
        );
    }

    /// REQ-PARSE-007-SC02
    #[test]
    fn xml_tool_calls_json_body() {
        let input = r#"<shell>{"command":"ls -la"}</shell>"#;
        let calls = parse_xml_tool_calls(input).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "shell");
        assert_eq!(
            calls[0].arguments.get("command").and_then(|v| v.as_str()),
            Some("ls -la")
        );
    }

    /// REQ-PARSE-007-SC03
    #[test]
    fn xml_tool_calls_meta_skipped() {
        // tool_call is a meta tag and should be skipped;
        // the inner content should still parse
        let input = "<tool_call><shell>pwd</shell></tool_call>";
        // parse_xml_tool_calls skips meta tags, so it should find shell inside
        let result = parse_xml_tool_calls(input);
        // The tool_call meta tag is skipped, inner shell tag should be found
        assert!(result.is_some());
        let calls = result.unwrap();
        assert_eq!(calls[0].name, "shell");
    }

    // ── REQ-PARSE-008: JSON Value Extraction ─────────────────────────

    /// REQ-PARSE-008-SC01
    #[test]
    fn json_extract_standalone() {
        let values = extract_json_values(r#"{"name":"shell"}"#);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], json!({"name": "shell"}));
    }

    /// REQ-PARSE-008-SC02
    #[test]
    fn json_extract_embedded() {
        let input = r#"Here is a tool call: {"name":"shell","arguments":{"command":"ls"}} done."#;
        let values = extract_json_values(input);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].get("name").and_then(|v| v.as_str()), Some("shell"));
    }

    /// REQ-PARSE-008-SC03
    #[test]
    fn json_extract_empty() {
        assert!(extract_json_values("").is_empty());
        assert!(extract_json_values("   ").is_empty());
    }

    // ── REQ-PARSE-009: JSON End Finder ───────────────────────────────

    /// REQ-PARSE-009-SC01
    #[test]
    fn json_end_simple() {
        let input = r#"{"key":"value"} trailing"#;
        let end = find_json_end(input).unwrap();
        assert_eq!(&input[..end], r#"{"key":"value"}"#);
    }

    /// REQ-PARSE-009-SC02
    #[test]
    fn json_end_nested() {
        let input = r#"{"a":{"b":"c"}} trailing"#;
        let end = find_json_end(input).unwrap();
        assert_eq!(&input[..end], r#"{"a":{"b":"c"}}"#);
    }

    /// REQ-PARSE-009-SC03
    #[test]
    fn json_end_escaped_braces() {
        let input = r#"{"msg":"hello {world}"} trailing"#;
        let end = find_json_end(input).unwrap();
        assert_eq!(&input[..end], r#"{"msg":"hello {world}"}"#);
    }

    /// REQ-PARSE-009-SC04
    #[test]
    fn json_end_non_object() {
        assert!(find_json_end("not json").is_none());
        assert!(find_json_end("[1,2,3]").is_none());
    }

    // ── REQ-PARSE-010: Build Curl Command ────────────────────────────

    /// REQ-PARSE-010-SC01
    #[test]
    fn curl_valid_url() {
        let result = build_curl_command("https://example.com/api");
        assert_eq!(result, Some("curl -s 'https://example.com/api'".to_string()));

        let result = build_curl_command("http://localhost:8080");
        assert_eq!(result, Some("curl -s 'http://localhost:8080'".to_string()));
    }

    /// REQ-PARSE-010-SC02
    #[test]
    fn curl_non_url() {
        assert_eq!(build_curl_command("ls -la"), None);
        assert_eq!(build_curl_command("ftp://server/file"), None);
    }

    /// REQ-PARSE-010-SC03
    #[test]
    fn curl_url_with_whitespace() {
        assert_eq!(build_curl_command("https://example.com/path with spaces"), None);
    }

    // ── REQ-PARSE-011: Default Parameter for Tool ────────────────────

    /// REQ-PARSE-011-SC01
    #[test]
    fn default_param_shell() {
        for tool in ["shell", "bash", "sh", "exec", "command", "cmd"] {
            assert_eq!(default_param_for_tool(tool), "command", "tool {tool}");
        }
    }

    /// REQ-PARSE-011-SC02
    #[test]
    fn default_param_file() {
        for tool in ["file_read", "file_write", "file_list", "fileread", "writefile"] {
            assert_eq!(default_param_for_tool(tool), "path", "tool {tool}");
        }
    }

    /// REQ-PARSE-011-SC03
    #[test]
    fn default_param_memory() {
        assert_eq!(default_param_for_tool("memory_recall"), "query");
        assert_eq!(default_param_for_tool("recall"), "query");
        assert_eq!(default_param_for_tool("memory_store"), "content");
        assert_eq!(default_param_for_tool("memory_observe"), "observation");
    }

    /// REQ-PARSE-011-SC04
    #[test]
    fn default_param_unknown() {
        assert_eq!(default_param_for_tool("my_custom_tool"), "input");
        assert_eq!(default_param_for_tool("foobar"), "input");
    }

    // ── REQ-PARSE-012: Tool Call Signature ───────────────────────────

    /// REQ-PARSE-012-SC01
    #[test]
    fn signature_sorts_keys() {
        let unsorted = json!({"z": 1, "a": 2, "m": 3});
        let canonical = canonicalize_json_for_tool_signature(&unsorted);
        let keys: Vec<&String> = canonical.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["a", "m", "z"]);
    }

    /// REQ-PARSE-012-SC02
    #[test]
    fn signature_lowercase_name() {
        let (name, _args) = tool_call_signature("SHELL", &json!({"command": "ls"}));
        assert_eq!(name, "shell");
    }

    // ── REQ-PARSE-013: GLM Shortened Body Parsing ────────────────────

    /// REQ-PARSE-013-SC01
    #[test]
    fn glm_shortened_single_value() {
        let call = parse_glm_shortened_body("shell>ls -la").unwrap();
        assert_eq!(call.name, "shell");
        assert_eq!(
            call.arguments.get("command").and_then(|v| v.as_str()),
            Some("ls -la")
        );
    }

    /// REQ-PARSE-013-SC02
    #[test]
    fn glm_shortened_yaml_multiline() {
        let body = "file_read>\npath: /tmp/test.txt\nencoding: utf-8";
        let call = parse_glm_shortened_body(body).unwrap();
        assert_eq!(call.name, "file_read");
        assert_eq!(
            call.arguments.get("path").and_then(|v| v.as_str()),
            Some("/tmp/test.txt")
        );
        assert_eq!(
            call.arguments.get("encoding").and_then(|v| v.as_str()),
            Some("utf-8")
        );
    }

    /// REQ-PARSE-013-SC03
    #[test]
    fn glm_shortened_attribute_style() {
        let body = r#"shell command="ls -la" description="list files""#;
        let call = parse_glm_shortened_body(body).unwrap();
        assert_eq!(call.name, "shell");
        assert_eq!(
            call.arguments.get("command").and_then(|v| v.as_str()),
            Some("ls -la")
        );
    }

    /// REQ-PARSE-013-SC04
    #[test]
    fn glm_shortened_url_to_curl() {
        let call = parse_glm_shortened_body("shell>https://example.com").unwrap();
        assert_eq!(call.name, "shell");
        let cmd = call.arguments.get("command").and_then(|v| v.as_str()).unwrap();
        assert!(cmd.starts_with("curl -s"));
        assert!(cmd.contains("example.com"));
    }

    /// REQ-PARSE-013-SC05
    #[test]
    fn glm_shortened_empty() {
        assert!(parse_glm_shortened_body("").is_none());
        assert!(parse_glm_shortened_body("   ").is_none());
    }

    // ── REQ-PARSE-014: Main Parse Entry Point ────────────────────────

    /// REQ-PARSE-014-SC01
    #[test]
    fn parse_openai_json_format() {
        let response = json!({
            "tool_calls": [{
                "function": {
                    "name": "shell",
                    "arguments": "{\"command\":\"ls\"}"
                },
                "id": "call_123"
            }]
        })
        .to_string();
        let (text, calls) = parse_tool_calls(&response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "shell");
        assert_eq!(
            calls[0].arguments.get("command").and_then(|v| v.as_str()),
            Some("ls")
        );
        assert_eq!(calls[0].tool_call_id, Some("call_123".to_string()));
        assert!(text.is_empty());
    }

    /// REQ-PARSE-014-SC02
    #[test]
    fn parse_xml_tag_format() {
        let response =
            r#"<tool_call>{"name":"shell","arguments":{"command":"pwd"}}</tool_call>"#;
        let (text, calls) = parse_tool_calls(response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "shell");
        assert_eq!(
            calls[0].arguments.get("command").and_then(|v| v.as_str()),
            Some("pwd")
        );
        assert!(text.is_empty());
    }

    /// REQ-PARSE-014-SC03
    #[test]
    fn parse_markdown_tool_call() {
        let response = "```tool_call\n{\"name\":\"shell\",\"arguments\":{\"command\":\"date\"}}\n```";
        let (_text, calls) = parse_tool_calls(response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "shell");
    }

    /// REQ-PARSE-014-SC04
    #[test]
    fn parse_preserves_surrounding_text() {
        let response = "Here is the result:\n<tool_call>{\"name\":\"shell\",\"arguments\":{\"command\":\"ls\"}}</tool_call>\nDone!";
        let (text, calls) = parse_tool_calls(response);
        assert_eq!(calls.len(), 1);
        assert!(text.contains("Here is the result:"));
        assert!(text.contains("Done!"));
    }

    /// REQ-PARSE-014-SC05
    #[test]
    fn parse_plain_text_no_calls() {
        let response = "The answer to your question is 42.";
        let (text, calls) = parse_tool_calls(response);
        assert!(calls.is_empty());
        assert_eq!(text, response);
    }

    // ── REQ-PARSE-015: Tool Call Parse Issue Detection ────────────────

    /// REQ-PARSE-015-SC01
    #[test]
    fn detect_issue_unparsed_tool_call() {
        let response = "<tool_call>malformed content here</tool_call>";
        let result = detect_tool_call_parse_issue(response, &[]);
        assert!(result.is_some());
        assert!(result.unwrap().contains("tool-call payload"));
    }

    /// REQ-PARSE-015-SC02
    #[test]
    fn detect_issue_clean_text() {
        let result = detect_tool_call_parse_issue("Hello, world!", &[]);
        assert!(result.is_none());
    }

    /// REQ-PARSE-015-SC03
    #[test]
    fn detect_issue_parsed_ok() {
        let calls = vec![ParsedToolCall {
            name: "shell".to_string(),
            arguments: json!({"command": "ls"}),
            tool_call_id: None,
        }];
        let result = detect_tool_call_parse_issue("<tool_call>...</tool_call>", &calls);
        assert!(result.is_none());
    }

    // ── REQ-PARSE-016: Structured Tool Call Parsing ──────────────────

    /// REQ-PARSE-016-SC01
    #[test]
    fn structured_valid_call() {
        let tool_calls = vec![ToolCall {
            id: "call_1".to_string(),
            name: "shell".to_string(),
            arguments: r#"{"command":"ls"}"#.to_string(),
        }];
        let result = parse_structured_tool_calls(&tool_calls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "shell");
        assert_eq!(
            result[0].arguments.get("command").and_then(|v| v.as_str()),
            Some("ls")
        );
        assert_eq!(result[0].tool_call_id, Some("call_1".to_string()));
    }

    /// REQ-PARSE-016-SC02
    #[test]
    fn structured_invalid_args() {
        let tool_calls = vec![ToolCall {
            id: "call_2".to_string(),
            name: "shell".to_string(),
            arguments: "not json".to_string(),
        }];
        let result = parse_structured_tool_calls(&tool_calls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "shell");
        // Invalid JSON falls back to empty; then raw_string_hint "not json"
        // may be used for shell normalization
        assert!(result[0].arguments.is_object());
    }

    // ── REQ-PARSE-017: MiniMax Invoke Parsing ────────────────────────

    /// REQ-PARSE-017-SC01
    #[test]
    fn minimax_standard_invoke() {
        let response =
            r#"<invoke name="shell"><parameter name="command">pwd</parameter></invoke>"#;
        let (text, calls) = parse_minimax_invoke_calls(response).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "shell");
        assert_eq!(
            calls[0].arguments.get("command").and_then(|v| v.as_str()),
            Some("pwd")
        );
        assert!(text.is_empty());
    }

    /// REQ-PARSE-017-SC02
    #[test]
    fn minimax_no_invoke() {
        assert!(parse_minimax_invoke_calls("Just some plain text").is_none());
    }

    // ── REQ-PARSE-018: Perl-Style Tool Call Parsing ──────────────────

    /// REQ-PARSE-018-SC01
    #[test]
    fn perl_style_standard() {
        let response = r#"TOOL_CALL
{tool => "shell", args => {
  --command "ls -la"
  --description "list dir"
}}
/TOOL_CALL"#;
        let calls = parse_perl_style_tool_calls(response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "shell");
        assert_eq!(
            calls[0].arguments.get("command").and_then(|v| v.as_str()),
            Some("ls -la")
        );
    }

    // ── REQ-PARSE-019: FunctionCall-Style Parsing ────────────────────

    /// REQ-PARSE-019-SC01
    #[test]
    fn function_call_standard() {
        let response = "<FunctionCall>\nfile_read\n<code>path>/tmp/file.txt</code>\n</FunctionCall>";
        let calls = parse_function_call_tool_calls(response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "file_read");
        assert_eq!(
            calls[0].arguments.get("path").and_then(|v| v.as_str()),
            Some("/tmp/file.txt")
        );
    }

    // ── REQ-PARSE-020: XML Attribute Tool Call Parsing ────────────────

    /// REQ-PARSE-020-SC01
    #[test]
    fn xml_attribute_standard() {
        let response = r#"<invoke name="shell"><parameter name="command">ls</parameter></invoke>"#;
        let calls = parse_xml_attribute_tool_calls(response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "shell");
        assert_eq!(
            calls[0].arguments.get("command").and_then(|v| v.as_str()),
            Some("ls")
        );
    }

    // ── REQ-PARSE-021: Tool Call ID Parsing ──────────────────────────

    /// REQ-PARSE-021-SC01
    #[test]
    fn tool_call_id_from_function() {
        let root = json!({"function": {"id": "fn_id_123", "name": "shell"}});
        let func = root.get("function");
        let id = parse_tool_call_id(&root, func);
        assert_eq!(id, Some("fn_id_123".to_string()));
    }

    /// REQ-PARSE-021-SC02
    #[test]
    fn tool_call_id_from_root() {
        let root = json!({"id": "root_id_456", "name": "shell"});
        let id = parse_tool_call_id(&root, None);
        assert_eq!(id, Some("root_id_456".to_string()));

        let root2 = json!({"tool_call_id": "tc_789", "name": "shell"});
        let id2 = parse_tool_call_id(&root2, None);
        assert_eq!(id2, Some("tc_789".to_string()));

        let root3 = json!({"call_id": "ci_012", "name": "shell"});
        let id3 = parse_tool_call_id(&root3, None);
        assert_eq!(id3, Some("ci_012".to_string()));
    }

    /// REQ-PARSE-021-SC03
    #[test]
    fn tool_call_id_none() {
        let root = json!({"name": "shell"});
        let id = parse_tool_call_id(&root, None);
        assert!(id.is_none());
    }

    // ── REQ-PARSE-022: Close Tag Matching ────────────────────────────

    /// REQ-PARSE-022-SC01
    #[test]
    fn close_tag_matching() {
        assert_eq!(
            matching_tool_call_close_tag("<tool_call>"),
            Some("</tool_call>")
        );
        assert_eq!(
            matching_tool_call_close_tag("<toolcall>"),
            Some("</toolcall>")
        );
        assert_eq!(
            matching_tool_call_close_tag("<tool-call>"),
            Some("</tool-call>")
        );
        assert_eq!(
            matching_tool_call_close_tag("<invoke>"),
            Some("</invoke>")
        );
        assert_eq!(
            matching_tool_call_close_tag("<minimax:tool_call>"),
            Some("</minimax:tool_call>")
        );
        assert_eq!(
            matching_tool_call_close_tag("<minimax:toolcall>"),
            Some("</minimax:toolcall>")
        );
    }

    /// REQ-PARSE-022-SC02
    #[test]
    fn close_tag_unknown() {
        assert!(matching_tool_call_close_tag("<unknown>").is_none());
        assert!(matching_tool_call_close_tag("not a tag").is_none());
    }
}
