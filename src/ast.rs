use crate::{diagnostic::DiagnosticReport, types::TypeRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationUnit {
    pub source: String,
    pub typed_methods: Vec<TypedMethod>,
    pub diagnostics: DiagnosticReport,
}

impl CompilationUnit {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedMethod {
    pub name: String,
    pub params: Vec<TypedParam>,
    pub return_type: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedParam {
    pub name: String,
    pub type_ref: TypeRef,
    pub default: Option<String>,
}
