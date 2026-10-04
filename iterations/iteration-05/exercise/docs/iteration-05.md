# Iteration 5：インデックス，`add`，`ls-files`

このIterationでは，インデックス`.git/index`をバイト列から読み書きする型を作り，`rgit add`でファイルを登録し，`rgit ls-files`で一覧を表示する．
本物の`git`と同じ形式で書くので，`rgit add`したファイルを`git status`で確かめられる．

## 5-1 準備

このディレクトリ(`iterations/iteration-05/exercise`)に移動し，`cargo test`で，Iteration 4から引き継いだテストがすべて通ることを確かめる．

本物の`git`が書くインデックスを見ておく．一時ディレクトリでリポジトリを作り，`git add`のあとで`xxd`を実行する．

```console
mkdir /tmp/index-demo && cd /tmp/index-demo
git init -q
printf 'hello\n' > hello.txt
git add hello.txt
xxd .git/index
```

時刻やiノードの欄は環境によって違う．[Gitのノート](../../../../docs/git/iteration-05.md)の表と見比べ，ヘッダー，モード，ID，パス，NULの位置を探す．

## 5-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-05.md)：整数とバイト列の変換，`as`，バイト列の組み立てと読み取り，`BTreeMap`，`Default`と更新構文，Unixのファイルの情報，再帰，クロージャとイテレーター
- [Gitのノート](../../../../docs/git/iteration-05.md)：インデックスの役割，ファイルの状態，インデックスの形式

読み終えたら，`src/lib.rs`の末尾に`#[cfg(test)] mod practice`を作り，次の課題を確かめる．

1. `2u32`と`0x1234_5678u32`を`to_be_bytes`と`to_le_bytes`でバイト列にし，`[0, 0, 1, 0]`を`from_be_bytes`で数にする．
2. `BTreeMap`に3つのパスを順不同で入れ，`keys()`がパスの順に返すことを確かめる．同じキーで`insert`すると，値はどうなるか．
3. ディレクトリの下のファイルの数を数える再帰の関数を書き，`TempDir`に作った3つのファイル(うち2つはサブディレクトリの中)で確かめる．
4. `String`の配列から，`src/`で始まるものだけを`filter`と`cloned`で`Vec<String>`に集める．
5. インデックスのエントリーの長さ(固定部62バイトとパス)を，1〜8個のNULで8の倍数にする関数を書く．パスの長さが9，10，11のとき，それぞれ何バイトになるか．

確かめ終えたら，`mod practice`を消す．

## 5-3 テストリスト

### 要件

- インデックス`.git/index`(版2)を読み書きする．
  - ヘッダーは`DIRC`，版，エントリーの数である．数はビッグエンディアンの32ビット整数である．
  - エントリーは，ファイルの状態(作成と変更の時刻，デバイス，iノード，モード，所有者，大きさ)，ID，フラグ，パスからなり，NULで8バイトの境界まで埋める．
  - 末尾には，それより前のバイト列のSHA-1を置く．読むときに検査し，合わなければエラーにする．
  - 拡張は読み飛ばす．エントリーはパスのバイト順に並べる．
- `rgit add <path>...`は，指定したファイル，またはディレクトリの下のすべてのファイルをblobとして書き込み，インデックスに登録する．
  - `.git`は含めない．実行可能なファイルのモードは`100755`，ほかは`100644`とする．
  - 指定したパスの下のファイルが作業ディレクトリから消えていれば，そのファイルをインデックスから除く．
  - 一致するファイルがなければ`pathspec '<path>' did not match any files`のエラーにする．作業ディレクトリの外を指定すると，エラーにする．
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

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `index` | `pub struct Index`(`BTreeMap<String, IndexEntry>`を持つ)，`pub struct IndexEntry`，ファイルの状態`pub struct Stat`，`Index::parse`，`Index::to_bytes`，`Index::load`，`Index::save` |
| `worktree` | `pub fn list_files(work_dir: &Path, dir: &Path) -> Result<Vec<String>, Error>`，`pub fn relative_path(work_dir: &Path, path: &Path) -> Option<String>` |
| `tree` | `Mode`に`u32`との変換を加える |
| `oid` | 20バイトの値を返す`ObjectId::as_bytes`を加える |
| `repo` | `Repository`に作業ディレクトリの場所を加える．`Repository::index_path`，`Repository::add` |
| `cli` | `add`と`ls-files`のサブコマンド |

### 書くときに考えること

- インデックスの形式は，空のインデックス，1エントリー，複数のエントリーの順に確かめると，1項目ずつ小さく進める．
- NULの埋め方の境界を考える．パスの長さによって，NULが1個のときと8個のときがある．
- 書いたものを読み戻して同じになるか(往復)を確かめるテストは，`parse`と`to_bytes`の両方の誤りを見つける．
- 結合テストでは，本物の`git ls-files --stage`と`git status --porcelain`で，`rgit`が書いたインデックスを確かめられる．

## 5-4 図

`design/types.md`を，このIterationの終わりの状態に更新する．

- 新しいモジュール`index`と`worktree`の名前空間を加える．`Index`，`IndexEntry`，`Stat`は，どれがどれを持つか．
- バイト列を読み進める型を作るなら，それも`index`に描く．
- `Repository`に加わるフィールドとメソッド，`Mode`と`ObjectId`に加わるメソッドを書く．
- `Repository`と`cli`は，`index`と`worktree`の何を使うか．

## 5-5 テスト駆動の実装

### 実装のヒント

- `src/lib.rs`に`mod index;`と`mod worktree;`を加える．
- `Index`は`BTreeMap<String, IndexEntry>`を非公開のフィールドに持ち，`entries`，`insert`，`remove`で操作させる．
- `to_bytes`では，ヘッダーとエントリーを`Vec<u8>`に足していき，最後に，それまでのバイト列の`Sha1::digest`を足す．
- `parse`では，先に末尾の20バイトを切り離してチェックサムを確かめる．残りを先頭から読む．`take(n)`，`u32()`，`u16()`を持つ小さな読み取りの型を作ると，`?`で長さの不足を扱える．
- NULの数は，そのエントリーで書いた(読んだ)バイト数から計算する．
- `Stat::from_metadata`は，`MetadataExt`のメソッドの値を`as u32`で切り詰める．
- `list_files`は，ディレクトリなら`fs::read_dir`の各エントリーで自分を呼び直す再帰で書く．名前が`.git`のエントリーは飛ばす．
- `relative_path`は，`strip_prefix`で作業ディレクトリを取り除き，`components()`の各部分を`/`でつなぐ．`..`が出てきたら作業ディレクトリの外である．
- `Repository::add`は，まずインデックスを読む．パスごとにファイルを集めてblobを書いてインデックスに入れ，最後にインデックスを書き戻す．消えたファイルを除くために，指定したパスの下にあるインデックスのパスを`filter`で選ぶ．
- テストで実行可能なファイルを作るには，`fs::set_permissions(&path, fs::Permissions::from_mode(0o755))`を使う．

### ツールの操作

`rgit add`のあとの`.git/index`を`xxd`で表示し，本物の`git add`のインデックスと見比べる．

## 5-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．
2. エントリーを`Vec`ではなく`BTreeMap`で持った．`Vec`で持つ設計と比べて，`add`と`to_bytes`はどう変わるか．
3. `Index`の`entries`を非公開にし，メソッドで操作させた．フィールドを公開する設計と比べて，何を守れるか．
4. 往復のテスト(書いて読み戻す)と，バイト列を直接確かめるテストは，それぞれ何を見つけられるか．
5. 図と実装を見比べ，違うところがあれば図を直す．

## 5-7 発展課題

`git rm --cached <path>`は，作業ディレクトリのファイルを残したまま，インデックスからだけ除く．
`rgit rm --cached <path>...`を作る．

```console
$ rgit rm --cached src
$ rgit ls-files
hello.txt
```

これまでと同じく，テストリスト，図，実装の順に進める．
