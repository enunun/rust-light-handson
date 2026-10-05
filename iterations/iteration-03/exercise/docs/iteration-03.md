# Iteration 3：`cat-file`

このIterationでは，オブジェクトのファイルを展開してヘッダーを解析し，`rgit cat-file`で種類，大きさ，内容を表示する．
短縮形のIDからオブジェクトを探す処理も作る．

## 3-1 準備

このディレクトリ(`iterations/iteration-03/exercise`)に移動し，`cargo test`で，Iteration 2から引き継いだテストがすべて通ることを確かめる．
このIterationでは，依存を追加しない．

## 3-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-03.md)：`Read`トレイト，スライスの操作，クロージャ，借用を返す関数とライフタイムの省略，`Option`と`Result`の変換，ディレクトリの読み取り，clapのグループ
- [Gitのノート](../../../../docs/git/iteration-03.md)：オブジェクトを読む手順，`git cat-file`，短縮形の解決

読み終えたら，`src/lib.rs`の末尾に`#[cfg(test)] mod practice`を作り，次の課題を確かめる．

1. `b"blob 6\0hello\n"`の最初の0の位置を`iter().position`で求め，`split_at`で前と後ろに分ける．
2. `fn after_nul(data: &[u8]) -> Option<&[u8]>`を書く．0の後ろのスライスを返し，0がなければ`None`を返す．`?`を`Option`に使う．
3. ノートの`content`関数(関数の中の`Vec`を借りて返す)を書き，コンパイルエラーを読む．`content`の戻り値を`Vec<u8>`にして直す．
4. `"x".parse::<u32>()`の結果を`map_err`で`Err("bad")`に，`None::<u32>`を`ok_or`で`Err("none")`にする．

確かめ終えたら，`mod practice`を消す．

## 3-3 テストリスト

### 要件

- `rgit cat-file (-t | -s | -p) <object>`は，オブジェクトの種類，大きさ，内容を出力する．
  - `-t`と`-s`は，blob，tree，commitのすべてに使える．
  - `-p`は内容をそのまま出力する．blobとcommitは，本物の`git cat-file -p`と同じ出力になる．treeの内容を読みやすく整える表示は，Iteration 4で作る．
- オブジェクトは，4桁以上40桁以下の16進数で指定できる．
  - 一致するオブジェクトがなければ`Not a valid object name <object>`のエラーにする．
  - 2つ以上一致すれば`short object ID <object> is ambiguous`のエラーにする．
- ヘッダーが壊れている，または大きさが内容と合わなければ，`corrupt object: <理由>`のエラーにする．

### 使用例

```console
$ rgit cat-file -t ce01362
blob
$ rgit cat-file -s ce01362
6
$ rgit cat-file -p ce01362
hello
```

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `object` | `pub enum ObjectKind { Blob, Tree, Commit }`(`Display`と`FromStr`)，`pub fn parse_header(data: &[u8]) -> Result<(ObjectKind, &[u8]), Error>`．`blob_bytes`を，種類も引数で受け取る`encode`にする |
| `repo` | `Repository::read_object`(種類と内容を返す)，`Repository::resolve_prefix` |
| `error` | 見つからない，曖昧，壊れているオブジェクトのエラー |
| `cli` | `cat-file`のサブコマンド．`-t`，`-s`，`-p`はどれか1つだけを指定できる |

### 書くときに考えること

- ヘッダーの壊れ方にはどんなものがあるか．すべてを項目にしなくてよいが，大きさの不一致は必ず確かめる．
- 短縮形の境界(3桁，4桁，40桁)と，大文字の扱いを考える．
- 曖昧な短縮形のテストには，先頭の4桁が同じ2つのオブジェクトが要る．`195\n`と`389\n`のblobは，どちらも`6bb2`で始まる．
- commitのオブジェクトは，まだ`rgit`で作れない．結合テストでは本物の`git`で作る．`git write-tree`が空のツリーを作り，`git commit-tree`がそれをコミットにする．`git commit-tree`には，`-c user.name=…`と`-c user.email=…`で作者を渡す．

## 3-4 図

`design/types.md`を，このIterationの終わりの状態に更新する．

- `object`に置く新しい型は何か．`object_mod`の公開関数はどう変わるか．
- `Repository`に加わるメソッドと，その戻り値の型を書く．タプルを含む戻り値の書き方は，[図の書き方](../../../../docs/design.md)を読む．
- `-t`，`-s`，`-p`をまとめた構造体を，`cli`に描く．
- `Error`に加わる列挙子を書く．

## 3-5 テスト駆動の実装

### 実装のヒント

- `ObjectKind`の`Display`は，`match self`で名前を選んで`f.write_str`に渡す．`FromStr`は，`match s`で文字列のリテラルと比べる．
- `encode`の`format!`の中で`{kind}`と書けば，`ObjectKind`の`Display`が使われる．
- `parse_header`は，`position`でNULを探し，`split_at`で分ける．NULの後ろの内容は`&rest[1..]`である．`ok_or`と`map_err`で`Option`やほかのエラーを`Error`に変え，`?`で返す．
- `read_object`では，ファイルがないことを`io::ErrorKind::NotFound`で見分け，`ObjectNotFound`に変える．
- `parse_header`が返す内容は，`read_object`の中で作った`Vec`を借りている．`read_object`から返すには`to_vec()`で複製する．
- `resolve_prefix`では，先頭2桁のディレクトリを`fs::read_dir`で読み，残りの桁で始まるファイル名を集める．集めた数で結果を分ける．
- `cat-file`の3つのフラグは，`#[derive(Args)]`と`#[group(required = true, multiple = false)]`を付けた構造体にまとめ，`#[command(flatten)]`で取り込む．
- `-p`の内容は`writeln!`ではなく`out.write_all`で書く．内容の末尾の改行は内容に含まれている．

## 3-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．
2. `parse_header`は内容を複製せずにスライスで返した．複製して`Vec<u8>`を返す設計と比べて，呼び出す側に何が求められるか．
3. エラーの理由を`CorruptObject(&'static str)`で持った．理由ごとに列挙子を分ける設計と比べて，テストの書き方はどう変わるか．
4. 本物の`git`が書いたオブジェクトを`rgit`で読むテストと，`rgit`が書いたオブジェクトを`rgit`で読むテストは，それぞれ何を保証するか．
5. 図と実装を見比べ，違うところがあれば図を直す．

## 3-7 発展課題

本物の`git`は，曖昧な短縮形の候補を一覧にする．
`rgit`のエラーメッセージにも，候補の短縮形をIDの順に加える．

```text
fatal: short object ID 6bb2 is ambiguous
hint: The candidates are:
hint:   6bb2f4e
hint:   6bb2f98
```

これまでと同じく，テストリスト，図，実装の順に進める．
