# Iteration 1：オブジェクトID(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 1-1 準備

演習の`Cargo.toml`は，Iteration 0の模範解答の`Cargo.toml`からパッケージ名だけを`rgit`に変えたものである．
引き継いだ単体テスト4つと結合テスト2つが通る．

## 1-2 文法と概念

課題1〜4の解答例である．`src/lib.rs`の末尾に書いた．

```rust
#[cfg(test)]
mod practice {
    use std::fmt;
    use std::str::FromStr;

    #[test]
    fn clone_keeps_the_original() {
        let hex = String::from("ce01362");
        let moved = hex.clone();
        assert_eq!(hex.len() + moved.len(), 14);
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Point(i32, i32);

    #[test]
    fn copy_keeps_the_original() {
        let p = Point(1, 2);
        let q = p;
        assert_eq!(p, q);
    }

    impl fmt::Display for Point {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&format!("({}, {})", self.0, self.1))
        }
    }

    #[test]
    fn displays_point() {
        assert_eq!(Point(1, 2).to_string(), "(1, 2)");
    }

    #[derive(Debug, PartialEq)]
    enum PointError {
        MissingParen,
        MissingComma,
        InvalidNumber,
    }

    impl FromStr for Point {
        type Err = PointError;

        fn from_str(s: &str) -> Result<Point, PointError> {
            let inner = match s.strip_prefix('(') {
                Some(rest) => match rest.strip_suffix(')') {
                    Some(inner) => inner,
                    None => return Err(PointError::MissingParen),
                },
                None => return Err(PointError::MissingParen),
            };
            let (x, y) = match inner.split_once(", ") {
                Some(pair) => pair,
                None => return Err(PointError::MissingComma),
            };
            match (x.parse(), y.parse()) {
                (Ok(x), Ok(y)) => Ok(Point(x, y)),
                _ => Err(PointError::InvalidNumber),
            }
        }
    }

    #[test]
    fn parses_point() {
        assert_eq!("(1, 2)".parse::<Point>(), Ok(Point(1, 2)));
        assert_eq!("1, 2)".parse::<Point>(), Err(PointError::MissingParen));
        assert_eq!("(1 2)".parse::<Point>(), Err(PointError::MissingComma));
        assert_eq!("(a, 2)".parse::<Point>(), Err(PointError::InvalidNumber));
    }
}
```

- 1：`.clone()`を外すと，ノートと同じ`E0382`(`borrow of moved value`)になる．
- 2：`Copy`を外すと，`assert_eq!(p, q)`で`p`がムーブ済みだというエラーになる．
- 4：`strip_prefix`は，先頭が一致すれば残りを`Some`で，一致しなければ`None`を返す．`split_once(", ")`は，最初の`", "`の前後を組で返す．数の変換は2つの`parse`の結果を組にして`match`で分けた．`(Ok(x), Ok(y))`以外はすべて`_`に当たる．
- 5：`.clone()`の版では，`hex`の下線は`hex.len()`まで続く．`let moved = hex;`の版では，`hex`の下線は7行目の`hex`で終わり，その位置には黄色(ムーブ)の下線が引かれる．ムーブした時点で，`hex`の生きている範囲は終わる．

## 1-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

- `oid`は，表示(`Display`と`short`)，正しい文字列からの変換，誤った文字列のエラーの順に並べた．表示を先に作ると，変換のテストの期待値を`to_string()`で書ける．
- 誤った文字列は，短い場合，長い場合，16進数でない文字の場合に分けた．16進数でない文字は，最後の位置(39)に置いた．先頭に置くと，位置を数え間違えても0で通ってしまうことがある．
- `hash_blob`の戻り値が変わるので，既存の単体テストと結合テストを「`to_string()`と比べるように変える」項目にした．`to_hex`のテストは，同じ振る舞いを`ObjectId`の表示のテストに移した．
- 使用例は，結合テスト`tests/object_id.rs`にした．

## 1-4 図

模範解答は[design/types.md](../design/types.md)にある．Iteration 0からの変更は次のとおりである．

- `oid`の名前空間を加え，`ObjectId`と`ParseObjectIdError`を置いた．`ObjectId`には，非公開のフィールド`0`，2つのメソッド，実装するトレイト`Display`と`FromStr`を書いた．
- `hash_blob`の戻り値を`ObjectId`に変え，`object_mod`から`ObjectId`への依存を描いた．
- `FromStr`の`from_str`が`ParseObjectIdError`を返すので，`ObjectId`から`ParseObjectIdError`への依存を描いた．
- 導出するトレイトと，大文字を受け付けることは，図の下の説明に書いた．

## 1-5 テスト駆動の実装

### 20バイトの値は，2桁の小文字の16進数で表示する

`src/lib.rs`に`mod oid;`を加え，`src/oid.rs`にテストを書く．

```rust
#[test]
fn displays_each_byte_as_two_lowercase_hex_digits() {
    let mut bytes = [0xab; 20];
    bytes[0] = 0x00;
    bytes[1] = 0x0f;
    let id = ObjectId::from_bytes(bytes);
    assert_eq!(
        id.to_string(),
        "000fabababababababababababababababababab"
    );
}
```

`ObjectId`がないのでコンパイルエラーになる．型と`from_bytes`を作り，`to_hex`の中身を`Display`に移す．

```rust
use std::fmt;

/// GitのオブジェクトID．中身はSHA-1の20バイトである．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectId([u8; 20]);

impl ObjectId {
    pub fn from_bytes(bytes: [u8; 20]) -> ObjectId {
        ObjectId(bytes)
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut hex = String::new();
        for byte in &self.0 {
            hex.push_str(&format!("{byte:02x}"));
        }
        f.write_str(&hex)
    }
}
```

`Copy`は，`ObjectId`が20バイトの配列だけを持つ小さな値なので付けた．整数と同じように，代入しても元の変数を使い続けられる．

### `short`は先頭の7桁を返す

```rust
#[test]
fn short_is_first_seven_digits() {
    let id = ObjectId::from_bytes([0xab; 20]);
    assert_eq!(id.short(), "abababa");
}
```

```rust
/// 先頭の7桁を返す．
pub fn short(&self) -> String {
    let mut hex = self.to_string();
    hex.truncate(7);
    hex
}
```

### 40桁の16進数から作る

```rust
const HELLO: &str = "ce013625030ba8dba906f756967f9e9ca394464a";

#[test]
fn parses_forty_hex_digits() {
    let id: ObjectId = HELLO.parse().unwrap();
    assert_eq!(id.to_string(), HELLO);
}
```

`FromStr`がないのでコンパイルエラーになる．エラーの型を先に作り，1文字ずつ数にして積み上げる．

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum ParseObjectIdError {
    InvalidChar { position: usize, ch: char },
}

impl FromStr for ObjectId {
    type Err = ParseObjectIdError;

    fn from_str(s: &str) -> Result<ObjectId, ParseObjectIdError> {
        let mut bytes = [0u8; 20];
        for (position, ch) in s.chars().enumerate() {
            let digit = match ch.to_digit(16) {
                Some(digit) => digit as u8,
                None => return Err(ParseObjectIdError::InvalidChar { position, ch }),
            };
            bytes[position / 2] = bytes[position / 2] * 16 + digit;
        }
        Ok(ObjectId(bytes))
    }
}
```

1バイト目は位置0と1の2文字からなる．位置0の数を16倍し，位置1の数を足せば，そのバイトの値になる．
`to_digit`が`None`を返す場合は`return`でエラーを返す．エラーのテストはまだないが，`match`はすべての場合を書かなければならないので，ここで列挙子を1つ作った．

### 大文字の16進数からも作る

```rust
#[test]
fn parses_uppercase_hex_digits() {
    let upper = HELLO.to_uppercase();
    assert_eq!(upper.parse::<ObjectId>(), HELLO.parse::<ObjectId>());
}
```

`to_digit(16)`は大文字も受け付けるので，実装を変えずに通る．

### 長さのエラー

```rust
#[test]
fn rejects_short_string_with_its_length() {
    assert_eq!(
        "ce01".parse::<ObjectId>(),
        Err(ParseObjectIdError::InvalidLength(4))
    );
}
```

列挙子`InvalidLength`がないのでコンパイルエラーになる．列挙子を加え，`from_str`の先頭で長さを調べる．

```rust
if s.len() != 40 {
    return Err(ParseObjectIdError::InvalidLength(s.len()));
}
```

この検査がないと，41桁の文字列は`bytes[20]`に書こうとしてパニックになる．41桁のテストは，その誤りを見つける．

```rust
#[test]
fn rejects_long_string_with_its_length() {
    let long = format!("{HELLO}0");
    assert_eq!(
        long.parse::<ObjectId>(),
        Err(ParseObjectIdError::InvalidLength(41))
    );
}
```

### 16進数でない文字のエラー

```rust
#[test]
fn rejects_non_hex_char_with_its_position() {
    let text = format!("{}g", &HELLO[..39]);
    assert_eq!(
        text.parse::<ObjectId>(),
        Err(ParseObjectIdError::InvalidChar {
            position: 39,
            ch: 'g'
        })
    );
}
```

実装を変えずに通る．

### `hash_blob`が`ObjectId`を返す

`hash_blob`の戻り値を`ObjectId`に変える．`finalize()`の結果は`.into()`で`[u8; 20]`にできる．

```rust
use crate::oid::ObjectId;

/// データをblobオブジェクトにしたときのIDを返す．
pub fn hash_blob(data: &[u8]) -> ObjectId {
    let header = format!("blob {}\0", data.len());
    let mut hasher = Sha1::new();
    hasher.update(header.as_bytes());
    hasher.update(data);
    ObjectId::from_bytes(hasher.finalize().into())
}
```

`crate::oid::ObjectId`は，同じクレートの`oid`モジュールの`ObjectId`を指す．
既存のテストは，`ObjectId`と文字列を比べているのでコンパイルエラーになる．

```text
error[E0308]: mismatched types
  --> src/object.rs:22:13
   |
22 |             "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `ObjectId`, found `&str`
```

テストリストのとおり，`hash_blob(b"").to_string()`と比べるように変える．`to_hex`とそのテストは消す．
`main.rs`は`println!("{}", …)`で表示しているので，変えずに動く．

### 結合テスト

`src/lib.rs`で新しい型を公開する．

```rust
mod object;
mod oid;

pub use object::hash_blob;
pub use oid::{ObjectId, ParseObjectIdError};
```

`tests/hash_blob.rs`の2つのテストを`to_string()`と比べるように変え，使用例を`tests/object_id.rs`に書く．

```rust
use rgit::{ObjectId, ParseObjectIdError};

#[test]
fn hash_of_blob_can_be_shown_shortened_and_parsed_back() {
    let id = rgit::hash_blob(b"hello\n");
    assert_eq!(id.to_string(), "ce013625030ba8dba906f756967f9e9ca394464a");
    assert_eq!(id.short(), "ce01362");
    assert_eq!(
        "CE013625030BA8DBA906F756967F9E9CA394464A".parse::<ObjectId>(),
        Ok(id)
    );
}

#[test]
fn short_hex_is_not_an_object_id() {
    assert_eq!(
        "ce01".parse::<ObjectId>(),
        Err(ParseObjectIdError::InvalidLength(4))
    );
}
```

`Ok(id)`は`id`をムーブするが，`ObjectId`は`Copy`なので問題にならない．`Copy`がない場合は，`id`を使う行を前へ移すか，`Ok(id.clone())`と書く．

## 1-6 振り返り

1. 既存のテストへの影響は，ロードマップの「既存テストへの影響」に書いてある．`to_hex`のテストの行き先は書いていないので，自分で決める必要がある．
2. 利点は，`ObjectId`を作る道が`from_bytes`(ハッシュの結果)と`parse`(検査付き)だけになることである．欠点は，テストで好きな値を作るにも`from_bytes`を通す必要があることである．
3. `ObjectId`を受け取る関数は，長さと文字の検査を省ける．`String`を受け取る関数は，受け取るたびに検査するか，検査済みだと信じるしかない．
4. 呼び出す側は`match`で理由ごとに処理を分けられる．例えばIteration 3では，短縮形のIDを受け付けるときに，長さのエラーだけを特別に扱える．
5. 図に描いた型と関数は，コードと一致している．

## 1-7 発展課題

`Debug`を導出から外し，`fmt::Debug`を実装する．`write!`は，`format!`と同じ書式で`f`に書き込む．

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ObjectId([u8; 20]);

impl fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectId({self})")
    }
}
```

`{self}`は，`Display`の実装を使って表示する．
