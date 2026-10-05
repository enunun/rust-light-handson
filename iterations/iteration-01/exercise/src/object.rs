use sha1::{Digest, Sha1};

/// データをblobオブジェクトにしたときのハッシュを，40桁の16進数で返す．
pub fn hash_blob(data: &[u8]) -> String {
    let header = format!("blob {}\0", data.len());
    let mut hasher = Sha1::new();
    hasher.update(header.as_bytes());
    hasher.update(data);
    to_hex(&hasher.finalize())
}

/// バイト列を，1バイトあたり2桁の小文字の16進数にする．
fn to_hex(bytes: &[u8]) -> String {
    let mut hex = String::new();
    for byte in bytes {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bytes_become_empty_hex() {
        assert_eq!(to_hex(&[]), "");
    }

    #[test]
    fn each_byte_becomes_two_lowercase_hex_digits() {
        assert_eq!(to_hex(&[0x00, 0x0f, 0xab, 0xff]), "000fabff");
    }

    #[test]
    fn hashes_empty_blob() {
        assert_eq!(hash_blob(b""), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    }

    #[test]
    fn hashes_blob_with_header() {
        assert_eq!(
            hash_blob(b"hello\n"),
            "ce013625030ba8dba906f756967f9e9ca394464a"
        );
    }
}
