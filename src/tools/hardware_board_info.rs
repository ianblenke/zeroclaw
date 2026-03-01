//! Hardware board info tool — returns chip name, architecture, memory map for Telegram/agent.
//!
//! Use when user asks "what board do I have?", "board info", "connected hardware", etc.
//! Uses probe-rs for Nucleo when available; otherwise static datasheet info.

use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::json;

/// Static board info (datasheets). Used when probe-rs is unavailable.
const BOARD_INFO: &[(&str, &str, &str)] = &[
    (
        "nucleo-f401re",
        "STM32F401RET6",
        "ARM Cortex-M4, 84 MHz. Flash: 512 KB, RAM: 128 KB. User LED on PA5 (pin 13).",
    ),
    (
        "nucleo-f411re",
        "STM32F411RET6",
        "ARM Cortex-M4, 100 MHz. Flash: 512 KB, RAM: 128 KB. User LED on PA5 (pin 13).",
    ),
    (
        "arduino-uno",
        "ATmega328P",
        "8-bit AVR, 16 MHz. Flash: 16 KB, SRAM: 2 KB. Built-in LED on pin 13.",
    ),
    (
        "arduino-uno-q",
        "STM32U585 + Qualcomm",
        "Dual-core: STM32 (MCU) + Linux (aarch64). GPIO via Bridge app on port 9999.",
    ),
    (
        "esp32",
        "ESP32",
        "Dual-core Xtensa LX6, 240 MHz. Flash: 4 MB typical. Built-in LED on GPIO 2.",
    ),
    (
        "rpi-gpio",
        "Raspberry Pi",
        "ARM Linux. Native GPIO via sysfs/rppal. No fixed LED pin.",
    ),
];

/// Tool: return full board info (chip, architecture, memory map) for agent/Telegram.
pub struct HardwareBoardInfoTool {
    boards: Vec<String>,
}

impl HardwareBoardInfoTool {
    pub fn new(boards: Vec<String>) -> Self {
        Self { boards }
    }

    fn static_info_for_board(&self, board: &str) -> Option<String> {
        BOARD_INFO
            .iter()
            .find(|(b, _, _)| *b == board)
            .map(|(_, chip, desc)| {
                format!(
                    "**Board:** {}\n**Chip:** {}\n**Description:** {}",
                    board, chip, desc
                )
            })
    }
}

#[async_trait]
impl Tool for HardwareBoardInfoTool {
    fn name(&self) -> &str {
        "hardware_board_info"
    }

    fn description(&self) -> &str {
        "Return full board info (chip, architecture, memory map) for connected hardware. Use when: user asks for 'board info', 'what board do I have', 'connected hardware', 'chip info', 'what hardware', or 'memory map'."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "board": {
                    "type": "string",
                    "description": "Optional board name (e.g. nucleo-f401re). If omitted, returns info for first configured board."
                }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let board = args
            .get("board")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.boards.first().cloned());

        let board = board.as_deref().unwrap_or("unknown");

        if self.boards.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(
                    "No peripherals configured. Add boards to config.toml [peripherals.boards]."
                        .into(),
                ),
            });
        }

        let mut output = String::new();

        #[cfg(feature = "probe")]
        if board == "nucleo-f401re" || board == "nucleo-f411re" {
            let chip = if board == "nucleo-f411re" {
                "STM32F411RETx"
            } else {
                "STM32F401RETx"
            };
            match probe_board_info(chip) {
                Ok(info) => {
                    return Ok(ToolResult {
                        success: true,
                        output: info,
                        error: None,
                    });
                }
                Err(e) => {
                    use std::fmt::Write;
                    let _ = write!(
                        output,
                        "probe-rs attach failed: {e}. Using static info.\n\n"
                    );
                }
            }
        }

        if let Some(info) = self.static_info_for_board(board) {
            output.push_str(&info);
            if let Some(mem) = memory_map_static(board) {
                use std::fmt::Write;
                let _ = write!(output, "\n\n**Memory map:**\n{mem}");
            }
        } else {
            use std::fmt::Write;
            let _ = write!(
                output,
                "Board '{board}' configured. No static info available."
            );
        }

        Ok(ToolResult {
            success: true,
            output,
            error: None,
        })
    }
}

#[cfg(feature = "probe")]
fn probe_board_info(chip: &str) -> anyhow::Result<String> {
    use probe_rs::config::MemoryRegion;
    use probe_rs::{Session, SessionConfig};

    let session = Session::auto_attach(chip, SessionConfig::default())
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let target = session.target();
    let arch = session.architecture();

    let mut out = format!(
        "**Board:** {}\n**Chip:** {}\n**Architecture:** {:?}\n\n**Memory map:**\n",
        chip, target.name, arch
    );
    for region in target.memory_map.iter() {
        match region {
            MemoryRegion::Ram(ram) => {
                let (start, end) = (ram.range.start, ram.range.end);
                out.push_str(&format!(
                    "RAM: 0x{:08X} - 0x{:08X} ({} KB)\n",
                    start,
                    end,
                    (end - start) / 1024
                ));
            }
            MemoryRegion::Nvm(flash) => {
                let (start, end) = (flash.range.start, flash.range.end);
                out.push_str(&format!(
                    "Flash: 0x{:08X} - 0x{:08X} ({} KB)\n",
                    start,
                    end,
                    (end - start) / 1024
                ));
            }
            _ => {}
        }
    }
    out.push_str("\n(Info read via USB/SWD — no firmware on target needed.)");
    Ok(out)
}

fn memory_map_static(board: &str) -> Option<&'static str> {
    match board {
        "nucleo-f401re" | "nucleo-f411re" => Some(
            "Flash: 0x0800_0000 - 0x0807_FFFF (512 KB)\nRAM: 0x2000_0000 - 0x2001_FFFF (128 KB)",
        ),
        "arduino-uno" => Some("Flash: 16 KB, SRAM: 2 KB, EEPROM: 1 KB"),
        "esp32" => Some("Flash: 4 MB, IRAM/DRAM per ESP-IDF layout"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-HW-001-SC01
    #[test]
    fn tool_name_and_description() {
        let tool = HardwareBoardInfoTool::new(vec!["nucleo-f401re".into()]);
        assert_eq!(tool.name(), "hardware_board_info");
        assert!(!tool.description().is_empty());
    }

    /// REQ-HW-001-SC02
    #[test]
    fn parameters_schema_has_board_property() {
        let tool = HardwareBoardInfoTool::new(vec![]);
        let schema = tool.parameters_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["board"].is_object());
    }

    /// REQ-HW-001-SC03
    #[test]
    fn static_info_for_known_board() {
        let tool = HardwareBoardInfoTool::new(vec!["nucleo-f401re".into()]);
        let info = tool.static_info_for_board("nucleo-f401re");
        assert!(info.is_some());
        let info = info.unwrap();
        assert!(info.contains("STM32F401RET6"));
        assert!(info.contains("nucleo-f401re"));
    }

    /// REQ-HW-001-SC04
    #[test]
    fn static_info_for_unknown_board() {
        let tool = HardwareBoardInfoTool::new(vec!["unknown-board".into()]);
        let info = tool.static_info_for_board("unknown-board");
        assert!(info.is_none());
    }

    /// REQ-HW-001-SC05
    #[test]
    fn static_info_covers_all_board_entries() {
        let tool = HardwareBoardInfoTool::new(vec![]);
        for &(board, chip, _) in BOARD_INFO {
            let info = tool.static_info_for_board(board).unwrap();
            assert!(info.contains(chip), "Board {board} should mention chip {chip}");
        }
    }

    /// REQ-HW-001-SC06
    #[test]
    fn memory_map_static_nucleo() {
        let map = memory_map_static("nucleo-f401re");
        assert!(map.is_some());
        assert!(map.unwrap().contains("Flash"));
        // f411re returns same map
        assert_eq!(memory_map_static("nucleo-f411re"), map);
    }

    /// REQ-HW-001-SC07
    #[test]
    fn memory_map_static_arduino() {
        let map = memory_map_static("arduino-uno");
        assert!(map.is_some());
        assert!(map.unwrap().contains("SRAM"));
    }

    /// REQ-HW-001-SC08
    #[test]
    fn memory_map_static_esp32() {
        let map = memory_map_static("esp32");
        assert!(map.is_some());
        assert!(map.unwrap().contains("IRAM"));
    }

    /// REQ-HW-001-SC09
    #[test]
    fn memory_map_static_unknown_returns_none() {
        assert!(memory_map_static("rpi-gpio").is_none());
        assert!(memory_map_static("nonexistent").is_none());
    }

    /// REQ-HW-001-SC10
    #[tokio::test]
    async fn execute_no_boards_returns_error() {
        let tool = HardwareBoardInfoTool::new(vec![]);
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("No peripherals"));
    }

    /// REQ-HW-001-SC11
    #[tokio::test]
    async fn execute_known_board_returns_static_info() {
        let tool = HardwareBoardInfoTool::new(vec!["esp32".into()]);
        let result = tool.execute(serde_json::json!({"board": "esp32"})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("ESP32"));
    }

    /// REQ-HW-001-SC12
    #[tokio::test]
    async fn execute_unknown_board_falls_back() {
        let tool = HardwareBoardInfoTool::new(vec!["custom-board".into()]);
        let result = tool.execute(serde_json::json!({"board": "custom-board"})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("No static info available"));
    }

    /// REQ-HW-001-SC13
    #[tokio::test]
    async fn execute_defaults_to_first_board() {
        let tool = HardwareBoardInfoTool::new(vec!["arduino-uno".into(), "esp32".into()]);
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("ATmega328P"));
    }

    /// REQ-HW-001-SC14
    #[tokio::test]
    async fn execute_nucleo_includes_memory_map() {
        let tool = HardwareBoardInfoTool::new(vec!["nucleo-f401re".into()]);
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Memory map"));
        assert!(result.output.contains("Flash"));
    }
}
