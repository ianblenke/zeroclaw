//! Peripheral trait — hardware boards (STM32, RPi GPIO) that expose tools.
//!
//! Peripherals are the agent's "arms and legs": remote devices that run minimal
//! firmware and expose capabilities (GPIO, sensors, actuators) as tools.
//! See `docs/hardware-peripherals-design.md` for the communication protocol
//! and firmware integration guide.

use async_trait::async_trait;

use crate::tools::Tool;

/// A hardware peripheral that exposes capabilities as agent tools.
///
/// Implement this trait for each supported board type (e.g., Nucleo-F401RE
/// over serial, Raspberry Pi GPIO via sysfs/gpiod). When the agent connects
/// to a peripheral, the tools returned by [`tools`](Peripheral::tools) are
/// merged into the agent's tool registry, making hardware capabilities
/// available to the LLM as callable functions.
///
/// The lifecycle follows a connect → use → disconnect pattern. Implementations
/// must be `Send + Sync` because the peripheral may be accessed from multiple
/// async tasks after connection.
#[async_trait]
pub trait Peripheral: Send + Sync {
    /// Return the human-readable instance name of this peripheral.
    ///
    /// Should uniquely identify a specific device instance, including an index
    /// or serial number when multiple boards of the same type are connected
    /// (e.g., `"nucleo-f401re-0"`, `"rpi-gpio-hat-1"`).
    fn name(&self) -> &str;

    /// Return the board type identifier for this peripheral.
    ///
    /// A stable, lowercase string used in configuration and factory registration
    /// (e.g., `"nucleo-f401re"`, `"rpi-gpio"`). Must match the key used in
    /// the config schema's peripheral section.
    fn board_type(&self) -> &str;

    /// Establish a connection to the peripheral hardware.
    ///
    /// Opens the underlying transport (serial port, GPIO bus, I²C, etc.) and
    /// performs any initialization handshake required by the firmware.
    ///
    /// # Errors
    ///
    /// Returns an error if the device is unreachable, the transport cannot be
    /// opened, or the firmware handshake fails.
    async fn connect(&mut self) -> anyhow::Result<()>;

    /// Disconnect from the peripheral and release all held resources.
    ///
    /// Closes serial ports, unexports GPIO pins, and performs any cleanup
    /// required for a safe shutdown. After this call, [`health_check`](Peripheral::health_check)
    /// should return `false` until [`connect`](Peripheral::connect) is called again.
    ///
    /// # Errors
    ///
    /// Returns an error if resource cleanup fails (e.g., serial port busy).
    async fn disconnect(&mut self) -> anyhow::Result<()>;

    /// Check whether the peripheral is reachable and responsive.
    ///
    /// Performs a lightweight probe (e.g., a ping command over serial) without
    /// altering device state. Returns `true` if the device responds within an
    /// implementation-defined timeout.
    async fn health_check(&self) -> bool;

    /// Return the tools this peripheral exposes to the agent.
    ///
    /// Each returned [`Tool`] delegates execution to the underlying hardware
    /// (e.g., `gpio_read`, `gpio_write`, `sensor_read`). The agent merges
    /// these into its tool registry after a successful
    /// [`connect`](Peripheral::connect).
    fn tools(&self) -> Vec<Box<dyn Tool>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::traits::{ToolResult, ToolSpec};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    struct MockTool {
        tool_name: String,
    }

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            &self.tool_name
        }
        fn description(&self) -> &str {
            "mock tool"
        }
        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({"type": "object"})
        }
        async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
            Ok(ToolResult {
                success: true,
                output: "mock".into(),
                error: None,
            })
        }
    }

    struct MockPeripheral {
        connected: Arc<AtomicBool>,
    }

    impl MockPeripheral {
        fn new() -> Self {
            Self {
                connected: Arc::new(AtomicBool::new(false)),
            }
        }
    }

    #[async_trait]
    impl Peripheral for MockPeripheral {
        fn name(&self) -> &str {
            "mock-peripheral-0"
        }

        fn board_type(&self) -> &str {
            "mock-board"
        }

        async fn connect(&mut self) -> anyhow::Result<()> {
            self.connected.store(true, Ordering::SeqCst);
            Ok(())
        }

        async fn disconnect(&mut self) -> anyhow::Result<()> {
            self.connected.store(false, Ordering::SeqCst);
            Ok(())
        }

        async fn health_check(&self) -> bool {
            self.connected.load(Ordering::SeqCst)
        }

        fn tools(&self) -> Vec<Box<dyn Tool>> {
            vec![Box::new(MockTool {
                tool_name: "gpio_read".into(),
            })]
        }
    }

    /// REQ-PERIPH-001-SC01
    #[tokio::test]
    async fn peripheral_lifecycle_connect_check_disconnect() {
        let mut peripheral = MockPeripheral::new();

        // Before connect: health check should fail
        assert!(!peripheral.health_check().await);

        // Connect
        peripheral.connect().await.unwrap();
        assert!(peripheral.health_check().await);

        // Tools are available
        let tools = peripheral.tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name(), "gpio_read");

        // Disconnect
        peripheral.disconnect().await.unwrap();
        assert!(!peripheral.health_check().await);
    }

    /// REQ-PERIPH-002-SC01
    #[test]
    fn peripheral_name_and_board_type() {
        let peripheral = MockPeripheral::new();
        assert_eq!(peripheral.name(), "mock-peripheral-0");
        assert_eq!(peripheral.board_type(), "mock-board");
    }
}
