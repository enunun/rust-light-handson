mod common;

use std::fs;

use common::{git, rgit, rgit_error};
use tempfile::TempDir;

/// `hello.txt`をインデックスに登録し，treeを書いたリポジトリと，treeのIDを返す．
fn repository_with_tree() -> (TempDir, String) {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    rgit(dir.path(), &["add", "."]);
    let tree = rgit(dir.path(), &["write-tree"]).trim().to_string();
    (dir, tree)
}

#[test]
fn commit_tree_matches_git_commit_tree() {
    let (dir, tree) = repository_with_tree();
    let commit = rgit(dir.path(), &["commit-tree", &tree, "-m", "first"]);
    assert_eq!(
        commit,
        git(dir.path(), &["commit-tree", &tree, "-m", "first"])
    );
    assert_eq!(
        git(dir.path(), &["cat-file", "-t", commit.trim()]),
        "commit\n"
    );
}

#[test]
fn commit_tree_with_parent() {
    let (dir, tree) = repository_with_tree();
    let first = rgit(dir.path(), &["commit-tree", &tree, "-m", "first"]);
    let second = rgit(
        dir.path(),
        &["commit-tree", &tree, "-p", first.trim(), "-m", "second"],
    );
    assert_eq!(
        second,
        git(
            dir.path(),
            &["commit-tree", &tree, "-p", first.trim(), "-m", "second"]
        )
    );
    assert_eq!(
        git(dir.path(), &["log", "--format=%s", second.trim()]),
        "second\nfirst\n"
    );
}

#[test]
fn commit_tree_without_author_is_an_error() {
    let (dir, tree) = repository_with_tree();
    let mut out = Vec::new();
    let env = std::collections::HashMap::new();
    let error = rgit::cli::run(
        &["commit-tree", &tree, "-m", "first"],
        dir.path(),
        &env,
        &mut out,
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "environment variable GIT_AUTHOR_NAME is not set"
    );
}

#[test]
fn unknown_tree_is_an_error() {
    let (dir, _) = repository_with_tree();
    let error = rgit_error(dir.path(), &["commit-tree", "abcd", "-m", "first"]);
    assert_eq!(error.to_string(), "Not a valid object name abcd");
}
