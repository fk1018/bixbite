use std::{env, fs, path::Path};

use anyhow::{Context, Result};

use crate::{
    emitter::{ruby::RubyEmitter, Emitter},
    lexer, parser,
    project::Project,
};

/// Summary of a build invocation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildSummary {
    /// Number of `.bixb` files discovered under the source root.
    pub discovered_files: usize,
    /// Number of `.rb` files written to the output directory.
    pub written_files: usize,
}

/// Runs the `bixbite build` command for the current working directory.
///
/// Errors include IO failures, parsing diagnostics with errors, or output writes.
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

/// Builds all source files in the project using the provided emitter.
///
/// Parsing diagnostics are printed in human format. If any errors are present, the
/// build aborts and returns an error.
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
            ast.diagnostics.print_human_stderr();
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
