mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;

use common::{git, rgit};
use tempfile::TempDir;

/// 1から20までの数を1行ずつ書いた中身．
fn numbers() -> String {
    (1..=20).map(|n| format!("{n}\n")).collect()
}

/// いくつかのファイルをコミットしたリポジトリを作る．
fn committed_repository() -> TempDir {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    let path = |name: &str| dir.path().join(name);
    fs::write(path("hello.txt"), "hello\n").unwrap();
    fs::write(path("numbers.txt"), numbers()).unwrap();
    fs::write(path("d.txt"), "d\n").unwrap();
    fs::write(path("nonl.txt"), "abc").unwrap();
    fs::write(path("bin.dat"), b"a\0b").unwrap();
    fs::write(path("x.sh"), "1\n").unwrap();
    rgit(dir.path(), &["add", "."]);
    rgit(dir.path(), &["commit", "-m", "first"]);
    dir
}

/// 作業ディレクトリのファイルに，いろいろな変更を加える．
fn change_work_tree(dir: &TempDir) {
    let path = |name: &str| dir.path().join(name);
    fs::write(path("hello.txt"), "hello\nworld\n").unwrap();
    // 3行目と9行目を変え，末尾に1行加える．3つの変更のうち，前の2つは1つのハンクにまとまる．
    let changed: String = (1..=21)
        .map(|n| match n {
            3 => String::from("three\n"),
            9 => String::from("nine\n"),
            n => format!("{n}\n"),
        })
        .collect();
    fs::write(path("numbers.txt"), changed).unwrap();
    fs::remove_file(path("d.txt")).unwrap();
    fs::write(path("nonl.txt"), "abd").unwrap();
    fs::write(path("bin.dat"), b"a\0c").unwrap();
    fs::set_permissions(path("x.sh"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(path("untracked.txt"), "u\n").unwrap();
}

#[test]
fn clean_work_tree_has_empty_diff() {
    let dir = committed_repository();
    assert_eq!(rgit(dir.path(), &["diff"]), "");
    assert_eq!(rgit(dir.path(), &["diff", "--cached"]), "");
}

#[test]
fn diff_shows_one_line_addition() {
    let dir = committed_repository();
    fs::write(dir.path().join("hello.txt"), "hello\nworld\n").unwrap();
    assert_eq!(
        rgit(dir.path(), &["diff"]),
        "diff --git a/hello.txt b/hello.txt\n\
         index ce01362..94954ab 100644\n\
         --- a/hello.txt\n\
         +++ b/hello.txt\n\
         @@ -1 +1,2 @@\n \
         hello\n\
         +world\n"
    );
}

#[test]
fn diff_of_work_tree_matches_git() {
    let dir = committed_repository();
    change_work_tree(&dir);
    assert_eq!(rgit(dir.path(), &["diff"]), git(dir.path(), &["diff"]));
}

#[test]
fn diff_cached_matches_git() {
    let dir = committed_repository();
    change_work_tree(&dir);
    fs::write(dir.path().join("empty.txt"), "").unwrap();
    rgit(dir.path(), &["add", "."]);
    assert_eq!(rgit(dir.path(), &["diff"]), "");
    assert_eq!(
        rgit(dir.path(), &["diff", "--cached"]),
        git(dir.path(), &["diff", "--cached"])
    );
}

#[test]
fn diff_cached_before_first_commit_shows_new_files() {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    rgit(dir.path(), &["add", "hello.txt"]);
    assert_eq!(
        rgit(dir.path(), &["diff", "--cached"]),
        git(dir.path(), &["diff", "--cached"])
    );
}
