# Iteration 0：Cargo，関数，バイト列

Iteration 0では，バイト列からblobのハッシュを計算する関数を作る．
このノートでは，Cargoのパッケージ，関数，整数とバイト列，文字列の組み立て，外部のクレート，テスト，モジュールを説明する．

## Cargo

Cargoは，Rustのビルドツール兼パッケージマネージャーである．
1つの`Cargo.toml`で管理する単位をパッケージと呼び，パッケージはクレートを含む．

- ライブラリクレート：`src/lib.rs`から始まる．ほかのクレートから使う関数や型を公開する．
- バイナリクレート：`src/main.rs`から始まる．`main`関数を持ち，実行できるプログラムになる．

1つのパッケージは，ライブラリクレートとバイナリクレートを1つずつ持てる．
バイナリクレートからは，同じパッケージのライブラリクレートを，パッケージの名前で使える(`rgit::hash_blob`)．

`Cargo.toml`の`[dependencies]`には，使う外部のクレートと版を書く．
`cargo add クレート名`は，最新の版を調べて`[dependencies]`に書き加える．

```toml
[package]
name = "rgit"
version = "0.1.0"
edition = "2024"

[dependencies]
sha1 = "0.11.0"
```

`edition`は，Rustの言語の版である．このハンズオンでは2024を使う．

## 関数

関数は`fn`で定義する．引数と戻り値には型を書く．
最後の式に`;`を付けなければ，それが戻り値になる．

```rust
fn double(x: u32) -> u32 {
    x * 2
}
```

`pub`を付けた関数は，モジュールの外から呼べる．付けなければ，そのモジュールの中でだけ使える．

変数は`let`で宣言する．変数は既定で変更できない．値を変えるには`let mut`で宣言する．

```rust
let mut total = 0;
total += 1;
```

## 整数とバイト

Rustの整数型は，大きさと符号で分かれる．

| 型 | 範囲 |
| --- | --- |
| `u8` | 0〜255(1バイト) |
| `u32` | 0〜4294967295 |
| `i64` | -9223372036854775808〜9223372036854775807 |
| `usize` | 0〜(64ビットの計算機では)2^64-1．長さや添字に使う |

ファイルの中身は，`u8`の並び(バイト列)として扱う．

### `Vec<u8>`と`&[u8]`

- `Vec<u8>`：伸び縮みするバイト列．中身を持つ(所有する)．
- `&[u8]`：どこかにあるバイト列の一部分を指す参照(スライス)．中身を持たず，借りて読む．

関数の引数には，ふつう`&[u8]`を使う．`Vec<u8>`の値`v`からは`&v`で，配列`[1, 2, 3]`からは`&[1, 2, 3]`でスライスを作れる．
どちらからでも同じ関数に渡せる．所有と借用はIteration 1で詳しく扱う．

```rust
fn first(bytes: &[u8]) -> u8 {
    bytes[0]
}

let v: Vec<u8> = vec![10, 20];
assert_eq!(first(&v), 10);
assert_eq!(first(&[30, 40]), 30);
```

`b"hello"`と書くと，文字列ではなくバイト列のリテラルになる．型は`&[u8; 5]`(5バイトの配列への参照)で，`&[u8]`の引数に渡せる．

`for`は，スライスの要素を順に取り出す．`for byte in bytes`の`byte`は，要素への参照`&u8`である．

```rust
fn sum(bytes: &[u8]) -> u32 {
    let mut total = 0;
    for byte in bytes {
        total += *byte as u32;
    }
    total
}
```

`*byte`は，参照から値を取り出す．`as u32`は，`u8`を`u32`に変換する．

## 文字列の組み立て

`String`は，伸び縮みするUTF-8の文字列である．`&str`は，文字列を指す参照である．
`format!`は，書式に値を埋め込んだ`String`を作る．`{}`に値が入る．

```rust
let n = 6;
assert_eq!(format!("blob {}", n), "blob 6");
assert_eq!(format!("blob {n}"), "blob 6");
```

`{n}`のように，`{}`の中に変数の名前を書くこともできる．
`{:x}`は値を16進数にする．`{:02x}`は，2桁に満たなければ前を0で埋める．

```rust
assert_eq!(format!("{:x}", 255), "ff");
assert_eq!(format!("{:02x}", 10u8), "0a");
```

`10u8`の`u8`は，整数のリテラルの型を指定する．

文字列の後ろに足すには，`let mut s = String::new();`で空の文字列を作り，`s.push_str(...)`を呼ぶ．
`String`の`as_bytes()`は，中身のUTF-8のバイト列を`&[u8]`で返す．

```rust
assert_eq!("abc".as_bytes(), &[97, 98, 99]);
assert_eq!("あ".len(), 3);
```

`len()`は，文字の数ではなくバイトの数を返す．「あ」はUTF-8で3バイトである．
文字列の中の`\0`はバイト値0の文字，`\n`は改行である．

## 外部のクレートとトレイト

`sha1`クレートは，SHA-1を計算する型`Sha1`を持つ．

```rust
use sha1::{Digest, Sha1};

let mut hasher = Sha1::new();
hasher.update(b"ab");
hasher.update(b"c");
let digest = hasher.finalize();
assert_eq!(digest.len(), 20);
assert_eq!(digest[0], 0xa9);
```

- `use`は，ほかのモジュールやクレートの名前を，短い名前で使えるようにする．
- `Sha1::new()`で計算を始め，`update`でバイト列を何回かに分けて渡し，`finalize`で20バイトの結果を受け取る．
- `finalize`の結果は，`&`を付けると`&[u8]`として関数に渡せる．

`new`，`update`，`finalize`は，トレイト`Digest`のメソッドである．
トレイトは，型が持つメソッドの集まりを定める仕組みである(Iteration 9で自分のトレイトを作る)．
トレイトのメソッドを呼ぶには，そのトレイトを`use`しておく必要がある．`Digest`を`use`しないと，`Sha1::new()`はコンパイルエラーになる．

## テスト

`#[test]`を付けた関数がテストになる．`cargo test`で実行する．
`assert_eq!(実際の値, 期待値)`は，2つが等しくなければテストを失敗させる．

単体テストは，ファイルの末尾の`#[cfg(test)] mod tests`に書く．
`#[cfg(test)]`を付けたモジュールは，テストのときだけコンパイルされる．
`use super::*;`で，親のモジュール(テストされる側)の関数を，非公開のものも含めて使える．

```rust
fn double(x: u32) -> u32 {
    x * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles() {
        assert_eq!(double(21), 42);
    }
}
```

結合テストは，パッケージの`tests/`の下のファイルに書く．ライブラリを外から使うので，公開した関数だけを`rgit::hash_blob`のように呼べる．

## モジュール

1つのクレートの中を，モジュールに分けられる．
`src/lib.rs`に`mod object;`と書くと，`src/object.rs`がモジュール`object`になる．

`mod object;`だけでは，`object`は非公開のモジュールで，クレートの外から`rgit::object::hash_blob`とは書けない．
`pub use object::hash_blob;`と書くと，`hash_blob`をクレートの直下に公開できる．利用者は`rgit::hash_blob`と書ける．

```rust
// src/lib.rs
mod object;

pub use object::hash_blob;
```

## `Option`と`match`

値がないこともある場合は，`Option<T>`で表す．
値があれば`Some(値)`，なければ`None`である．
`match`は，値の形ごとに処理を分け，`Some`の中の値に名前を付けて取り出す．

```rust
fn describe(arg: Option<&str>) -> String {
    match arg {
        Some(name) => format!("file {name}"),
        None => String::from("stdin"),
    }
}

assert_eq!(describe(Some("a.txt")), "file a.txt");
assert_eq!(describe(None), "stdin");
```

`match`は，すべての形(`Some`と`None`)を扱わなければコンパイルエラーになる．

## 標準入力を読む

`main.rs`の`main`関数は，プログラムを実行すると呼ばれる．
標準入力をすべて読むには，`std::io::stdin()`の`read_to_end`に，読んだバイトを入れる`Vec<u8>`を渡す．

```rust
use std::io::Read;

fn main() {
    let mut data = Vec::new();
    std::io::stdin().read_to_end(&mut data).unwrap();
    println!("{} bytes", data.len());
}
```

- `read_to_end`は，トレイト`std::io::Read`のメソッドなので，`use std::io::Read;`が必要である．Iteration 3で`Read`を詳しく扱う．
- `&mut data`は，`data`を書き換えられる形で貸す．
- 読み込みは失敗することがあるので，`read_to_end`は`Result`を返す．`unwrap()`は，成功なら中の値を取り出し，失敗ならプログラムを止める．エラーの扱いはIteration 1と2で学ぶ．
- `println!`は，`format!`と同じ書式で，標準出力に1行を書く．
