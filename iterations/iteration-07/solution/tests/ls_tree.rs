mod common;

use std::fs;

use common::{git, rgit, rgit_error};
use tempfile::TempDir;

/// 本物の`git`で，いろいろなモードのエントリーを持つtreeを作り，リポジトリとtreeのIDを返す．
fn repository_with_tree() -> (TempDir, String) {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "-q"]);
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    git(dir.path(), &["add", "hello.txt", "src"]);
    let blob = git(dir.path(), &["hash-object", "-w", "hello.txt"]);
    let blob = blob.trim();
    git(
        dir.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("100755,{blob},run.sh"),
        ],
    );
    git(
        dir.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("120000,{blob},link"),
        ],
    );
    let tree = git(dir.path(), &["write-tree"]).trim().to_string();
    (dir, tree)
}

#[test]
fn ls_tree_matches_git() {
    let (dir, tree) = repository_with_tree();
    assert_eq!(
        rgit(dir.path(), &["ls-tree", &tree]),
        git(dir.path(), &["ls-tree", &tree])
    );
}

#[test]
fn cat_file_p_shows_tree_like_git() {
    let (dir, tree) = repository_with_tree();
    assert_eq!(
        rgit(dir.path(), &["cat-file", "-p", &tree]),
        git(dir.path(), &["cat-file", "-p", &tree])
    );
}

#[test]
fn ls_tree_of_subdirectory_tree() {
    let (dir, _) = repository_with_tree();
    assert_eq!(
        rgit(dir.path(), &["ls-tree", "5d90422"]),
        "100644 blob f328e4d9d04c31d0d70d16d21a07d1613be9d577\tmain.rs\n"
    );
}

#[test]
fn ls_tree_of_blob_is_an_error() {
    let (dir, _) = repository_with_tree();
    let error = rgit_error(dir.path(), &["ls-tree", "ce01362"]);
    assert_eq!(error.to_string(), "not a tree object");
}
