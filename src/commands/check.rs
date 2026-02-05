use std::fmt;

use anyhow::{bail, Result};
use clap::ValueEnum;

use crate::{
    checker::{noop::NoopChecker, TypeChecker},
    diagnostic::DiagnosticReport,
    emitter::ruby::RubyEmitter,
    project::Project,
};

use super::build;

/// Output format for diagnostic reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable diagnostics.
    Human,
    /// Machine-readable JSON diagnostics.
    Json,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Human => f.write_str("human"),
            Self::Json => f.write_str("json"),
        }
    }
}

/// Options for the `bixbite check` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckOptions {
    /// Diagnostic output format.
    pub format: OutputFormat,
}

/// Runs the `bixbite check` command with the provided options.
///
/// This performs a build followed by the configured type checker and emits diagnostics
/// in the requested format.
pub fn run(options: CheckOptions) -> Result<()> {
    let project = Project::load(std::env::current_dir()?)?;
    let emitter = RubyEmitter;
    let summary = build::build_project(&project, &emitter)?;

    let checker: Box<dyn TypeChecker> = Box::new(NoopChecker);
    let report = checker.check(&project)?;

    print_diagnostics(&report, options.format)?;

    if report.has_errors() {
        bail!("`{}` checker reported errors", checker.name());
    }

    match options.format {
        OutputFormat::Human => {
            println!(
                "Checked {} file(s) with {} backend.",
                summary.discovered_files,
                checker.name()
            );
        }
        OutputFormat::Json => {
            eprintln!(
                "Checked {} file(s) with {} backend.",
                summary.discovered_files,
                checker.name()
            );
        }
    }

    Ok(())
}

fn print_diagnostics(report: &DiagnosticReport, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Human => {
            if report.diagnostics.is_empty() {
                return Ok(());
            }
            report.print_human_stderr();
        }
        OutputFormat::Json => {
            println!("{}", json_diagnostics(report)?);
        }
    }

    Ok(())
}

fn json_diagnostics(report: &DiagnosticReport) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_output_includes_empty_report() {
        let report = DiagnosticReport::default();
        let output = json_diagnostics(&report).expect("serialize diagnostics");
        let value: serde_json::Value =
            serde_json::from_str(&output).expect("parse json diagnostics");
        assert_eq!(value["diagnostics"].as_array().map(Vec::len), Some(0));
    }
}
