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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Human,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckOptions {
    pub format: OutputFormat,
}

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
    if report.diagnostics.is_empty() {
        return Ok(());
    }

    match format {
        OutputFormat::Human => {
            report.print_human_stderr();
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(report)?);
        }
    }

    Ok(())
}
