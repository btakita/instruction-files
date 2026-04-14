# Library Context Policy

Canonical rules for how libraries in the agent-loop workspace (and any library authored under the same convention) expose agent context. This file is the single source of truth. Leaf libraries link to it; they do not copy it.

## Scope

Applies to any library that ships a `.claude/` skill, an AI agent surface, or is consumed by AI coding agents. "Library" here means any unit a developer can extract and use independently — a Rust crate, an npm package, a Python package, a standalone skill.

## Rules

### 1. Every library provides both an `AGENTS.md` and a runbook set

- `AGENTS.md` at the library root describes **what the library is** and **how an agent should reason about it**: contracts, invariants, domain vocabulary, non-obvious decisions. Stable prose.
- `runbooks/` contains **task-shaped procedures**: release, precommit, debugging a specific failure mode, operational playbooks. Each runbook answers one question.
- `CLAUDE.md` is either a symlink to `AGENTS.md` or omitted.

Rationale: stable context (AGENTS.md) and procedural context (runbooks) have different edit cadences, different audiences, and different loading costs. Collapsing them causes AGENTS.md to bloat with procedures that should be on-demand.

### 2. Every library's `AGENTS.md` carries a Library Context Policy pointer

Required section, verbatim format:

```markdown
## Library Context Policy

This library follows the agent-loop library-context policy. Contributors
authoring `AGENTS.md`, `SKILL.md`, or runbooks in this repo must read:

<POLICY_URL>

before making changes.
```

`<POLICY_URL>` resolves to this file. Acceptable forms:

- Relative path for in-workspace libraries: `../instruction-files/LIBRARY_CONTEXT_POLICY.md`
- Absolute GitHub URL for published libraries: `https://github.com/btakita/agent-loop/blob/main/src/instruction-files/LIBRARY_CONTEXT_POLICY.md`

External contributors who clone a leaf library in isolation see the pointer, follow the link, and get the full rule. Drift is impossible — there's only one canonical copy.

### 3. Canonical spec never duplicates into leaf libraries

Do not copy the rule body into individual library AGENTS.md files. If the rule needs to change, it changes here. Leaf stubs are pointers, not caches.

### 4. `instruction-files` audit enforces presence

The `instruction-files` crate audit (run via `make precommit`) verifies every library AGENTS.md contains the `## Library Context Policy` section with a resolvable link. Violations fail precommit. The check is mechanical — no prose matching, just section presence and link validity.

### 5. Scaffolders write the stub for new libraries

The `rust-package` skill (and any other library scaffolder under this workspace) writes the `## Library Context Policy` stub into new library AGENTS.md files at creation time. Existing libraries get the stub via a one-time sweep commit.

## Non-Rules (things this policy does NOT mandate)

- **Content of AGENTS.md** beyond the pointer section. Each library decides what domain knowledge belongs there.
- **Runbook naming** beyond living under `runbooks/`. Each library picks its own conventions.
- **Linter schemas**. `instruction-files` checks presence and link validity; it does not grade prose.

## Amending This Policy

Changes land via PR to this file. The `instruction-files` audit surfaces any breakage across the workspace on the next `make precommit` run.
