# Iteration 6：並び順と型状態パターン

Iteration 6では，インデックスからtreeを書き，commitオブジェクトを組み立てる．
このノートでは，`Ordering`と並べ替え，イテレーターの比較，ジェネリクス，型で状態を表す型状態パターン，環境変数と時刻を説明する．

## `Ordering`と並べ替え

2つの値の大小は，`std::cmp::Ordering`の`Less`，`Equal`，`Greater`で表す．
`a.cmp(&b)`は，`Ord`を実装した型の2つの値を比べて`Ordering`を返す．

`slice.sort_by(比べる関数)`は，その関数の結果に従って並べ替える．
`then`は，前の比較が`Equal`のときだけ，次の比較を使う．

```rust
let mut names = vec!["bb", "a", "ab", "c"];
names.sort_by(|a, b| a.len().cmp(&b.len()).then(a.cmp(b)));
assert_eq!(names, ["a", "c", "ab", "bb"]);
```

`sort_by`は，名前の付いた関数も受け取れる(`entries.sort_by(compare_entries)`)．

## イテレーターの比較

`Iterator::cmp`は，2つのイテレーターの要素を先頭から1つずつ比べ，最初に違った要素で大小を決める．
一方が先に終われば，短い方が小さい．文字列の辞書順の比較と同じである．

`chain`は，2つのイテレーターをつなぐ．`Option`もイテレーターとして`chain`に渡せる(`Some(x)`は1要素，`None`は0要素)．

```rust
let dir = "a".bytes().chain(Some(b'/'));
let file = "a.txt".bytes().chain(None);
assert_eq!(dir.cmp(file), Ordering::Greater);
```

`bool`の`then_some(x)`は，`true`なら`Some(x)`を，`false`なら`None`を返す．
Gitのtreeの並び順は，次のように書ける．文字列を作らずに，「名前の後ろに`/`がある」ものとして比べられる．

```rust
pub fn compare_entries(a: &TreeEntry, b: &TreeEntry) -> Ordering {
    let slash = |entry: &TreeEntry| (entry.mode == Mode::Directory).then_some(b'/');
    let a_key = a.name.bytes().chain(slash(a));
    let b_key = b.name.bytes().chain(slash(b));
    a_key.cmp(b_key)
}
```

## `while let`

`while let パターン = 式 { … }`は，式がパターンに一致する間くり返す．
スライスの先頭を見ながら読み進めるのに使える．

```rust
let mut rest = entries;
while let Some(&(path, entry)) = rest.first() {
    // …
    rest = &rest[1..];
}
```

`first()`は，先頭の要素への参照を`Option`で返す．空なら`None`で，ループが終わる．
`&(path, entry)`は，参照を外して組の中身を取り出すパターンである．

## 値を受け取る関数

`tree_bytes`は，エントリーの`Vec`を参照ではなく値で受け取り，並べ替えてから内容にする．

```rust
pub fn tree_bytes(mut entries: Vec<TreeEntry<'_>>) -> Vec<u8> {
    entries.sort_by(compare_entries);
    // …
}
```

呼び出す側は，`Vec`の所有権を渡す．`tree_bytes`は受け取った`Vec`を自由に並べ替えられ，呼び出す側の値を書き換えたかどうかを気にしなくてよい．
引数の`mut`は，受け取った値を関数の中で変えることを表す．

## ジェネリクス

型引数を持つ構造体は，`<T>`で型引数を宣言し，フィールドの型に使う．

```rust
struct Pair<T> {
    first: T,
    second: T,
}
```

`Pair<i32>`と`Pair<String>`は別の型である．`impl<T> Pair<T> { … }`は，どの`T`の`Pair`にも使えるメソッドを書く．
`impl Pair<i32> { … }`のように型引数を決めて書くと，その型の`Pair`にだけメソッドを持たせられる．

## 型状態パターン

コミットを作るには，tree，作者，コミッターが必要である．
組み立てる途中の値(ビルダー)の型に「どれを指定したか」を持たせると，必要なものがそろう前に`build`を呼ぶコードを，コンパイルエラーにできる．
型で状態を表すこの書き方を，型状態パターンと呼ぶ．

```rust
pub struct Missing;

pub struct CommitBuilder<T, A, C> {
    tree: T,
    parents: Vec<ObjectId>,
    author: A,
    committer: C,
    message: String,
}
```

- `Missing`は，まだ指定していないことを表す型である．フィールドを持たないので，値の大きさは0バイトである(ゼロサイズ型)．
- `Commit::builder()`は`CommitBuilder<Missing, Missing, Missing>`を返す．
- `tree(id)`は，`T`を`ObjectId`に変えた`CommitBuilder<ObjectId, A, C>`を返す．`author`，`committer`も同じである．
- `build`は，`impl CommitBuilder<ObjectId, Signature, Signature>`の中にだけ書く．

```rust
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
}

impl CommitBuilder<ObjectId, Signature, Signature> {
    pub fn build(self) -> Commit {
        // …
    }
}
```

`tree`は`self`を値で受け取り，フィールドを新しい型の`CommitBuilder`に移す．古いビルダーは使えなくなる．
`tree`を呼ばずに`build`を呼ぶと，`Missing`の状態の型には`build`がないので，コンパイルエラーになる．

```text
error[E0599]: no method named `build` found for struct `CommitBuilder<Missing, Signature, Signature>` in the current scope
   --> src/commit.rs:300:10
    |
134 |   pub struct CommitBuilder<T, A, C> {
    |   --------------------------------- method `build` not found for this struct
...
296 | /     Commit::builder()
297 | |         .author(alice.clone())
298 | |         .committer(alice)
299 | |         .message("first\n")
300 | |         .build()
    | |         -^^^^^ method not found in `CommitBuilder<Missing, Signature, Signature>`
    | |_________|
    |
    |
    = note: the method was found for `CommitBuilder<ObjectId, Signature, Signature>`
```

ほかの言語のビルダーでは，`build`の中で「treeが`null`なら例外」と実行時に調べることが多い．
型状態パターンでは，その検査をコンパイラーが行い，`build`の中では必ずtreeがある．

### `PhantomData`

状態を表す型を，値を持たずに型引数だけで持ちたいときは，`std::marker::PhantomData<S>`をフィールドにする．
`PhantomData`も大きさ0の型で，「`S`を使っている」ことだけをコンパイラーに伝える．

```rust
use std::marker::PhantomData;

struct Open;
struct Closed;

struct Door<S> {
    name: String,
    state: PhantomData<S>,
}

impl Door<Closed> {
    fn open(self) -> Door<Open> {
        Door { name: self.name, state: PhantomData }
    }
}

impl Door<Open> {
    fn walk_through(&self) -> String {
        format!("walked through {}", self.name)
    }
}
```

`CommitBuilder`は，状態を表す型引数の位置に実際の値(`ObjectId`や`Signature`)を持つので，`PhantomData`が要らない．

## 構造体の更新構文の使いどころ

型の違う構造体には，更新構文`..self`を使えない．`CommitBuilder<Missing, …>`から`CommitBuilder<ObjectId, …>`を作るときは，フィールドを1つずつ書く．

## 環境変数と`HashMap`

`std::env::vars()`は，環境変数の名前と値の組を返すイテレーターである．`collect()`で`HashMap<String, String>`にできる．
`rgit`の`cli::run`は，環境変数を`HashMap`の引数で受け取る．テストは，好きな環境変数を渡して実行できる．

```rust
let mut env = HashMap::new();
env.insert(String::from("GIT_AUTHOR_NAME"), String::from("Alice"));
assert_eq!(env.get("GIT_AUTHOR_NAME").cloned(), Some(String::from("Alice")));
assert_eq!(env.get("GIT_AUTHOR_EMAIL"), None);
```

`get`は`Option<&String>`を返す．`cloned()`は，`Option`の中の参照を複製して`Option<String>`にする．

## 文字列の分割

| メソッド | 動作 |
| --- | --- |
| `s.split_once(' ')` | 最初の区切りの前後を組で返す．区切りがなければ`None` |
| `s.strip_prefix('@')` | 先頭が一致すれば，残りを`Some`で返す |
| `s.split_at_checked(1)` | 位置1の前後を組で返す．文字の境界でなければ`None` |
| `s.lines()` | 1行ずつ返す．行末の改行は含まない |

`split_at`は，位置が文字列の長さを超えるとパニックになる．`split_at_checked`は`None`を返すので，外から来た文字列に使える．

## 現在の時刻

`SystemTime::now()`は現在の時刻を返す．`duration_since(UNIX_EPOCH)`で，1970年1月1日0時からの経過時間(`Duration`)を得る．

```rust
use std::time::{SystemTime, UNIX_EPOCH};

let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
let seconds = now.as_secs() as i64;
```

`i64`と`i32`は符号付きの整数である．タイムゾーンのずれは負になりうるので，`i32`で持つ．
`abs()`は絶対値，`/`は整数の割り算(切り捨て)，`%`は余りである．
