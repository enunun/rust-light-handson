# Iteration 0：blobのハッシュの計算

このIterationでは，Cargoで`rgit`のパッケージを作り，バイト列からGitのblobのIDを計算する関数を作る．
テストリスト → 図の更新 → テスト駆動の実装 → 設計レビューという，全Iterationに共通する流れを初めて1周する．

## 0-1 準備

### パッケージを作る

ターミナルで，このディレクトリ(`iterations/iteration-00/exercise`)に移動する．

```console
cd iterations/iteration-00/exercise
```

`cargo init`で，このディレクトリをライブラリクレート`rgit`のパッケージにする．

```console
$ cargo init --lib --name rgit
    Creating library package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
```

`Cargo.toml`と`src/lib.rs`ができる．`src/lib.rs`には，サンプルの関数`add`とそのテストが入っている．

```rust
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
```

このディレクトリは，リポジトリのルートの`Cargo.toml`(模範解答のワークスペース)から除外してある．
そのため，このIterationの`cargo`のコマンドはすべて，このディレクトリの中で実行する．

### 依存を追加する

SHA-1の計算には，`sha1`クレートを使う．`cargo add`で依存に追加する．

```console
$ cargo add sha1
    Updating crates.io index
      Adding sha1 v0.11.0 to dependencies
             Features:
             + alloc
             + oid
             - zeroize
    Updating crates.io index
     Locking 10 packages to latest Rust 1.98.1 compatible versions
```

`Cargo.toml`の`[dependencies]`に`sha1 = "0.11.0"`が加わったことを確かめる．

### ビルドとテスト

`cargo build`でビルドし，`cargo test`でサンプルのテストを実行する．
初回のビルドでは，`sha1`と，`sha1`の使うクレートをダウンロードしてコンパイルする．

```console
$ cargo build
 Downloading crates ...
  Downloaded cpufeatures v0.3.1
(略)
   Compiling sha1 v0.11.0
   Compiling rgit v0.1.0 (/workspaces/iterations/iteration-00/exercise)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.25s
$ cargo test
   Compiling rgit v0.1.0 (/workspaces/iterations/iteration-00/exercise)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running unittests src/lib.rs (target/debug/deps/rgit-ba7693f30ae6f8c5)

running 1 test
test tests::it_works ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rgit

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 0-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-00.md)：Cargo，関数，整数とバイト列，文字列の組み立て，外部のクレート，テスト，モジュール，標準入力
- [Gitのノート](../../../../docs/git/iteration-00.md)：内容アドレス，blobオブジェクト，SHA-1

読み終えたら，`src/lib.rs`の`mod tests`の中にテストを書いて，次の課題を確かめる．
RustにはREPLがないので，このように小さなテストを書いて`cargo test`で動かす．

1. `b"abc"`の長さと先頭の要素，`"あ"`の`len()`と`as_bytes()`を，`assert_eq!`で確かめる．
2. `format!`で，`10u8`を2桁の16進数`"0a"`に，`255`を16進数`"ff"`にする．
3. バイト列の要素の合計を`u32`で返す関数`sum(bytes: &[u8]) -> u32`を書き，`sum(b"")`と`sum(&[1, 2, 255])`を確かめる．
4. `sha1`で`b"abc"`のSHA-1を計算し，結果が20バイトで，先頭のバイトが`0xa9`であることを確かめる．`sha1sum`の結果(`printf 'abc' | sha1sum`)とも比べる．

確かめ終えたら，`src/lib.rs`の中身をすべて消す．0-5で`rgit`のコードを書き始める．

## 0-3 テストリスト

作るものの要件と使用例を読み，確かめるべき振る舞いを`TESTLIST.md`に書き出す．
テストリストの書き方は[テスト駆動開発とテストリスト](../../../../docs/tdd.md)を読む．

### 要件

- ファイルの内容(バイト列)から，Gitのblobオブジェクトとしてのハッシュを計算する．
  - ハッシュは，`blob <バイト数>\0`というヘッダーと内容を連結したバイト列のSHA-1である．
  - 結果は40桁の小文字の16進数の文字列で返す．
- `src/main.rs`は，標準入力をすべて読み，そのハッシュを1行で出力する．

### 使用例

```rust
assert_eq!(rgit::hash_blob(b"hello\n"), "ce013625030ba8dba906f756967f9e9ca394464a");
assert_eq!(rgit::hash_blob(b""), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
```

```console
$ printf 'hello\n' | cargo run -q
ce013625030ba8dba906f756967f9e9ca394464a
$ printf 'hello\n' | git hash-object --stdin
ce013625030ba8dba906f756967f9e9ca394464a
```

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `object` | `pub fn hash_blob(data: &[u8]) -> String`，バイト列を16進数の文字列にする`fn to_hex(bytes: &[u8]) -> String` |
| ルート(`lib.rs`) | `mod object;`と，`hash_blob`の`pub use` |
| `main.rs` | 標準入力を読み，`hash_blob`の結果を出力する |

### 書くときに考えること

- `to_hex`と`hash_blob`のどちらから作ると，1項目ずつ小さく進めるか．
- 境界を考える．空のバイト列，`0x00`や`0x0f`のように16進数で1桁になるバイトはどうなるか．
- 期待値は，本物の`git hash-object --stdin`で調べられる．文字列でないバイト列は`printf '\x00\xff'`のように作る．
- 単体テスト(`object`の中)と結合テスト(`tests/`から公開APIを呼ぶ)に分ける．使用例は結合テストにする．

## 0-4 図

[型とモジュールの図の書き方](../../../../docs/design.md)を読み，`design/types.md`に，このIterationの終わりの状態を描く．
ファイルには，描くものを説明するコメントが入っている．コメントは描き終えたら消す．

- 「作るもの」のモジュールのうち，図に描くのはどれか．`lib.rs`と`main.rs`は描かない．
- `object`の公開関数はどれか．`to_hex`は公開関数か．図に描かないものは，図の下の説明に書く．

描き終えたら，Mermaidの構文を検査する．

```console
$ node ../../../scripts/check-mermaid.mjs design/types.md
mermaid: 1 blocks, 0 errors
```

## 0-5 テスト駆動の実装

`TESTLIST.md`の項目を上から1つずつ，Red → Green → Refactorで実装する．
テストを書いたら，実装の前に`cargo test`を実行して，失敗することを確かめる．

### モジュールを用意する

`src/lib.rs`にモジュールを宣言し，`src/object.rs`を作る．
`object`の単体テストは，`src/object.rs`の末尾の`#[cfg(test)] mod tests`に書く．

```rust
mod object;
```

### 実装のヒント

- 最初のテストは，テストする関数がまだないのでコンパイルエラーになる．これもRedである．
- `hash_blob`の最初のテストは，期待値をそのまま返す仮実装でも通せる．2つ目の期待値のテストで，本当の計算が必要になる．
- 使わない引数はコンパイラーが警告する．仮実装の間は，引数の名前を`_data`のように`_`で始める．
- ヘッダーは`format!`で作り，`as_bytes()`でバイト列にする．`update`を2回呼べば，ヘッダーと内容を連結したバイト列のハッシュになる．
- `finalize`の結果は，`&`を付けて`to_hex`に渡せる．
- `cargo build`は，どこからも使われていない関数を警告する．`lib.rs`で`pub use`すると，警告は消える．

### `main.rs`を作る

単体テストと結合テストが通ったら，`src/main.rs`を作り，標準入力のハッシュを出力する．
`cargo run`は，バイナリクレートをビルドして実行する．`-q`を付けると，ビルドの表示を省く．

```console
$ printf 'hello\n' | cargo run -q
ce013625030ba8dba906f756967f9e9ca394464a
```

本物の`git hash-object --stdin`の結果と比べる．

### ツールの操作

- 単体テストだけを実行するには`cargo test --lib`を使う．
- `cargo test hashes`のように名前の一部を渡すと，その文字列を名前に含むテストだけを実行する．
- 項目を1つ通すたびに`cargo fmt`で整形する．最後に`cargo clippy`を実行し，警告がないことを確かめる．

## 0-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．自分にない項目，自分にだけある項目はどれか．その項目は必要か．
2. `hash_blob`の1つ目のテストは仮実装で通った．仮実装で通るテストを書くことに，どんな意味があるか．
3. 単体テストと結合テストの両方で`hash_blob`を確かめている．結合テストは，単体テストでは確かめられない何を確かめているか．
4. `to_hex`を公開しなかった．公開すると，利用者と`rgit`の作者にとって何が変わるか．
5. 図と実装を見比べ，違うところがあれば図を直す．図は，次のコマンドでコードと照合できる．

```console
node ../../../scripts/check-design.mjs .
```

## 0-7 発展課題

`main.rs`は，引数でファイルのパスを指定されたら，標準入力の代わりにそのファイルを読むようにする．
`std::env::args()`でコマンドラインの引数を，`std::fs::read(パス)`でファイルの中身を`Vec<u8>`で得られる．

```console
$ printf 'hello\n' > hello.txt
$ cargo run -q -- hello.txt
ce013625030ba8dba906f756967f9e9ca394464a
```

これまでと同じく，テストリスト，図，実装の順に進める．
