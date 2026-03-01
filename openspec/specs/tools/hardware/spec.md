# Tools Hardware Specification

## Purpose

Define requirements for the hardware peripheral tool implementations: board info and memory read.

## Scope

- Files: `src/tools/hardware_board_info.rs` (209 LOC, 0→14 tests), `src/tools/hardware_memory_read.rs` (184 LOC, 0→13 tests)
- Risk tier: MEDIUM (read-only hardware probing; feature-gated probe-rs dependency)
- Existing tests: 0 → 27 (new)

## Requirements

### REQ-HW-001: Hardware Board Info Tool

`HardwareBoardInfoTool` MUST implement the `Tool` trait with name `"hardware_board_info"` and return static or probe-based board information.

#### Scenario: Tool identity

- WHEN `name()` and `description()` are called
- THEN MUST return `"hardware_board_info"` and a non-empty description
- Test: `tool_name_and_description` in `src/tools/hardware_board_info.rs`

#### Scenario: Schema has board property

- WHEN `parameters_schema()` is called
- THEN MUST return JSON schema with optional `board` property
- Test: `parameters_schema_has_board_property` in `src/tools/hardware_board_info.rs`

#### Scenario: Static info for known board

- WHEN a known board name is queried
- THEN MUST return chip name and description from BOARD_INFO
- Test: `static_info_for_known_board` in `src/tools/hardware_board_info.rs`

#### Scenario: Static info for unknown board

- WHEN an unknown board name is queried
- THEN MUST return None
- Test: `static_info_for_unknown_board` in `src/tools/hardware_board_info.rs`

#### Scenario: All boards have static info

- WHEN each board in BOARD_INFO is queried
- THEN MUST return info containing the chip name
- Test: `static_info_covers_all_board_entries` in `src/tools/hardware_board_info.rs`

#### Scenario: Memory map for nucleo boards

- WHEN `memory_map_static` is called for nucleo boards
- THEN MUST return flash and RAM regions
- Test: `memory_map_static_nucleo` in `src/tools/hardware_board_info.rs`

#### Scenario: Memory map for unknown boards

- WHEN `memory_map_static` is called for an unsupported board
- THEN MUST return None
- Test: `memory_map_static_unknown_returns_none` in `src/tools/hardware_board_info.rs`

#### Scenario: Execute with no boards configured

- WHEN no boards are configured
- THEN MUST return error about missing peripherals config
- Test: `execute_no_boards_returns_error` in `src/tools/hardware_board_info.rs`

#### Scenario: Execute with known board

- WHEN a known board is configured and queried
- THEN MUST return static info with chip name
- Test: `execute_known_board_returns_static_info` in `src/tools/hardware_board_info.rs`

#### Scenario: Execute with unknown board

- WHEN an unknown board is configured
- THEN MUST return fallback message
- Test: `execute_unknown_board_falls_back` in `src/tools/hardware_board_info.rs`

#### Scenario: Execute defaults to first board

- WHEN no board parameter is specified
- THEN MUST use first configured board
- Test: `execute_defaults_to_first_board` in `src/tools/hardware_board_info.rs`

#### Scenario: Execute nucleo includes memory map

- WHEN nucleo board is queried
- THEN MUST include memory map in output
- Test: `execute_nucleo_includes_memory_map` in `src/tools/hardware_board_info.rs`

### REQ-HW-002: Hardware Memory Read Tool

`HardwareMemoryReadTool` MUST implement the `Tool` trait with name `"hardware_memory_read"` and read memory from connected hardware via probe-rs.

#### Scenario: Tool identity

- WHEN `name()` and `description()` are called
- THEN MUST return `"hardware_memory_read"` and a non-empty description
- Test: `tool_name_and_description` in `src/tools/hardware_memory_read.rs`

#### Scenario: Schema has required properties

- WHEN `parameters_schema()` is called
- THEN MUST return JSON schema with address, length, and board properties
- Test: `parameters_schema_has_required_properties` in `src/tools/hardware_memory_read.rs`

#### Scenario: Hex address parsing with 0x prefix

- WHEN a hex address with 0x/0X prefix is provided
- THEN MUST parse correctly
- Test: `parse_hex_with_0x_prefix` in `src/tools/hardware_memory_read.rs`

#### Scenario: Hex address parsing without prefix

- WHEN a hex address without prefix is provided
- THEN MUST parse as hex
- Test: `parse_hex_without_prefix` in `src/tools/hardware_memory_read.rs`

#### Scenario: Hex address parsing with whitespace

- WHEN a hex address has leading/trailing whitespace
- THEN MUST trim and parse correctly
- Test: `parse_hex_with_whitespace` in `src/tools/hardware_memory_read.rs`

#### Scenario: Invalid hex address

- WHEN an invalid hex string is provided
- THEN MUST return None
- Test: `parse_hex_invalid_returns_none` in `src/tools/hardware_memory_read.rs`

#### Scenario: Chip mapping for known boards

- WHEN a supported board name is provided
- THEN MUST return correct chip identifier
- Test: `chip_for_board_known` in `src/tools/hardware_memory_read.rs`

#### Scenario: Chip mapping for unknown boards

- WHEN an unsupported board name is provided
- THEN MUST return None
- Test: `chip_for_board_unknown` in `src/tools/hardware_memory_read.rs`

#### Scenario: Execute with no boards configured

- WHEN no boards are configured
- THEN MUST return error about missing peripherals config
- Test: `execute_no_boards_returns_error` in `src/tools/hardware_memory_read.rs`

#### Scenario: Execute with unsupported board

- WHEN a non-nucleo board is specified
- THEN MUST return error listing supported boards
- Test: `execute_unsupported_board_returns_error` in `src/tools/hardware_memory_read.rs`

#### Scenario: Execute without probe feature

- WHEN probe feature is not enabled
- THEN MUST return error about build feature requirement
- Test: `execute_without_probe_feature_returns_error` in `src/tools/hardware_memory_read.rs`

## Mock Strategy

- Board info: Direct construction with `Vec<String>` board names
- Memory read: Feature-gated; non-probe tests exercise validation and pure functions
- No external hardware required for tests — probe-rs paths only tested with `--features probe`
