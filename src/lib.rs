//! Discovery, auditing, and sync for AGENTS.md/CLAUDE.md instruction files.

mod audit;
mod discovery;
mod types;

pub use audit::{check_actionable, check_line_budget, check_staleness, check_tree_paths};
pub use discovery::{find_instruction_files, find_root};
pub use types::{AuditConfig, Issue};

use anyhow::Result;
use std::path::Path;

/// Run the full audit with the given configuration.
///
/// Returns `Ok(())` on success, calls `std::process::exit(1)` on issues found.
pub fn run(config: &AuditConfig, root_override: Option<&Path>) -> Result<()> {
    println!("Auditing docs...\n");

    let root = match root_override {
        Some(p) => p.to_path_buf(),
        None => find_root(config),
    };
    let files = find_instruction_files(&root, config);
    let mut issues: Vec<Issue> = Vec::new();

    for doc in &files {
        let rel = doc
            .strip_prefix(&root)
            .unwrap_or(doc)
            .to_string_lossy()
            .to_string();
        if let Ok(content) = std::fs::read_to_string(doc) {
            issues.extend(check_tree_paths(&rel, &content, &root));
            issues.extend(check_actionable(&rel, &content, config));
        }
    }

    let (budget_issues, counts, total) = check_line_budget(&files, &root, config);
    issues.extend(budget_issues);
    issues.extend(check_staleness(&files, &root, config));

    for issue in &issues {
        let mut loc = format!("  {}", issue.file);
        if issue.line > 0 {
            if issue.end_line > issue.line {
                loc.push_str(&format!(":{}-{}", issue.line, issue.end_line));
            } else {
                loc.push_str(&format!(":{}", issue.line));
            }
        }
        let marker = if issue.warning { "\u{26a0}" } else { "\u{2717}" };
        println!("{:<50} {} {}", loc, marker, issue.message);
    }

    let mark = if total <= LINE_BUDGET {
        "\u{2713}"
    } else {
        "\u{2717}"
    };
    println!(
        "\nCombined instruction files: {} lines (budget: {}) {}",
        total, LINE_BUDGET, mark
    );
    for (name, n) in &counts {
        println!("  {}: {}", name, n);
    }

    let n = issues.len();
    if n > 0 {
        println!("\nFound {} issue(s)", n);
        std::process::exit(1);
    } else {
        println!("\nNo issues found \u{2713}");
    }

    Ok(())
}

const LINE_BUDGET: usize = 1000;
