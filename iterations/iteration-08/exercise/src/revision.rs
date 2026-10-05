use crate::error::Error;
use crate::oid::ObjectId;
use crate::refs::RefName;
use crate::repo::Repository;

/// リビジョンの指定(`HEAD`，ブランチ名，`refs/`で始まる参照名，4桁以上のID)を，コミットなどのIDにする．
pub fn resolve(repo: &Repository, rev: &str) -> Result<ObjectId, Error> {
    for candidate in [rev.to_string(), format!("refs/heads/{rev}")] {
        if let Ok(name) = RefName::try_from(candidate.as_str())
            && let Some(id) = repo.resolve_ref(&name)?
        {
            return Ok(id);
        }
    }
    if rev == "HEAD" {
        return Err(Error::ObjectNotFound(rev.to_string()));
    }
    repo.resolve_prefix(rev)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn resolves_head_branch_and_full_ref_name() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let id = repo.write_blob(b"hello\n").unwrap();
        repo.update_ref(&RefName::branch("main").unwrap(), id)
            .unwrap();
        assert_eq!(resolve(&repo, "HEAD").unwrap(), id);
        assert_eq!(resolve(&repo, "main").unwrap(), id);
        assert_eq!(resolve(&repo, "refs/heads/main").unwrap(), id);
    }

    #[test]
    fn resolves_object_id_prefix() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let id = repo.write_blob(b"hello\n").unwrap();
        assert_eq!(resolve(&repo, "ce01362").unwrap(), id);
    }

    #[test]
    fn unborn_head_is_not_found() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        assert!(matches!(
            resolve(&repo, "HEAD"),
            Err(Error::ObjectNotFound(rev)) if rev == "HEAD"
        ));
    }
}
