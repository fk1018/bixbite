# AGENTS.md

This project uses AI coding agents.

Source of truth:
- `README.md` defines the current MVP behavior and user-facing contract.
- `TASKS.md` tracks completion status and next work.

## Rules

- Do not expand scope beyond the spec.
- Prefer correctness and clarity over cleverness.
- Keep MVP v0.1 small.
- Follow Rust idioms and the existing project structure.
- Add or update tests when behavior changes.
- Keep output deterministic across platforms (Windows path separators, newline conventions, etc.).

## Architecture rules (do not violate)

- Keep the AST and parser backend-agnostic.
- Core language types remain neutral:
  - Use `TypeRef` (e.g., `Path`, `Boolean`) in AST and typed IR.
  - Do not embed backend-specific syntax or formatting in AST nodes.
- All errors must flow through `Diagnostic` with spans.
- Diagnostics must support both machine-readable output (`--format json`) and human output.
- New backends must be added by implementing the `Emitter` and/or `TypeChecker` traits without changing the parser/AST.

## Documentation rules

Write professional Rust documentation focused on public APIs, invariants, and recovery behavior.

Minimum standard:
- Every `pub` struct/enum/trait/function in `src/` must have a `///` rustdoc comment that explains:
  - what it represents/does,
  - key invariants/guarantees (what is always present/valid),
  - and error/recovery behavior where relevant (especially parsing/checking/diagnostics/project discovery).
- Public data types should document fields either:
  - with `///` on each field, or
  - clearly in the type-level doc comment.
- Avoid noise: do not comment obvious code. Prefer documenting “why” and “invariants” over line-by-line narration.
- Rustdoc examples must be consistent:
  - Use real doctests only if they compile in this crate.
  - Otherwise use ```ignore``` with a short illustrative snippet.

Formatting and quality:
- Run Rust tooling in Docker, not on the host machine.
- Prefer the repo's Docker Compose service over ad hoc containers.
- Preferred pattern: `docker compose run --rm bixbite cargo <subcommand>`.
- Use `docker compose build bixbite` after Docker-related changes or before the first Rust command on a new machine.
- Run `cargo fmt` after changes.
- Keep documentation accurate when behavior changes.

## Workflow

- Read `README.md` and `TASKS.md` before making changes.
- Explain what you changed and why.
- Provide run instructions after changes using Docker-based commands.
