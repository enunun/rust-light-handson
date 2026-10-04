# Iteration 8：`Iterator`の実装

Iteration 8では，コミットのグラフをたどる`RevWalk`を作り，`log`で表示する．
このノートでは，`Iterator`トレイトの実装と関連型，イテレーターの遅延評価，参照を持つ構造体，`BinaryHeap`と`HashSet`，`Ord`と`Hash`の導出，`let … else`を説明する．

## `Iterator`トレイト

`for`で回せる値や，`map`や`take`をつなげる値は，どれも`Iterator`トレイトを実装している．
`Iterator`を実装するには，関連型`Item`と，`next`メソッドを書く．

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // map，filter，take，collectなどは，nextを使って書かれていて，実装しなくても使える
}
```

`next`は，次の要素があれば`Some(要素)`を，なければ`None`を返す．
`type Item`は，そのイテレーターが返す要素の型を決める．Iteration 1の`FromStr`の`type Err`と同じく，関連型と呼ぶ．

数を1つずつ数えるイテレーターは，次のように書ける．

```rust
struct Countdown {
    remaining: u32,
}

impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        Some(self.remaining + 1)
    }
}

let numbers: Vec<u32> = Countdown { remaining: 3 }.collect();
assert_eq!(numbers, [3, 2, 1]);
```

`next`を書くだけで，`collect`，`map`，`take`など，`Iterator`のほかのメソッドがすべて使える．

### 失敗しうる要素

`RevWalk`は，コミットを読むたびに失敗しうる．要素の型を`Result`にして，失敗も要素として返す．

```rust
impl Iterator for RevWalk<'_> {
    type Item = Result<(ObjectId, Commit), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let (_, id) = self.queue.pop()?;
        let commit = match self.repo.read_commit(id) {
            Ok(commit) => commit,
            Err(error) => return Some(Err(error)),
        };
        // …
        Some(Ok((id, commit)))
    }
}
```

`next`は`Option`を返すので，中で使える`?`は`Option`の`?`である．`self.queue.pop()?`は，待ち行列が空なら`None`を返して終わる．
`Result`のエラーは`?`で返せないので，`match`で`Some(Err(error))`にして返す．
使う側は，`for item in walk { let (id, commit) = item?; … }`のように，要素ごとに`?`を使う．

## 遅延評価

イテレーターは，`next`が呼ばれるまで次の要素を計算しない．
`take(n)`は，n個を返したあとは元のイテレーターの`next`を呼ばない．

```rust
let walk = RevWalk::new(&repo, head).take(1);
```

`log -n 1`は，最新のコミットを1つ読むだけで終わり，残りの履歴は読まない．
すべてのコミットを`Vec`に集めてから先頭を取り出す実装と比べて，大きな履歴でも速い．

## 参照を持つ構造体

`RevWalk`は，コミットを読むために`Repository`を借りる．

```rust
pub struct RevWalk<'r> {
    repo: &'r Repository,
    queue: BinaryHeap<(i64, ObjectId)>,
    seen: HashSet<ObjectId>,
}

impl<'r> RevWalk<'r> {
    pub fn new(repo: &'r Repository, start: ObjectId) -> RevWalk<'r> {
        // …
    }
}
```

`RevWalk<'r>`は，`'r`の間だけ有効な`Repository`への参照を持つ．`RevWalk`は，`Repository`より長く使えない．
`Repository`を所有せずに借りるので，1つの`Repository`から何本でも`RevWalk`を作れる．
`impl Iterator for RevWalk<'_>`の`'_`は，`next`の中で`'r`を名前で使わないことを表す．

## `BinaryHeap`

`std::collections::BinaryHeap`は，最大の要素を素早く取り出せる集まり(ヒープ)である．

| メソッド | 動作 |
| --- | --- |
| `push(x)` | 要素を加える |
| `pop()` | 最大の要素を取り出して`Option`で返す．空なら`None` |
| `peek()` | 最大の要素を取り出さずに参照する |

```rust
use std::collections::BinaryHeap;

let mut heap = BinaryHeap::new();
heap.push((200, "second"));
heap.push((300, "third"));
heap.push((100, "first"));
assert_eq!(heap.pop(), Some((300, "third")));
assert_eq!(heap.pop(), Some((200, "second")));
```

タプルは，先頭の要素から順に比べる．`(時刻, ID)`を入れると，時刻の新しいものから取り出せる．時刻が同じなら，IDの大きいものが先になる．
小さいものから取り出したいときは，`std::cmp::Reverse(x)`で包んで大小を逆にする．

## `HashSet`

`HashSet<T>`は，同じ値を1つだけ持つ集まりである．
`insert(x)`は，`x`がまだなければ加えて`true`を，すでにあれば`false`を返す．

```rust
let mut seen = HashSet::new();
assert!(seen.insert("first"));
assert!(!seen.insert("first"));
```

`if self.seen.insert(parent) { … }`と書くと，「まだ見ていなければ，見たことにして処理する」を1回の操作で書ける．

## `Ord`と`Hash`の導出

`BinaryHeap`の要素には大小が，`HashSet`の要素にはハッシュ値が要る．
`ObjectId`に`PartialOrd`，`Ord`，`Hash`を導出する．

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId([u8; 20]);
```

- `PartialOrd`と`Ord`の導出は，フィールドの大小で比べる．`[u8; 20]`は先頭のバイトから比べるので，16進数の表記の辞書順と同じになる．
- `Hash`の導出は，フィールドのハッシュ値からハッシュ値を作る．`Hash`と`Eq`は，等しい値が同じハッシュ値になるように，一緒に導出する．

## `let … else`

`let パターン = 式 else { … };`は，式がパターンに一致すれば変数を作り，一致しなければ`else`の中を実行する．
`else`の中は，`return`などで必ず抜ける．

```rust
let Some((base, count)) = rev.split_once('~') else {
    return resolve_name(repo, rev);
};
```

一致しない場合を先に片付けると，残りのコードの字下げが深くならない．

## そのほか

- `slice.first()`は，先頭の要素への参照を`Option`で返す．
- `*`は，参照から値を取り出す．`ObjectId`は`Copy`なので，`*parents.first()?`で値を写せる．
- `usize::MAX`と`i64::MAX`は，それぞれの型の最大値である．
