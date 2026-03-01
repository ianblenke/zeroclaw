//! Hardware memory read tool — read actual memory/register values from Nucleo via probe-rs.
//!
//! Use when user asks to "read register values", "read memory at address", "dump lower memory", etc.
//! Requires probe feature and Nucleo connected via USB.

use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::json;

/// RAM base for Nucleo-F401RE (STM32F401)
const NUCLEO_RAM_BASE: u64 = 0x2000_0000;

/// Tool: read memory at address from connected Nucleo via probe-rs.
pub struct HardwareMemoryReadTool {
    boards: Vec<String>,
}

impl HardwareMemoryReadTool {
    pub fn new(boards: Vec<String>) -> Self {
        Self { boards }
    }

    fn chip_for_board(board: &str) -> Option<&'static str> {
        match board {
            "nucleo-f401re" => Some("STM32F401RETx"),
            "nucleo-f411re" => Some("STM32F411RETx"),
            _ => None,
        }
    }
}

#[async_trait]
impl Tool for HardwareMemoryReadTool {
    fn name(&self) -> &str {
        "hardware_memory_read"
    }

    fn description(&self) -> &str {
        "Read actual memory/register values from Nucleo via USB. Use when: user asks to 'read register values', 'read memory at address', 'dump memory', 'lower memory 0-126', or 'give address and value'. Returns hex dump. Requires Nucleo connected via USB and probe feature. Params: address (hex, e.g. 0x20000000 for RAM start), length (bytes, default 128)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "address": {
                    "type": "string",
                    "description": "Memory address in hex (e.g. 0x20000000 for RAM start). Default: 0x20000000 (RAM base)."
                },
                "length": {
                    "type": "integer",
                    "description": "Number of bytes to read (default 128, max 256)."
                },
                "board": {
                    "type": "string",
                    "description": "Board name (nucleo-f401re). Optional if only one configured."
                }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        if self.boards.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(
                    "No peripherals configured. Add nucleo-f401re to config.toml [peripherals.boards]."
                        .into(),
                ),
            });
        }

        let board = args
            .get("board")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.boards.first().cloned())
            .unwrap_or_else(|| "nucleo-f401re".into());

        let chip = Self::chip_for_board(&board);
        if chip.is_none() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Memory read only supports nucleo-f401re, nucleo-f411re. Got: {}",
                    board
                )),
            });
        }

        let address_str = args
            .get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("0x20000000");
        let _address = parse_hex_address(address_str).unwrap_or(NUCLEO_RAM_BASE);

        let requested_length = args.get("length").and_then(|v| v.as_u64()).unwrap_or(128);
        let _length = usize::try_from(requested_length)
            .unwrap_or(256)
            .clamp(1, 256);

        #[cfg(feature = "probe")]
        {
            match probe_read_memory(chip.unwrap(), _address, _length) {
                Ok(output) => {
                    return Ok(ToolResult {
                        success: true,
                        output,
                        error: None,
                    });
                }
                Err(e) => {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!(
                            "probe-rs read failed: {}. Ensure Nucleo is connected via USB and built with --features probe.",
                            e
                        )),
                    });
                }
            }
        }

        #[cfg(not(feature = "probe"))]
        {
            Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(
                    "Memory read requires probe feature. Build with: cargo build --features hardware,probe"
                        .into(),
                ),
            })
        }
    }
}

fn parse_hex_address(s: &str) -> Option<u64> {
    let s = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    u64::from_str_radix(s, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-HW-002-SC01
    #[test]
    fn tool_name_and_description() {
        let tool = HardwareMemoryReadTool::new(vec!["nucleo-f401re".into()]);
        assert_eq!(tool.name(), "hardware_memory_read");
        assert!(!tool.description().is_empty());
    }

    /// REQ-HW-002-SC02
    #[test]
    fn parameters_schema_has_required_properties() {
        let tool = HardwareMemoryReadTool::new(vec![]);
        let schema = tool.parameters_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["address"].is_object());
        assert!(schema["properties"]["length"].is_object());
        assert!(schema["properties"]["board"].is_object());
    }

    /// REQ-HW-002-SC03
    #[test]
    fn parse_hex_with_0x_prefix() {
        assert_eq!(parse_hex_address("0x20000000"), Some(0x2000_0000));
        assert_eq!(parse_hex_address("0X20000000"), Some(0x2000_0000));
    }

    /// REQ-HW-002-SC04
    #[test]
    fn parse_hex_without_prefix() {
        assert_eq!(parse_hex_address("20000000"), Some(0x2000_0000));
        assert_eq!(parse_hex_address("FF"), Some(0xFF));
    }

    /// REQ-HW-002-SC05
    #[test]
    fn parse_hex_with_whitespace() {
        assert_eq!(parse_hex_address("  0x100  "), Some(0x100));
    }

    /// REQ-HW-002-SC06
    #[test]
    fn parse_hex_invalid_returns_none() {
        assert_eq!(parse_hex_address("not_hex"), None);
        assert_eq!(parse_hex_address("0xGG"), None);
    }

    /// REQ-HW-002-SC07
    #[test]
    fn chip_for_board_known() {
        assert_eq!(
            HardwareMemoryReadTool::chip_for_board("nucleo-f401re"),
            Some("STM32F401RETx")
        );
        assert_eq!(
            HardwareMemoryReadTool::chip_for_board("nucleo-f411re"),
            Some("STM32F411RETx")
        );
    }

    /// REQ-HW-002-SC08
    #[test]
    fn chip_for_board_unknown() {
        assert_eq!(HardwareMemoryReadTool::chip_for_board("esp32"), None);
        assert_eq!(HardwareMemoryReadTool::chip_for_board("rpi-gpio"), None);
    }

    /// REQ-HW-002-SC09
    #[tokio::test]
    async fn execute_no_boards_returns_error() {
        let tool = HardwareMemoryReadTool::new(vec![]);
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("No peripherals"));
    }

    /// REQ-HW-002-SC10
    #[tokio::test]
    async fn execute_unsupported_board_returns_error() {
        let tool = HardwareMemoryReadTool::new(vec!["esp32".into()]);
        let result = tool.execute(serde_json::json!({"board": "esp32"})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("only supports"));
    }

    /// REQ-HW-002-SC11
    #[tokio::test]
    async fn execute_without_probe_feature_returns_error() {
        // Without the "probe" feature, execute should return a build feature error
        let tool = HardwareMemoryReadTool::new(vec!["nucleo-f401re".into()]);
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        // Without probe feature, this returns an error about needing the feature
        #[cfg(not(feature = "probe"))]
        {
            assert!(!result.success);
            assert!(result.error.as_ref().unwrap().contains("probe feature"));
        }
        #[cfg(feature = "probe")]
        {
            // With probe feature, it would try to connect — just check it returns
            let _ = result;
        }
    }

    /// REQ-HW-002-SC12
    #[tokio::test]
    async fn execute_defaults_to_first_board() {
        let tool = HardwareMemoryReadTool::new(vec!["nucleo-f401re".into()]);
        // Without probe feature, it will fail but with the correct board selected
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        #[cfg(not(feature = "probe"))]
        {
            assert!(!result.success);
            assert!(result.error.as_ref().unwrap().contains("probe feature"));
        }
    }

    /// REQ-HW-002-SC13
    #[test]
    fn nucleo_ram_base_constant() {
        assert_eq!(NUCLEO_RAM_BASE, 0x2000_0000);
    }
}

#[cfg(feature = "probe")]
fn probe_read_memory(chip: &str, address: u64, length: usize) -> anyhow::Result<String> {
    use probe_rs::MemoryInterface;
    use probe_rs::Session;
    use probe_rs::SessionConfig;

    let mut session = Session::auto_attach(chip, SessionConfig::default())
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut core = session.core(0)?;
    let mut buf = vec![0u8; length];
    core.read_8(address, &mut buf)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Format as hex dump: address | bytes (16 per line)
    let mut out = format!("Memory read from 0x{:08X} ({} bytes):\n\n", address, length);
    const COLS: usize = 16;
    for (i, chunk) in buf.chunks(COLS).enumerate() {
        let addr = address + (i * COLS) as u64;
        let hex: String = chunk
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        let ascii: String = chunk
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        out.push_str(&format!("0x{:08X}  {:48}  {}\n", addr, hex, ascii));
    }
    Ok(out)
}
