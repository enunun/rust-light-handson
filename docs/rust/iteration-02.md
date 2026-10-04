# Iteration 2：コマンドライン，ファイル，エラー型

Iteration 2では，`rgit init`と`rgit hash-object -w`を作る．
このノートでは，属性とderiveマクロ，clapによる引数の解析，パスとファイルの操作，`Write`トレイト，thiserrorによるエラー型と`?`，テストで外部のコマンドを実行する方法を説明する．

## 属性とderiveマクロ

`#[…]`を属性と呼ぶ．直後の項目(関数，型など)に，コンパイラーへの指示を付ける．
`#[test]`や`#[cfg(test)]`も属性である．

`#[derive(…)]`は，型の定義を読んで，トレイトの実装を生成する．
Iteration 1で使った`Debug`や`Clone`は標準のものだが，外部のクレートも独自のderiveを提供できる．
clapの`Parser`やthiserrorの`Error`がその例である．

## clapによる引数の解析

clapは，構造体と`enum`の定義から，コマンドラインの引数の解析器を作るクレートである．

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rgit", about = "A Git-compatible version control tool")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create an empty repository
    Init {
        /// Directory to create the repository in
        directory: Option<PathBuf>,
    },
    /// Compute the object ID of a file
    HashObject {
        /// Write the object into the object database
        #[arg(short = 'w')]
        write: bool,
        /// File to hash
        file: PathBuf,
    },
}
```

- `Subcommand`を導出した`enum`の列挙子が，サブコマンドになる．`HashObject`は`hash-object`という名前になる．
- 列挙子のフィールドが引数になる．`Option<T>`は省略できる引数，`bool`はフラグ，それ以外は必須の引数である．
- `#[arg(short = 'w')]`は，`-w`という短い名前のオプションにする．
- `///`で始まるコメント(ドキュメントコメント)は，`--help`の説明になる．

`Cli::try_parse_from(引数の列)`は，引数を解析して`Result<Cli, clap::Error>`を返す．引数の列の先頭は，プログラムの名前である．
解析できたら，`match`で列挙子ごとに処理を分ける．

```rust
match cli.command {
    Command::Init { directory } => { /* … */ }
    Command::HashObject { write, file } => { /* … */ }
}
```

`clap::Error`の`exit()`は，エラーや`--help`の表示を出力してプログラムを終える．終了コードは，`--help`なら0，引数の誤りなら2である．

`cargo add clap --features derive`のように，クレートの機能(feature)を選んで追加できる．deriveを使うには，`derive`の機能が必要である．

## パス

`Path`はファイルシステムのパスを指す参照の型，`PathBuf`はパスを所有する型である．
`&str`と`String`の関係と同じで，関数の引数には`&Path`を使う．

```rust
use std::path::{Path, PathBuf};

let dir = Path::new("/work");
let git_dir: PathBuf = dir.join(".git");
assert_eq!(git_dir, Path::new("/work/.git"));
assert_eq!(git_dir.parent(), Some(dir));
```

| メソッド | 動作 |
| --- | --- |
| `join(p)` | パスをつなげた`PathBuf`を返す．`p`が絶対パスなら`p`そのものになる |
| `parent()` | 親のディレクトリを`Option<&Path>`で返す |
| `ancestors()` | 自分，親，親の親，…の順にパスを返すイテレーター |
| `is_dir()`，`exists()` | ディレクトリか，存在するかを調べる |
| `to_path_buf()` | `&Path`から`PathBuf`を作る(複製する) |
| `display()` | `{}`で表示できる形にする |

```rust
let found: Vec<&Path> = Path::new("/a/b").ancestors().collect();
assert_eq!(found, vec![Path::new("/a/b"), Path::new("/a"), Path::new("/")]);
```

## ファイルの操作

`std::fs`の関数で，ファイルとディレクトリを操作する．どれも`io::Result<T>`(`Result<T, io::Error>`)を返す．

| 関数 | 動作 |
| --- | --- |
| `fs::read(path)` | ファイルの中身を`Vec<u8>`で読む |
| `fs::read_to_string(path)` | ファイルの中身を`String`で読む |
| `fs::write(path, data)` | ファイルを作って(あれば上書きして)書く |
| `fs::create_dir_all(path)` | ディレクトリを，途中のディレクトリも含めて作る．すでにあってもよい |
| `fs::canonicalize(path)` | `..`やシンボリックリンクを解決した絶対パスを返す |

## `Write`トレイト

`std::io::Write`は，バイト列を書き込める先を表すトレイトである．
ファイル，標準出力，`Vec<u8>`がこのトレイトを実装している．

- `write_all(&bytes)`：バイト列をすべて書く．
- `writeln!(out, "書式", …)`：`println!`と同じ書式で，`out`に1行を書く．`io::Result<()>`を返す．

引数の型に`impl Write`と書くと，`Write`を実装した任意の型を受け取れる．

```rust
use std::io::Write;

fn greet(out: &mut impl Write) -> std::io::Result<()> {
    writeln!(out, "hello")
}

let mut buffer = Vec::new();
greet(&mut buffer).unwrap();
assert_eq!(buffer, b"hello\n");
```

`main`では標準出力を，テストでは`Vec<u8>`を渡す．テストは出力を文字列として確かめられる．
`String::from_utf8(bytes)`は，`Vec<u8>`をUTF-8として`String`に変える．

flate2の`ZlibEncoder`は，書き込まれたバイト列を圧縮して，内側の`Write`(ここでは`Vec<u8>`)に書く．
`finish()`で圧縮を終え，内側の値を返す．

```rust
use flate2::Compression;
use flate2::write::ZlibEncoder;

let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
encoder.write_all(b"blob 6\0hello\n")?;
let compressed: Vec<u8> = encoder.finish()?;
```

## エラー型と`?`

### `?`演算子

`Result`を返す関数の中では，`式?`と書ける．
式が`Ok(値)`なら値を取り出し，`Err(e)`なら，その場で`Err(e)`を返して関数を終える．

```rust
fn read_size(path: &Path) -> Result<usize, std::io::Error> {
    let data = fs::read(path)?;
    Ok(data.len())
}
```

`?`は，エラーの型を関数の戻り値のエラーの型に変換してから返す．
変換には`From`トレイトを使う．`From<io::Error>`を実装したエラー型なら，`io::Error`を`?`でそのまま返せる．

### thiserror

thiserrorは，エラーの`enum`に`Display`，`std::error::Error`，`From`を実装するクレートである．

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("not a git repository (or any of the parent directories): .git")]
    NotARepository,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Usage(#[from] clap::Error),
}
```

- `#[error("…")]`は，その列挙子の`Display`の文字列である．
- `#[from]`を付けたフィールドの型から，その列挙子への`From`を実装する．`?`で`io::Error`が`Error::Io`に変わる．
- `#[error(transparent)]`は，包んだエラーの`Display`をそのまま使う．

`rgit`の関数は`Result<T, Error>`を返す．ファイルの操作でも引数の解析でも，`?`だけでエラーを呼び出し元へ伝えられる．

## `pub mod`

`pub mod cli;`と書くと，モジュール`cli`を公開する．利用者は`rgit::cli::run`と書ける．
`mod`だけのモジュールの中身は，`pub use`で公開したものしか外から使えない．

## `if let`と`matches!`

`if let パターン = 式 { … }`は，式がパターンに一致したときだけ実行する．
1つの形だけを扱いたいときに，`match`より短く書ける．

```rust
if let Err(error) = rgit::cli::run(&args, &cwd, &mut out) {
    eprintln!("fatal: {error}");
}
```

`matches!(式, パターン)`は，式がパターンに一致すれば`true`を返す．テストで列挙子を確かめるのに使う．

```rust
assert!(matches!(result, Err(Error::NotARepository)));
```

## イテレーターの基本

`std::env::args()`は，コマンドラインの引数を`String`で1つずつ返すイテレーターである．先頭はプログラムの名前である．

```rust
let args: Vec<String> = std::env::args().skip(1).collect();
let args: Vec<&str> = args.iter().map(String::as_str).collect();
```

- `skip(n)`：先頭のn個を飛ばす．
- `map(関数)`：各要素に関数を適用する．`String::as_str`は，`&String`から`&str`を作る関数である．
- `collect()`：要素を集めて`Vec`などにする．何に集めるかは，受け取る変数の型で決まる．

イテレーターとクロージャはIteration 5で詳しく扱う．

`vec!["rgit"]`は，要素を並べて`Vec`を作るマクロである．`extend_from_slice`は，スライスの要素を複製して後ろに足す．
`String`の`into_bytes()`は，`String`をムーブして，中身のバイト列を`Vec<u8>`で返す．

## プログラムの終了

- `eprintln!`は，標準エラー出力に1行を書く．
- `std::process::exit(code)`は，終了コードを指定してプログラムを終える．

## テストで外部のコマンドを実行する

`std::process::Command`は，外部のプログラムを実行する．

```rust
use std::process::Command;

let output = Command::new("git")
    .args(["cat-file", "-p", "ce01362"])
    .current_dir(dir)
    .env("GIT_CONFIG_NOSYSTEM", "1")
    .output()
    .unwrap();
assert!(output.status.success());
assert_eq!(output.stdout, b"hello\n");
```

`output()`は，プログラムの終了を待ち，終了の状態，標準出力，標準エラー出力を返す．

### 一時ディレクトリ

`tempfile`クレートの`TempDir::new()`は，一時ディレクトリを作る．
`TempDir`の値がスコープを抜けると，ディレクトリは中身ごと消える．テストのたびに新しいリポジトリを作れる．
テストでだけ使うクレートは，`cargo add --dev tempfile`で`[dev-dependencies]`に加える．

### 結合テストの補助関数

`tests/`の下のファイルは，それぞれが別のクレートとしてコンパイルされる．
複数のファイルで使う関数は`tests/common/mod.rs`に置き，各ファイルの先頭で`mod common;`と書く．
`tests/common.rs`とすると，それ自体が1つのテストのファイルとして扱われる．
