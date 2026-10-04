use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::error::Error;
use crate::object::{ObjectKind, encode, hash_blob, parse_header};
use crate::oid::ObjectId;

/// Gitのリポジトリ．`.git`ディレクトリの場所を持つ．
pub struct Repository {
    git_dir: PathBuf,
}

impl Repository {
    /// `dir`に空のリポジトリを作る．
    pub fn init(dir: &Path) -> Result<Repository, Error> {
        let git_dir = dir.join(".git");
        fs::create_dir_all(git_dir.join("objects"))?;
        fs::create_dir_all(git_dir.join("refs").join("heads"))?;
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;
        Ok(Repository { git_dir })
    }

    /// `start`から親へ順に`.git`を探し，見つけたリポジトリを開く．
    pub fn discover(start: &Path) -> Result<Repository, Error> {
        for dir in start.ancestors() {
            let git_dir = dir.join(".git");
            if git_dir.is_dir() {
                return Ok(Repository { git_dir });
            }
        }
        Err(Error::NotARepository)
    }

    pub fn git_dir(&self) -> &Path {
        &self.git_dir
    }

    /// データをblobオブジェクトとして書き込み，そのIDを返す．
    pub fn write_blob(&self, data: &[u8]) -> Result<ObjectId, Error> {
        let id = hash_blob(data);
        let path = self.object_path(id);
        if path.exists() {
            return Ok(id);
        }
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&encode(ObjectKind::Blob, data))?;
        let compressed = encoder.finish()?;
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, compressed)?;
        Ok(id)
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

    /// オブジェクトのファイルのパス(`.git/objects/ce/013625…`)を返す．
    fn object_path(&self, id: ObjectId) -> PathBuf {
        let hex = id.to_string();
        self.git_dir.join("objects").join(&hex[..2]).join(&hex[2..])
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

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
}
