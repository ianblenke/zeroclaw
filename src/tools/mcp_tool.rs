//! Wraps a discovered MCP tool as a zeroclaw [`Tool`] so it is dispatched
//! through the existing tool registry and agent loop without modification.

use std::sync::Arc;

use async_trait::async_trait;

use crate::tools::mcp_client::McpRegistry;
use crate::tools::mcp_protocol::McpToolDef;
use crate::tools::traits::{Tool, ToolResult};

/// A zeroclaw [`Tool`] backed by an MCP server tool.
///
/// The `prefixed_name` (e.g. `filesystem__read_file`) is what the agent loop
/// sees. The registry knows how to route it to the correct server.
pub struct McpToolWrapper {
    /// Prefixed name: `<server_name>__<tool_name>`.
    prefixed_name: String,
    /// Description extracted from the MCP tool definition. Stored as an owned
    /// String so that `description()` can return `&str` with self's lifetime.
    description: String,
    /// JSON schema for the tool's input parameters.
    input_schema: serde_json::Value,
    /// Shared registry — used to dispatch actual tool calls.
    registry: Arc<McpRegistry>,
}

impl McpToolWrapper {
    pub fn new(prefixed_name: String, def: McpToolDef, registry: Arc<McpRegistry>) -> Self {
        let description = def.description.unwrap_or_else(|| "MCP tool".to_string());
        Self {
            prefixed_name,
            description,
            input_schema: def.input_schema,
            registry,
        }
    }
}

#[async_trait]
impl Tool for McpToolWrapper {
    fn name(&self) -> &str {
        &self.prefixed_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> serde_json::Value {
        self.input_schema.clone()
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        match self.registry.call_tool(&self.prefixed_name, args).await {
            Ok(output) => Ok(ToolResult {
                success: true,
                output,
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal McpToolDef for testing.
    fn test_tool_def(desc: Option<&str>) -> McpToolDef {
        McpToolDef {
            name: "read_file".to_string(),
            description: desc.map(String::from),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }),
        }
    }

    /// Helper: construct an empty McpRegistry (no servers).
    async fn empty_registry() -> Arc<McpRegistry> {
        Arc::new(McpRegistry::connect_all(&[]).await.unwrap())
    }

    /// REQ-DELEG-013-SC01
    #[tokio::test]
    async fn mcp_tool_wrapper_name() {
        let registry = empty_registry().await;
        let def = test_tool_def(Some("Read a file"));
        let wrapper = McpToolWrapper::new(
            "filesystem__read_file".to_string(),
            def,
            registry,
        );
        assert_eq!(wrapper.name(), "filesystem__read_file");
    }

    /// REQ-DELEG-013-SC02
    #[tokio::test]
    async fn mcp_tool_wrapper_description() {
        let registry = empty_registry().await;
        let def = test_tool_def(Some("Read a file from disk"));
        let wrapper = McpToolWrapper::new("fs__read".to_string(), def, registry);
        assert_eq!(wrapper.description(), "Read a file from disk");
    }

    /// REQ-DELEG-013-SC02 (default description when None)
    #[tokio::test]
    async fn mcp_tool_wrapper_default_description() {
        let registry = empty_registry().await;
        let def = test_tool_def(None);
        let wrapper = McpToolWrapper::new("srv__tool".to_string(), def, registry);
        assert_eq!(wrapper.description(), "MCP tool");
    }

    /// REQ-DELEG-013-SC03
    #[tokio::test]
    async fn mcp_tool_wrapper_schema() {
        let registry = empty_registry().await;
        let def = test_tool_def(Some("test"));
        let wrapper = McpToolWrapper::new("x__y".to_string(), def, registry);
        let schema = wrapper.parameters_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["path"].is_object());
    }

    /// REQ-DELEG-013-SC01 (execute with unknown tool returns error)
    #[tokio::test]
    async fn mcp_tool_wrapper_execute_unknown_tool() {
        let registry = empty_registry().await;
        let def = test_tool_def(Some("test"));
        let wrapper = McpToolWrapper::new("srv__unknown".to_string(), def, registry);
        let result = wrapper.execute(serde_json::json!({})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("unknown MCP tool"));
    }
}
