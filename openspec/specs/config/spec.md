# Config Specification

## Purpose
Define the behavioral contract for ZeroClaw's configuration system: schema definition, loading, merging, validation, and config trait interfaces.

## Scope
- Files: `src/config/schema.rs` (250 tests), `src/config/mod.rs` (3 tests), `src/config/traits.rs` (2 tests)
- Total: 255 tests
- Risk tier: HIGH (config keys are public contract per CLAUDE.md)

## Requirements

---

### Config Traits (`src/config/traits.rs`)

### REQ-CFG-TRAIT-001: ChannelConfig MUST provide static name and description
Implementors of ChannelConfig MUST return static string references for name() and desc().

#### Scenario: Static name and desc
- WHEN ChannelConfig::name() and ChannelConfig::desc() are called
- THEN they return the configured static strings
- Tests:
  - `channel_config_returns_static_name_and_desc` in `src/config/traits.rs`

### REQ-CFG-TRAIT-002: ConfigHandle MUST provide instance-level name and description
Implementors of ConfigHandle MUST return static string references from &self methods.

#### Scenario: Instance name and desc
- WHEN handle.name() and handle.desc() are called on an instance
- THEN they return the configured static strings
- Tests:
  - `config_handle_returns_static_name_and_desc` in `src/config/traits.rs`

---

### Config Module (`src/config/mod.rs`)

### REQ-CFG-MOD-001: Config::default() MUST produce a valid constructible config

#### Scenario: Default config
- WHEN Config::default() is called
- THEN default_provider, default_model, and default_temperature are populated
- Tests:
  - `reexported_config_default_is_constructible` in `src/config/mod.rs`

### REQ-CFG-MOD-002: Channel configs MUST be constructible with required fields

#### Scenario: Telegram, Discord, Lark configs
- WHEN channel config structs are instantiated with required fields
- THEN they construct without error
- Tests:
  - `reexported_channel_configs_are_constructible` in `src/config/mod.rs`

### REQ-CFG-MOD-003: HttpRequestConfig MUST be constructible with defaults

#### Scenario: HttpRequestConfig default construction
- WHEN HttpRequestConfig is instantiated
- THEN it constructs without error and has expected default fields
- Tests:
  - `reexported_http_request_config_is_constructible` in `src/config/mod.rs`

---

### Config Schema (`src/config/schema.rs`) — 250 tests

---

### REQ-CFG-SCHEMA-001: Config defaults MUST be sensible and documented

All top-level Config fields and subsystem defaults MUST have reasonable, documented values when Config::default() is used.

#### Scenario: Top-level config defaults
- WHEN Config::default() is examined
- THEN temperature, provider, model, and core fields have documented defaults
- Tests:
  - `config_default_has_sane_values` in `src/config/schema.rs`

#### Scenario: HttpRequestConfig defaults
- WHEN HttpRequestConfig::default() is examined
- THEN credential header name, value prefix, and other fields have correct default values
- Tests:
  - `http_request_config_default_has_correct_values` in `src/config/schema.rs`

#### Scenario: Observability config defaults
- WHEN ObservabilityConfig::default() is examined
- THEN default observability settings are populated
- Tests:
  - `observability_config_default` in `src/config/schema.rs`

#### Scenario: Autonomy config defaults
- WHEN AutonomyConfig::default() is examined
- THEN auto_approve, always_ask, and workspace scope are populated correctly
- Tests:
  - `autonomy_config_default` in `src/config/schema.rs`
  - `checklist_autonomy_default_is_workspace_scoped` in `src/config/schema.rs`

#### Scenario: Runtime config defaults
- WHEN RuntimeConfig::default() is examined
- THEN runtime kind, docker image, wasm settings, and other fields have correct defaults
- Tests:
  - `runtime_config_default` in `src/config/schema.rs`

#### Scenario: Heartbeat config defaults
- WHEN HeartbeatConfig::default() is examined
- THEN interval and turn interval have correct default values
- Tests:
  - `heartbeat_config_default` in `src/config/schema.rs`

#### Scenario: Cron config defaults
- WHEN CronConfig is missing from TOML
- THEN defaults are applied
- Tests:
  - `cron_config_default` in `src/config/schema.rs`
  - `config_defaults_cron_when_section_missing` in `src/config/schema.rs`

#### Scenario: Memory config defaults
- WHEN MemoryConfig::default() is examined
- THEN hygiene settings (archive_after_days, purge_after_days, etc.) are correct
- Tests:
  - `memory_config_default_hygiene_settings` in `src/config/schema.rs`

#### Scenario: Storage provider config defaults
- WHEN StorageProviderConfig::default() is examined
- THEN schema, table, and storage settings have correct defaults
- Tests:
  - `storage_provider_config_defaults` in `src/config/schema.rs`

#### Scenario: Channels config defaults
- WHEN ChannelsConfig::default() is examined
- THEN no channels are pre-configured
- Tests:
  - `channels_config_default` in `src/config/schema.rs`
  - `channels_config_default_has_no_imessage_matrix` in `src/config/schema.rs`
  - `channels_config_default_has_no_whatsapp` in `src/config/schema.rs`
  - `channels_config_default_has_no_nextcloud_talk` in `src/config/schema.rs`

#### Scenario: Agent config defaults
- WHEN AgentConfig::default() is examined
- THEN max_tool_iterations, max_history_messages, session settings have correct defaults
- Tests:
  - `agent_config_defaults` in `src/config/schema.rs`

#### Scenario: Wasm config defaults and validation
- WHEN WasmConfig::default() is examined
- THEN memory_limit_mb, fuel_limit, and registry_url have correct defaults
- WHEN invalid wasm config values are provided
- THEN validation rejects them
- Tests:
  - `wasm_config_default_has_correct_values` in `src/config/schema.rs`
  - `wasm_config_invalid_values_rejected` in `src/config/schema.rs`

#### Scenario: Web search config defaults
- WHEN WebSearchConfig::default() is examined
- THEN provider, max_results, timeout, retries, and extended fields have correct defaults
- Tests:
  - `web_search_config_default_extended_fields` in `src/config/schema.rs`

#### Scenario: Coordination config defaults
- WHEN CoordinationConfig::default() is examined
- THEN enabled, lead_agent, and limits have correct defaults
- Tests:
  - `coordination_config_defaults` in `src/config/schema.rs`

#### Scenario: Cost enforcement defaults
- WHEN CostEnforcementConfig::default() is examined
- THEN enforcement mode, reserve percent, and limits are stable
- Tests:
  - `cost_enforcement_defaults_are_stable` in `src/config/schema.rs`

#### Scenario: Composio config defaults
- WHEN ComposioConfig::default() is examined
- THEN it is disabled by default
- Tests:
  - `composio_config_default_disabled` in `src/config/schema.rs`
  - `config_default_has_composio_and_secrets` in `src/config/schema.rs`

#### Scenario: Secrets config defaults
- WHEN SecretsConfig::default() is examined
- THEN encryption is enabled by default
- Tests:
  - `secrets_config_default_encrypts` in `src/config/schema.rs`

#### Scenario: Browser config defaults
- WHEN BrowserConfig::default() is examined
- THEN browser is disabled by default with correct sub-field defaults
- Tests:
  - `browser_config_default_disabled` in `src/config/schema.rs`

#### Scenario: Gateway config defaults
- WHEN GatewayConfig::default() is examined
- THEN port, host, rate limits, and idempotency settings have correct defaults
- Tests:
  - `gateway_config_default_values` in `src/config/schema.rs`

#### Scenario: Peripherals config defaults
- WHEN PeripheralsConfig::default() is examined
- THEN peripherals are disabled by default with correct board config defaults
- Tests:
  - `peripherals_config_default_disabled` in `src/config/schema.rs`
  - `peripheral_board_config_defaults` in `src/config/schema.rs`

#### Scenario: Transcription config defaults
- WHEN TranscriptionConfig::default() is examined
- THEN api_url, model, and max_duration_secs have correct defaults
- Tests:
  - `transcription_config_defaults` in `src/config/schema.rs`

#### Scenario: Signal config defaults
- WHEN SignalConfig is constructed without explicit values
- THEN defaults are applied correctly
- Tests:
  - `signal_config_defaults` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-002: Config MUST deserialize from TOML correctly

All config structs MUST correctly deserialize from TOML, including roundtrip serialization/deserialization and backward compatibility with older formats.

#### Scenario: Full config TOML roundtrip
- WHEN a complete TOML config string is deserialized and re-serialized
- THEN all fields survive the roundtrip correctly
- Tests:
  - `config_toml_roundtrip` in `src/config/schema.rs`

#### Scenario: Minimal TOML uses defaults
- WHEN a minimal TOML config is deserialized
- THEN missing fields are filled with defaults
- Tests:
  - `config_minimal_toml_uses_defaults` in `src/config/schema.rs`

#### Scenario: Config schema export shape
- WHEN config schema is exported
- THEN it contains expected contract shape
- Tests:
  - `config_schema_export_contains_expected_contract_shape` in `src/config/schema.rs`

#### Scenario: Autonomy config serde with non-CLI excluded tools
- WHEN autonomy config is deserialized
- THEN non_cli_excluded_tools defaults are applied correctly
- Tests:
  - `autonomy_config_serde_defaults_non_cli_excluded_tools` in `src/config/schema.rs`

#### Scenario: Heartbeat config delivery alias parsing
- WHEN heartbeat config with delivery aliases is deserialized
- THEN aliases are parsed correctly
- Tests:
  - `heartbeat_config_parses_delivery_aliases` in `src/config/schema.rs`

#### Scenario: Cron config serde roundtrip
- WHEN CronConfig is serialized and deserialized
- THEN all fields survive the roundtrip
- Tests:
  - `cron_config_serde_roundtrip` in `src/config/schema.rs`

#### Scenario: Storage provider db_url alias deserializes
- WHEN storage provider config uses db_url alias
- THEN it deserializes correctly
- Tests:
  - `storage_provider_dburl_alias_deserializes` in `src/config/schema.rs`

#### Scenario: Runtime reasoning enabled deserializes
- WHEN runtime config has reasoning_enabled field
- THEN it deserializes correctly
- Tests:
  - `runtime_reasoning_enabled_deserializes` in `src/config/schema.rs`

#### Scenario: Runtime WASM config deserializes
- WHEN runtime WASM config is provided in TOML
- THEN it deserializes correctly with all template variants
- Tests:
  - `runtime_wasm_deserializes` in `src/config/schema.rs`
  - `runtime_wasm_dev_template_deserializes` in `src/config/schema.rs`
  - `runtime_wasm_staging_template_deserializes` in `src/config/schema.rs`
  - `runtime_wasm_prod_template_deserializes` in `src/config/schema.rs`

#### Scenario: Model support vision deserializes
- WHEN model_support_vision is set in config
- THEN it deserializes correctly
- Tests:
  - `model_support_vision_deserializes` in `src/config/schema.rs`

#### Scenario: Provider reasoning level deserializes and aliases
- WHEN provider reasoning level is set (including runtime alias)
- THEN it deserializes correctly, with provider-level winning over runtime alias
- Tests:
  - `provider_reasoning_level_deserializes` in `src/config/schema.rs`
  - `runtime_reasoning_level_alias_deserializes` in `src/config/schema.rs`
  - `provider_reasoning_level_wins_over_runtime_alias` in `src/config/schema.rs`

#### Scenario: Agent config deserializes
- WHEN agent config TOML is provided
- THEN it deserializes correctly
- Tests:
  - `agent_config_deserializes` in `src/config/schema.rs`

#### Scenario: Progress mode deserializes variants
- WHEN progress mode variants (verbose, off, etc.) are provided
- THEN they deserialize correctly
- Tests:
  - `progress_mode_deserializes_variants` in `src/config/schema.rs`

#### Scenario: TOML supports model_provider and model alias fields
- WHEN TOML uses model_provider and model alias fields
- THEN they deserialize correctly
- Tests:
  - `toml_supports_model_provider_and_model_alias_fields` in `src/config/schema.rs`

#### Scenario: Config roundtrip with transcription
- WHEN config with transcription section is serialized/deserialized
- THEN all transcription fields survive the roundtrip
- Tests:
  - `config_roundtrip_with_transcription` in `src/config/schema.rs`
  - `config_without_transcription_uses_defaults` in `src/config/schema.rs`

#### Scenario: Config roundtrip with coordination section
- WHEN config with coordination section is serialized/deserialized
- THEN all coordination fields survive the roundtrip
- Tests:
  - `config_roundtrip_with_coordination_section` in `src/config/schema.rs`

#### Scenario: Cost enforcement config parses route-down mode
- WHEN cost enforcement config with route_down mode is deserialized
- THEN it parses correctly
- Tests:
  - `cost_enforcement_config_parses_route_down_mode` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-003: Channel configs MUST deserialize and roundtrip correctly

Each channel config MUST correctly deserialize from TOML, support roundtrip serialization, and handle backward compatibility.

#### Scenario: Telegram config serde and features
- WHEN Telegram config is deserialized/serialized
- THEN all fields survive roundtrip including stream mode, custom base URL, and progress mode
- Tests:
  - `telegram_config_serde` in `src/config/schema.rs`
  - `telegram_config_defaults_stream_off` in `src/config/schema.rs`
  - `telegram_config_custom_base_url` in `src/config/schema.rs`
  - `telegram_config_deserializes_progress_mode_verbose` in `src/config/schema.rs`
  - `telegram_config_deserializes_progress_mode_off` in `src/config/schema.rs`

#### Scenario: Telegram group reply config
- WHEN Telegram group reply config overrides legacy mention_only
- THEN the new group_reply config takes precedence
- Tests:
  - `telegram_group_reply_config_overrides_legacy_mention_only` in `src/config/schema.rs`

#### Scenario: Discord config serde and features
- WHEN Discord config is deserialized/serialized
- THEN all fields survive roundtrip including optional guild, allowed users, and backward compat
- Tests:
  - `discord_config_serde` in `src/config/schema.rs`
  - `discord_config_optional_guild` in `src/config/schema.rs`
  - `discord_config_deserializes_without_allowed_users` in `src/config/schema.rs`
  - `discord_config_deserializes_with_allowed_users` in `src/config/schema.rs`
  - `discord_config_toml_backward_compat` in `src/config/schema.rs`

#### Scenario: Discord group reply mode
- WHEN Discord group reply mode is configured
- THEN it falls back to legacy mention_only when not set, and overrides when set
- Tests:
  - `discord_group_reply_mode_falls_back_to_legacy_mention_only` in `src/config/schema.rs`
  - `discord_group_reply_mode_overrides_legacy_mention_only` in `src/config/schema.rs`

#### Scenario: iMessage config serde
- WHEN iMessage config is deserialized
- THEN it handles contacts, empty contacts, and wildcard correctly
- Tests:
  - `imessage_config_serde` in `src/config/schema.rs`
  - `imessage_config_empty_contacts` in `src/config/schema.rs`
  - `imessage_config_wildcard` in `src/config/schema.rs`

#### Scenario: Matrix config serde and backward compat
- WHEN Matrix config is deserialized/serialized
- THEN all fields survive roundtrip including backward compatibility without session hints
- Tests:
  - `matrix_config_serde` in `src/config/schema.rs`
  - `matrix_config_toml_roundtrip` in `src/config/schema.rs`
  - `matrix_config_backward_compatible_without_session_hints` in `src/config/schema.rs`

#### Scenario: Signal config serde
- WHEN Signal config is deserialized/serialized
- THEN all fields survive roundtrip with correct defaults
- Tests:
  - `signal_config_serde` in `src/config/schema.rs`
  - `signal_config_toml_roundtrip` in `src/config/schema.rs`

#### Scenario: Channels config with multiple channel types
- WHEN channels config includes iMessage and Matrix
- THEN both are correctly deserialized and accessible
- Tests:
  - `channels_config_with_imessage_and_matrix` in `src/config/schema.rs`

#### Scenario: Channels ack reaction config
- WHEN channels ack reaction config is provided
- THEN it roundtrips correctly and defaults to empty when missing
- Tests:
  - `channels_ack_reaction_config_roundtrip` in `src/config/schema.rs`
  - `channels_ack_reaction_defaults_empty` in `src/config/schema.rs`

#### Scenario: Slack config serde and features
- WHEN Slack config is deserialized
- THEN it handles allowed users, backward compat, and group reply sender overrides
- Tests:
  - `slack_config_deserializes_without_allowed_users` in `src/config/schema.rs`
  - `slack_config_deserializes_with_allowed_users` in `src/config/schema.rs`
  - `slack_config_toml_backward_compat` in `src/config/schema.rs`
  - `slack_group_reply_config_supports_sender_overrides` in `src/config/schema.rs`

#### Scenario: Mattermost group reply mode
- WHEN Mattermost group reply mode is configured
- THEN it falls back to legacy mention_only when not set, and overrides when set
- Tests:
  - `mattermost_group_reply_mode_falls_back_to_legacy_mention_only` in `src/config/schema.rs`
  - `mattermost_group_reply_mode_overrides_legacy_mention_only` in `src/config/schema.rs`

#### Scenario: Webhook config
- WHEN webhook config is provided with and without secret
- THEN it deserializes correctly
- Tests:
  - `webhook_config_with_secret` in `src/config/schema.rs`
  - `webhook_config_without_secret` in `src/config/schema.rs`

#### Scenario: WhatsApp config serde and features
- WHEN WhatsApp config is deserialized/serialized
- THEN all fields survive roundtrip including backend type detection and allowed numbers
- Tests:
  - `whatsapp_config_serde` in `src/config/schema.rs`
  - `whatsapp_config_toml_roundtrip` in `src/config/schema.rs`
  - `whatsapp_config_deserializes_without_allowed_numbers` in `src/config/schema.rs`
  - `whatsapp_config_wildcard_allowed` in `src/config/schema.rs`
  - `whatsapp_config_backend_type_cloud_precedence_when_ambiguous` in `src/config/schema.rs`
  - `whatsapp_config_backend_type_web` in `src/config/schema.rs`
  - `channels_config_with_whatsapp` in `src/config/schema.rs`

#### Scenario: Lark config serde and features
- WHEN Lark config is deserialized/serialized
- THEN all fields survive roundtrip including optional fields, endpoint defaults, and wildcard users
- Tests:
  - `lark_config_serde` in `src/config/schema.rs`
  - `lark_config_toml_roundtrip` in `src/config/schema.rs`
  - `lark_config_deserializes_without_optional_fields` in `src/config/schema.rs`
  - `lark_config_defaults_to_lark_endpoint` in `src/config/schema.rs`
  - `lark_config_with_wildcard_allowed_users` in `src/config/schema.rs`

#### Scenario: Lark group reply mode
- WHEN Lark group reply config overrides legacy mention_only
- THEN the new group_reply config takes precedence
- Tests:
  - `lark_group_reply_mode_overrides_legacy_mention_only` in `src/config/schema.rs`

#### Scenario: Feishu config serde and features
- WHEN Feishu config is deserialized/serialized
- THEN all fields survive roundtrip including optional fields and group reply mode
- Tests:
  - `feishu_config_serde` in `src/config/schema.rs`
  - `feishu_config_toml_roundtrip` in `src/config/schema.rs`
  - `feishu_config_deserializes_without_optional_fields` in `src/config/schema.rs`
  - `feishu_group_reply_mode_supports_mention_only` in `src/config/schema.rs`

#### Scenario: QQ config serde
- WHEN QQ config is deserialized
- THEN it defaults to webhook receive mode and roundtrips correctly
- Tests:
  - `qq_config_defaults_to_webhook_receive_mode` in `src/config/schema.rs`
  - `qq_config_toml_roundtrip_receive_mode` in `src/config/schema.rs`

#### Scenario: DingTalk config serde
- WHEN DingTalk config is deserialized
- THEN allowed_users defaults to empty and config roundtrips correctly
- Tests:
  - `dingtalk_config_defaults_allowed_users_to_empty` in `src/config/schema.rs`
  - `dingtalk_config_toml_roundtrip` in `src/config/schema.rs`
  - `channels_except_webhook_reports_dingtalk_as_enabled` in `src/config/schema.rs`

#### Scenario: Nextcloud Talk config serde
- WHEN Nextcloud Talk config is deserialized
- THEN it handles required fields and optional field defaults
- Tests:
  - `nextcloud_talk_config_serde` in `src/config/schema.rs`
  - `nextcloud_talk_config_defaults_optional_fields` in `src/config/schema.rs`

#### Scenario: OneBot/NapCat channel alias support
- WHEN channels config uses OneBot alias with ws_url
- THEN it deserializes correctly; NapCat still accepts ws_url alias
- Tests:
  - `channels_config_accepts_onebot_alias_with_ws_url` in `src/config/schema.rs`
  - `channels_config_napcat_still_accepts_ws_url_alias` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-004: Security config defaults MUST be deny-by-default

Security defaults MUST be restrictive. Gateway pairing, sandbox, and access control defaults MUST enforce least-privilege.

#### Scenario: Gateway default requires pairing
- WHEN GatewayConfig::default() is examined
- THEN pairing is required by default
- Tests:
  - `checklist_gateway_default_requires_pairing` in `src/config/schema.rs`

#### Scenario: Gateway default blocks public bind
- WHEN GatewayConfig::default() is examined
- THEN it does not bind to public interfaces
- Tests:
  - `checklist_gateway_default_blocks_public_bind` in `src/config/schema.rs`

#### Scenario: Gateway default has no tokens
- WHEN GatewayConfig::default() is examined
- THEN no tokens are pre-configured
- Tests:
  - `checklist_gateway_default_no_tokens` in `src/config/schema.rs`

#### Scenario: Gateway CLI default host is localhost
- WHEN gateway CLI defaults are examined
- THEN host defaults to localhost
- Tests:
  - `checklist_gateway_cli_default_host_is_localhost` in `src/config/schema.rs`

#### Scenario: Gateway serde roundtrip
- WHEN gateway config is serialized/deserialized
- THEN all security-relevant fields survive the roundtrip
- Tests:
  - `checklist_gateway_serde_roundtrip` in `src/config/schema.rs`

#### Scenario: Gateway backward compat without gateway section
- WHEN TOML config has no gateway section
- THEN defaults are applied (secure-by-default)
- Tests:
  - `checklist_gateway_backward_compat_no_gateway_section` in `src/config/schema.rs`

#### Scenario: Security defaults backward compatibility
- WHEN SecurityConfig::default() is examined
- THEN defaults match the backward-compatible baseline (sandbox, pairing, access control)
- Tests:
  - `security_defaults_are_backward_compatible` in `src/config/schema.rs`

#### Scenario: Security TOML parses OTP and E-Stop sections
- WHEN security config TOML includes OTP and E-Stop sections
- THEN they parse correctly with all sub-fields
- Tests:
  - `security_toml_parses_otp_and_estop_sections` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-005: Security validation MUST reject invalid configurations

Security config validation MUST reject malformed, blank, out-of-range, and logically invalid values.

#### Scenario: Invalid domain glob rejected
- WHEN a security config has an invalid domain glob pattern
- THEN validation rejects it
- Tests:
  - `security_validation_rejects_invalid_domain_glob` in `src/config/schema.rs`

#### Scenario: Invalid URL access CIDR rejected
- WHEN a security config has an invalid CIDR notation
- THEN validation rejects it
- Tests:
  - `security_validation_rejects_invalid_url_access_cidr` in `src/config/schema.rs`

#### Scenario: Blank URL access domain entries rejected
- WHEN a security config has blank domain entries in various URL access lists
- THEN validation rejects them
- Tests:
  - `security_validation_rejects_blank_url_access_domain` in `src/config/schema.rs`
  - `security_validation_rejects_blank_url_access_domain_allowlist_entry` in `src/config/schema.rs`
  - `security_validation_rejects_blank_url_access_domain_blocklist_entry` in `src/config/schema.rs`
  - `security_validation_rejects_blank_url_access_approved_domain_entry` in `src/config/schema.rs`

#### Scenario: URL access enforcement requires allowlist
- WHEN URL access enforcement is enabled without an allowlist
- THEN validation rejects it
- Tests:
  - `security_validation_requires_allowlist_when_enforcement_enabled` in `src/config/schema.rs`

#### Scenario: Invalid HTTP credential profile rejected
- WHEN a credential profile has an invalid env var name or empty header name
- THEN validation rejects it
- Tests:
  - `security_validation_rejects_invalid_http_credential_profile_env_var` in `src/config/schema.rs`
  - `security_validation_rejects_empty_http_credential_profile_header_name` in `src/config/schema.rs`

#### Scenario: Unknown domain category rejected
- WHEN a security config references an unknown domain category
- THEN validation rejects it
- Tests:
  - `security_validation_rejects_unknown_domain_category` in `src/config/schema.rs`

#### Scenario: OTP zero-value parameters rejected
- WHEN OTP config has zero token TTL, challenge timeout, or challenge attempts
- THEN validation rejects each
- Tests:
  - `security_validation_rejects_zero_token_ttl` in `src/config/schema.rs`
  - `security_validation_rejects_zero_challenge_timeout` in `src/config/schema.rs`
  - `security_validation_rejects_zero_challenge_attempts` in `src/config/schema.rs`

#### Scenario: RBAC invalid role config rejected
- WHEN a security config has an unknown role parent or duplicate role name
- THEN validation rejects it
- Tests:
  - `security_validation_rejects_unknown_role_parent` in `src/config/schema.rs`
  - `security_validation_rejects_duplicate_role_name` in `src/config/schema.rs`

#### Scenario: Syscall anomaly invalid config rejected
- WHEN syscall anomaly config has zero thresholds, invalid baseline names, zero alert budget, or zero cooldown
- THEN validation rejects each
- Tests:
  - `security_validation_rejects_zero_syscall_threshold` in `src/config/schema.rs`
  - `security_validation_rejects_invalid_syscall_baseline_name` in `src/config/schema.rs`
  - `security_validation_rejects_zero_syscall_alert_budget` in `src/config/schema.rs`
  - `security_validation_rejects_zero_syscall_cooldown` in `src/config/schema.rs`
  - `security_validation_rejects_denied_threshold_above_total_threshold` in `src/config/schema.rs`

#### Scenario: Prompt injection detection invalid config rejected
- WHEN perplexity threshold or symbol ratio threshold is out of valid range
- THEN validation rejects it
- Tests:
  - `security_validation_rejects_invalid_perplexity_threshold` in `src/config/schema.rs`
  - `security_validation_rejects_invalid_perplexity_symbol_ratio_threshold` in `src/config/schema.rs`

#### Scenario: Outbound leak guard invalid sensitivity rejected
- WHEN outbound leak guard sensitivity is out of valid range
- THEN validation rejects it
- Tests:
  - `security_validation_rejects_invalid_outbound_leak_guard_sensitivity` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-006: Proxy config MUST validate scope and service keys

Proxy configuration MUST validate scope, service entries, and proxy URLs.

#### Scenario: Proxy scope services requires entries when enabled
- WHEN proxy config has scope "services" but no service entries
- THEN validation rejects it
- Tests:
  - `proxy_config_scope_services_requires_entries_when_enabled` in `src/config/schema.rs`

#### Scenario: Runtime proxy client cache behavior
- WHEN runtime proxy config is set and clients are built
- THEN cache reuses default profile key and clearing config clears cache
- Tests:
  - `runtime_proxy_client_cache_reuses_default_profile_key` in `src/config/schema.rs`
  - `set_runtime_proxy_config_clears_runtime_proxy_client_cache` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-007: Model fallback resolution MUST work per provider

Each provider MUST have a default model fallback chain. resolve_default_model_id MUST prefer configured model, then use provider-specific fallback.

#### Scenario: Configured model takes precedence
- WHEN a model is explicitly configured
- THEN resolve_default_model_id returns the configured model
- Tests:
  - `resolve_default_model_id_prefers_configured_model` in `src/config/schema.rs`

#### Scenario: Provider-specific fallback used when no model configured
- WHEN no model is explicitly configured
- THEN resolve_default_model_id returns the provider-specific fallback
- Tests:
  - `resolve_default_model_id_uses_provider_specific_fallback` in `src/config/schema.rs`

#### Scenario: Special provider aliases handled
- WHEN provider has aliases (e.g. regional variants)
- THEN resolve_default_model_id handles them correctly
- Tests:
  - `resolve_default_model_id_handles_special_provider_aliases` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-008: Model provider profiles MUST map to correct endpoints and keys

Model provider profiles (named route configurations) MUST correctly resolve API endpoints, keys, and model overrides.

#### Scenario: Profile maps to custom endpoint
- WHEN a model provider profile is configured with a custom API URL
- THEN it maps to the correct endpoint
- Tests:
  - `model_provider_profile_maps_to_custom_endpoint` in `src/config/schema.rs`

#### Scenario: Profile uses OpenAI Codex endpoint
- WHEN a "responses" model provider profile is configured
- THEN it uses OpenAI Codex endpoint and OpenAI key
- Tests:
  - `model_provider_profile_responses_uses_openai_codex_and_openai_key` in `src/config/schema.rs`

#### Scenario: Profile uses profile-specific API key
- WHEN a profile has its own API key and global key is missing
- THEN it uses the profile API key
- Tests:
  - `model_provider_profile_uses_profile_api_key_when_global_is_missing` in `src/config/schema.rs`

#### Scenario: Profile can override default model for OpenRouter
- WHEN a profile overrides the default model and OpenRouter default is set
- THEN the profile model takes precedence
- Tests:
  - `model_provider_profile_can_override_default_model_when_openrouter_default_is_set` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-009: Config validation MUST reject invalid provider and model route configurations

Provider API, model routes, and transport settings MUST be validated.

#### Scenario: Provider API requires custom default provider
- WHEN provider_api is set without a custom default provider
- THEN validation rejects it
- Tests:
  - `provider_api_requires_custom_default_provider` in `src/config/schema.rs`

#### Scenario: Provider API invalid value rejected
- WHEN provider_api has an invalid value
- THEN validation rejects it
- Tests:
  - `provider_api_invalid_value_is_rejected` in `src/config/schema.rs`

#### Scenario: Model route max_tokens must be positive
- WHEN model route has max_tokens set to zero or negative
- THEN validation rejects it
- Tests:
  - `model_route_max_tokens_must_be_positive_when_set` in `src/config/schema.rs`

#### Scenario: Default model hint must match a model route
- WHEN default_model_hint is set but no matching model route exists
- THEN validation rejects it
- WHEN default_model_hint matches an existing model route
- THEN validation accepts it (including with whitespace)
- Tests:
  - `default_model_hint_requires_matching_model_route` in `src/config/schema.rs`
  - `default_model_hint_accepts_matching_model_route` in `src/config/schema.rs`
  - `default_model_hint_accepts_matching_model_route_with_whitespace` in `src/config/schema.rs`

#### Scenario: Provider transport normalization and validation
- WHEN provider transport is set with aliases
- THEN aliases are normalized correctly
- WHEN provider transport has an invalid value
- THEN validation rejects it
- Tests:
  - `provider_transport_normalizes_aliases` in `src/config/schema.rs`
  - `provider_transport_invalid_is_rejected` in `src/config/schema.rs`
  - `model_route_transport_invalid_is_rejected` in `src/config/schema.rs`

#### Scenario: Ollama cloud model validation
- WHEN Ollama model is configured without a remote API URL
- THEN validation rejects it
- WHEN Ollama model has remote endpoint and env key
- THEN validation accepts it
- Tests:
  - `validate_ollama_cloud_model_requires_remote_api_url` in `src/config/schema.rs`
  - `validate_ollama_cloud_model_accepts_remote_endpoint_and_env_key` in `src/config/schema.rs`

#### Scenario: Unknown model provider wire API rejected
- WHEN model provider wire API has an unknown value
- THEN validation rejects it
- Tests:
  - `validate_rejects_unknown_model_provider_wire_api` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-010: Config validation MUST reject invalid browser and web search configurations

Browser open mode, backend, auto backend priority, and web search provider/settings MUST be validated.

#### Scenario: Unknown browser open value rejected
- WHEN browser config has an unknown open value
- THEN validation rejects it
- Tests:
  - `config_validate_rejects_unknown_browser_open_value` in `src/config/schema.rs`

#### Scenario: Unknown browser backend value rejected
- WHEN browser config has an unknown backend value
- THEN validation rejects it
- Tests:
  - `config_validate_rejects_unknown_browser_backend_value` in `src/config/schema.rs`

#### Scenario: Invalid auto backend priority rejected
- WHEN browser config has an invalid auto_backend_priority value
- THEN validation rejects it
- Tests:
  - `config_validate_rejects_invalid_auto_backend_priority_value` in `src/config/schema.rs`

#### Scenario: Web search DDG alias accepted
- WHEN web search provider is set to "ddg" alias
- THEN validation accepts it
- Tests:
  - `config_validate_accepts_web_search_ddg_alias` in `src/config/schema.rs`

#### Scenario: Unknown web search provider/fallback rejected
- WHEN web search config has an unknown provider or fallback provider
- THEN validation rejects it
- Tests:
  - `config_validate_rejects_unknown_web_search_provider` in `src/config/schema.rs`
  - `config_validate_rejects_unknown_web_search_fallback_provider` in `src/config/schema.rs`

#### Scenario: Invalid web search Exa search type rejected
- WHEN web search config has an invalid exa_search_type
- THEN validation rejects it
- Tests:
  - `config_validate_rejects_invalid_web_search_exa_search_type` in `src/config/schema.rs`

#### Scenario: Web search out-of-range values rejected
- WHEN web search config has out-of-range numeric values
- THEN validation rejects them
- Tests:
  - `config_validate_rejects_web_search_out_of_range_values` in `src/config/schema.rs`

#### Scenario: Excessive web search retries rejected
- WHEN web search config has excessive retry count
- THEN validation rejects it
- Tests:
  - `config_validate_rejects_web_search_excessive_retries` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-011: Config validation MUST reject duplicate and invalid autonomy entries

Autonomy non_cli_excluded_tools MUST not contain duplicates.

#### Scenario: Duplicate non-CLI excluded tools rejected
- WHEN autonomy config has duplicate entries in non_cli_excluded_tools
- THEN validation rejects it
- Tests:
  - `config_validate_rejects_duplicate_non_cli_excluded_tools` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-012: Coordination validation MUST reject invalid limits and lead agent

Coordination config MUST validate limit values, lead agent, and disabled-state behavior.

#### Scenario: Invalid limits and lead agent rejected
- WHEN coordination config has invalid limits or lead agent
- THEN validation rejects it
- Tests:
  - `coordination_validation_rejects_invalid_limits_and_lead_agent` in `src/config/schema.rs`

#### Scenario: Empty lead agent allowed when disabled
- WHEN coordination is disabled and lead agent is empty
- THEN validation accepts it
- Tests:
  - `coordination_validation_allows_empty_lead_agent_when_disabled` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-013: Cost enforcement validation MUST reject invalid reserve and route-down configs

Cost enforcement config MUST validate reserve_percent and route_down model hint consistency with model routes.

#### Scenario: Reserve over 100 rejected
- WHEN cost enforcement reserve_percent exceeds 100
- THEN validation rejects it
- Tests:
  - `validation_rejects_cost_enforcement_reserve_over_100` in `src/config/schema.rs`

#### Scenario: Route-down hint requires matching route
- WHEN route_down hint does not match any model route
- THEN validation rejects it
- WHEN route_down hint matches a model route
- THEN validation accepts it
- Tests:
  - `validation_rejects_route_down_hint_without_matching_route` in `src/config/schema.rs`
  - `validation_accepts_route_down_hint_with_matching_route` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-014: Environment variable overrides MUST apply correctly

Config MUST support environment variable overrides for all major fields, with correct fallback priority and invalid-value rejection.

#### Scenario: API key overrides
- WHEN ZEROCLAW_API_KEY or fallback env vars are set
- THEN they override or supplement the config API key with correct priority
- Tests:
  - `env_override_api_key` in `src/config/schema.rs`
  - `env_override_api_key_fallback` in `src/config/schema.rs`
  - `env_override_api_key_generic_does_not_override_config` in `src/config/schema.rs`
  - `env_override_zeroclaw_api_key_overrides_config` in `src/config/schema.rs`

#### Scenario: Provider overrides
- WHEN provider env vars are set
- THEN they override the config provider with correct fallback behavior
- Tests:
  - `env_override_provider` in `src/config/schema.rs`
  - `env_override_model_provider_alias` in `src/config/schema.rs`
  - `env_override_provider_fallback` in `src/config/schema.rs`
  - `env_override_provider_fallback_does_not_replace_non_default_provider` in `src/config/schema.rs`
  - `env_override_zero_claw_provider_overrides_non_default_provider` in `src/config/schema.rs`

#### Scenario: Model overrides
- WHEN model env vars are set
- THEN they override the config model
- Tests:
  - `env_override_model` in `src/config/schema.rs`
  - `env_override_model_fallback` in `src/config/schema.rs`

#### Scenario: Gateway port and host overrides
- WHEN gateway port/host env vars are set
- THEN they override the config with correct fallback behavior; invalid ports are ignored
- Tests:
  - `env_override_gateway_port` in `src/config/schema.rs`
  - `env_override_port_fallback` in `src/config/schema.rs`
  - `env_override_gateway_host` in `src/config/schema.rs`
  - `env_override_host_fallback` in `src/config/schema.rs`
  - `env_override_invalid_port_ignored` in `src/config/schema.rs`

#### Scenario: Temperature overrides
- WHEN temperature env var is set
- THEN it overrides config temperature; out-of-range values are ignored
- Tests:
  - `env_override_temperature` in `src/config/schema.rs`
  - `env_override_temperature_out_of_range_ignored` in `src/config/schema.rs`

#### Scenario: Reasoning overrides
- WHEN reasoning env vars are set
- THEN they override config reasoning settings; invalid values are ignored
- Tests:
  - `env_override_reasoning_enabled` in `src/config/schema.rs`
  - `env_override_reasoning_invalid_value_ignored` in `src/config/schema.rs`
  - `env_override_reasoning_level_alias` in `src/config/schema.rs`
  - `env_override_reasoning_level_alias_invalid_ignored` in `src/config/schema.rs`

#### Scenario: Provider transport overrides
- WHEN provider transport env vars are set with aliases
- THEN they normalize correctly; invalid values do not override existing config
- Tests:
  - `env_override_provider_transport_normalizes_zeroclaw_alias` in `src/config/schema.rs`
  - `env_override_provider_transport_normalizes_legacy_alias` in `src/config/schema.rs`
  - `env_override_provider_transport_invalid_zeroclaw_does_not_override_existing` in `src/config/schema.rs`
  - `env_override_provider_transport_invalid_legacy_does_not_override_existing` in `src/config/schema.rs`

#### Scenario: Model support vision override
- WHEN model_support_vision env var is set
- THEN it overrides config model support vision
- Tests:
  - `env_override_model_support_vision` in `src/config/schema.rs`

#### Scenario: Web search config overrides
- WHEN web search env vars are set
- THEN they override config web search settings; invalid values are ignored
- Tests:
  - `env_override_web_search_config` in `src/config/schema.rs`
  - `env_override_web_search_invalid_values_ignored` in `src/config/schema.rs`

#### Scenario: URL access policy override
- WHEN URL access policy env var is set
- THEN it overrides config URL access policy
- Tests:
  - `env_override_url_access_policy` in `src/config/schema.rs`

#### Scenario: Storage provider config override
- WHEN storage provider env vars are set
- THEN they override config storage provider settings
- Tests:
  - `env_override_storage_provider_config` in `src/config/schema.rs`

#### Scenario: Proxy scope override
- WHEN proxy scope env vars are set
- THEN services scope and environment scope apply correctly
- Tests:
  - `env_override_proxy_scope_services` in `src/config/schema.rs`
  - `env_override_proxy_scope_environment_applies_process_env` in `src/config/schema.rs`

#### Scenario: Empty env values ignored
- WHEN env vars are set to empty strings
- THEN they are ignored
- Tests:
  - `env_override_empty_values_ignored` in `src/config/schema.rs`

#### Scenario: Workspace override
- WHEN workspace env var is set
- THEN it overrides config workspace
- Tests:
  - `env_override_workspace` in `src/config/schema.rs`

#### Scenario: Open skills enabled and dir overrides
- WHEN open_skills_enabled and open_skills_dir env vars are set
- THEN they override config; invalid boolean values keep existing config
- Tests:
  - `env_override_open_skills_enabled_and_dir` in `src/config/schema.rs`
  - `env_override_open_skills_enabled_invalid_value_keeps_existing_value` in `src/config/schema.rs`

#### Scenario: Regional API key overrides
- WHEN GLM_API_KEY or ZAI_API_KEY env vars are set for regional aliases
- THEN they apply correctly; ZEROCLAW_API_KEY takes priority
- Tests:
  - `env_override_glm_api_key_for_regional_aliases` in `src/config/schema.rs`
  - `env_override_zeroclaw_api_key_beats_glm_api_key_for_regional_aliases` in `src/config/schema.rs`
  - `env_override_zai_api_key_for_regional_aliases` in `src/config/schema.rs`
  - `env_override_zeroclaw_api_key_beats_zai_api_key_for_regional_aliases` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-015: Config MUST redact sensitive values in debug output

Debug output MUST NOT expose secrets, API keys, tokens, or server URL credentials.

#### Scenario: Config debug redacts sensitive values
- WHEN config is formatted for debug output
- THEN API keys, tokens, and other sensitive values are redacted
- Tests:
  - `config_debug_redacts_sensitive_values` in `src/config/schema.rs`

#### Scenario: BlueBubbles debug redacts server URL userinfo
- WHEN BlueBubbles config is formatted for debug output
- THEN server URL userinfo (credentials) is redacted
- Tests:
  - `bluebubbles_debug_redacts_server_url_userinfo` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-016: Config save/load MUST handle persistence correctly

Config save MUST handle file permissions, encryption, atomic writes, and roundtrip persistence.

#### Scenario: Config dir creation error message quality
- WHEN config directory creation fails
- THEN error message mentions openrc and path
- Tests:
  - `config_dir_creation_error_mentions_openrc_and_path` in `src/config/schema.rs`

#### Scenario: Save sets config permissions on new file
- WHEN config is saved to a new file
- THEN file permissions are restrictive (owner-only)
- Tests:
  - `save_sets_config_permissions_on_new_file` in `src/config/schema.rs`

#### Scenario: Config save and load roundtrip in tmpdir
- WHEN config is saved and loaded from a temp directory
- THEN all fields survive the roundtrip
- Tests:
  - `config_save_and_load_tmpdir` in `src/config/schema.rs`

#### Scenario: Config save encrypts nested credentials
- WHEN config is saved
- THEN nested credentials (channel tokens, API keys) are encrypted
- Tests:
  - `config_save_encrypts_nested_credentials` in `src/config/schema.rs`

#### Scenario: Config save atomic cleanup
- WHEN config save completes
- THEN temporary files are cleaned up atomically
- Tests:
  - `config_save_atomic_cleanup` in `src/config/schema.rs`

#### Scenario: Sync directory handles existing directory
- WHEN sync_directory is called on an existing directory
- THEN it succeeds without error
- Tests:
  - `sync_directory_handles_existing_directory` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-017: Config file permissions MUST enforce owner-only access

Config files MUST have restricted permissions. World-readable configs MUST be detectable and correctable.

#### Scenario: New config file has restricted permissions
- WHEN a new config file is created
- THEN it has restricted (owner-only) permissions
- Tests:
  - `new_config_file_has_restricted_permissions` in `src/config/schema.rs`

#### Scenario: Save restricts world-readable config to owner-only
- WHEN an existing world-readable config is saved
- THEN permissions are restricted to owner-only
- Tests:
  - `save_restricts_existing_world_readable_config_to_owner_only` in `src/config/schema.rs`

#### Scenario: World-readable config is detectable
- WHEN a config file has world-readable permissions
- THEN it is detectable
- Tests:
  - `world_readable_config_is_detectable` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-018: Composio and secrets config MUST support serde roundtrip and backward compat

Composio and secrets config sections MUST deserialize correctly, support backward compatibility, and handle partial TOML.

#### Scenario: Composio config serde roundtrip
- WHEN Composio config is serialized/deserialized
- THEN all fields survive the roundtrip
- Tests:
  - `composio_config_serde_roundtrip` in `src/config/schema.rs`

#### Scenario: Composio config backward compat with missing section
- WHEN TOML has no composio section
- THEN defaults are applied
- Tests:
  - `composio_config_backward_compat_missing_section` in `src/config/schema.rs`

#### Scenario: Composio config partial TOML
- WHEN composio TOML has only some fields
- THEN missing fields get defaults
- Tests:
  - `composio_config_partial_toml` in `src/config/schema.rs`

#### Scenario: Composio config enable alias supported
- WHEN composio config uses "enable" alias
- THEN it is accepted
- Tests:
  - `composio_config_enable_alias_supported` in `src/config/schema.rs`

#### Scenario: Secrets config serde roundtrip
- WHEN SecretsConfig is serialized/deserialized
- THEN all fields survive the roundtrip
- Tests:
  - `secrets_config_serde_roundtrip` in `src/config/schema.rs`

#### Scenario: Secrets config backward compat with missing section
- WHEN TOML has no secrets section
- THEN defaults are applied
- Tests:
  - `secrets_config_backward_compat_missing_section` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-019: Browser config MUST support serde roundtrip and backward compat

Browser config section MUST deserialize correctly and handle missing sections.

#### Scenario: Browser config serde roundtrip
- WHEN browser config is serialized/deserialized
- THEN all fields survive the roundtrip
- Tests:
  - `browser_config_serde_roundtrip` in `src/config/schema.rs`

#### Scenario: Browser config backward compat with missing section
- WHEN TOML has no browser section
- THEN defaults are applied
- Tests:
  - `browser_config_backward_compat_missing_section` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-020: Peripherals config MUST support TOML roundtrip

Peripherals config MUST deserialize and roundtrip correctly.

#### Scenario: Peripherals config TOML roundtrip
- WHEN peripherals config is serialized/deserialized
- THEN all fields survive the roundtrip
- Tests:
  - `peripherals_config_toml_roundtrip` in `src/config/schema.rs`

---

### REQ-CFG-SCHEMA-021: Workspace and config directory resolution MUST follow priority order

Config directory resolution MUST follow: env override > active workspace marker > default layout. load_or_init MUST handle workspace overrides, legacy layouts, and persisted markers.

#### Scenario: Env workspace override takes priority
- WHEN ZEROCLAW_WORKSPACE env var is set
- THEN it takes priority for config dir resolution
- Tests:
  - `resolve_runtime_config_dirs_uses_env_workspace_first` in `src/config/schema.rs`

#### Scenario: Env config dir override takes priority
- WHEN ZEROCLAW_CONFIG_DIR env var is set
- THEN it takes priority for config dir resolution
- Tests:
  - `resolve_runtime_config_dirs_uses_env_config_dir_first` in `src/config/schema.rs`

#### Scenario: Active workspace marker used
- WHEN an active workspace marker file exists
- THEN it is used for config dir resolution
- Tests:
  - `resolve_runtime_config_dirs_uses_active_workspace_marker` in `src/config/schema.rs`

#### Scenario: Falls back to default layout
- WHEN no env overrides or markers exist
- THEN the default layout is used
- Tests:
  - `resolve_runtime_config_dirs_falls_back_to_default_layout` in `src/config/schema.rs`

#### Scenario: load_or_init workspace override uses workspace root for config
- WHEN workspace override is provided to load_or_init
- THEN it uses the workspace root for config
- Tests:
  - `load_or_init_workspace_override_uses_workspace_root_for_config` in `src/config/schema.rs`

#### Scenario: load_or_init workspace suffix uses legacy config layout
- WHEN workspace suffix is provided
- THEN it uses legacy config layout
- Tests:
  - `load_or_init_workspace_suffix_uses_legacy_config_layout` in `src/config/schema.rs`

#### Scenario: load_or_init workspace override keeps existing legacy config
- WHEN workspace override is provided and legacy config exists
- THEN it keeps the existing legacy config
- Tests:
  - `load_or_init_workspace_override_keeps_existing_legacy_config` in `src/config/schema.rs`

#### Scenario: load_or_init uses persisted active workspace marker
- WHEN a persisted active workspace marker exists
- THEN load_or_init uses it
- Tests:
  - `load_or_init_uses_persisted_active_workspace_marker` in `src/config/schema.rs`

#### Scenario: Env workspace override takes priority over marker
- WHEN both env workspace override and marker exist
- THEN env override takes priority
- Tests:
  - `load_or_init_env_workspace_override_takes_priority_over_marker` in `src/config/schema.rs`

#### Scenario: Persist active workspace marker cleared for default config dir
- WHEN default config dir is used
- THEN active workspace marker is cleared
- Tests:
  - `persist_active_workspace_marker_is_cleared_for_default_config_dir` in `src/config/schema.rs`

---

## Mock Strategy
Config tests use direct struct construction and TOML parsing -- no external dependencies. All tests are pure unit tests. Environment variable override tests use a shared mutex lock (`env_override_lock`) and helper (`clear_proxy_env_test_vars`) to prevent test interference.

## Coverage Notes
- `src/config/schema.rs`: 250 tests (comprehensive coverage of schema, validation, serde, env overrides, file permissions, and workspace resolution)
- `src/config/mod.rs`: 3 tests (re-export verification for Config, channel configs, and HttpRequestConfig)
- `src/config/traits.rs`: 2 tests (covers both ChannelConfig and ConfigHandle trait contracts)
- Total: 255 tests across the config subsystem
- This module has thorough test coverage; the spec captures the full behavioral contract with every test explicitly named
