use std::fmt;

use crate::error::Error;
use crate::object::ObjectKind;
use crate::oid::ObjectId;
use crate::store::ObjectStore;

/// 作者やコミッターの署名．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub name: String,
    pub email: String,
    /// UNIX時刻(秒)
    pub time: i64,
    /// UTCからのずれ(分)．`+0900`なら540である．
    pub offset_minutes: i32,
}

impl Signature {
    /// `<名前> <<メール>> <UNIX時刻> <±hhmm>`の形の文字列を読む．
    pub fn parse(text: &str) -> Result<Signature, Error> {
        let invalid = || Error::CorruptObject("invalid signature");
        let (name, rest) = text.split_once(" <").ok_or_else(invalid)?;
        let (email, rest) = rest.split_once("> ").ok_or_else(invalid)?;
        let (time, offset) = rest.split_once(' ').ok_or_else(invalid)?;
        Ok(Signature {
            name: name.to_string(),
            email: email.to_string(),
            time: time.parse().map_err(|_| invalid())?,
            offset_minutes: parse_offset(offset).ok_or_else(invalid)?,
        })
    }
}

/// `+0900`や`-0130`を，分の数にする．
pub fn parse_offset(text: &str) -> Option<i32> {
    let (sign, digits) = match text.split_at_checked(1)? {
        ("+", digits) => (1, digits),
        ("-", digits) => (-1, digits),
        _ => return None,
    };
    if digits.len() != 4 {
        return None;
    }
    let hours: i32 = digits[..2].parse().ok()?;
    let minutes: i32 = digits[2..].parse().ok()?;
    Some(sign * (hours * 60 + minutes))
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.offset_minutes < 0 { '-' } else { '+' };
        let offset = self.offset_minutes.abs();
        write!(
            f,
            "{} <{}> {} {sign}{:02}{:02}",
            self.name,
            self.email,
            self.time,
            offset / 60,
            offset % 60
        )
    }
}

/// commitオブジェクト．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub tree: ObjectId,
    pub parents: Vec<ObjectId>,
    pub author: Signature,
    pub committer: Signature,
    pub message: String,
}

impl Commit {
    /// 組み立てを始める．`tree`，`author`，`committer`を呼ぶまで`build`できない．
    pub fn builder() -> CommitBuilder<Missing, Missing, Missing> {
        CommitBuilder {
            tree: Missing,
            parents: Vec::new(),
            author: Missing,
            committer: Missing,
            message: String::new(),
        }
    }

    /// commitオブジェクトの内容を読む．
    pub fn parse(data: &[u8]) -> Result<Commit, Error> {
        let invalid = || Error::CorruptObject("invalid commit");
        let text = std::str::from_utf8(data).map_err(|_| invalid())?;
        let (headers, message) = text.split_once("\n\n").ok_or_else(invalid)?;
        let mut tree = None;
        let mut parents = Vec::new();
        let mut author = None;
        let mut committer = None;
        for line in headers.lines() {
            let (key, value) = line.split_once(' ').ok_or_else(invalid)?;
            match key {
                "tree" => tree = Some(value.parse().map_err(|_| invalid())?),
                "parent" => parents.push(value.parse().map_err(|_| invalid())?),
                "author" => author = Some(Signature::parse(value)?),
                "committer" => committer = Some(Signature::parse(value)?),
                _ => return Err(Error::CorruptObject("unsupported commit header")),
            }
        }
        Ok(Commit {
            tree: tree.ok_or_else(invalid)?,
            parents,
            author: author.ok_or_else(invalid)?,
            committer: committer.ok_or_else(invalid)?,
            message: message.to_string(),
        })
    }

    /// commitオブジェクトの内容のバイト列を作る．
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut text = format!("tree {}\n", self.tree);
        for parent in &self.parents {
            text.push_str(&format!("parent {parent}\n"));
        }
        text.push_str(&format!("author {}\n", self.author));
        text.push_str(&format!("committer {}\n", self.committer));
        text.push('\n');
        text.push_str(&self.message);
        text.into_bytes()
    }
}

/// オブジェクトストアからcommitオブジェクトを読む．
pub fn read_commit<S: ObjectStore + ?Sized>(store: &S, id: ObjectId) -> Result<Commit, Error> {
    match store.read(id)? {
        (ObjectKind::Commit, content) => Commit::parse(&content),
        _ => Err(Error::NotACommit(id.to_string())),
    }
}

/// まだ指定していないことを表す型．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Missing;

/// `Commit`を組み立てる．型引数`T`，`A`，`C`は，tree，作者，コミッターを指定したかどうかを表す．
#[derive(Debug)]
pub struct CommitBuilder<T, A, C> {
    tree: T,
    parents: Vec<ObjectId>,
    author: A,
    committer: C,
    message: String,
}

impl<T, A, C> CommitBuilder<T, A, C> {
    pub fn tree(self, tree: ObjectId) -> CommitBuilder<ObjectId, A, C> {
        CommitBuilder {
            tree,
            parents: self.parents,
            author: self.author,
            committer: self.committer,
            message: self.message,
        }
    }

    pub fn author(self, author: Signature) -> CommitBuilder<T, Signature, C> {
        CommitBuilder {
            tree: self.tree,
            parents: self.parents,
            author,
            committer: self.committer,
            message: self.message,
        }
    }

    pub fn committer(self, committer: Signature) -> CommitBuilder<T, A, Signature> {
        CommitBuilder {
            tree: self.tree,
            parents: self.parents,
            author: self.author,
            committer,
            message: self.message,
        }
    }

    pub fn parent(mut self, parent: ObjectId) -> CommitBuilder<T, A, C> {
        self.parents.push(parent);
        self
    }

    pub fn message(mut self, message: &str) -> CommitBuilder<T, A, C> {
        self.message = message.to_string();
        self
    }
}

impl CommitBuilder<ObjectId, Signature, Signature> {
    pub fn build(self) -> Commit {
        Commit {
            tree: self.tree,
            parents: self.parents,
            author: self.author,
            committer: self.committer,
            message: self.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::hash_blob;

    fn alice() -> Signature {
        Signature {
            name: String::from("Alice"),
            email: String::from("alice@example.com"),
            time: 1767225600,
            offset_minutes: 540,
        }
    }

    #[test]
    fn signature_is_shown_with_offset() {
        assert_eq!(
            alice().to_string(),
            "Alice <alice@example.com> 1767225600 +0900"
        );
        let west = Signature {
            offset_minutes: -90,
            ..alice()
        };
        assert!(west.to_string().ends_with(" -0130"));
    }

    #[test]
    fn signature_is_parsed_back() {
        let text = "Alice Smith <alice@example.com> 1767225600 -0130";
        let signature = Signature::parse(text).unwrap();
        assert_eq!(signature.name, "Alice Smith");
        assert_eq!(signature.offset_minutes, -90);
        assert_eq!(signature.to_string(), text);
    }

    #[test]
    fn offset_needs_sign_and_four_digits() {
        assert_eq!(parse_offset("+0900"), Some(540));
        assert_eq!(parse_offset("0900"), None);
        assert_eq!(parse_offset("+900"), None);
    }

    #[test]
    fn builder_makes_commit_with_parents_in_order() {
        let tree = hash_blob(b"tree");
        let first = hash_blob(b"first");
        let second = hash_blob(b"second");
        let commit = Commit::builder()
            .message("merge\n")
            .parent(first)
            .parent(second)
            .tree(tree)
            .committer(alice())
            .author(alice())
            .build();
        assert_eq!(commit.tree, tree);
        assert_eq!(commit.parents, [first, second]);
        assert_eq!(commit.message, "merge\n");
    }

    #[test]
    fn commit_bytes_have_headers_blank_line_and_message() {
        let commit = Commit::builder()
            .tree("aae2b3618f4a481bc1bde056dae4b7617edb7e83".parse().unwrap())
            .author(alice())
            .committer(alice())
            .message("first\n")
            .build();
        assert_eq!(
            String::from_utf8(commit.to_bytes()).unwrap(),
            "tree aae2b3618f4a481bc1bde056dae4b7617edb7e83\n\
             author Alice <alice@example.com> 1767225600 +0900\n\
             committer Alice <alice@example.com> 1767225600 +0900\n\
             \n\
             first\n"
        );
    }

    #[test]
    fn commit_is_parsed_back() {
        let commit = Commit::builder()
            .tree(hash_blob(b"tree"))
            .parent(hash_blob(b"parent"))
            .author(alice())
            .committer(alice())
            .message("second\n\nbody\n")
            .build();
        assert_eq!(Commit::parse(&commit.to_bytes()).unwrap(), commit);
    }

    #[test]
    fn commit_without_tree_is_corrupt() {
        let data =
            b"author Alice <a@example.com> 0 +0000\ncommitter Alice <a@example.com> 0 +0000\n\nx\n";
        assert!(Commit::parse(data).is_err());
    }
}
