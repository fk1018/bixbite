use std::fs;

use bixbite::{
    commands::{
        build,
        check::{self, OutputFormat},
    },
    emitter::ruby::RubyEmitter,
    project::Project,
};
use tempfile::tempdir;

#[test]
fn human_diagnostics_render_with_relative_paths() {
    let temp = tempdir().expect("temp dir should be created");
    let source_dir = temp.path().join("src");
    fs::create_dir_all(&source_dir).expect("source dir should be created");
    fs::write(
        source_dir.join("bad.bixb"),
        "def f(x: Integer, y) -> Integer\n",
    )
    .expect("source file should be written");

    let project = Project::load(temp.path()).expect("project should load");
    let report = build::build_project(&project, &RubyEmitter);
    let rendered = check::render_diagnostics(&report.diagnostics, OutputFormat::Human)
        .expect("human diagnostics should render");

    assert!(rendered.contains("src/bad.bixb:1:19: error"));
    assert!(rendered.contains("Typed method signature requires all params to be typed."));
    assert!(rendered.contains("help: Add `: Type` to this parameter."));
}

#[test]
fn json_diagnostics_match_range_shape() {
    let temp = tempdir().expect("temp dir should be created");
    let source_dir = temp.path().join("src");
    fs::create_dir_all(&source_dir).expect("source dir should be created");
    fs::write(
        source_dir.join("bad.bixb"),
        "def f(x: Integer, y) -> Integer\n",
    )
    .expect("source file should be written");

    let project = Project::load(temp.path()).expect("project should load");
    let report = build::build_project(&project, &RubyEmitter);
    let rendered = check::render_diagnostics(&report.diagnostics, OutputFormat::Json)
        .expect("json diagnostics should render");
    let value: serde_json::Value =
        serde_json::from_str(&rendered).expect("json output should parse");

    assert_eq!(value["diagnostics"][0]["file"], "src/bad.bixb");
    assert_eq!(value["diagnostics"][0]["code"], "BIX001");
    assert_eq!(value["diagnostics"][0]["range"]["start"]["line"], 1);
    assert_eq!(value["diagnostics"][0]["range"]["start"]["col"], 19);
    assert!(value["diagnostics"][0].get("span").is_none());
    assert_eq!(
        value["diagnostics"][0]["suggestion"],
        "Add `: Type` to this parameter."
    );
}
