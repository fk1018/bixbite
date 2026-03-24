use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};

use crate::{
    diagnostic::{Diagnostic, DiagnosticReport, Span},
    emitter::{ruby::RubyEmitter, Emitter},
    lexer, parser,
    project::{Project, SourceFile},
};

const BUILD_IO_ERROR_CODE: &str = "BIX202";

/// Summary of a build invocation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildSummary {
    /// Number of `.bixb` files discovered under the source root.
    pub discovered_files: usize,
    /// Number of `.rb` files written to the output directory.
    pub written_files: usize,
}

/// Result of building a project, including structured diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildReport {
    /// Summary counts for discovery and output writes.
    pub summary: BuildSummary,
    /// Diagnostics produced while lexing or parsing source files.
    pub diagnostics: DiagnosticReport,
}

/// Runs the `bixbite build` command for the current working directory.
///
/// Errors include failures to resolve the current directory and any build diagnostics
/// with error severity.
pub fn run() -> Result<()> {
    let project_root = env::current_dir().context("failed to determine current directory")?;
    let project = match Project::load(project_root) {
        Ok(project) => project,
        Err(report) => {
            report.print_human_stderr();
            bail!("build failed");
        }
    };
    let emitter = RubyEmitter;
    let report = build_project(&project, &emitter);

    if !report.diagnostics.is_empty() {
        report.diagnostics.print_human_stderr();
    }
    if report.diagnostics.has_errors() {
        bail!("build failed");
    }

    println!(
        "Discovered {} file(s) in {} and wrote {} file(s) to {}.",
        report.summary.discovered_files,
        project.config.source_dir.to_string_lossy(),
        report.summary.written_files,
        project.config.out_dir.to_string_lossy()
    );

    Ok(())
}

/// Builds all source files in the project using the provided emitter.
///
/// Diagnostics are returned to the caller so the command layer can choose the output format.
pub fn build_project(project: &Project, emitter: &dyn Emitter) -> BuildReport {
    let mut summary = BuildSummary::default();
    let mut diagnostics = DiagnosticReport::default();

    if let Err(report) = project.ensure_out_dir() {
        diagnostics.extend(report);
        return BuildReport {
            summary,
            diagnostics,
        };
    }

    let source_files = match project.discover_sources() {
        Ok(source_files) => source_files,
        Err(report) => {
            diagnostics.extend(report);
            return BuildReport {
                summary,
                diagnostics,
            };
        }
    };
    summary.discovered_files = source_files.len();

    for source_file in source_files {
        let source = match fs::read_to_string(&source_file.source_path) {
            Ok(source) => source,
            Err(error) => {
                let display_path = relativize_path(project, &source_file.source_path);
                diagnostics.push(build_io_diagnostic(
                    project,
                    &source_file.source_path,
                    format!("Failed to read source file {}: {}.", display_path, error),
                ));
                continue;
            }
        };
        let logical_path = logical_source_path(project, &source_file);
        let logical_path_string = normalize_path(&logical_path);
        let tokens = lexer::tokenize(&source, logical_path_string.clone());
        let ast = parser::parse(tokens);

        let file_diagnostics = ast.diagnostics.clone();
        let has_errors = file_diagnostics.has_errors();
        diagnostics.extend(file_diagnostics);
        if has_errors {
            continue;
        }

        let emitted = emitter.emit(&ast, Path::new(&logical_path_string));
        match write_if_changed(project, &source_file.output_path, &emitted) {
            Ok(true) => summary.written_files += 1,
            Ok(false) => {}
            Err(report) => diagnostics.extend(report),
        }
    }

    BuildReport {
        summary,
        diagnostics,
    }
}

fn logical_source_path(project: &Project, source_file: &SourceFile) -> PathBuf {
    project.config.source_dir.join(&source_file.relative_path)
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn write_if_changed(
    project: &Project,
    path: &Path,
    contents: &str,
) -> std::result::Result<bool, DiagnosticReport> {
    match fs::read_to_string(path) {
        Ok(existing) if existing == contents => return Ok(false),
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            let display_path = relativize_path(project, path);
            return Err(DiagnosticReport::single(build_io_diagnostic(
                project,
                path,
                format!("Failed to read {}: {}.", display_path, error),
            )));
        }
    }

    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            let display_path = relativize_path(project, parent);
            return Err(DiagnosticReport::single(build_io_diagnostic(
                project,
                parent,
                format!(
                    "Failed to create output subdirectory {}: {}.",
                    display_path, error
                ),
            )));
        }
    }

    if let Err(error) = fs::write(path, contents) {
        let display_path = relativize_path(project, path);
        return Err(DiagnosticReport::single(build_io_diagnostic(
            project,
            path,
            format!("Failed to write {}: {}.", display_path, error),
        )));
    }

    Ok(true)
}

fn build_io_diagnostic(project: &Project, path: &Path, message: String) -> Diagnostic {
    Diagnostic::error(
        BUILD_IO_ERROR_CODE,
        relativize_path(project, path),
        message,
        Span::point(1, 1),
    )
}

fn relativize_path(project: &Project, path: &Path) -> String {
    normalize_path(path.strip_prefix(project.root()).unwrap_or(path))
}
