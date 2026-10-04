# Iteration 4：ライフタイムと参照を持つ構造体

Iteration 4では，treeオブジェクトの内容を，エントリーの列に解析する．
このノートでは，ライフタイム注釈，参照を持つ構造体，借用したまま解析する設計と所有するデータに写す設計の比較，`TryFrom`，スライスから配列への変換，`while`による繰り返しを説明する．

## 参照を持つ構造体

treeのエントリーは，モード，名前，IDからなる．名前を`String`で持てば，解析のたびに名前を複製することになる．
名前を，treeの内容のバイト列の一部を指す`&str`で持てば，複製せずに済む．

構造体のフィールドに参照を持たせるには，ライフタイム引数を書く．

```rust
pub struct TreeEntry<'a> {
    pub mode: Mode,
    pub name: &'a str,
    pub id: ObjectId,
}
```

`'a`はライフタイムの名前で，「`TreeEntry<'a>`の値は，`'a`の間だけ有効な文字列を借りている」ことを表す．
`TreeEntry`の値は，借りている文字列より長く使えない．

## ライフタイム注釈

Iteration 3では，引数の参照が1つの関数の戻り値に，ライフタイムを書かなくてよかった(省略の規則)．
`parse_tree`も同じで，戻り値の`TreeEntry`の名前は，引数`data`から借りたものである．

```rust
pub fn parse_tree(data: &[u8]) -> Result<Vec<TreeEntry<'_>>, Error>
```

`'_`は「省略の規則で決まるライフタイム」を表す．名前を付けて書くと次と同じ意味になる．

```rust
pub fn parse_tree<'a>(data: &'a [u8]) -> Result<Vec<TreeEntry<'a>>, Error>
```

`<'a>`で関数にライフタイム引数を宣言し，引数と戻り値に同じ`'a`を付けて，戻り値が`data`から借りていることを示す．
省略できる場面では，名前を書かずに`'_`を使う．`cargo clippy`は，省略できるのに書いた名前を指摘する．

`impl`にも，ライフタイムを書く．ライフタイムを使わない`impl`では`'_`と書ける．

```rust
impl fmt::Display for TreeEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}\t{}", self.mode, self.mode.kind(), self.id, self.name)
    }
}
```

`write!`は，`writeln!`と同じ書式で，改行を付けずに書く．

### 借用が元の値より長く生きるとき

借りたものを，貸し主が片付けられたあとで使うと，コンパイルエラーになる．

```rust
pub struct Entry<'a> {
    pub name: &'a str,
}

pub fn parse(data: &str) -> Entry<'_> {
    Entry { name: &data[..3] }
}

pub fn first_name() -> usize {
    let entry;
    {
        let data = String::from("src main.rs");
        entry = parse(&data);
    }
    entry.name.len()
}
```

```text
error[E0597]: `data` does not live long enough
  --> src/lib.rs:13:23
   |
12 |         let data = String::from("src main.rs");
   |             ---- binding `data` declared here
13 |         entry = parse(&data);
   |                       ^^^^^ borrowed value does not live long enough
14 |     }
   |     - `data` dropped here while still borrowed
15 |     entry.name.len()
   |     ---------- borrow later used here
```

`data`は内側の`{ }`の終わりで片付けられるが，`entry`はその後も`data`を借りたまま使われる．
コンパイラーは，ライフタイム注釈から，`entry`が`data`を借りていることを知っている．

## 借用する設計と所有する設計

| | 借用する(`name: &'a str`) | 所有する(`name: String`) |
| --- | --- | --- |
| 解析の費用 | 名前を複製しない | 名前ごとに`String`を作る |
| 使える期間 | 元のバイト列が生きている間だけ | 自由．元のバイト列を捨ててよい |
| 型 | ライフタイム引数を持つ | ライフタイム引数を持たない |

`rgit`の`cat-file -p`と`ls-tree`は，treeを読んで表示したらすぐに捨てるので，借用する設計で足りる．
Iteration 6では，インデックスのパスを借りて，新しいtreeのエントリーを作る．
エントリーを長く持ち回る必要が出たら，所有する型に写す．

## RustOwlでライフタイムを見る

VS Codeで`cli.rs`の`cat-file`の処理を開き，`content`にカーソルを置くと，`content`が生きている範囲が下線で示される．
`write_tree_entries(&content, out)`で`content`が借用され，`parse_tree`が返すエントリーは，その借用の間だけ使える．
端末では，パッケージのディレクトリで次のように実行する．

```console
mise run rustowl -- tree::parse_tree name
```

`name`は，ループの中で`rest`の一部を借りて作られ，`entries.push`でエントリーに渡される．

## `TryFrom`と`TryInto`

`TryFrom<T>`は，`T`から失敗しうる変換で値を作るトレイトである．`type Error`に失敗の型を書く．

```rust
impl TryFrom<&[u8]> for Mode {
    type Error = Error;

    fn try_from(digits: &[u8]) -> Result<Mode, Error> {
        match digits {
            b"100644" => Ok(Mode::File),
            b"40000" => Ok(Mode::Directory),
            // …
            _ => Err(Error::CorruptObject("unknown mode")),
        }
    }
}
```

`match`のパターンには，バイト列のリテラル`b"100644"`も書ける．スライスの中身が一致するかを比べる．
`Mode::try_from(bytes)`で呼べる．`TryFrom`を実装すると，逆向きの`bytes.try_into()`も使えるようになる．

`FromStr`は文字列からの変換だけを表す．バイト列のように文字列以外から変換するときは`TryFrom`を使う．

## スライスから配列へ

`&[u8]`は長さが決まっていないが，`[u8; 20]`は長さが型の一部である．
スライスから配列への変換は，長さが合わなければ失敗するので，`try_into()`で行う．

```rust
let (id, after) = rest.split_at(20);
let id: [u8; 20] = id.try_into().unwrap();
```

`split_at(20)`の前半は必ず20バイトなので，`unwrap()`は失敗しない．
`split_at`は，長さが足りないとパニックになる．先に`rest.len() < 20`を調べてエラーを返す．

## `while`による繰り返し

`while 条件 { … }`は，条件が真の間くり返す．
`parse_tree`は，残りのバイト列`rest`が空になるまで，エントリーを1つずつ読む．

```rust
let mut rest = data;
while !rest.is_empty() {
    // restの先頭からエントリーを1つ読み，restを後ろへずらす
    rest = &rest[space + 1..];
}
```

`rest`は`&[u8]`の変数で，`mut`なので別のスライスを指し直せる．指し直しても，元のバイト列は変わらない．

`for i in 0..20`は，`i`を0から19まで変えてくり返す．`0..20`は，20を含まない範囲である．
`u8::from_str_radix("ce", 16)`は，16進数の文字列を`u8`にする．

## 関数を値として渡す

`ok_or_else(f)`は，`None`のときだけ`f()`を呼んでエラーを作る．
クロージャを変数に入れておけば，その変数を渡せる．

```rust
let corrupt = || Error::CorruptObject("invalid tree entry");
let space = rest
    .iter()
    .position(|byte| *byte == b' ')
    .ok_or_else(corrupt)?;
```

`ok_or(e)`はエラーの値を先に作る．`ok_or_else(f)`は必要になったときにだけ作る．
`b' '`は，空白の文字コードを`u8`で表すリテラルである．
