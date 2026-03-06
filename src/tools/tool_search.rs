//! Tool search meta-tool for context-efficient tool discovery.
//!
//! When the registered tool count exceeds a configured threshold, this tool
//! replaces the full tool listing in the system prompt. The model calls
//! `tool_search(query)` to discover relevant tools by keyword, receiving their
//! full specs so it can invoke them on subsequent turns.

use async_trait::async_trait;
use serde_json::json;

use super::traits::{Tool, ToolResult, ToolSpec};

/// Lightweight keyword scorer for tool discovery.
///
/// Scores each tool against a query by counting keyword hits in the tool name
/// (weighted 3x) and description (weighted 1x). No embedding model required.
pub struct ToolSearchTool {
    catalog: Vec<ToolSpec>,
    max_results: usize,
}

impl ToolSearchTool {
    /// Create a new tool search over the given catalog.
    pub fn new(catalog: Vec<ToolSpec>, max_results: usize) -> Self {
        Self {
            catalog,
            max_results,
        }
    }

    /// Score a tool against query keywords.
    fn score_tool(spec: &ToolSpec, keywords: &[String]) -> f64 {
        let name_lower = spec.name.to_lowercase();
        // Split on underscores and hyphens for tool name matching
        let name_tokens: Vec<&str> = name_lower
            .split(|c: char| c == '_' || c == '-')
            .collect();
        let desc_lower = spec.description.to_lowercase();

        let mut score = 0.0;
        for kw in keywords {
            // Name substring match (weighted 3x)
            if name_lower.contains(kw.as_str()) {
                score += 3.0;
            }
            // Name token exact match (bonus 2x)
            if name_tokens.iter().any(|t| t == kw) {
                score += 2.0;
            }
            // Description substring match (weighted 1x)
            if desc_lower.contains(kw.as_str()) {
                score += 1.0;
            }
        }
        score
    }

    /// Search the catalog and return top matching specs.
    fn search(&self, query: &str) -> Vec<&ToolSpec> {
        let keywords: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() >= 2) // skip single-char noise
            .map(String::from)
            .collect();

        if keywords.is_empty() {
            // No meaningful query — return nothing rather than everything
            return Vec::new();
        }

        let mut scored: Vec<(f64, &ToolSpec)> = self
            .catalog
            .iter()
            .map(|spec| (Self::score_tool(spec, &keywords), spec))
            .filter(|(score, _)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(self.max_results);
        scored.into_iter().map(|(_, spec)| spec).collect()
    }
}

#[async_trait]
impl Tool for ToolSearchTool {
    fn name(&self) -> &str {
        "tool_search"
    }

    fn description(&self) -> &str {
        "Search for available tools by keyword. Use this when you need a tool that isn't in your current list. Returns tool definitions you can call on your next turn."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query describing what you want to do (e.g., 'manage speakers', 'control sprinklers', 'check tide level')"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if query.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Missing required parameter: query".into()),
            });
        }

        let results = self.search(query);

        if results.is_empty() {
            return Ok(ToolResult {
                success: true,
                output: format!("No tools found matching \"{query}\". Try different keywords."),
                error: None,
            });
        }

        // Format as tool definitions the model can use
        let tool_defs: Vec<serde_json::Value> = results
            .iter()
            .map(|spec| {
                json!({
                    "name": spec.name,
                    "description": spec.description,
                    "parameters": spec.parameters,
                })
            })
            .collect();

        let output = serde_json::to_string_pretty(&tool_defs)
            .unwrap_or_else(|_| format!("{tool_defs:?}"));

        Ok(ToolResult {
            success: true,
            output: format!(
                "Found {} tool(s) matching \"{query}\". You can now call these tools:\n\n{output}",
                results.len()
            ),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_catalog() -> Vec<ToolSpec> {
        vec![
            ToolSpec {
                name: "list_speakers".into(),
                description: "List all known speakers with their IDs and names".into(),
                parameters: json!({"type": "object", "properties": {}}),
            },
            ToolSpec {
                name: "rename_speaker".into(),
                description: "Rename a speaker by ID".into(),
                parameters: json!({"type": "object", "properties": {"id": {"type": "string"}, "name": {"type": "string"}}}),
            },
            ToolSpec {
                name: "delete_speaker".into(),
                description: "Delete a speaker profile by ID".into(),
                parameters: json!({"type": "object", "properties": {"id": {"type": "string"}}}),
            },
            ToolSpec {
                name: "get_tide_current".into(),
                description: "Get current tide level and pumping safety status".into(),
                parameters: json!({"type": "object", "properties": {}}),
            },
            ToolSpec {
                name: "get_sprinkler_status".into(),
                description: "Check if sprinklers are running and which zone".into(),
                parameters: json!({"type": "object", "properties": {}}),
            },
            ToolSpec {
                name: "shell".into(),
                description: "Execute a shell command in the workspace directory".into(),
                parameters: json!({"type": "object", "properties": {"command": {"type": "string"}}}),
            },
            ToolSpec {
                name: "memory_store".into(),
                description: "Save information to persistent memory".into(),
                parameters: json!({"type": "object", "properties": {"key": {"type": "string"}, "value": {"type": "string"}}}),
            },
        ]
    }

    #[test]
    fn search_returns_matching_tools() {
        let tool = ToolSearchTool::new(make_catalog(), 5);
        let results = tool.search("speaker");
        let names: Vec<&str> = results.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"list_speakers"));
        assert!(names.contains(&"rename_speaker"));
        assert!(names.contains(&"delete_speaker"));
        // Should not include unrelated tools
        assert!(!names.contains(&"get_tide_current"));
        assert!(!names.contains(&"shell"));
    }

    #[test]
    fn search_returns_empty_for_no_match() {
        let tool = ToolSearchTool::new(make_catalog(), 5);
        let results = tool.search("calendar appointments");
        assert!(results.is_empty());
    }

    #[test]
    fn name_matches_weighted_higher_than_description() {
        let tool = ToolSearchTool::new(make_catalog(), 5);
        let results = tool.search("tide");
        assert!(!results.is_empty());
        // get_tide_current should rank first (name match)
        assert_eq!(results[0].name, "get_tide_current");
    }

    #[test]
    fn max_results_respected() {
        let tool = ToolSearchTool::new(make_catalog(), 2);
        let results = tool.search("speaker");
        assert!(results.len() <= 2);
    }

    #[test]
    fn empty_query_returns_nothing() {
        let tool = ToolSearchTool::new(make_catalog(), 5);
        let results = tool.search("");
        assert!(results.is_empty());
    }

    #[test]
    fn single_char_keywords_filtered() {
        let tool = ToolSearchTool::new(make_catalog(), 5);
        // "a" is a single char, should be filtered out
        let results = tool.search("a");
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn execute_returns_tool_definitions() {
        let tool = ToolSearchTool::new(make_catalog(), 5);
        let result = tool
            .execute(json!({"query": "sprinkler"}))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("get_sprinkler_status"));
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn execute_missing_query_returns_error() {
        let tool = ToolSearchTool::new(make_catalog(), 5);
        let result = tool.execute(json!({})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn spec_has_correct_name_and_schema() {
        let tool = ToolSearchTool::new(vec![], 5);
        let spec = tool.spec();
        assert_eq!(spec.name, "tool_search");
        assert_eq!(spec.parameters["required"][0], "query");
    }
}
