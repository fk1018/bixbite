#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeRef {
    pub segments: Vec<String>,
}

impl TypeRef {
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    pub fn from_path(path: &str) -> Self {
        let segments = path
            .split("::")
            .filter(|segment| !segment.is_empty())
            .map(str::to_owned)
            .collect();
        Self { segments }
    }
}
