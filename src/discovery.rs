//! Instruction file discovery: find project root and instruction files.

use crate::types::AuditConfig;
use std::path::{Path, PathBuf};

/// Find the project root by walking up from CWD.
///
/// Strategy depends on config:
/// - Pass 1: Check `config.root_markers` in order
/// - Pass 2: Check for `.git` directory
/// - Pass 3: Fall back to CWD
pub fn find_root(config: &AuditConfig) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Pass 1: Look for project marker files
    let mut dir = cwd.as_path();
    loop {
        for marker in &config.root_markers {
            if dir.join(marker).exists() {
                return dir.to_path_buf();
            }
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p,
            _ => break,
        }
    }

    // Pass 2: Look for .git directory
    dir = cwd.as_path();
    loop {
        if dir.join(".git").exists() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p,
            _ => break,
        }
    }

    // Pass 3: Fall back to CWD
    eprintln!("Warning: no project root marker found, using current directory");
    cwd
}

/// Discover all instruction files under the given root.
///
/// Searches for:
/// - Root-level: AGENTS.md, README.md, SPECS.md, and optionally CLAUDE.md
/// - Glob patterns: .claude/**/SKILL.md, .agents/**/SKILL.md, .agents/**/AGENTS.md, src/**/AGENTS.md
/// - If `config.include_claude_md`: also .claude/**/CLAUDE.md, src/**/CLAUDE.md
pub fn find_instruction_files(root: &Path, config: &AuditConfig) -> Vec<PathBuf> {
    let mut root_patterns = vec!["AGENTS.md", "README.md", "SPECS.md"];
    if config.include_claude_md {
        root_patterns.push("CLAUDE.md");
    }

    let mut found = std::collections::HashSet::new();

    for pattern in &root_patterns {
        let path = root.join(pattern);
        if path.exists() {
            found.insert(path);
        }
    }

    // Common glob patterns
    let mut glob_patterns = vec![
        ".claude/**/SKILL.md",
        ".agents/**/SKILL.md",
        ".agents/**/AGENTS.md",
        "src/**/AGENTS.md",
    ];

    if config.include_claude_md {
        glob_patterns.push(".claude/**/CLAUDE.md");
        glob_patterns.push("src/**/CLAUDE.md");
    }

    for pattern in &glob_patterns {
        if let Ok(entries) = glob::glob(&root.join(pattern).to_string_lossy()) {
            for entry in entries.flatten() {
                found.insert(entry);
            }
        }
    }

    let mut result: Vec<PathBuf> = found.into_iter().collect();
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn find_instruction_files_root_patterns_with_claude() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::write(root.join("CLAUDE.md"), "# Doc").unwrap();
        fs::write(root.join("README.md"), "# Readme").unwrap();
        fs::write(root.join("AGENTS.md"), "# Agents").unwrap();

        let config = AuditConfig::agent_doc();
        let files = find_instruction_files(root, &config);
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.ends_with("CLAUDE.md")));
        assert!(files.iter().any(|f| f.ends_with("README.md")));
        assert!(files.iter().any(|f| f.ends_with("AGENTS.md")));
    }

    #[test]
    fn find_instruction_files_root_patterns_without_claude() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::write(root.join("CLAUDE.md"), "# Doc").unwrap();
        fs::write(root.join("README.md"), "# Readme").unwrap();
        fs::write(root.join("AGENTS.md"), "# Agents").unwrap();

        let config = AuditConfig::corky();
        let files = find_instruction_files(root, &config);
        assert_eq!(files.len(), 2);
        assert!(!files.iter().any(|f| f.ends_with("CLAUDE.md")));
        assert!(files.iter().any(|f| f.ends_with("README.md")));
        assert!(files.iter().any(|f| f.ends_with("AGENTS.md")));
    }

    #[test]
    fn find_instruction_files_glob_patterns() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join(".claude/skills/email")).unwrap();
        fs::write(root.join(".claude/skills/email/SKILL.md"), "# Skill").unwrap();

        fs::create_dir_all(root.join(".claude/settings")).unwrap();
        fs::write(root.join(".claude/settings/CLAUDE.md"), "# Claude").unwrap();

        fs::create_dir_all(root.join("src/agent")).unwrap();
        fs::write(root.join("src/agent/CLAUDE.md"), "# Agent").unwrap();
        fs::write(root.join("src/agent/AGENTS.md"), "# Agents").unwrap();

        let config = AuditConfig::agent_doc();
        let files = find_instruction_files(root, &config);
        assert_eq!(files.len(), 4);
    }

    #[test]
    fn find_instruction_files_glob_patterns_corky() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join(".claude/skills/email")).unwrap();
        fs::write(root.join(".claude/skills/email/SKILL.md"), "# Skill").unwrap();

        fs::create_dir_all(root.join("src/agent")).unwrap();
        fs::write(root.join("src/agent/CLAUDE.md"), "# Agent").unwrap();
        fs::write(root.join("src/agent/AGENTS.md"), "# Agents").unwrap();

        let config = AuditConfig::corky();
        let files = find_instruction_files(root, &config);
        // SKILL.md + src/AGENTS.md (no CLAUDE.md)
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn find_instruction_files_empty() {
        let tmp = TempDir::new().unwrap();
        let config = AuditConfig::agent_doc();
        let files = find_instruction_files(tmp.path(), &config);
        assert!(files.is_empty());
    }

    #[test]
    fn find_instruction_files_sorted() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::write(root.join("README.md"), "# R").unwrap();
        fs::write(root.join("CLAUDE.md"), "# C").unwrap();
        fs::write(root.join("AGENTS.md"), "# A").unwrap();

        let config = AuditConfig::agent_doc();
        let files = find_instruction_files(root, &config);
        let names: Vec<_> = files.iter().map(|f| f.file_name().unwrap()).collect();
        assert!(names.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn find_instruction_files_discovers_specs_md() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::write(root.join("SPECS.md"), "# Spec").unwrap();
        fs::write(root.join("AGENTS.md"), "# Agents").unwrap();

        let config = AuditConfig::corky();
        let files = find_instruction_files(root, &config);
        assert!(files.iter().any(|f| f.ends_with("SPECS.md")));
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn find_instruction_files_deduplicates() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::write(root.join("CLAUDE.md"), "# Doc").unwrap();

        let config = AuditConfig::agent_doc();
        let files = find_instruction_files(root, &config);
        assert_eq!(files.len(), 1);
    }
}
