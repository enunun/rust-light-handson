# Iteration 3：`Read`，スライス，借用を返す関数

Iteration 3では，オブジェクトを読んでヘッダーを解析する`cat-file`を作る．
このノートでは，`Read`トレイト，スライスの操作，引数の借用を返す関数とライフタイムの省略，`Option`と`Result`の変換，クロージャの書き方，ディレクトリの読み取りを説明する．

## `Read`トレイト

`std::io::Read`は，バイト列を読み出せる元を表すトレイトである．ファイル，標準入力，`&[u8]`がこのトレイトを実装している．
`read_to_end(&mut vec)`は，終わりまで読んで`vec`の後ろに足す．

flate2の`read::ZlibDecoder`は，内側の`Read`(ここではファイル)から圧縮されたバイト列を読み，展開したバイト列を返す`Read`である．

```rust
use std::io::Read;
use flate2::read::ZlibDecoder;

let file = fs::File::open(path)?;
let mut data = Vec::new();
ZlibDecoder::new(file).read_to_end(&mut data)?;
```

Iteration 2の`write::ZlibEncoder`は書き込む側で圧縮した．`read::ZlibDecoder`は読み出す側で展開する．
どちらも，`Read`や`Write`を実装した値を包んで，同じトレイトを実装した値を作る．
包む値の型を問わないので，ファイルの代わりに`&[u8]`や`Vec<u8>`を包んでもよい．

## ファイルを開くときのエラーの種類

`fs::File::open`は，失敗すると`io::Error`を返す．`error.kind()`で失敗の種類(`io::ErrorKind`)を調べられる．
ファイルがないことだけを特別に扱うには，`match`の腕にガード(`if 条件`)を付ける．

```rust
let file = match fs::File::open(path) {
    Ok(file) => file,
    Err(error) if error.kind() == ErrorKind::NotFound => {
        return Err(Error::ObjectNotFound(id.to_string()));
    }
    Err(error) => return Err(error.into()),
};
```

ガードが偽なら，次の腕に進む．`error.into()`は，`From`の実装を使って`io::Error`を`Error`に変える．

## スライスの操作

`&[u8]`のスライスには，次のようなメソッドがある．

| 書き方 | 結果 |
| --- | --- |
| `data.iter().position(\|byte\| *byte == 0)` | 最初に条件を満たす要素の位置を`Option<usize>`で返す |
| `data.split_at(n)` | 位置`n`の前と後ろの2つのスライスを組で返す |
| `&data[n..]` | 位置`n`から後ろのスライス |
| `data.to_vec()` | 中身を複製した`Vec<u8>` |

```rust
let (head, rest) = b"blob 6\0hello\n".split_at(6);
assert_eq!(head, b"blob 6");
assert_eq!(rest, b"\0hello\n");
```

`split_at`と`&data[n..]`は，中身を複製しない．元のバイト列の一部を指すスライスを作るだけである．

`std::str::from_utf8(bytes)`は，バイト列がUTF-8として正しければ`Ok(&str)`を返す．これも複製しない．

## クロージャ

`|byte| *byte == 0`は，引数`byte`を受け取って`*byte == 0`を返す，名前のない関数である．これをクロージャと呼ぶ．
クロージャは，`position`や`map_err`のような，関数を受け取るメソッドに渡す．
クロージャは，周りの変数を使える．

```rust
let not_found = || Error::ObjectNotFound(prefix.to_string());
return Err(not_found());
```

`||`は引数のないクロージャである．同じエラーを何か所かで作るときに，1か所にまとめられる．
クロージャはIteration 5で詳しく扱う．

## 引数の借用を返す関数

`parse_header`は，オブジェクトのバイト列を受け取り，内容の部分をスライスで返す．

```rust
pub fn parse_header(data: &[u8]) -> Result<(ObjectKind, &[u8]), Error>
```

戻り値の`&[u8]`は，引数`data`の一部を借りている．内容を`Vec<u8>`に複製しないので，大きなファイルでも速い．
借りたものは，貸し主より長く使えない．戻り値の内容は，`data`が生きている間だけ使える．

関数の中で作った値を借りて返すことはできない．関数を抜けると，その値は片付けられるからである．

```rust
pub fn content(data: &[u8]) -> &[u8] {
    let copy = data.to_vec();
    &copy[1..]
}
```

```text
error[E0515]: cannot return value referencing local variable `copy`
 --> src/lib.rs:3:5
  |
3 |     &copy[1..]
  |     ^----^^^^^
  |     ||
  |     |`copy` is borrowed here
  |     returns a value referencing data owned by the current function
```

### ライフタイムの省略

参照が有効な期間をライフタイムと呼ぶ．戻り値の参照がどの引数から借りたものかは，本来`'a`のような名前を付けて示す．名前を書く形はIteration 4で使う．

```rust
pub fn parse_header<'a>(data: &'a [u8]) -> Result<(ObjectKind, &'a [u8]), Error>
```

参照の引数が1つだけなら，戻り値の参照はその引数から借りたものだとコンパイラーが決める．この規則をライフタイムの省略と呼ぶ．
`parse_header`の引数の参照は`data`だけなので，`'a`を書かなくてよい．
メソッドの`&self`も同じで，`&self`を持つメソッドの戻り値の参照は，`self`から借りたものになる(Iteration 2の`git_dir()`)．

呼び出す側は，借りた内容を手放す前に複製する．`read_object`は，展開したバイト列`data`を関数の中で作るので，`content.to_vec()`で複製して返す．

## `Option`と`Result`の変換

`?`は`Result`だけでなく`Option`にも使えるが，1つの関数の中で混ぜることはできない．
`Result`を返す関数の中では，`Option`を`Result`に変えてから`?`を使う．

| メソッド | 動作 |
| --- | --- |
| `option.ok_or(e)` | `Some(v)`を`Ok(v)`に，`None`を`Err(e)`にする |
| `result.map_err(f)` | `Err(e)`を`Err(f(e))`にする．エラーの型を変える |
| `result.ok()` | `Ok(v)`を`Some(v)`に，`Err`を`None`にする |
| `option.and_then(f)` | `Some(v)`なら`f(v)`(これも`Option`)を返す |

```rust
let nul = data
    .iter()
    .position(|byte| *byte == 0)
    .ok_or(Error::CorruptObject("missing header"))?;
let size: usize = size
    .parse()
    .map_err(|_| Error::CorruptObject("invalid size"))?;
```

`|_|`は，引数を受け取るが使わないクロージャである．`parse`のエラー(`ParseIntError`)を捨てて，`rgit`のエラーにする．

## `&'static str`

`"missing header"`のような文字列リテラルの型は`&'static str`である．`'static`は，プログラムの終わりまで有効なことを表す．
エラーの理由を決まった文字列から選ぶなら，`String`ではなく`&'static str`で持てる．

## ディレクトリを読む

`fs::read_dir(dir)`は，ディレクトリの中のエントリーを1つずつ返すイテレーターである．
読み取りは失敗することがあるので，各要素は`io::Result<DirEntry>`である．

```rust
for entry in fs::read_dir(dir)? {
    let name = entry?.file_name();
    if let Some(name) = name.to_str()
        && name.starts_with(rest)
    {
        found.push(format!("{dir_name}{name}"));
    }
}
```

- `file_name()`は，ファイル名を`OsString`で返す．OSのファイル名はUTF-8とは限らないので，`String`とは別の型になっている．`to_str()`は，UTF-8なら`Some(&str)`を返す．
- `if let … && 条件`は，パターンが一致し，かつ条件が真のときだけ実行する(let chain)．

## 文字列の検査

- `s.chars().all(|ch| ch.is_ascii_hexdigit())`：すべての文字が16進数の数字なら`true`
- `s.to_ascii_lowercase()`：英字を小文字にした`String`
- `s.starts_with(prefix)`：`prefix`で始まれば`true`

## clapのフラグのグループ

`-t`，`-s`，`-p`のどれか1つだけを受け付けるには，フラグを1つの構造体にまとめ，`#[group]`を付ける．

```rust
#[derive(Args)]
#[group(required = true, multiple = false)]
struct CatFileMode {
    #[arg(short = 't')]
    kind: bool,
    #[arg(short = 's')]
    size: bool,
    #[arg(short = 'p')]
    pretty: bool,
}
```

サブコマンドの列挙子では，`#[command(flatten)] mode: CatFileMode`のように，構造体のフィールドを引数として展開する．
`required = true`はどれか1つを必須にし，`multiple = false`は2つ以上の指定をエラーにする．
