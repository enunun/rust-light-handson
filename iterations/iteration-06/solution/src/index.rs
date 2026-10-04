use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use sha1::{Digest, Sha1};

use crate::error::Error;
use crate::oid::ObjectId;
use crate::tree::Mode;

/// インデックスに記録するファイルの状態．Gitは，これで変更のないファイルを見分ける．
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stat {
    pub ctime: u32,
    pub ctime_nsec: u32,
    pub mtime: u32,
    pub mtime_nsec: u32,
    pub dev: u32,
    pub ino: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
}

impl Stat {
    /// ファイルのメタデータから作る．インデックスの欄に合わせて，32ビットに切り詰める．
    pub fn from_metadata(meta: &fs::Metadata) -> Stat {
        Stat {
            ctime: meta.ctime() as u32,
            ctime_nsec: meta.ctime_nsec() as u32,
            mtime: meta.mtime() as u32,
            mtime_nsec: meta.mtime_nsec() as u32,
            dev: meta.dev() as u32,
            ino: meta.ino() as u32,
            uid: meta.uid(),
            gid: meta.gid(),
            size: meta.size() as u32,
        }
    }
}

/// インデックスの1つのエントリー．パスは`Index`のキーとして持つ．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexEntry {
    pub stat: Stat,
    pub mode: Mode,
    pub id: ObjectId,
}

/// インデックス(`.git/index`の版2)．エントリーをパスの順に持つ．
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Index {
    entries: BTreeMap<String, IndexEntry>,
}

impl Index {
    pub fn entries(&self) -> &BTreeMap<String, IndexEntry> {
        &self.entries
    }

    pub fn insert(&mut self, path: String, entry: IndexEntry) {
        self.entries.insert(path, entry);
    }

    pub fn remove(&mut self, path: &str) {
        self.entries.remove(path);
    }

    /// インデックスのファイルを読む．ファイルがなければ空のインデックスを返す．
    pub fn load(path: &Path) -> Result<Index, Error> {
        match fs::read(path) {
            Ok(data) => Index::parse(&data),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Index::default()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), Error> {
        fs::write(path, self.to_bytes())?;
        Ok(())
    }

    /// インデックスのバイト列を解析する．
    pub fn parse(data: &[u8]) -> Result<Index, Error> {
        if data.len() < 32 {
            return Err(Error::CorruptIndex("too short"));
        }
        let (body, checksum) = data.split_at(data.len() - 20);
        if Sha1::digest(body).as_slice() != checksum {
            return Err(Error::CorruptIndex("checksum mismatch"));
        }
        let mut reader = Reader { rest: body };
        if reader.take(4)? != b"DIRC" {
            return Err(Error::CorruptIndex("bad signature"));
        }
        if reader.u32()? != 2 {
            return Err(Error::CorruptIndex("unsupported version"));
        }
        let count = reader.u32()?;
        let mut index = Index::default();
        for _ in 0..count {
            let entry_start = reader.rest.len();
            let ctime = reader.u32()?;
            let ctime_nsec = reader.u32()?;
            let mtime = reader.u32()?;
            let mtime_nsec = reader.u32()?;
            let dev = reader.u32()?;
            let ino = reader.u32()?;
            let mode = Mode::try_from(reader.u32()?)?;
            let uid = reader.u32()?;
            let gid = reader.u32()?;
            let size = reader.u32()?;
            let stat = Stat {
                ctime,
                ctime_nsec,
                mtime,
                mtime_nsec,
                dev,
                ino,
                uid,
                gid,
                size,
            };
            let id: [u8; 20] = reader.take(20)?.try_into().unwrap();
            let flags = reader.u16()?;
            let name_len = usize::from(flags & 0x0fff);
            let path = std::str::from_utf8(reader.take(name_len)?)
                .map_err(|_| Error::CorruptIndex("path is not UTF-8"))?;
            // エントリーの長さが8の倍数になるまで，1〜8個のNULで埋めてある．
            let read = entry_start - reader.rest.len();
            reader.take(8 - read % 8)?;
            index.insert(
                path.to_string(),
                IndexEntry {
                    stat,
                    mode,
                    id: ObjectId::from_bytes(id),
                },
            );
        }
        Ok(index)
    }

    /// インデックスのバイト列(末尾のチェックサムを含む)を作る．
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"DIRC");
        bytes.extend_from_slice(&2u32.to_be_bytes());
        bytes.extend_from_slice(&(self.entries.len() as u32).to_be_bytes());
        for (path, entry) in &self.entries {
            let entry_start = bytes.len();
            let stat = entry.stat;
            for value in [
                stat.ctime,
                stat.ctime_nsec,
                stat.mtime,
                stat.mtime_nsec,
                stat.dev,
                stat.ino,
                entry.mode.bits(),
                stat.uid,
                stat.gid,
                stat.size,
            ] {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            bytes.extend_from_slice(entry.id.as_bytes());
            let flags = path.len().min(0x0fff) as u16;
            bytes.extend_from_slice(&flags.to_be_bytes());
            bytes.extend_from_slice(path.as_bytes());
            let written = bytes.len() - entry_start;
            bytes.resize(bytes.len() + 8 - written % 8, 0);
        }
        let checksum = Sha1::digest(&bytes);
        bytes.extend_from_slice(&checksum);
        bytes
    }
}

/// バイト列を先頭から順に読む．
struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        if self.rest.len() < n {
            return Err(Error::CorruptIndex("unexpected end"));
        }
        let (taken, rest) = self.rest.split_at(n);
        self.rest = rest;
        Ok(taken)
    }

    fn u32(&mut self) -> Result<u32, Error> {
        let bytes: [u8; 4] = self.take(4)?.try_into().unwrap();
        Ok(u32::from_be_bytes(bytes))
    }

    fn u16(&mut self) -> Result<u16, Error> {
        let bytes: [u8; 2] = self.take(2)?.try_into().unwrap();
        Ok(u16::from_be_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::hash_blob;

    fn entry(mode: Mode, data: &[u8]) -> IndexEntry {
        IndexEntry {
            stat: Stat {
                mtime: 1767225600,
                size: data.len() as u32,
                ..Stat::default()
            },
            mode,
            id: hash_blob(data),
        }
    }

    #[test]
    fn empty_index_has_header_and_checksum() {
        let bytes = Index::default().to_bytes();
        assert_eq!(&bytes[..12], b"DIRC\0\0\0\x02\0\0\0\0");
        assert_eq!(bytes.len(), 12 + 20);
        assert_eq!(&bytes[12..], Sha1::digest(&bytes[..12]).as_slice());
    }

    #[test]
    fn entry_is_padded_to_multiple_of_eight() {
        let mut index = Index::default();
        index.insert("hello.txt".to_string(), entry(Mode::File, b"hello\n"));
        let bytes = index.to_bytes();
        // 62バイトの固定部と9バイトのパスの後ろに，1個のNULを足して72バイトにする．
        assert_eq!(bytes.len(), 12 + 72 + 20);
        assert_eq!(&bytes[12 + 62..12 + 71], b"hello.txt");
        assert_eq!(bytes[12 + 71], 0);
    }

    #[test]
    fn entries_are_written_in_path_order_and_read_back() {
        let mut index = Index::default();
        index.insert(
            "src/main.rs".to_string(),
            entry(Mode::File, b"fn main() {}\n"),
        );
        index.insert("run.sh".to_string(), entry(Mode::Executable, b"echo\n"));
        index.insert("a.txt".to_string(), entry(Mode::File, b""));
        let parsed = Index::parse(&index.to_bytes()).unwrap();
        let paths: Vec<&String> = parsed.entries().keys().collect();
        assert_eq!(paths, ["a.txt", "run.sh", "src/main.rs"]);
        assert_eq!(parsed, index);
    }

    #[test]
    fn broken_checksum_is_an_error() {
        let mut bytes = Index::default().to_bytes();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
        assert!(matches!(
            Index::parse(&bytes),
            Err(Error::CorruptIndex("checksum mismatch"))
        ));
    }

    #[test]
    fn missing_index_file_is_empty_index() {
        let dir = tempfile::TempDir::new().unwrap();
        let index = Index::load(&dir.path().join("index")).unwrap();
        assert!(index.entries().is_empty());
    }
}
