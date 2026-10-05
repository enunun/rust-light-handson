use std::fmt;

use crate::error::Error;
use crate::object::ObjectKind;
use crate::oid::ObjectId;

/// treeのエントリーのモード．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// 通常のファイル(`100644`)
    File,
    /// 実行可能なファイル(`100755`)
    Executable,
    /// シンボリックリンク(`120000`)
    Symlink,
    /// ディレクトリ(`40000`)
    Directory,
    /// サブモジュール(`160000`)
    Submodule,
}

impl Mode {
    /// このモードのエントリーが指すオブジェクトの種類．
    pub fn kind(self) -> ObjectKind {
        match self {
            Mode::Directory => ObjectKind::Tree,
            Mode::Submodule => ObjectKind::Commit,
            Mode::File | Mode::Executable | Mode::Symlink => ObjectKind::Blob,
        }
    }

    /// インデックスに書く数としてのモード(`0o100644`など)．
    pub fn bits(self) -> u32 {
        match self {
            Mode::File => 0o100644,
            Mode::Executable => 0o100755,
            Mode::Symlink => 0o120000,
            Mode::Directory => 0o40000,
            Mode::Submodule => 0o160000,
        }
    }
}

/// インデックスに書かれた数から作る．
impl TryFrom<u32> for Mode {
    type Error = Error;

    fn try_from(bits: u32) -> Result<Mode, Error> {
        match bits {
            0o100644 => Ok(Mode::File),
            0o100755 => Ok(Mode::Executable),
            0o120000 => Ok(Mode::Symlink),
            0o40000 => Ok(Mode::Directory),
            0o160000 => Ok(Mode::Submodule),
            _ => Err(Error::CorruptObject("unknown mode")),
        }
    }
}

/// `ls-tree`と同じく，6桁で表示する(ディレクトリは`040000`)．
impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = match self {
            Mode::File => "100644",
            Mode::Executable => "100755",
            Mode::Symlink => "120000",
            Mode::Directory => "040000",
            Mode::Submodule => "160000",
        };
        f.write_str(digits)
    }
}

/// treeオブジェクトの中のモードの表記(ディレクトリは`40000`)から作る．
impl TryFrom<&[u8]> for Mode {
    type Error = Error;

    fn try_from(digits: &[u8]) -> Result<Mode, Error> {
        match digits {
            b"100644" => Ok(Mode::File),
            b"100755" => Ok(Mode::Executable),
            b"120000" => Ok(Mode::Symlink),
            b"40000" => Ok(Mode::Directory),
            b"160000" => Ok(Mode::Submodule),
            _ => Err(Error::CorruptObject("unknown mode")),
        }
    }
}

/// treeの1つのエントリー．名前は，treeの内容のバイト列を借りている．
#[derive(Debug, PartialEq, Eq)]
pub struct TreeEntry<'a> {
    pub mode: Mode,
    pub name: &'a str,
    pub id: ObjectId,
}

/// `<モード> <種類> <ID>\t<名前>`の形で表示する．
impl fmt::Display for TreeEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}\t{}",
            self.mode,
            self.mode.kind(),
            self.id,
            self.name
        )
    }
}

/// treeオブジェクトの内容を，エントリーの列にする．
pub fn parse_tree(data: &[u8]) -> Result<Vec<TreeEntry<'_>>, Error> {
    let corrupt = || Error::CorruptObject("invalid tree entry");
    let mut entries = Vec::new();
    let mut rest = data;
    while !rest.is_empty() {
        let space = rest
            .iter()
            .position(|byte| *byte == b' ')
            .ok_or_else(corrupt)?;
        let mode = Mode::try_from(&rest[..space])?;
        rest = &rest[space + 1..];

        let nul = rest
            .iter()
            .position(|byte| *byte == 0)
            .ok_or_else(corrupt)?;
        let name = std::str::from_utf8(&rest[..nul]).map_err(|_| corrupt())?;
        rest = &rest[nul + 1..];

        if rest.len() < 20 {
            return Err(corrupt());
        }
        let (id, after) = rest.split_at(20);
        let id: [u8; 20] = id.try_into().unwrap();
        rest = after;

        entries.push(TreeEntry {
            mode,
            name,
            id: ObjectId::from_bytes(id),
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::hash_blob;

    /// treeの内容の1つのエントリーのバイト列を作る．
    fn entry_bytes(mode: &str, name: &str, id: ObjectId) -> Vec<u8> {
        let mut bytes = format!("{mode} {name}\0").into_bytes();
        let hex = id.to_string();
        for i in 0..20 {
            bytes.push(u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap());
        }
        bytes
    }

    #[test]
    fn modes_are_read_from_tree_digits() {
        assert_eq!(Mode::try_from(&b"100644"[..]).unwrap(), Mode::File);
        assert_eq!(Mode::try_from(&b"100755"[..]).unwrap(), Mode::Executable);
        assert_eq!(Mode::try_from(&b"120000"[..]).unwrap(), Mode::Symlink);
        assert_eq!(Mode::try_from(&b"40000"[..]).unwrap(), Mode::Directory);
        assert_eq!(Mode::try_from(&b"160000"[..]).unwrap(), Mode::Submodule);
    }

    #[test]
    fn unknown_mode_is_corrupt() {
        let result = Mode::try_from(&b"100600"[..]);
        assert!(matches!(result, Err(Error::CorruptObject("unknown mode"))));
    }

    #[test]
    fn modes_convert_to_and_from_index_bits() {
        assert_eq!(Mode::Executable.bits(), 0o100755);
        assert_eq!(Mode::try_from(0o100644).unwrap(), Mode::File);
        assert!(Mode::try_from(0o100600).is_err());
    }

    #[test]
    fn directory_mode_is_shown_with_six_digits() {
        assert_eq!(Mode::Directory.to_string(), "040000");
        assert_eq!(Mode::Directory.kind(), ObjectKind::Tree);
    }

    #[test]
    fn empty_tree_has_no_entries() {
        assert_eq!(parse_tree(b"").unwrap(), vec![]);
    }

    #[test]
    fn parses_entries_in_order() {
        let hello = hash_blob(b"hello\n");
        let mut data = entry_bytes("100644", "hello.txt", hello);
        data.extend(entry_bytes("40000", "src", hello));
        let entries = parse_tree(&data).unwrap();
        assert_eq!(
            entries,
            vec![
                TreeEntry {
                    mode: Mode::File,
                    name: "hello.txt",
                    id: hello
                },
                TreeEntry {
                    mode: Mode::Directory,
                    name: "src",
                    id: hello
                },
            ]
        );
    }

    #[test]
    fn truncated_id_is_corrupt() {
        let data = entry_bytes("100644", "hello.txt", hash_blob(b"hello\n"));
        let result = parse_tree(&data[..data.len() - 1]);
        assert!(matches!(
            result,
            Err(Error::CorruptObject("invalid tree entry"))
        ));
    }

    #[test]
    fn name_must_be_utf8() {
        let mut data = b"100644 \xff\0".to_vec();
        data.extend([0u8; 20]);
        assert!(parse_tree(&data).is_err());
    }

    #[test]
    fn entry_is_shown_like_ls_tree() {
        let entry = TreeEntry {
            mode: Mode::Directory,
            name: "src",
            id: hash_blob(b"hello\n"),
        };
        assert_eq!(
            entry.to_string(),
            "040000 tree ce013625030ba8dba906f756967f9e9ca394464a\tsrc"
        );
    }
}
