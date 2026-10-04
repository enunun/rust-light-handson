mod common;

use std::fs;
use std::path::Path;

use common::{git, rgit, rgit_error};
use tempfile::TempDir;

/// 本物の`git`でリポジトリを作り，`hello.txt`をオブジェクトとして書く．
fn repository_with_hello() -> TempDir {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "-q"]);
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    git(dir.path(), &["hash-object", "-w", "hello.txt"]);
    dir
}

/// 空のツリーを親のない1つのコミットにし，そのIDを返す．
fn commit_empty_tree(dir: &Path) -> String {
    let tree = git(dir, &["write-tree"]);
    let commit = git(
        dir,
        &[
            "-c",
            "user.name=Alice",
            "-c",
            "user.email=alice@example.com",
            "commit-tree",
            tree.trim(),
            "-m",
            "first",
        ],
    );
    commit.trim().to_string()
}

#[test]
fn shows_type_of_blob_written_by_git() {
    let dir = repository_with_hello();
    assert_eq!(rgit(dir.path(), &["cat-file", "-t", "ce01362"]), "blob\n");
}

#[test]
fn shows_size_of_blob() {
    let dir = repository_with_hello();
    assert_eq!(rgit(dir.path(), &["cat-file", "-s", "ce01362"]), "6\n");
}

#[test]
fn shows_content_of_blob() {
    let dir = repository_with_hello();
    assert_eq!(rgit(dir.path(), &["cat-file", "-p", "ce01362"]), "hello\n");
}

#[test]
fn shows_commit_like_git() {
    let dir = repository_with_hello();
    let commit = commit_empty_tree(dir.path());
    assert_eq!(rgit(dir.path(), &["cat-file", "-t", &commit]), "commit\n");
    assert_eq!(
        rgit(dir.path(), &["cat-file", "-p", &commit]),
        git(dir.path(), &["cat-file", "-p", &commit])
    );
    assert_eq!(
        rgit(dir.path(), &["cat-file", "-s", &commit]),
        git(dir.path(), &["cat-file", "-s", &commit])
    );
}

#[test]
fn unknown_object_is_an_error() {
    let dir = repository_with_hello();
    let error = rgit_error(dir.path(), &["cat-file", "-t", "abcd"]);
    assert_eq!(error.to_string(), "Not a valid object name abcd");
}

#[test]
fn ambiguous_prefix_is_an_error() {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "-q"]);
    fs::write(dir.path().join("a.txt"), "195\n").unwrap();
    fs::write(dir.path().join("b.txt"), "389\n").unwrap();
    git(dir.path(), &["hash-object", "-w", "a.txt", "b.txt"]);
    let error = rgit_error(dir.path(), &["cat-file", "-t", "6bb2"]);
    assert_eq!(error.to_string(), "short object ID 6bb2 is ambiguous");
}

#[test]
fn only_one_of_t_s_p_can_be_given() {
    let dir = repository_with_hello();
    let error = rgit_error(dir.path(), &["cat-file", "-t", "-p", "ce01362"]);
    assert!(matches!(error, rgit::Error::Usage(_)));
}
