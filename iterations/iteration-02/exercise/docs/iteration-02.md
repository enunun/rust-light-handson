# Iteration 2：`init`と`hash-object -w`

このIterationでは，`rgit`をコマンドラインのプログラムにし，`rgit init`と`rgit hash-object -w`を作る．
本物の`git`が，`rgit`の作ったリポジトリとオブジェクトを読めることをテストで確かめる．

## 2-1 準備

### 引き継いだテスト

このディレクトリ(`iterations/iteration-02/exercise`)に移動し，`cargo test`で，Iteration 1から引き継いだテストがすべて通ることを確かめる．

### 依存を追加する

引数の解析にclapを，zlibの圧縮にflate2を，エラー型にthiserrorを使う．
clapは，deriveで解析器を作る機能(`derive`)を選んで追加する．

```console
$ cargo add clap --features derive
    Updating crates.io index
      Adding clap v4.6.7 to dependencies
             Features:
             + color
             + derive
(略)
$ cargo add flate2 thiserror
    Updating crates.io index
      Adding flate2 v1.1.10 to dependencies
(略)
      Adding thiserror v2.0.21 to dependencies
(略)
```

テストで一時ディレクトリを作る`tempfile`は，テストでだけ使うので`--dev`を付けて追加する．

```console
$ cargo add --dev tempfile
    Updating crates.io index
      Adding tempfile v3.27.0 to dev-dependencies
(略)
```

`Cargo.toml`に`[dependencies]`の3つと，`[dev-dependencies]`の`tempfile`が加わったことを確かめる．

## 2-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-02.md)：属性とderive，clap，パス，ファイルの操作，`Write`トレイト，`?`とthiserror，`if let`と`matches!`，外部のコマンドを実行するテスト
- [Gitのノート](../../../../docs/git/iteration-02.md)：`.git`の構成，ゆるいオブジェクト，zlib

読み終えたら，`src/lib.rs`の末尾に`#[cfg(test)] mod practice`を作り，次の課題を確かめる．

1. `Path::new("/a/b/c").ancestors()`が返すパスを`Vec`に集めて確かめる．
2. `TempDir`の中に`fs::write`でファイルを書き，`fs::read_to_string`で読み戻す．
3. `fn greet(out: &mut impl Write) -> std::io::Result<()>`を書き，`Vec<u8>`に書いた結果を確かめる．
4. 存在しないファイルを`fs::read`で読む関数を，戻り値を`Result<Vec<u8>, MyError>`にして書く．`MyError`はthiserrorで作り，`#[from] std::io::Error`を持つ列挙子を1つ持つ．`?`で`io::Error`が`MyError`に変わることと，`to_string()`の結果を確かめる．

確かめ終えたら，`mod practice`を消す．

## 2-3 テストリスト

### 要件

- `rgit init [<dir>]`は，`<dir>`(省略時はカレントディレクトリ)に`.git`，`.git/objects`，`.git/refs/heads`を作る．
  - `.git/HEAD`に`ref: refs/heads/main`と書く．本物の`git`は，`HEAD`のないディレクトリをリポジトリと認めない．
  - 作り終えたら`Initialized empty Git repository in <.gitの絶対パス>/`を出力する．
- `rgit hash-object [-w] <file>`は，ファイルのblobとしてのIDを出力する．
  - `-w`があれば，オブジェクトをリポジトリに書き込む．書き込み先は`.git/objects/<IDの先頭2桁>/<残りの38桁>`で，内容はヘッダーと内容をzlibで圧縮したものである．
  - `-w`のときは，カレントディレクトリから親へ順に`.git`を探してリポジトリを見つける．見つからなければ`not a git repository (or any of the parent directories): .git`のエラーにする．`-w`がなければ，リポジトリの外でも動く．
- エラーは標準エラー出力に`fatal: <メッセージ>`と出力し，終了コード128で終わる．引数の誤りはclapのメッセージを出力し，終了コード2で終わる．
- 本物の`git`は，`rgit`が書いたオブジェクトを読める．

### 使用例

```console
$ rgit init
Initialized empty Git repository in /tmp/demo/.git/
$ printf 'hello\n' > hello.txt
$ rgit hash-object -w hello.txt
ce013625030ba8dba906f756967f9e9ca394464a
$ git cat-file -p ce013625030ba8dba906f756967f9e9ca394464a
hello
```

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `cli` | clapのderiveによる`struct Cli`と`enum Command`，`pub fn run(args: &[&str], cwd: &Path, out: &mut impl Write) -> Result<(), Error>` |
| `repo` | `.git`の場所を持つ`pub struct Repository`，`Repository::init`，`Repository::discover`，`Repository::write_blob` |
| `error` | thiserrorによる`pub enum Error` |
| `object` | ヘッダーと内容を連結したバイト列を作る`blob_bytes`を加え，`hash_blob`はそのハッシュを計算する(リファクタリング) |
| `main.rs` | `cli::run`を呼び，エラーを出力する |
| ルート(`lib.rs`) | `cli`を公開モジュールにし，`Error`を公開する |

### 書くときに考えること

- `run`は，引数，カレントディレクトリ，出力先を引数に取る．結合テストは`TempDir`をカレントディレクトリとして`run`を呼び，`Vec<u8>`に書かれた出力を確かめられる．
- 本物の`git`で確かめられる振る舞いはどれか．`git cat-file`と`git rev-parse --git-dir`が使える．
- 単体テストで確かめるもの(`Repository`の各関数)と，結合テストで確かめるもの(コマンドの出力と`git`との互換)を分ける．
- `main.rs`の終了コードは，テストで確かめにくい．手で動かして確かめる．

## 2-4 図

`design/types.md`を，このIterationの終わりの状態に更新する．

- 新しいモジュール`cli`，`repo`，`error`の名前空間を加える．`cli`の中の`Cli`と`Command`は，どんな関係か．
- `Repository`のフィールドとメソッド，`Error`の列挙子を書く．
- `cli`はどのモジュールの何を使うか．`Repository`はどのモジュールの何を使うか．

## 2-5 テスト駆動の実装

### 実装のヒント

- `blob_bytes`は，`format!`で作ったヘッダーを`into_bytes()`で`Vec<u8>`にし，`extend_from_slice`で内容を足す．`hash_blob`は`Sha1::digest(blob_bytes(data))`で書ける．
- `Repository`の単体テストでは，`TempDir::new().unwrap()`でディレクトリを作り，`dir.path()`を渡す．
- `write_blob`のテストで圧縮したファイルを読むには，flate2の`read::ZlibDecoder`と`Read`の`read_to_end`を使う．
- `object_path`のような補助のメソッドは，`impl`の中に`pub`を付けずに書く．`&hex[..2]`と`&hex[2..]`で，先頭2桁と残りに分けられる．
- `cli::run`では，`vec!["rgit"]`に`args`を足してから`Cli::try_parse_from`に渡す．`?`で`clap::Error`が`Error::Usage`に変わる．
- `hash-object`のファイルのパスは，`cwd.join(file)`でカレントディレクトリからのパスにする．
- `init`の出力の絶対パスは，`fs::canonicalize`で求める．テストの期待値も，`TempDir`のパスを`fs::canonicalize`したものから作る．
- 結合テストの補助関数(`rgit`を実行して出力を返す関数，本物の`git`を実行する関数)を`tests/common/mod.rs`に置く．本物の`git`には，環境変数`GIT_CONFIG_NOSYSTEM=1`と`GIT_CONFIG_GLOBAL=/dev/null`を渡し，利用者の設定を読ませない．

### ツールの操作

- 1つの結合テストのファイルだけを実行するには，`cargo test --test hash_object`のようにファイル名を指定する．
- プログラムに引数を渡して実行するには，`cargo run -- init`のように`--`の後ろに引数を書く．`--`の前は`cargo`の引数である．
- `exercise/`の中で`rgit init`を実行すると，`exercise/`の中に`.git`ができてしまう．別のディレクトリで試すには，`cargo run`の別名を作る．

```console
$ alias rgit="cargo run -q --manifest-path $PWD/Cargo.toml --"
$ mkdir /tmp/demo && cd /tmp/demo
$ rgit init
Initialized empty Git repository in /tmp/demo/.git/
```

別名は，`exercise/`の最新のコードをビルドしてから実行する．
エラーの表示と終了コードも確かめる．

```console
$ rgit hash-object
error: the following required arguments were not provided:
  <FILE>

Usage: rgit hash-object <FILE>

For more information, try '--help'.
$ echo $?
2
$ cd /tmp
$ rgit hash-object -w demo/hello.txt
fatal: not a git repository (or any of the parent directories): .git
$ echo $?
128
```

## 2-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．
2. `run`は標準出力に直接書かず，`out`に書く．標準出力に直接書く設計と比べて，テストはどう変わるか．
3. `Error`に`io::Error`と`clap::Error`を包んだ．`run`の中で`?`を使えるのはなぜか．包まずに`Box<dyn std::error::Error>`を返す設計と比べて，呼び出す側は何ができるか．
4. 本物の`git`を使うテストと，`rgit`の中だけで確かめるテストは，それぞれ何を保証するか．
5. 図と実装を見比べ，違うところがあれば図を直す．

## 2-7 発展課題

本物の`git init`は，すでにリポジトリがあるディレクトリでは`Reinitialized existing Git repository in <パス>/`と出力する．
`rgit init`も，`.git`がすでにあればこのメッセージを出力するようにする．`HEAD`は書き換えない．

これまでと同じく，テストリスト，図，実装の順に進める．
