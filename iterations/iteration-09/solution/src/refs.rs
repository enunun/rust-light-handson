use std::fmt;

use crate::error::Error;
use crate::oid::ObjectId;

/// 検査済みの参照名(`HEAD`か，`refs/`で始まる名前)．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefName(String);

impl RefName {
    /// `HEAD`の参照名．
    pub fn head() -> RefName {
        RefName(String::from("HEAD"))
    }

    /// ブランチ名から`refs/heads/<name>`の参照名を作る．
    pub fn branch(name: &str) -> Result<RefName, Error> {
        RefName::try_from(format!("refs/heads/{name}").as_str())
            .map_err(|_| Error::InvalidBranchName(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// ブランチなら，`refs/heads/`を除いたブランチ名を返す．
    pub fn branch_name(&self) -> Option<&str> {
        self.0.strip_prefix("refs/heads/")
    }
}

impl TryFrom<&str> for RefName {
    type Error = Error;

    fn try_from(name: &str) -> Result<RefName, Error> {
        if name == "HEAD" || is_valid_ref_path(name) {
            Ok(RefName(name.to_string()))
        } else {
            Err(Error::InvalidRefName(name.to_string()))
        }
    }
}

/// `refs/`で始まり，Gitの参照名の規則の一部を満たすか．
fn is_valid_ref_path(name: &str) -> bool {
    let forbidden = |ch: char| ch.is_ascii_control() || " ~^:?*[\\".contains(ch);
    name.starts_with("refs/")
        && !name.contains("..")
        && !name.contains(forbidden)
        && name
            .split('/')
            .all(|part| !part.is_empty() && !part.starts_with('.') && !part.ends_with(".lock"))
}

impl fmt::Display for RefName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// 参照の中身．IDを直接指すか，別の参照を指す(シンボリック参照)．
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ref {
    Direct(ObjectId),
    Symbolic(RefName),
}

impl Ref {
    /// 参照のファイルの中身を読む．
    pub fn parse(text: &str) -> Result<Ref, Error> {
        let text = text.trim_end();
        match text.strip_prefix("ref: ") {
            Some(target) => Ok(Ref::Symbolic(RefName::try_from(target)?)),
            None => text
                .parse()
                .map(Ref::Direct)
                .map_err(|_| Error::InvalidRefName(text.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::hash_blob;

    #[test]
    fn head_and_names_under_refs_are_valid() {
        assert_eq!(RefName::try_from("HEAD").unwrap().as_str(), "HEAD");
        assert!(RefName::try_from("refs/heads/main").is_ok());
        assert!(RefName::try_from("refs/heads/feature/login").is_ok());
    }

    #[test]
    fn invalid_names_are_rejected() {
        for name in [
            "main",
            "refs/heads/",
            "refs//main",
            "refs/heads/.hidden",
            "refs/heads/a..b",
            "refs/heads/bad name",
            "refs/heads/a~1",
            "refs/heads/main.lock",
        ] {
            assert!(RefName::try_from(name).is_err(), "{name} should be invalid");
        }
    }

    #[test]
    fn branch_name_is_under_refs_heads() {
        let name = RefName::branch("topic").unwrap();
        assert_eq!(name.as_str(), "refs/heads/topic");
        assert_eq!(name.branch_name(), Some("topic"));
        assert!(matches!(
            RefName::branch("bad name"),
            Err(Error::InvalidBranchName(name)) if name == "bad name"
        ));
    }

    #[test]
    fn parses_direct_and_symbolic_refs() {
        let id = hash_blob(b"hello\n");
        assert_eq!(Ref::parse(&format!("{id}\n")).unwrap(), Ref::Direct(id));
        assert_eq!(
            Ref::parse("ref: refs/heads/main\n").unwrap(),
            Ref::Symbolic(RefName::branch("main").unwrap())
        );
    }
}
