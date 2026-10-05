use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

use crate::commit::read_commit;
use crate::error::Error;
use crate::index::Index;
use crate::object::hash_blob;
use crate::oid::ObjectId;
use crate::parallel::map_parallel;
use crate::store::ObjectStore;
use crate::tree::{Mode, flatten_tree};
use crate::worktree::{file_mode, list_files};

/// 2つの状態の間の，1つのファイルの違い．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Added,
    Modified,
    Deleted,
}

impl Change {
    /// `status`の1文字の表記．
    fn code(self) -> char {
        match self {
            Change::Added => 'A',
            Change::Modified => 'M',
            Change::Deleted => 'D',
        }
    }
}

/// `status`の1行．
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusEntry {
    /// 追跡しているファイルの変更．`staged`はHEADとインデックス，`unstaged`はインデックスと作業ディレクトリの違いである．
    Changed {
        path: String,
        staged: Option<Change>,
        unstaged: Option<Change>,
    },
    /// インデックスにないファイル．
    Untracked(String),
}

/// `git status --porcelain`と同じ`XY パス`の形で表示する．
impl fmt::Display for StatusEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusEntry::Changed {
                path,
                staged,
                unstaged,
            } => {
                let code = |change: &Option<Change>| change.map_or(' ', Change::code);
                write!(f, "{}{} {path}", code(staged), code(unstaged))
            }
            StatusEntry::Untracked(path) => write!(f, "?? {path}"),
        }
    }
}

/// HEADのコミット，インデックス，作業ディレクトリを比べ，変更のあるファイルを返す．
/// 変更のあるファイルをパスの順に並べ，そのあとに追跡していないファイルをパスの順に並べる．
/// 作業ディレクトリのファイルのハッシュは，`jobs`個までのスレッドで計算する．
pub fn status<S: ObjectStore + ?Sized>(
    store: &S,
    head: Option<ObjectId>,
    index: &Index,
    work_dir: &Path,
    jobs: usize,
) -> Result<Vec<StatusEntry>, Error> {
    let head_files = head_files(store, head)?;
    let index_files = index_files(index);
    let work_files = work_tree_files(work_dir, jobs)?;

    let mut entries = Vec::new();
    let paths: BTreeSet<&String> = head_files.keys().chain(index_files.keys()).collect();
    for path in paths {
        let staged = compare(head_files.get(path), index_files.get(path));
        let unstaged = match index_files.get(path) {
            Some(_) => compare(index_files.get(path), work_files.get(path)),
            None => None,
        };
        if staged.is_some() || unstaged.is_some() {
            entries.push(StatusEntry::Changed {
                path: path.clone(),
                staged,
                unstaged,
            });
        }
    }
    for path in work_files.keys() {
        if !index_files.contains_key(path) {
            entries.push(StatusEntry::Untracked(path.clone()));
        }
    }
    Ok(entries)
}

/// HEADのコミットのファイルを，パスとモードとIDの表にする．HEADがなければ空の表を返す．
pub fn head_files<S: ObjectStore + ?Sized>(
    store: &S,
    head: Option<ObjectId>,
) -> Result<BTreeMap<String, (Mode, ObjectId)>, Error> {
    match head {
        Some(commit) => flatten_tree(store, read_commit(store, commit)?.tree),
        None => Ok(BTreeMap::new()),
    }
}

/// インデックスのファイルを，パスとモードとIDの表にする．
pub fn index_files(index: &Index) -> BTreeMap<String, (Mode, ObjectId)> {
    index
        .entries()
        .iter()
        .map(|(path, entry)| (path.clone(), (entry.mode, entry.id)))
        .collect()
}

/// 古い状態と新しい状態の，1つのファイルの違い．
pub fn compare(old: Option<&(Mode, ObjectId)>, new: Option<&(Mode, ObjectId)>) -> Option<Change> {
    match (old, new) {
        (None, None) => None,
        (None, Some(_)) => Some(Change::Added),
        (Some(_), None) => Some(Change::Deleted),
        (Some(old), Some(new)) if old != new => Some(Change::Modified),
        (Some(_), Some(_)) => None,
    }
}

/// 作業ディレクトリのファイルを読み，パスとモードとIDの表にする．ハッシュは`jobs`個までのスレッドで計算する．
pub fn work_tree_files(
    work_dir: &Path,
    jobs: usize,
) -> Result<BTreeMap<String, (Mode, ObjectId)>, Error> {
    let paths = list_files(work_dir, work_dir)?;
    let hashed = map_parallel(&paths, jobs, |path| -> Result<(Mode, ObjectId), Error> {
        let full_path = work_dir.join(path);
        let mode = file_mode(&fs::metadata(&full_path)?);
        Ok((mode, hash_blob(&fs::read(&full_path)?)))
    });
    paths
        .into_iter()
        .zip(hashed)
        .map(|(path, file)| Ok((path, file?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::commit::{Commit, Signature};
    use crate::index::{IndexEntry, Stat};
    use crate::object::ObjectKind;
    use crate::store::MemoryObjectStore;
    use crate::tree::write_tree;

    /// 作業ディレクトリとインデックスとオブジェクトストアを，まとめて扱う．
    struct Fixture {
        dir: TempDir,
        store: MemoryObjectStore,
        index: Index,
        head: Option<ObjectId>,
    }

    impl Fixture {
        fn new() -> Fixture {
            Fixture {
                dir: TempDir::new().unwrap(),
                store: MemoryObjectStore::default(),
                index: Index::default(),
                head: None,
            }
        }

        fn write(&self, path: &str, text: &str) {
            let full_path = self.dir.path().join(path);
            fs::create_dir_all(full_path.parent().unwrap()).unwrap();
            fs::write(full_path, text).unwrap();
        }

        /// 作業ディレクトリのファイルをインデックスに登録する．
        fn add(&mut self, path: &str) {
            let data = fs::read(self.dir.path().join(path)).unwrap();
            let id = self.store.write(ObjectKind::Blob, &data).unwrap();
            let entry = IndexEntry {
                stat: Stat::default(),
                mode: Mode::File,
                id,
            };
            self.index.insert(path.to_string(), entry);
        }

        /// インデックスの中身をコミットし，HEADにする．
        fn commit(&mut self) {
            let tree = write_tree(&self.store, &self.index).unwrap();
            let alice = Signature {
                name: String::from("Alice"),
                email: String::from("alice@example.com"),
                time: 0,
                offset_minutes: 0,
            };
            let mut builder = Commit::builder()
                .tree(tree)
                .author(alice.clone())
                .committer(alice)
                .message("commit\n");
            if let Some(parent) = self.head {
                builder = builder.parent(parent);
            }
            let bytes = builder.build().to_bytes();
            self.head = Some(self.store.write(ObjectKind::Commit, &bytes).unwrap());
        }

        fn status(&self) -> Vec<String> {
            status(&self.store, self.head, &self.index, self.dir.path(), 2)
                .unwrap()
                .iter()
                .map(|entry| entry.to_string())
                .collect()
        }
    }

    #[test]
    fn clean_work_tree_has_no_entries() {
        let mut fixture = Fixture::new();
        fixture.write("hello.txt", "hello\n");
        fixture.add("hello.txt");
        fixture.commit();
        assert_eq!(fixture.status(), Vec::<String>::new());
    }

    #[test]
    fn added_file_without_commit_is_staged_addition() {
        let mut fixture = Fixture::new();
        fixture.write("hello.txt", "hello\n");
        fixture.add("hello.txt");
        assert_eq!(fixture.status(), ["A  hello.txt"]);
    }

    #[test]
    fn modified_file_is_unstaged_then_staged() {
        let mut fixture = Fixture::new();
        fixture.write("hello.txt", "hello\n");
        fixture.add("hello.txt");
        fixture.commit();
        fixture.write("hello.txt", "hello\nworld\n");
        assert_eq!(fixture.status(), [" M hello.txt"]);
        fixture.add("hello.txt");
        assert_eq!(fixture.status(), ["M  hello.txt"]);
        fixture.write("hello.txt", "changed again\n");
        assert_eq!(fixture.status(), ["MM hello.txt"]);
    }

    #[test]
    fn deleted_file_is_unstaged_then_staged_deletion() {
        let mut fixture = Fixture::new();
        fixture.write("hello.txt", "hello\n");
        fixture.add("hello.txt");
        fixture.commit();
        fs::remove_file(fixture.dir.path().join("hello.txt")).unwrap();
        assert_eq!(fixture.status(), [" D hello.txt"]);
        fixture.index.remove("hello.txt");
        assert_eq!(fixture.status(), ["D  hello.txt"]);
    }

    #[test]
    fn untracked_files_come_after_changes() {
        let mut fixture = Fixture::new();
        fixture.write("b.txt", "b\n");
        fixture.add("b.txt");
        fixture.write("a.txt", "a\n");
        fixture.write("src/c.txt", "c\n");
        assert_eq!(fixture.status(), ["A  b.txt", "?? a.txt", "?? src/c.txt"]);
    }

    #[test]
    fn compare_detects_mode_change() {
        let id = hash_blob(b"x");
        let file = (Mode::File, id);
        let executable = (Mode::Executable, id);
        assert_eq!(
            compare(Some(&file), Some(&executable)),
            Some(Change::Modified)
        );
        assert_eq!(compare(Some(&file), Some(&file)), None);
    }
}
