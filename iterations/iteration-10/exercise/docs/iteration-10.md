# Iteration 10：`diff`

このIterationでは，2つの列の最短の編集を求める関数を，要素の型を問わないジェネリックな関数として作る．
その関数で行の差分を求め，`git diff`と同じunified形式で表示する`rgit diff`と`rgit diff --cached`を作る．

## 10-1 準備

このディレクトリ(`iterations/iteration-10/exercise`)に移動し，`cargo test`で，Iteration 9から引き継いだテストがすべて通ることを確かめる．

## 10-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-10.md)：要素の型を問わない関数，操作を表す`enum`，`matches!`，`usize`と`isize`，`unreachable!`，文字列を行に分ける，文字列を組み立てる，タプル構造体の`Display`，`chunk_by`，`Option::transpose`，ベンチマークのグループ
- [Gitのノート](../../../../docs/git/iteration-10.md)：`git diff`が比べるもの，最短の編集，編集グラフとMyersのアルゴリズム，計算とメモリーの量，unified形式

読み終えたら，`src/lib.rs`の末尾に`#[cfg(test)] mod practice`を作り，次の課題を確かめる．

1. 2つのスライスの同じ位置の要素が等しい数を数える`fn count_equal<T: PartialEq>(a: &[T], b: &[T]) -> usize`を書き，数と文字列で確かめる．`: PartialEq`を消すと，どんなコンパイルエラーになるかも確かめる．
2. 中身を持つバリアントと持たないバリアントの`enum`を作り，その配列から，あるバリアントの数を`matches!`で数える．
3. `usize`の`2`と`5`について，`isize`に変えた差，`saturating_sub`，`checked_sub`の結果を確かめる．
4. `"a\nb\n"`と`"a\nb"`を，`split_inclusive('\n')`と`lines()`で分けて比べる．
5. 空の`String`に，`write!`と`writeln!`で書き足す．
6. `[1, 2, 3, 7, 8, 10]`を，`chunk_by`で連続する数のまとまりに分ける．
7. `Option<&str>`を受け取り，`Some`なら数として読む`fn parse_optional(text: Option<&str>) -> Result<Option<i32>, std::num::ParseIntError>`を，`transpose`で書く．

確かめ終えたら，`mod practice`を消す．

## 10-3 テストリスト

### 要件

- 2つの列の最短の編集(一致，削除，挿入の列)を，Myersのアルゴリズムで求める．要素の型は，`==`で比べられるものなら何でもよい．
- 編集の列から，変更の前後3つの文脈を含むハンクを作る．間の一致が6つ以下の変更は，1つのハンクにまとめる．
- `rgit diff`はインデックスと作業ディレクトリの，`rgit diff --cached`はHEADとインデックスの差分を，unified形式でパスの順に出力する．
  - ファイルごとに`diff --git a/<パス> b/<パス>`，`index <短縮ID>..<短縮ID> <モード>`，`--- a/<パス>`，`+++ b/<パス>`を出力する．
  - 追加されたファイルは`new file mode <モード>`を，削除されたファイルは`deleted file mode <モード>`を出力し，ない側を`/dev/null`とする．
  - モードが変わったファイルは`old mode <モード>`と`new mode <モード>`を出力する．中身が同じなら，`index`の行とハンクを出力しない．
  - ハンクの見出しは`@@ -<開始>,<行数> +<開始>,<行数> @@`で，行数が1なら省略する．
  - 末尾に改行のないファイルは，最後の行の後ろに`\ No newline at end of file`を出力する．
  - NULを含むファイルは`Binary files a/<パス> and b/<パス> differ`と出力する．
- 結果は，本物の`git diff`と一致する．ただし，ハンクの見出しの後ろの関数名は出力しない．同じ長さの編集が複数あるときは，`git`と違う編集を選ぶことがある．

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

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `diff` | `pub enum Edit`，`pub fn diff<T: PartialEq>(a: &[T], b: &[T]) -> Vec<Edit>`，`pub struct Hunk`(見出しの`Display`を持つ)，`pub fn hunks(edits: &[Edit], context: usize) -> Vec<Hunk>`．`lib.rs`で公開モジュールにする |
| `patch` | `pub struct FileVersion`(モード，ID，中身)，1つのファイルの差分を文字列にする`pub fn file_patch`，`pub fn diff_work_tree`，`pub fn diff_cached` |
| `status` | HEAD，インデックス，作業ディレクトリの表を作る関数と`compare`を公開し，`patch`からも使う |
| `cli` | `diff`のサブコマンドと`--cached` |

### 書くときに考えること

- `diff`の単体テストは，ファイルを使わず，数や文字の列で書ける．編集の数が最短であることと，編集を当てはめると新しい列になることを確かめるとよい．
- ハンクの境界は，間の一致が`2 * context`個ちょうどの場合と，1つ多い場合の両方を試す．
- unified形式の細かい規則(空の範囲，追加，削除，モード，末尾の改行，バイナリー)は，本物の`git`で試して集める．ノートの例が参考になる．
- 結合テストは，いろいろな変更をまとめて加えた作業ディレクトリで，`git diff`と比べる．同じ長さの編集が複数ある変更は避ける．

## 10-4 図

`design/types.md`を，このIterationの終わりの状態に更新する．

- 新しいモジュール`diff`と`patch`の名前空間を加える．`diff`には`Edit`と`Hunk`を，`patch`には`FileVersion`を描く．
- `status_mod`に，公開した関数を書く．
- `patch_mod`から`diff_mod`，`status_mod`，`ObjectStore`への依存を描く．
- `Command`に`Diff`を加える．

## 10-5 テスト駆動の実装

### 進め方

1. `diff`：空の列，同じ列，片方が空の列から始め，削除と挿入が混ざる列に進む．
2. `hunks`：変更が1つの場合から始め，まとめる場合と分ける場合に進む．見出しの`Display`もここで作る．
3. `file_patch`：変更したファイルから始め，追加，削除，空のファイル，末尾の改行，モード，バイナリーの順に広げる．
4. `diff_work_tree`と`diff_cached`：`status`の表を作る部分を関数に分け，両方から使う．
5. `cli`：サブコマンドを加え，結合テストで`git diff`と比べる．

### 実装のヒント

- `diff`は，ノートの手順のとおり，`d`ごとに`v`を複製して記録しておく．最後に，終点から記録を逆にたどって編集を集め，`reverse`で始点からの順にする．
- 対角線の番号`k`は負にもなるので，`v`の添字には一定の数を足してずらす．`v`の長さは`2 * (n + m) + 3`あれば足りる．
- `hunks`では，まず`Equal`でない編集の位置を集め，`chunk_by`で近いものをまとめる．各まとまりの前後に文脈を付け，`saturating_sub`と`min`で範囲を列の中に収める．
- ハンクの古い側の開始の行は，ハンクより前にある`Equal`と`Delete`の数である．新しい側は，`Equal`と`Insert`を数える．
- 行は`split_inclusive('\n')`で分ける．行の末尾が`'\n'`でなければ，その行の後ろに`\ No newline at end of file`を書く．
- `index`の行のIDは`ObjectId::short`で7文字にする．ない側は`0000000`である．
- `git diff`は，インデックスにない作業ディレクトリのファイルを表示しない．

### 速度を測る

テストがすべて通ったら，`diff`の速さを測り，遅い部分を直す．

1. `Cargo.toml`に`[[bench]]`の`diff`を加え，`benches/diff.rs`を作る．
2. 数を1つずつ並べた行と，その行のいくつかを書き換えた行を作る関数を書く．
3. 2つのベンチマークのグループを作る．
   - `diff/length`：書き換える行を10個にし，行の数を1000，10000，100000と変える．
   - `diff/changes`：行の数を20000にし，書き換える行を10，100，1000個と変える．
4. `cargo bench --bench diff`で測る．
5. 10万行で10行を書き換えただけの`diff`に，何msかかるか．`diff`の中で，行の数に比例する仕事を探す．
6. 見つけた部分を直し，`cargo test`がすべて通ることを確かめてから，もう一度測る．criterionが表示する`change`で，どれだけ速くなったかを確かめる．

5で原因がわからなければ，次のヒントを読む．

- 各段階の初めに，長さ`2 * (n + m) + 3`の`v`を丸ごと複製している．
- 段階`d`で`v`から読むのは，対角線`-d - 1`から`d + 1`までの値だけである．
- その範囲だけを記録すると，`trace[d]`の先頭は対角線`-d - 1`になる．`backtrack`では，対角線`k`の添字が`slot(k, d)`になる．

## 10-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．
2. `diff`を`&[&str]`だけを受け取る関数にした場合と比べる．テストの書きやすさと，ほかの使い道はどう変わるか．
3. `Edit`が要素ではなく添字を持つ設計と，要素の参照を持つ設計(`Edit<'a, T>`)を比べる．
4. `file_patch`は`String`を返す．`impl io::Write`に直接書く設計と比べて，テストのしやすさはどう変わるか．
5. 直す前と直したあとで，`trace`の記録の量は，行の数と編集の長さにどう比例するか．中身をすべて書き換えた1万行のファイルの`diff`では，記録はどれだけになるか．
6. 図と実装を見比べ，違うところがあれば図を直す．

## 10-7 発展課題

文脈の行数を指定する`-U <行数>`(`--unified <行数>`)を，`rgit diff`に加える．省略すれば3行である．
本物の`git diff -U1`や`git diff -U0`と結果が一致することを，結合テストで確かめる．
文脈が少ないと，ハンクより前に英字で始まる行が残りやすく，`git`は見出しに関数名を付ける．比べるファイルは，数の行だけにするとよい．

次は，ノートの`numbers.txt`の例を，本物の`git`の`-U1`で表示したものである．

```console
$ git diff -U1
diff --git a/numbers.txt b/numbers.txt
index 0ff3bbb..8cdb13d 100644
--- a/numbers.txt
+++ b/numbers.txt
@@ -2,3 +2,3 @@
 2
-3
+three
 4
@@ -8,3 +8,3 @@
 8
-9
+nine
 10
@@ -20 +20,2 @@
 20
+21
```

これまでと同じく，テストリスト，図，実装の順に進める．
