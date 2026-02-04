/// Backend-neutral reference to a declared type.
///
/// Invariants:
/// - `Path` segments are non-empty and represent Ruby constant names.
/// - `Boolean` is a special-case alias for the `.bixb`-only Boolean type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeRef {
    /// A Ruby constant path such as `Foo::Bar`.
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
    /// The string `"Boolean"` is treated as the special Boolean type.
    pub fn from_path(path: &str) -> Self {
        if path.trim() == "Boolean" {
            return Self::Boolean;
        }
        let segments = path
            .split("::")
            .filter(|segment| !segment.is_empty())
            .map(str::to_owned)
            .collect();
        Self::Path(segments)
    }
}
