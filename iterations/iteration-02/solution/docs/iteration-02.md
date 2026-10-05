# Iteration 2：`init`と`hash-object -w`(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 2-1 準備

依存を加えたあとの模範解答の`Cargo.toml`である．

```toml
[package]
name = "rgit-02-solution"
version = "0.1.0"
edition = "2024"

[lib]
name = "rgit"

[dependencies]
clap = { version = "4.6.7", features = ["derive"] }
flate2 = "1.1.10"
sha1 = "0.11.0"
thiserror = "2.0.21"

[dev-dependencies]
criterion = "0.8.2"
tempfile = "3.27.0"

[[bench]]
name = "object"
harness = false
```

`[[bench]]`は，2-5の最後で速度を測るときに書き加える．

## 2-2 文法と概念

課題の解答例である．

```rust
#[cfg(test)]
mod practice {
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    use tempfile::TempDir;

    #[test]
    fn ancestors_go_up_to_root() {
        let found: Vec<&Path> = Path::new("/a/b/c").ancestors().collect();
        assert_eq!(
            found,
            vec![
                Path::new("/a/b/c"),
                Path::new("/a/b"),
                Path::new("/a"),
                Path::new("/")
            ]
        );
    }

    #[test]
    fn writes_and_reads_back_a_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("note.txt");
        fs::write(&path, "hi\n").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hi\n");
    }

    fn greet(out: &mut impl Write) -> std::io::Result<()> {
        writeln!(out, "hello")
    }

    #[test]
    fn greets_into_a_vec() {
        let mut out = Vec::new();
        greet(&mut out).unwrap();
        assert_eq!(out, b"hello\n");
    }

    #[derive(Debug, thiserror::Error)]
    enum MyError {
        #[error("cannot read: {0}")]
        Io(#[from] std::io::Error),
    }

    fn read_file(path: &Path) -> Result<Vec<u8>, MyError> {
        let data = fs::read(path)?;
        Ok(data)
    }

    #[test]
    fn io_error_becomes_my_error() {
        let error = read_file(Path::new("/no/such/file")).unwrap_err();
        assert!(matches!(error, MyError::Io(_)));
        assert_eq!(
            error.to_string(),
            "cannot read: No such file or directory (os error 2)"
        );
    }
}
```

- 2：`fs::write(&path, …)`の`&`を外すと，`path`がムーブされて次の行で使えなくなる．
- 4：`#[error("cannot read: {0}")]`の`{0}`は，列挙子の1つ目のフィールド(`io::Error`)の表示である．`?`は，`fs::read`の`io::Error`を，`#[from]`が作った`From`で`MyError::Io`に変える．

## 2-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

- 単体テストは，`blob_bytes`と，`Repository`の4つの振る舞いを確かめる．4つとは，リポジトリを作ること，親から見つけること，見つからないこと，オブジェクトを書き込むことである．
- 結合テストは，コマンドの出力と，本物の`git`との互換を確かめる．`git rev-parse --git-dir`は，`HEAD`を書き忘れると失敗する．`git cat-file`は，オブジェクトの置き場所，圧縮，ヘッダーのどれかを誤ると失敗する．
- `-w`なしで書き込まないことも項目にした．`-w`の判定を誤ると，`-w`なしでもオブジェクトが増える．
- `main.rs`の終了コードは，テストにせず，2-5の手順で動かして確かめた．

## 2-4 図

模範解答は[design/types.md](../design/types.md)にある．Iteration 1からの変更は次のとおりである．

- `cli`，`repo`，`error`の名前空間を加えた．
- `Cli`は`Command`をフィールドに持つので，所有の矢印でつないだ．
- `cli_mod`から，使う`Cli`，`Repository`，`Error`，`object_mod`(`hash_blob`)への依存を描いた．
- `Repository`から，`object_mod`(`blob_bytes`と`hash_blob`)，戻り値の`ObjectId`と`Error`への依存を描いた．
- `run`の戻り値`Result<(), Error>`は，Mermaidで`()`を表示できないので，`Result`とだけ書き，正確な型を図の下に書いた．
- 外部のクレートの型(`io::Error`，`clap::Error`，`PathBuf`)は，クラスにせず，フィールドや列挙子の型として書いた．

## 2-5 テスト駆動の実装

### `blob_bytes`

```rust
#[test]
fn blob_bytes_start_with_kind_and_size() {
    assert_eq!(blob_bytes(b"hello\n"), b"blob 6\0hello\n");
}
```

```rust
/// データをblobオブジェクトにしたバイト列(ヘッダーと内容)を返す．
pub fn blob_bytes(data: &[u8]) -> Vec<u8> {
    let mut bytes = format!("blob {}\0", data.len()).into_bytes();
    bytes.extend_from_slice(data);
    bytes
}

/// データをblobオブジェクトにしたときのIDを返す．
pub fn hash_blob(data: &[u8]) -> ObjectId {
    let digest = Sha1::digest(blob_bytes(data));
    ObjectId::from_bytes(digest.into())
}
```

テストが通ったあと，`hash_blob`を`blob_bytes`を使う形に書き換えた(Refactor)．`hash_blob`のテストは通ったままである．
`write_blob`は，同じ`blob_bytes`を圧縮して書く．ハッシュを計算するバイト列と書き込むバイト列が，同じ関数から作られる．

### `Repository::init`

`src/lib.rs`に`mod error;`と`mod repo;`を加える．`Error`は，まず`Io`だけを持つ形で作った．

```rust
#[test]
fn init_creates_git_directories_and_head() {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    assert_eq!(repo.git_dir(), dir.path().join(".git"));
    assert!(repo.git_dir().join("objects").is_dir());
    assert!(repo.git_dir().join("refs/heads").is_dir());
    assert_eq!(
        fs::read_to_string(repo.git_dir().join("HEAD")).unwrap(),
        "ref: refs/heads/main\n"
    );
}
```

```rust
/// Gitのリポジトリ．`.git`ディレクトリの場所を持つ．
pub struct Repository {
    git_dir: PathBuf,
}

impl Repository {
    /// `dir`に空のリポジトリを作る．
    pub fn init(dir: &Path) -> Result<Repository, Error> {
        let git_dir = dir.join(".git");
        fs::create_dir_all(git_dir.join("objects"))?;
        fs::create_dir_all(git_dir.join("refs").join("heads"))?;
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;
        Ok(Repository { git_dir })
    }

    pub fn git_dir(&self) -> &Path {
        &self.git_dir
    }
}
```

`git_dir`は`&self.git_dir`を返す．`PathBuf`は`&Path`として借りられるので，戻り値の型は`&Path`にした．
`Repository { git_dir }`は，`Repository { git_dir: git_dir }`の省略形である．

### `Repository::discover`

```rust
#[test]
fn discover_finds_repository_in_parent_directory() {
    let dir = TempDir::new().unwrap();
    Repository::init(dir.path()).unwrap();
    let sub = dir.path().join("a/b");
    fs::create_dir_all(&sub).unwrap();
    let repo = Repository::discover(&sub).unwrap();
    assert_eq!(repo.git_dir(), dir.path().join(".git"));
}

#[test]
fn discover_fails_outside_repository() {
    let dir = TempDir::new().unwrap();
    let result = Repository::discover(dir.path());
    assert!(matches!(result, Err(Error::NotARepository)));
}
```

```rust
/// `start`から親へ順に`.git`を探し，見つけたリポジトリを開く．
pub fn discover(start: &Path) -> Result<Repository, Error> {
    for dir in start.ancestors() {
        let git_dir = dir.join(".git");
        if git_dir.is_dir() {
            return Ok(Repository { git_dir });
        }
    }
    Err(Error::NotARepository)
}
```

`Error`に`NotARepository`を加え，メッセージを本物の`git`に合わせた．

2つ目のテストは，`unwrap_err()`でエラーを取り出すと，コンパイルエラーになる．
`unwrap_err`は，成功の値(`Repository`)を表示する可能性があるので，`Repository`に`Debug`を求める．

```text
error[E0277]: `repo::Repository` doesn't implement `Debug`
  --> src/repo.rs:99:54
   |
99 |         let error = Repository::discover(dir.path()).unwrap_err();
   |                                                      ^^^^^^^^^^ the trait `Debug` is not implemented for `repo::Repository`
```

`matches!`で`Result`全体を調べれば，`Debug`は要らない．

### `Repository::write_blob`

```rust
#[test]
fn write_blob_stores_compressed_object_under_its_id() {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let id = repo.write_blob(b"hello\n").unwrap();
    let path = repo
        .git_dir()
        .join("objects/ce/013625030ba8dba906f756967f9e9ca394464a");
    let mut content = Vec::new();
    ZlibDecoder::new(fs::File::open(path).unwrap())
        .read_to_end(&mut content)
        .unwrap();
    assert_eq!(id.to_string(), "ce013625030ba8dba906f756967f9e9ca394464a");
    assert_eq!(content, b"blob 6\0hello\n");
}
```

```rust
/// データをblobオブジェクトとして書き込み，そのIDを返す．
pub fn write_blob(&self, data: &[u8]) -> Result<ObjectId, Error> {
    let id = hash_blob(data);
    let path = self.object_path(id);
    if path.exists() {
        return Ok(id);
    }
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&blob_bytes(data))?;
    let compressed = encoder.finish()?;
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, compressed)?;
    Ok(id)
}

/// オブジェクトのファイルのパス(`.git/objects/ce/013625…`)を返す．
fn object_path(&self, id: ObjectId) -> PathBuf {
    let hex = id.to_string();
    self.git_dir
        .join("objects")
        .join(&hex[..2])
        .join(&hex[2..])
}
```

- 同じIDのファイルがあれば，書かずに返す．中身はIDで決まるので，書き直す必要はない．
- `path.parent()`は`Option`を返す．`objects/ce/…`には必ず親があるので，`unwrap()`で取り出した．
- `object_path`は`ObjectId`を値で受け取る．`ObjectId`は`Copy`なので，呼び出したあとも`id`を使える．

### `cli::run`と`init`

`src/lib.rs`に`pub mod cli;`を加え，`Error`を公開する．

```rust
pub mod cli;
mod error;
mod object;
mod oid;
mod repo;

pub use error::Error;
pub use object::hash_blob;
pub use oid::{ObjectId, ParseObjectIdError};
```

`tests/common/mod.rs`に，`rgit`と本物の`git`を実行する補助関数を置く．

```rust
use std::path::Path;
use std::process::Command;

/// `rgit`を`dir`で実行し，標準出力に書いた文字列を返す．
pub fn rgit(dir: &Path, args: &[&str]) -> String {
    let mut out = Vec::new();
    rgit::cli::run(args, dir, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// 本物の`git`を`dir`で実行し，標準出力を返す．利用者の設定は読まない．
pub fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
```

`git`の補助関数は，失敗したら標準エラー出力を含めてテストを失敗させる．`assert!`の2つ目以降の引数は，失敗したときに表示するメッセージである．

`tests/init.rs`の最初のテストである．

```rust
#[test]
fn init_prints_absolute_path_of_git_directory() {
    let dir = TempDir::new().unwrap();
    let git_dir = fs::canonicalize(dir.path()).unwrap().join(".git");
    assert_eq!(
        rgit(dir.path(), &["init"]),
        format!("Initialized empty Git repository in {}/\n", git_dir.display())
    );
}
```

`cli.rs`に，clapの型と`run`を書く．

```rust
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
}

/// コマンドラインの引数(プログラム名を除く)を解釈して実行し，結果を`out`に書く．
pub fn run(args: &[&str], cwd: &Path, out: &mut impl Write) -> Result<(), Error> {
    let mut argv = vec!["rgit"];
    argv.extend_from_slice(args);
    let cli = Cli::try_parse_from(argv)?;
    match cli.command {
        Command::Init { directory } => {
            let dir = match directory {
                Some(directory) => cwd.join(directory),
                None => cwd.to_path_buf(),
            };
            let repo = Repository::init(&dir)?;
            let git_dir = fs::canonicalize(repo.git_dir())?;
            writeln!(out, "Initialized empty Git repository in {}/", git_dir.display())?;
        }
    }
    Ok(())
}
```

`Error`に`Usage(#[from] clap::Error)`を加えると，`try_parse_from`の結果に`?`を使える．
`init project`と，本物の`git`がリポジトリを認めるテストは，実装を変えずに通る．

### `hash-object`

`tests/hash_object.rs`に，`-w`なしの項目から順に書く．`Command`に列挙子`HashObject`を加え，`match`に腕を足す．

```rust
Command::HashObject { write, file } => {
    let data = fs::read(cwd.join(file))?;
    let id = if write {
        Repository::discover(cwd)?.write_blob(&data)?
    } else {
        hash_blob(&data)
    };
    writeln!(out, "{id}")?;
}
```

`if`は値を返す式なので，`let id = if … { … } else { … };`と書ける．
`Repository::discover(cwd)?.write_blob(&data)?`は，見つけたリポジトリに続けて書き込む．どちらかが失敗すれば，`?`で`run`から返る．

リポジトリの外のテストは，`run`が返すエラーのメッセージを確かめる．

```rust
#[test]
fn writing_outside_repository_is_an_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    let mut out = Vec::new();
    let error = rgit::cli::run(&["hash-object", "-w", "hello.txt"], dir.path(), &mut out)
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "not a git repository (or any of the parent directories): .git"
    );
}
```

`run`の成功の値は`()`で，`Debug`を実装しているので，`unwrap_err`を使える．

### `main.rs`

```rust
use rgit::Error;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let cwd = std::env::current_dir().unwrap();
    let mut out = std::io::stdout();
    if let Err(error) = rgit::cli::run(&args, &cwd, &mut out) {
        match error {
            Error::Usage(error) => error.exit(),
            error => {
                eprintln!("fatal: {error}");
                std::process::exit(128);
            }
        }
    }
}
```

`match`の2つ目の腕のパターン`error`は，`Error::Usage`以外のすべての値と一致する．腕の中では，その値を`error`という名前で使える．
`Error::Usage`の`exit()`は，`--help`なら終了コード0で，引数の誤りなら2で終わる．

### 速度を測る

`benches/object.rs`である．
圧縮は，`write_blob`と同じくflate2の`ZlibEncoder`で行い，強さを引数で選べるようにした．

```rust
use std::hint::black_box;
use std::io::Write;

use criterion::{Criterion, criterion_group, criterion_main};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use rgit::hash_blob;

/// 1MiBのテキスト(数を1行ずつ並べたもの)．
fn sample() -> Vec<u8> {
    (0..)
        .flat_map(|i| format!("{i}\n").into_bytes())
        .take(1 << 20)
        .collect()
}

/// zlibで圧縮する．`Repository::write_blob`は`Compression::default()`を使う．
fn compress(data: &[u8], level: Compression) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), level);
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

fn object(c: &mut Criterion) {
    let data = sample();
    c.bench_function("hash_blob", |b| b.iter(|| hash_blob(black_box(&data))));
    c.bench_function("compress_default", |b| {
        b.iter(|| compress(black_box(&data), Compression::default()))
    });
    c.bench_function("compress_fast", |b| {
        b.iter(|| compress(black_box(&data), Compression::fast()))
    });
}

criterion_group!(benches, object);
criterion_main!(benches);
```

- データは，数を1行ずつ並べたテキストを`take(1 << 20)`で1MiBにしたものである．同じ文字の繰り返しのようなデータは，圧縮がとても速く終わり，実際のファイルに近くない．
- データはベンチマークの前に1回だけ作り，`b.iter`の中では測りたい処理だけを呼ぶ．

最適化したビルドの結果である(値は計算機によって変わる)．

```console
$ cargo bench --bench object
(略)
    Finished `bench` profile [optimized] target(s) in 1.71s
     Running benches/object.rs (target/release/deps/object-3bdc14af7c08dec4)
Gnuplot not found, using plotters backend
hash_blob               time:   [759.61 µs 765.73 µs 773.42 µs]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

compress_default        time:   [41.189 ms 41.614 ms 42.077 ms]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

compress_fast           time:   [4.3219 ms 4.4170 ms 4.5207 ms]
Found 11 outliers among 100 measurements (11.00%)
  9 (9.00%) high mild
  2 (2.00%) high severe
```

最適化しないビルドの結果である．`change`は，直前の最適化したビルドとの差である．

```console
$ cargo bench --profile dev --bench object
(略)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.54s
     Running benches/object.rs (target/debug/deps/object-051526f38eb29513)
Gnuplot not found, using plotters backend
hash_blob               time:   [9.8023 ms 9.9718 ms 10.173 ms]
                        change: [+1149.0% +1175.8% +1206.3%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 15 outliers among 100 measurements (15.00%)
  7 (7.00%) high mild
  8 (8.00%) high severe

Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 27.4s, or reduce sample count to 10.
compress_default        time:   [281.76 ms 286.35 ms 291.30 ms]
                        change: [+575.09% +588.12% +602.15%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild

Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.4s, or reduce sample count to 60.
compress_fast           time:   [72.317 ms 73.393 ms 74.582 ms]
                        change: [+1515.8% +1561.6% +1608.0%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
```

| 処理 | 最適化あり | 最適化なし |
| --- | --- | --- |
| `hash_blob` | 0.77ms | 10.0ms |
| 圧縮(`Compression::default()`) | 41.6ms | 286ms |
| 圧縮(`Compression::fast()`) | 4.4ms | 73.4ms |

- 最適化したビルドでは，強さ6の圧縮はSHA-1の計算の約50倍の時間がかかる．`hash-object -w`の時間の多くは圧縮である．
- 強さ1の圧縮は，強さ6の約9分の1の時間で終わる．
- 最適化しないビルドは，処理によって7倍から17倍遅い．
- 最適化しないビルドの圧縮は，既定の5秒では100回を測りきれない．criterionは`Warning`を出し，時間を延ばして100回を測る．

## 2-6 振り返り

1. 模範解答の結合テストは，`git`で確かめられることは`git`で確かめている．`rgit`の中だけで確かめると，誤った形式で書いて誤った形式で読むテストが通ってしまう．
2. `out`に書く設計では，テストは`Vec<u8>`を渡して出力を文字列として比べられる．標準出力に直接書くと，テストはプログラムを別のプロセスとして起動し，その出力を読む必要がある．
3. `Error`が`From<io::Error>`と`From<clap::Error>`を持つので，`?`はどちらのエラーも`Error`に変えて返せる．呼び出す側(`main.rs`)は列挙子で`match`し，引数の誤りだけをclapに表示させられる．`Box<dyn std::error::Error>`では，どの種類のエラーかを型で区別できない．
4. 本物の`git`を使うテストは，`rgit`の出力が本物のGitの形式と一致することを保証する．`rgit`の中だけのテストは，`rgit`の関数が設計どおりに動くことを保証し，`git`がなくても動く．
5. 時間の多くは圧縮に使われる．ゆるいオブジェクトは`add`や`commit`のたびに書くので，`git`は少し大きくなっても速い強さ1を選ぶ．小ささが大事なパックファイルは，`git gc`でまとめるときに強く圧縮し直す．
6. `cargo test`は最適化しないビルドで動き，テストの時間の多くはコンパイルと`git`の起動である．速さは，最適化したビルドで，測りたい処理だけを繰り返して測る．
7. 図に描いた関係は，コードと一致している．

## 2-7 発展課題

`cli.rs`で，`Repository::init`を呼ぶ前に`.git`があるかを調べ，メッセージを変える．

```rust
let existed = dir.join(".git").is_dir();
let repo = Repository::init(&dir)?;
let git_dir = fs::canonicalize(repo.git_dir())?;
let action = if existed {
    "Reinitialized existing"
} else {
    "Initialized empty"
};
writeln!(out, "{action} Git repository in {}/", git_dir.display())?;
```

`Repository::init`は，`HEAD`がなければ書く．

```rust
let head = git_dir.join("HEAD");
if !head.exists() {
    fs::write(head, "ref: refs/heads/main\n")?;
}
```

テストでは，1回目の`init`のあとで`HEAD`を書き換え，2回目の`init`のあとも残っていることを確かめる．
