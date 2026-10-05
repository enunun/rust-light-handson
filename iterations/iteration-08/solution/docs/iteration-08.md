# Iteration 8：`log`(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 8-1 準備

`Cargo.toml`は，Iteration 7の模範解答からパッケージ名だけを変えたものである．依存は変わらない．

## 8-2 文法と概念

課題の解答例である．

```rust
#[cfg(test)]
mod practice {
    use std::collections::{BinaryHeap, HashSet};

    struct Countdown {
        remaining: u32,
        calls: u32,
    }

    impl Iterator for Countdown {
        type Item = u32;

        fn next(&mut self) -> Option<u32> {
            self.calls += 1;
            if self.remaining == 0 {
                return None;
            }
            self.remaining -= 1;
            Some(self.remaining + 1)
        }
    }

    fn countdown(from: u32) -> Countdown {
        Countdown {
            remaining: from,
            calls: 0,
        }
    }

    #[test]
    fn counts_down() {
        let numbers: Vec<u32> = countdown(3).collect();
        assert_eq!(numbers, [3, 2, 1]);
    }

    #[test]
    fn adapters_call_next_lazily() {
        let tens: Vec<u32> = countdown(3).map(|n| n * 10).filter(|n| *n != 20).collect();
        assert_eq!(tens, [30, 10]);
        let mut counter = countdown(100);
        let first_two: Vec<u32> = counter.by_ref().take(2).collect();
        assert_eq!(first_two, [100, 99]);
        assert_eq!(counter.calls, 2);
    }

    #[test]
    fn heap_returns_newest_first() {
        let mut heap = BinaryHeap::new();
        heap.push((200, "second"));
        heap.push((300, "third"));
        heap.push((100, "first"));
        assert_eq!(heap.pop(), Some((300, "third")));
        assert_eq!(heap.pop(), Some((200, "second")));
        assert_eq!(heap.pop(), Some((100, "first")));
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn set_insert_reports_new_values() {
        let mut seen = HashSet::new();
        assert!(seen.insert("first"));
        assert!(!seen.insert("first"));
    }

    #[test]
    fn let_else_splits_tilde() {
        let Some((base, count)) = "x~2".split_once('~') else {
            panic!("no tilde");
        };
        assert_eq!((base, count), ("x", "2"));
    }
}
```

- 2：`take`はイテレーターをムーブするので，そのままでは`take`のあとで`counter.calls`を読めない．`by_ref()`は，イテレーターを借りたまま`take`などをつなぐ．100から数えても，`next`は2回しか呼ばれない．
- 5：`panic!`は，その場でプログラムを止める．`let … else`の`else`の中は抜ける必要があり，`panic!`もその1つである．

## 8-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

- `revwalk`は，一直線，合流，途中で止める，壊れた履歴の順に並べた．合流のテストでは，左の枝(`left`)を右の枝(`right`)より新しくして，2番目の親が先に出ることを確かめる．
- `revision`の`~N`は，`HEAD~0`，`HEAD~`，`main~2`を1つのテストにまとめた．どれも`resolve`の同じ処理を通る．
- 結合テストのコミットは，`rgit_at`で1時間おきの時刻を付けて作った．同じ時刻のコミットばかりだと，順序の誤りが見つからない．

## 8-4 図

模範解答は[design/types.md](../design/types.md)にある．Iteration 7からの変更は次のとおりである．

- `revwalk`の名前空間に`RevWalk~'r~`を描き，`Repository`への参照を関連の矢印で描いた．
- `RevWalk`が`Commit`を返すので，`Commit`への依存を描いた．`queue`の正確な型は図の下に書いた．
- `Repository`に`read_commit`を，`ObjectId`に`Ord`と`Hash`の実装を，`Error`に`NotACommit`を加えた．

## 8-5 テスト駆動の実装

### 一直線の履歴

単体テストの中に，時刻と親を決めてコミットを書く補助関数を作った．

```rust
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
```

```rust
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
```

コミットを読むために，`Repository`に`read_commit`を加えた．種類がcommitでなければ`NotACommit`を返す．
`RevWalk`は，最初は待ち行列と`next`だけで作った．

```rust
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
```

`ObjectId`には，`PartialOrd`，`Ord`，`Hash`を導出した．`BinaryHeap`の`(i64, ObjectId)`には大小が，`HashSet`の要素にはハッシュ値が要る．

### 合流する履歴

```rust
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
```

見たかどうかを調べずに親を待ち行列に入れると，`base`が2回出てくる．

```text
thread 'revwalk::tests::shared_ancestor_is_returned_once_in_time_order' (29317) panicked at src/revwalk.rs:107:9:
assertion `left == right` failed
  left: ["merge", "left", "right", "base", "base"]
 right: ["merge", "left", "right", "base"]
```

親はまず`seen`に加え，新しく加わったときだけ待ち行列へ入れる．

```rust
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
```

親の時刻を知るために，親のコミットを待ち行列に入れる時点で読む．取り出したときにもう一度読むが，コミットの数だけの読み取りで済む．

### 途中で止める，壊れた履歴

`take(1)`の項目は，実装を変えずに通る．イテレーターは要素を1つずつ作るので，`take`が止めれば，それ以上のコミットは読まない．
親がない項目も，`read_commit`のエラーを`Some(Err(…))`で返しているので，そのまま通る．

### `~N`

```rust
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
        id = *repo.read_commit(id)?.parents.first().ok_or_else(not_found)?;
    }
    Ok(id)
}
```

Iteration 7の`resolve`は，名前を`resolve_name`に変えて，`~`のない指定の解決に使った．
単体テストでは，`hello.txt`を書き換えながら`Repository::commit`で3つのコミットを作った．

### `log`

```rust
Command::Log { max_count, rev } => {
    let repo = Repository::discover(cwd)?;
    let start = resolve(&repo, rev.as_deref().unwrap_or("HEAD"))?;
    let walk = RevWalk::new(&repo, start).take(max_count.unwrap_or(usize::MAX));
    for item in walk {
        let (id, commit) = item?;
        let summary = commit.message.lines().next().unwrap_or("");
        writeln!(out, "{} {summary}", id.short())?;
    }
}
```

結合テストのマージのコミットは，`commit-tree`に`-p`を2回渡して作り，`git log --oneline`の出力と比べた．

## 8-6 振り返り

1. 模範解答の合流する履歴では，枝の時刻の順序と親の順序が逆である．時刻の順に出すことと，親の順に出すことの違いが分かる．
2. `Vec`を返す関数では，`log -n 1`でも履歴全体を読む．使う側は`for`で回すだけでよいので，コードはほとんど変わらない．イテレーターなら，必要な分だけ読む．
3. 借りる設計では，`RevWalk`を`Repository`より長く持てない(関数から`RevWalk`だけを返せない)．その代わり，同じ`Repository`から何本でも作れ，`RevWalk`を使っている間もほかの読み取りに`Repository`を使える．
4. `None`で終わると，使う側は「履歴を最後まで読んだ」と区別できない．`Result`なら，壊れた履歴をエラーとして表示できる．
5. 図に描いた型と関係は，コードと一致している．

## 8-7 発展課題

`RevWalk`に`first_parent`のフィールドを加え，それを`true`にするメソッドを作る．`self`を受け取って返すので，`RevWalk::new(…).first_parent()`とつなげられる．

```rust
/// 最初の親だけをたどるようにする．
pub fn first_parent(mut self) -> RevWalk<'r> {
    self.first_parent = true;
    self
}
```

`push_parents`は，最初の親だけをたどるなら1つ，そうでなければすべての親を見る．

```rust
let count = if self.first_parent { 1 } else { commit.parents.len() };
for &parent in commit.parents.iter().take(count) {
    // …
}
```

`log`の結合テストで，マージのある履歴の`--first-parent`の出力を，`git log --oneline --first-parent`と比べる．
