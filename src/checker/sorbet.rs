use std::{io::Read, process::Command, time::Duration};

use anyhow::Result;
use wait_timeout::ChildExt;

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
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        let report = match command_output {
            Ok(mut child) => {
                let stdout = child.stdout.take();
                let stderr = child.stderr.take();
                let timeout = Duration::from_secs(30);
                match child.wait_timeout(timeout)? {
                    Some(status) if status.success() => DiagnosticReport::default(),
                    Some(status) => {
                        let mut out = String::new();
                        let mut err = String::new();
                        if let Some(mut handle) = stdout {
                            let _ = handle.read_to_string(&mut out);
                        }
                        if let Some(mut handle) = stderr {
                            let _ = handle.read_to_string(&mut err);
                        }
                        let message = first_non_empty(&[
                            err,
                            out,
                            format!("`srb tc` failed with status {status}"),
                        ]);

                        DiagnosticReport {
                            diagnostics: vec![Diagnostic {
                                code: "BIX900".to_owned(),
                                severity: Severity::Error,
                                file: "<sorbet>".to_owned(),
                                message: format!("Sorbet checker failed: {}", message.trim()),
                                span: Span::point(1, 1),
                                suggestion: Some(
                                    "Run `srb tc` directly for full details.".to_owned(),
                                ),
                            }],
                        }
                    }
                    None => {
                        let _ = child.kill();
                        let _ = child.wait();
                        DiagnosticReport {
                            diagnostics: vec![Diagnostic {
                                code: "BIX902".to_owned(),
                                severity: Severity::Error,
                                file: "<sorbet>".to_owned(),
                                message: "Sorbet checker timed out.".to_owned(),
                                span: Span::point(1, 1),
                                suggestion: Some(
                                    "Re-run with `srb tc` directly to investigate.".to_owned(),
                                ),
                            }],
                        }
                    }
                }
            }
            Err(error) => DiagnosticReport {
                diagnostics: vec![Diagnostic {
                    code: "BIX901".to_owned(),
                    severity: Severity::Error,
                    file: "<sorbet>".to_owned(),
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
        .find(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_default()
}
