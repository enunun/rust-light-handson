use sha1::{Digest, Sha1};

use crate::oid::ObjectId;

/// データをblobオブジェクトにしたときのIDを返す．
pub fn hash_blob(data: &[u8]) -> ObjectId {
    let header = format!("blob {}\0", data.len());
    let mut hasher = Sha1::new();
    hasher.update(header.as_bytes());
    hasher.update(data);
    ObjectId::from_bytes(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

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
