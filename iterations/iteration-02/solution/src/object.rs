use sha1::{Digest, Sha1};

use crate::oid::ObjectId;

/// データをblobオブジェクトにしたバイト列(ヘッダーと内容)を返す．
pub fn blob_bytes(data: &[u8]) -> Vec<u8> {
    let mut bytes = format!("blob {}\0", data.len()).into_bytes();
    bytes.extend_from_slice(data);
    bytes
}

/// データをblobオブジェクトにしたときのIDを返す．
pub fn hash_blob(data: &[u8]) -> ObjectId {
    let digest = Sha1::digest(blob_bytes(data));
    ObjectId::from_bytes(digest.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_bytes_start_with_kind_and_size() {
        assert_eq!(blob_bytes(b"hello\n"), b"blob 6\0hello\n");
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
