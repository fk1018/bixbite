# TASKS

## MVP v0.1

- [x] CLI commands for `bixbite build` and `bixbite check`
- [x] Project config loading from `bixbite.toml` or `bixbite.json`
- [x] Recursive `.bixb` file discovery under the configured source root
- [x] Lexer with span tracking and structured diagnostics
- [x] Parser for typed single-line method signatures
- [x] Mixed typed/untyped parameter validation for typed methods
- [x] Ruby emitter that removes type annotations from `def` lines
- [x] Generated file header with normalized source paths
- [x] Human-readable and JSON diagnostic rendering
- [x] Build output write-only-if-changed behavior
- [x] Tests covering config parsing, file discovery, lexer/parser behavior, emitter output, build flow, and diagnostic formatting

## Next Candidates

- [ ] Replace the noop checker with a real static checking backend
- [ ] Add fixture-based end-to-end command tests once a Rust toolchain is available in CI
- [ ] Start v0.2 features only after the current MVP behavior is stable
