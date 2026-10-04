# Iteration 8：`log`

このIterationでは，コミットのグラフを時刻の新しい順にたどるイテレーター`RevWalk`を作り，`rgit log`で履歴を表示する．
`HEAD~1`のように，最初の親をたどる指定も受け付ける．

## 8-1 準備

このディレクトリ(`iterations/iteration-08/exercise`)に移動し，`cargo test`で，Iteration 7から引き継いだテストがすべて通ることを確かめる．

## 8-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-08.md)：`Iterator`トレイトの実装，失敗しうる要素，遅延評価，参照を持つ構造体，`BinaryHeap`，`HashSet`，`Ord`と`Hash`の導出，`let … else`
- [Gitのノート](../../../../docs/git/iteration-08.md)：コミットのグラフ，`git log`の順序，`~N`による指定

読み終えたら，`src/lib.rs`の末尾に`#[cfg(test)] mod practice`を作り，次の課題を確かめる．

1. 3から1まで数えるイテレーター`Countdown`を書き，`collect`で`[3, 2, 1]`になることを確かめる．
2. `Countdown`に`map(|n| n * 10)`と`filter`をつないで結果を確かめる．`take(2)`をつなぐと，`next`は何回呼ばれるか．`next`の中で呼ばれた回数を数えて確かめる．
3. `BinaryHeap`に`(時刻, 名前)`の組を3つ入れ，`pop`で時刻の新しい順に出てくることを確かめる．
4. `HashSet`の`insert`が，2回目に`false`を返すことを確かめる．
5. `let Some((a, b)) = "x~2".split_once('~') else { panic!() };`の形で，`~`の前後を取り出す．

確かめ終えたら，`mod practice`を消す．

## 8-3 テストリスト

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

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `revwalk` | `pub struct RevWalk<'r> { repo: &'r Repository, queue: BinaryHeap<(i64, ObjectId)>, seen: HashSet<ObjectId> }`，`RevWalk::new`，`impl Iterator for RevWalk<'_>`(`Item = Result<(ObjectId, Commit), Error>`) |
| `revision` | `~N`を解釈する |
| `repo` | commitオブジェクトを読む`Repository::read_commit` |
| `oid` | `ObjectId`に`PartialOrd`，`Ord`，`Hash`を導出する |
| `cli` | `log`のサブコマンド |

### 書くときに考えること

- 単体テストのコミットは，時刻を自由に決めて作れる．`Commit::builder()`でコミットを作り，`write_object`で書く補助関数をテストの中に作るとよい．treeには空のtree(`4b825dc…`)を使える．
- マージのある履歴では，時刻の順序と，共通の祖先が1回だけ出ることを確かめる．枝の時刻を入れ替えると，順序はどう変わるか．
- `~N`の境界を考える．`~0`，`~`(数を省略)，ルートを超える場合．
- 結合テストでコミットの時刻を変えるには，`GIT_AUTHOR_DATE`と`GIT_COMMITTER_DATE`を変えて`rgit`を実行する補助関数を，`tests/common/mod.rs`に加える．

## 8-4 図

`design/types.md`を，このIterationの終わりの状態に更新する．

- 新しいモジュール`revwalk`の名前空間に，`RevWalk~'r~`を描く．`Repository`への参照を持つことを，関連の矢印(`-->`)で描く．
- `RevWalk`が実装する標準のトレイトを書く．
- `Repository`と`ObjectId`に加わるものを書く．

## 8-5 テスト駆動の実装

### 実装のヒント

- `src/lib.rs`に`mod revwalk;`を加える．
- `RevWalk::new`は，最初のコミットを待ち行列に入れ，見たことにしておく．最初のコミットの時刻は，ほかのどのコミットよりも先に取り出されるように決める．
- `next`では，待ち行列から最新のコミットを取り出して読み，まだ見ていない親を，その時刻とともに待ち行列に入れる．
- `next`は`Option`を返すので，`Result`のエラーは`match`で`Some(Err(…))`にして返す．
- `ObjectId`を`BinaryHeap`のタプルと`HashSet`に入れるには，`Ord`と`Hash`が要る．導出を加える．
- `~N`は，`split_once('~')`で分け，前を参照名やIDとして解決したあと，`read_commit`で最初の親をN回たどる．
- `log`では，`RevWalk::new(…).take(N)`で件数を絞る．`-n`がなければ`usize::MAX`にする．

## 8-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．
2. `RevWalk`をイテレーターにせず，すべてのコミットを`Vec<Commit>`にして返す関数にした場合と比べる．`log -n 1`の速さと，使う側のコードはどう変わるか．
3. `RevWalk`は`Repository`を借りる．`Repository`を所有する設計と比べて，何ができなくなり，何ができるか．
4. 要素の型を`Result`にした．エラーがあったら`None`を返して終わる設計と比べて，使う側は何を知ることができるか．
5. 図と実装を見比べ，違うところがあれば図を直す．

## 8-7 発展課題

`git log --first-parent`は，マージのコミットで最初の親だけをたどる．
`rgit log --first-parent`を作る．`RevWalk`に，最初の親だけをたどる設定を加えるとよい．

これまでと同じく，テストリスト，図，実装の順に進める．
