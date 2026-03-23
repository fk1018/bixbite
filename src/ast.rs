use std::ops::Range;

use crate::{diagnostic::DiagnosticReport, types::TypeRef};

/// Parsed representation of a single source file plus any diagnostics found.
///
/// Invariants:
/// - `source` is the original source text.
/// - `typed_methods` only contains methods with fully-typed parameters in source order.
/// - `diagnostics` includes all parser errors and recovery notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationUnit {
    /// Original source text for the file.
    pub source: String,
    /// Methods that successfully parsed with complete type annotations.
    pub typed_methods: Vec<TypedMethod>,
    /// Diagnostics captured during parsing.
    pub diagnostics: DiagnosticReport,
}

impl CompilationUnit {
    /// Constructs a compilation unit from its parsed components.
    ///
    /// This does not validate or modify the input; callers are responsible for ensuring
    /// `typed_methods` only includes well-formed signatures and diagnostics cover errors.
    pub fn from_source(
        source: String,
        typed_methods: Vec<TypedMethod>,
        diagnostics: DiagnosticReport,
    ) -> Self {
        Self {
            source,
            typed_methods,
            diagnostics,
        }
    }
}

/// A parsed method signature with fully-typed parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedMethod {
    /// Method name, including optional `self.` prefix.
    pub name: String,
    /// Parameters with required types and optional defaults.
    pub params: Vec<TypedParam>,
    /// Declared return type for the method.
    pub return_type: TypeRef,
    /// Byte range within `CompilationUnit::source` covering the original signature text.
    ///
    /// This range starts at the `def` keyword and ends at the final return-type token.
    pub signature_byte_range: Range<usize>,
}

/// A parsed method parameter that carries a required type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedParam {
    /// Parameter identifier as written in source.
    pub name: String,
    /// Declared type reference for the parameter.
    pub type_ref: TypeRef,
    /// Default expression text, if present.
    pub default: Option<String>,
}
