use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};

use crate::{
    diagnostic::DiagnosticReport,
    emitter::{ruby::RubyEmitter, Emitter},
    lexer, parser,
    project::{Project, SourceFile},
};

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
/// Errors include IO failures and any build diagnostics with error severity.
pub fn run() -> Result<()> {
    let project_root = env::current_dir().context("failed to determine current directory")?;
    let project = Project::load(project_root)?;
    let emitter = RubyEmitter;
    let report = build_project(&project, &emitter)?;

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
pub fn build_project(project: &Project, emitter: &dyn Emitter) -> Result<BuildReport> {
    project.ensure_out_dir()?;
    let source_files = project.discover_sources()?;
    let mut summary = BuildSummary {
        discovered_files: source_files.len(),
        written_files: 0,
    };
    let mut diagnostics = DiagnosticReport::default();

    for source_file in source_files {
        let source = fs::read_to_string(&source_file.source_path).with_context(|| {
            format!(
                "failed to read source file {}",
                source_file.source_path.to_string_lossy()
            )
        })?;
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
        if write_if_changed(&source_file.output_path, &emitted)? {
            summary.written_files += 1;
        }
    }

    Ok(BuildReport {
        summary,
        diagnostics,
    })
}

fn logical_source_path(project: &Project, source_file: &SourceFile) -> PathBuf {
    project.config.source_dir.join(&source_file.relative_path)
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn write_if_changed(path: &Path, contents: &str) -> Result<bool> {
    match fs::read_to_string(path) {
        Ok(existing) if existing == contents => return Ok(false),
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(anyhow!(
                "failed to read {}: {}",
                path.to_string_lossy(),
                error
            ));
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create output subdirectory {}",
                parent.to_string_lossy()
            )
        })?;
    }

    fs::write(path, contents)
        .with_context(|| format!("failed to write {}", path.to_string_lossy()))?;
    Ok(true)
}
