# ロードマップ

このハンズオンでは，Gitの互換実装`rgit`を，12回のIterationで少しずつ育てる．
オブジェクトのハッシュの計算から始めて，オブジェクトデータベース，ツリー，インデックス，コミット，参照，`log`，`status`，`diff`までを作る．
最後のIterationで，ファイルのハッシュの計算をスレッドで並列にする．

`rgit`は本物のGitと同じ形式でリポジトリを読み書きする．
`rgit`で作ったコミットは`git log`で読め，`git`で作ったリポジトリは`rgit`で読める．
テストでは本物の`git`を呼び出し，同じ操作から同じハッシュと同じ出力ができることを確かめる．

## 完成形

最後のIterationを終えると，`rgit`は次のように動く．

```console
$ rgit init
Initialized empty Git repository in /home/alice/demo/.git/
$ printf 'hello\n' > hello.txt
$ mkdir src && printf 'fn main() {}\n' > src/main.rs
$ rgit add .
$ rgit status
A  hello.txt
A  src/main.rs
$ export GIT_AUTHOR_NAME=Alice GIT_AUTHOR_EMAIL=alice@example.com GIT_AUTHOR_DATE='@1767225600 +0900'
$ export GIT_COMMITTER_NAME=Alice GIT_COMMITTER_EMAIL=alice@example.com GIT_COMMITTER_DATE='@1767225600 +0900'
$ rgit commit -m first
[main (root-commit) 6c04901] first
$ rgit cat-file -p HEAD
tree aae2b3618f4a481bc1bde056dae4b7617edb7e83
author Alice <alice@example.com> 1767225600 +0900
committer Alice <alice@example.com> 1767225600 +0900

first
$ printf 'world\n' >> hello.txt
$ printf 'notes\n' > todo.txt
$ rgit status
 M hello.txt
?? todo.txt
$ rgit diff
diff --git a/hello.txt b/hello.txt
index ce01362..94954ab 100644
--- a/hello.txt
+++ b/hello.txt
@@ -1 +1,2 @@
 hello
+world
$ rgit add hello.txt
$ export GIT_AUTHOR_DATE='@1767229200 +0900' GIT_COMMITTER_DATE='@1767229200 +0900'
$ rgit commit -m second
[main 85cd268] second
$ rgit log
85cd268 second
6c04901 first
$ git log --oneline
85cd268 second
6c04901 first
```

ファイルの多いディレクトリでは，`add`と`status`がファイルの読み込みとハッシュの計算を複数のスレッドで行う．

```console
$ rgit status --jobs 8
?? todo.txt
$ rgit add --jobs 8 .
$ rgit status --jobs 8
A  todo.txt
```

## 対応するGitの範囲

- オブジェクト：blob，tree，commitを，ゆるいオブジェクト(`.git/objects/xx/…`)として読み書きする．
- インデックス：`.git/index`の版2を読み書きする．拡張は読み飛ばし，書かない．
- 作業ディレクトリ：通常のファイルと実行可能なファイルを扱う．
- 参照：`HEAD`，`refs/heads/`の下のブランチ，シンボリック参照．
- リビジョンの指定：40桁と短縮形のオブジェクトID，`HEAD`，ブランチ名，末尾の`~N`．
- コマンド：`init`，`hash-object`，`cat-file`，`ls-tree`，`add`，`ls-files`，`write-tree`，`commit-tree`，`commit`，`rev-parse`，`branch`，`log`，`status`，`diff`．

## Iterationの進め方

各Iterationは`iterations/iteration-NN/`にあり，`exercise/`と`solution/`の2つのディレクトリからなる．
Iteration 0では，受講者が`exercise/`に`cargo init`でCargoパッケージを作る．
Iteration 1からの`exercise/`は，1つ前のIterationの`solution/`と同じ内容から始まる．
`exercise/`は単独のCargoパッケージなので，`cargo`のコマンドは`exercise/`の中で実行する．
受講者は`exercise/`で次の順に作業する．

1. このロードマップの「要件」と「使用例」を読み，確かめるべき振る舞いをテストリストに書き出す．
2. `design/types.md`の型とモジュールの図を，このIterationの終わりの状態に更新する．
3. テストリストの項目を1つずつ，Red → Green → Refactorで実装する．
4. 実装を終えたら図と実装を見比べ，名前と関係を一致させる(設計レビュー)．

作業を終えたら`solution/`と見比べる．`solution/`にはテストリスト，図，実装の模範解答と，その解説がある．

テストリストの書き方は[tdd.md](tdd.md)，図の書き方は[design.md](design.md)にある．
新しく使うRustの文法と概念は`docs/rust/iteration-NN.md`，Gitの仕組みは`docs/git/iteration-NN.md`で説明する．

## テストの分け方

- 単体テストは，各モジュールの`#[cfg(test)] mod tests`に書く．オブジェクトIDの変換，ヘッダーやツリーの解析，インデックスのバイト列，コミットの直列化，差分の計算など，モジュールの関数を直接確かめる．
- 結合テストは，パッケージの`tests/`に書く．一時ディレクトリにリポジトリを作り，`rgit::cli::run`にコマンドラインの引数を渡して，出力とリポジトリの中身を確かめる．
  - Iteration 2からは，同じ操作を本物の`git`でも行い，ハッシュや出力が一致することを確かめる．

## Iteration一覧

| # | 作る機能 | Rustで学ぶこと | Gitで学ぶこと |
| --- | --- | --- | --- |
| 0 | blobのハッシュの計算 | Cargo，関数，`&[u8]`と`Vec<u8>`，外部クレート，`#[test]` | 内容アドレス，blobとSHA-1 |
| 1 | オブジェクトID | 所有権，ムーブ，借用，`Copy`，ニュータイプ，`Display`と`FromStr` | オブジェクトIDと16進表記 |
| 2 | `init`と`hash-object -w` | clapのderive，`Path`と`PathBuf`，`std::fs`，`Write`トレイト，thiserrorと`?` | `.git`の構成，ゆるいオブジェクトとzlib |
| 3 | `cat-file` | `Read`トレイト，スライスの分割，ライフタイムの省略，`Option`と`Result`の変換 | オブジェクトのヘッダー，IDの短縮形 |
| 4 | ツリーの読み取りと`ls-tree` | ライフタイム注釈，参照を持つ構造体，`TryFrom` | treeオブジェクト，ファイルのモード |
| 5 | インデックス，`add`，`ls-files` | バイト列の読み書き，`from_be_bytes`，`BTreeMap`，再帰，`collect`と`Result` | インデックスの形式，ステージング |
| 6 | `write-tree`と`commit-tree` | `Ordering`とイテレータの比較，ジェネリクス，`PhantomData`，型状態パターン | インデックスからツリーを作る，commitオブジェクト |
| 7 | 参照，`commit`，`branch` | 検査済みのニュータイプ，`Drop`とRAII，`self`を消費するメソッド | 参照，`HEAD`，ロックファイル |
| 8 | `log` | `Iterator`の実装，構造体の中の参照，`BinaryHeap`，`HashSet` | コミットのグラフ，`~N` |
| 9 | オブジェクトストアの抽象化と`status` | トレイトの設計，ジェネリクスとトレイトオブジェクト，`BTreeMap`の突き合わせ | HEAD，インデックス，作業ディレクトリの3者の比較 |
| 10 | `diff` | トレイト境界を持つジェネリック関数，`enum`による編集の表現，`fmt::Write` | Myersの差分アルゴリズム，unified形式 |
| 11 | `add`と`status`の並列化 | `std::thread::scope`，`Send`と`Sync`，`Mutex`，チャネル | オブジェクトの書き込みの原子性 |

## Iteration 0：blobのハッシュの計算

### 要件

- `cargo init`で，ライブラリクレート`rgit`のパッケージを作る．
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

### モジュール

- `object`：`pub fn hash_blob(data: &[u8]) -> String`，バイト列を16進数の文字列にする`fn to_hex(bytes: &[u8]) -> String`
- `lib.rs`：`mod object;`と`pub use`
- `main.rs`：標準入力を読み，`hash_blob`の結果を出力する．

### 図の更新

- `types.md`：`object`の名前空間と，`hash_blob`を持つ`object`モジュールのクラスを描く．

### 学ぶこと

- Rust：Cargoのパッケージとクレート，`Cargo.toml`，`fn`，整数型と`u8`，バイト列のリテラル`b"…"`，`&[u8]`と`Vec<u8>`の基本
- Rust：`String`と`format!`，`for`，`mod`と`pub use`，`#[test]`と`assert_eq!`，`sha1`クレートの`Digest`
- Git：内容アドレス(内容からIDが決まる)，blobオブジェクトの形式，SHA-1

### 受講者が行うツール操作

- `cargo init --lib --name rgit`で，`exercise/`をCargoパッケージにする．
- `cargo add sha1`で依存を追加する．
- `cargo build`，`cargo test`，`cargo test --lib`，`cargo fmt`，`cargo clippy`を実行する．
- `src/main.rs`を加え，`cargo run`で実行する．

## Iteration 1：オブジェクトID

### 要件

- オブジェクトIDを表す型`ObjectId`を作る．中身は20バイトのSHA-1である．
- `ObjectId`は40桁の小文字の16進数として表示する．
- 40桁の16進数の文字列から`ObjectId`を作れる．大文字も受け付ける．
  - 長さが40でなければ，その長さを含むエラーにする．
  - 16進数でない文字があれば，その位置(先頭を0とする)と文字を含むエラーにする．
- `ObjectId::short`は，先頭の7桁を返す．
- `hash_blob`は`ObjectId`を返す．

### 使用例

```rust
use rgit::{ObjectId, ParseObjectIdError};

let id = rgit::hash_blob(b"hello\n");
assert_eq!(id.to_string(), "ce013625030ba8dba906f756967f9e9ca394464a");
assert_eq!(id.short(), "ce01362");
assert_eq!("CE013625030BA8DBA906F756967F9E9CA394464A".parse::<ObjectId>(), Ok(id));
assert_eq!("ce01".parse::<ObjectId>(), Err(ParseObjectIdError::InvalidLength(4)));
```

### モジュール

- `oid`：`pub struct ObjectId([u8; 20])`，`impl Display`，`impl FromStr`，`pub fn short(&self) -> String`，`pub enum ParseObjectIdError`
- `object`：`hash_blob`の戻り値を`ObjectId`にする．`to_hex`は`ObjectId`の`Display`に移す．

### 図の更新

- `types.md`：`oid`の名前空間に`ObjectId`と`ParseObjectIdError`を加え，`Display`と`FromStr`の実装を書く．`object`から`ObjectId`への依存を描く．

### 学ぶこと

- Rust：所有権，ムーブ，借用(`&`と`&mut`)，`Copy`と`Clone`，固定長の配列`[u8; 20]`
- Rust：タプル構造体とニュータイプパターン，`impl`とメソッド，`#[derive]`，`Display`と`FromStr`の実装，`str::parse`，データを持つ`enum`によるエラー
- ツール：RustOwlで，値の所有権がムーブする位置と借用の範囲を見る．
- Git：オブジェクトIDの16進表記と短縮形

### 既存テストへの影響

- `hash_blob`の結果を文字列と比べていたテストを，`to_string()`の結果と比べるように変える．

## Iteration 2：`init`と`hash-object -w`

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
Initialized empty Git repository in /home/alice/demo/.git/
$ printf 'hello\n' > hello.txt
$ rgit hash-object -w hello.txt
ce013625030ba8dba906f756967f9e9ca394464a
$ git cat-file -p ce013625030ba8dba906f756967f9e9ca394464a
hello
```

### モジュール

- `cli`：clapのderiveによる`struct Cli`と`enum Command`，`pub fn run(args: &[&str], cwd: &Path, out: &mut impl Write) -> Result<(), Error>`
- `repo`：`.git`の場所を持つ`pub struct Repository`，`Repository::init`，`Repository::discover`，`Repository::write_blob`
- `error`：thiserrorによる`pub enum Error`
- `object`：ヘッダーと内容を連結したバイト列を作る`blob_bytes`を加え，`hash_blob`はそのハッシュを計算する(リファクタリング)．
- `main.rs`：`cli::run`を呼び，エラーを出力する．

### 図の更新

- `types.md`：`cli`，`repo`，`error`の名前空間と，`Cli`，`Command`，`Repository`，`Error`を加える．`cli`から`Repository`への依存を描く．

### 学ぶこと

- Rust：属性とderiveマクロ，clapのderive(`Parser`，`Subcommand`)，`Path`と`PathBuf`，`Path::ancestors`，`std::fs`
- Rust：`io::Write`トレイト，引数の`impl Write`，`Vec<u8>`への書き込み，thiserrorによるエラー型，`#[from]`と`?`による変換
- Rust：`if let`と`matches!`，イテレーターの`skip`，`map`，`collect`の基本，`std::process::exit`，`std::process::Command`を使うテスト
- Git：`.git`ディレクトリの構成，ゆるいオブジェクト，zlib

### 受講者が行うツール操作

- `cargo add clap --features derive`，`cargo add flate2 thiserror`で依存を追加する．
- `cargo add --dev tempfile`で，テストだけで使う依存を追加する．
- `cargo run -- init`のように，`--`の後ろにプログラムの引数を渡す．
- `cargo test --test 名前`で，1つの結合テストのファイルだけを実行する．
- `cargo run`の`--manifest-path`を使う別名`rgit`を作り，別のディレクトリで試す．

### 既存テストへの影響

- `src/main.rs`は標準入力を読まなくなる．標準入力のハッシュの計算は，発展課題の`hash-object --stdin`で扱う．

## Iteration 3：`cat-file`

### 要件

- `rgit cat-file (-t | -s | -p) <object>`は，オブジェクトの種類，大きさ，内容を出力する．
  - `-t`と`-s`は，blob，tree，commitのすべてに使える．
  - `-p`はblobとcommitに使える．内容をそのまま出力する．treeはIteration 4で扱う．
- オブジェクトは，4桁以上40桁以下の16進数で指定できる．
  - 一致するオブジェクトがなければ`Not a valid object name <object>`のエラーにする．
  - 2つ以上一致すれば`short object ID <object> is ambiguous`のエラーにする．
- ヘッダーが壊れている，または大きさが内容と合わなければエラーにする．

### 使用例

```console
$ rgit cat-file -t ce01362
blob
$ rgit cat-file -s ce01362
6
$ rgit cat-file -p ce01362
hello
```

### モジュール

- `object`：`pub enum ObjectKind { Blob, Tree, Commit }`，`impl FromStr`と`impl Display`，`pub fn parse_header(data: &[u8]) -> Result<(ObjectKind, &[u8]), Error>`
- `object`：`blob_bytes`を一般にし，種類も引数で受け取る`encode`にする．
- `repo`：`Repository::read_object`(種類と内容を返す)，`Repository::resolve_prefix`
- `cli`：`cat-file`のサブコマンド．`-t`，`-s`，`-p`はどれか1つだけを指定できる．

### 図の更新

- `types.md`：`ObjectKind`を加え，`Repository`から`ObjectKind`への依存を描く．

### 学ぶこと

- Rust：`io::Read`トレイトと`read_to_end`，スライスの`iter().position`と`split_at`，`std::str::from_utf8`，`fs::read_dir`の基本
- Rust：引数の借用を返す関数，ライフタイムの省略規則，`Option`と`Result`の変換(`ok_or`，`map_err`，`and_then`)
- Git：オブジェクトのヘッダー，IDの短縮形と曖昧さ

## Iteration 4：ツリーの読み取りと`ls-tree`

### 要件

- treeオブジェクトの内容を解析する．
  - 内容は`<モード> <名前>\0<20バイトのID>`の並びである．
  - モードは`100644`(通常のファイル)，`100755`(実行可能なファイル)，`120000`(シンボリックリンク)，`40000`(ディレクトリ)，`160000`(サブモジュール)のどれかである．
  - 名前はUTF-8でなければエラーにする．
- `rgit cat-file -p <tree>`と`rgit ls-tree <tree>`は，エントリーを`<6桁のモード> <種類> <ID>\t<名前>`の形で1行ずつ出力する．

### 使用例

```console
$ rgit ls-tree aae2b36
100644 blob ce013625030ba8dba906f756967f9e9ca394464a	hello.txt
040000 tree 5d90422423db5ef6b431e8b9e60e0baf04b8742a	src
```

### モジュール

- `tree`：`pub enum Mode`(`TryFrom<&[u8]>`)，`pub struct TreeEntry<'a> { mode, name: &'a str, id }`
- `tree`：`pub fn parse_tree(data: &[u8]) -> Result<Vec<TreeEntry<'_>>, Error>`
- `cli`：`ls-tree`のサブコマンド．`cat-file -p`でtreeを扱う．

### 図の更新

- `types.md`：`tree`の名前空間に`Mode`と`TreeEntry`を加える．`TreeEntry`が`Mode`と`ObjectId`を持つことを描く．

### 学ぶこと

- Rust：ライフタイム注釈`'a`，参照を持つ構造体，元のバイト列を借用したまま解析する設計と，所有するデータに写す設計の比較
- Rust：`TryFrom`と`TryInto`，スライスから配列への変換
- ツール：RustOwlで，`TreeEntry`の名前が元のバイト列を借用している範囲を見る．
- Git：treeオブジェクトの形式，ファイルのモード

## Iteration 5：インデックス，`add`，`ls-files`

### 要件

- インデックス`.git/index`(版2)を読み書きする．
  - ヘッダーは`DIRC`，版，エントリーの数である．数はビッグエンディアンの32ビット整数である．
  - エントリーは，ファイルの状態(作成と変更の時刻，デバイス，iノード，モード，所有者，大きさ)，ID，フラグ，パスからなり，NULで8バイトの境界まで埋める．
  - 末尾には，それより前のバイト列のSHA-1を置く．読むときに検査し，合わなければエラーにする．
  - 拡張は読み飛ばす．エントリーはパスのバイト順に並べる．
- `rgit add <path>...`は，指定したファイル，またはディレクトリの下のすべてのファイルをblobとして書き込み，インデックスに登録する．
  - `.git`は含めない．実行可能なファイルのモードは`100755`，ほかは`100644`とする．
  - 指定したパスの下のファイルが作業ディレクトリから消えていれば，そのファイルをインデックスから除く．
- `rgit ls-files`はインデックスのパスを，`rgit ls-files --stage`は`<モード> <ID> 0\t<パス>`を1行ずつ出力する．
- 本物の`git`は`rgit`が書いたインデックスを読め，`rgit`は`git add`で書いたインデックスを読める．

### 使用例

```console
$ rgit add .
$ rgit ls-files --stage
100644 ce013625030ba8dba906f756967f9e9ca394464a 0	hello.txt
100644 f328e4d9d04c31d0d70d16d21a07d1613be9d577 0	src/main.rs
$ git ls-files --stage
100644 ce013625030ba8dba906f756967f9e9ca394464a 0	hello.txt
100644 f328e4d9d04c31d0d70d16d21a07d1613be9d577 0	src/main.rs
```

### モジュール

- `index`：`pub struct Index`(`BTreeMap<String, IndexEntry>`を持つ)，`pub struct IndexEntry`，`Index::parse`，`Index::to_bytes`，`Index::load`，`Index::save`
- `worktree`：`pub fn list_files(work_dir: &Path, dir: &Path) -> Result<Vec<String>, Error>`
- `tree`：`Mode`に`u32`との変換を加える．
- `repo`：`Repository::add`
- `cli`：`add`と`ls-files`のサブコマンド

### 図の更新

- `types.md`：`index`と`worktree`の名前空間を加え，`Index`が`IndexEntry`を持ち，`IndexEntry`が`Mode`と`ObjectId`を持つことを描く．

### 学ぶこと

- Rust：`u32::from_be_bytes`と`to_be_bytes`，`as`による整数の変換，`Vec::extend_from_slice`，バイト列を先頭から読む小さな読み取り器
- Rust：`BTreeMap`と`range`，`std::os::unix::fs::MetadataExt`と`PermissionsExt`
- Rust：再帰と`Result`，`fs::read_dir`と`DirEntry`，クロージャ，`collect::<Result<Vec<_>, _>>()`
- Git：インデックス(ステージングエリア)の役割と形式，ファイルの状態の記録

### 受講者が行うツール操作

- `xxd`で`.git/index`のバイト列を表示して，形式を確かめる．

## Iteration 6：`write-tree`と`commit-tree`

### 要件

- `rgit write-tree`は，インデックスからtreeオブジェクトを作って書き込み，最上位のtreeのIDを出力する．
  - ディレクトリごとにtreeを作る．エントリーはGitの順序で並べる．ディレクトリの名前は，末尾に`/`があるものとして比べる．
  - 結果は，同じインデックスで`git write-tree`をしたものと一致する．
- commitオブジェクトを作り，解析し，直列化する．
  - 内容は`tree`，0個以上の`parent`，`author`，`committer`の行，空行，メッセージである．
  - 署名は`<名前> <<メール>> <UNIX時刻> <±hhmm>`の形である．
- `rgit commit-tree <tree> [-p <parent>]... -m <message>`は，commitを書き込み，そのIDを出力する．
  - 作者とコミッターは環境変数`GIT_AUTHOR_NAME`，`GIT_AUTHOR_EMAIL`，`GIT_AUTHOR_DATE`，`GIT_COMMITTER_NAME`，`GIT_COMMITTER_EMAIL`，`GIT_COMMITTER_DATE`から読む．
  - 時刻は`@<UNIX時刻> <±hhmm>`の形とし，省略すれば現在の時刻と`+0000`を使う．名前かメールがなければエラーにする．
  - メッセージの末尾には改行を1つ付ける．
  - 同じ環境変数で本物の`git commit-tree`を実行した結果と，IDが一致する．
- `cat-file -p`は，commitを解析してから直列化して出力する．

### 使用例

```console
$ rgit write-tree
aae2b3618f4a481bc1bde056dae4b7617edb7e83
$ rgit commit-tree aae2b36 -m first
6c049013df700446ee9afd1bdaa0303bf75d842f
```

```rust
let commit = Commit::builder()
    .tree(tree_id)
    .author(alice.clone())
    .committer(alice)
    .message("first\n")
    .build();
```

`tree`，`author`，`committer`を呼ぶ前の`build`はコンパイルエラーになる．

### モジュール

- `tree`：`pub fn tree_bytes(entries: &[TreeEntry]) -> Vec<u8>`，エントリーの順序を決める関数
- `repo`：`Repository::write_tree`
- `commit`：`pub struct Signature`，`pub struct Commit`，`Commit::parse`，`Commit::to_bytes`，`pub struct CommitBuilder<T, A, C>`と状態を表す型
- `cli`：`write-tree`と`commit-tree`のサブコマンド．`run`は環境変数を`&HashMap<String, String>`で受け取る．

### 図の更新

- `types.md`：`commit`の名前空間に`Commit`，`Signature`，`CommitBuilder`と状態を表す型を加える．`Repository`から`TreeEntry`と`Index`への依存を描く．

### 学ぶこと

- Rust：`Ordering`，`sort_by`，イテレータの`chain`と`cmp`による比較，スライスのパターンと`split_first`
- Rust：ジェネリクスの型引数，ゼロサイズ型と`PhantomData`，型状態パターン(状態ごとに`impl`を分ける)
- Rust：`str::split_once`と`strip_prefix`，`HashMap`，`SystemTime`
- Git：インデックスからツリーを組み立てる手順，エントリーの並び順，commitオブジェクトの形式，作者とコミッター

### 既存テストへの影響

- `cli::run`の引数に環境変数が加わるので，結合テストの補助関数を変える．

## Iteration 7：参照，`commit`，`branch`

### 要件

- 参照名を表す型`RefName`を作る．`HEAD`か，`refs/`で始まる名前だけを受け付ける．
  - 空の要素，`.`で始まる要素，`..`，空白，`~^:?*[\`，末尾の`/`と`.lock`を含む名前はエラーにする．
- 参照はファイル`.git/<参照名>`に書く．中身は40桁のIDか，`ref: <参照名>`(シンボリック参照)である．
- 参照とインデックスを更新するときは`<ファイル名>.lock`を作って書き込み，名前を変えて置き換える．
  - `.lock`がすでにあればエラーにする．途中で失敗したら`.lock`を消す．
- `rgit commit -m <message>`は，インデックスから`write-tree`をし，`HEAD`が指すコミットを親にしてcommitを作り，`HEAD`が指すブランチを更新する．
  - 出力は`[<ブランチ名> <短縮ID>] <メッセージの1行目>`で，最初のコミットでは`(root-commit)`を付ける．
- `rgit rev-parse <rev>`は，`HEAD`，ブランチ名，40桁または短縮形のIDを，40桁のIDにして出力する．
- `rgit branch`はブランチの一覧を，今のブランチに`*`を付けて出力する．`rgit branch <name> [<rev>]`はブランチを作る．

### 使用例

```console
$ rgit commit -m first
[main (root-commit) 6c04901] first
$ rgit branch topic
$ rgit branch
* main
  topic
$ rgit rev-parse topic
6c049013df700446ee9afd1bdaa0303bf75d842f
```

### モジュール

- `refs`：`pub struct RefName(String)`，`impl TryFrom<&str>`，`pub enum Ref { Direct(ObjectId), Symbolic(RefName) }`
- `lockfile`：`pub struct LockFile`，`LockFile::acquire`，`LockFile::commit(self)`，`impl Drop`
- `revision`：`pub fn resolve(repo: &Repository, rev: &str) -> Result<ObjectId, Error>`
- `repo`：参照を読み書きするメソッド，`Repository::commit`
- `index`：`Index::save`は`LockFile`で書く(リファクタリング)．
- `cli`：`commit`，`rev-parse`，`branch`のサブコマンド

### 図の更新

- `types.md`：`refs`，`lockfile`，`revision`の名前空間と，`RefName`，`Ref`，`LockFile`を加える．`Ref`が`RefName`を持つことを描く．

### 学ぶこと

- Rust：検査済みの値だけを持つニュータイプ，`TryFrom`による変換，`AsRef<str>`
- Rust：`Drop`とRAII，`self`を受け取って値を消費するメソッド，`OpenOptions::create_new`，`fs::rename`
- ツール：RustOwlで，`LockFile`が`commit`でムーブされたあとに使えないことを見る．
- Git：参照，ブランチ，`HEAD`，シンボリック参照，ロックファイルによる更新

## Iteration 8：`log`

### 要件

- `rgit log [-n <N>] [<rev>]`は，`<rev>`(省略時は`HEAD`)から親をたどり，コミットを`<短縮ID> <メッセージの1行目>`の形で1行ずつ出力する．
  - コミッターの時刻の新しい順に出力する．同じコミットは1回だけ出力する．
  - `-n`があれば，最大`N`件を出力する．
- リビジョンの指定で`<rev>~N`(最初の親をN回たどる)を使える．
- 結果は，本物の`git log --oneline`と一致する．

### 使用例

```console
$ rgit log
85cd268 second
6c04901 first
$ rgit log -n 1 HEAD~1
6c04901 first
```

### モジュール

- `revwalk`：`pub struct RevWalk<'r> { repo: &'r Repository, queue: BinaryHeap<…>, seen: HashSet<ObjectId> }`
- `revwalk`：`impl Iterator for RevWalk<'_>`(`Item = Result<(ObjectId, Commit), Error>`)
- `revision`：`~N`を解釈する．
- `cli`：`log`のサブコマンド

### 図の更新

- `types.md`：`revwalk`の名前空間に`RevWalk`を加え，`Iterator`の実装と`Repository`への参照を描く．

### 学ぶこと

- Rust：`Iterator`トレイトの実装と関連型`Item`，イテレータの遅延評価と`take`，参照を持つ構造体のライフタイム
- Rust：`BinaryHeap`と`Reverse`，`HashSet`，`Hash`の導出
- Git：コミットのグラフ(有向非巡回グラフ)，`log`の出力の順序，`~N`

## Iteration 9：オブジェクトストアの抽象化と`status`

### 要件

- オブジェクトの読み書きをトレイト`ObjectStore`にまとめる(リファクタリング)．
  - ディスクのゆるいオブジェクトを読み書きする`LooseObjectStore`と，メモリーに持つ`MemoryObjectStore`の2つを実装する．
  - ツリーを書く関数，`RevWalk`，`status`はオブジェクトストアを型引数に取る．単体テストは`MemoryObjectStore`で書く．
- `rgit status`は，HEADのツリー，インデックス，作業ディレクトリを比べ，`<X><Y> <パス>`の形でパスの順に出力する．
  - `X`はHEADとインデックスの違いで，追加は`A`，変更は`M`，削除は`D`，同じなら空白である．
  - `Y`はインデックスと作業ディレクトリの違いで，変更は`M`，削除は`D`，同じなら空白である．作業ディレクトリのファイルはハッシュを計算して比べる．
  - インデックスにないファイルは`?? <パス>`と出力する．
- 結果は，本物の`git status --porcelain -uall`と一致する．

### 使用例

```console
$ rgit status
 M hello.txt
?? todo.txt
$ rgit add hello.txt
$ rgit status
M  hello.txt
?? todo.txt
```

### モジュール

- `store`：`pub trait ObjectStore`，`pub struct LooseObjectStore`，`pub struct MemoryObjectStore`
- `repo`：`Repository`はオブジェクトストアとして`LooseObjectStore`を持つ．
- `tree`：`pub fn flatten_tree<S: ObjectStore>(store: &S, id: ObjectId) -> Result<BTreeMap<String, (Mode, ObjectId)>, Error>`
- `status`：`pub enum Change`，`pub struct StatusEntry`，`pub fn status<S: ObjectStore>(…) -> Result<Vec<StatusEntry>, Error>`
- `cli`：`status`のサブコマンド

### 図の更新

- `types.md`：`store`の名前空間に`ObjectStore`と2つの実装を加え，実現の関係を描く．`status`の名前空間を加える．

### 学ぶこと

- Rust：トレイトの設計(何をトレイトにし，何を具体的な型に残すか)，トレイト境界とジェネリクス，`?Sized`
- Rust：`dyn Trait`との比較(静的ディスパッチと動的ディスパッチ)，テストのための差し替え
- Rust：2つの`BTreeMap`の突き合わせ，`Option`のタプルによる`match`
- Git：HEAD，インデックス，作業ディレクトリの3者の比較，追跡されていないファイル

### 既存テストへの影響

- `Repository`の書き込みのメソッドを使っていた単体テストは，`MemoryObjectStore`を使う形に変わる．結合テストは変わらない．

## Iteration 10：`diff`

### 要件

- 2つの列の最短の編集(一致，削除，挿入の列)を，Myersのアルゴリズムで求める．要素の型は比べられるものなら何でもよい．
- `rgit diff`はインデックスと作業ディレクトリの，`rgit diff --cached`はHEADとインデックスの差分を，unified形式で出力する．
  - ファイルごとに`diff --git a/<パス> b/<パス>`，`index <短縮ID>..<短縮ID> <モード>`，`--- a/<パス>`，`+++ b/<パス>`を出力する．
  - 追加されたファイルは`new file mode <モード>`を，削除されたファイルは`deleted file mode <モード>`を出力し，ない側を`/dev/null`とする．
  - 変更の前後3行を文脈として含め，近いハンクはまとめる．ハンクの見出しは`@@ -<開始>,<行数> +<開始>,<行数> @@`で，行数が1なら省略する．
  - 末尾に改行のないファイルは，最後の行の後ろに`\ No newline at end of file`を出力する．
  - NULを含むファイルは`Binary files a/<パス> and b/<パス> differ`と出力する．
- 結果は，本物の`git diff`と一致する(ハンクの見出しの後ろの関数名を除く)．

### 使用例

```console
$ rgit diff
diff --git a/hello.txt b/hello.txt
index ce01362..94954ab 100644
--- a/hello.txt
+++ b/hello.txt
@@ -1 +1,2 @@
 hello
+world
```

```rust
use rgit::diff::{diff, Edit};

assert_eq!(
    diff(&["a", "b", "c"], &["a", "c", "d"]),
    vec![Edit::Equal(0, 0), Edit::Delete(1), Edit::Equal(2, 1), Edit::Insert(2)],
);
```

### モジュール

- `diff`：`pub enum Edit`，`pub fn diff<T: PartialEq>(a: &[T], b: &[T]) -> Vec<Edit>`，`pub struct Hunk`，`pub fn hunks(edits: &[Edit], context: usize) -> Vec<Hunk>`
- `patch`：ファイルの差分をunified形式の文字列にする関数
- `cli`：`diff`のサブコマンドと`--cached`

### 図の更新

- `types.md`：`diff`と`patch`の名前空間と，`Edit`，`Hunk`を加える．`patch`から`diff`と`status`への依存を描く．

### 学ぶこと

- Rust：トレイト境界を持つジェネリック関数，`enum`による操作の表現，`usize`と`isize`の変換と添字の計算
- Rust：`str::split_inclusive`，`fmt::Write`と`write!`による文字列の組み立て，`windows`と`chunk_by`
- Git：Myersの差分アルゴリズム(編集グラフと対角線)，unified形式，ハンクと文脈

## Iteration 11：`add`と`status`の並列化

### 要件

- `rgit add`と`rgit status`は，ファイルの読み込みとハッシュの計算を複数のスレッドで行う．
  - スレッドの数は`--jobs <N>`で指定する．省略すれば`std::thread::available_parallelism`の値を使う．
  - 結果は，スレッドの数によらず同じである．
- `ObjectStore`の書き込みは`&self`で行い，トレイトに`Send + Sync`を求める．
  - `MemoryObjectStore`は`Mutex`で中身を守る．
  - `LooseObjectStore`は一時ファイルに書いてから名前を変え，同じオブジェクトを同時に書いても壊れないようにする．

### 使用例

```console
$ rgit add --jobs 8 .
$ rgit status --jobs 8
A  hello.txt
A  src/main.rs
```

### モジュール

- `store`：`ObjectStore::write`を`&self`にする．`MemoryObjectStore`の中身を`Mutex<HashMap<…>>`にする．
- `parallel`：`pub fn map_parallel<T, R, F>(items: &[T], jobs: usize, f: F) -> Vec<R>`(`std::thread::scope`とチャネルを使う)
- `repo`と`status`：ファイルごとの処理を`map_parallel`で行う．
- `cli`：`--jobs`の引数

### 図の更新

- `types.md`：`ObjectStore`に`Send + Sync`の境界を書き，`MemoryObjectStore`が`Mutex`を持つことを書く．`parallel`の名前空間を加える．

### 学ぶこと

- Rust：スレッド，`std::thread::scope`，クロージャのトレイト`Fn`と`Send`，`Send`と`Sync`，`available_parallelism`
- Rust：`Mutex`と内部可変性，`RefCell`を持つ型を共有しようとしたときのコンパイルエラー，`mpsc`のチャネル，`thread::spawn`と`Arc`との比較
- Git：オブジェクトの書き込みの原子性(一時ファイルと名前の変更)

### 既存テストへの影響

- `ObjectStore::write`が`&self`になるので，`&mut`で呼んでいたテストを変える．
