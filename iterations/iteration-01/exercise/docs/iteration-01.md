# Iteration 1：オブジェクトID

このIterationでは，オブジェクトIDを表す型`ObjectId`を作り，`hash_blob`が`ObjectId`を返すようにする．
文字列のまま持ち回っていた値を，不正な値を作れない型に置き換える．

## 1-1 準備

このディレクトリ(`iterations/iteration-01/exercise`)に移動し，Iteration 0から引き継いだテストがすべて通ることを確かめる．

```console
$ cargo test
(略)
test object::tests::each_byte_becomes_two_lowercase_hex_digits ... ok
test object::tests::empty_bytes_become_empty_hex ... ok
test object::tests::hashes_empty_blob ... ok
test object::tests::hashes_blob_with_header ... ok
(略)
test hash_matches_git_for_bytes_that_are_not_text ... ok
test hash_matches_git_for_text ... ok
(略)
```

## 1-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-01.md)：所有権とムーブ，借用，`Copy`と`Clone`，配列，ニュータイプ，メソッド，`Display`と`FromStr`，エラーの`enum`，RustOwl
- [Gitのノート](../../../../docs/git/iteration-01.md)：20バイトと40桁，短縮形

読み終えたら，`src/lib.rs`の末尾に`#[cfg(test)] mod practice`を作り，その中にテストを書いて次の課題を確かめる．

1. `String`の変数を別の変数に代入したあと，元の変数を使うとどんなコンパイルエラーになるか．`.clone()`で直す．
2. `#[derive(Debug, Clone, Copy, PartialEq)]`を付けた`struct Point(i32, i32)`を作る．代入したあとも元の変数が使えることを確かめる．`Copy`を外すとどうなるか．
3. `Point`に`Display`を実装し，`Point(1, 2).to_string()`が`"(1, 2)"`になることを確かめる．
4. `"(1, 2)"`の形の文字列から`Point`を作る`FromStr`を実装する．`(`で始まらなければ`Err(PointError::MissingParen)`を返す．`strip_prefix`と`strip_suffix`，`split_once`を調べて使う．
5. `src/lib.rs`に，次の関数`moves`を書く．VS CodeでRustOwlを使って，`hex`が生きている範囲と，ムーブされる位置を見る．`hex.clone()`を`hex`に変え，戻り値を`moved.len()`だけにしたら，範囲はどう変わるか．端末では`mise run rustowl -- moves hex`で見られる．

```rust
pub fn moves() -> usize {
    let hex = String::from("ce01362");
    let moved = hex.clone();
    hex.len() + moved.len()
}
```

確かめ終えたら，`mod practice`と`moves`を消す．

## 1-3 テストリスト

作るものの要件と使用例を読み，確かめるべき振る舞いを`TESTLIST.md`に書き出す．

### 要件

- オブジェクトIDを表す型`ObjectId`を作る．中身は20バイトのSHA-1である．
- `ObjectId`は40桁の小文字の16進数として表示する．
- 40桁の16進数の文字列から`ObjectId`を作れる．大文字も受け付ける．
  - 長さが40でなければ，その長さを含むエラーにする．
  - 16進数でない文字があれば，その位置(先頭を0とする)と文字を含むエラーにする．
- `ObjectId::short`は，先頭の7桁を返す．
- `hash_blob`は`ObjectId`を返す．

### 使用例

```rust
use rgit::{ObjectId, ParseObjectIdError};

let id = rgit::hash_blob(b"hello\n");
assert_eq!(id.to_string(), "ce013625030ba8dba906f756967f9e9ca394464a");
assert_eq!(id.short(), "ce01362");
assert_eq!("CE013625030BA8DBA906F756967F9E9CA394464A".parse::<ObjectId>(), Ok(id));
assert_eq!("ce01".parse::<ObjectId>(), Err(ParseObjectIdError::InvalidLength(4)));
```

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `oid` | `pub struct ObjectId([u8; 20])`，`ObjectId::from_bytes`，`ObjectId::short`，`impl Display`，`impl FromStr`，`pub enum ParseObjectIdError`(`InvalidLength`と`InvalidChar`) |
| `object` | `hash_blob`の戻り値を`ObjectId`にする．`to_hex`の役目は`ObjectId`の`Display`に移す |
| ルート(`lib.rs`) | `ObjectId`と`ParseObjectIdError`を公開する |

### 書くときに考えること

- `hash_blob`の戻り値の型が変わると，既存のテストはどうなるか．変わるテストを「〜に変える」の形で書く．
- `to_hex`のテストは，どこへ移すか．
- 長さのエラーは，短い場合と長い場合の両方を確かめるか．16進数でない文字のエラーは，どの位置の文字で確かめると誤りを見つけやすいか．

## 1-4 図

`design/types.md`を，このIterationの終わりの状態に更新する．

- 新しいモジュール`oid`の名前空間を加える．その中に置く型は何か．
- `ObjectId`のフィールド，メソッド，実装する標準のトレイトを書く．列挙子のデータは`InvalidLength: usize`のように書く．
- `object`の公開関数の戻り値の型が変わる．`object`から`oid`のどの型への依存を描くか．

## 1-5 テスト駆動の実装

### モジュールを用意する

`src/lib.rs`に`mod oid;`を加え，`src/oid.rs`を作る．

### 実装のヒント

- 最初は`ObjectId`と`from_bytes`だけを作り，`Display`を実装する．`to_hex`の中身がそのまま使える．
- `Display`を実装すると，`to_string()`が使えるようになる．`short`は`to_string()`の結果を短くすればよい．
- `FromStr`では，先に長さを調べてから，`chars().enumerate()`で1文字ずつ`to_digit(16)`で数にする．2文字で1バイトになるので，位置を2で割った添字のバイトに`16倍して足す`形で積み上げる．
- `assert_eq!`で`Result<ObjectId, ParseObjectIdError>`を比べるには，`ObjectId`と`ParseObjectIdError`の両方に`Debug`と`PartialEq`が必要である．
- `hash_blob`の戻り値を変えたら，`cargo test`で失敗するテストを順に直す．`main.rs`は`{}`で表示しているので，`Display`があれば変えずに済む．
- `ObjectId`に`Copy`を付けるかどうかは，使用例の`Ok(id)`のあとで`id`を使えるかを考えて決める．

### ツールの操作

- `cargo test oid`のように，モジュール名で単体テストを絞り込める．
- 結合テストのファイルを加えたら，`cargo test --test object_id`のようにファイル名で絞り込める．

## 1-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．既存のテストの変更を，テストリストに書けていたか．
2. `ObjectId`のフィールドを非公開にした．`ObjectId([0; 20])`をモジュールの外で書けないことには，どんな利点と欠点があるか．
3. オブジェクトIDを`String`のまま持つ設計と比べる．`ObjectId`を引数に取る関数は，どんな検査を省けるか．
4. `ParseObjectIdError`を1つの文字列のエラーメッセージにしなかった．`enum`にしたことで，呼び出す側は何ができるか．
5. 図と実装を見比べ，違うところがあれば図を直す．

```console
node ../../../scripts/check-design.mjs .
```

## 1-7 発展課題

`ObjectId`の`Debug`の表示は，導出したままでは20個の数の並びになり，`assert_eq!`の失敗の表示が読みにくい．
`Debug`を導出せずに自分で実装し，`ObjectId(ce013625030ba8dba906f756967f9e9ca394464a)`と表示するようにする．

```rust
let id = rgit::hash_blob(b"hello\n");
assert_eq!(format!("{id:?}"), "ObjectId(ce013625030ba8dba906f756967f9e9ca394464a)");
```

これまでと同じく，テストリスト，図，実装の順に進める．
