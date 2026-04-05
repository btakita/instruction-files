#![cfg(feature = "spec-audit")]
//! Spec audit: validate SPEC.md files for required sections.
//!
//! Behind the `spec-audit` feature gate. Uses the shared [`Issue`] type
//! from `agent_kit::audit_common` (re-exported via `crate::types`).

/// An issue found during spec auditing.
///
/// Mirrors the shape of `agent_kit::audit_common::Issue` but is
/// self-contained so this module compiles with only `module-harness`
/// (no `agent-kit` re-export needed at the call-site).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecIssue {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub warning: bool,
}

/// Validate a SPEC.md file for required sections.
///
/// Only runs checks when `rel` ends with `SPEC.md` (case-sensitive).
/// Returns an empty vec for non-SPEC files.
///
/// Checks:
/// 1. **H1 title** (error if missing)
/// 2. **`## Agentic Contracts`** section (warning if missing)
/// 3. **`## Evals`** section (warning if missing)
pub fn check_spec(rel: &str, content: &str) -> Vec<SpecIssue> {
    let name = std::path::Path::new(rel)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if name != "SPEC.md" {
        return Vec::new();
    }

    let mut issues = Vec::new();

    // Check for H1 title
    let has_h1 = content.lines().any(|l| l.starts_with("# "));
    if !has_h1 {
        issues.push(SpecIssue {
            file: rel.to_string(),
            line: 1,
            message: "SPEC.md is missing an H1 title".to_string(),
            warning: false,
        });
    }

    // Check for ## Agentic Contracts
    let has_contracts = content
        .lines()
        .any(|l| l.trim() == "## Agentic Contracts");
    if !has_contracts {
        issues.push(SpecIssue {
            file: rel.to_string(),
            line: 0,
            message: "SPEC.md is missing `## Agentic Contracts` section".to_string(),
            warning: true,
        });
    }

    // Check for ## Evals
    let has_evals = content.lines().any(|l| l.trim() == "## Evals");
    if !has_evals {
        issues.push(SpecIssue {
            file: rel.to_string(),
            line: 0,
            message: "SPEC.md is missing `## Evals` section".to_string(),
            warning: true,
        });
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_with_all_sections_no_issues() {
        let content = "\
# My Module

Overview of the module.

## Agentic Contracts

- Contract A
- Contract B

## Evals

- Eval suite 1
";
        let issues = check_spec("src/foo/SPEC.md", content);
        assert!(issues.is_empty(), "Expected no issues, got: {:?}", issues);
    }

    #[test]
    fn spec_missing_contracts_warns() {
        let content = "\
# My Module

Overview.

## Evals

- Eval suite 1
";
        let issues = check_spec("SPEC.md", content);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].warning);
        assert!(issues[0].message.contains("Agentic Contracts"));
    }

    #[test]
    fn spec_missing_contracts_and_evals_two_warnings() {
        let content = "\
# My Module

Just a title and some text.
";
        let issues = check_spec("SPEC.md", content);
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().all(|i| i.warning));
        assert!(issues[0].message.contains("Agentic Contracts"));
        assert!(issues[1].message.contains("Evals"));
    }

    #[test]
    fn spec_missing_h1_is_error() {
        let content = "\
## Agentic Contracts

Some contracts.

## Evals

Some evals.
";
        let issues = check_spec("SPEC.md", content);
        assert_eq!(issues.len(), 1);
        assert!(!issues[0].warning, "Missing H1 should be an error, not a warning");
        assert!(issues[0].message.contains("H1 title"));
    }

    #[test]
    fn non_spec_file_returns_empty() {
        let content = "# README\n\nSome content.\n";
        let issues = check_spec("README.md", content);
        assert!(issues.is_empty());

        let issues2 = check_spec("src/AGENTS.md", "# Agents\n");
        assert!(issues2.is_empty());
    }
}
