use std::process::Command;

use anyhow::Result;

use crate::{
    diagnostic::{Diagnostic, DiagnosticReport, Severity, Span},
    project::Project,
};

use super::TypeChecker;

#[derive(Debug, Clone, Copy, Default)]
pub struct SorbetChecker;

impl TypeChecker for SorbetChecker {
    fn name(&self) -> &'static str {
        "sorbet"
    }

    fn check(&self, project: &Project) -> Result<DiagnosticReport> {
        let command_output = Command::new("srb")
            .arg("tc")
            .current_dir(project.root())
            .output();

        let report = match command_output {
            Ok(output) if output.status.success() => DiagnosticReport::default(),
            Ok(output) => {
                let message = first_non_empty(&[
                    String::from_utf8_lossy(&output.stderr).to_string(),
                    String::from_utf8_lossy(&output.stdout).to_string(),
                    format!("`srb tc` failed with status {}", output.status),
                ]);

                DiagnosticReport {
                    diagnostics: vec![Diagnostic {
                        code: "BIX900".to_owned(),
                        severity: Severity::Error,
                        file: project.config.out_dir.to_string_lossy().to_string(),
                        message: format!("Sorbet checker failed: {}", message.trim()),
                        span: Span::point(1, 1),
                        suggestion: Some("Run `srb tc` directly for full details.".to_owned()),
                    }],
                }
            }
            Err(error) => DiagnosticReport {
                diagnostics: vec![Diagnostic {
                    code: "BIX901".to_owned(),
                    severity: Severity::Error,
                    file: project.config.out_dir.to_string_lossy().to_string(),
                    message: format!("Failed to invoke `srb tc`: {error}"),
                    span: Span::point(1, 1),
                    suggestion: Some("Install Sorbet or run without `--sorbet`.".to_owned()),
                }],
            },
        };

        Ok(report)
    }
}

fn first_non_empty(candidates: &[String]) -> String {
    candidates
        .iter()
        .find_map(|value| {
            if value.trim().is_empty() {
                None
            } else {
                Some(value.clone())
            }
        })
        .unwrap_or_default()
}
