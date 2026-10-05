mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use common::{git, rgit, rgit_error};
use tempfile::TempDir;

/// `hello.txt`と`src/main.rs`のある作業ディレクトリを，`rgit init`で作る．
fn work_tree() -> TempDir {
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    dir
}

#[test]
fn ls_files_stage_after_add_matches_git() {
    let dir = work_tree();
    rgit(dir.path(), &["add", "."]);
    assert_eq!(
        rgit(dir.path(), &["ls-files", "--stage"]),
        "100644 ce013625030ba8dba906f756967f9e9ca394464a 0\thello.txt\n\
         100644 f328e4d9d04c31d0d70d16d21a07d1613be9d577 0\tsrc/main.rs\n"
    );
    assert_eq!(
        git(dir.path(), &["ls-files", "--stage"]),
        rgit(dir.path(), &["ls-files", "--stage"])
    );
}

#[test]
fn ls_files_shows_paths() {
    let dir = work_tree();
    rgit(dir.path(), &["add", "."]);
    assert_eq!(rgit(dir.path(), &["ls-files"]), "hello.txt\nsrc/main.rs\n");
}

#[test]
fn git_sees_files_added_by_rgit_as_staged() {
    let dir = work_tree();
    rgit(dir.path(), &["add", "."]);
    assert_eq!(
        git(dir.path(), &["status", "--porcelain"]),
        "A  hello.txt\nA  src/main.rs\n"
    );
}

#[test]
fn rgit_reads_index_written_by_git() {
    let dir = work_tree();
    git(dir.path(), &["add", "."]);
    assert_eq!(
        rgit(dir.path(), &["ls-files", "--stage"]),
        git(dir.path(), &["ls-files", "--stage"])
    );
}

#[test]
fn add_from_subdirectory_uses_path_from_work_tree() {
    let dir = work_tree();
    rgit(&dir.path().join("src"), &["add", "main.rs"]);
    assert_eq!(rgit(dir.path(), &["ls-files"]), "src/main.rs\n");
}

#[test]
fn add_records_executable_files() {
    let dir = work_tree();
    let script = dir.path().join("run.sh");
    fs::write(&script, "echo hi\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    rgit(dir.path(), &["add", "run.sh"]);
    assert_eq!(
        git(dir.path(), &["ls-files", "--stage"]),
        "100755 8b2fe5434fec16870a71cd8b272c7fcf6d352536 0\trun.sh\n"
    );
}

#[test]
fn add_removes_deleted_file_from_index() {
    let dir = work_tree();
    rgit(dir.path(), &["add", "."]);
    fs::remove_file(dir.path().join("hello.txt")).unwrap();
    rgit(dir.path(), &["add", "."]);
    assert_eq!(rgit(dir.path(), &["ls-files"]), "src/main.rs\n");
}

#[test]
fn add_of_unknown_path_is_an_error() {
    let dir = work_tree();
    let error = rgit_error(dir.path(), &["add", "nothing"]);
    assert_eq!(
        error.to_string(),
        "pathspec 'nothing' did not match any files"
    );
}

/// `dir`に，10個のディレクトリに分けた200個のファイルを書く．
fn write_many_files(dir: &Path) {
    for i in 0..200 {
        let sub = dir.join(format!("files/{}", i % 10));
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join(format!("{i}.txt")), format!("file {i}\n")).unwrap();
    }
}

#[test]
fn add_with_any_number_of_jobs_matches_git() {
    let expected_dir = TempDir::new().unwrap();
    git(expected_dir.path(), &["init", "--quiet"]);
    write_many_files(expected_dir.path());
    git(expected_dir.path(), &["add", "."]);
    let expected = git(expected_dir.path(), &["ls-files", "--stage"]);
    for jobs in ["1", "4", "16"] {
        let dir = TempDir::new().unwrap();
        rgit(dir.path(), &["init"]);
        write_many_files(dir.path());
        rgit(dir.path(), &["add", "--jobs", jobs, "."]);
        assert_eq!(rgit(dir.path(), &["ls-files", "--stage"]), expected);
        // 並列に書いたオブジェクトが壊れていないことを，gitに確かめさせる．
        git(dir.path(), &["fsck", "--no-progress"]);
    }
}
