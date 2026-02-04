/// Abstract syntax tree (AST) types for parsed Bixbite source.
pub mod ast;
/// Type-checker backends and related traits.
pub mod checker;
/// CLI command implementations.
pub mod commands;
/// Diagnostic types and reporting helpers.
pub mod diagnostic;
/// Code emitter backends.
pub mod emitter;
/// Tokenizer/lexer for Bixbite source.
pub mod lexer;
/// Parser for token streams into AST structures.
pub mod parser;
/// Project discovery, config loading, and file layout helpers.
pub mod project;
/// Core language type references used in AST/IR.
pub mod types;
