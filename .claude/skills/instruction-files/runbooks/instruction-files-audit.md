# Instruction Files Audit Runbook

Step-by-step procedure for auditing instruction files in a project.

## Prerequisites

- `instruction-files` crate available (as dependency or standalone)
- Working directory is at or below the project root

## Procedure

### 1. Discover files

The audit automatically discovers instruction files by scanning:
- Root-level: `AGENTS.md`, `README.md`, `SPEC.md`, `CLAUDE.md`
- Glob patterns: `.claude/**/SKILL.md`, `.agent/runbooks/*.md`, `src/**/AGENTS.md`

### 2. Run checks

For each discovered file, the audit runs:

- [ ] **Tree paths**: Do all paths in `## Project Structure` blocks exist?
- [ ] **Actionable content**: Are agent files (AGENTS.md, SKILL.md, CLAUDE.md) actionable, not just informational?
- [ ] **Context invariant**: Are there machine-local paths (`~/`, `/home/user/`) that should be repo-relative?

Across all files:

- [ ] **Line budget**: Combined agent instruction files under 1000 lines?
- [ ] **Staleness**: Are instruction files newer than the latest source change?

### 3. Resolve issues

| Issue | Resolution |
|-------|-----------|
| Missing tree path | Remove from structure block or create the file |
| Informational section | Move to README.md, keep only actionable content in AGENTS.md |
| Large code block | Add imperative context ("Use this pattern:") or move to README.md |
| Machine-local path | Replace with repo-relative path or remove |
| Over line budget | Split content: reference docs to README.md, details to SPEC.md |
| Stale instruction file | Review and touch, or update content to match source changes |

### 4. Verify

Re-run the audit to confirm zero issues:

```bash
# Should exit 0 with "No issues found"
cargo run -- audit  # or however the project invokes it
```
