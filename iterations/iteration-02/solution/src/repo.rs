use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::error::Error;
use crate::object::{blob_bytes, hash_blob};
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
        encoder.write_all(&blob_bytes(data))?;
        let compressed = encoder.finish()?;
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, compressed)?;
        Ok(id)
    }

    /// オブジェクトのファイルのパス(`.git/objects/ce/013625…`)を返す．
    fn object_path(&self, id: ObjectId) -> PathBuf {
        let hex = id.to_string();
        self.git_dir.join("objects").join(&hex[..2]).join(&hex[2..])
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use flate2::read::ZlibDecoder;
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
}
