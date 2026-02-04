# AGENTS.md

This project uses AI coding agents.

Source of truth:

- `project_handoff.md` defines what Bixbite is and what to build.

Rules:

- Do not expand scope beyond the spec.
- Prefer correctness and clarity over cleverness.
- Keep MVP v0.1 small.
- Follow Rust idioms and project structure.
- Add tests for new behavior.

Workflow:

- Read `project_handoff.md` before making changes.
- Explain what you changed and why.
- Provide run instructions after changes.

## Architecture rules (do not violate)

Bixbite is designed so the backend can evolve without rewriting the frontend.

- The AST and parser must remain backend-agnostic.
- Core types must remain neutral:
  - Use `TypeRef` (e.g., Path, Boolean) in AST — never embed backend-specific syntax in AST nodes.
- All errors must flow through `Diagnostic` with spans and support JSON output (`--format json`).

If you need a new backend later, it should be added by implementing the `Emitter` and/or `TypeChecker` traits without changing the parser/AST.
