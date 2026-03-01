# Tools Web Specification

## Purpose

Define requirements for the web-facing tool implementations: browser automation, web search, HTTP request, and web fetch.

## Scope

- Files: `src/tools/browser.rs` (3057 LOC, 33 tests), `src/tools/web_search_tool.rs` (1292 LOC, 20 tests), `src/tools/http_request.rs` (1035 LOC, 57 tests), `src/tools/web_fetch.rs` (869 LOC, 31 tests)
- Risk tier: HIGH (outbound HTTP, SSRF risk, credential handling, URL policy enforcement)
- Existing tests: 141 total (33 browser + 20 web_search + 57 http_request + 31 web_fetch)

## Requirements

### REQ-WEB-001: Browser Tool Identity

`BrowserTool` MUST implement the `Tool` trait with name `"browser"`, non-empty description, and a valid JSON parameter schema.

#### Scenario: Name returns browser

- WHEN `name()` is called
- THEN MUST return `"browser"`
- Test: `browser_tool_name` in `src/tools/browser.rs`

### REQ-WEB-002: Browser URL Validation

`BrowserTool` MUST validate URLs before access, enforcing domain allowlists and blocking SSRF vectors.

#### Scenario: Domain normalization

- WHEN URL domains are normalized for matching
- THEN MUST strip scheme, path, and normalize case
- Test: `normalize_domains_works` in `src/tools/browser.rs`

#### Scenario: Host extraction from URLs

- WHEN a URL is parsed for host extraction
- THEN MUST correctly extract the host component
- Tests:
  - `extract_host_works` in `src/tools/browser.rs`
  - `extract_host_handles_ipv6` in `src/tools/browser.rs`

#### Scenario: Allowlist exact match

- WHEN a host is checked against an exact allowlist entry
- THEN MUST match the exact domain
- Test: `host_matches_allowlist_exact` in `src/tools/browser.rs`

#### Scenario: Allowlist wildcard match

- WHEN a host is checked against a wildcard allowlist entry
- THEN MUST match subdomain patterns using wildcard rules
- Test: `host_matches_allowlist_wildcard` in `src/tools/browser.rs`

#### Scenario: Allowlist star-all match

- WHEN a host is checked against a `*` allowlist entry
- THEN MUST match any public domain
- Test: `host_matches_allowlist_star` in `src/tools/browser.rs`

#### Scenario: URL validation with allowed domains

- WHEN a URL is validated against configured allowed domains
- THEN MUST accept allowed URLs and reject disallowed ones
- Test: `browser_tool_validates_url` in `src/tools/browser.rs`

#### Scenario: Empty allowlist blocks all URLs

- WHEN the allowlist is empty
- THEN MUST block all URLs
- Test: `browser_tool_empty_allowlist_blocks` in `src/tools/browser.rs`

#### Scenario: Private host detection for localhost and private IPs

- WHEN a URL targets localhost, private IPv4, or link-local addresses
- THEN MUST detect and block them as private hosts
- Tests:
  - `is_private_host_detects_local` in `src/tools/browser.rs`
  - `is_private_host_blocks_multicast_and_reserved` in `src/tools/browser.rs`

#### Scenario: IPv6 SSRF blocking

- WHEN an IPv6 address is used in a URL
- THEN MUST block private/loopback IPv6 addresses and IPv4-mapped IPv6
- Tests:
  - `is_private_host_catches_ipv6` in `src/tools/browser.rs`
  - `is_private_host_catches_mapped_ipv4` in `src/tools/browser.rs`
  - `is_private_host_catches_ipv6_private_ranges` in `src/tools/browser.rs`
  - `validate_url_blocks_ipv6_ssrf` in `src/tools/browser.rs`

### REQ-WEB-003: Browser Backend Configuration

`BrowserTool` MUST support multiple backends (agent_browser, rust_native, computer_use, auto) with validated configuration.

#### Scenario: Backend parser accepts supported values

- WHEN a supported backend kind string is provided (agent_browser, rust_native, computer_use, auto)
- THEN MUST parse to the correct backend variant
- Test: `browser_backend_parser_accepts_supported_values` in `src/tools/browser.rs`

#### Scenario: Backend parser rejects unknown values

- WHEN an unsupported backend kind string is provided
- THEN MUST return an error
- Test: `browser_backend_parser_rejects_unknown_values` in `src/tools/browser.rs`

#### Scenario: Default backend is agent_browser

- WHEN no backend is explicitly configured
- THEN MUST default to agent_browser
- Test: `browser_tool_default_backend_is_agent_browser` in `src/tools/browser.rs`

#### Scenario: Auto backend configuration accepted

- WHEN auto backend is configured
- THEN MUST accept and store the configuration correctly
- Test: `browser_tool_accepts_auto_backend_config` in `src/tools/browser.rs`

#### Scenario: Computer-use backend configuration accepted

- WHEN computer-use backend is configured with endpoint
- THEN MUST accept and store the configuration correctly
- Test: `browser_tool_accepts_computer_use_backend_config` in `src/tools/browser.rs`

#### Scenario: Auto backend priority defaults

- WHEN auto backend priority is not explicitly set
- THEN MUST use default priority ordering
- Test: `auto_backend_priority_defaults_when_unset` in `src/tools/browser.rs`

#### Scenario: Auto backend priority custom order and deduplication

- WHEN a custom auto backend priority is configured with duplicates
- THEN MUST accept the order and deduplicate entries
- Test: `auto_backend_priority_accepts_custom_order_and_dedupes` in `src/tools/browser.rs`

#### Scenario: Auto backend priority rejects invalid entries

- WHEN auto backend priority contains invalid backend names
- THEN MUST reject with an error
- Test: `auto_backend_priority_rejects_invalid_entries` in `src/tools/browser.rs`

#### Scenario: Computer-use endpoint rejects public HTTP

- WHEN a computer-use endpoint URL uses HTTP for a public host
- THEN MUST reject by default (require HTTPS for non-local endpoints)
- Test: `computer_use_endpoint_rejects_public_http_by_default` in `src/tools/browser.rs`

#### Scenario: Computer-use endpoint requires HTTPS for public remote

- WHEN a computer-use endpoint URL targets a public remote host
- THEN MUST require HTTPS
- Test: `computer_use_endpoint_requires_https_for_public_remote` in `src/tools/browser.rs`

### REQ-WEB-004: Browser Session Management

`BrowserTool` MUST manage browser sessions with coordinate validation, screenshot paths, and error recovery.

#### Scenario: Coordinate validation applies limits

- WHEN click/fill coordinates exceed viewport bounds
- THEN MUST reject with validation error
- Test: `computer_use_coordinate_validation_applies_limits` in `src/tools/browser.rs`

#### Scenario: Screenshot path validation blocks escaped paths

- WHEN a screenshot path contains path traversal sequences
- THEN MUST reject the path
- Test: `screenshot_path_validation_blocks_escaped_paths` in `src/tools/browser.rs`

#### Scenario: Computer-use key actions validate params

- WHEN computer-use key action parameters are provided
- THEN MUST validate required and optional parameters
- Test: `computer_use_key_actions_validate_params` in `src/tools/browser.rs`

#### Scenario: Computer-use-only action detection

- WHEN an action string is checked for computer-use exclusivity
- THEN MUST correctly identify actions that only work with computer-use backend
- Test: `computer_use_only_action_detection_is_correct` in `src/tools/browser.rs`

#### Scenario: Unavailable action error preserves backend context

- WHEN an action is unavailable for the resolved backend
- THEN MUST return an error message that includes backend context
- Test: `unavailable_action_error_preserves_backend_context` in `src/tools/browser.rs`

#### Scenario: Recoverable error detection matches session patterns

- WHEN a rust-native backend error occurs
- THEN MUST correctly identify recoverable session errors for retry
- Test: `recoverable_error_detection_matches_session_patterns` in `src/tools/browser.rs`

#### Scenario: Non-recoverable error detection rejects policy errors

- WHEN a policy or validation error occurs
- THEN MUST NOT classify it as recoverable
- Test: `non_recoverable_error_detection_rejects_policy_errors` in `src/tools/browser.rs`

#### Scenario: Reset session is idempotent without active client

- WHEN `reset_session` is called without an active browser client
- THEN MUST complete without error (idempotent)
- Test: `reset_session_is_idempotent_without_client` in `src/tools/browser.rs`

### REQ-WEB-005: Web Search Tool Identity

`WebSearchTool` MUST implement the `Tool` trait with name `"web_search_tool"`, non-empty description, and a valid JSON parameter schema.

#### Scenario: Name returns web_search_tool

- WHEN `name()` is called
- THEN MUST return `"web_search_tool"`
- Test: `test_tool_name` in `src/tools/web_search_tool.rs`

#### Scenario: Description is non-empty

- WHEN `description()` is called
- THEN MUST return a non-empty description string
- Test: `test_tool_description` in `src/tools/web_search_tool.rs`

#### Scenario: Parameters schema includes query

- WHEN `parameters_schema()` is called
- THEN MUST return a valid JSON schema with a query parameter
- Test: `test_parameters_schema` in `src/tools/web_search_tool.rs`

### REQ-WEB-006: Web Search Provider Chain

`WebSearchTool` MUST construct a fallback provider chain from configuration with deduplication.

#### Scenario: Provider chain uses primary plus fallbacks with deduplication

- WHEN providers are configured with a primary and fallback list
- THEN MUST build an ordered chain with deduplication
- Test: `provider_chain_uses_primary_plus_fallbacks_and_dedupes` in `src/tools/web_search_tool.rs`

#### Scenario: Provider chain rejects unknown provider

- WHEN an unknown provider name is given
- THEN MUST return an error
- Test: `provider_chain_rejects_unknown_provider` in `src/tools/web_search_tool.rs`

### REQ-WEB-007: Web Search API Key Management

`WebSearchTool` MUST support comma-separated API keys with round-robin rotation per provider.

#### Scenario: API key parsing from comma-separated string

- WHEN multiple comma-separated keys are configured
- THEN MUST parse into individual keys
- Test: `test_parses_multiple_api_keys` in `src/tools/web_search_tool.rs`

#### Scenario: Round-robin API key rotation

- WHEN multiple API keys are available
- THEN MUST rotate through them in round-robin order
- Test: `test_round_robin_api_key_selection_cycles` in `src/tools/web_search_tool.rs`

### REQ-WEB-008: Web Search HTML Parsing

`WebSearchTool` MUST parse search results from HTML/JSON responses per provider format.

#### Scenario: HTML tag stripping

- WHEN HTML content is processed
- THEN MUST strip HTML tags from content
- Test: `test_strip_tags` in `src/tools/web_search_tool.rs`

#### Scenario: DuckDuckGo empty results parsing

- WHEN DuckDuckGo returns no results
- THEN MUST handle empty response gracefully
- Test: `test_parse_duckduckgo_results_empty` in `src/tools/web_search_tool.rs`

#### Scenario: DuckDuckGo results with data

- WHEN DuckDuckGo returns HTML with search results
- THEN MUST extract results correctly
- Test: `test_parse_duckduckgo_results_with_data` in `src/tools/web_search_tool.rs`

#### Scenario: DuckDuckGo redirect URL decoding

- WHEN DuckDuckGo results contain redirect URLs
- THEN MUST decode redirect URLs to final destinations
- Test: `test_parse_duckduckgo_results_decodes_redirect_url` in `src/tools/web_search_tool.rs`

#### Scenario: DuckDuckGo 403 status hint suggests provider switch

- WHEN DuckDuckGo returns HTTP 403
- THEN MUST provide a hint suggesting provider switch
- Test: `duckduckgo_status_hint_for_403_mentions_provider_switch` in `src/tools/web_search_tool.rs`

#### Scenario: DuckDuckGo 500 status hint is empty

- WHEN DuckDuckGo returns HTTP 500
- THEN MUST return an empty status hint
- Test: `duckduckgo_status_hint_for_500_is_empty` in `src/tools/web_search_tool.rs`

### REQ-WEB-009: Web Search Read-Only Mode

`WebSearchTool` MUST respect read-only mode enforcement.

#### Scenario: Read-only mode blocks search execution

- WHEN read-only mode is active
- THEN MUST reject search requests
- Test: `test_execute_blocked_in_read_only_mode` in `src/tools/web_search_tool.rs`

### REQ-WEB-010: HTTP Request Tool Identity

`HttpRequestTool` MUST implement the `Tool` trait with name `"http_request"`, non-empty description, and valid JSON parameter schema supporting all HTTP methods.

#### Scenario: URL domain normalization

- WHEN allowed domains are normalized
- THEN MUST strip scheme, path, and normalize case
- Tests:
  - `normalize_domain_strips_scheme_path_and_case` in `src/tools/http_request.rs`
  - `normalize_allowed_domains_deduplicates` in `src/tools/http_request.rs`

### REQ-WEB-011: HTTP Request URL Validation

`HttpRequestTool` MUST validate URLs with SSRF protection blocking private IPs, localhost, and link-local addresses.

#### Scenario: Accepts exact allowed domain

- WHEN a URL exactly matches an allowlist domain
- THEN MUST accept the request
- Test: `validate_accepts_exact_domain` in `src/tools/http_request.rs`

#### Scenario: Accepts HTTP scheme

- WHEN a URL uses the HTTP scheme for an allowed domain
- THEN MUST accept the request
- Test: `validate_accepts_http` in `src/tools/http_request.rs`

#### Scenario: Accepts subdomain of allowed domain

- WHEN a URL is a subdomain of an allowed domain
- THEN MUST accept the request
- Test: `validate_accepts_subdomain` in `src/tools/http_request.rs`

#### Scenario: Wildcard allowlist accepts public host

- WHEN a wildcard (`*`) allowlist is configured and a public host is requested
- THEN MUST accept the request
- Test: `validate_accepts_wildcard_allowlist_for_public_host` in `src/tools/http_request.rs`

#### Scenario: Wildcard allowlist still rejects private host

- WHEN a wildcard (`*`) allowlist is configured and a private host is requested
- THEN MUST reject the request (SSRF protection overrides wildcard)
- Test: `validate_wildcard_allowlist_still_rejects_private_host` in `src/tools/http_request.rs`

#### Scenario: Wildcard subdomain pattern matching

- WHEN a wildcard subdomain pattern (e.g. `*.example.com`) is in the allowlist
- THEN MUST accept matching subdomains
- Test: `validate_accepts_wildcard_subdomain_pattern` in `src/tools/http_request.rs`

#### Scenario: Rejects allowlist miss

- WHEN a URL does not match any allowlist entry
- THEN MUST reject the request
- Test: `validate_rejects_allowlist_miss` in `src/tools/http_request.rs`

#### Scenario: Rejects localhost

- WHEN a URL targets localhost
- THEN MUST reject the request
- Test: `validate_rejects_localhost` in `src/tools/http_request.rs`

#### Scenario: Rejects private IPv4

- WHEN a URL targets a private IPv4 address
- THEN MUST reject the request
- Test: `validate_rejects_private_ipv4` in `src/tools/http_request.rs`

#### Scenario: Rejects whitespace in URL

- WHEN a URL contains whitespace
- THEN MUST reject the request
- Test: `validate_rejects_whitespace` in `src/tools/http_request.rs`

#### Scenario: Rejects userinfo in URL

- WHEN a URL contains userinfo (e.g. `user:pass@host`)
- THEN MUST reject the request
- Test: `validate_rejects_userinfo` in `src/tools/http_request.rs`

#### Scenario: Requires allowlist

- WHEN no allowlist is configured
- THEN MUST reject all requests
- Test: `validate_requires_allowlist` in `src/tools/http_request.rs`

#### Scenario: Blocks multicast IPv4

- WHEN a URL targets a multicast IPv4 address
- THEN MUST reject the request
- Test: `blocks_multicast_ipv4` in `src/tools/http_request.rs`

#### Scenario: Blocks broadcast address

- WHEN a URL targets the broadcast address
- THEN MUST reject the request
- Test: `blocks_broadcast` in `src/tools/http_request.rs`

#### Scenario: Blocks reserved IPv4

- WHEN a URL targets a reserved IPv4 range
- THEN MUST reject the request
- Test: `blocks_reserved_ipv4` in `src/tools/http_request.rs`

#### Scenario: Blocks documentation ranges

- WHEN a URL targets IPv4 documentation ranges (192.0.2.x, 198.51.100.x, 203.0.113.x)
- THEN MUST reject the request
- Test: `blocks_documentation_ranges` in `src/tools/http_request.rs`

#### Scenario: Blocks benchmarking range

- WHEN a URL targets the benchmarking range (198.18.x.x)
- THEN MUST reject the request
- Test: `blocks_benchmarking_range` in `src/tools/http_request.rs`

#### Scenario: Blocks IPv6 localhost

- WHEN a URL targets IPv6 localhost (::1)
- THEN MUST reject the request
- Test: `blocks_ipv6_localhost` in `src/tools/http_request.rs`

#### Scenario: Blocks IPv6 multicast

- WHEN a URL targets an IPv6 multicast address
- THEN MUST reject the request
- Test: `blocks_ipv6_multicast` in `src/tools/http_request.rs`

#### Scenario: Blocks IPv6 link-local

- WHEN a URL targets an IPv6 link-local address (fe80::)
- THEN MUST reject the request
- Test: `blocks_ipv6_link_local` in `src/tools/http_request.rs`

#### Scenario: Blocks IPv6 unique local

- WHEN a URL targets an IPv6 unique-local address (fd00::)
- THEN MUST reject the request
- Test: `blocks_ipv6_unique_local` in `src/tools/http_request.rs`

#### Scenario: Blocks IPv4-mapped IPv6

- WHEN a URL targets an IPv4-mapped IPv6 address
- THEN MUST reject the request
- Test: `blocks_ipv4_mapped_ipv6` in `src/tools/http_request.rs`

#### Scenario: Allows public IPv4

- WHEN a URL targets a public IPv4 address on an allowed domain
- THEN MUST accept the request
- Test: `allows_public_ipv4` in `src/tools/http_request.rs`

#### Scenario: Blocks IPv6 documentation range

- WHEN a URL targets an IPv6 documentation range (2001:db8::)
- THEN MUST reject the request
- Test: `blocks_ipv6_documentation_range` in `src/tools/http_request.rs`

#### Scenario: Allows public IPv6

- WHEN a URL targets a public IPv6 address on an allowed domain
- THEN MUST accept the request
- Test: `allows_public_ipv6` in `src/tools/http_request.rs`

#### Scenario: Blocks shared address space

- WHEN a URL targets the shared address space (100.64.x.x)
- THEN MUST reject the request
- Test: `blocks_shared_address_space` in `src/tools/http_request.rs`

#### Scenario: SSRF octal loopback not parsed as IP

- WHEN an octal notation loopback address (e.g. 0177.0.0.1) is used
- THEN MUST NOT parse it as a valid IP and MUST reject via URL validation
- Test: `ssrf_octal_loopback_not_parsed_as_ip` in `src/tools/http_request.rs`

#### Scenario: SSRF hex loopback not parsed as IP

- WHEN a hex notation loopback address (e.g. 0x7f.0.0.1) is used
- THEN MUST NOT parse it as a valid IP and MUST reject via URL validation
- Test: `ssrf_hex_loopback_not_parsed_as_ip` in `src/tools/http_request.rs`

#### Scenario: SSRF decimal loopback not parsed as IP

- WHEN a decimal notation loopback address (e.g. 2130706433) is used
- THEN MUST NOT parse it as a valid IP and MUST reject via URL validation
- Test: `ssrf_decimal_loopback_not_parsed_as_ip` in `src/tools/http_request.rs`

#### Scenario: SSRF zero-padded loopback not parsed as IP

- WHEN a zero-padded loopback address (e.g. 127.000.000.001) is used
- THEN MUST NOT parse it as a valid IP and MUST reject via URL validation
- Test: `ssrf_zero_padded_loopback_not_parsed_as_ip` in `src/tools/http_request.rs`

#### Scenario: SSRF alternate notations rejected by validate_url

- WHEN alternate IP notations targeting private ranges are used
- THEN MUST be rejected by URL validation
- Test: `ssrf_alternate_notations_rejected_by_validate_url` in `src/tools/http_request.rs`

#### Scenario: SSRF blocks loopback 127.x range

- WHEN a URL targets any address in the 127.0.0.0/8 range
- THEN MUST reject the request
- Test: `ssrf_blocks_loopback_127_range` in `src/tools/http_request.rs`

#### Scenario: SSRF blocks RFC1918 10.x range

- WHEN a URL targets a 10.0.0.0/8 address
- THEN MUST reject the request
- Test: `ssrf_blocks_rfc1918_10_range` in `src/tools/http_request.rs`

#### Scenario: SSRF blocks RFC1918 172.x range

- WHEN a URL targets a 172.16.0.0/12 address
- THEN MUST reject the request
- Test: `ssrf_blocks_rfc1918_172_range` in `src/tools/http_request.rs`

#### Scenario: SSRF blocks unspecified address

- WHEN a URL targets the unspecified address (0.0.0.0)
- THEN MUST reject the request
- Test: `ssrf_blocks_unspecified_address` in `src/tools/http_request.rs`

#### Scenario: SSRF blocks .localhost subdomain

- WHEN a URL targets a .localhost subdomain
- THEN MUST reject the request
- Test: `ssrf_blocks_dot_localhost_subdomain` in `src/tools/http_request.rs`

#### Scenario: SSRF blocks .local TLD

- WHEN a URL targets a .local TLD
- THEN MUST reject the request
- Test: `ssrf_blocks_dot_local_tld` in `src/tools/http_request.rs`

#### Scenario: SSRF blocks IPv6 unspecified

- WHEN a URL targets the IPv6 unspecified address (::)
- THEN MUST reject the request
- Test: `ssrf_ipv6_unspecified` in `src/tools/http_request.rs`

#### Scenario: Rejects FTP scheme

- WHEN a URL uses the FTP scheme
- THEN MUST reject the request
- Test: `validate_rejects_ftp_scheme` in `src/tools/http_request.rs`

#### Scenario: Rejects empty URL

- WHEN an empty URL string is provided
- THEN MUST reject the request
- Test: `validate_rejects_empty_url` in `src/tools/http_request.rs`

#### Scenario: Rejects IPv6 host directly

- WHEN a raw IPv6 host is used in the URL
- THEN MUST reject the request
- Test: `validate_rejects_ipv6_host` in `src/tools/http_request.rs`

### REQ-WEB-012: HTTP Request Method Validation

`HttpRequestTool` MUST validate HTTP methods against the supported set (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS).

#### Scenario: Valid methods accepted

- WHEN a supported HTTP method is provided
- THEN MUST accept and parse it
- Test: `validate_accepts_valid_methods` in `src/tools/http_request.rs`

#### Scenario: Invalid method rejected

- WHEN an unsupported HTTP method is provided
- THEN MUST reject with an error
- Test: `validate_rejects_invalid_method` in `src/tools/http_request.rs`

### REQ-WEB-013: HTTP Request Header Redaction

`HttpRequestTool` MUST redact sensitive headers (Authorization, API-Key, Token, Secret) in display output.

#### Scenario: Sensitive header masking in display

- WHEN response headers contain sensitive names (Authorization, API-Key, etc.)
- THEN MUST mask values in display output
- Test: `redact_headers_for_display_redacts_sensitive` in `src/tools/http_request.rs`

#### Scenario: Redaction does not alter original headers

- WHEN headers are redacted for display
- THEN MUST NOT modify the original header data
- Test: `redact_headers_does_not_alter_original` in `src/tools/http_request.rs`

#### Scenario: Header parsing preserves original values

- WHEN headers are parsed from JSON input
- THEN MUST preserve original key-value pairs
- Test: `parse_headers_preserves_original_values` in `src/tools/http_request.rs`

### REQ-WEB-014: HTTP Request Credential Profiles

`HttpRequestTool` MUST support credential profile resolution and substitution into requests.

#### Scenario: Credential profile injects env-backed header

- WHEN a credential profile specifies an env-backed header
- THEN MUST resolve the environment variable and inject the header
- Test: `resolve_credential_profile_injects_env_backed_header` in `src/tools/http_request.rs`

#### Scenario: Missing env var fails credential resolution

- WHEN a credential profile references a missing environment variable
- THEN MUST return an error
- Test: `resolve_credential_profile_missing_env_var_fails` in `src/tools/http_request.rs`

#### Scenario: Header name conflict is case-insensitive

- WHEN checking for header name conflicts between user-provided and credential headers
- THEN MUST detect conflicts case-insensitively
- Test: `has_header_name_conflict_is_case_insensitive` in `src/tools/http_request.rs`

#### Scenario: Sensitive values scrubbed from output

- WHEN credential-injected values appear in response output
- THEN MUST redact the sensitive values
- Test: `redact_sensitive_values_scrubs_injected_secrets` in `src/tools/http_request.rs`

### REQ-WEB-015: HTTP Request Security Modes

`HttpRequestTool` MUST respect read-only mode, rate limiting, and redirect policy.

#### Scenario: Read-only mode blocks execution

- WHEN read-only mode is active
- THEN MUST reject requests
- Test: `execute_blocks_readonly_mode` in `src/tools/http_request.rs`

#### Scenario: Rate limiting blocks execution

- WHEN rate limit is exceeded
- THEN MUST reject requests
- Test: `execute_blocks_when_rate_limited` in `src/tools/http_request.rs`

#### Scenario: Redirect policy is none (no auto-follow)

- WHEN the HTTP client is built
- THEN MUST configure redirect policy to none (no automatic following)
- Test: `redirect_policy_is_none` in `src/tools/http_request.rs`

### REQ-WEB-016: Web Fetch Tool Identity

`WebFetchTool` MUST implement the `Tool` trait with name `"web_fetch"`, non-empty description, and a valid JSON parameter schema.

#### Scenario: Name returns web_fetch

- WHEN `name()` is called
- THEN MUST return `"web_fetch"`
- Test: `name_is_web_fetch` in `src/tools/web_fetch.rs`

#### Scenario: Parameters schema requires url

- WHEN `parameters_schema()` is called
- THEN MUST return a JSON schema with a required url parameter
- Test: `parameters_schema_requires_url` in `src/tools/web_fetch.rs`

### REQ-WEB-017: Web Fetch URL Validation

`WebFetchTool` MUST validate URLs against allowlist/blocklist with SSRF protection.

#### Scenario: Accepts exact allowed domain

- WHEN a URL exactly matches an allowlist domain
- THEN MUST accept the request
- Test: `validate_accepts_exact_domain` in `src/tools/web_fetch.rs`

#### Scenario: Accepts subdomain of allowed domain

- WHEN a URL is a subdomain of an allowed domain
- THEN MUST accept the request
- Test: `validate_accepts_subdomain` in `src/tools/web_fetch.rs`

#### Scenario: Accepts wildcard allowlist

- WHEN a wildcard allowlist is configured
- THEN MUST accept public domains
- Test: `validate_accepts_wildcard` in `src/tools/web_fetch.rs`

#### Scenario: Rejects empty URL

- WHEN an empty URL string is provided
- THEN MUST reject the request
- Test: `validate_rejects_empty_url` in `src/tools/web_fetch.rs`

#### Scenario: Rejects missing URL parameter

- WHEN the URL parameter is missing from arguments
- THEN MUST reject the request
- Test: `validate_rejects_missing_url` in `src/tools/web_fetch.rs`

#### Scenario: FTP scheme rejection

- WHEN a non-HTTP scheme (e.g. ftp://) is used
- THEN MUST reject the request
- Test: `validate_rejects_ftp_scheme` in `src/tools/web_fetch.rs`

#### Scenario: Rejects allowlist miss

- WHEN a URL does not match any allowlist entry
- THEN MUST reject the request
- Test: `validate_rejects_allowlist_miss` in `src/tools/web_fetch.rs`

#### Scenario: Requires allowlist

- WHEN no allowlist is configured
- THEN MUST reject all requests
- Test: `validate_requires_allowlist` in `src/tools/web_fetch.rs`

#### Scenario: SSRF blocks localhost

- WHEN a URL targets localhost
- THEN MUST reject the request
- Test: `ssrf_blocks_localhost` in `src/tools/web_fetch.rs`

#### Scenario: SSRF blocks private IPv4

- WHEN a URL targets a private IPv4 address
- THEN MUST reject the request
- Test: `ssrf_blocks_private_ipv4` in `src/tools/web_fetch.rs`

#### Scenario: SSRF blocks loopback

- WHEN a URL targets any loopback address
- THEN MUST reject the request
- Test: `ssrf_blocks_loopback` in `src/tools/web_fetch.rs`

#### Scenario: SSRF blocks RFC1918 ranges

- WHEN a URL targets RFC1918 private address ranges
- THEN MUST reject the request
- Test: `ssrf_blocks_rfc1918` in `src/tools/web_fetch.rs`

#### Scenario: Wildcard allowlist still blocks private hosts

- WHEN a wildcard allowlist is configured but URL targets a private host
- THEN MUST reject the request (SSRF protection overrides wildcard)
- Test: `ssrf_wildcard_still_blocks_private` in `src/tools/web_fetch.rs`

#### Scenario: Domain normalization

- WHEN domains are normalized for matching
- THEN MUST strip scheme and normalize case
- Test: `normalize_domain_strips_scheme_and_case` in `src/tools/web_fetch.rs`

#### Scenario: Domain deduplication

- WHEN duplicate domains are in the allowlist
- THEN MUST deduplicate them
- Test: `normalize_deduplicates` in `src/tools/web_fetch.rs`

### REQ-WEB-018: Web Fetch HTML Conversion

`WebFetchTool` MUST convert HTML to clean text/markdown with noise element stripping.

#### Scenario: HTML tag removal

- WHEN HTML content is converted to text
- THEN MUST remove HTML tags
- Test: `html_conversion_removes_tags` in `src/tools/web_fetch.rs`

#### Scenario: Noise element stripping

- WHEN HTML contains nav, script, style, footer, sidebar elements
- THEN MUST strip noise elements before conversion
- Test: `strip_noise_removes_nav_scripts_footer` in `src/tools/web_fetch.rs`

### REQ-WEB-019: Web Fetch Response Truncation

`WebFetchTool` MUST truncate responses exceeding configured size limits.

#### Scenario: Response within limit

- WHEN response body is within the size limit
- THEN MUST return it unchanged
- Test: `truncate_within_limit` in `src/tools/web_fetch.rs`

#### Scenario: Response over limit

- WHEN response body exceeds the size limit
- THEN MUST truncate to configured maximum
- Test: `truncate_over_limit` in `src/tools/web_fetch.rs`

### REQ-WEB-020: Web Fetch Provider Support

`WebFetchTool` MUST support multiple providers (nanohtml2text, firecrawl, tavily) with fallback.

#### Scenario: Firecrawl provider requires API key

- WHEN firecrawl provider is configured without an API key
- THEN MUST return an error
- Test: `firecrawl_provider_requires_api_key` in `src/tools/web_fetch.rs`

#### Scenario: Tavily provider requires API key

- WHEN tavily provider is configured without an API key
- THEN MUST return an error
- Test: `tavily_provider_requires_api_key` in `src/tools/web_fetch.rs`

#### Scenario: API key parsing from comma-separated string

- WHEN multiple comma-separated API keys are configured
- THEN MUST parse into individual keys
- Test: `parses_multiple_api_keys` in `src/tools/web_fetch.rs`

#### Scenario: Round-robin API key rotation

- WHEN multiple API keys are available
- THEN MUST rotate through them in round-robin order
- Test: `round_robin_api_key_selection_cycles` in `src/tools/web_fetch.rs`

### REQ-WEB-021: Web Fetch Security Modes

`WebFetchTool` MUST respect read-only mode and rate limiting.

#### Scenario: Read-only mode blocks fetch

- WHEN read-only mode is active
- THEN MUST reject requests
- Test: `blocks_readonly_mode` in `src/tools/web_fetch.rs`

#### Scenario: Rate limiting blocks fetch

- WHEN rate limit is exceeded
- THEN MUST reject requests
- Test: `blocks_rate_limited` in `src/tools/web_fetch.rs`

### REQ-WEB-022: Web Fetch Blocklist Enforcement

`WebFetchTool` MUST enforce blocklist rules that override allowlist entries.

#### Scenario: Blocklist rejects exact match

- WHEN a URL matches a blocklist entry exactly
- THEN MUST reject the request
- Test: `blocklist_rejects_exact_match` in `src/tools/web_fetch.rs`

#### Scenario: Blocklist rejects subdomain

- WHEN a URL is a subdomain of a blocklisted domain
- THEN MUST reject the request
- Test: `blocklist_rejects_subdomain` in `src/tools/web_fetch.rs`

#### Scenario: Blocklist wins over allowlist

- WHEN a domain appears in both allowlist and blocklist
- THEN blocklist MUST take precedence and reject the request
- Test: `blocklist_wins_over_allowlist` in `src/tools/web_fetch.rs`

#### Scenario: Blocklist allows non-blocked domains

- WHEN a URL does not match any blocklist entry
- THEN MUST allow the request (if allowlist permits)
- Test: `blocklist_allows_non_blocked` in `src/tools/web_fetch.rs`

### REQ-WEB-023: HTTP Request Response Truncation

`HttpRequestTool` MUST truncate responses exceeding configured size limits.

#### Scenario: Response within limit

- WHEN response body is within the size limit
- THEN MUST return it unchanged
- Test: `truncate_response_within_limit` in `src/tools/http_request.rs`

#### Scenario: Response over limit

- WHEN response body exceeds the size limit
- THEN MUST truncate to configured maximum
- Test: `truncate_response_over_limit` in `src/tools/http_request.rs`

### REQ-WEB-024: Web Search Input Validation

`WebSearchTool` MUST validate input parameters and configuration limits.

#### Scenario: Constructor clamps search limits

- WHEN the tool is constructed with out-of-range search limits
- THEN MUST clamp to valid bounds
- Test: `test_constructor_clamps_web_search_limits` in `src/tools/web_search_tool.rs`

#### Scenario: Missing query parameter

- WHEN execute is called without a query parameter
- THEN MUST return an error
- Test: `test_execute_missing_query` in `src/tools/web_search_tool.rs`

#### Scenario: Empty query parameter

- WHEN execute is called with an empty query string
- THEN MUST return an error
- Test: `test_execute_empty_query` in `src/tools/web_search_tool.rs`

### REQ-WEB-025: Web Search Provider API Key Validation

`WebSearchTool` MUST validate API key availability before executing provider-specific searches.

#### Scenario: Brave search without API key

- WHEN Brave search is attempted without an API key configured
- THEN MUST return an error
- Test: `test_execute_brave_without_api_key` in `src/tools/web_search_tool.rs`

#### Scenario: Firecrawl search without API key

- WHEN Firecrawl search is attempted without an API key configured
- THEN MUST return an error
- Test: `test_execute_firecrawl_without_api_key` in `src/tools/web_search_tool.rs`

#### Scenario: Tavily search without API key

- WHEN Tavily search is attempted without an API key configured
- THEN MUST return an error
- Test: `test_execute_tavily_without_api_key` in `src/tools/web_search_tool.rs`

## Coverage Notes

Total test coverage: 141 tests across 4 source files.

| File | Tests | Requirements Covered |
|------|-------|---------------------|
| `src/tools/browser.rs` | 33 | REQ-WEB-001 through REQ-WEB-004 |
| `src/tools/web_search_tool.rs` | 20 | REQ-WEB-005 through REQ-WEB-009, REQ-WEB-024, REQ-WEB-025 |
| `src/tools/http_request.rs` | 57 | REQ-WEB-010 through REQ-WEB-015, REQ-WEB-023 |
| `src/tools/web_fetch.rs` | 31 | REQ-WEB-016 through REQ-WEB-022 |

Every test function is explicitly named in at least one scenario above.

## Mock Strategy

- HTTP requests: `wiremock::MockServer` for API endpoint simulation
- URL validation: Direct function testing with crafted URLs
- Browser backends: Feature-gated; test URL validation and config parsing directly
- SecurityPolicy: Default construction with test-safe settings
- API keys: Test-only placeholder values for key rotation tests
