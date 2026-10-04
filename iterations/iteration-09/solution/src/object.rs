use std::fmt;
use std::str::FromStr;

use sha1::{Digest, Sha1};

use crate::error::Error;
use crate::oid::ObjectId;

/// オブジェクトの種類．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ObjectKind::Blob => "blob",
            ObjectKind::Tree => "tree",
            ObjectKind::Commit => "commit",
        };
        f.write_str(name)
    }
}

impl FromStr for ObjectKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<ObjectKind, Error> {
        match s {
            "blob" => Ok(ObjectKind::Blob),
            "tree" => Ok(ObjectKind::Tree),
            "commit" => Ok(ObjectKind::Commit),
            _ => Err(Error::CorruptObject("unknown object type")),
        }
    }
}

/// オブジェクトのバイト列(`<種類> <大きさ>\0<内容>`)を作る．
pub fn encode(kind: ObjectKind, data: &[u8]) -> Vec<u8> {
    let mut bytes = format!("{kind} {}\0", data.len()).into_bytes();
    bytes.extend_from_slice(data);
    bytes
}

/// オブジェクトのバイト列を，種類と内容に分ける．内容は`data`の一部を借りて返す．
pub fn parse_header(data: &[u8]) -> Result<(ObjectKind, &[u8]), Error> {
    let nul = data
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(Error::CorruptObject("missing header"))?;
    let (header, rest) = data.split_at(nul);
    let content = &rest[1..];
    let header = std::str::from_utf8(header).map_err(|_| Error::CorruptObject("invalid header"))?;
    let (kind, size) = header
        .split_once(' ')
        .ok_or(Error::CorruptObject("invalid header"))?;
    let kind: ObjectKind = kind.parse()?;
    let size: usize = size
        .parse()
        .map_err(|_| Error::CorruptObject("invalid size"))?;
    if size != content.len() {
        return Err(Error::CorruptObject("size mismatch"));
    }
    Ok((kind, content))
}

/// 種類と内容からオブジェクトのIDを計算する．
pub fn hash_object(kind: ObjectKind, data: &[u8]) -> ObjectId {
    let digest = Sha1::digest(encode(kind, data));
    ObjectId::from_bytes(digest.into())
}

/// データをblobオブジェクトにしたときのIDを返す．
pub fn hash_blob(data: &[u8]) -> ObjectId {
    hash_object(ObjectKind::Blob, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_shown_and_parsed_by_name() {
        assert_eq!(ObjectKind::Commit.to_string(), "commit");
        assert_eq!("tree".parse::<ObjectKind>().unwrap(), ObjectKind::Tree);
    }

    #[test]
    fn unknown_kind_is_corrupt() {
        let result = "tag".parse::<ObjectKind>();
        assert!(matches!(
            result,
            Err(Error::CorruptObject("unknown object type"))
        ));
    }

    #[test]
    fn encode_starts_with_kind_and_size() {
        assert_eq!(encode(ObjectKind::Blob, b"hello\n"), b"blob 6\0hello\n");
    }

    #[test]
    fn parse_header_splits_kind_and_content() {
        let (kind, content) = parse_header(b"blob 6\0hello\n").unwrap();
        assert_eq!(kind, ObjectKind::Blob);
        assert_eq!(content, b"hello\n");
    }

    #[test]
    fn header_without_nul_is_corrupt() {
        let result = parse_header(b"blob 6");
        assert!(matches!(
            result,
            Err(Error::CorruptObject("missing header"))
        ));
    }

    #[test]
    fn size_must_match_content() {
        let result = parse_header(b"blob 5\0hello\n");
        assert!(matches!(result, Err(Error::CorruptObject("size mismatch"))));
    }

    #[test]
    fn hashes_empty_blob() {
        assert_eq!(
            hash_blob(b"").to_string(),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
    }

    #[test]
    fn hashes_blob_with_header() {
        assert_eq!(
            hash_blob(b"hello\n").to_string(),
            "ce013625030ba8dba906f756967f9e9ca394464a"
        );
    }
}
