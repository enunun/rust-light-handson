mod common;

use std::fs;

use common::{git, rgit};
use tempfile::TempDir;

/// Gitの並び順を確かめられる名前のファイル(`a-b`，`a.txt`，`a/b`)を持つ作業ディレクトリを作る．
fn work_tree() -> TempDir {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::create_dir_all(dir.path().join("a")).unwrap();
    fs::write(dir.path().join("a/b"), "y\n").unwrap();
    fs::write(dir.path().join("a-b"), "z\n").unwrap();
    fs::write(dir.path().join("a.txt"), "x\n").unwrap();
    dir
}

#[test]
fn write_tree_matches_git_write_tree() {
    let dir = work_tree();
    rgit(dir.path(), &["add", "."]);
    assert_eq!(
        rgit(dir.path(), &["write-tree"]),
        git(dir.path(), &["write-tree"])
    );
}

#[test]
fn trees_written_by_rgit_can_be_listed_by_git() {
    let dir = work_tree();
    rgit(dir.path(), &["add", "."]);
    let tree = rgit(dir.path(), &["write-tree"]);
    assert_eq!(
        git(dir.path(), &["ls-tree", "-r", "--name-only", tree.trim()]),
        "a-b\na.txt\na/b\nhello.txt\nsrc/main.rs\n"
    );
}

#[test]
fn write_tree_uses_index_not_work_tree() {
    let dir = work_tree();
    rgit(dir.path(), &["add", "hello.txt"]);
    fs::write(dir.path().join("hello.txt"), "changed\n").unwrap();
    assert_eq!(
        rgit(dir.path(), &["write-tree"]),
        "aaa96ced2d9a1c8e72c56b253a0e2fe78393feb7\n"
    );
}
