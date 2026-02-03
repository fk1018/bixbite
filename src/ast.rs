use crate::types::TypeRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationUnit {
    pub source: String,
    pub typed_methods: Vec<TypedMethod>,
}

impl CompilationUnit {
    pub fn from_source(source: String) -> Self {
        Self {
            source,
            typed_methods: Vec::new(),
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
