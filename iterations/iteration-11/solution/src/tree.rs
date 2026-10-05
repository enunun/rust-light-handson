use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

use crate::error::Error;
use crate::index::{Index, IndexEntry};
use crate::object::ObjectKind;
use crate::oid::ObjectId;
use crate::store::ObjectStore;

/// treeのエントリーのモード．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// 通常のファイル(`100644`)
    File,
    /// 実行可能なファイル(`100755`)
    Executable,
    /// シンボリックリンク(`120000`)
    Symlink,
    /// ディレクトリ(`40000`)
    Directory,
    /// サブモジュール(`160000`)
    Submodule,
}

impl Mode {
    /// このモードのエントリーが指すオブジェクトの種類．
    pub fn kind(self) -> ObjectKind {
        match self {
            Mode::Directory => ObjectKind::Tree,
            Mode::Submodule => ObjectKind::Commit,
            Mode::File | Mode::Executable | Mode::Symlink => ObjectKind::Blob,
        }
    }

    /// インデックスに書く数としてのモード(`0o100644`など)．
    pub fn bits(self) -> u32 {
        match self {
            Mode::File => 0o100644,
            Mode::Executable => 0o100755,
            Mode::Symlink => 0o120000,
            Mode::Directory => 0o40000,
            Mode::Submodule => 0o160000,
        }
    }
}

/// インデックスに書かれた数から作る．
impl TryFrom<u32> for Mode {
    type Error = Error;

    fn try_from(bits: u32) -> Result<Mode, Error> {
        match bits {
            0o100644 => Ok(Mode::File),
            0o100755 => Ok(Mode::Executable),
            0o120000 => Ok(Mode::Symlink),
            0o40000 => Ok(Mode::Directory),
            0o160000 => Ok(Mode::Submodule),
            _ => Err(Error::CorruptObject("unknown mode")),
        }
    }
}

/// `ls-tree`と同じく，6桁で表示する(ディレクトリは`040000`)．
impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = match self {
            Mode::File => "100644",
            Mode::Executable => "100755",
            Mode::Symlink => "120000",
            Mode::Directory => "040000",
            Mode::Submodule => "160000",
        };
        f.write_str(digits)
    }
}

/// treeオブジェクトの中のモードの表記(ディレクトリは`40000`)から作る．
impl TryFrom<&[u8]> for Mode {
    type Error = Error;

    fn try_from(digits: &[u8]) -> Result<Mode, Error> {
        match digits {
            b"100644" => Ok(Mode::File),
            b"100755" => Ok(Mode::Executable),
            b"120000" => Ok(Mode::Symlink),
            b"40000" => Ok(Mode::Directory),
            b"160000" => Ok(Mode::Submodule),
            _ => Err(Error::CorruptObject("unknown mode")),
        }
    }
}

/// treeの1つのエントリー．名前は，treeの内容のバイト列を借りている．
#[derive(Debug, PartialEq, Eq)]
pub struct TreeEntry<'a> {
    pub mode: Mode,
    pub name: &'a str,
    pub id: ObjectId,
}

/// `<モード> <種類> <ID>\t<名前>`の形で表示する．
impl fmt::Display for TreeEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}\t{}",
            self.mode,
            self.mode.kind(),
            self.id,
            self.name
        )
    }
}

/// treeオブジェクトの内容を，エントリーの列にする．
pub fn parse_tree(data: &[u8]) -> Result<Vec<TreeEntry<'_>>, Error> {
    let corrupt = || Error::CorruptObject("invalid tree entry");
    let mut entries = Vec::new();
    let mut rest = data;
    while !rest.is_empty() {
        let space = rest
            .iter()
            .position(|byte| *byte == b' ')
            .ok_or_else(corrupt)?;
        let mode = Mode::try_from(&rest[..space])?;
        rest = &rest[space + 1..];

        let nul = rest
            .iter()
            .position(|byte| *byte == 0)
            .ok_or_else(corrupt)?;
        let name = std::str::from_utf8(&rest[..nul]).map_err(|_| corrupt())?;
        rest = &rest[nul + 1..];

        if rest.len() < 20 {
            return Err(corrupt());
        }
        let (id, after) = rest.split_at(20);
        let id: [u8; 20] = id.try_into().unwrap();
        rest = after;

        entries.push(TreeEntry {
            mode,
            name,
            id: ObjectId::from_bytes(id),
        });
    }
    Ok(entries)
}

/// treeのエントリーの並び順．名前のバイト順だが，ディレクトリは名前の後ろに`/`があるものとして比べる．
pub fn compare_entries(a: &TreeEntry, b: &TreeEntry) -> Ordering {
    let slash = |entry: &TreeEntry| (entry.mode == Mode::Directory).then_some(b'/');
    let a_key = a.name.bytes().chain(slash(a));
    let b_key = b.name.bytes().chain(slash(b));
    a_key.cmp(b_key)
}

/// エントリーをGitの順に並べ，treeオブジェクトの内容のバイト列にする．
pub fn tree_bytes(mut entries: Vec<TreeEntry<'_>>) -> Vec<u8> {
    entries.sort_by(compare_entries);
    let mut bytes = Vec::new();
    for entry in &entries {
        bytes.extend_from_slice(format!("{:o} {}\0", entry.mode.bits(), entry.name).as_bytes());
        bytes.extend_from_slice(entry.id.as_bytes());
    }
    bytes
}

/// インデックスからtreeオブジェクトを作って書き込み，最上位のtreeのIDを返す．
pub fn write_tree<S: ObjectStore + ?Sized>(store: &S, index: &Index) -> Result<ObjectId, Error> {
    let entries: Vec<(&str, &IndexEntry)> = index
        .entries()
        .iter()
        .map(|(path, entry)| (path.as_str(), entry))
        .collect();
    write_subtree(store, &entries)
}

/// パスの順に並んだエントリーから，1つのディレクトリのtreeを書く．
/// パスは，このディレクトリからの相対パスである．
fn write_subtree<S: ObjectStore + ?Sized>(
    store: &S,
    entries: &[(&str, &IndexEntry)],
) -> Result<ObjectId, Error> {
    let mut tree = Vec::new();
    let mut rest = entries;
    while let Some(&(path, entry)) = rest.first() {
        match path.split_once('/') {
            None => {
                tree.push(TreeEntry {
                    mode: entry.mode,
                    name: path,
                    id: entry.id,
                });
                rest = &rest[1..];
            }
            Some((dir, _)) => {
                let prefix = format!("{dir}/");
                let end = rest
                    .iter()
                    .position(|(path, _)| !path.starts_with(&prefix))
                    .unwrap_or(rest.len());
                let children: Vec<(&str, &IndexEntry)> = rest[..end]
                    .iter()
                    .map(|&(path, entry)| (&path[prefix.len()..], entry))
                    .collect();
                let id = write_subtree(store, &children)?;
                tree.push(TreeEntry {
                    mode: Mode::Directory,
                    name: dir,
                    id,
                });
                rest = &rest[end..];
            }
        }
    }
    store.write(ObjectKind::Tree, &tree_bytes(tree))
}

/// treeをサブディレクトリまでたどり，ファイルのパスとモードとIDの表にする．
pub fn flatten_tree<S: ObjectStore + ?Sized>(
    store: &S,
    id: ObjectId,
) -> Result<BTreeMap<String, (Mode, ObjectId)>, Error> {
    let mut files = BTreeMap::new();
    collect_tree(store, id, "", &mut files)?;
    Ok(files)
}

fn collect_tree<S: ObjectStore + ?Sized>(
    store: &S,
    id: ObjectId,
    prefix: &str,
    files: &mut BTreeMap<String, (Mode, ObjectId)>,
) -> Result<(), Error> {
    let (kind, content) = store.read(id)?;
    if kind != ObjectKind::Tree {
        return Err(Error::NotATree);
    }
    for entry in parse_tree(&content)? {
        let path = format!("{prefix}{}", entry.name);
        if entry.mode == Mode::Directory {
            collect_tree(store, entry.id, &format!("{path}/"), files)?;
        } else {
            files.insert(path, (entry.mode, entry.id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Stat;
    use crate::object::hash_blob;
    use crate::store::MemoryObjectStore;

    /// treeの内容の1つのエントリーのバイト列を作る．
    fn entry_bytes(mode: &str, name: &str, id: ObjectId) -> Vec<u8> {
        let mut bytes = format!("{mode} {name}\0").into_bytes();
        let hex = id.to_string();
        for i in 0..20 {
            bytes.push(u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap());
        }
        bytes
    }

    #[test]
    fn modes_are_read_from_tree_digits() {
        assert_eq!(Mode::try_from(&b"100644"[..]).unwrap(), Mode::File);
        assert_eq!(Mode::try_from(&b"100755"[..]).unwrap(), Mode::Executable);
        assert_eq!(Mode::try_from(&b"120000"[..]).unwrap(), Mode::Symlink);
        assert_eq!(Mode::try_from(&b"40000"[..]).unwrap(), Mode::Directory);
        assert_eq!(Mode::try_from(&b"160000"[..]).unwrap(), Mode::Submodule);
    }

    #[test]
    fn unknown_mode_is_corrupt() {
        let result = Mode::try_from(&b"100600"[..]);
        assert!(matches!(result, Err(Error::CorruptObject("unknown mode"))));
    }

    #[test]
    fn modes_convert_to_and_from_index_bits() {
        assert_eq!(Mode::Executable.bits(), 0o100755);
        assert_eq!(Mode::try_from(0o100644).unwrap(), Mode::File);
        assert!(Mode::try_from(0o100600).is_err());
    }

    #[test]
    fn directory_mode_is_shown_with_six_digits() {
        assert_eq!(Mode::Directory.to_string(), "040000");
        assert_eq!(Mode::Directory.kind(), ObjectKind::Tree);
    }

    #[test]
    fn empty_tree_has_no_entries() {
        assert_eq!(parse_tree(b"").unwrap(), vec![]);
    }

    #[test]
    fn parses_entries_in_order() {
        let hello = hash_blob(b"hello\n");
        let mut data = entry_bytes("100644", "hello.txt", hello);
        data.extend(entry_bytes("40000", "src", hello));
        let entries = parse_tree(&data).unwrap();
        assert_eq!(
            entries,
            vec![
                TreeEntry {
                    mode: Mode::File,
                    name: "hello.txt",
                    id: hello
                },
                TreeEntry {
                    mode: Mode::Directory,
                    name: "src",
                    id: hello
                },
            ]
        );
    }

    #[test]
    fn truncated_id_is_corrupt() {
        let data = entry_bytes("100644", "hello.txt", hash_blob(b"hello\n"));
        let result = parse_tree(&data[..data.len() - 1]);
        assert!(matches!(
            result,
            Err(Error::CorruptObject("invalid tree entry"))
        ));
    }

    #[test]
    fn name_must_be_utf8() {
        let mut data = b"100644 \xff\0".to_vec();
        data.extend([0u8; 20]);
        assert!(parse_tree(&data).is_err());
    }

    #[test]
    fn entry_is_shown_like_ls_tree() {
        let entry = TreeEntry {
            mode: Mode::Directory,
            name: "src",
            id: hash_blob(b"hello\n"),
        };
        assert_eq!(
            entry.to_string(),
            "040000 tree ce013625030ba8dba906f756967f9e9ca394464a\tsrc"
        );
    }

    fn named(mode: Mode, name: &str) -> TreeEntry<'_> {
        TreeEntry {
            mode,
            name,
            id: hash_blob(b""),
        }
    }

    #[test]
    fn entries_are_ordered_by_name_bytes() {
        let a = named(Mode::File, "a.txt");
        let b = named(Mode::File, "b.txt");
        assert_eq!(compare_entries(&a, &b), Ordering::Less);
        assert_eq!(compare_entries(&b, &a), Ordering::Greater);
    }

    #[test]
    fn directory_is_ordered_as_if_followed_by_slash() {
        let dir = named(Mode::Directory, "a");
        let dash = named(Mode::File, "a-b");
        let dot = named(Mode::File, "a.txt");
        assert_eq!(compare_entries(&dash, &dir), Ordering::Less);
        assert_eq!(compare_entries(&dot, &dir), Ordering::Less);
        let file = named(Mode::File, "a");
        assert_eq!(compare_entries(&file, &dot), Ordering::Less);
    }

    #[test]
    fn tree_bytes_sorts_entries_and_parses_back() {
        let entries = vec![
            named(Mode::Directory, "a"),
            named(Mode::File, "a.txt"),
            named(Mode::Executable, "a-b"),
        ];
        let bytes = tree_bytes(entries);
        let parsed = parse_tree(&bytes).unwrap();
        let names: Vec<&str> = parsed.iter().map(|entry| entry.name).collect();
        assert_eq!(names, ["a-b", "a.txt", "a"]);
        assert_eq!(parsed[0].mode, Mode::Executable);
        assert_eq!(parsed[2].mode, Mode::Directory);
    }

    fn index_of(files: &[(&str, Mode, &[u8])]) -> Index {
        let mut index = Index::default();
        for &(path, mode, data) in files {
            let entry = IndexEntry {
                stat: Stat::default(),
                mode,
                id: hash_blob(data),
            };
            index.insert(path.to_string(), entry);
        }
        index
    }

    #[test]
    fn write_tree_nests_directories() {
        let store = MemoryObjectStore::default();
        let index = index_of(&[
            ("hello.txt", Mode::File, b"hello\n"),
            ("src/bin/tool.rs", Mode::File, b"fn main() {}\n"),
        ]);
        let root = write_tree(&store, &index).unwrap();

        let (kind, content) = store.read(root).unwrap();
        assert_eq!(kind, ObjectKind::Tree);
        let entries = parse_tree(&content).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(
            (entries[0].name, entries[0].mode),
            ("hello.txt", Mode::File)
        );
        assert_eq!((entries[1].name, entries[1].mode), ("src", Mode::Directory));

        let (_, src) = store.read(entries[1].id).unwrap();
        let src_entries = parse_tree(&src).unwrap();
        assert_eq!(
            (src_entries[0].name, src_entries[0].mode),
            ("bin", Mode::Directory)
        );
    }

    #[test]
    fn write_tree_of_empty_index_is_empty_tree() {
        let store = MemoryObjectStore::default();
        let id = write_tree(&store, &Index::default()).unwrap();
        assert_eq!(id.to_string(), "4b825dc642cb6eb9a060e54bf8d69288fbee4904");
    }

    #[test]
    fn flatten_tree_lists_files_with_full_paths() {
        let store = MemoryObjectStore::default();
        let index = index_of(&[
            ("hello.txt", Mode::File, b"hello\n"),
            ("src/bin/tool.rs", Mode::Executable, b"fn main() {}\n"),
        ]);
        let root = write_tree(&store, &index).unwrap();
        let files = flatten_tree(&store, root).unwrap();
        let expected: BTreeMap<String, (Mode, ObjectId)> = index
            .entries()
            .iter()
            .map(|(path, entry)| (path.clone(), (entry.mode, entry.id)))
            .collect();
        assert_eq!(files, expected);
    }
}
