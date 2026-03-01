/// A single file to be written when scaffolding from this template.
pub struct TemplateFile {
    /// Relative path inside the skill directory (e.g. "src/main.rs")
    pub path: &'static str,
    pub content: &'static str,
}

/// A complete, runnable skill template.
pub struct SkillTemplate {
    pub name: &'static str,
    pub language: &'static str,
    pub description: &'static str,
    /// Example args JSON for `zeroclaw skill test`
    pub test_args: &'static str,
    pub files: &'static [TemplateFile],
}

// ── Rust templates ────────────────────────────────────────────────────────────

const RUST_WEATHER_FILES: &[TemplateFile] = &[
    TemplateFile {
        path: "Cargo.toml",
        content: include_str!("../../templates/rust/weather_lookup/Cargo.toml"),
    },
    TemplateFile {
        path: "src/main.rs",
        content: include_str!("../../templates/rust/weather_lookup/src/main.rs"),
    },
    TemplateFile {
        path: "manifest.json",
        content: include_str!("../../templates/rust/weather_lookup/manifest.json"),
    },
    TemplateFile {
        path: ".cargo/config.toml",
        content: include_str!("../../templates/rust/weather_lookup/.cargo/config.toml"),
    },
];

const RUST_CALCULATOR_FILES: &[TemplateFile] = &[
    TemplateFile {
        path: "Cargo.toml",
        content: include_str!("../../templates/rust/calculator/Cargo.toml"),
    },
    TemplateFile {
        path: "src/main.rs",
        content: include_str!("../../templates/rust/calculator/src/main.rs"),
    },
    TemplateFile {
        path: "manifest.json",
        content: include_str!("../../templates/rust/calculator/manifest.json"),
    },
    TemplateFile {
        path: ".cargo/config.toml",
        content: include_str!("../../templates/rust/calculator/.cargo/config.toml"),
    },
];

// ── TypeScript templates ──────────────────────────────────────────────────────

const TS_HELLO_FILES: &[TemplateFile] = &[
    TemplateFile {
        path: "package.json",
        content: include_str!("../../templates/typescript/hello_world/package.json"),
    },
    TemplateFile {
        path: "tsconfig.json",
        content: include_str!("../../templates/typescript/hello_world/tsconfig.json"),
    },
    TemplateFile {
        path: "src/index.ts",
        content: include_str!("../../templates/typescript/hello_world/src/index.ts"),
    },
    TemplateFile {
        path: "manifest.json",
        content: include_str!("../../templates/typescript/hello_world/manifest.json"),
    },
];

// ── Go templates ─────────────────────────────────────────────────────────────

const GO_WORD_COUNT_FILES: &[TemplateFile] = &[
    TemplateFile {
        path: "go.mod",
        content: include_str!("../../templates/go/word_count/go.mod"),
    },
    TemplateFile {
        path: "main.go",
        content: include_str!("../../templates/go/word_count/main.go"),
    },
    TemplateFile {
        path: "manifest.json",
        content: include_str!("../../templates/go/word_count/manifest.json"),
    },
];

// ── Python templates ──────────────────────────────────────────────────────────

const PY_TEXT_TRANSFORM_FILES: &[TemplateFile] = &[
    TemplateFile {
        path: "main.py",
        content: include_str!("../../templates/python/text_transform/main.py"),
    },
    TemplateFile {
        path: "manifest.json",
        content: include_str!("../../templates/python/text_transform/manifest.json"),
    },
];

// ── Registry ──────────────────────────────────────────────────────────────────

pub const ALL: &[SkillTemplate] = &[
    SkillTemplate {
        name: "weather_lookup",
        language: "rust",
        description: "Look up current weather for a city (mock data, WASI-safe)",
        test_args: r#"{"city":"hanoi"}"#,
        files: RUST_WEATHER_FILES,
    },
    SkillTemplate {
        name: "calculator",
        language: "rust",
        description: "Arithmetic calculator — add, subtract, multiply, divide",
        test_args: r#"{"op":"add","a":3,"b":7}"#,
        files: RUST_CALCULATOR_FILES,
    },
    SkillTemplate {
        name: "hello_world",
        language: "typescript",
        description: "Greet a user by name (TypeScript + Javy)",
        test_args: r#"{"name":"ZeroClaw"}"#,
        files: TS_HELLO_FILES,
    },
    SkillTemplate {
        name: "word_count",
        language: "go",
        description: "Count words, lines, and characters in text (Go + TinyGo)",
        test_args: r#"{"text":"hello world foo bar"}"#,
        files: GO_WORD_COUNT_FILES,
    },
    SkillTemplate {
        name: "text_transform",
        language: "python",
        description: "Transform text: uppercase, lowercase, reverse, title case",
        test_args: r#"{"text":"hello world","transform":"uppercase"}"#,
        files: PY_TEXT_TRANSFORM_FILES,
    },
];

/// Find a template by name. Also accepts language aliases ("rust", "typescript", "go", "python").
pub fn find(name: &str) -> Option<&'static SkillTemplate> {
    // Exact name match first
    if let Some(t) = ALL.iter().find(|t| t.name == name) {
        return Some(t);
    }
    // Language alias → first template for that language
    let lang = match name {
        "rust" => "rust",
        "typescript" | "ts" => "typescript",
        "go" => "go",
        "python" | "py" => "python",
        _ => return None,
    };
    ALL.iter().find(|t| t.language == lang)
}

/// Apply `__SKILL_NAME__` / `__BIN_NAME__` substitutions to template content.
pub fn apply(content: &str, name: &str, bin_name: &str) -> String {
    content
        .replace("__SKILL_NAME__", name)
        .replace("__BIN_NAME__", bin_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-SKILL-001-SC01
    #[test]
    fn find_by_exact_name() {
        let t = find("weather_lookup").unwrap();
        assert_eq!(t.name, "weather_lookup");
        assert_eq!(t.language, "rust");
    }

    /// REQ-SKILL-001-SC02
    #[test]
    fn find_by_language_alias() {
        let t = find("rust").unwrap();
        assert_eq!(t.language, "rust");
    }

    /// REQ-SKILL-001-SC03
    #[test]
    fn find_typescript_aliases() {
        assert!(find("typescript").is_some());
        assert!(find("ts").is_some());
        assert_eq!(find("typescript").unwrap().name, find("ts").unwrap().name);
    }

    /// REQ-SKILL-001-SC04
    #[test]
    fn find_python_aliases() {
        assert!(find("python").is_some());
        assert!(find("py").is_some());
    }

    /// REQ-SKILL-001-SC05
    #[test]
    fn find_unknown_returns_none() {
        assert!(find("nonexistent").is_none());
        assert!(find("java").is_none());
    }

    /// REQ-SKILL-001-SC06
    #[test]
    fn all_templates_have_files() {
        for t in ALL {
            assert!(!t.files.is_empty(), "Template {} has no files", t.name);
            assert!(!t.name.is_empty());
            assert!(!t.language.is_empty());
            assert!(!t.description.is_empty());
            assert!(!t.test_args.is_empty());
        }
    }

    /// REQ-SKILL-001-SC07
    #[test]
    fn apply_substitutions() {
        let content = "name=__SKILL_NAME__ bin=__BIN_NAME__";
        let result = apply(content, "my_skill", "my-bin");
        assert_eq!(result, "name=my_skill bin=my-bin");
    }

    /// REQ-SKILL-001-SC08
    #[test]
    fn apply_no_substitutions() {
        let content = "no placeholders here";
        let result = apply(content, "skill", "bin");
        assert_eq!(result, "no placeholders here");
    }

    /// REQ-SKILL-001-SC09
    #[test]
    fn all_templates_count() {
        assert_eq!(ALL.len(), 5);
    }

    /// REQ-SKILL-001-SC10
    #[test]
    fn find_go_template() {
        let t = find("go").unwrap();
        assert_eq!(t.language, "go");
        assert_eq!(t.name, "word_count");
    }
}
