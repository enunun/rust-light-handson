mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;

use common::{git, rgit};
use tempfile::TempDir;

/// 4つのファイルをコミットしたリポジトリを作る．
fn committed_repository() -> TempDir {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    for name in ["b.txt", "d.txt", "m.txt", "x.sh"] {
        fs::write(dir.path().join(name), "1\n").unwrap();
    }
    rgit(dir.path(), &["add", "."]);
    rgit(dir.path(), &["commit", "-m", "first"]);
    dir
}

fn assert_same_as_git(dir: &TempDir) {
    assert_eq!(
        rgit(dir.path(), &["status"]),
        git(dir.path(), &["status", "--porcelain", "-uall"])
    );
}

#[test]
fn clean_repository_has_empty_status() {
    let dir = committed_repository();
    assert_eq!(rgit(dir.path(), &["status"]), "");
    assert_same_as_git(&dir);
}

#[test]
fn status_before_first_commit_shows_additions() {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    fs::write(dir.path().join("todo.txt"), "notes\n").unwrap();
    rgit(dir.path(), &["add", "hello.txt"]);
    assert_eq!(rgit(dir.path(), &["status"]), "A  hello.txt\n?? todo.txt\n");
    assert_same_as_git(&dir);
}

#[test]
fn status_shows_every_kind_of_change_like_git() {
    let dir = committed_repository();
    let path = |name: &str| dir.path().join(name);
    fs::write(path("b.txt"), "2\n").unwrap();
    rgit(dir.path(), &["add", "b.txt"]);
    fs::write(path("b.txt"), "3\n").unwrap();
    fs::remove_file(path("d.txt")).unwrap();
    fs::write(path("m.txt"), "2\n").unwrap();
    rgit(dir.path(), &["add", "m.txt"]);
    fs::write(path("a.txt"), "new\n").unwrap();
    fs::write(path("n.txt"), "n\n").unwrap();
    rgit(dir.path(), &["add", "n.txt"]);
    fs::set_permissions(path("x.sh"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::create_dir(path("sub")).unwrap();
    fs::write(path("sub/u.txt"), "u\n").unwrap();
    assert_eq!(
        rgit(dir.path(), &["status"]),
        "MM b.txt\n D d.txt\nM  m.txt\nA  n.txt\n M x.sh\n?? a.txt\n?? sub/u.txt\n"
    );
    assert_same_as_git(&dir);
}

#[test]
fn staged_deletion_after_add() {
    let dir = committed_repository();
    fs::remove_file(dir.path().join("d.txt")).unwrap();
    rgit(dir.path(), &["add", "."]);
    assert_eq!(rgit(dir.path(), &["status"]), "D  d.txt\n");
    assert_same_as_git(&dir);
}
