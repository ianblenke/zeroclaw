# Proposal: Tool Search for Context-Efficient Tool Discovery

**Date:** 2026-03-06
**Status:** Proposed
**Scope:** `src/agent/`, `src/tools/`

## Problem

ZeroClaw injects all registered tool definitions into the system prompt on every request. As MCP servers are added (Home Assistant: 17 tools, tide-sprinkler: 7, speaker-id: 5, plus ~10 built-in), the tool count grows quickly. This causes two compounding problems:

1. **Context bloat** — Tool definitions consume a large portion of the context window before the model processes any actual user input. With ~39 tools today, this is already significant for smaller models (Qwen 3.5 9B).

2. **Tool selection degradation** — Model accuracy in picking the correct tool degrades significantly beyond 30–50 tools, especially for smaller models. The model sees sprinkler tools when the user asks about speakers, and vice versa.

## Proposed Solution: Tool Search Meta-Tool

Implement a tool search mechanism inspired by [Claude's tool_search_tool](https://docs.anthropic.com/en/docs/agents-and-tools/tool-use/tool-search-tool):

1. Instead of injecting all tool definitions into the system prompt, inject only a lightweight `tool_search` meta-tool plus a small set of always-loaded core tools.
2. When the model calls `tool_search(query="delete speakers")`, ZeroClaw searches the full tool catalog and returns the 3–5 most relevant tool definitions.
3. Those definitions are injected into the next turn's context so the model can call them.
4. Tools discovered in a conversation turn remain available for subsequent turns in the same session.

### Search Strategy

For local/small models, BM25 or simple keyword matching over tool names + descriptions would be sufficient. No embedding model needed:

```
tool_search("speaker management")
→ returns: list_speakers, rename_speaker, delete_speaker, merge_speakers, delete_unnamed_speakers
```

### Configuration

```toml
[agent]
# Tool search: "disabled" (all tools in prompt), "auto" (enable when >N tools), "enabled"
tool_search = "auto"
tool_search_threshold = 20  # enable when tool count exceeds this

# Tools always loaded (never deferred)
always_loaded_tools = ["shell", "file_read", "file_write", "memory_store", "memory_recall", "web_search_tool"]
```

### Architecture

This fits naturally into the existing agent loop:

- `src/agent/loop_.rs` — add tool search dispatch in the tool-call processing loop
- `src/tools/mod.rs` — add `ToolSearchTool` that searches the registry
- System prompt builder — conditionally defer tool definitions when tool search is active

The tool catalog (name, description, parameter schema) is already available from `tool.spec()` on each registered tool. The search just needs to match queries against this metadata.

### Behavior by Scenario

| Scenario | Behavior |
|----------|----------|
| Tool count < threshold | All tools in prompt (no change) |
| Tool count >= threshold | Only core tools + `tool_search` in prompt |
| Model calls `tool_search` | Return top 3–5 matching tools as definitions |
| Discovered tools in session | Remain available for subsequent turns |
| Model calls unknown tool | Return error suggesting `tool_search` |

## Motivation

Haven (home assistant deployment) currently runs 3 MCP servers + built-in tools = ~39 tools on a Qwen 3.5 9B model. Adding more integrations (calendar, music, shopping lists, etc.) would push this well past the accuracy threshold. Tool search would allow scaling to hundreds of tools without degrading the user experience.

## Risk

- **Low:** Additive feature, disabled by default (`"auto"` only activates above threshold).
- **Rollback:** Set `tool_search = "disabled"` in config to revert to current behavior.

## References

- [Claude tool_search_tool docs](https://docs.anthropic.com/en/docs/agents-and-tools/tool-use/tool-search-tool)
- [Anthropic: Advanced tool use](https://www.anthropic.com/engineering/advanced-tool-use)
- [Effective context engineering](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)
