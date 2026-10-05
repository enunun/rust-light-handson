# Iteration 4：ツリーの読み取りと`ls-tree`

このIterationでは，treeオブジェクトの内容をエントリーの列に解析し，`rgit ls-tree`と`rgit cat-file -p`で表示する．
エントリーの名前を複製せず，元のバイト列を借りたまま持つ型を作る．

## 4-1 準備

このディレクトリ(`iterations/iteration-04/exercise`)に移動し，`cargo test`で，Iteration 3から引き継いだテストがすべて通ることを確かめる．

## 4-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-04.md)：参照を持つ構造体，ライフタイム注釈，借用する設計と所有する設計，RustOwlでライフタイムを見る，`TryFrom`，スライスから配列へ，`while`
- [Gitのノート](../../../../docs/git/iteration-04.md)：treeオブジェクト，`git ls-tree`，ファイルのモード，treeの内容のバイト列

読み終えたら，`src/lib.rs`の末尾に`#[cfg(test)] mod practice`を作り，次の課題を確かめる．

1. `struct Word<'a> { text: &'a str }`と，文字列の最初の空白までを`Word`で返す`fn first_word(s: &str) -> Word<'_>`を書く．`"100644 hello.txt"`から`"100644"`を得る．`str::find`を調べて使う．
2. 課題1のテストで，元の`String`を内側の`{ }`の中で作り，`Word`を外で使ってみる．どんなエラーになるか．
3. `enum Bit { Zero, One }`に`TryFrom<u8>`を実装する．0と1以外は`Err(String)`にする．`Bit::try_from(1)`と`0u8.try_into()`を確かめる．
4. 5バイトの配列の先頭4バイトを`[u8; 4]`に`try_into`で変換する．先頭3バイトでは失敗することを確かめる．

確かめ終えたら，`mod practice`を消す．

## 4-3 テストリスト

### 要件

- treeオブジェクトの内容を解析する．
  - 内容は`<モード> <名前>\0<20バイトのID>`の並びである．
  - モードは`100644`(通常のファイル)，`100755`(実行可能なファイル)，`120000`(シンボリックリンク)，`40000`(ディレクトリ)，`160000`(サブモジュール)のどれかである．
  - 名前はUTF-8でなければエラーにする．
- `rgit cat-file -p <tree>`と`rgit ls-tree <tree>`は，エントリーを`<6桁のモード> <種類> <ID>\t<名前>`の形で1行ずつ出力する．
- `rgit ls-tree`にtreeでないオブジェクトを指定すると，`not a tree object`のエラーにする．

### 使用例

```console
$ rgit ls-tree aae2b36
100644 blob ce013625030ba8dba906f756967f9e9ca394464a	hello.txt
040000 tree 5d90422423db5ef6b431e8b9e60e0baf04b8742a	src
```

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `tree` | `pub enum Mode`(`TryFrom<&[u8]>`)，`pub struct TreeEntry<'a> { mode, name: &'a str, id }`，`pub fn parse_tree(data: &[u8]) -> Result<Vec<TreeEntry<'_>>, Error>` |
| `cli` | `ls-tree`のサブコマンド．`cat-file -p`でtreeを扱う |
| `error` | treeでないオブジェクトのエラー |

### 書くときに考えること

- 単体テストでtreeの内容のバイト列を作るには，IDを20バイトの値にする必要がある．テストの中に，エントリーのバイト列を作る補助関数を書くとよい．
- ディレクトリのモードは，treeの中の表記と表示の形が違う．両方を確かめる．
- 壊れたtreeには，どんな形があるか．IDの途中で終わっている場合を必ず確かめる．
- 結合テストでは，本物の`git`でtreeを作る．`git add`と`git write-tree`でtreeができる．実行可能なファイルやシンボリックリンクは，`git update-index --add --cacheinfo 100755,<blobのID>,run.sh`のようにインデックスに直接登録できる．

## 4-4 図

`design/types.md`を，このIterationの終わりの状態に更新する．

- 新しいモジュール`tree`の名前空間を加え，`Mode`，`TreeEntry`，`parse_tree`を書く．ライフタイム引数を持つクラスは`TreeEntry~'a~`と書ける．
- `TreeEntry`がフィールドに持つ型への関係を描く．`Mode`は`ObjectKind`を使うか．
- `cli`から`tree`への依存を描く．`Error`の列挙子を加える．

## 4-5 テスト駆動の実装

### 実装のヒント

- `src/lib.rs`に`mod tree;`を加え，`src/tree.rs`を作る．
- `Mode::try_from`は，`match`でバイト列のリテラル(`b"100644"`)と比べる．`Display`は6桁の表記を書く．
- `Mode`から`ObjectKind`を返すメソッド`kind`を作ると，表示の`blob`や`tree`を書ける．
- `parse_tree`は，`rest`という可変のスライスを先頭から読み進める．空白までがモード，NULまでが名前，その後ろの20バイトがIDである．
- IDの20バイトは`split_at(20)`で切り出し，`try_into()`で`[u8; 20]`にする．先に長さを調べないと，`split_at`がパニックになる．
- `cat-file -p`と`ls-tree`で同じ表示をするので，エントリーを書く補助関数を1つ作る．
- 実装したら，VS CodeでRustOwlを使い，`parse_tree`の`name`や，`cli.rs`の`content`の生きている範囲を確かめる．

## 4-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．
2. `TreeEntry`の`name`を`String`にしたら，コードとテストはどう変わるか．どちらの設計が`rgit`の今の使い方に合っているか．
3. `Mode`の表示(`040000`)とtreeの中の表記(`40000`)が違う．その違いを型のどこに閉じ込めたか．
4. `parse_tree`のテストでは，バイト列を手で組み立てた．本物の`git`が作ったtreeを使う方法と比べて，何が確かめやすいか．
5. 図と実装を見比べ，違うところがあれば図を直す．

## 4-7 発展課題

`git ls-tree -r`は，サブディレクトリのtreeをたどり，blobだけを`<ディレクトリ>/<名前>`のパスで表示する．
`rgit ls-tree -r`を作る．

```console
$ git ls-tree -r e5e07f5
100644 blob ce013625030ba8dba906f756967f9e9ca394464a	hello.txt
120000 blob ce013625030ba8dba906f756967f9e9ca394464a	link
100755 blob ce013625030ba8dba906f756967f9e9ca394464a	run.sh
100644 blob f328e4d9d04c31d0d70d16d21a07d1613be9d577	src/main.rs
```

これまでと同じく，テストリスト，図，実装の順に進める．
