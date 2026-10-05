mod common;

use std::fs;

use common::{git, rgit, rgit_error};
use tempfile::TempDir;

/// `hello.txt`と`src/main.rs`を書き，`init_and_add`でリポジトリを作って`add`する．
fn staged_repository(init_and_add: fn(&std::path::Path)) -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    init_and_add(dir.path());
    dir
}

fn rgit_init(dir: &std::path::Path) {
    rgit(dir, &["init"]);
    rgit(dir, &["add", "."]);
}

fn git_init(dir: &std::path::Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "."]);
}

#[test]
fn first_commit_is_root_commit_on_main() {
    let dir = staged_repository(rgit_init);
    assert_eq!(
        rgit(dir.path(), &["commit", "-m", "first"]),
        "[main (root-commit) 6c04901] first\n"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join(".git/refs/heads/main")).unwrap(),
        "6c049013df700446ee9afd1bdaa0303bf75d842f\n"
    );
}

#[test]
fn commits_match_git_commit() {
    let ours = staged_repository(rgit_init);
    let theirs = staged_repository(git_init);
    rgit(ours.path(), &["commit", "-m", "first"]);
    git(theirs.path(), &["commit", "-q", "-m", "first"]);
    for dir in [ours.path(), theirs.path()] {
        fs::write(dir.join("hello.txt"), "hello\nworld\n").unwrap();
    }
    rgit(ours.path(), &["add", "hello.txt"]);
    git(theirs.path(), &["add", "hello.txt"]);
    assert_eq!(
        rgit(ours.path(), &["commit", "-m", "second"]),
        "[main 6c5c8da] second\n"
    );
    git(theirs.path(), &["commit", "-q", "-m", "second"]);
    assert_eq!(
        git(ours.path(), &["rev-parse", "HEAD"]),
        git(theirs.path(), &["rev-parse", "HEAD"])
    );
    assert_eq!(git(ours.path(), &["log", "--format=%s"]), "second\nfirst\n");
}

#[test]
fn rev_parse_resolves_head_branch_and_prefix() {
    let dir = staged_repository(rgit_init);
    rgit(dir.path(), &["commit", "-m", "first"]);
    let full = "6c049013df700446ee9afd1bdaa0303bf75d842f\n";
    assert_eq!(rgit(dir.path(), &["rev-parse", "HEAD"]), full);
    assert_eq!(rgit(dir.path(), &["rev-parse", "main"]), full);
    assert_eq!(rgit(dir.path(), &["rev-parse", "6c04901"]), full);
}

#[test]
fn cat_file_accepts_head() {
    let dir = staged_repository(rgit_init);
    rgit(dir.path(), &["commit", "-m", "first"]);
    assert_eq!(
        rgit(dir.path(), &["cat-file", "-p", "HEAD"]),
        git(dir.path(), &["cat-file", "-p", "HEAD"])
    );
}

#[test]
fn commit_fails_while_branch_is_locked() {
    let dir = staged_repository(rgit_init);
    rgit(dir.path(), &["commit", "-m", "first"]);
    let lock = dir.path().join(".git/refs/heads/main.lock");
    fs::write(&lock, "").unwrap();
    let error = rgit_error(dir.path(), &["commit", "-m", "second"]);
    assert_eq!(
        error.to_string(),
        format!("Unable to create '{}': File exists.", lock.display())
    );
    assert_eq!(
        rgit(dir.path(), &["rev-parse", "HEAD"]),
        "6c049013df700446ee9afd1bdaa0303bf75d842f\n"
    );
}

#[test]
fn rev_parse_of_unborn_head_is_an_error() {
    let dir = staged_repository(rgit_init);
    let error = rgit_error(dir.path(), &["rev-parse", "HEAD"]);
    assert_eq!(error.to_string(), "Not a valid object name HEAD");
}
