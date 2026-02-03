use anyhow::Result;

use crate::{diagnostic::DiagnosticReport, project::Project};

use super::TypeChecker;

#[derive(Debug, Clone, Copy, Default)]
pub struct NoopChecker;

impl TypeChecker for NoopChecker {
    fn name(&self) -> &'static str {
        "noop"
    }

    fn check(&self, _project: &Project) -> Result<DiagnosticReport> {
        Ok(DiagnosticReport::default())
    }
}
