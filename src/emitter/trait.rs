use std::path::Path;

use crate::ast::CompilationUnit;

/// Backend that emits target language output from a parsed compilation unit.
///
/// Implementations must be deterministic: the same input should always produce
/// identical output.
pub trait Emitter {
    /// Emits output for the given compilation unit and source path.
    ///
    /// `source_path` should be the project-relative path used for headers.
    fn emit(&self, unit: &CompilationUnit, source_path: &Path) -> String;
}
