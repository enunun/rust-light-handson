mod common;

use std::fs;

use common::{git, rgit, rgit_at, rgit_error};
use tempfile::TempDir;

/// `hello.txt`を書き換えながら，1時間おきに3つのコミットを作る．
fn linear_history() -> TempDir {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    for (hour, message) in ["first", "second", "third"].iter().enumerate() {
        fs::write(dir.path().join("hello.txt"), format!("{message}\n")).unwrap();
        rgit(dir.path(), &["add", "."]);
        let time = 1767225600 + 3600 * hour as i64;
        rgit_at(dir.path(), time, &["commit", "-m", message]);
    }
    dir
}

#[test]
fn log_matches_git_log_oneline() {
    let dir = linear_history();
    let log = rgit(dir.path(), &["log"]);
    assert_eq!(log, git(dir.path(), &["log", "--oneline"]));
    let messages: Vec<&str> = log.lines().map(|line| &line[8..]).collect();
    assert_eq!(messages, ["third", "second", "first"]);
}

#[test]
fn log_with_limit_and_start() {
    let dir = linear_history();
    assert_eq!(
        rgit(dir.path(), &["log", "-n", "1", "HEAD~1"]),
        git(dir.path(), &["log", "--oneline", "-n", "1", "HEAD~1"])
    );
}

#[test]
fn log_of_merge_shows_each_commit_once() {
    let dir = linear_history();
    let tree = rgit(dir.path(), &["write-tree"]).trim().to_string();
    let base = rgit(dir.path(), &["rev-parse", "HEAD~2"])
        .trim()
        .to_string();
    let side = rgit_at(
        dir.path(),
        1767240000,
        &["commit-tree", &tree, "-p", &base, "-m", "side"],
    );
    let merge = rgit_at(
        dir.path(),
        1767250000,
        &[
            "commit-tree",
            &tree,
            "-p",
            "HEAD",
            "-p",
            side.trim(),
            "-m",
            "merge",
        ],
    );
    assert_eq!(
        rgit(dir.path(), &["log", merge.trim()]),
        git(dir.path(), &["log", "--oneline", merge.trim()])
    );
    assert_eq!(rgit(dir.path(), &["log", merge.trim()]).lines().count(), 5);
}

#[test]
fn log_of_unborn_branch_is_an_error() {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    let error = rgit_error(dir.path(), &["log"]);
    assert_eq!(error.to_string(), "Not a valid object name HEAD");
}
