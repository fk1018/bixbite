# AGENTS.md

This project uses AI coding agents.

Source of truth:
- `project_handoff.md` defines what Bixbite is and what to build.

## Rules

- Do not expand scope beyond the spec.
- Prefer correctness and clarity over cleverness.
- Keep MVP v0.1 small.
- Follow Rust idioms and the existing project structure.
- Add or update tests when behavior changes.
- Keep output deterministic across platforms (Windows path separators, newline conventions, etc.).

## Architecture rules (do not violate)

- The AST and parser must remain backend-agnostic.
- Core language types must remain neutral:
  - Use `TypeRef` (e.g., Path, Boolean) in AST and typed IR.
  - Do not embed backend-specific syntax or formatting into AST nodes.
- All errors must flow through `Diagnostic` with spans.
- Diagnostics must support machine-readable output (`--format json`) and human output.

If you need a new backend later, it should be added by implementing the `Emitter` and/or `TypeChecker` traits without changing the parser/AST.

## Documentation rules

Write professional Rust documentation focused on public APIs and invariants.

Minimum standard:
- Every `pub` struct/enum/trait/function in `src/` must have a `///` rustdoc comment explaining:
  - what it represents/does,
  - important invariants (what is guaranteed to be present/valid),
  - and any error/recovery behavior (especially in parsing/checking).
- Public data types should document fields either:
  - with `///` on each field, or
  - clearly in the type-level doc comment.
- Avoid noise: do not comment obvious code. Prefer documenting “why” and “invariants” over “what the line does.”
- Rustdoc examples must be consistent:
  - Use real doctests only if they compile in this crate.
  - Otherwise use ```ignore``` with a clear illustrative snippet.

Formatting and quality:
- Run `cargo fmt` after changes.
- Keep documentation accurate when behavior changes.

## Workflow

- Read `project_handoff.md` before making changes.
- Explain what you changed and why.
- Provide run instructions after changes (e.g., `cargo fmt`, `cargo test`).
