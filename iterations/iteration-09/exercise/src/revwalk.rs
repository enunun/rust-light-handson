use std::collections::{BinaryHeap, HashSet};

use crate::commit::Commit;
use crate::error::Error;
use crate::oid::ObjectId;
use crate::repo::Repository;

/// コミットから親をたどり，コミッターの時刻の新しい順にコミットを返すイテレーター．
pub struct RevWalk<'r> {
    repo: &'r Repository,
    queue: BinaryHeap<(i64, ObjectId)>,
    seen: HashSet<ObjectId>,
}

impl<'r> RevWalk<'r> {
    pub fn new(repo: &'r Repository, start: ObjectId) -> RevWalk<'r> {
        let mut walk = RevWalk {
            repo,
            queue: BinaryHeap::new(),
            seen: HashSet::new(),
        };
        walk.seen.insert(start);
        // 最初のコミットの時刻は使わないので，どの時刻よりも先に取り出される値にする．
        walk.queue.push((i64::MAX, start));
        walk
    }

    /// まだ見ていない親を，時刻とともに待ち行列に入れる．
    fn push_parents(&mut self, commit: &Commit) -> Result<(), Error> {
        for &parent in &commit.parents {
            if self.seen.insert(parent) {
                let time = self.repo.read_commit(parent)?.committer.time;
                self.queue.push((time, parent));
            }
        }
        Ok(())
    }
}

impl Iterator for RevWalk<'_> {
    type Item = Result<(ObjectId, Commit), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let (_, id) = self.queue.pop()?;
        let commit = match self.repo.read_commit(id) {
            Ok(commit) => commit,
            Err(error) => return Some(Err(error)),
        };
        if let Err(error) = self.push_parents(&commit) {
            return Some(Err(error));
        }
        Some(Ok((id, commit)))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::commit::Signature;
    use crate::object::ObjectKind;

    /// 時刻`time`に，親`parents`を持つコミットを書く．
    fn commit(repo: &Repository, message: &str, time: i64, parents: &[ObjectId]) -> ObjectId {
        let alice = Signature {
            name: String::from("Alice"),
            email: String::from("alice@example.com"),
            time,
            offset_minutes: 0,
        };
        let mut builder = Commit::builder()
            .tree("4b825dc642cb6eb9a060e54bf8d69288fbee4904".parse().unwrap())
            .author(alice.clone())
            .committer(alice)
            .message(message);
        for &parent in parents {
            builder = builder.parent(parent);
        }
        let bytes = builder.build().to_bytes();
        repo.write_object(ObjectKind::Commit, &bytes).unwrap()
    }

    fn messages(walk: RevWalk) -> Vec<String> {
        walk.map(|item| item.unwrap().1.message).collect()
    }

    #[test]
    fn walks_linear_history_from_newest() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let first = commit(&repo, "first", 100, &[]);
        let second = commit(&repo, "second", 200, &[first]);
        let third = commit(&repo, "third", 300, &[second]);
        assert_eq!(
            messages(RevWalk::new(&repo, third)),
            ["third", "second", "first"]
        );
    }

    #[test]
    fn shared_ancestor_is_returned_once_in_time_order() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let base = commit(&repo, "base", 100, &[]);
        let left = commit(&repo, "left", 300, &[base]);
        let right = commit(&repo, "right", 200, &[base]);
        let merge = commit(&repo, "merge", 400, &[right, left]);
        assert_eq!(
            messages(RevWalk::new(&repo, merge)),
            ["merge", "left", "right", "base"]
        );
    }

    #[test]
    fn take_stops_walking_early() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let first = commit(&repo, "first", 100, &[]);
        let second = commit(&repo, "second", 200, &[first]);
        let walk = RevWalk::new(&repo, second).take(1);
        let messages: Vec<String> = walk.map(|item| item.unwrap().1.message).collect();
        assert_eq!(messages, ["second"]);
    }

    #[test]
    fn missing_parent_is_an_error() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let missing = crate::object::hash_blob(b"no such commit");
        let orphan = commit(&repo, "orphan", 100, &[missing]);
        let mut walk = RevWalk::new(&repo, orphan);
        assert!(matches!(walk.next(), Some(Err(Error::ObjectNotFound(_)))));
    }
}
