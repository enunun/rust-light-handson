use std::collections::{BinaryHeap, HashSet};

use crate::commit::{Commit, read_commit};
use crate::error::Error;
use crate::oid::ObjectId;
use crate::store::ObjectStore;

/// コミットから親をたどり，コミッターの時刻の新しい順にコミットを返すイテレーター．
pub struct RevWalk<'s, S: ObjectStore + ?Sized> {
    store: &'s S,
    queue: BinaryHeap<(i64, ObjectId)>,
    seen: HashSet<ObjectId>,
}

impl<'s, S: ObjectStore + ?Sized> RevWalk<'s, S> {
    pub fn new(store: &'s S, start: ObjectId) -> RevWalk<'s, S> {
        let mut walk = RevWalk {
            store,
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
                let time = read_commit(self.store, parent)?.committer.time;
                self.queue.push((time, parent));
            }
        }
        Ok(())
    }
}

impl<S: ObjectStore + ?Sized> Iterator for RevWalk<'_, S> {
    type Item = Result<(ObjectId, Commit), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let (_, id) = self.queue.pop()?;
        let commit = match read_commit(self.store, id) {
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
    use super::*;
    use crate::commit::Signature;
    use crate::object::ObjectKind;
    use crate::store::MemoryObjectStore;

    /// 時刻`time`に，親`parents`を持つコミットを書く．
    fn commit(
        store: &MemoryObjectStore,
        message: &str,
        time: i64,
        parents: &[ObjectId],
    ) -> ObjectId {
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
        store.write(ObjectKind::Commit, &bytes).unwrap()
    }

    fn messages(walk: RevWalk<MemoryObjectStore>) -> Vec<String> {
        walk.map(|item| item.unwrap().1.message).collect()
    }

    #[test]
    fn walks_linear_history_from_newest() {
        let store = MemoryObjectStore::default();
        let first = commit(&store, "first", 100, &[]);
        let second = commit(&store, "second", 200, &[first]);
        let third = commit(&store, "third", 300, &[second]);
        assert_eq!(
            messages(RevWalk::new(&store, third)),
            ["third", "second", "first"]
        );
    }

    #[test]
    fn shared_ancestor_is_returned_once_in_time_order() {
        let store = MemoryObjectStore::default();
        let base = commit(&store, "base", 100, &[]);
        let left = commit(&store, "left", 300, &[base]);
        let right = commit(&store, "right", 200, &[base]);
        let merge = commit(&store, "merge", 400, &[right, left]);
        assert_eq!(
            messages(RevWalk::new(&store, merge)),
            ["merge", "left", "right", "base"]
        );
    }

    #[test]
    fn take_stops_walking_early() {
        let store = MemoryObjectStore::default();
        let first = commit(&store, "first", 100, &[]);
        let second = commit(&store, "second", 200, &[first]);
        let walk = RevWalk::new(&store, second).take(1);
        let messages: Vec<String> = walk.map(|item| item.unwrap().1.message).collect();
        assert_eq!(messages, ["second"]);
    }

    #[test]
    fn missing_parent_is_an_error() {
        let store = MemoryObjectStore::default();
        let missing = crate::object::hash_blob(b"no such commit");
        let orphan = commit(&store, "orphan", 100, &[missing]);
        let mut walk = RevWalk::new(&store, orphan);
        assert!(matches!(walk.next(), Some(Err(Error::ObjectNotFound(_)))));
    }
}
