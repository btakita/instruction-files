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
| **Context invariant** | Machine-local paths (`~/`, `/home/user/`) that won't resolve on other machines |

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

- **Root level:** `AGENTS.md`, `README.md`, `SPEC.md`, optionally `CLAUDE.md`
- **Skills:** `.claude/**/SKILL.md`, `.agents/**/SKILL.md`
- **Runbooks:** `.agent/runbooks/*.md`, `.claude/skills/**/runbooks/*.md`
- **Package level:** `.agents/**/AGENTS.md`, `src/**/AGENTS.md`

Project root is found by walking up from CWD, checking for marker files (`Cargo.toml`, `package.json`, etc.), then `.git`, then falling back to CWD.

## Used by

- [**agent-doc**](https://github.com/btakita/agent-doc) -- Interactive document sessions with AI agents. `agent-doc audit-docs` delegates to this crate with `AuditConfig::agent_doc()`.
- [**corky**](https://github.com/btakita/corky) -- Email sync and draft management. `corky audit-docs` delegates with `AuditConfig::corky()`.

## Architecture

instruction-files is the central audit crate. It integrates with two companion specs via optional features:

```
instruction-files (core: discovery + audit)
├── [ontology] existence crate — validate ontology terms in instruction files
└── [spec-audit] module-harness crate — audit module-level specs and contracts
```

**Bundled runbooks:** The crate embeds generic `precommit.md` and `prerelease.md` runbooks via `include_str!`. `init_runbooks(root)` scaffolds `.agent/runbooks/` with these defaults (never overwrites).

**Companion specs** (external repos):
- **[agent-runbooks](https://github.com/btakita/agent-runbooks)** — convention for externalizing procedures into on-demand runbook files
- **[agent-memories](https://github.com/btakita/agent-memories)** — convention for committed memories (type, scope, why, how to apply)
- **[agent-rules](https://github.com/btakita/agent-rules)** — convention for prescribed policy in instruction files
- **[skill-harness](https://github.com/btakita/skill-harness)** — lifecycle management for AI agent skills (install, audit, eval)

instruction-files discovers and validates files in the formats these specs define.

## Install

### Quick install (prebuilt binary)

```bash
curl -fsSL https://raw.githubusercontent.com/btakita/instruction-files/main/install.sh | sh
```

### Cargo

```bash
cargo install instruction-files
```

### As a library dependency

```toml
[dependencies]
instruction-files = "0.2"

# Optional integrations
instruction-files = { version = "0.2", features = ["ontology", "spec-audit"] }
```

## License

MIT
