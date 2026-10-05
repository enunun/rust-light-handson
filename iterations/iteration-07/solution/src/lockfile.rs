use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use crate::error::Error;

/// ファイルを置き換えるためのロック．`<ファイル名>.lock`に書き，`commit`で名前を変えて置き換える．
/// `commit`せずに捨てると，`.lock`のファイルを消す．
pub struct LockFile {
    path: PathBuf,
    lock_path: PathBuf,
    file: File,
    committed: bool,
}

impl LockFile {
    /// `<path>.lock`を作る．すでにあれば，ほかの処理が更新中なのでエラーにする．
    pub fn acquire(path: &Path) -> Result<LockFile, Error> {
        let mut lock_path = path.as_os_str().to_owned();
        lock_path.push(".lock");
        let lock_path = PathBuf::from(lock_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                return Err(Error::Locked(lock_path.display().to_string()));
            }
            Err(error) => return Err(error.into()),
        };
        Ok(LockFile {
            path: path.to_path_buf(),
            lock_path,
            file,
            committed: false,
        })
    }

    pub fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        self.file.write_all(data)?;
        Ok(())
    }

    /// 書いた内容で，元のファイルを置き換える．
    pub fn commit(mut self) -> Result<(), Error> {
        fs::rename(&self.lock_path, &self.path)?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(&self.lock_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn commit_replaces_file_and_removes_lock() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("main");
        fs::write(&path, "old\n").unwrap();
        let mut lock = LockFile::acquire(&path).unwrap();
        lock.write_all(b"new\n").unwrap();
        assert!(dir.path().join("main.lock").exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), "old\n");
        lock.commit().unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "new\n");
        assert!(!dir.path().join("main.lock").exists());
    }

    #[test]
    fn dropping_without_commit_removes_lock_and_keeps_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("main");
        fs::write(&path, "old\n").unwrap();
        {
            let mut lock = LockFile::acquire(&path).unwrap();
            lock.write_all(b"new\n").unwrap();
        }
        assert_eq!(fs::read_to_string(&path).unwrap(), "old\n");
        assert!(!dir.path().join("main.lock").exists());
    }

    #[test]
    fn existing_lock_is_an_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("main");
        let _first = LockFile::acquire(&path).unwrap();
        let second = LockFile::acquire(&path);
        assert!(matches!(second, Err(Error::Locked(_))));
    }

    #[test]
    fn creates_parent_directories() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("refs/heads/feature/login");
        LockFile::acquire(&path).unwrap().commit().unwrap();
        assert!(path.exists());
    }
}
