//! Ontology term validation for instruction files.
//!
//! Scans markdown content for `[term:Name]` annotations and verifies
//! that each referenced term has a corresponding `.md` file in the
//! ontology directory.

use crate::types::Issue;
use regex::Regex;
use std::path::Path;

/// Scan `content` for `[term:Name]` annotations and verify each term
/// has a `.md` file under `ontology_dir/src/`.
///
/// Returns an empty vec if `ontology_dir` does not exist (ontology not configured).
pub fn check_ontology_terms(file: &str, content: &str, ontology_dir: &Path) -> Vec<Issue> {
    if !ontology_dir.exists() {
        return Vec::new();
    }

    let re = Regex::new(r"\[term:([A-Za-z][A-Za-z0-9_-]*)\]").expect("valid regex");
    let mut issues = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for cap in re.captures_iter(line) {
            let term = &cap[1];
            let term_lower = term.to_lowercase();
            // Check for term.md in ontology_dir/src/
            let term_path = ontology_dir.join("src").join(format!("{}.md", term_lower));
            if !term_path.exists() {
                issues.push(Issue {
                    file: file.to_string(),
                    line: line_num + 1,
                    end_line: 0,
                    message: format!(
                        "Ontology term '{}' not found (expected {})",
                        term,
                        term_path.display()
                    ),
                    warning: false,
                });
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn term_found_in_ontology_dir() {
        let tmp = TempDir::new().unwrap();
        let onto_dir = tmp.path().join("ontology");
        fs::create_dir_all(onto_dir.join("src")).unwrap();
        fs::write(onto_dir.join("src/existence.md"), "# Existence\n").unwrap();

        let content = "See [term:Existence] for details.\n";
        let issues = check_ontology_terms("CLAUDE.md", content, &onto_dir);
        assert!(issues.is_empty(), "expected no issues, got {} issue(s)", issues.len());
    }

    #[test]
    fn term_not_found_reports_issue() {
        let tmp = TempDir::new().unwrap();
        let onto_dir = tmp.path().join("ontology");
        fs::create_dir_all(onto_dir.join("src")).unwrap();

        let content = "See [term:Nonexistent] here.\n";
        let issues = check_ontology_terms("CLAUDE.md", content, &onto_dir);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("Nonexistent"));
        assert!(!issues[0].warning);
        assert_eq!(issues[0].line, 1);
    }

    #[test]
    fn no_ontology_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let missing = tmp.path().join("does-not-exist");

        let content = "See [term:Anything] here.\n";
        let issues = check_ontology_terms("CLAUDE.md", content, &missing);
        assert!(issues.is_empty());
    }

    #[test]
    fn multiple_terms_on_same_line() {
        let tmp = TempDir::new().unwrap();
        let onto_dir = tmp.path().join("ontology");
        fs::create_dir_all(onto_dir.join("src")).unwrap();
        fs::write(onto_dir.join("src/scope.md"), "# Scope\n").unwrap();

        let content = "Both [term:Scope] and [term:Missing] appear.\n";
        let issues = check_ontology_terms("AGENTS.md", content, &onto_dir);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("Missing"));
    }

    #[test]
    fn case_insensitive_file_lookup() {
        let tmp = TempDir::new().unwrap();
        let onto_dir = tmp.path().join("ontology");
        fs::create_dir_all(onto_dir.join("src")).unwrap();
        fs::write(onto_dir.join("src/context.md"), "# Context\n").unwrap();

        let content = "See [term:Context] here.\n";
        let issues = check_ontology_terms("CLAUDE.md", content, &onto_dir);
        assert!(issues.is_empty());
    }
}
