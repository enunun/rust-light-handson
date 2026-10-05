use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::commit::Commit;
use crate::error::Error;
use crate::index::{Index, IndexEntry, Stat};
use crate::lockfile::LockFile;
use crate::object::{ObjectKind, encode, hash_object, parse_header};
use crate::oid::ObjectId;
use crate::refs::{Ref, RefName};
use crate::tree::{Mode, TreeEntry, tree_bytes};
use crate::worktree::{list_files, relative_path};

/// Gitのリポジトリ．作業ディレクトリと`.git`ディレクトリの場所を持つ．
pub struct Repository {
    work_dir: PathBuf,
    git_dir: PathBuf,
}

impl Repository {
    /// `dir`に空のリポジトリを作る．
    pub fn init(dir: &Path) -> Result<Repository, Error> {
        let git_dir = dir.join(".git");
        fs::create_dir_all(git_dir.join("objects"))?;
        fs::create_dir_all(git_dir.join("refs").join("heads"))?;
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;
        Ok(Repository {
            work_dir: dir.to_path_buf(),
            git_dir,
        })
    }

    /// `start`から親へ順に`.git`を探し，見つけたリポジトリを開く．
    pub fn discover(start: &Path) -> Result<Repository, Error> {
        for dir in start.ancestors() {
            let git_dir = dir.join(".git");
            if git_dir.is_dir() {
                return Ok(Repository {
                    work_dir: dir.to_path_buf(),
                    git_dir,
                });
            }
        }
        Err(Error::NotARepository)
    }

    pub fn git_dir(&self) -> &Path {
        &self.git_dir
    }

    pub fn index_path(&self) -> PathBuf {
        self.git_dir.join("index")
    }

    /// `cwd`からの相対パスで指定したファイルやディレクトリをblobとして書き込み，インデックスに登録する．
    /// 指定したパスの下で作業ディレクトリから消えたファイルは，インデックスから除く．
    pub fn add(&self, cwd: &Path, pathspecs: &[PathBuf]) -> Result<(), Error> {
        let mut index = Index::load(&self.index_path())?;
        for spec in pathspecs {
            let spec_name = spec.display().to_string();
            let path = cwd.join(spec);
            let prefix = relative_path(&self.work_dir, &path)
                .ok_or_else(|| Error::OutsideRepository(spec_name.clone()))?;
            let files = list_files(&self.work_dir, &path)?;
            let tracked: Vec<String> = index
                .entries()
                .keys()
                .filter(|tracked| is_under(tracked, &prefix))
                .cloned()
                .collect();
            if files.is_empty() && tracked.is_empty() {
                return Err(Error::PathspecNotMatched(spec_name));
            }
            for tracked in tracked {
                if !files.contains(&tracked) {
                    index.remove(&tracked);
                }
            }
            for file in files {
                let full_path = self.work_dir.join(&file);
                let metadata = fs::metadata(&full_path)?;
                let id = self.write_blob(&fs::read(&full_path)?)?;
                let mode = if metadata.permissions().mode() & 0o111 != 0 {
                    Mode::Executable
                } else {
                    Mode::File
                };
                let stat = Stat::from_metadata(&metadata);
                index.insert(file, IndexEntry { stat, mode, id });
            }
        }
        index.save(&self.index_path())
    }

    /// データをblobオブジェクトとして書き込み，そのIDを返す．
    pub fn write_blob(&self, data: &[u8]) -> Result<ObjectId, Error> {
        self.write_object(ObjectKind::Blob, data)
    }

    /// オブジェクトを書き込み，そのIDを返す．
    pub fn write_object(&self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error> {
        let id = hash_object(kind, data);
        let path = self.object_path(id);
        if path.exists() {
            return Ok(id);
        }
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&encode(kind, data))?;
        let compressed = encoder.finish()?;
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, compressed)?;
        Ok(id)
    }

    /// インデックスからtreeオブジェクトを作って書き込み，最上位のtreeのIDを返す．
    pub fn write_tree(&self, index: &Index) -> Result<ObjectId, Error> {
        let entries: Vec<(&str, &IndexEntry)> = index
            .entries()
            .iter()
            .map(|(path, entry)| (path.as_str(), entry))
            .collect();
        self.write_subtree(&entries)
    }

    /// パスの順に並んだエントリーから，1つのディレクトリのtreeを書く．
    /// パスは，このディレクトリからの相対パスである．
    fn write_subtree(&self, entries: &[(&str, &IndexEntry)]) -> Result<ObjectId, Error> {
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
                    let id = self.write_subtree(&children)?;
                    tree.push(TreeEntry {
                        mode: Mode::Directory,
                        name: dir,
                        id,
                    });
                    rest = &rest[end..];
                }
            }
        }
        self.write_object(ObjectKind::Tree, &tree_bytes(tree))
    }

    /// オブジェクトを読み，種類と内容を返す．
    pub fn read_object(&self, id: ObjectId) -> Result<(ObjectKind, Vec<u8>), Error> {
        let file = match fs::File::open(self.object_path(id)) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Err(Error::ObjectNotFound(id.to_string()));
            }
            Err(error) => return Err(error.into()),
        };
        let mut data = Vec::new();
        ZlibDecoder::new(file).read_to_end(&mut data)?;
        let (kind, content) = parse_header(&data)?;
        Ok((kind, content.to_vec()))
    }

    /// commitオブジェクトを読む．
    pub fn read_commit(&self, id: ObjectId) -> Result<Commit, Error> {
        match self.read_object(id)? {
            (ObjectKind::Commit, content) => Commit::parse(&content),
            _ => Err(Error::NotACommit(id.to_string())),
        }
    }

    /// 4桁以上40桁以下の16進数を，それで始まるただ1つのオブジェクトのIDにする．
    pub fn resolve_prefix(&self, prefix: &str) -> Result<ObjectId, Error> {
        let not_found = || Error::ObjectNotFound(prefix.to_string());
        let is_hex = prefix.chars().all(|ch| ch.is_ascii_hexdigit());
        if prefix.len() < 4 || prefix.len() > 40 || !is_hex {
            return Err(not_found());
        }
        let prefix_lower = prefix.to_ascii_lowercase();
        let (dir_name, rest) = prefix_lower.split_at(2);
        let dir = self.git_dir.join("objects").join(dir_name);
        if !dir.is_dir() {
            return Err(not_found());
        }
        let mut found = Vec::new();
        for entry in fs::read_dir(dir)? {
            let name = entry?.file_name();
            if let Some(name) = name.to_str()
                && name.starts_with(rest)
            {
                found.push(format!("{dir_name}{name}"));
            }
        }
        match found.len() {
            0 => Err(not_found()),
            1 => found[0].parse().map_err(|_| not_found()),
            _ => Err(Error::AmbiguousObject(prefix.to_string())),
        }
    }

    /// 参照を読む．参照のファイルがなければ`None`を返す．
    pub fn read_ref(&self, name: &RefName) -> Result<Option<Ref>, Error> {
        match fs::read_to_string(self.git_dir.join(name.as_str())) {
            Ok(text) => Ok(Some(Ref::parse(&text)?)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// シンボリック参照をたどり，最後に`Direct`の参照を持つ参照名を返す．
    /// `HEAD`がまだコミットのないブランチを指していれば，そのブランチ名を返す．
    pub fn final_ref_name(&self, name: &RefName) -> Result<RefName, Error> {
        let mut name = name.clone();
        for _ in 0..5 {
            match self.read_ref(&name)? {
                Some(Ref::Symbolic(target)) => name = target,
                Some(Ref::Direct(_)) | None => return Ok(name),
            }
        }
        Err(Error::InvalidRefName(name.to_string()))
    }

    /// 参照が指すコミットのIDを返す．参照がなければ`None`を返す．
    pub fn resolve_ref(&self, name: &RefName) -> Result<Option<ObjectId>, Error> {
        match self.read_ref(&self.final_ref_name(name)?)? {
            Some(Ref::Direct(id)) => Ok(Some(id)),
            _ => Ok(None),
        }
    }

    /// 参照をIDに更新する．ロックファイルを通して書く．
    pub fn update_ref(&self, name: &RefName, id: ObjectId) -> Result<(), Error> {
        let mut lock = LockFile::acquire(&self.git_dir.join(name.as_str()))?;
        lock.write_all(format!("{id}\n").as_bytes())?;
        lock.commit()
    }

    /// ブランチ名を名前の順に返す．
    pub fn branches(&self) -> Result<Vec<String>, Error> {
        let heads = self.git_dir.join("refs").join("heads");
        let mut names = list_files(&heads, &heads)?;
        names.sort();
        Ok(names)
    }

    /// コミットを書き込み，`HEAD`が指すブランチ(切り離された`HEAD`なら`HEAD`)をそのコミットに進める．
    pub fn commit(&self, commit: Commit) -> Result<ObjectId, Error> {
        let id = self.write_object(ObjectKind::Commit, &commit.to_bytes())?;
        let target = self.final_ref_name(&RefName::head())?;
        self.update_ref(&target, id)?;
        Ok(id)
    }

    /// オブジェクトのファイルのパス(`.git/objects/ce/013625…`)を返す．
    fn object_path(&self, id: ObjectId) -> PathBuf {
        let hex = id.to_string();
        self.git_dir.join("objects").join(&hex[..2]).join(&hex[2..])
    }
}

/// `path`が`prefix`そのものか，`prefix`のディレクトリの下にあるか．`prefix`が空なら，すべてのパスが当てはまる．
fn is_under(path: &str, prefix: &str) -> bool {
    prefix.is_empty() || path == prefix || path.starts_with(&format!("{prefix}/"))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::object::hash_blob;

    #[test]
    fn init_creates_git_directories_and_head() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        assert_eq!(repo.git_dir(), dir.path().join(".git"));
        assert!(repo.git_dir().join("objects").is_dir());
        assert!(repo.git_dir().join("refs/heads").is_dir());
        assert_eq!(
            fs::read_to_string(repo.git_dir().join("HEAD")).unwrap(),
            "ref: refs/heads/main\n"
        );
    }

    #[test]
    fn discover_finds_repository_in_parent_directory() {
        let dir = TempDir::new().unwrap();
        Repository::init(dir.path()).unwrap();
        let sub = dir.path().join("a/b");
        fs::create_dir_all(&sub).unwrap();
        let repo = Repository::discover(&sub).unwrap();
        assert_eq!(repo.git_dir(), dir.path().join(".git"));
    }

    #[test]
    fn discover_fails_outside_repository() {
        let dir = TempDir::new().unwrap();
        let result = Repository::discover(dir.path());
        assert!(matches!(result, Err(Error::NotARepository)));
    }

    #[test]
    fn write_blob_stores_compressed_object_under_its_id() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let id = repo.write_blob(b"hello\n").unwrap();
        let path = repo
            .git_dir()
            .join("objects/ce/013625030ba8dba906f756967f9e9ca394464a");
        let mut content = Vec::new();
        ZlibDecoder::new(fs::File::open(path).unwrap())
            .read_to_end(&mut content)
            .unwrap();
        assert_eq!(id.to_string(), "ce013625030ba8dba906f756967f9e9ca394464a");
        assert_eq!(content, b"blob 6\0hello\n");
    }

    #[test]
    fn read_object_returns_kind_and_content_of_written_blob() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let id = repo.write_blob(b"hello\n").unwrap();
        let (kind, content) = repo.read_object(id).unwrap();
        assert_eq!(kind, ObjectKind::Blob);
        assert_eq!(content, b"hello\n");
    }

    #[test]
    fn read_object_reports_missing_object() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let id = hash_blob(b"hello\n");
        let result = repo.read_object(id);
        assert!(matches!(result, Err(Error::ObjectNotFound(name)) if name == id.to_string()));
    }

    #[test]
    fn resolve_prefix_finds_unique_object() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let id = repo.write_blob(b"hello\n").unwrap();
        assert_eq!(repo.resolve_prefix("ce01").unwrap(), id);
        assert_eq!(repo.resolve_prefix("CE01362").unwrap(), id);
        assert_eq!(repo.resolve_prefix(&id.to_string()).unwrap(), id);
    }

    #[test]
    fn resolve_prefix_rejects_too_short_prefix() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        repo.write_blob(b"hello\n").unwrap();
        let result = repo.resolve_prefix("ce0");
        assert!(matches!(result, Err(Error::ObjectNotFound(_))));
    }

    #[test]
    fn resolve_prefix_reports_missing_object() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let result = repo.resolve_prefix("ce01");
        assert!(matches!(result, Err(Error::ObjectNotFound(_))));
    }

    #[test]
    fn resolve_prefix_reports_ambiguous_prefix() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        // 2つのblobのIDは，どちらも6bb2で始まる．
        repo.write_blob(b"195\n").unwrap();
        repo.write_blob(b"389\n").unwrap();
        let result = repo.resolve_prefix("6bb2");
        assert!(matches!(result, Err(Error::AmbiguousObject(_))));
        assert!(repo.resolve_prefix("6bb2f9").is_ok());
    }

    #[test]
    fn add_registers_files_under_directory() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
        repo.add(dir.path(), &[PathBuf::from("src")]).unwrap();
        let index = Index::load(&repo.index_path()).unwrap();
        let paths: Vec<&String> = index.entries().keys().collect();
        assert_eq!(paths, ["src/main.rs"]);
        let entry = index.entries()["src/main.rs"];
        assert_eq!(entry.mode, Mode::File);
        assert_eq!(entry.id, hash_blob(b"fn main() {}\n"));
        assert_eq!(entry.stat.size, 13);
        assert!(repo.read_object(entry.id).is_ok());
    }

    #[test]
    fn add_records_executable_mode() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let script = dir.path().join("run.sh");
        fs::write(&script, "echo hi\n").unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
        repo.add(dir.path(), &[PathBuf::from("run.sh")]).unwrap();
        let index = Index::load(&repo.index_path()).unwrap();
        assert_eq!(index.entries()["run.sh"].mode, Mode::Executable);
    }

    #[test]
    fn add_removes_deleted_files_under_pathspec() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("a.txt"), "a\n").unwrap();
        fs::write(dir.path().join("b.txt"), "b\n").unwrap();
        repo.add(dir.path(), &[PathBuf::from(".")]).unwrap();
        fs::remove_file(dir.path().join("a.txt")).unwrap();
        repo.add(dir.path(), &[PathBuf::from(".")]).unwrap();
        let index = Index::load(&repo.index_path()).unwrap();
        let paths: Vec<&String> = index.entries().keys().collect();
        assert_eq!(paths, ["b.txt"]);
    }

    #[test]
    fn add_of_unknown_path_is_an_error() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let result = repo.add(dir.path(), &[PathBuf::from("nothing")]);
        assert!(matches!(result, Err(Error::PathspecNotMatched(spec)) if spec == "nothing"));
    }

    #[test]
    fn is_under_matches_directory_prefix_only() {
        assert!(is_under("src/main.rs", "src"));
        assert!(is_under("src", "src"));
        assert!(!is_under("srcs/a", "src"));
        assert!(is_under("anything", ""));
    }

    #[test]
    fn write_tree_nests_directories() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        fs::create_dir_all(dir.path().join("src/bin")).unwrap();
        fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
        fs::write(dir.path().join("src/bin/tool.rs"), "fn main() {}\n").unwrap();
        repo.add(dir.path(), &[PathBuf::from(".")]).unwrap();
        let index = Index::load(&repo.index_path()).unwrap();
        let root = repo.write_tree(&index).unwrap();

        let (kind, content) = repo.read_object(root).unwrap();
        assert_eq!(kind, ObjectKind::Tree);
        let entries = crate::tree::parse_tree(&content).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(
            (entries[0].name, entries[0].mode),
            ("hello.txt", Mode::File)
        );
        assert_eq!((entries[1].name, entries[1].mode), ("src", Mode::Directory));

        let (_, src) = repo.read_object(entries[1].id).unwrap();
        let src_entries = crate::tree::parse_tree(&src).unwrap();
        assert_eq!(
            (src_entries[0].name, src_entries[0].mode),
            ("bin", Mode::Directory)
        );
    }

    #[test]
    fn write_tree_of_empty_index_is_empty_tree() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let id = repo.write_tree(&Index::default()).unwrap();
        assert_eq!(id.to_string(), "4b825dc642cb6eb9a060e54bf8d69288fbee4904");
    }

    #[test]
    fn head_of_new_repository_points_to_unborn_main() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let head = RefName::head();
        assert_eq!(
            repo.read_ref(&head).unwrap(),
            Some(Ref::Symbolic(RefName::branch("main").unwrap()))
        );
        assert_eq!(
            repo.final_ref_name(&head).unwrap().as_str(),
            "refs/heads/main"
        );
        assert_eq!(repo.resolve_ref(&head).unwrap(), None);
    }

    #[test]
    fn updated_branch_is_resolved_through_head() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let id = hash_blob(b"hello\n");
        repo.update_ref(&RefName::branch("main").unwrap(), id)
            .unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join(".git/refs/heads/main")).unwrap(),
            format!("{id}\n")
        );
        assert_eq!(repo.resolve_ref(&RefName::head()).unwrap(), Some(id));
    }

    #[test]
    fn branches_are_listed_in_name_order() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let id = hash_blob(b"hello\n");
        for name in ["topic", "main", "feature/login"] {
            repo.update_ref(&RefName::branch(name).unwrap(), id)
                .unwrap();
        }
        assert_eq!(repo.branches().unwrap(), ["feature/login", "main", "topic"]);
    }
}
