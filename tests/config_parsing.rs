use std::{fs, path::PathBuf};

use bixbite::project::Config;
use tempfile::tempdir;

#[test]
fn uses_default_config_when_no_file_exists() {
    let temp = tempdir().expect("temp dir should be created");

    let config = Config::load(temp.path()).expect("default config should load");

    assert_eq!(config.source_dir, PathBuf::from("src"));
    assert_eq!(config.out_dir, PathBuf::from("build"));
}

#[test]
fn parses_bixbite_toml() {
    let temp = tempdir().expect("temp dir should be created");
    fs::write(
        temp.path().join("bixbite.toml"),
        "source_dir = \"sources\"\nout_dir = \"dist\"\n",
    )
    .expect("config should be written");

    let config = Config::load(temp.path()).expect("toml config should load");

    assert_eq!(config.source_dir, PathBuf::from("sources"));
    assert_eq!(config.out_dir, PathBuf::from("dist"));
}

#[test]
fn parses_bixbite_json_when_toml_is_missing() {
    let temp = tempdir().expect("temp dir should be created");
    fs::write(
        temp.path().join("bixbite.json"),
        r#"{"source_dir":"json_src","out_dir":"json_build"}"#,
    )
    .expect("config should be written");

    let config = Config::load(temp.path()).expect("json config should load");

    assert_eq!(config.source_dir, PathBuf::from("json_src"));
    assert_eq!(config.out_dir, PathBuf::from("json_build"));
}
