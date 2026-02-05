use bixbite::{
    checker::{noop::NoopChecker, TypeChecker},
    project::Project,
};
use tempfile::tempdir;

#[test]
fn noop_checker_returns_empty_report() {
    let temp = tempdir().expect("temp dir should be created");
    let project = Project::load(temp.path()).expect("project should load");
    let checker = NoopChecker;

    let report = checker
        .check(&project)
        .expect("noop checker should not fail");

    assert!(report.diagnostics.is_empty());
    assert!(!report.has_errors());
}
