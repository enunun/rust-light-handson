use crate::commit::read_commit;
use crate::error::Error;
use crate::oid::ObjectId;
use crate::refs::RefName;
use crate::repo::Repository;

/// リビジョンの指定を，コミットなどのIDにする．
/// `HEAD`，ブランチ名，`refs/`で始まる参照名，4桁以上のIDと，その後ろの`~N`(最初の親をN回たどる)を受け付ける．
pub fn resolve(repo: &Repository, rev: &str) -> Result<ObjectId, Error> {
    let not_found = || Error::ObjectNotFound(rev.to_string());
    let Some((base, count)) = rev.split_once('~') else {
        return resolve_name(repo, rev);
    };
    let count: usize = if count.is_empty() {
        1
    } else {
        count.parse().map_err(|_| not_found())?
    };
    let mut id = resolve_name(repo, base)?;
    for _ in 0..count {
        id = *read_commit(repo.objects(), id)?
            .parents
            .first()
            .ok_or_else(not_found)?;
    }
    Ok(id)
}

/// `~N`のない指定を，IDにする．
fn resolve_name(repo: &Repository, rev: &str) -> Result<ObjectId, Error> {
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
    use crate::object::ObjectKind;
    use crate::store::ObjectStore;

    #[test]
    fn resolves_head_branch_and_full_ref_name() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let id = repo.objects().write(ObjectKind::Blob, b"hello\n").unwrap();
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
        let id = repo.objects().write(ObjectKind::Blob, b"hello\n").unwrap();
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

    /// `hello.txt`の中身を変えながら，3つのコミットを作る．
    fn three_commits(repo: &Repository, dir: &std::path::Path) -> Vec<ObjectId> {
        use crate::commit::{Commit, Signature};
        use crate::index::Index;
        let alice = Signature {
            name: String::from("Alice"),
            email: String::from("alice@example.com"),
            time: 0,
            offset_minutes: 0,
        };
        let mut ids: Vec<ObjectId> = Vec::new();
        for text in ["one\n", "two\n", "three\n"] {
            std::fs::write(dir.join("hello.txt"), text).unwrap();
            repo.add(dir, &[std::path::PathBuf::from("hello.txt")], 1)
                .unwrap();
            let index = Index::load(&repo.index_path()).unwrap();
            let tree = crate::tree::write_tree(repo.objects(), &index).unwrap();
            let mut builder = Commit::builder()
                .tree(tree)
                .author(alice.clone())
                .committer(alice.clone())
                .message(text);
            if let Some(&parent) = ids.last() {
                builder = builder.parent(parent);
            }
            ids.push(repo.commit(builder.build()).unwrap());
        }
        ids
    }

    #[test]
    fn tilde_follows_first_parents() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let ids = three_commits(&repo, dir.path());
        assert_eq!(resolve(&repo, "HEAD~0").unwrap(), ids[2]);
        assert_eq!(resolve(&repo, "HEAD~").unwrap(), ids[1]);
        assert_eq!(resolve(&repo, "main~2").unwrap(), ids[0]);
    }

    #[test]
    fn tilde_beyond_root_is_not_found() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        three_commits(&repo, dir.path());
        assert!(matches!(
            resolve(&repo, "HEAD~3"),
            Err(Error::ObjectNotFound(rev)) if rev == "HEAD~3"
        ));
    }
}
