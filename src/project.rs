use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;
use walkdir::WalkDir;

const TOML_FILE: &str = "bixbite.toml";
const JSON_FILE: &str = "bixbite.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub source_dir: PathBuf,
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

    pub fn from_toml_str(contents: &str) -> Result<Self> {
        let parsed: RawConfig = toml::from_str(contents).context("failed to parse bixbite.toml")?;
        Ok(Self::from_raw(parsed))
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub source_path: PathBuf,
    pub relative_path: PathBuf,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Project {
    root: PathBuf,
    pub config: Config,
}

impl Project {
    pub fn load(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let config = Config::load(&root)?;
        Ok(Self { root, config })
    }

    pub fn from_root_and_config(root: impl Into<PathBuf>, config: Config) -> Self {
        Self {
            root: root.into(),
            config,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn source_root(&self) -> PathBuf {
        self.root.join(&self.config.source_dir)
    }

    pub fn out_root(&self) -> PathBuf {
        self.root.join(&self.config.out_dir)
    }

    pub fn ensure_out_dir(&self) -> Result<()> {
        let out_root = self.out_root();
        fs::create_dir_all(&out_root).with_context(|| {
            format!(
                "failed to create output directory {}",
                out_root.to_string_lossy()
            )
        })
    }

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
