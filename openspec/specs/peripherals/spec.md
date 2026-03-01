# Peripherals Specification

## Purpose

Define requirements for the peripheral trait contract, hardware board implementations, and device management.

## Scope

- Files: 10 files in `src/peripherals/` (~1,673 LOC) + 9 files in `src/hardware/` (~3,137 LOC)
- Risk tier: MEDIUM (hardware interactions; feature-gated for probe/GPIO)
- Total tests: 10 in peripherals/, 64 in hardware/ = 74 tests

## Requirements

---

### Peripheral Trait (`src/peripherals/traits.rs`)

### REQ-PERIPH-001: Peripheral MUST implement full lifecycle

All peripherals MUST implement the `Peripheral` trait with connect, disconnect, health_check, and tools methods.

#### Scenario: Mock peripheral lifecycle
- WHEN a mock peripheral connects, checks health, lists tools, then disconnects
- THEN connect succeeds, health_check returns true when connected, tools() returns registered tools, disconnect succeeds, health_check returns false after disconnect
- Test: `peripheral_lifecycle_connect_check_disconnect` in `src/peripherals/traits.rs`

#### Scenario: Peripheral identity
- WHEN `name()` and `board_type()` are called
- THEN they return the configured values
- Test: `peripheral_name_and_board_type` in `src/peripherals/traits.rs`

---

### Capabilities Tool (`src/peripherals/capabilities_tool.rs`)

### REQ-PERIPH-002: HardwareCapabilitiesTool MUST query device capabilities

#### Scenario: Tool name and description
- WHEN `name()` and `description()` are called
- THEN name is `"hardware_capabilities"` and description is non-empty
- Test: `tool_name_and_description` in `src/peripherals/capabilities_tool.rs`

#### Scenario: Schema has board property
- WHEN `parameters_schema()` is called
- THEN schema is an object with a `board` property
- Test: `parameters_schema_has_board_property` in `src/peripherals/capabilities_tool.rs`

#### Scenario: Execute with no boards
- WHEN `execute({})` is called with no boards configured
- THEN result is not successful and output contains "No serial boards"
- Test: `execute_no_boards_returns_no_boards_message` in `src/peripherals/capabilities_tool.rs`

#### Scenario: Execute with filter no match
- WHEN `execute({"board": "nonexistent"})` is called
- THEN result is not successful and output contains "No matching board"
- Test: `execute_with_filter_no_match_returns_message` in `src/peripherals/capabilities_tool.rs`

---

### Peripheral Module (`src/peripherals/mod.rs`)

### REQ-PERIPH-003: list_configured_boards MUST respect enabled flag and board list

#### Scenario: Disabled returns empty
- WHEN peripherals are disabled in config
- THEN `list_configured_boards()` returns empty vec
- Test: `list_configured_boards_when_disabled_returns_empty` in `src/peripherals/mod.rs`

#### Scenario: Enabled with boards returns boards
- WHEN peripherals are enabled with board configs
- THEN `list_configured_boards()` returns configured boards
- Test: `list_configured_boards_when_enabled_with_boards` in `src/peripherals/mod.rs`

#### Scenario: Enabled but no boards returns empty
- WHEN peripherals are enabled but no boards configured
- THEN `list_configured_boards()` returns empty vec
- Test: `list_configured_boards_when_enabled_but_no_boards` in `src/peripherals/mod.rs`

### REQ-PERIPH-004: create_peripheral_tools MUST return empty when disabled

#### Scenario: Disabled config
- WHEN peripherals are disabled in config
- THEN `create_peripheral_tools()` returns empty vec
- Test: `create_peripheral_tools_returns_empty_when_disabled` in `src/peripherals/mod.rs`

---

### Hardware Device Abstraction (`src/hardware/device.rs`)

### REQ-PERIPH-005: DeviceRegistry MUST manage device lifecycle with alias assignment

#### Scenario: Sequential alias assignment
- WHEN multiple devices of same type are registered
- THEN aliases are assigned sequentially (e.g. `pico0`, `pico1`)
- Test: `registry_assigns_sequential_aliases` in `src/hardware/device.rs`

#### Scenario: Get device by alias
- WHEN `get_device()` is called with a valid alias
- THEN the corresponding device is returned
- Test: `registry_get_device_by_alias` in `src/hardware/device.rs`

#### Scenario: Unknown alias returns None
- WHEN `get_device()` is called with an unknown alias
- THEN it returns None
- Test: `registry_unknown_alias_returns_none` in `src/hardware/device.rs`

#### Scenario: Context without transport
- WHEN `context()` is called for a device without transport attached
- THEN it returns None
- Test: `registry_context_none_without_transport` in `src/hardware/device.rs`

#### Scenario: Default registry is empty
- WHEN `DeviceRegistry::default()` is created
- THEN it is empty
- Test: `registry_default_is_empty` in `src/hardware/device.rs`

#### Scenario: Aliases returns all registered aliases
- WHEN `aliases()` is called
- THEN all registered aliases are returned
- Test: `registry_aliases_returns_all` in `src/hardware/device.rs`

#### Scenario: get() is alias for get_device()
- WHEN `get()` is called
- THEN it returns same result as `get_device()`
- Test: `registry_get_is_alias_for_get_device` in `src/hardware/device.rs`

#### Scenario: all() returns every device
- WHEN `all()` is called
- THEN every registered device is returned
- Test: `registry_all_returns_every_device` in `src/hardware/device.rs`

### REQ-PERIPH-006: DeviceRegistry prompt_summary MUST format device list

#### Scenario: Empty summary
- WHEN no devices are registered
- THEN `prompt_summary()` returns empty/default string
- Test: `registry_prompt_summary_empty` in `src/hardware/device.rs`

#### Scenario: Summary with devices
- WHEN devices are registered
- THEN `prompt_summary()` includes device info
- Test: `registry_prompt_summary_with_devices` in `src/hardware/device.rs`

#### Scenario: Summary one-liner per device
- WHEN `summary()` is called with devices
- THEN each device gets one line
- Test: `registry_summary_one_liner_per_device` in `src/hardware/device.rs`

#### Scenario: Summary empty when no devices
- WHEN `summary()` is called with no devices
- THEN result is empty
- Test: `registry_summary_empty_when_no_devices` in `src/hardware/device.rs`

### REQ-PERIPH-007: Device MUST provide capabilities and identity

#### Scenario: Default capabilities all false
- WHEN `DeviceCapabilities::from_kind()` is called
- THEN capabilities reflect the device kind
- Test: `device_capabilities_default_all_false` in `src/hardware/device.rs`

#### Scenario: Device port returns device path
- WHEN device has a path set
- THEN `port()` returns it
- Test: `device_port_returns_device_path` in `src/hardware/device.rs`

#### Scenario: Device port None without path
- WHEN device has no path
- THEN `port()` returns None
- Test: `device_port_none_without_path` in `src/hardware/device.rs`

### REQ-PERIPH-008: DeviceKind MUST identify from USB vendor ID

#### Scenario: Known VID maps to kind
- WHEN `DeviceKind::from_vid()` is called with a known vendor ID
- THEN it returns the correct DeviceKind
- Test: `device_kind_from_vid_known` in `src/hardware/device.rs`

#### Scenario: Unknown VID returns None
- WHEN `DeviceKind::from_vid()` is called with unknown vendor ID
- THEN it returns None
- Test: `device_kind_from_vid_unknown` in `src/hardware/device.rs`

#### Scenario: DeviceKind display
- WHEN DeviceKind variants are displayed
- THEN they produce human-readable names
- Test: `device_kind_display` in `src/hardware/device.rs`

#### Scenario: Register sets kind from VID
- WHEN a device is registered with a known VID
- THEN the kind is set automatically
- Test: `register_sets_kind_from_vid` in `src/hardware/device.rs`

### REQ-PERIPH-009: alias_prefix MUST map board names to short prefixes

#### Scenario: Pico variants
- WHEN alias_prefix is called with Pico board names
- THEN it returns "pico" prefix
- Test: `alias_prefix_pico_variants` in `src/hardware/device.rs`

#### Scenario: Arduino prefix
- WHEN alias_prefix is called with Arduino board name
- THEN it returns "ard" prefix
- Test: `alias_prefix_arduino` in `src/hardware/device.rs`

#### Scenario: ESP prefix
- WHEN alias_prefix is called with ESP board name
- THEN it returns "esp" prefix
- Test: `alias_prefix_esp` in `src/hardware/device.rs`

#### Scenario: Nucleo prefix
- WHEN alias_prefix is called with Nucleo board name
- THEN it returns "nuc" prefix
- Test: `alias_prefix_nucleo` in `src/hardware/device.rs`

#### Scenario: RPi prefix
- WHEN alias_prefix is called with RPi board name
- THEN it returns "rpi" prefix
- Test: `alias_prefix_rpi` in `src/hardware/device.rs`

#### Scenario: Unknown prefix
- WHEN alias_prefix is called with unknown board name
- THEN it returns "dev" prefix
- Test: `alias_prefix_unknown` in `src/hardware/device.rs`

---

### Hardware Board Registry (`src/hardware/registry.rs`)

### REQ-PERIPH-010: Board lookup MUST match by VID/PID

#### Scenario: Known board (Nucleo F401RE)
- WHEN `lookup_board()` is called with Nucleo VID/PID
- THEN it returns the matching BoardInfo
- Test: `lookup_nucleo_f401re` in `src/hardware/registry.rs`

#### Scenario: Unknown board returns None
- WHEN `lookup_board()` is called with unknown VID/PID
- THEN it returns None
- Test: `lookup_unknown_returns_none` in `src/hardware/registry.rs`

#### Scenario: Known boards list is not empty
- WHEN `known_boards()` is called
- THEN it returns a non-empty list
- Test: `known_boards_not_empty` in `src/hardware/registry.rs`

#### Scenario: Pico standard lookup
- WHEN `lookup_board()` is called with standard Pico VID/PID
- THEN it returns Pico board info
- Test: `lookup_pico_standard` in `src/hardware/registry.rs`

#### Scenario: Pico W lookup
- WHEN `lookup_board()` is called with Pico W VID/PID
- THEN it returns Pico W board info
- Test: `lookup_pico_w` in `src/hardware/registry.rs`

---

### GPIO Tools (`src/hardware/gpio.rs`)

### REQ-PERIPH-011: GpioWriteTool MUST write pin values via transport

#### Scenario: Write high success
- WHEN `gpio_write` is called with valid device, pin, and value=high
- THEN transport receives the write command and returns success
- Test: `gpio_write_success` in `src/hardware/gpio.rs`

#### Scenario: Write low success
- WHEN `gpio_write` is called with value=low
- THEN transport receives write command with low value
- Test: `gpio_write_low` in `src/hardware/gpio.rs`

#### Scenario: Write device error
- WHEN device returns an error response
- THEN tool result contains the error message
- Test: `gpio_write_device_error` in `src/hardware/gpio.rs`

#### Scenario: Write transport disconnected
- WHEN transport is disconnected
- THEN tool result indicates transport failure
- Test: `gpio_write_transport_disconnected` in `src/hardware/gpio.rs`

#### Scenario: Write unknown device
- WHEN device alias is not found in registry
- THEN tool result indicates unknown device
- Test: `gpio_write_unknown_device` in `src/hardware/gpio.rs`

#### Scenario: Write invalid value
- WHEN value is neither "high" nor "low"
- THEN tool result indicates invalid value
- Test: `gpio_write_invalid_value` in `src/hardware/gpio.rs`

#### Scenario: Write missing parameters
- WHEN required parameters are missing
- THEN tool result indicates missing params
- Test: `gpio_write_missing_params` in `src/hardware/gpio.rs`

### REQ-PERIPH-012: GpioReadTool MUST read pin values via transport

#### Scenario: Read high success
- WHEN `gpio_read` is called with valid device and pin
- THEN transport receives read command and returns pin value
- Test: `gpio_read_success` in `src/hardware/gpio.rs`

#### Scenario: Read low success
- WHEN pin reads as low
- THEN tool result contains low value
- Test: `gpio_read_low` in `src/hardware/gpio.rs`

#### Scenario: Read device error
- WHEN device returns an error response
- THEN tool result contains the error message
- Test: `gpio_read_device_error` in `src/hardware/gpio.rs`

#### Scenario: Read transport disconnected
- WHEN transport is disconnected
- THEN tool result indicates transport failure
- Test: `gpio_read_transport_disconnected` in `src/hardware/gpio.rs`

#### Scenario: Read missing parameters
- WHEN required parameters are missing
- THEN tool result indicates missing params
- Test: `gpio_read_missing_params` in `src/hardware/gpio.rs`

### REQ-PERIPH-013: gpio_tools factory MUST return both read and write tools

#### Scenario: Factory returns two tools
- WHEN `gpio_tools()` is called with a registry
- THEN it returns exactly 2 tools (read + write)
- Test: `gpio_tools_factory_returns_two` in `src/hardware/gpio.rs`

#### Scenario: Write spec is valid
- WHEN GpioWriteTool spec is examined
- THEN it has correct name and schema
- Test: `gpio_write_spec_is_valid` in `src/hardware/gpio.rs`

#### Scenario: Read spec is valid
- WHEN GpioReadTool spec is examined
- THEN it has correct name and schema
- Test: `gpio_read_spec_is_valid` in `src/hardware/gpio.rs`

---

### Serial Transport (`src/hardware/serial.rs`)

### REQ-PERIPH-014: HardwareSerialTransport MUST manage serial connection state

#### Scenario: Constructor stores path and baud
- WHEN `SerialTransport::new()` is called with path and baud rate
- THEN both are stored correctly
- Test: `serial_transport_new_stores_path_and_baud` in `src/hardware/serial.rs`

#### Scenario: Default baud rate
- WHEN `with_default_baud()` is called
- THEN the default baud rate is used
- Test: `serial_transport_default_baud` in `src/hardware/serial.rs`

#### Scenario: Transport kind is Serial
- WHEN `kind()` is called
- THEN it returns `TransportKind::Serial`
- Test: `serial_transport_kind_is_serial` in `src/hardware/serial.rs`

#### Scenario: Not connected for nonexistent path
- WHEN `is_connected()` is called for a nonexistent device path
- THEN it returns false
- Test: `is_connected_false_for_nonexistent_path` in `src/hardware/serial.rs`

### REQ-PERIPH-015: Serial path validation MUST enforce allowed prefixes

#### Scenario: Valid path prefixes accepted
- WHEN paths with `/dev/ttyUSB`, `/dev/ttyACM`, `/dev/cu.` prefixes are checked
- THEN they are accepted
- Test: `allowed_paths_accept_valid_prefixes` in `src/hardware/serial.rs`

#### Scenario: Invalid path prefixes rejected
- WHEN paths without valid serial prefixes are checked
- THEN they are rejected
- Test: `allowed_paths_reject_invalid_prefixes` in `src/hardware/serial.rs`

#### Scenario: Send rejects disallowed path
- WHEN `send()` is called with a disallowed device path
- THEN it returns an error
- Test: `send_rejects_disallowed_path` in `src/hardware/serial.rs`

#### Scenario: Send returns disconnected for missing device
- WHEN `send()` is called for a non-existent device
- THEN it returns a disconnected error
- Test: `send_returns_disconnected_for_missing_device` in `src/hardware/serial.rs`

#### Scenario: Ping returns false for missing device
- WHEN `ping_handshake()` is called for a non-existent device
- THEN it returns false
- Test: `ping_handshake_returns_false_for_missing_device` in `src/hardware/serial.rs`

---

### Wire Protocol (`src/hardware/protocol.rs`)

### REQ-PERIPH-016: ZcCommand MUST serialize/deserialize correctly

#### Scenario: Command serialization roundtrip
- WHEN a ZcCommand is serialized to JSON and back
- THEN all fields are preserved
- Test: `zc_command_serialization_roundtrip` in `src/hardware/protocol.rs`

#### Scenario: Simple command has empty params
- WHEN `ZcCommand::simple("ping")` is created
- THEN params is an empty JSON object
- Test: `zc_command_simple_has_empty_params` in `src/hardware/protocol.rs`

#### Scenario: Wire format matches spec
- WHEN ZcCommand is serialized
- THEN JSON format matches the ZC wire protocol specification
- Test: `zc_command_wire_format_matches_spec` in `src/hardware/protocol.rs`

### REQ-PERIPH-017: ZcResponse MUST handle success, error, and firmware formats

#### Scenario: Success response roundtrip
- WHEN `ZcResponse::success()` is created and serialized
- THEN status is "ok" and data is preserved
- Test: `zc_response_success_roundtrip` in `src/hardware/protocol.rs`

#### Scenario: Error response roundtrip
- WHEN `ZcResponse::error()` is created and serialized
- THEN status is "error" and message is preserved
- Test: `zc_response_error_roundtrip` in `src/hardware/protocol.rs`

#### Scenario: Response from firmware JSON
- WHEN real firmware JSON is parsed
- THEN ZcResponse is correctly deserialized
- Test: `zc_response_from_firmware_json` in `src/hardware/protocol.rs`

#### Scenario: Missing optional fields
- WHEN response JSON omits optional fields
- THEN deserialization succeeds with None values
- Test: `zc_response_missing_optional_fields` in `src/hardware/protocol.rs`

---

### Transport Abstraction (`src/hardware/transport.rs`)

### REQ-PERIPH-018: TransportKind and TransportError MUST display correctly

#### Scenario: TransportKind display
- WHEN TransportKind variants are formatted
- THEN they produce human-readable strings
- Test: `transport_kind_display` in `src/hardware/transport.rs`

#### Scenario: TransportError display
- WHEN TransportError variants are formatted
- THEN they produce descriptive error messages
- Test: `transport_error_display` in `src/hardware/transport.rs`

#### Scenario: TransportKind equality
- WHEN two TransportKind values are compared
- THEN same variants are equal, different variants are not
- Test: `transport_kind_equality` in `src/hardware/transport.rs`

---

## Mock Strategy

- Serial: Mock serial port (test without real hardware); path validation tested directly
- GPIO: Mock transport struct implementing Transport trait with canned responses
- Probe: Feature-gated; test non-probe paths only
- Peripheral boards: Direct construction with test config; mock Peripheral trait impl
- Device registry: Direct construction with test device data

## Coverage Notes
- `src/peripherals/traits.rs`: 2 tests — lifecycle and identity
- `src/peripherals/capabilities_tool.rs`: 4 tests — tool metadata and execute edge cases
- `src/peripherals/mod.rs`: 4 tests — board listing and tool creation
- `src/hardware/device.rs`: 25 tests — registry, aliases, capabilities, kind, port, summary
- `src/hardware/gpio.rs`: 15 tests — read/write success/error/edge cases, factory, specs
- `src/hardware/serial.rs`: 9 tests — constructor, path validation, connection, handshake
- `src/hardware/protocol.rs`: 7 tests — command/response serialization, wire format
- `src/hardware/transport.rs`: 3 tests — display and equality
- `src/hardware/registry.rs`: 5 tests — board lookup by VID/PID
