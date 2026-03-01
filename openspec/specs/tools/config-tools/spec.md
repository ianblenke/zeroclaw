# Tools Configuration Specification

## Purpose

Define requirements for the runtime configuration tools: model routing, channel acknowledgement, web search, proxy, and web access configuration.

## Scope

- Files: `src/tools/model_routing_config.rs` (8 tests), `src/tools/channel_ack_config.rs` (5 tests), `src/tools/web_search_config.rs` (3 tests), `src/tools/proxy_config.rs` (4 tests), `src/tools/web_access_config.rs` (2 tests)
- Total tests: 22
- Risk tier: MEDIUM (config changes affect runtime behavior; some have security implications)

## Requirements

---

### Model Routing Configuration (`src/tools/model_routing_config.rs`)

### REQ-CFG-001: ModelRoutingConfigTool MUST manage default model settings

#### Scenario: Set default updates provider, model, and temperature
- WHEN set_default action is called with provider, model, and temperature
- THEN MUST persist the updated defaults to TOML and confirm changes
- Test: `set_default_updates_provider_model_and_temperature` in `src/tools/model_routing_config.rs`

### REQ-CFG-001A: ModelRoutingConfigTool MUST manage scenario-based routing

#### Scenario: Upsert creates route and classification rule
- WHEN upsert_scenario is called with a hint, provider, and model
- THEN MUST create both a route entry and a classification rule
- Test: `upsert_scenario_creates_route_and_rule` in `src/tools/model_routing_config.rs`

#### Scenario: Transport alias is canonicalized
- WHEN upsert_scenario is called with a transport alias (e.g. "sse")
- THEN MUST normalize it to the canonical transport name
- Test: `upsert_scenario_transport_alias_is_canonicalized` in `src/tools/model_routing_config.rs`

#### Scenario: Invalid transport is rejected
- WHEN upsert_scenario is called with an invalid transport value
- THEN MUST return an error
- Test: `upsert_scenario_rejects_invalid_transport` in `src/tools/model_routing_config.rs`

#### Scenario: Remove scenario also removes associated rule
- WHEN remove_scenario is called for an existing hint
- THEN MUST remove both the route and its classification rule
- Test: `remove_scenario_also_removes_rule` in `src/tools/model_routing_config.rs`

### REQ-CFG-001B: ModelRoutingConfigTool MUST manage delegate agent profiles

#### Scenario: Upsert and remove delegate agent
- WHEN upsert_agent and remove_agent actions are called
- THEN MUST create and then remove the agent profile from config
- Test: `upsert_and_remove_delegate_agent` in `src/tools/model_routing_config.rs`

### REQ-CFG-001C: ModelRoutingConfigTool MUST enforce security

#### Scenario: Read-only mode blocks mutations
- WHEN read-only mode is active and a mutating action is attempted
- THEN MUST return an error blocking the operation
- Test: `read_only_mode_blocks_mutating_actions` in `src/tools/model_routing_config.rs`

### REQ-CFG-001D: ModelRoutingConfigTool MUST report env-backed credentials

#### Scenario: Get reports environment-backed credentials
- WHEN get action is called and env vars are set for provider credentials
- THEN MUST include env-backed credential status in the report
- Test: `get_reports_env_backed_credentials_for_routes_and_agents` in `src/tools/model_routing_config.rs`

---

### Channel Acknowledgement Configuration (`src/tools/channel_ack_config.rs`)

### REQ-CFG-002: ChannelAckConfigTool MUST manage per-channel acknowledgement settings

#### Scenario: Set and get channel policy
- WHEN set action configures ack policy for a channel and get retrieves it
- THEN MUST return the configured policy with correct fields
- Test: `set_and_get_channel_policy` in `src/tools/channel_ack_config.rs`

#### Scenario: Add and remove rule roundtrip
- WHEN add_rule and remove_rule actions are called
- THEN MUST add and then remove the rule from the channel config
- Test: `add_and_remove_rule_roundtrip` in `src/tools/channel_ack_config.rs`

### REQ-CFG-002A: ChannelAckConfigTool MUST enforce security

#### Scenario: Read-only mode blocks mutation
- WHEN read-only mode is active and a mutating action is attempted
- THEN MUST return an error
- Test: `readonly_mode_blocks_mutation` in `src/tools/channel_ack_config.rs`

### REQ-CFG-002B: ChannelAckConfigTool MUST support simulation

#### Scenario: Simulate reports rule selection
- WHEN simulate action is called with message context
- THEN MUST report which rule would be selected and the resulting emoji
- Test: `simulate_reports_rule_selection` in `src/tools/channel_ack_config.rs`

#### Scenario: Simulate runs reports aggregate counts
- WHEN simulate action is called with multiple runs
- THEN MUST report aggregate emoji selection counts
- Test: `simulate_runs_reports_aggregate_counts` in `src/tools/channel_ack_config.rs`

---

### Web Search Configuration (`src/tools/web_search_config.rs`)

### REQ-CFG-003: WebSearchConfigTool MUST manage web search settings

#### Scenario: List providers includes extended providers
- WHEN list_providers action is called
- THEN MUST return all supported search providers
- Test: `list_providers_includes_extended_providers` in `src/tools/web_search_config.rs`

#### Scenario: Set normalizes provider and deduplicates fallbacks
- WHEN set action is called with provider aliases and duplicate fallbacks
- THEN MUST normalize names and remove duplicates
- Test: `set_normalizes_provider_and_deduplicates_fallbacks` in `src/tools/web_search_config.rs`

#### Scenario: Unknown provider is rejected
- WHEN set action is called with an unknown provider name
- THEN MUST return an error
- Test: `set_rejects_unknown_provider` in `src/tools/web_search_config.rs`

---

### Proxy Configuration (`src/tools/proxy_config.rs`)

### REQ-CFG-004: ProxyConfigTool MUST manage proxy settings

#### Scenario: List services returns known keys
- WHEN list_services action is called
- THEN MUST return all known service endpoint keys
- Test: `list_services_action_returns_known_keys` in `src/tools/proxy_config.rs`

#### Scenario: Services scope requires services entries
- WHEN set action with scope=services is called without service entries
- THEN MUST return an error
- Test: `set_scope_services_requires_services_entries` in `src/tools/proxy_config.rs`

#### Scenario: Set and get roundtrip
- WHEN set action configures proxy and get retrieves it
- THEN MUST return the configured proxy settings
- Test: `set_and_get_round_trip_proxy_scope` in `src/tools/proxy_config.rs`

#### Scenario: Null proxy URL clears existing value
- WHEN set action is called with null proxy_url
- THEN MUST clear the existing proxy URL
- Test: `set_null_proxy_url_clears_existing_value` in `src/tools/proxy_config.rs`

---

### Web Access Configuration (`src/tools/web_access_config.rs`)

### REQ-CFG-005: WebAccessConfigTool MUST manage URL access policies

#### Scenario: Check URL reports first-visit approval requirement
- WHEN check_url action is called for an unknown URL with first-visit approval enabled
- THEN MUST report that approval is required
- Test: `check_url_reports_first_visit_approval_requirement` in `src/tools/web_access_config.rs`

#### Scenario: Set supports add and remove domain lists
- WHEN set action is called with add_allowed and remove_blocked domain lists
- THEN MUST merge additions and remove specified domains
- Test: `set_supports_add_and_remove_domain_lists` in `src/tools/web_access_config.rs`

## Mock Strategy

- Config files: `tempfile::TempDir` + TOML fixtures
- SecurityPolicy: Default construction with test-safe settings (read-only variants for enforcement tests)
- Runtime Config: Constructed with test defaults

## Coverage Notes
- `src/tools/model_routing_config.rs`: 8 tests (defaults, scenarios, agents, security, env creds)
- `src/tools/channel_ack_config.rs`: 5 tests (set/get, rules, security, simulation)
- `src/tools/web_search_config.rs`: 3 tests (list providers, set/normalize, reject unknown)
- `src/tools/proxy_config.rs`: 4 tests (list services, scope validation, roundtrip, clear)
- `src/tools/web_access_config.rs`: 2 tests (check URL, domain lists)
- Total: 22 tests across 5 files
