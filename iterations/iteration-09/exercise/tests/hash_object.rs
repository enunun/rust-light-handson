mod common;

use std::fs;

use common::{git, rgit, rgit_error};
use tempfile::TempDir;

const HELLO: &str = "ce013625030ba8dba906f756967f9e9ca394464a";

#[test]
fn hash_object_prints_id_without_repository() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    assert_eq!(
        rgit(dir.path(), &["hash-object", "hello.txt"]),
        format!("{HELLO}\n")
    );
}

#[test]
fn hash_object_without_w_writes_nothing() {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    rgit(dir.path(), &["hash-object", "hello.txt"]);
    assert!(!dir.path().join(".git/objects/ce").exists());
}

#[test]
fn written_object_can_be_read_by_git() {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    assert_eq!(
        rgit(dir.path(), &["hash-object", "-w", "hello.txt"]),
        format!("{HELLO}\n")
    );
    assert_eq!(git(dir.path(), &["cat-file", "-t", HELLO]), "blob\n");
    assert_eq!(git(dir.path(), &["cat-file", "-p", HELLO]), "hello\n");
}

#[test]
fn hash_object_finds_repository_from_subdirectory() {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    let sub = dir.path().join("src");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("hello.txt"), "hello\n").unwrap();
    rgit(&sub, &["hash-object", "-w", "hello.txt"]);
    assert_eq!(git(dir.path(), &["cat-file", "-p", HELLO]), "hello\n");
}

#[test]
fn writing_outside_repository_is_an_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    let error = rgit_error(dir.path(), &["hash-object", "-w", "hello.txt"]);
    assert_eq!(
        error.to_string(),
        "not a git repository (or any of the parent directories): .git"
    );
}
