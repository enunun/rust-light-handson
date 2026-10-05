use std::fmt;
use std::str::FromStr;

/// GitのオブジェクトID．中身はSHA-1の20バイトである．
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId([u8; 20]);

/// 16進数の文字列を`ObjectId`にできなかった理由．
#[derive(Debug, PartialEq, Eq)]
pub enum ParseObjectIdError {
    /// 文字列の長さが40でない．
    InvalidLength(usize),
    /// 16進数でない文字がある．`position`は先頭を0とする位置である．
    InvalidChar { position: usize, ch: char },
}

impl ObjectId {
    pub fn from_bytes(bytes: [u8; 20]) -> ObjectId {
        ObjectId(bytes)
    }

    /// 20バイトの値を返す．
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// 先頭の7桁を返す．
    pub fn short(&self) -> String {
        let mut hex = self.to_string();
        hex.truncate(7);
        hex
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut hex = String::new();
        for byte in &self.0 {
            hex.push_str(&format!("{byte:02x}"));
        }
        f.write_str(&hex)
    }
}

impl FromStr for ObjectId {
    type Err = ParseObjectIdError;

    fn from_str(s: &str) -> Result<ObjectId, ParseObjectIdError> {
        if s.len() != 40 {
            return Err(ParseObjectIdError::InvalidLength(s.len()));
        }
        let mut bytes = [0u8; 20];
        for (position, ch) in s.chars().enumerate() {
            let digit = match ch.to_digit(16) {
                Some(digit) => digit as u8,
                None => return Err(ParseObjectIdError::InvalidChar { position, ch }),
            };
            bytes[position / 2] = bytes[position / 2] * 16 + digit;
        }
        Ok(ObjectId(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO: &str = "ce013625030ba8dba906f756967f9e9ca394464a";

    #[test]
    fn displays_each_byte_as_two_lowercase_hex_digits() {
        let mut bytes = [0xab; 20];
        bytes[0] = 0x00;
        bytes[1] = 0x0f;
        let id = ObjectId::from_bytes(bytes);
        assert_eq!(id.to_string(), "000fabababababababababababababababababab");
    }

    #[test]
    fn short_is_first_seven_digits() {
        let id = ObjectId::from_bytes([0xab; 20]);
        assert_eq!(id.short(), "abababa");
    }

    #[test]
    fn parses_forty_hex_digits() {
        let id: ObjectId = HELLO.parse().unwrap();
        assert_eq!(id.to_string(), HELLO);
    }

    #[test]
    fn parses_uppercase_hex_digits() {
        let upper = HELLO.to_uppercase();
        assert_eq!(upper.parse::<ObjectId>(), HELLO.parse::<ObjectId>());
    }

    #[test]
    fn rejects_short_string_with_its_length() {
        assert_eq!(
            "ce01".parse::<ObjectId>(),
            Err(ParseObjectIdError::InvalidLength(4))
        );
    }

    #[test]
    fn rejects_long_string_with_its_length() {
        let long = format!("{HELLO}0");
        assert_eq!(
            long.parse::<ObjectId>(),
            Err(ParseObjectIdError::InvalidLength(41))
        );
    }

    #[test]
    fn rejects_non_hex_char_with_its_position() {
        let text = format!("{}g", &HELLO[..39]);
        assert_eq!(
            text.parse::<ObjectId>(),
            Err(ParseObjectIdError::InvalidChar {
                position: 39,
                ch: 'g'
            })
        );
    }
}
