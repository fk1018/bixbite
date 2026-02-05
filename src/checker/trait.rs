use anyhow::Result;

use crate::{diagnostic::DiagnosticReport, project::Project};

/// Backend that performs static type checks over a project.
pub trait TypeChecker {
    /// Returns the short backend name used in CLI output.
    fn name(&self) -> &'static str;
    /// Runs type checking and returns any diagnostics.
    fn check(&self, project: &Project) -> Result<DiagnosticReport>;
}
