//! Core types for instruction file auditing.

use std::path::Path;

/// Configuration for instruction file discovery and auditing.
///
/// Different projects can customize behavior by providing different configs.
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Project root marker files, checked in order.
    /// agent-doc uses many (Cargo.toml, package.json, etc.); corky uses only Cargo.toml.
    pub root_markers: Vec<&'static str>,

    /// Whether to include CLAUDE.md in root-level discovery and agent file checks.
    /// agent-doc: true, corky: false.
    pub include_claude_md: bool,

    /// Source file extensions to check for staleness comparison.
    /// agent-doc: broad (rs, ts, py, etc.); corky: just "rs".
    pub source_extensions: Vec<&'static str>,

    /// Source directories to scan for staleness.
    /// agent-doc: ["src", "lib", "app", ...]; corky: just ["src"].
    pub source_dirs: Vec<&'static str>,

    /// Directories to skip when scanning for source files.
    pub skip_dirs: Vec<&'static str>,
}

impl AuditConfig {
    /// Config matching agent-doc's current behavior: broad project detection,
    /// includes CLAUDE.md, scans many source extensions.
    pub fn agent_doc() -> Self {
        Self {
            root_markers: vec![
                "Cargo.toml",
                "package.json",
                "pyproject.toml",
                "setup.py",
                "go.mod",
                "Gemfile",
                "pom.xml",
                "build.gradle",
                "CMakeLists.txt",
                "Makefile",
                "flake.nix",
                "deno.json",
                "composer.json",
            ],
            include_claude_md: true,
            source_extensions: vec![
                "rs", "ts", "tsx", "js", "jsx", "py", "go", "rb", "java", "kt", "c", "cpp", "h",
                "hpp", "cs", "swift", "zig", "hs", "ml", "ex", "exs", "clj", "scala", "lua",
                "php", "sh", "bash", "zsh",
            ],
            source_dirs: vec!["src", "lib", "app", "pkg", "cmd", "internal"],
            skip_dirs: vec![
                "node_modules",
                "target",
                "build",
                "dist",
                ".git",
                "__pycache__",
                ".venv",
                "vendor",
                ".next",
                "out",
            ],
        }
    }

    /// Config matching corky's current behavior: Cargo.toml-only root detection,
    /// excludes CLAUDE.md from audit, scans only .rs files.
    pub fn corky() -> Self {
        Self {
            root_markers: vec!["Cargo.toml"],
            include_claude_md: false,
            source_extensions: vec!["rs"],
            source_dirs: vec!["src"],
            skip_dirs: vec!["target", ".git"],
        }
    }
}

/// An issue found during auditing.
pub struct Issue {
    pub file: String,
    pub line: usize,
    pub end_line: usize,
    pub message: String,
    pub warning: bool,
}

/// Check if a file path refers to an agent instruction file.
pub fn is_agent_file(rel: &str, config: &AuditConfig) -> bool {
    let name = Path::new(rel)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if name == "AGENTS.md" || name == "SKILL.md" {
        return true;
    }
    if config.include_claude_md && name == "CLAUDE.md" {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_agent_file_with_claude() {
        let config = AuditConfig::agent_doc();
        assert!(is_agent_file("AGENTS.md", &config));
        assert!(is_agent_file("SKILL.md", &config));
        assert!(is_agent_file("CLAUDE.md", &config));
        assert!(is_agent_file("src/AGENTS.md", &config));
        assert!(is_agent_file(".claude/skills/email/SKILL.md", &config));
        assert!(is_agent_file("nested/path/CLAUDE.md", &config));
    }

    #[test]
    fn is_agent_file_without_claude() {
        let config = AuditConfig::corky();
        assert!(is_agent_file("AGENTS.md", &config));
        assert!(is_agent_file("SKILL.md", &config));
        assert!(!is_agent_file("CLAUDE.md", &config));
    }

    #[test]
    fn is_agent_file_rejects() {
        let config = AuditConfig::agent_doc();
        assert!(!is_agent_file("README.md", &config));
        assert!(!is_agent_file("agents.md", &config));
        assert!(!is_agent_file("CHANGELOG.md", &config));
        assert!(!is_agent_file("src/main.rs", &config));
    }
}
