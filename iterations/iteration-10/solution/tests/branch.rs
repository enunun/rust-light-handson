mod common;

use std::fs;

use common::{git, rgit, rgit_error};
use tempfile::TempDir;

/// 1つ目のコミットのあるリポジトリを作る．
fn committed_repository() -> TempDir {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    rgit(dir.path(), &["add", "."]);
    rgit(dir.path(), &["commit", "-m", "first"]);
    dir
}

#[test]
fn branch_lists_branches_and_marks_current_one() {
    let dir = committed_repository();
    rgit(dir.path(), &["branch", "topic"]);
    assert_eq!(rgit(dir.path(), &["branch"]), "* main\n  topic\n");
    assert_eq!(rgit(dir.path(), &["branch"]), git(dir.path(), &["branch"]));
}

#[test]
fn new_branch_points_to_head() {
    let dir = committed_repository();
    rgit(dir.path(), &["branch", "topic"]);
    assert_eq!(
        rgit(dir.path(), &["rev-parse", "topic"]),
        rgit(dir.path(), &["rev-parse", "HEAD"])
    );
}

#[test]
fn branch_can_start_at_given_revision() {
    let dir = committed_repository();
    let first = rgit(dir.path(), &["rev-parse", "HEAD"]);
    fs::write(dir.path().join("hello.txt"), "hello\nworld\n").unwrap();
    rgit(dir.path(), &["add", "."]);
    rgit(dir.path(), &["commit", "-m", "second"]);
    rgit(dir.path(), &["branch", "old", first.trim()]);
    assert_eq!(rgit(dir.path(), &["rev-parse", "old"]), first);
}

#[test]
fn branch_with_slash_in_name() {
    let dir = committed_repository();
    rgit(dir.path(), &["branch", "feature/login"]);
    assert_eq!(rgit(dir.path(), &["branch"]), "  feature/login\n* main\n");
}

#[test]
fn invalid_branch_name_is_an_error() {
    let dir = committed_repository();
    let error = rgit_error(dir.path(), &["branch", "bad name"]);
    assert_eq!(error.to_string(), "'bad name' is not a valid branch name");
}

#[test]
fn existing_branch_is_an_error() {
    let dir = committed_repository();
    rgit(dir.path(), &["branch", "topic"]);
    let error = rgit_error(dir.path(), &["branch", "topic"]);
    assert_eq!(error.to_string(), "a branch named 'topic' already exists");
}
