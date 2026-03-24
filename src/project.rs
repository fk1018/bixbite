use std::{
    fs,
    ops::Range,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use walkdir::WalkDir;

use crate::diagnostic::{Diagnostic, DiagnosticReport, Pos, Span};

const TOML_FILE: &str = "bixbite.toml";
const JSON_FILE: &str = "bixbite.json";
const CONFIG_ERROR_CODE: &str = "BIX200";
const PROJECT_IO_ERROR_CODE: &str = "BIX201";

type ProjectResult<T> = std::result::Result<T, DiagnosticReport>;

/// Project configuration describing source and output roots.
///
/// Invariants:
/// - `source_dir` and `out_dir` are relative to the project root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Directory containing `.bixb` sources.
    pub source_dir: PathBuf,
    /// Directory where `.rb` output is emitted.
    pub out_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source_dir: PathBuf::from("src"),
            out_dir: PathBuf::from("build"),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    source_dir: Option<PathBuf>,
    out_dir: Option<PathBuf>,
}

impl Config {
    /// Loads configuration from `bixbite.toml`, `bixbite.json`, or defaults.
    ///
    /// Errors are reported as diagnostics for config read or parse failures.
    pub fn load(project_root: &Path) -> ProjectResult<Self> {
        let toml_path = project_root.join(TOML_FILE);
        if toml_path.exists() {
            return Self::from_toml_file(&toml_path);
        }

        let json_path = project_root.join(JSON_FILE);
        if json_path.exists() {
            return Self::from_json_file(&json_path);
        }

        Ok(Self::default())
    }

    /// Parses configuration from TOML contents.
    pub fn from_toml_str(contents: &str) -> ProjectResult<Self> {
        toml::from_str(contents)
            .map(Self::from_raw)
            .map_err(|error| {
                config_parse_report(
                    TOML_FILE,
                    format!(
                        "Failed to parse {}: {}.",
                        TOML_FILE,
                        flatten_message(error.message())
                    ),
                    error
                        .span()
                        .map(|range| span_from_byte_range(contents, range))
                        .unwrap_or_else(|| Span::point(1, 1)),
                )
            })
    }

    /// Parses configuration from JSON contents.
    pub fn from_json_str(contents: &str) -> ProjectResult<Self> {
        serde_json::from_str(contents)
            .map(Self::from_raw)
            .map_err(|error| {
                config_parse_report(
                    JSON_FILE,
                    format!("Failed to parse {}: {}.", JSON_FILE, error),
                    Span::point(error.line().max(1), error.column().max(1)),
                )
            })
    }

    fn from_toml_file(path: &Path) -> ProjectResult<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|error| config_read_report(path, error.to_string()))?;
        Self::from_toml_str(&contents)
    }

    fn from_json_file(path: &Path) -> ProjectResult<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|error| config_read_report(path, error.to_string()))?;
        Self::from_json_str(&contents)
    }

    fn from_raw(raw: RawConfig) -> Self {
        let defaults = Self::default();

        Self {
            source_dir: raw.source_dir.unwrap_or(defaults.source_dir),
            out_dir: raw.out_dir.unwrap_or(defaults.out_dir),
        }
    }
}

/// A discovered `.bixb` source file and its derived output path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Absolute path to the source file.
    pub source_path: PathBuf,
    /// Path to the source file relative to the source root.
    pub relative_path: PathBuf,
    /// Absolute path to the expected `.rb` output file.
    pub output_path: PathBuf,
}

/// A loaded project with a root path and configuration.
#[derive(Debug, Clone)]
pub struct Project {
    root: PathBuf,
    /// Effective configuration for this project.
    pub config: Config,
}

impl Project {
    /// Loads a project from the given root directory.
    ///
    /// Errors are reported as diagnostics for configuration discovery or parsing failures.
    pub fn load(root: impl Into<PathBuf>) -> ProjectResult<Self> {
        let root = root.into();
        let config = Config::load(&root)?;
        Ok(Self { root, config })
    }

    /// Constructs a project from an explicit root and config.
    pub fn from_root_and_config(root: impl Into<PathBuf>, config: Config) -> Self {
        Self {
            root: root.into(),
            config,
        }
    }

    /// Returns the project root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the absolute path to the source root directory.
    pub fn source_root(&self) -> PathBuf {
        self.root.join(&self.config.source_dir)
    }

    /// Returns the absolute path to the output root directory.
    pub fn out_root(&self) -> PathBuf {
        self.root.join(&self.config.out_dir)
    }

    /// Ensures the output directory exists.
    ///
    /// Errors are reported as diagnostics for filesystem failures.
    pub fn ensure_out_dir(&self) -> ProjectResult<()> {
        let out_root = self.out_root();
        fs::create_dir_all(&out_root).map_err(|error| {
            project_io_report(
                &self.config.out_dir,
                format!(
                    "Failed to create output directory {}: {}.",
                    normalize_path(&self.config.out_dir),
                    error
                ),
            )
        })
    }

    /// Walks the source tree to discover `.bixb` files.
    ///
    /// Returns an ordered list of source files. If the source directory does not exist,
    /// the result is an empty list rather than an error.
    pub fn discover_sources(&self) -> ProjectResult<Vec<SourceFile>> {
        let source_root = self.source_root();
        let out_root = self.out_root();
        if !source_root.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        for entry in WalkDir::new(&source_root) {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    let path = error.path().unwrap_or(source_root.as_path());
                    return Err(project_io_report(
                        relativize_path(self.root(), path),
                        format!(
                            "Failed to discover source files under {}: {}.",
                            normalize_path(&self.config.source_dir),
                            error
                        ),
                    ));
                }
            };
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|ext| ext.to_str()) != Some("bixb") {
                continue;
            }

            let source_path = entry.into_path();
            let relative_path = match source_path.strip_prefix(&source_root) {
                Ok(relative_path) => relative_path.to_path_buf(),
                Err(_) => {
                    return Err(project_io_report(
                        relativize_path(self.root(), &source_path),
                        format!(
                            "Source file {} is not under source root {}.",
                            normalize_path(&source_path),
                            normalize_path(&source_root)
                        ),
                    ));
                }
            };
            let output_path = out_root.join(&relative_path).with_extension("rb");

            files.push(SourceFile {
                source_path,
                relative_path,
                output_path,
            });
        }

        files.sort_by(|left, right| left.source_path.cmp(&right.source_path));
        Ok(files)
    }
}

fn config_read_report(path: &Path, error: String) -> DiagnosticReport {
    DiagnosticReport::single(Diagnostic::error(
        CONFIG_ERROR_CODE,
        config_identifier(path),
        format!("Failed to read {}: {}.", config_identifier(path), error),
        Span::point(1, 1),
    ))
}

fn config_parse_report(file: &str, error: String, span: Span) -> DiagnosticReport {
    DiagnosticReport::single(Diagnostic::error(CONFIG_ERROR_CODE, file, error, span))
}

fn project_io_report(path: impl AsRef<Path>, message: String) -> DiagnosticReport {
    DiagnosticReport::single(Diagnostic::error(
        PROJECT_IO_ERROR_CODE,
        normalize_path(path.as_ref()),
        message,
        Span::point(1, 1),
    ))
}

fn config_identifier(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| normalize_path(path))
}

fn relativize_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn flatten_message(message: &str) -> String {
    message.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn span_from_byte_range(contents: &str, range: Range<usize>) -> Span {
    let start = byte_index_to_pos(contents, range.start);
    let end_index = range.end.saturating_sub(1).max(range.start);
    let end = byte_index_to_pos(contents, end_index);
    Span::new(start, end)
}

fn byte_index_to_pos(contents: &str, index: usize) -> Pos {
    let mut line = 1;
    let mut col = 1;

    for (offset, ch) in contents.char_indices() {
        if offset >= index {
            break;
        }

        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    Pos::new(line, col)
}
