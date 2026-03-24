use std::{
    fs,
    path::{Path, PathBuf},
};

use bixbite::{
    commands::{
        build,
        check::{self, OutputFormat},
    },
    emitter::ruby::RubyEmitter,
    project::Project,
};
use tempfile::{tempdir, TempDir};
use walkdir::WalkDir;

#[test]
fn mvp_matrix_fixture_matches_expected_ruby_output() {
    let temp = stage_fixture("mvp_matrix");
    let project = Project::load(temp.path()).expect("fixture project should load");
    let report = build::build_project(&project, &RubyEmitter);

    assert!(
        report.diagnostics.is_empty(),
        "expected fixture to build without diagnostics"
    );
    assert_eq!(report.summary.discovered_files, 1);
    assert_eq!(report.summary.written_files, 1);

    assert_tree_matches(
        &temp.path().join("build"),
        &fixture_root("mvp_matrix").join("expected/build"),
    );
}

#[test]
fn mixed_param_types_fixture_matches_expected_diagnostics() {
    let temp = stage_fixture("mixed_param_types");
    let project = Project::load(temp.path()).expect("fixture project should load");
    let report = build::build_project(&project, &RubyEmitter);

    assert!(report.diagnostics.has_errors());
    assert_eq!(
        check::render_diagnostics(&report.diagnostics, OutputFormat::Human)
            .expect("human diagnostics should render"),
        expected_text("mixed_param_types", "expected/human.txt")
    );
    assert_eq!(
        check::render_diagnostics(&report.diagnostics, OutputFormat::Json)
            .expect("json diagnostics should render"),
        expected_text("mixed_param_types", "expected/json.json")
    );
    assert!(!temp.path().join("build/bad.rb").exists());
}

#[test]
fn malformed_toml_fixture_matches_expected_diagnostics() {
    let temp = stage_fixture("malformed_toml");
    let report = Project::load(temp.path()).expect_err("fixture config should fail to parse");

    assert_eq!(
        check::render_diagnostics(&report, OutputFormat::Human)
            .expect("human diagnostics should render"),
        expected_text("malformed_toml", "expected/human.txt")
    );
    assert_eq!(
        check::render_diagnostics(&report, OutputFormat::Json)
            .expect("json diagnostics should render"),
        expected_text("malformed_toml", "expected/json.json")
    );
}

#[test]
fn crlf_source_fixture_emits_normalized_lf_output() {
    let temp = stage_fixture("mvp_matrix");
    let source_path = temp.path().join("src/example.bixb");
    let source = fs::read_to_string(&source_path).expect("fixture source should exist");
    let crlf_source = source.lines().collect::<Vec<_>>().join("\r\n");
    fs::write(&source_path, format!("{crlf_source}\r\n")).expect("crlf source should be written");

    let project = Project::load(temp.path()).expect("fixture project should load");
    let report = build::build_project(&project, &RubyEmitter);
    assert!(
        report.diagnostics.is_empty(),
        "expected crlf fixture to build without diagnostics"
    );

    let actual = fs::read_to_string(temp.path().join("build/example.rb"))
        .expect("normalized build output should exist");
    assert_eq!(
        actual,
        expected_text("mvp_matrix", "expected/build/example.rb")
    );
    assert!(!actual.contains('\r'));
}

fn stage_fixture(name: &str) -> TempDir {
    let temp = tempdir().expect("temp dir should be created");
    copy_tree(&fixture_root(name), temp.path());
    temp
}

fn fixture_root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn expected_text(name: &str, relative_path: &str) -> String {
    let contents = read_expected_snapshot(fixture_root(name).join(relative_path));
    if relative_path.ends_with(".json") {
        contents.trim_end_matches('\n').to_string()
    } else {
        contents
    }
}

fn read_expected_snapshot(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path)
        .expect("fixture snapshot should be readable")
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

fn copy_tree(source: &Path, destination: &Path) {
    for entry in WalkDir::new(source).min_depth(1) {
        let entry = entry.expect("fixture walk should succeed");
        let relative = entry
            .path()
            .strip_prefix(source)
            .expect("fixture entry should be relative");
        let target = destination.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).expect("fixture directory should be created");
            continue;
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).expect("fixture parent should be created");
        }
        fs::copy(entry.path(), &target).expect("fixture file should be copied");
    }
}

fn assert_tree_matches(actual_root: &Path, expected_root: &Path) {
    let mut expected_files = Vec::new();

    for entry in WalkDir::new(expected_root).min_depth(1) {
        let entry = entry.expect("expected fixture walk should succeed");
        if !entry.file_type().is_file() {
            continue;
        }

        let relative = entry
            .path()
            .strip_prefix(expected_root)
            .expect("expected fixture entry should be relative")
            .to_path_buf();
        expected_files.push(relative.clone());

        let actual_path = actual_root.join(&relative);
        let expected_contents = read_expected_snapshot(entry.path());
        let actual_contents =
            fs::read_to_string(&actual_path).expect("actual generated file should be readable");
        assert_eq!(
            actual_contents,
            expected_contents,
            "generated output for {} did not match fixture",
            relative.display()
        );
    }

    let mut actual_files = WalkDir::new(actual_root)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| {
            entry
                .path()
                .strip_prefix(actual_root)
                .expect("actual generated file should be relative")
                .to_path_buf()
        })
        .collect::<Vec<_>>();

    expected_files.sort();
    actual_files.sort();
    assert_eq!(actual_files, expected_files);
}
