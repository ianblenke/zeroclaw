//! Claude Code CLI provider.
//!
//! Spawns the `claude` CLI (Claude Code) as a subprocess in print mode (`-p`),
//! which is non-interactive and handles its own OAuth authentication.
//! This allows users with Claude Code installed to use it as an LLM provider
//! without needing a separate API key.
//!
//! # Usage
//!
//! The `claude` binary must be available in `PATH`, or its location must be
//! set via the `CLAUDE_CODE_PATH` environment variable.
//!
//! Claude Code is invoked as:
//! ```text
//! claude -p "<prompt>" --output-format json [--model <model>]
//! ```
//!
//! Model IDs like `claude-code/opus` are mapped to CLI `--model` flags.
//! If the model is `"default"` or empty, the `--model` flag is omitted
//! and Claude Code's own default model is used.
//!
//! # Authentication
//!
//! Authentication is handled by the Claude Code CLI itself (OAuth / local
//! credential store at `~/.claude/credentials.json`). No explicit API key
//! is required by this provider.
//!
//! # Environment variables
//!
//! - `CLAUDE_CODE_PATH` — override the path to the `claude` binary (default: `"claude"`)
//! - `CLAUDE_CODE_SKIP_PERMISSIONS` — set to `"true"` to add `--dangerously-skip-permissions`

use crate::providers::traits::{ChatRequest, ChatResponse, Provider, TokenUsage};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

/// Environment variable for overriding the path to the `claude` binary.
pub const CLAUDE_CODE_PATH_ENV: &str = "CLAUDE_CODE_PATH";

/// Environment variable to enable `--dangerously-skip-permissions`.
pub const CLAUDE_CODE_SKIP_PERMISSIONS_ENV: &str = "CLAUDE_CODE_SKIP_PERMISSIONS";

/// Default `claude` binary name (resolved via `PATH`).
const DEFAULT_CLAUDE_BINARY: &str = "claude";

/// Model name used to signal "use Claude Code's own default model".
const DEFAULT_MODEL_MARKER: &str = "default";

/// Claude Code requests are bounded to avoid hung subprocesses.
const CLAUDE_CODE_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Avoid leaking oversized stderr payloads.
const MAX_STDERR_CHARS: usize = 512;

/// Environment variable names to strip from the subprocess to prevent
/// leaking API keys from other providers.
const SENSITIVE_ENV_EXACT: &[&str] = &[
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "GROQ_API_KEY",
    "DEEPSEEK_API_KEY",
    "MISTRAL_API_KEY",
    "TOGETHER_API_KEY",
    "FIREWORKS_API_KEY",
    "OPENROUTER_API_KEY",
    "PERPLEXITY_API_KEY",
    "COHERE_API_KEY",
    "AI21_API_KEY",
    "CEREBRAS_API_KEY",
    "SAMBANOVA_API_KEY",
    "HUGGINGFACE_API_KEY",
    "XAI_API_KEY",
    "REPLICATE_API_TOKEN",
    "BRAVE_API_KEY",
    "TAVILY_API_KEY",
    "ELEVENLABS_API_KEY",
];

/// Suffixes that indicate a secret — remove any env var ending with these
/// unless it starts with `CLAUDE_`.
const SENSITIVE_SUFFIXES: &[&str] = &["_SECRET", "_TOKEN", "_PASSWORD"];

/// JSON output from `claude -p --output-format json`.
///
/// The CLI may return the response text in different fields depending on
/// version: `result`, `content`, or `text`. We try all three.
#[derive(Debug, Deserialize)]
struct ClaudeJsonOutput {
    result: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    usage: Option<ClaudeUsage>,
}

/// Usage stats from Claude CLI JSON output.
#[derive(Debug, Deserialize, Default)]
struct ClaudeUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
}

/// Provider that invokes the Claude Code CLI as a subprocess.
///
/// Each inference request spawns a fresh `claude` process in print mode.
/// The CLI handles its own OAuth authentication, so no API key is needed.
pub struct ClaudeCodeProvider {
    /// Path to the `claude` binary.
    cli_path: PathBuf,
    /// Whether to add `--dangerously-skip-permissions` to CLI invocations.
    skip_permissions: bool,
}

impl ClaudeCodeProvider {
    /// Create a new `ClaudeCodeProvider`.
    ///
    /// The binary path is resolved from `CLAUDE_CODE_PATH` env var if set,
    /// otherwise defaults to `"claude"` (found via `PATH`).
    ///
    /// Skip-permissions is resolved from `CLAUDE_CODE_SKIP_PERMISSIONS` env var.
    pub fn new() -> Self {
        let cli_path = std::env::var(CLAUDE_CODE_PATH_ENV)
            .ok()
            .filter(|path| !path.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CLAUDE_BINARY));

        let skip_permissions = std::env::var(CLAUDE_CODE_SKIP_PERMISSIONS_ENV)
            .ok()
            .map(|v| v.trim().eq_ignore_ascii_case("true") || v.trim() == "1")
            .unwrap_or(false);

        Self {
            cli_path,
            skip_permissions,
        }
    }

    /// Parse a model ID like `"claude-code/opus-high"` into (model_flag, effort).
    ///
    /// Model IDs may encode effort level as a suffix: `opus-low`, `sonnet-medium`,
    /// `opus-high`. If no effort suffix is found, effort defaults to `None`.
    fn parse_model_and_effort(model: &str) -> (Option<String>, Option<String>) {
        let trimmed = model.trim();
        if trimmed.is_empty() || trimmed == DEFAULT_MODEL_MARKER {
            return (None, None);
        }
        let stripped = trimmed
            .strip_prefix("claude-code/")
            .unwrap_or(trimmed);

        // Check for effort suffix: -low, -medium, -high
        for effort in &["low", "medium", "high"] {
            let suffix = format!("-{effort}");
            if let Some(base) = stripped.strip_suffix(&suffix) {
                if !base.is_empty() {
                    return (Some(base.to_string()), Some((*effort).to_string()));
                }
            }
        }

        (Some(stripped.to_string()), None)
    }

    /// Apply security env filtering to a command.
    ///
    /// Instead of `env_clear()` (which breaks Node.js, NVM, SSL, proxies),
    /// we keep the full environment and only remove known sensitive API keys
    /// from other LLM providers.
    fn apply_env_filter(cmd: &mut Command) {
        for key in SENSITIVE_ENV_EXACT {
            cmd.env_remove(key);
        }
        for (key, _) in std::env::vars() {
            if key.starts_with("CLAUDE_") {
                continue;
            }
            let upper = key.to_uppercase();
            for suffix in SENSITIVE_SUFFIXES {
                if upper.ends_with(suffix) {
                    cmd.env_remove(&key);
                    break;
                }
            }
        }
    }

    fn redact_stderr(stderr: &[u8]) -> String {
        let text = String::from_utf8_lossy(stderr);
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        if trimmed.chars().count() <= MAX_STDERR_CHARS {
            return trimmed.to_string();
        }
        let clipped: String = trimmed.chars().take(MAX_STDERR_CHARS).collect();
        format!("{clipped}...")
    }

    /// Invoke the Claude Code CLI with the given prompt and optional model.
    /// Returns the trimmed response text.
    async fn invoke_claude(&self, message: &str, model: &str) -> anyhow::Result<String> {
        let (model_flag, effort) = Self::parse_model_and_effort(model);

        let mut cmd = Command::new(&self.cli_path);
        cmd.arg("-p")
            .arg(message)
            .arg("--output-format")
            .arg("json");

        if self.skip_permissions {
            cmd.arg("--dangerously-skip-permissions");
        }

        if let Some(ref model_value) = model_flag {
            cmd.arg("--model").arg(model_value);
        }

        if let Some(ref effort_value) = effort {
            cmd.arg("--effort").arg(effort_value);
        }

        Self::apply_env_filter(&mut cmd);

        cmd.kill_on_drop(true);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let child = cmd.spawn().map_err(|err| {
            anyhow::anyhow!(
                "Failed to spawn Claude Code CLI at {:?}: {err}. \
                 Install: npm install -g @anthropic-ai/claude-code && claude auth",
                self.cli_path
            )
        })?;

        let output = timeout(CLAUDE_CODE_REQUEST_TIMEOUT, child.wait_with_output())
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "Claude Code request timed out after {:?} (binary: {:?})",
                    CLAUDE_CODE_REQUEST_TIMEOUT,
                    self.cli_path
                )
            })?
            .map_err(|err| anyhow::anyhow!("Claude Code process failed: {err}"))?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            let stderr_excerpt = Self::redact_stderr(&output.stderr);
            let stdout_text = String::from_utf8_lossy(&output.stdout);
            let detail = if !stderr_excerpt.is_empty() {
                &stderr_excerpt
            } else {
                stdout_text.trim()
            };

            // Provide actionable error messages
            if detail.contains("not authenticated")
                || detail.contains("auth")
                || detail.contains("login")
                || detail.contains("credentials")
            {
                anyhow::bail!(
                    "Claude Code CLI is not authenticated. Run: claude auth\nDetail: {detail}"
                );
            } else if detail.contains("permission")
                || detail.contains("--dangerously-skip-permissions")
            {
                anyhow::bail!(
                    "Claude Code CLI requires permissions acceptance. \
                     Set CLAUDE_CODE_SKIP_PERMISSIONS=true or run: \
                     claude --dangerously-skip-permissions (once to accept)\nDetail: {detail}"
                );
            } else {
                anyhow::bail!("Claude Code CLI exited with code {code}: {detail}");
            }
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Try JSON parse first
        if let Ok(parsed) = serde_json::from_str::<ClaudeJsonOutput>(&stdout) {
            let text = parsed
                .result
                .or(parsed.content)
                .or(parsed.text)
                .unwrap_or_default();
            return Ok(text);
        }

        // Fallback: treat entire stdout as plain text
        Ok(stdout.trim().to_string())
    }

    /// Extract usage from JSON output if available.
    fn parse_usage(stdout: &str) -> TokenUsage {
        if let Ok(parsed) = serde_json::from_str::<ClaudeJsonOutput>(stdout) {
            if let Some(usage) = parsed.usage {
                return TokenUsage {
                    input_tokens: Some(usage.input_tokens),
                    output_tokens: Some(usage.output_tokens),
                };
            }
        }
        TokenUsage::default()
    }
}

impl Default for ClaudeCodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for ClaudeCodeProvider {
    async fn chat_with_system(
        &self,
        system_prompt: Option<&str>,
        message: &str,
        model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        // Prepend system prompt to user message — the CLI does not expose
        // a dedicated system-prompt flag.
        let full_message = match system_prompt {
            Some(system) if !system.is_empty() => {
                format!("{system}\n\n{message}")
            }
            _ => message.to_string(),
        };

        self.invoke_claude(&full_message, model).await
    }

    async fn chat(
        &self,
        request: ChatRequest<'_>,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<ChatResponse> {
        let text = self
            .chat_with_history(request.messages, model, temperature)
            .await?;

        Ok(ChatResponse {
            text: Some(text),
            tool_calls: Vec::new(),
            usage: Some(TokenUsage::default()),
            reasoning_content: None,
            quota_metadata: None,
            stop_reason: None,
            raw_stop_reason: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock poisoned")
    }

    #[test]
    fn new_uses_env_override() {
        let _guard = env_lock();
        let orig = std::env::var(CLAUDE_CODE_PATH_ENV).ok();
        std::env::set_var(CLAUDE_CODE_PATH_ENV, "/usr/local/bin/claude");
        let provider = ClaudeCodeProvider::new();
        assert_eq!(provider.cli_path, PathBuf::from("/usr/local/bin/claude"));
        match orig {
            Some(v) => std::env::set_var(CLAUDE_CODE_PATH_ENV, v),
            None => std::env::remove_var(CLAUDE_CODE_PATH_ENV),
        }
    }

    #[test]
    fn new_defaults_to_claude() {
        let _guard = env_lock();
        let orig = std::env::var(CLAUDE_CODE_PATH_ENV).ok();
        std::env::remove_var(CLAUDE_CODE_PATH_ENV);
        let provider = ClaudeCodeProvider::new();
        assert_eq!(provider.cli_path, PathBuf::from("claude"));
        if let Some(v) = orig {
            std::env::set_var(CLAUDE_CODE_PATH_ENV, v);
        }
    }

    #[test]
    fn new_ignores_blank_env_override() {
        let _guard = env_lock();
        let orig = std::env::var(CLAUDE_CODE_PATH_ENV).ok();
        std::env::set_var(CLAUDE_CODE_PATH_ENV, "   ");
        let provider = ClaudeCodeProvider::new();
        assert_eq!(provider.cli_path, PathBuf::from("claude"));
        match orig {
            Some(v) => std::env::set_var(CLAUDE_CODE_PATH_ENV, v),
            None => std::env::remove_var(CLAUDE_CODE_PATH_ENV),
        }
    }

    #[test]
    fn parse_model_strips_prefix() {
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort("claude-code/opus"),
            (Some("opus".to_string()), None)
        );
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort("claude-code/sonnet"),
            (Some("sonnet".to_string()), None)
        );
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort("claude-code/haiku"),
            (Some("haiku".to_string()), None)
        );
    }

    #[test]
    fn parse_model_extracts_effort() {
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort("claude-code/opus-low"),
            (Some("opus".to_string()), Some("low".to_string()))
        );
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort("claude-code/opus-medium"),
            (Some("opus".to_string()), Some("medium".to_string()))
        );
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort("claude-code/opus-high"),
            (Some("opus".to_string()), Some("high".to_string()))
        );
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort("claude-code/sonnet-low"),
            (Some("sonnet".to_string()), Some("low".to_string()))
        );
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort("claude-code/sonnet-high"),
            (Some("sonnet".to_string()), Some("high".to_string()))
        );
    }

    #[test]
    fn parse_model_passes_through_custom() {
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort("custom-model"),
            (Some("custom-model".to_string()), None)
        );
    }

    #[test]
    fn parse_model_returns_none_for_default() {
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort("default"),
            (None, None)
        );
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort(""),
            (None, None)
        );
        assert_eq!(
            ClaudeCodeProvider::parse_model_and_effort("   "),
            (None, None)
        );
    }

    #[test]
    fn skip_permissions_from_env() {
        let _guard = env_lock();
        let orig_path = std::env::var(CLAUDE_CODE_PATH_ENV).ok();
        let orig_skip = std::env::var(CLAUDE_CODE_SKIP_PERMISSIONS_ENV).ok();
        std::env::remove_var(CLAUDE_CODE_PATH_ENV);
        std::env::set_var(CLAUDE_CODE_SKIP_PERMISSIONS_ENV, "true");
        let provider = ClaudeCodeProvider::new();
        assert!(provider.skip_permissions);
        match orig_path {
            Some(v) => std::env::set_var(CLAUDE_CODE_PATH_ENV, v),
            None => std::env::remove_var(CLAUDE_CODE_PATH_ENV),
        }
        match orig_skip {
            Some(v) => std::env::set_var(CLAUDE_CODE_SKIP_PERMISSIONS_ENV, v),
            None => std::env::remove_var(CLAUDE_CODE_SKIP_PERMISSIONS_ENV),
        }
    }

    #[test]
    fn sensitive_env_list_coverage() {
        assert!(SENSITIVE_ENV_EXACT.contains(&"OPENAI_API_KEY"));
        assert!(SENSITIVE_ENV_EXACT.contains(&"ANTHROPIC_API_KEY"));
        assert!(SENSITIVE_ENV_EXACT.contains(&"GEMINI_API_KEY"));
        assert!(SENSITIVE_ENV_EXACT.contains(&"GROQ_API_KEY"));
        assert!(SENSITIVE_ENV_EXACT.contains(&"DEEPSEEK_API_KEY"));
    }

    #[test]
    fn redact_stderr_empty() {
        assert_eq!(ClaudeCodeProvider::redact_stderr(b""), "");
        assert_eq!(ClaudeCodeProvider::redact_stderr(b"  \n "), "");
    }

    #[test]
    fn redact_stderr_short() {
        assert_eq!(
            ClaudeCodeProvider::redact_stderr(b"some error"),
            "some error"
        );
    }

    #[tokio::test]
    async fn invoke_missing_binary_returns_error() {
        let provider = ClaudeCodeProvider {
            cli_path: PathBuf::from("/nonexistent/path/to/claude"),
            skip_permissions: false,
        };
        let result = provider.invoke_claude("hello", "opus").await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Failed to spawn Claude Code CLI"),
            "unexpected error message: {msg}"
        );
    }

    #[test]
    fn parse_usage_from_json() {
        let json = r#"{"result":"hi","usage":{"input_tokens":10,"output_tokens":5}}"#;
        let usage = ClaudeCodeProvider::parse_usage(json);
        assert_eq!(usage.input_tokens, Some(10));
        assert_eq!(usage.output_tokens, Some(5));
    }

    #[test]
    fn parse_usage_from_invalid_json() {
        let usage = ClaudeCodeProvider::parse_usage("not json");
        assert_eq!(usage.input_tokens, None);
        assert_eq!(usage.output_tokens, None);
    }
}
