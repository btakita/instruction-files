//! Audit checks for instruction files.
//!
//! Cross-cutting checks (check_context_invariant, check_staleness, check_line_budget)
//! are re-exported from `agent-kit::audit_common`. Domain-specific checks
//! (check_actionable, check_tree_paths) are re-exported from `agent-rules`.

pub use agent_kit::audit_common::{check_context_invariant, check_line_budget, check_staleness};
pub use agent_rules::{check_actionable, check_tree_paths};

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
