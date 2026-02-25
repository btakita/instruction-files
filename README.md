# instruction-files

Discovery, auditing, and sync for AI agent instruction files (`AGENTS.md`, `CLAUDE.md`, `SKILL.md`).

## What it does

AI coding tools like Claude Code use markdown instruction files to configure agent behavior per-project. These files drift out of sync as code evolves. `instruction-files` catches that drift automatically.

**Checks:**

| Check | What it catches |
|---|---|
| **Staleness** | Instruction files older than source code they describe |
| **Tree paths** | `## Project Structure` blocks referencing files/dirs that don't exist |
| **Line budget** | Combined instruction files exceeding 1000 lines (context window pressure) |
| **Actionable content** | Large code blocks or tables without imperative context (copy-paste, not instructions) |

## Usage

```rust
use instruction_files::{AuditConfig, run};

// Use a preset config
let config = AuditConfig::agent_doc();  // broad: many languages, many root markers
// or
let config = AuditConfig::corky();      // narrow: Rust-only, Cargo.toml root

// Run the full audit
run(&config, None)?;
```

### Custom config

```rust
let config = AuditConfig {
    root_markers: vec!["Cargo.toml", "package.json"],
    include_claude_md: true,
    source_extensions: vec!["rs", "ts", "py"],
    source_dirs: vec!["src", "lib"],
    skip_dirs: vec!["target", "node_modules", ".git"],
};
```

### Individual checks

```rust
use instruction_files::*;

let root = find_root(&config);
let files = find_instruction_files(&root, &config);

// Run checks individually
let issues = check_staleness(&files, &root, &config);
let issues = check_tree_paths("CLAUDE.md", &content, &root);
let issues = check_actionable("AGENTS.md", &content, &config);
let (issues, counts, total) = check_line_budget(&files, &root);
```

## File discovery

Searches for instruction files in standard locations:

- **Root level:** `AGENTS.md`, `README.md`, optionally `CLAUDE.md`
- **Skills:** `.claude/**/SKILL.md`, `.agents/**/SKILL.md`
- **Package level:** `.agents/**/AGENTS.md`, `src/**/AGENTS.md`

Project root is found by walking up from CWD, checking for marker files (`Cargo.toml`, `package.json`, etc.), then `.git`, then falling back to CWD.

## Used by

- [**agent-doc**](https://github.com/btakita/agent-doc) -- Interactive document sessions with AI agents. `agent-doc audit-docs` delegates to this crate with `AuditConfig::agent_doc()`.
- [**corky**](https://github.com/btakita/corky) -- Email sync and draft management. `corky audit-docs` delegates with `AuditConfig::corky()`.

## Install

```toml
[dependencies]
instruction-files = "0.1"
```

## License

MIT
