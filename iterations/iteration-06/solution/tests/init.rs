mod common;

use std::fs;

use common::{git, rgit};
use tempfile::TempDir;

#[test]
fn init_prints_absolute_path_of_git_directory() {
    let dir = TempDir::new().unwrap();
    let git_dir = fs::canonicalize(dir.path()).unwrap().join(".git");
    assert_eq!(
        rgit(dir.path(), &["init"]),
        format!(
            "Initialized empty Git repository in {}/\n",
            git_dir.display()
        )
    );
}

#[test]
fn init_with_directory_creates_repository_there() {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init", "project"]);
    assert!(dir.path().join("project/.git/objects").is_dir());
}

#[test]
fn git_recognizes_initialized_repository() {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    assert_eq!(git(dir.path(), &["rev-parse", "--git-dir"]), ".git\n");
}
