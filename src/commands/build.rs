use std::{env, fs, path::Path};

use anyhow::{Context, Result};

use crate::{
    diagnostic::Severity,
    emitter::{ruby::RubyEmitter, Emitter},
    lexer, parser,
    project::Project,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildSummary {
    pub discovered_files: usize,
    pub written_files: usize,
}

pub fn run() -> Result<()> {
    let project_root = env::current_dir().context("failed to determine current directory")?;
    let project = Project::load(project_root)?;
    let emitter = RubyEmitter;
    let summary = build_project(&project, &emitter)?;

    println!(
        "Built {} file(s) from {} to {}.",
        summary.written_files,
        project.config.source_dir.to_string_lossy(),
        project.config.out_dir.to_string_lossy()
    );

    Ok(())
}

pub fn build_project(project: &Project, emitter: &dyn Emitter) -> Result<BuildSummary> {
    project.ensure_out_dir()?;
    let source_files = project.discover_sources()?;
    let mut summary = BuildSummary {
        discovered_files: source_files.len(),
        written_files: 0,
    };

    for source_file in source_files {
        if let Some(parent) = source_file.output_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create output subdirectory {}",
                    parent.to_string_lossy()
                )
            })?;
        }

        let source = fs::read_to_string(&source_file.source_path).with_context(|| {
            format!(
                "failed to read source file {}",
                source_file.source_path.to_string_lossy()
            )
        })?;
        let tokens = lexer::tokenize(
            &source,
            source_file.source_path.to_string_lossy().to_string(),
        );
        let ast = parser::parse(tokens);
        if !ast.diagnostics.diagnostics.is_empty() {
            print_diagnostics(&ast.diagnostics);
            if ast.diagnostics.has_errors() {
                anyhow::bail!(
                    "failed to parse {}",
                    source_file.source_path.to_string_lossy()
                );
            }
        }

        let source_for_header: &Path = source_file
            .source_path
            .strip_prefix(project.root())
            .unwrap_or(&source_file.relative_path);
        let emitted = emitter.emit(&ast, source_for_header);

        fs::write(&source_file.output_path, emitted).with_context(|| {
            format!(
                "failed to write {}",
                source_file.output_path.to_string_lossy()
            )
        })?;
        summary.written_files += 1;
    }

    Ok(summary)
}

fn print_diagnostics(report: &crate::diagnostic::DiagnosticReport) {
    for diagnostic in &report.diagnostics {
        let severity = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warn => "warn",
        };
        eprintln!(
            "{}:{}:{}: {} {} ({})",
            diagnostic.file,
            diagnostic.span.start.line,
            diagnostic.span.start.col,
            severity,
            diagnostic.message,
            diagnostic.code
        );
        if let Some(suggestion) = &diagnostic.suggestion {
            eprintln!("  help: {suggestion}");
        }
    }
}
