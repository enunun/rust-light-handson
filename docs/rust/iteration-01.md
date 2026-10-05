# Iteration 1：所有権とニュータイプ

Iteration 1では，オブジェクトIDを表す型`ObjectId`を作る．
このノートでは，所有権とムーブ，借用，`Copy`と`Clone`，固定長の配列，ニュータイプパターン，メソッド，標準のトレイトの実装，エラーを表す`enum`を説明する．
最後に，所有権と借用を目で確かめるツールRustOwlの使い方を説明する．

## 所有権とムーブ

Rustでは，どの値にも所有者(その値を持つ変数)が1つだけある．
所有者がスコープを抜けると，値は片付けられる(`String`なら，文字列のメモリーが解放される)．

`let b = a;`のように値を別の変数に代入すると，所有権が`a`から`b`に移る．これをムーブと呼ぶ．
ムーブしたあとの`a`は使えない．

```rust
let hex = String::from("ce01362");
let moved = hex;
hex.len() + moved.len()
```

```text
error[E0382]: borrow of moved value: `hex`
 --> src/lib.rs:4:5
  |
2 |     let hex = String::from("ce01362");
  |         --- move occurs because `hex` has type `String`, which does not implement the `Copy` trait
3 |     let moved = hex;
  |                 --- value moved here
4 |     hex.len() + moved.len()
  |     ^^^ value borrowed here after move
```

関数に値を渡すときも，戻り値を受け取るときも，同じようにムーブが起きる．
ほかの言語では2つの変数が同じ文字列を指せるが，Rustでは所有者が1つなので，誰がいつ片付けるかがコンパイル時に決まる．

## 借用

値をムーブせずに使わせるには，参照を渡す．これを借用と呼ぶ．

- `&値`：共有の参照．読むだけである．同時にいくつでも作れる．
- `&mut 値`：可変の参照．書き換えられる．ある時点で1つしか作れず，その間は共有の参照も作れない．

```rust
fn length(text: &String) -> usize {
    text.len()
}

let hex = String::from("ce01362");
let n = length(&hex);
assert_eq!(n, 7);
assert_eq!(hex, "ce01362"); // 貸しただけなので，hexはまだ使える
```

Iteration 0の`fn hash_blob(data: &[u8])`も，バイト列を借りて読む関数である．
メソッドの`&self`は「自分を借りる」，`&mut self`は「自分を書き換えられる形で借りる」，`self`は「自分をムーブで受け取る」という意味になる．

## `Copy`と`Clone`

整数や`bool`のように小さく，複製しても問題のない型は`Copy`である．
`Copy`の型では，代入はムーブではなくコピーになり，元の変数も使い続けられる．

```rust
let a: u32 = 7;
let b = a;
assert_eq!(a + b, 14);
```

`Clone`は，`.clone()`で明示的に複製できる型を表す．`String`は`Clone`だが`Copy`ではない．
`.clone()`は中身のメモリーを複製するので，費用がかかる．`Copy`の型の代入は，メモリーをそのまま写すだけである．

自分で作った型は，既定では`Copy`と`Clone`のどちらも実装しない．`#[derive(Clone, Copy)]`を付けると，両方になる．
`Copy`を付けられるのは，すべてのフィールドが`Copy`の型だけである．`String`を持つ型は`Copy`にできない．

```text
error[E0382]: borrow of moved value: `id`
 --> src/lib.rs:7:5
  |
5 |     let id = ObjectId([0; 20]);
  |         -- move occurs because `id` has type `ObjectId`, which does not implement the `Copy` trait
6 |     let copied = id;
  |                  -- value moved here
7 |     id == copied
  |     ^^ value borrowed here after move
```

## 固定長の配列

`[u8; 20]`は，`u8`がちょうど20個並んだ配列の型である．長さも型の一部で，`[u8; 19]`とは別の型になる．
`[0u8; 20]`は，0を20個並べた配列を作る．

```rust
let mut bytes = [0u8; 20];
bytes[0] = 0xce;
assert_eq!(bytes.len(), 20);
```

`Vec<u8>`と違い，配列は長さが変わらない．要素の型が`Copy`なら，配列も`Copy`である．

## ニュータイプパターン

フィールドに名前のない構造体をタプル構造体と呼ぶ．
1つの値だけを包むタプル構造体で新しい型を作る書き方を，ニュータイプパターンと呼ぶ．

```rust
pub struct ObjectId([u8; 20]);
```

`[u8; 20]`をそのまま使う代わりに`ObjectId`で包むと，次のことができる．

- 型で意味を区別できる．SHA-1の結果と，たまたま20バイトのほかのデータを取り違えると，コンパイルエラーになる．
- 不正な値を作れなくできる．フィールドを非公開にすれば，モジュールの外では決めた関数でしか`ObjectId`を作れない．
- 自分の型なので，`Display`などのトレイトを好きに実装できる．

フィールドは`id.0`で読める．非公開のフィールドは，そのモジュールの中でだけ読める．

ほかの言語なら，オブジェクトIDを40桁の文字列のまま持ち回ることが多い．
文字列では，39桁の値や`g`を含む値も作れてしまうので，使うたびに検査が要る．
`ObjectId`は20バイトの配列しか持てないので，作った時点で正しいことが保証され，使う側は検査しなくてよい．

## `impl`とメソッド

`impl 型名 { … }`の中に，その型の関数を書く．
最初の引数が`self`，`&self`，`&mut self`の関数はメソッドで，`値.メソッド()`の形で呼ぶ．
`self`を取らない関数は関連関数で，`型名::関数()`の形で呼ぶ．

```rust
impl ObjectId {
    pub fn from_bytes(bytes: [u8; 20]) -> ObjectId {
        ObjectId(bytes)
    }

    pub fn short(&self) -> String {
        let mut hex = self.to_string();
        hex.truncate(7);
        hex
    }
}
```

`String::truncate(n)`は，先頭のnバイトだけを残す．

## `#[derive]`

`#[derive(...)]`を型に付けると，よく使うトレイトの実装をコンパイラーが書く．

| トレイト | できるようになること |
| --- | --- |
| `Debug` | `{:?}`で表示する．`assert_eq!`が失敗したときの表示に必要 |
| `Clone` | `.clone()`で複製する |
| `Copy` | 代入や引数渡しでムーブせずにコピーする |
| `PartialEq`，`Eq` | `==`で比べる．`assert_eq!`に必要 |

## 標準のトレイトを実装する

トレイトは，型が持つべきメソッドを定めたものである．`impl トレイト for 型 { … }`で実装する．

### `Display`

`Display`を実装した型は，`{}`で表示でき，`to_string()`で`String`にできる．

```rust
use std::fmt;

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

`fmt`は，表示する文字列を`f`に書く．`f.write_str`の結果(`fmt::Result`)をそのまま返す．
`&self.0`は，フィールドの配列を借りる．`for`で要素を1つずつ取り出せる．

### `FromStr`と`parse`

`FromStr`を実装した型は，`"…".parse::<型>()`で文字列から作れる．
`type Err`に，失敗したときのエラーの型を書く．これを関連型と呼ぶ．

```rust
use std::str::FromStr;

impl FromStr for ObjectId {
    type Err = ParseObjectIdError;

    fn from_str(s: &str) -> Result<ObjectId, ParseObjectIdError> {
        // …
    }
}

let id: ObjectId = "ce013625030ba8dba906f756967f9e9ca394464a".parse().unwrap();
```

受け取る変数に型を書けば，`parse::<ObjectId>()`の`::<ObjectId>`は省ける．

## `Result`とエラーの`enum`

失敗しうる処理は，`Result<T, E>`を返す．成功なら`Ok(値)`，失敗なら`Err(エラー)`である．
エラーの型には，失敗の理由ごとに列挙子を持つ`enum`を使う．列挙子はデータを持てる．

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum ParseObjectIdError {
    InvalidLength(usize),
    InvalidChar { position: usize, ch: char },
}
```

`InvalidLength(usize)`はタプルの形，`InvalidChar { position, ch }`は名前付きのフィールドの形の列挙子である．
利用者は`match`で理由を分けて，それぞれのデータを取り出せる．
例外を投げる言語と違い，関数が失敗しうることと，失敗の種類が型に現れる．

`return Err(…);`で，関数の途中から失敗を返せる．

```rust
if s.len() != 40 {
    return Err(ParseObjectIdError::InvalidLength(s.len()));
}
```

## 文字列から文字を読む

- `s.chars()`は，文字列の文字を先頭から1つずつ返すイテレーターである．
- `.enumerate()`を付けると，`(位置, 文字)`の組を返す．位置は0から数える．
- `ch.to_digit(16)`は，16進数の1桁なら`Some(値)`を，そうでなければ`None`を返す．大文字の`A`〜`F`も受け付ける．

```rust
for (position, ch) in "a1".chars().enumerate() {
    println!("{position}: {ch}");
}
assert_eq!('f'.to_digit(16), Some(15));
assert_eq!('F'.to_digit(16), Some(15));
assert_eq!('g'.to_digit(16), None);
```

`&s[..39]`は，文字列の先頭から39バイトまでを指す`&str`である．`..`は範囲を表す．

## 定数

`const 名前: 型 = 値;`で定数を定義できる．テストで何度も使う値をまとめるのに便利である．

```rust
const HELLO: &str = "ce013625030ba8dba906f756967f9e9ca394464a";
```

## 型の変換`into`

`a.into()`は，`a`を別の型に変換する．変換先の型は，受け取る側から決まる．
`sha1`の`finalize()`の結果は，`.into()`で`[u8; 20]`に変換できる．

```rust
ObjectId::from_bytes(hasher.finalize().into())
```

## RustOwlで所有権を見る

RustOwlは，変数の所有権と借用が，ソースコードのどこからどこまで続くかを色付きの下線で示すツールである．
Dev Containerには，VS Codeの拡張機能とコマンドの両方が入っている．

### VS Codeで見る

1. Rustのファイルを保存する．初回は解析に少し時間がかかる．
2. 調べたい変数にカーソルを置き，2秒ほど待つ．
3. 変数が生きている範囲，借用されている範囲，ムーブされる位置に下線が引かれる．

### 端末で見る

`mise run rustowl -- 関数のパス 変数名`で，関数の中の1つの変数を端末に表示する．パッケージのディレクトリで実行する．
関数のパスは`crate::`を省いて，`object::hash_blob`のように書く．

```console
mise run rustowl -- object::hash_blob hasher
```

下線の色の意味は，出力の最後に表示される．

```text
Legend:
  --- definitely live (lifetime)
  ~~~ maybe live
  --- immutable borrow
  --- mutable borrow
  --- move / call
  ~~~ outlive / shared mutable
```

| 色 | 意味 |
| --- | --- |
| 緑の`---` | 変数が確実に生きている範囲 |
| 緑の`~~~` | 分岐の先などで，生きている可能性のある範囲 |
| 水色の`---` | 共有の参照で借用されている範囲 |
| 紫の`---` | 可変の参照で借用されている範囲 |
| 黄色の`---` | ムーブされる位置，または関数に渡される位置 |
| 赤の`~~~` | 借用が値より長く生きようとしている範囲(エラーの原因) |

`String`の変数を調べると，代入の位置に黄色(ムーブ)の下線が引かれ，生きている範囲はそこで終わる．
`ObjectId`に`Copy`を付けると，代入のあとも元の変数が生き続ける．
