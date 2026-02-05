use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;
use walkdir::WalkDir;

const TOML_FILE: &str = "bixbite.toml";
const JSON_FILE: &str = "bixbite.json";

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
    /// Errors reflect IO or parse failures for discovered config files.
    pub fn load(project_root: &Path) -> Result<Self> {
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
    pub fn from_toml_str(contents: &str) -> Result<Self> {
        let parsed: RawConfig = toml::from_str(contents).context("failed to parse bixbite.toml")?;
        Ok(Self::from_raw(parsed))
    }

    /// Parses configuration from JSON contents.
    pub fn from_json_str(contents: &str) -> Result<Self> {
        let parsed: RawConfig =
            serde_json::from_str(contents).context("failed to parse bixbite.json")?;
        Ok(Self::from_raw(parsed))
    }

    fn from_toml_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.to_string_lossy()))?;
        Self::from_toml_str(&contents)
    }

    fn from_json_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.to_string_lossy()))?;
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
    /// Errors reflect configuration discovery or parsing failures.
    pub fn load(root: impl Into<PathBuf>) -> Result<Self> {
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
    /// Errors indicate failures to create the output directory.
    pub fn ensure_out_dir(&self) -> Result<()> {
        let out_root = self.out_root();
        fs::create_dir_all(&out_root).with_context(|| {
            format!(
                "failed to create output directory {}",
                out_root.to_string_lossy()
            )
        })
    }

    /// Walks the source tree to discover `.bixb` files.
    ///
    /// Returns an ordered list of source files. If the source directory does not exist,
    /// the result is an empty list rather than an error.
    pub fn discover_sources(&self) -> Result<Vec<SourceFile>> {
        let source_root = self.source_root();
        let out_root = self.out_root();
        if !source_root.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        for entry in WalkDir::new(&source_root) {
            let entry = entry.with_context(|| {
                format!("failed while walking {}", source_root.to_string_lossy())
            })?;
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|ext| ext.to_str()) != Some("bixb") {
                continue;
            }

            let source_path = entry.into_path();
            let relative_path = source_path
                .strip_prefix(&source_root)
                .with_context(|| {
                    format!(
                        "source file {} is not under source root {}",
                        source_path.to_string_lossy(),
                        source_root.to_string_lossy()
                    )
                })?
                .to_path_buf();
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
