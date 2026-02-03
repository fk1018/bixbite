use std::path::Path;

use crate::ast::CompilationUnit;

pub trait Emitter {
    fn emit(&self, unit: &CompilationUnit, source_path: &Path) -> String;
}
