# Iteration 6：`write-tree`と`commit-tree`

このIterationでは，インデックスから入れ子のtreeオブジェクトを書く`rgit write-tree`と，commitオブジェクトを作る`rgit commit-tree`を作る．
commitは，必要なものがそろう前に組み立てを終えるとコンパイルエラーになるビルダーで作る．

## 6-1 準備

このディレクトリ(`iterations/iteration-06/exercise`)に移動し，`cargo test`で，Iteration 5から引き継いだテストがすべて通ることを確かめる．

## 6-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-06.md)：`Ordering`と並べ替え，イテレーターの比較，`while let`，値を受け取る関数，ジェネリクス，型状態パターンと`PhantomData`，環境変数と`HashMap`，文字列の分割，現在の時刻
- [Gitのノート](../../../../docs/git/iteration-06.md)：インデックスからtreeを作る手順，エントリーの並び順，commitオブジェクト，署名と時刻，環境変数

読み終えたら，`src/lib.rs`の末尾に`#[cfg(test)] mod practice`を作り，次の課題を確かめる．

1. `["bb", "a", "ab", "c"]`を，長さの順，同じ長さなら辞書順に`sort_by`で並べる．
2. `"a".bytes().chain(Some(b'/'))`と`"a.txt".bytes().chain(None)`を`cmp`で比べる．`true.then_some(b'/')`の値も確かめる．
3. ノートの`Door<S>`を書き，開けたドアだけを通れることを確かめる．閉じたドアで`walk_through`を呼ぶと，どんなエラーになるか．
4. `HashMap<String, String>`に環境変数のような組を入れ，`get`と`cloned`で取り出す．`SystemTime::now()`の秒数が，1700000000より大きいことを確かめる．
5. `"+0900".split_at_checked(1)`と`"".split_at_checked(1)`の結果を確かめる．

確かめ終えたら，`mod practice`を消す．

## 6-3 テストリスト

### 要件

- `rgit write-tree`は，インデックスからtreeオブジェクトを作って書き込み，最上位のtreeのIDを出力する．
  - ディレクトリごとにtreeを作る．エントリーはGitの順序で並べる．ディレクトリの名前は，末尾に`/`があるものとして比べる．
  - 結果は，同じインデックスで`git write-tree`をしたものと一致する．
- commitオブジェクトを作り，解析し，直列化する．
  - 内容は`tree`，0個以上の`parent`，`author`，`committer`の行，空行，メッセージである．
  - 署名は`<名前> <<メール>> <UNIX時刻> <±hhmm>`の形である．
- `rgit commit-tree <tree> [-p <parent>]... -m <message>`は，commitを書き込み，そのIDを出力する．
  - 作者とコミッターは環境変数`GIT_AUTHOR_NAME`，`GIT_AUTHOR_EMAIL`，`GIT_AUTHOR_DATE`，`GIT_COMMITTER_NAME`，`GIT_COMMITTER_EMAIL`，`GIT_COMMITTER_DATE`から読む．
  - 時刻は`@<UNIX時刻> <±hhmm>`の形とし，省略すれば現在の時刻と`+0000`を使う．
  - 名前かメールがなければ`environment variable <名前> is not set`のエラーにする．
  - メッセージの末尾には改行を1つ付ける．
  - 同じ環境変数で本物の`git commit-tree`を実行した結果と，IDが一致する．
- commitの内容を解析した結果を直列化すると，元の内容に戻る．

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

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `tree` | エントリーの順序を決める`pub fn compare_entries(a: &TreeEntry, b: &TreeEntry) -> Ordering`，エントリーを並べて内容にする`pub fn tree_bytes(entries: Vec<TreeEntry<'_>>) -> Vec<u8>` |
| `object` | 種類を問わずIDを計算する`hash_object` |
| `repo` | `Repository::write_object`，`Repository::write_tree` |
| `commit` | `pub struct Signature`，`pub struct Commit`，`Commit::parse`，`Commit::to_bytes`，`pub struct CommitBuilder<T, A, C>`と，指定していないことを表す型`pub struct Missing` |
| `cli` | `write-tree`と`commit-tree`のサブコマンド．`run`は環境変数を`&HashMap<String, String>`で受け取る |

### 書くときに考えること

- 並び順の境界になる名前を選ぶ．`a`というディレクトリと，`a-b`，`a.txt`というファイルの順は，本物の`git`ではどうなるか．
- `write_tree`は，入れ子のディレクトリ(`src/bin/tool.rs`)と，空のインデックスで確かめる．
- 署名のタイムゾーンは，東(`+`)と西(`-`)，時間と分の両方を持つずれ(`-0130`)で確かめる．
- `cli::run`の引数が増えると，既存の結合テストはどうなるか．補助関数`rgit`だけを直せば済むように，エラーを確かめるテストも補助関数を通す形にそろえる．
- 結合テストでは，`rgit`と本物の`git`に同じ作者とコミッターの環境変数を渡す．`Command`の`envs`に`HashMap`を渡せる．

## 6-4 図

`design/types.md`を，このIterationの終わりの状態に更新する．

- 新しいモジュール`commit`の名前空間に，`Signature`，`Commit`，`Missing`，`CommitBuilder~T, A, C~`を描く．どれがどれを持ち，どれがどれを作るか．
- `tree_mod`，`object_mod`，`Repository`に加わる関数とメソッドを書く．
- `cli_mod`の`run`の引数が変わる．`cli`は`commit`の何を使うか．

## 6-5 テスト駆動の実装

### 実装のヒント

- `compare_entries`は，名前のバイト列に，ディレクトリなら`b'/'`を`chain`でつないだイテレーター同士を`cmp`で比べる．
- `tree_bytes`のモードは，treeの中の表記(`40000`)で書く．`format!("{:o}", mode.bits())`で8進数にできる．
- 種類を引数で受け取る`write_object`を作り，`write_blob`はそれを呼ぶだけの形にする(Refactor)．
- `write_tree`では，インデックスのエントリーを`(パス, &IndexEntry)`の列にし，1つのディレクトリのtreeを書くメソッドを再帰で呼ぶ．`/`を含むパスは，同じディレクトリ名で始まる連続した範囲を`position`で探し，ディレクトリ名と`/`を取り除いた列で呼び直す．
- `TreeEntry`の名前は，インデックスのパスの一部を借りられる．
- `CommitBuilder`の`tree`，`author`，`committer`は，`self`を受け取って，型引数の1つを変えた`CommitBuilder`を返す．`parent`と`message`は，型を変えずに`self`を返す．
- `Signature::parse`は，`split_once(" <")`，`split_once("> ")`，`split_once(' ')`の順に分ける．
- 環境変数は`env.get(&name)`で読む．`GIT_AUTHOR_DATE`がなければ`SystemTime`で現在の時刻を使う．
- `-p`は何回でも指定できる．clapでは，フィールドを`Vec<String>`にする．

## 6-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．
2. `CommitBuilder`を使わず，`Commit`のフィールドを直接書いて作る場合と比べる．ビルダーは何を保証し，何を書きにくくするか．
3. `build`で「treeを指定したか」を実行時に調べる設計(`Option<ObjectId>`を持ち，`None`ならエラー)と比べる．エラーはいつ，誰が見つけるか．
4. `run`に環境変数を引数で渡した．`run`の中で`std::env::var`を呼ぶ設計と比べて，テストはどう変わるか．
5. 図と実装を見比べ，違うところがあれば図を直す．

## 6-7 発展課題

本物の`git ls-tree`は，treeの代わりにコミットを指定すると，そのコミットのtreeを表示する．
`rgit ls-tree`も，コミットを受け付けるようにする．

```console
$ rgit ls-tree 6c04901
100644 blob ce013625030ba8dba906f756967f9e9ca394464a	hello.txt
040000 tree 5d90422423db5ef6b431e8b9e60e0baf04b8742a	src
```

これまでと同じく，テストリスト，図，実装の順に進める．
