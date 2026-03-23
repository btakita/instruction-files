---
project_type: rust-lib
publication_targets:
  - crates.io
secret_paths:
  crates_io: "btak/CARGO_REGISTRY_TOKEN"
---

# instruction-files — Release Notes

## Dependency: agent-doc depends on this crate

`instruction-files` is in the dependency chain: `instruction-files → agent-doc → corky`.
When releasing a new version, check if `agent-doc` needs its dependency bumped.

## No Makefile

This crate has no Makefile. Use `cargo clippy && cargo test` for checks and
`cargo publish` for release.

## No audit-docs

This crate provides the audit-docs functionality to other crates but does not
run audit-docs on itself (no `make precommit`).
