//! Audit checks for instruction files.
//!
//! Cross-cutting checks (check_context_invariant, check_staleness, check_line_budget)
//! are re-exported from `agent-kit::audit_common`. Domain-specific checks
//! (check_actionable, check_tree_paths) are re-exported from `agent-rules`.

use crate::types::Issue;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

pub use agent_kit::audit_common::{check_context_invariant, check_line_budget, check_staleness};
pub use agent_rules::{check_actionable, check_tree_paths};

/// Regex matching markdown links whose target ends with `LIBRARY_CONTEXT_POLICY.md`.
static POLICY_LINK_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[([^\]]*)\]\(([^)]*LIBRARY_CONTEXT_POLICY\.md[^)]*)\)|(?m)^((?:https?://|\.\.?/)[^\s]*LIBRARY_CONTEXT_POLICY\.md)\s*$").unwrap()
});

/// Check that a library AGENTS.md contains a `## Library Context Policy` section
/// with a resolvable link to `LIBRARY_CONTEXT_POLICY.md`.
///
/// Only applies to AGENTS.md files in subdirectories (not the project root).
pub fn check_library_context_policy(rel: &str, content: &str, root: &Path) -> Vec<Issue> {
    // Only check AGENTS.md files in subdirectories
    let path = std::path::Path::new(rel);
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name != "AGENTS.md" {
        return vec![];
    }
    // Skip root-level AGENTS.md
    let parent = path.parent().and_then(|p| p.to_str()).unwrap_or("");
    if parent.is_empty() {
        return vec![];
    }

    let mut issues = Vec::new();

    // Find the ## Library Context Policy section
    let mut section_line = 0;
    let mut in_section = false;
    let mut section_content = String::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "## Library Context Policy" {
            section_line = i + 1;
            in_section = true;
            continue;
        }
        if in_section {
            // Stop at next ## heading
            if trimmed.starts_with("## ") {
                break;
            }
            section_content.push_str(line);
            section_content.push('\n');
        }
    }

    if section_line == 0 {
        issues.push(Issue {
            file: rel.to_string(),
            line: 0,
            end_line: 0,
            message: "Missing required '## Library Context Policy' section".to_string(),
            warning: false,
        });
        return issues;
    }

    // Check for a link to LIBRARY_CONTEXT_POLICY.md
    if let Some(caps) = POLICY_LINK_RE.captures(&section_content) {
        // Extract the URL (group 2 for markdown links, group 3 for bare URLs)
        let url = caps.get(2).or_else(|| caps.get(3)).map(|m| m.as_str()).unwrap_or("");

        // Validate link resolves
        if url.starts_with("http://") || url.starts_with("https://") {
            // Accept any URL containing LIBRARY_CONTEXT_POLICY.md
        } else {
            // Relative path — resolve from the AGENTS.md file's directory
            let agents_dir = root.join(parent);
            let resolved = agents_dir.join(url);
            if !resolved.exists() {
                issues.push(Issue {
                    file: rel.to_string(),
                    line: section_line,
                    end_line: 0,
                    message: format!(
                        "Library Context Policy link '{}' does not resolve (expected at {})",
                        url,
                        resolved.display()
                    ),
                    warning: false,
                });
            }
        }
    } else {
        issues.push(Issue {
            file: rel.to_string(),
            line: section_line,
            end_line: 0,
            message: "Library Context Policy section has no link to LIBRARY_CONTEXT_POLICY.md"
                .to_string(),
            warning: false,
        });
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_rules::extract_tree_paths;
    use crate::types::AuditConfig;
    use std::fs;
    use tempfile::TempDir;

    // --- extract_tree_paths ---

    #[test]
    fn extract_tree_paths_basic() {
        let content = "\
## Project Structure

```
src/
  main.rs
  lib.rs
```
";
        let paths = extract_tree_paths(content);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].1, "src/main.rs");
        assert_eq!(paths[1].1, "src/lib.rs");
    }

    #[test]
    fn extract_tree_paths_nested() {
        let content = "\
## Project Structure

```
src/
  agent/
    mod.rs
    claude.rs
  main.rs
```
";
        let paths = extract_tree_paths(content);
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].1, "src/agent/mod.rs");
        assert_eq!(paths[1].1, "src/agent/claude.rs");
        assert_eq!(paths[2].1, "src/main.rs");
    }

    #[test]
    fn extract_tree_paths_symlink() {
        let content = "\
## Project Structure

```
mail -> ../data/mail
src/
  main.rs
```
";
        let paths = extract_tree_paths(content);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].1, "src/main.rs");
    }

    #[test]
    fn extract_tree_paths_with_comments() {
        let content = "\
## Project Structure

```
src/
  main.rs  # entry point
  lib.rs   # library
```
";
        let paths = extract_tree_paths(content);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].1, "src/main.rs");
        assert_eq!(paths[1].1, "src/lib.rs");
    }

    #[test]
    fn extract_tree_paths_no_section() {
        let content = "# Just a heading\n\nSome text.\n";
        let paths = extract_tree_paths(content);
        assert!(paths.is_empty());
    }

    #[test]
    fn extract_tree_paths_empty_block() {
        let content = "\
## Project Structure

```
```
";
        let paths = extract_tree_paths(content);
        assert!(paths.is_empty());
    }

    #[test]
    fn extract_tree_paths_stops_at_next_section() {
        let content = "\
## Project Structure

```
src/
  main.rs
```

## Other Section

Some text.
";
        let paths = extract_tree_paths(content);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].1, "src/main.rs");
    }

    #[test]
    fn extract_tree_paths_line_numbers() {
        let content = "\
## Project Structure

```
Cargo.toml
src/
  main.rs
```
";
        let paths = extract_tree_paths(content);
        assert_eq!(paths[0], (4, "Cargo.toml".to_string()));
        assert_eq!(paths[1], (6, "src/main.rs".to_string()));
    }

    // --- check_tree_paths ---

    #[test]
    fn check_tree_paths_existing() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

        let content = "\
## Project Structure

```
src/
  main.rs
```
";
        let issues = check_tree_paths("CLAUDE.md", content, root);
        assert!(issues.is_empty());
    }

    #[test]
    fn check_tree_paths_missing() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let content = "\
## Project Structure

```
src/
  missing.rs
```
";
        let issues = check_tree_paths("CLAUDE.md", content, root);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("missing.rs"));
        assert!(!issues[0].warning);
    }

    #[test]
    fn check_tree_paths_skips_brackets() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let content = "\
## Project Structure

```
src/
  [generated files]
```
";
        let issues = check_tree_paths("CLAUDE.md", content, root);
        assert!(issues.is_empty());
    }

    #[test]
    fn check_tree_paths_skips_env() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let content = "\
## Project Structure

```
.env
```
";
        let issues = check_tree_paths("CLAUDE.md", content, root);
        assert!(issues.is_empty());
    }

    // --- check_actionable ---

    #[test]
    fn check_actionable_skips_non_agent_files() {
        let config = AuditConfig::agent_doc();
        let issues = check_actionable("README.md", "## Overview\n\nSome overview.\n", &config);
        assert!(issues.is_empty());
    }

    #[test]
    fn check_actionable_informational_heading() {
        let config = AuditConfig::agent_doc();
        let content = "# Doc\n\n## Overview\n\nSome overview text.\n\n## Rules\n\nDo this.\n";
        let issues = check_actionable("CLAUDE.md", content, &config);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("Informational section"));
        assert!(issues[0].message.contains("Overview"));
        assert!(issues[0].warning);
    }

    #[test]
    fn check_actionable_no_informational_heading() {
        let config = AuditConfig::agent_doc();
        let content = "# Doc\n\n## Conventions\n\nUse serde.\n";
        let issues = check_actionable("AGENTS.md", content, &config);
        assert!(issues.is_empty());
    }

    #[test]
    fn check_actionable_large_code_block_without_context() {
        let config = AuditConfig::agent_doc();
        let mut lines = vec!["# Doc".to_string(), "".to_string()];
        lines.push("```rust".to_string());
        for i in 0..10 {
            lines.push(format!("let x{} = {};", i, i));
        }
        lines.push("```".to_string());
        let content = lines.join("\n");

        let issues = check_actionable("CLAUDE.md", &content, &config);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("Large code block"));
        assert!(issues[0].warning);
    }

    #[test]
    fn check_actionable_large_code_block_with_imperative() {
        let config = AuditConfig::agent_doc();
        let mut lines = vec![
            "# Doc".to_string(),
            "".to_string(),
            "Use the following pattern:".to_string(),
        ];
        lines.push("```rust".to_string());
        for i in 0..10 {
            lines.push(format!("let x{} = {};", i, i));
        }
        lines.push("```".to_string());
        let content = lines.join("\n");

        let issues = check_actionable("CLAUDE.md", &content, &config);
        assert!(issues.is_empty());
    }

    #[test]
    fn check_actionable_small_code_block_ok() {
        let config = AuditConfig::agent_doc();
        let content = "# Doc\n\n```\nfoo\nbar\n```\n";
        let issues = check_actionable("AGENTS.md", content, &config);
        assert!(issues.is_empty());
    }

    #[test]
    fn check_actionable_large_table() {
        let config = AuditConfig::agent_doc();
        let mut lines = vec!["# Doc".to_string(), "".to_string()];
        lines.push("| Col A | Col B |".to_string());
        lines.push("|-------|-------|".to_string());
        for i in 0..6 {
            lines.push(format!("| row{} | val{} |", i, i));
        }
        let content = lines.join("\n");

        let issues = check_actionable("CLAUDE.md", &content, &config);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("Large table"));
        assert!(issues[0].warning);
    }

    #[test]
    fn check_actionable_small_table_ok() {
        let config = AuditConfig::agent_doc();
        let content = "\
# Doc

| A | B |
|---|---|
| 1 | 2 |
| 3 | 4 |
";
        let issues = check_actionable("SKILL.md", content, &config);
        assert!(issues.is_empty());
    }

    #[test]
    fn check_actionable_link_heavy_list() {
        let config = AuditConfig::agent_doc();
        let mut lines = vec!["# Doc".to_string(), "".to_string()];
        for i in 0..12 {
            lines.push(format!("- [link{}](https://example.com/{})", i, i));
        }
        let content = lines.join("\n");

        let issues = check_actionable("CLAUDE.md", &content, &config);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("Link-heavy list"));
        assert!(issues[0].warning);
    }

    // --- check_library_context_policy ---

    #[test]
    fn library_context_policy_missing_section() {
        let tmp = TempDir::new().unwrap();
        let content = "# My Library\n\nSome content.\n";
        let issues = check_library_context_policy("src/mylib/AGENTS.md", content, tmp.path());
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("Missing required"));
        assert!(!issues[0].warning);
    }

    #[test]
    fn library_context_policy_missing_link() {
        let tmp = TempDir::new().unwrap();
        let content = "# My Library\n\n## Library Context Policy\n\nSome text without a link.\n";
        let issues = check_library_context_policy("src/mylib/AGENTS.md", content, tmp.path());
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("no link"));
    }

    #[test]
    fn library_context_policy_valid_relative_link() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/mylib")).unwrap();
        fs::create_dir_all(root.join("src/instruction-files")).unwrap();
        fs::write(
            root.join("src/instruction-files/LIBRARY_CONTEXT_POLICY.md"),
            "policy",
        )
        .unwrap();

        let content = "\
# My Library

## Library Context Policy

This library follows the agent-loop library-context policy. Contributors
authoring `AGENTS.md`, `SKILL.md`, or runbooks in this repo must read:

[Library Context Policy](../instruction-files/LIBRARY_CONTEXT_POLICY.md)

before making changes.
";
        let issues = check_library_context_policy("src/mylib/AGENTS.md", content, root);
        assert!(issues.is_empty());
    }

    #[test]
    fn library_context_policy_valid_url() {
        let tmp = TempDir::new().unwrap();
        let content = "\
# My Library

## Library Context Policy

This library follows the agent-loop library-context policy.

[Policy](https://github.com/btakita/agent-loop/blob/main/src/instruction-files/LIBRARY_CONTEXT_POLICY.md)
";
        let issues = check_library_context_policy("src/mylib/AGENTS.md", content, tmp.path());
        assert!(issues.is_empty());
    }

    #[test]
    fn library_context_policy_broken_relative_link() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src/mylib")).unwrap();

        let content = "\
# My Library

## Library Context Policy

[Policy](../instruction-files/LIBRARY_CONTEXT_POLICY.md)
";
        let issues =
            check_library_context_policy("src/mylib/AGENTS.md", content, tmp.path());
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("does not resolve"));
    }

    #[test]
    fn library_context_policy_skips_root_agents() {
        let tmp = TempDir::new().unwrap();
        let content = "# Root\n\nNo policy section needed.\n";
        let issues = check_library_context_policy("AGENTS.md", content, tmp.path());
        assert!(issues.is_empty());
    }

    #[test]
    fn library_context_policy_skips_non_agents() {
        let tmp = TempDir::new().unwrap();
        let content = "# Readme\n";
        let issues = check_library_context_policy("src/mylib/CLAUDE.md", content, tmp.path());
        assert!(issues.is_empty());
    }

    #[test]
    fn library_context_policy_bare_url() {
        let tmp = TempDir::new().unwrap();
        let content = "\
# My Library

## Library Context Policy

This library follows the agent-loop library-context policy.

https://github.com/btakita/agent-loop/blob/main/src/instruction-files/LIBRARY_CONTEXT_POLICY.md
";
        let issues = check_library_context_policy("src/mylib/AGENTS.md", content, tmp.path());
        assert!(issues.is_empty());
    }

    #[test]
    fn library_context_policy_bare_relative_path() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/mylib")).unwrap();
        fs::create_dir_all(root.join("src/instruction-files")).unwrap();
        fs::write(
            root.join("src/instruction-files/LIBRARY_CONTEXT_POLICY.md"),
            "policy",
        )
        .unwrap();

        let content = "\
# My Library

## Library Context Policy

This library follows the agent-loop library-context policy.

../instruction-files/LIBRARY_CONTEXT_POLICY.md
";
        let issues = check_library_context_policy("src/mylib/AGENTS.md", content, root);
        assert!(issues.is_empty());
    }

    #[test]
    fn check_actionable_short_link_list_ok() {
        let config = AuditConfig::agent_doc();
        let mut lines = vec!["# Doc".to_string(), "".to_string()];
        for i in 0..5 {
            lines.push(format!("- [link{}](https://example.com/{})", i, i));
        }
        let content = lines.join("\n");

        let issues = check_actionable("AGENTS.md", &content, &config);
        assert!(issues.is_empty());
    }

    #[test]
    fn check_actionable_claude_md_skipped_in_corky_config() {
        let config = AuditConfig::corky();
        let content = "# Doc\n\n## Overview\n\nSome overview.\n";
        let issues = check_actionable("CLAUDE.md", content, &config);
        assert!(issues.is_empty()); // CLAUDE.md is not an agent file in corky config
    }
}
