#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeRef {
    Path(Vec<String>),
    Boolean,
}

impl TypeRef {
    pub fn path(segments: Vec<String>) -> Self {
        Self::Path(segments)
    }

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
