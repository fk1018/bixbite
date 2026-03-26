/// Bixbite-owned primitive types accepted directly in typed signatures.
///
/// Invariants:
/// - Each variant corresponds to a single builtin source spelling.
/// - Primitive names are backend-agnostic and do not imply a Ruby runtime constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    /// Integer primitive written as `Int` in source.
    Int,
    /// String primitive written as `Str` in source.
    Str,
    /// Boolean primitive written as `Bool` in source.
    Bool,
}

impl PrimitiveType {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name.trim() {
            "Int" => Some(Self::Int),
            "Str" => Some(Self::Str),
            "Bool" => Some(Self::Bool),
            _ => None,
        }
    }
}

/// Core type reference used in the AST and typed IR.
///
/// Invariants:
/// - `Path` segments are non-empty, trimmed identifiers in source order.
/// - `Primitive` stores one of the Bixbite builtin types accepted in `.bixb` source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    /// A Bixbite builtin primitive like `Int`, `Str`, or `Bool`.
    Primitive(PrimitiveType),
    /// A constant path like `Foo::Bar` or `TrueClass`.
    Path(Vec<String>),
}

impl TypeRef {
    /// Creates a path type from explicit segments.
    pub fn path(segments: Vec<String>) -> Self {
        Self::Path(segments)
    }

    /// Parses a type reference from a `::`-separated path string.
    ///
    /// Returns `None` if the input is empty or does not contain any non-empty segments.
    /// Exact builtin spellings `Int`, `Str`, and `Bool` (after trimming) become primitives.
    pub fn try_from_path(path: &str) -> Option<Self> {
        if let Some(primitive) = PrimitiveType::from_name(path) {
            return Some(Self::Primitive(primitive));
        }

        let segments: Vec<String> = path
            .split("::")
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .map(str::to_owned)
            .collect();

        if segments.is_empty() {
            return None;
        }

        Some(Self::Path(segments))
    }

    /// Parses a type reference from a `::`-separated path string.
    ///
    /// # Panics
    /// Panics if the input is invalid (empty or contains only separators).
    pub fn from_path(path: &str) -> Self {
        Self::try_from_path(path)
            .expect("invalid type path: expected at least one non-empty segment")
    }
}

#[cfg(test)]
mod tests {
    use super::{PrimitiveType, TypeRef};

    #[test]
    fn test_try_from_path_recognizes_primitives() {
        assert_eq!(
            TypeRef::try_from_path(" Int "),
            Some(TypeRef::Primitive(PrimitiveType::Int))
        );
        assert_eq!(
            TypeRef::try_from_path(" Str "),
            Some(TypeRef::Primitive(PrimitiveType::Str))
        );
        assert_eq!(
            TypeRef::try_from_path(" Bool "),
            Some(TypeRef::Primitive(PrimitiveType::Bool))
        );
    }

    #[test]
    fn test_try_from_path_trims_segments() {
        assert_eq!(
            TypeRef::try_from_path(" Foo::Bar "),
            Some(TypeRef::Path(vec!["Foo".into(), "Bar".into()]))
        );
        assert_eq!(
            TypeRef::try_from_path("::  ::String::"),
            Some(TypeRef::Path(vec!["String".into()]))
        );
        assert_eq!(TypeRef::try_from_path(":: ::"), None);
    }
}
