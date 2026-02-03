# TASKS

## Milestone 0: Scaffold

- [ ] CLI (`bixbite build`)
- [ ] Config loading (bixbite.toml)
- [ ] File discovery (.bixb in src/)
- [ ] Output to build/

## Milestone 1: Lexer + AST

- [ ] Tokenizer for def signatures
- [ ] AST for method signatures
- [ ] Span tracking (line/col)

## Milestone 2: Parser

- [ ] Parse typed method signatures
- [ ] Error on mixed typed/untyped params

## Milestone 3: Emitter

- [ ] Emit Ruby + Sorbet sigs
- [ ] Add generated file header

## Milestone 4: Diagnostics

- [ ] Human-readable errors
- [ ] JSON diagnostics

## Milestone 5: Sorbet integration

- [ ] `bixbite check --sorbet`
