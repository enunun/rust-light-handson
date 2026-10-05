use std::collections::HashMap;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::error::Error;
use crate::object::{ObjectKind, encode, hash_object, parse_header};
use crate::oid::ObjectId;

/// オブジェクトを読み書きする場所．
pub trait ObjectStore {
    /// オブジェクトを読み，種類と内容を返す．なければ`ObjectNotFound`になる．
    fn read(&self, id: ObjectId) -> Result<(ObjectKind, Vec<u8>), Error>;

    /// オブジェクトを書き込み，そのIDを返す．
    fn write(&mut self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error>;

    /// IDが`prefix`(小文字の16進数)で始まるオブジェクトのIDを返す．
    fn find(&self, prefix: &str) -> Result<Vec<ObjectId>, Error>;
}

/// `.git/objects`の下のゆるいオブジェクト．
pub struct LooseObjectStore {
    objects_dir: PathBuf,
}

impl LooseObjectStore {
    pub fn new(objects_dir: &Path) -> LooseObjectStore {
        LooseObjectStore {
            objects_dir: objects_dir.to_path_buf(),
        }
    }

    /// オブジェクトのファイルのパス(`.git/objects/ce/013625…`)を返す．
    fn object_path(&self, id: ObjectId) -> PathBuf {
        let hex = id.to_string();
        self.objects_dir.join(&hex[..2]).join(&hex[2..])
    }
}

impl ObjectStore for LooseObjectStore {
    fn read(&self, id: ObjectId) -> Result<(ObjectKind, Vec<u8>), Error> {
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

    fn write(&mut self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error> {
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

    fn find(&self, prefix: &str) -> Result<Vec<ObjectId>, Error> {
        let (dir_name, rest) = prefix.split_at(2);
        let dir = self.objects_dir.join(dir_name);
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut found = Vec::new();
        for entry in fs::read_dir(dir)? {
            let name = entry?.file_name();
            if let Some(name) = name.to_str()
                && name.starts_with(rest)
                && let Ok(id) = format!("{dir_name}{name}").parse()
            {
                found.push(id);
            }
        }
        found.sort();
        Ok(found)
    }
}

/// メモリーの上のオブジェクト．テストで使う．
#[derive(Debug, Default)]
pub struct MemoryObjectStore {
    objects: HashMap<ObjectId, (ObjectKind, Vec<u8>)>,
}

impl ObjectStore for MemoryObjectStore {
    fn read(&self, id: ObjectId) -> Result<(ObjectKind, Vec<u8>), Error> {
        self.objects
            .get(&id)
            .cloned()
            .ok_or_else(|| Error::ObjectNotFound(id.to_string()))
    }

    fn write(&mut self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error> {
        let id = hash_object(kind, data);
        self.objects.insert(id, (kind, data.to_vec()));
        Ok(id)
    }

    fn find(&self, prefix: &str) -> Result<Vec<ObjectId>, Error> {
        let mut found: Vec<ObjectId> = self
            .objects
            .keys()
            .filter(|id| id.to_string().starts_with(prefix))
            .copied()
            .collect();
        found.sort();
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::object::hash_blob;

    /// どのオブジェクトストアにも成り立つ振る舞いを確かめる．
    fn check_store(store: &mut dyn ObjectStore) {
        let id = store.write(ObjectKind::Blob, b"hello\n").unwrap();
        assert_eq!(id, hash_blob(b"hello\n"));
        assert_eq!(
            store.read(id).unwrap(),
            (ObjectKind::Blob, b"hello\n".to_vec())
        );
        assert_eq!(store.find("ce01").unwrap(), [id]);
        assert_eq!(store.find("ffff").unwrap(), []);
        let missing = hash_blob(b"missing");
        assert!(matches!(
            store.read(missing),
            Err(Error::ObjectNotFound(name)) if name == missing.to_string()
        ));
    }

    #[test]
    fn memory_store_reads_what_it_wrote() {
        check_store(&mut MemoryObjectStore::default());
    }

    #[test]
    fn loose_store_reads_what_it_wrote() {
        let dir = TempDir::new().unwrap();
        check_store(&mut LooseObjectStore::new(dir.path()));
    }

    #[test]
    fn loose_store_writes_compressed_file_under_id() {
        let dir = TempDir::new().unwrap();
        let mut store = LooseObjectStore::new(dir.path());
        store.write(ObjectKind::Blob, b"hello\n").unwrap();
        let path = dir.path().join("ce/013625030ba8dba906f756967f9e9ca394464a");
        let mut content = Vec::new();
        ZlibDecoder::new(fs::File::open(path).unwrap())
            .read_to_end(&mut content)
            .unwrap();
        assert_eq!(content, b"blob 6\0hello\n");
    }

    #[test]
    fn find_returns_all_objects_with_prefix() {
        let mut store = MemoryObjectStore::default();
        // 2つのblobのIDは，どちらも6bb2で始まる．
        let a = store.write(ObjectKind::Blob, b"195\n").unwrap();
        let b = store.write(ObjectKind::Blob, b"389\n").unwrap();
        assert_eq!(store.find("6bb2").unwrap(), [b, a]);
    }
}
