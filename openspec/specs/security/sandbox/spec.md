# Security Sandbox Specification

## Purpose
Define the behavioral contract for ZeroClaw's OS-level sandbox subsystem: the Sandbox trait, NoopSandbox fallback, sandbox detection/selection logic, and backend implementations.

## Scope
- Files: `src/security/traits.rs` (4 tests), `src/security/detect.rs` (4 tests)
- Total tests: 8
- Risk tier: HIGH (OS-level isolation boundary)

## Requirements

---

### Sandbox Trait (`src/security/traits.rs`)

### REQ-SAND-001: Sandbox MUST provide name and availability check
Every Sandbox impl MUST report its name() and whether it is_available() on the current platform.

#### Scenario: NoopSandbox availability
- WHEN NoopSandbox.is_available() is called
- THEN it returns true
- Test: `noop_sandbox_is_always_available` in `src/security/traits.rs`

### REQ-SAND-002: NoopSandbox MUST leave commands unchanged
wrap_command() on NoopSandbox MUST not modify the program or arguments.

#### Scenario: Noop wrap is identity
- WHEN NoopSandbox.wrap_command(&mut cmd) is called
- THEN cmd.get_program() and cmd.get_args() are unchanged
- Test: `noop_sandbox_wrap_command_is_noop` in `src/security/traits.rs`

### REQ-SAND-003: NoopSandbox name MUST be "none"
The NoopSandbox reports itself as "none" for diagnostics.

#### Scenario: Noop name
- WHEN NoopSandbox.name() is called
- THEN it returns "none"
- Test: `noop_sandbox_name` in `src/security/traits.rs`

### REQ-SAND-004: NoopSandbox description MUST indicate no sandboxing
The description MUST contain "No sandboxing" to clearly communicate the security posture.

#### Scenario: Noop description
- WHEN NoopSandbox.description() is called
- THEN it contains "No sandboxing"
- Test: `noop_sandbox_description` in `src/security/traits.rs`

---

### Sandbox Detection (`src/security/detect.rs`)

### REQ-SAND-DET-001: detect_best_sandbox MUST always return a valid sandbox
Even when no backend is available, auto-detection MUST return at least NoopSandbox.

#### Scenario: Fallback to noop
- WHEN detect_best_sandbox() is called on any platform
- THEN returned sandbox.is_available() is true
- Test: `detect_best_sandbox_returns_something` in `src/security/detect.rs`

### REQ-SAND-DET-002: Explicit None backend MUST return NoopSandbox
When SandboxBackend::None is configured or enabled=false, create_sandbox MUST return noop.

#### Scenario: Explicit disable
- WHEN SecurityConfig has backend=None and enabled=false
- THEN sandbox.name() == "none"
- Test: `explicit_none_returns_noop` in `src/security/detect.rs`

### REQ-SAND-DET-003: Auto mode MUST detect and return available sandbox
When backend=Auto, create_sandbox MUST probe and return the best available backend.

#### Scenario: Auto detection
- WHEN SecurityConfig has backend=Auto
- THEN sandbox.is_available() is true
- Test: `auto_mode_detects_something` in `src/security/detect.rs`

### REQ-SAND-DET-004: Unavailable explicit backend MUST fall back to noop
When an explicit backend is requested but not available, create_sandbox MUST fall back to NoopSandbox.

#### Scenario: Unavailable backend fallback
- WHEN a backend like Landlock is requested but not available
- THEN sandbox.name() == "none" (or whatever is actually available)
- Test: `unavailable_backend_falls_back_to_noop` in `src/security/detect.rs`

## Mock Strategy
Sandbox tests use SecurityConfig::default() with field overrides. Backend availability is environment-dependent; tests verify the fallback chain rather than specific backends.

## Coverage Notes
- `src/security/traits.rs`: 4 tests — fully covers NoopSandbox
- `src/security/detect.rs`: 4 tests — covers create_sandbox and detect_best_sandbox
- Docker/Firejail/Bubblewrap/Landlock backends: environment-dependent, tested through detect.rs fallback chain
- Total: 8 tests across 2 files
