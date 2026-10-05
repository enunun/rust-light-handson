use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use crate::diff::{Edit, diff, hunks};
use crate::error::Error;
use crate::index::Index;
use crate::oid::ObjectId;
use crate::status::{compare, head_files, index_files, work_tree_files};
use crate::store::ObjectStore;
use crate::tree::Mode;

/// 差分の片側の，1つのファイル．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileVersion {
    pub mode: Mode,
    pub id: ObjectId,
    pub content: Vec<u8>,
}

/// 文脈として変更の前後に含める行の数．
const CONTEXT: usize = 3;

/// 1つのファイルの差分を，`git diff`と同じunified形式の文字列にする．
/// `old`がなければ追加されたファイル，`new`がなければ削除されたファイルとして扱う．
pub fn file_patch(path: &str, old: Option<&FileVersion>, new: Option<&FileVersion>) -> String {
    // Stringへの書き込みは失敗しないので，write!の結果はunwrapしてよい．
    let mut out = String::new();
    writeln!(out, "diff --git a/{path} b/{path}").unwrap();
    match (old, new) {
        (None, None) => return String::new(),
        (None, Some(new)) => {
            writeln!(out, "new file mode {}", new.mode).unwrap();
            writeln!(out, "index 0000000..{}", new.id.short()).unwrap();
        }
        (Some(old), None) => {
            writeln!(out, "deleted file mode {}", old.mode).unwrap();
            writeln!(out, "index {}..0000000", old.id.short()).unwrap();
        }
        (Some(old), Some(new)) if old.mode != new.mode => {
            writeln!(out, "old mode {}", old.mode).unwrap();
            writeln!(out, "new mode {}", new.mode).unwrap();
            if old.id == new.id {
                return out;
            }
            writeln!(out, "index {}..{}", old.id.short(), new.id.short()).unwrap();
        }
        (Some(old), Some(new)) => {
            let (old_id, new_id) = (old.id.short(), new.id.short());
            writeln!(out, "index {old_id}..{new_id} {}", old.mode).unwrap();
        }
    }

    let old_name = old.map_or(String::from("/dev/null"), |_| format!("a/{path}"));
    let new_name = new.map_or(String::from("/dev/null"), |_| format!("b/{path}"));
    let old_content = old.map_or(&[][..], |old| &old.content);
    let new_content = new.map_or(&[][..], |new| &new.content);
    if is_binary(old_content) || is_binary(new_content) {
        writeln!(out, "Binary files {old_name} and {new_name} differ").unwrap();
        return out;
    }

    let old_text = String::from_utf8_lossy(old_content);
    let new_text = String::from_utf8_lossy(new_content);
    let old_lines: Vec<&str> = old_text.split_inclusive('\n').collect();
    let new_lines: Vec<&str> = new_text.split_inclusive('\n').collect();
    let hunks = hunks(&diff(&old_lines, &new_lines), CONTEXT);
    if hunks.is_empty() {
        return out;
    }
    writeln!(out, "--- {old_name}").unwrap();
    writeln!(out, "+++ {new_name}").unwrap();
    for hunk in hunks {
        writeln!(out, "{hunk}").unwrap();
        for edit in hunk.edits {
            let (marker, line) = match edit {
                Edit::Equal(i, _) => (' ', old_lines[i]),
                Edit::Delete(i) => ('-', old_lines[i]),
                Edit::Insert(j) => ('+', new_lines[j]),
            };
            write!(out, "{marker}{line}").unwrap();
            if !line.ends_with('\n') {
                writeln!(out, "\n\\ No newline at end of file").unwrap();
            }
        }
    }
    out
}

/// NULを含む中身を，テキストではないとみなす．
fn is_binary(content: &[u8]) -> bool {
    content.contains(&0)
}

/// インデックスと作業ディレクトリの差分(`git diff`)を返す．作業ディレクトリのハッシュは`jobs`個までのスレッドで計算する．
pub fn diff_work_tree<S: ObjectStore + ?Sized>(
    store: &S,
    index: &Index,
    work_dir: &Path,
    jobs: usize,
) -> Result<String, Error> {
    let old = index_files(index);
    let new = work_tree_files(work_dir, jobs)?;
    // 作業ディレクトリにないファイルの追加は，`git diff`には出さない．
    let new = new
        .into_iter()
        .filter(|(path, _)| old.contains_key(path))
        .collect();
    diff_tables(store, &old, &new, Some(work_dir))
}

/// HEADのコミットとインデックスの差分(`git diff --cached`)を返す．
pub fn diff_cached<S: ObjectStore + ?Sized>(
    store: &S,
    head: Option<ObjectId>,
    index: &Index,
) -> Result<String, Error> {
    diff_tables(store, &head_files(store, head)?, &index_files(index), None)
}

/// 2つの表で違うファイルの差分を，パスの順につなげる．
/// 古い側の中身はオブジェクトストアから読む．新しい側の中身は，`work_dir`があればそこから，なければオブジェクトストアから読む．
fn diff_tables<S: ObjectStore + ?Sized>(
    store: &S,
    old: &BTreeMap<String, (Mode, ObjectId)>,
    new: &BTreeMap<String, (Mode, ObjectId)>,
    work_dir: Option<&Path>,
) -> Result<String, Error> {
    let mut out = String::new();
    let paths: BTreeSet<&String> = old.keys().chain(new.keys()).collect();
    for path in paths {
        if compare(old.get(path), new.get(path)).is_none() {
            continue;
        }
        let old_version = old
            .get(path)
            .map(|&(mode, id)| read_version(store, mode, id, None))
            .transpose()?;
        let new_version = new
            .get(path)
            .map(|&(mode, id)| read_version(store, mode, id, work_dir.map(|dir| dir.join(path))))
            .transpose()?;
        out.push_str(&file_patch(
            path,
            old_version.as_ref(),
            new_version.as_ref(),
        ));
    }
    Ok(out)
}

/// ファイルの中身を，`file`があればそこから，なければオブジェクトストアから読む．
fn read_version<S: ObjectStore + ?Sized>(
    store: &S,
    mode: Mode,
    id: ObjectId,
    file: Option<PathBuf>,
) -> Result<FileVersion, Error> {
    let content = match file {
        Some(file) => fs::read(file)?,
        None => store.read(id)?.1,
    };
    Ok(FileVersion { mode, id, content })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::hash_blob;

    fn version(content: &str) -> FileVersion {
        FileVersion {
            mode: Mode::File,
            id: hash_blob(content.as_bytes()),
            content: content.as_bytes().to_vec(),
        }
    }

    #[test]
    fn modified_file_has_index_line_and_hunk() {
        let patch = file_patch(
            "hello.txt",
            Some(&version("hello\n")),
            Some(&version("hello\nworld\n")),
        );
        assert_eq!(
            patch,
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
    fn added_and_deleted_files_use_dev_null() {
        let added = file_patch("n.txt", None, Some(&version("new\n")));
        assert_eq!(
            added,
            "diff --git a/n.txt b/n.txt\n\
             new file mode 100644\n\
             index 0000000..3e75765\n\
             --- /dev/null\n\
             +++ b/n.txt\n\
             @@ -0,0 +1 @@\n\
             +new\n"
        );
        let deleted = file_patch("d.txt", Some(&version("d\n")), None);
        assert_eq!(
            deleted,
            "diff --git a/d.txt b/d.txt\n\
             deleted file mode 100644\n\
             index 4bcfe98..0000000\n\
             --- a/d.txt\n\
             +++ /dev/null\n\
             @@ -1 +0,0 @@\n\
             -d\n"
        );
    }

    #[test]
    fn empty_new_file_has_no_hunk() {
        assert_eq!(
            file_patch("e.txt", None, Some(&version(""))),
            "diff --git a/e.txt b/e.txt\nnew file mode 100644\nindex 0000000..e69de29\n"
        );
    }

    #[test]
    fn missing_newline_at_end_is_marked() {
        let patch = file_patch("a.txt", Some(&version("abc")), Some(&version("abd")));
        assert!(patch.ends_with(
            "@@ -1 +1 @@\n\
             -abc\n\
             \\ No newline at end of file\n\
             +abd\n\
             \\ No newline at end of file\n"
        ));
    }

    #[test]
    fn mode_change_without_content_change_has_no_index_line() {
        let old = version("1\n");
        let new = FileVersion {
            mode: Mode::Executable,
            ..old.clone()
        };
        assert_eq!(
            file_patch("x.sh", Some(&old), Some(&new)),
            "diff --git a/x.sh b/x.sh\nold mode 100644\nnew mode 100755\n"
        );
    }

    #[test]
    fn content_with_nul_is_binary() {
        let patch = file_patch("b.dat", Some(&version("a\0b")), Some(&version("a\0c")));
        assert!(patch.ends_with("Binary files a/b.dat and b/b.dat differ\n"));
        let added = file_patch("b.dat", None, Some(&version("a\0b")));
        assert!(added.ends_with("Binary files /dev/null and b/b.dat differ\n"));
    }
}
