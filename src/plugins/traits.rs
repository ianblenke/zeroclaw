//! Plugin trait and API surface.
//!
//! Mirrors OpenClaw's `OpenClawPluginDefinition` + `OpenClawPluginApi`:
//! - `Plugin` is the trait every plugin crate implements
//! - `PluginApi` is the handle passed into `register()` so plugins can
//!   register tools, hooks, and services without coupling to host internals

use crate::hooks::HookHandler;
use crate::tools::traits::Tool;
use serde::{Deserialize, Serialize};

use super::manifest::PluginManifest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginCapability {
    Hooks,
    Tools,
    Providers,
    /// Permission to modify tool results via the `tool_result_persist` hook.
    ModifyToolResults,
}

/// Context passed to a plugin during registration.
///
/// Analogous to OpenClaw's `OpenClawPluginApi`. Plugins call methods on this
/// to register their contributions (tools, hooks) with the host.
pub struct PluginApi {
    pub(crate) plugin_id: String,
    pub(crate) tools: Vec<Box<dyn Tool>>,
    pub(crate) hooks: Vec<Box<dyn HookHandler>>,
    pub(crate) config: serde_json::Value,
    pub(crate) logger: PluginLogger,
}

impl PluginApi {
    /// The plugin's own ID.
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    /// Register a tool that the agent can invoke.
    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Register a hook handler for lifecycle events.
    pub fn register_hook(&mut self, handler: Box<dyn HookHandler>) {
        self.hooks.push(handler);
    }

    /// Access the plugin-specific config table from `[plugins.entries.<id>.config]`.
    pub fn plugin_config(&self) -> &serde_json::Value {
        &self.config
    }

    /// Logger scoped to this plugin.
    pub fn logger(&self) -> &PluginLogger {
        &self.logger
    }
}

/// Simple logger interface for plugins (mirrors OpenClaw's `PluginLogger`).
#[derive(Clone)]
pub struct PluginLogger {
    prefix: String,
}

impl PluginLogger {
    pub(crate) fn new(plugin_id: &str) -> Self {
        Self {
            prefix: format!("[plugin:{plugin_id}]"),
        }
    }

    pub fn info(&self, msg: &str) {
        tracing::info!("{} {}", self.prefix, msg);
    }

    pub fn warn(&self, msg: &str) {
        tracing::warn!("{} {}", self.prefix, msg);
    }

    pub fn error(&self, msg: &str) {
        tracing::error!("{} {}", self.prefix, msg);
    }

    pub fn debug(&self, msg: &str) {
        tracing::debug!("{} {}", self.prefix, msg);
    }
}

/// Trait that every ZeroClaw plugin must implement.
///
/// Analogous to OpenClaw's `OpenClawPluginDefinition`. The host calls
/// `register()` once during startup, passing a `PluginApi` the plugin uses
/// to contribute tools, hooks, and services.
pub trait Plugin: Send + Sync {
    /// Manifest metadata (id, name, version, etc.).
    fn manifest(&self) -> &PluginManifest;

    /// Called once during plugin loading. Use `api` to register tools, hooks,
    /// and services. Returning `Err` marks the plugin as failed without
    /// crashing the host.
    fn register(&self, api: &mut PluginApi) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubPlugin {
        manifest: PluginManifest,
    }

    impl Plugin for StubPlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }
        fn register(&self, api: &mut PluginApi) -> anyhow::Result<()> {
            api.logger().info("registered");
            Ok(())
        }
    }

    /// REQ-PLUG-003-SC01
    #[test]
    fn plugin_capability_serde_roundtrip() {
        let caps = vec![
            PluginCapability::Hooks,
            PluginCapability::Tools,
            PluginCapability::Providers,
            PluginCapability::ModifyToolResults,
        ];
        let json = serde_json::to_string(&caps).unwrap();
        let parsed: Vec<PluginCapability> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0], PluginCapability::Hooks);
        assert_eq!(parsed[1], PluginCapability::Tools);
        assert_eq!(parsed[2], PluginCapability::Providers);
        assert_eq!(parsed[3], PluginCapability::ModifyToolResults);
    }

    /// REQ-PLUG-004-SC01
    #[test]
    fn plugin_logger_has_correct_prefix() {
        let logger = PluginLogger::new("my-plugin");
        assert!(logger.prefix.contains("[plugin:my-plugin]"));
    }

    /// REQ-PLUG-002-SC01
    #[test]
    fn plugin_api_accumulates_tools_and_hooks() {
        use crate::hooks::HookHandler;
        use crate::tools::traits::{Tool, ToolResult};

        struct TestTool;
        #[async_trait]
        impl Tool for TestTool {
            fn name(&self) -> &str {
                "test-tool"
            }
            fn description(&self) -> &str {
                "test"
            }
            fn parameters_schema(&self) -> serde_json::Value {
                serde_json::json!({})
            }
            async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
                Ok(ToolResult {
                    success: true,
                    output: String::new(),
                    error: None,
                })
            }
        }

        struct TestHookHandler;
        #[async_trait]
        impl HookHandler for TestHookHandler {
            fn name(&self) -> &str {
                "test-hook"
            }
        }

        let mut api = PluginApi {
            plugin_id: "test".into(),
            tools: Vec::new(),
            hooks: Vec::new(),
            config: serde_json::Value::Object(serde_json::Map::new()),
            logger: PluginLogger::new("test"),
        };

        assert!(api.tools.is_empty());
        assert!(api.hooks.is_empty());

        api.register_tool(Box::new(TestTool));
        api.register_hook(Box::new(TestHookHandler));

        assert_eq!(api.tools.len(), 1);
        assert_eq!(api.hooks.len(), 1);
        assert_eq!(api.plugin_id(), "test");
        assert!(api.plugin_config().is_object());
    }

    #[test]
    fn plugin_api_collects_nothing_by_default() {
        let plugin = StubPlugin {
            manifest: PluginManifest {
                id: "stub".into(),
                name: None,
                description: None,
                version: None,
                config_schema: None,
                capabilities: vec![],
                module_path: String::new(),
                wit_packages: vec![],
                tools: vec![],
                providers: vec![],
            },
        };
        let mut api = PluginApi {
            plugin_id: "stub".into(),
            tools: Vec::new(),
            hooks: Vec::new(),
            config: serde_json::Value::Object(serde_json::Map::new()),
            logger: PluginLogger::new("stub"),
        };
        plugin.register(&mut api).unwrap();
        assert!(api.tools.is_empty());
        assert!(api.hooks.is_empty());
    }
}
