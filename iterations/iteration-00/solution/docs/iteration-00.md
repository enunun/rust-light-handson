# Iteration 0：blobのハッシュの計算(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 0-1 準備

演習では`cargo init --lib --name rgit`でパッケージを作り，`cargo add sha1`で依存を加えた．
受講者の`Cargo.toml`は次のようになる．

```toml
[package]
name = "rgit"
version = "0.1.0"
edition = "2024"

[dependencies]
sha1 = "0.11.0"
```

模範解答の`Cargo.toml`は，パッケージ名が`rgit-00-solution`で，`[lib]`の`name`でライブラリの名前を`rgit`にしている．
模範解答はすべてリポジトリのルートのワークスペースに属するので，パッケージ名をIterationごとに変えている．
ライブラリの名前は同じ`rgit`なので，テストの`rgit::hash_blob`は演習と模範解答で共通である．

```toml
[package]
name = "rgit-00-solution"
version = "0.1.0"
edition = "2024"

[lib]
name = "rgit"

[dependencies]
sha1 = "0.11.0"
```

## 0-2 文法と概念

課題の解答例である．4つのテストはすべて通る．

```rust
#[cfg(test)]
mod tests {
    use sha1::{Digest, Sha1};

    #[test]
    fn byte_strings_and_str() {
        let bytes: &[u8] = b"abc";
        assert_eq!(bytes.len(), 3);
        assert_eq!(bytes[0], 97);
        assert_eq!("あ".len(), 3);
        assert_eq!("あ".as_bytes(), &[0xe3, 0x81, 0x82]);
    }

    #[test]
    fn hex_format() {
        assert_eq!(format!("{:02x}", 10u8), "0a");
        assert_eq!(format!("{:x}", 255), "ff");
        assert_eq!(format!("{:02x}", 255u8), "ff");
    }

    fn sum(bytes: &[u8]) -> u32 {
        let mut total = 0;
        for byte in bytes {
            total += *byte as u32;
        }
        total
    }

    #[test]
    fn sums_bytes() {
        assert_eq!(sum(b""), 0);
        assert_eq!(sum(&[1, 2, 255]), 258);
    }

    #[test]
    fn sha1_of_abc() {
        let digest = Sha1::digest(b"abc");
        assert_eq!(digest.len(), 20);
        assert_eq!(digest[0], 0xa9);
    }
}
```

- 1：`b"abc"`の要素は文字ではなくバイトなので，先頭は`'a'`の文字コード`97`である．「あ」はUTF-8で3バイトになる．
- 3：`sum`の`total`の型は，戻り値の`u32`から決まる．`*byte as u32`としないと，`u8`の`255`を足したときに`u8`の範囲を超えてしまう．
- 4：`Sha1::digest`は，`new`，`update`，`finalize`を1回で行う関数である．`printf 'abc' | sha1sum`の結果`a9993e36…`の先頭`a9`と一致する．

## 0-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

### 並べ方

- `hash_blob`は`to_hex`を使うので，`to_hex`から作る．小さな部品から作ると，1項目で書くコードが少なくなる．
- `to_hex`は，空のバイト列から始める．次に，1桁になるバイト(`0x00`，`0x0f`)と2桁のバイト(`0xab`，`0xff`)を1つのテストで確かめる．
- `hash_blob`は，空のデータから始める．期待値は`printf '' | git hash-object --stdin`で調べた．

### 単体テストと結合テスト

- 単体テストは，`object`の中で`to_hex`と`hash_blob`を確かめる．`to_hex`は非公開なので，単体テストでしか確かめられない．
- 結合テストは，クレートの外から`rgit::hash_blob`を呼び，`git hash-object --stdin`の結果と比べる．`pub use`で公開できていることも確かめられる．
- 文字列でないバイト列の項目は，`printf '\x00\xff\x10' | git hash-object --stdin`で期待値を調べた．

## 0-4 図

模範解答は[design/types.md](../design/types.md)にある．

- 図に描くモジュールは`object`だけである．`lib.rs`は名前を公開するだけ，`main.rs`は入出力をするだけなので描かない．
- `object`の公開関数は`hash_blob`だけなので，`object_mod`に`hash_blob`を書いた．
- `to_hex`は非公開の補助関数なので，図の下の説明に書いた．外部のクレート`sha1`も説明に書いた．

## 0-5 テスト駆動の実装

### モジュールを用意する

`src/lib.rs`に`mod object;`と書き，空の`src/object.rs`を作った．

### 空のバイト列は，空の文字列になる

```rust
#[test]
fn empty_bytes_become_empty_hex() {
    assert_eq!(to_hex(&[]), "");
}
```

`to_hex`がないので，コンパイルエラーになる(Red)．

```text
error[E0425]: cannot find function `to_hex` in this scope
 --> src/object.rs:7:20
  |
7 |         assert_eq!(to_hex(&[]), "");
  |                    ^^^^^^ not found in this scope
```

空の`String`を返す仮実装で通す(Green)．

```rust
fn to_hex(_bytes: &[u8]) -> String {
    String::new()
}
```

### 各バイトは，2桁の小文字の16進数になる

```rust
#[test]
fn each_byte_becomes_two_lowercase_hex_digits() {
    assert_eq!(to_hex(&[0x00, 0x0f, 0xab, 0xff]), "000fabff");
}
```

仮実装は空の文字列を返すので失敗する．
`for`で1バイトずつ`{byte:02x}`の書式で文字列にし，後ろに足す．

```rust
/// バイト列を，1バイトあたり2桁の小文字の16進数にする．
fn to_hex(bytes: &[u8]) -> String {
    let mut hex = String::new();
    for byte in bytes {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}
```

`{:x}`だけでは，`0x00`が`0`に，`0x0f`が`f`になり，桁がずれる．このテストは，`02`の指定を忘れると失敗する．

### 空のデータのblobのハッシュは`e69de29…`になる

```rust
#[test]
fn hashes_empty_blob() {
    assert_eq!(hash_blob(b""), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
}
```

期待値をそのまま返す仮実装で通す．

```rust
pub fn hash_blob(_data: &[u8]) -> String {
    String::from("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391")
}
```

### `hello\n`のblobのハッシュは`ce01362…`になる

```rust
#[test]
fn hashes_blob_with_header() {
    assert_eq!(
        hash_blob(b"hello\n"),
        "ce013625030ba8dba906f756967f9e9ca394464a"
    );
}
```

仮実装のままでは失敗する．

```text
thread 'object::tests::hashes_blob_with_header' (4449) panicked at src/object.rs:12:9:
assertion `left == right` failed
  left: "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
 right: "ce013625030ba8dba906f756967f9e9ca394464a"
```

ヘッダーと内容を順に`update`に渡して，SHA-1を計算する．

```rust
use sha1::{Digest, Sha1};

/// データをblobオブジェクトにしたときのハッシュを，40桁の16進数で返す．
pub fn hash_blob(data: &[u8]) -> String {
    let header = format!("blob {}\0", data.len());
    let mut hasher = Sha1::new();
    hasher.update(header.as_bytes());
    hasher.update(data);
    to_hex(&hasher.finalize())
}
```

`update`を2回呼ぶので，ヘッダーと内容を1つの`Vec<u8>`に連結しなくてよい．

### 結合テスト

`src/lib.rs`で`hash_blob`を公開する．

```rust
mod object;

pub use object::hash_blob;
```

`tests/hash_blob.rs`で，`git hash-object --stdin`で調べた期待値と比べる．

```rust
// 期待値は，同じ内容を`git hash-object --stdin`に渡して得た値である．

#[test]
fn hash_matches_git_for_text() {
    assert_eq!(
        rgit::hash_blob(b"hello\n"),
        "ce013625030ba8dba906f756967f9e9ca394464a"
    );
}

#[test]
fn hash_matches_git_for_bytes_that_are_not_text() {
    assert_eq!(
        rgit::hash_blob(&[0x00, 0xff, 0x10]),
        "d553b66b6a09553981f4c9b617e12de89b8fe30c"
    );
}
```

どちらも，実装を変えずに通る．

### `main.rs`

```rust
use std::io::Read;

fn main() {
    let mut data = Vec::new();
    std::io::stdin().read_to_end(&mut data).unwrap();
    println!("{}", rgit::hash_blob(&data));
}
```

```console
$ printf 'hello\n' | cargo run -q
ce013625030ba8dba906f756967f9e9ca394464a
```

## 0-6 振り返り

1. 模範解答は，`to_hex`と`hash_blob`をそれぞれ2項目で確かめている．`to_hex`の項目がなくても`hash_blob`の項目で誤りは見つかるが，どの関数の誤りかが分かりにくくなる．
2. 仮実装で通るテストによって，テストが正しく書けていること，正しい理由で失敗し，正しい理由で通ることを先に確かめられる．仮実装を本当の実装に置き換えるのは，次の項目を通すためである．
3. 結合テストは，`hash_blob`がクレートの外から`rgit::hash_blob`という名前で使えることを確かめる．`pub use`を忘れると，単体テストは通っても結合テストはコンパイルエラーになる．
4. `to_hex`を公開すると，利用者がそれに頼るので，名前や振る舞いを変えにくくなる．公開する名前は，利用者に必要なものだけに絞る．Iteration 1では，`to_hex`の役目はオブジェクトIDの型の表示に移る．
5. 図に描いた`hash_blob`と，コードの公開関数は一致している．

## 0-7 発展課題

`std::env::args()`の`nth(1)`は，1つ目の引数を`Option<String>`で返す．`match`で，引数があればファイルを，なければ標準入力を読む．

```rust
use std::io::Read;

fn main() {
    let data = match std::env::args().nth(1) {
        Some(path) => std::fs::read(path).unwrap(),
        None => {
            let mut data = Vec::new();
            std::io::stdin().read_to_end(&mut data).unwrap();
            data
        }
    };
    println!("{}", rgit::hash_blob(&data));
}
```

```console
$ cargo run -q -- hello.txt
ce013625030ba8dba906f756967f9e9ca394464a
```

`nth(0)`はプログラム自身の名前である．`match`の腕は値を返すので，どちらの腕も`Vec<u8>`を返し，それを`data`にする．
