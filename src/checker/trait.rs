use anyhow::Result;

use crate::{diagnostic::DiagnosticReport, project::Project};

pub trait TypeChecker {
    fn name(&self) -> &'static str;
    fn check(&self, project: &Project) -> Result<DiagnosticReport>;
}
