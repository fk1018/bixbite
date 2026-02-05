/// Core type reference used in the AST and typed IR.
///
/// Invariants:
/// - `Path` segments are non-empty, trimmed identifiers in source order.
/// - `Boolean` represents the special-case `Boolean` type allowed in `.bixb` source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    /// A constant path like `String` or `Foo::Bar`.
    Path(Vec<String>),
    /// Special-case Boolean type allowed in `.bixb` source.
    Boolean,
}

impl TypeRef {
    /// Creates a path type from explicit segments.
    pub fn path(segments: Vec<String>) -> Self {
        Self::Path(segments)
    }

    /// Parses a type reference from a `::`-separated path string.
    ///
    /// Returns `None` if the input is empty or does not contain any non-empty segments.
    /// The string `"Boolean"` (after trimming) is treated as the special Boolean type.
    pub fn try_from_path(path: &str) -> Option<Self> {
        if path.trim() == "Boolean" {
            return Some(Self::Boolean);
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
    use super::TypeRef;

    #[test]
    fn test_try_from_path_trims_segments() {
        assert_eq!(
            TypeRef::try_from_path(" Integer "),
            Some(TypeRef::Path(vec!["Integer".into()]))
        );
        assert_eq!(
            TypeRef::try_from_path("::  ::Integer::"),
            Some(TypeRef::Path(vec!["Integer".into()]))
        );
        assert_eq!(TypeRef::try_from_path(":: ::"), None);
    }

    #[test]
    fn test_try_from_path_boolean_trim() {
        assert_eq!(TypeRef::try_from_path(" Boolean "), Some(TypeRef::Boolean));
    }
}
