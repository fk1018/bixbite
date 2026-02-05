use std::fs;

use bixbite::project::Project;
use tempfile::tempdir;

#[test]
fn discovers_only_bixb_files_recursively() {
    let temp = tempdir().expect("temp dir should be created");
    let source_dir = temp.path().join("src");
    fs::create_dir_all(source_dir.join("nested/deep")).expect("source tree should be created");

    fs::write(source_dir.join("a.bixb"), "def a() -> NilClass\nend\n").expect("a.bixb");
    fs::write(
        source_dir.join("nested/b.bixb"),
        "def b() -> NilClass\nend\n",
    )
    .expect("b.bixb");
    fs::write(
        source_dir.join("nested/deep/c.bixb"),
        "def c() -> NilClass\nend\n",
    )
    .expect("c.bixb");
    fs::write(source_dir.join("ignore.rb"), "puts :ignore").expect("ignore.rb");
    fs::write(source_dir.join("nested/ignore.txt"), "ignore").expect("ignore.txt");

    let project = Project::load(temp.path()).expect("project should load");
    let discovered = project
        .discover_sources()
        .expect("file discovery should succeed for valid tree");
    let mut discovered_relative: Vec<String> = discovered
        .iter()
        .map(|source_file| {
            source_file
                .relative_path
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    discovered_relative.sort();

    assert_eq!(
        discovered_relative,
        vec!["a.bixb", "nested/b.bixb", "nested/deep/c.bixb"]
    );
}

#[test]
fn returns_empty_when_source_directory_is_missing() {
    let temp = tempdir().expect("temp dir should be created");
    let project = Project::load(temp.path()).expect("project should load");
    let discovered = project
        .discover_sources()
        .expect("missing source should not fail");

    assert!(discovered.is_empty());
}
