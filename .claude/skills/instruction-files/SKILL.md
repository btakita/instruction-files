# instruction-files

Audit and manage instruction files (AGENTS.md, CLAUDE.md, SKILL.md, runbooks).

## Invocation

```
/instruction-files audit [--fix]
/instruction-files init
```

- `/instruction-files audit` — run the full audit suite against all discovered instruction files
- `/instruction-files audit --fix` — fix auto-fixable issues (staleness touch, budget hints)
- `/instruction-files init` — scaffold `.agent/runbooks/` with bundled defaults

## Audit Checks

The audit discovers and validates these file types:

| Pattern | Description |
|---------|-------------|
| `AGENTS.md`, `README.md`, `SPEC.md` | Root-level project docs |
| `CLAUDE.md` | Per-directory agent instructions (if `include_claude_md`) |
| `.claude/**/SKILL.md` | Claude Code skill definitions |
| `.agent/runbooks/*.md` | Project runbooks |
| `.claude/skills/**/runbooks/*.md` | Skill-specific runbooks |
| `src/**/AGENTS.md` | Submodule/package instructions |

### Check Suite

1. **Tree paths** — Verify paths in `## Project Structure` blocks exist on disk
2. **Actionable content** — Flag informational-only sections, large code blocks without imperative context, oversized tables, link-heavy lists
3. **Line budget** — Combined agent instruction files must stay under 1000 lines
4. **Staleness** — Flag instruction files older than the newest source file
5. **Context invariant** — Flag machine-local paths (`~/`, `/home/user/`, `/Users/user/`) that won't resolve on other machines

### Running the Audit

From the project root:

```bash
# Programmatic (Rust)
use instruction_files::{AuditConfig, run};
run(&AuditConfig::agent_doc(), None).unwrap();

# Via agent-doc's precommit
make check  # includes instruction-files audit
```

## Init Runbooks

`init_runbooks(root)` scaffolds `.agent/runbooks/` with bundled defaults:
- `precommit.md` — standard precommit checklist
- `prerelease.md` — standard prerelease checklist

Never overwrites existing files. Safe to run repeatedly.

## Runbooks

- [Instruction Files Audit](runbooks/instruction-files-audit.md) — step-by-step audit procedure
