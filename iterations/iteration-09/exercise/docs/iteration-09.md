# Iteration 9：オブジェクトストアの抽象化と`status`

このIterationでは，オブジェクトの読み書きをトレイト`ObjectStore`にまとめ，ディスクに書く実装とメモリーに持つ実装を作る．
treeを書く処理とコミットのグラフをたどる処理を，トレイトを使うジェネリックな関数に移し，メモリーの実装で単体テストを書けるようにする．
そのうえで，HEAD，インデックス，作業ディレクトリを比べる`rgit status`を作る．

## 9-1 準備

このディレクトリ(`iterations/iteration-09/exercise`)に移動し，`cargo test`で，Iteration 8から引き継いだテストがすべて通ることを確かめる．

## 9-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-09.md)：トレイトの定義と実装，何をトレイトにするか，トレイト境界，`?Sized`，トレイトオブジェクト，テストのための差し替え，2つの表の突き合わせ，テストの準備をまとめる構造体
- [Gitのノート](../../../../docs/git/iteration-09.md)：3つの状態，`git status --porcelain`，速さの工夫

読み終えたら，`src/lib.rs`の末尾に`#[cfg(test)] mod practice`を作り，次の課題を確かめる．

1. 文字列の何かを数えるトレイト`Counter`(`fn count(&self, text: &str) -> usize`)を定義し，文字の数を数える`Chars`と，単語の数を数える`Words`に実装する．
2. `Counter`を実装した任意の型で，文字列の配列の合計を数える`fn total<C: Counter + ?Sized>(counter: &C, texts: &[&str]) -> usize`を書く．`&dyn Counter`も渡せることを確かめる．
3. `Vec<Box<dyn Counter>>`に`Chars`と`Words`を入れ，`"a b"`をそれぞれで数える．
4. `#[derive(Default)]`を付けた構造体の既定値を確かめる．
5. 2つの`BTreeMap`のキーを`chain`でつないで`BTreeSet`に集め，重なりのない順序付きのキーになることを確かめる．`Option::map_or`も確かめる．

確かめ終えたら，`mod practice`を消す．

## 9-3 テストリスト

### 要件

- オブジェクトの読み書きをトレイト`ObjectStore`にまとめる(リファクタリング)．
  - ディスクのゆるいオブジェクトを読み書きする`LooseObjectStore`と，メモリーに持つ`MemoryObjectStore`の2つを実装する．
  - ツリーを書く関数，`RevWalk`，`status`はオブジェクトストアを型引数に取る．単体テストは`MemoryObjectStore`で書く．
- `rgit status`は，HEADのツリー，インデックス，作業ディレクトリを比べ，変更のあるファイルを`<X><Y> <パス>`の形でパスの順に出力する．
  - `X`はHEADとインデックスの違いで，追加は`A`，変更は`M`，削除は`D`，同じなら空白である．
  - `Y`はインデックスと作業ディレクトリの違いで，変更は`M`，削除は`D`，同じなら空白である．作業ディレクトリのファイルはハッシュを計算して比べる．
  - そのあとに，インデックスにないファイルを`?? <パス>`の形でパスの順に出力する．
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

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `store` | `pub trait ObjectStore`(`read`，`write(&mut self, …)`，`find`)，`pub struct LooseObjectStore`，`pub struct MemoryObjectStore` |
| `repo` | `Repository`はオブジェクトストアとして`LooseObjectStore`を持ち，`objects`と`objects_mut`で貸す．オブジェクトを読み書きするメソッドは`store`に移す |
| `tree` | `Repository::write_tree`を，オブジェクトストアを受け取る`pub fn write_tree<S: ObjectStore + ?Sized>(store: &mut S, index: &Index)`に移す．`pub fn flatten_tree<S: ObjectStore + ?Sized>(store: &S, id: ObjectId) -> Result<BTreeMap<String, (Mode, ObjectId)>, Error>` |
| `commit` | `Repository::read_commit`を，オブジェクトストアを受け取る`pub fn read_commit`に移す．`RevWalk`は`RevWalk<'s, S>`になる |
| `status` | `pub enum Change`，`pub enum StatusEntry`，`pub fn status<S: ObjectStore + ?Sized>(…) -> Result<Vec<StatusEntry>, Error>` |
| `worktree` | ファイルのモードを決める`pub fn file_mode` |
| `cli` | `status`のサブコマンド |

### 書くときに考えること

- リファクタリングは，振る舞いを変えない．引き継いだテストのうち，移すもの(`Repository`のオブジェクトの読み書き)と，書き方だけを変えるもの(`MemoryObjectStore`を使う形)を，テストリストに書く．
- 2つのオブジェクトストアに同じ確認をするには，`&mut dyn ObjectStore`を受け取る確認の関数を1つ作り，両方で呼ぶとよい．
- `status`の`X`と`Y`の組み合わせを，本物の`git`で試して集める．ノートの例が参考になる．
- `status`の単体テストには，作業ディレクトリ，インデックス，`MemoryObjectStore`，HEADのコミットが要る．準備をまとめる構造体をテストの中に作るとよい．

## 9-4 図

`design/types.md`を，このIterationの終わりの状態に更新する．

- 新しいモジュール`store`の名前空間に，トレイト`ObjectStore`(`<<trait>>`)と2つの実装を描き，実装の矢印(`..|>`)でつなぐ．
- `Repository`から消えるメソッドと，加わるフィールドとメソッドを書く．
- `tree_mod`，`commit_mod`，`RevWalk`は，`Repository`ではなく`ObjectStore`を使う．矢印を描き直す．
- 新しいモジュール`status`の名前空間を加える．

## 9-5 テスト駆動の実装

### リファクタリングの進め方

1. `src/store.rs`にトレイトと`MemoryObjectStore`を作り，`store`のテストを書いて通す．
2. `Repository`の`write_object`，`read_object`，`object_path`の中身を`LooseObjectStore`に移し，`store`のテストを`LooseObjectStore`でも通す．
3. `Repository`に`LooseObjectStore`のフィールドと`objects`，`objects_mut`を加え，`Repository`のオブジェクトのメソッドを消す．コンパイルエラーになった呼び出しを，1つずつ`repo.objects()`などに直す．
4. `write_tree`，`read_commit`，`RevWalk`を，オブジェクトストアを受け取る形に移す．単体テストを`MemoryObjectStore`で書き直す．
5. 各段階で`cargo test`を実行し，すべてのテストが通ったままであることを確かめる．

### 実装のヒント

- `ObjectStore::write`は`&mut self`を受け取る．書き込む`Repository`のメソッド(`add`，`commit`)も`&mut self`になり，`cli`では`let mut repo`で持つ．
- `resolve_prefix`の`fs::read_dir`の部分は，`ObjectStore::find`にする．桁数と文字の検査は`Repository`に残す．
- `MemoryObjectStore`には`#[derive(Default)]`を付けると，`MemoryObjectStore::default()`で作れる．
- `lib.rs`で公開しないと，テストでしか作らない`MemoryObjectStore`は「使われていない」と警告される．
- `status`では，3つの状態を`BTreeMap<String, (Mode, ObjectId)>`にそろえてから比べる．作業ディレクトリのファイルは`list_files`で集め，`hash_blob`でIDにする．
- `Change`の文字と，`StatusEntry`の`Display`で，`git status --porcelain`の形を作る．

## 9-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．
2. `ObjectStore`に`write_tree`や`read_commit`をメソッドとして加える設計と比べる．新しいオブジェクトストアを作る人の手間はどう変わるか．
3. `write_tree`をジェネリックな関数にした．`&mut dyn ObjectStore`を受け取る関数にした場合と比べて，何が変わり，何が変わらないか．
4. `status`の単体テストを`MemoryObjectStore`で書いた．`LooseObjectStore`で書く場合と比べて，何が楽になったか．逆に，何が確かめられなくなったか．
5. 図と実装を見比べ，違うところがあれば図を直す．

## 9-7 発展課題

ほかのオブジェクトストアを包み，書き込んだ回数を数える`CountingStore<S>`を作る．
`write_tree`が，ディレクトリの数だけtreeを書く(同じ内容のtreeは数えない)ことを，単体テストで確かめる．

```rust
let mut store = CountingStore::new(MemoryObjectStore::default());
write_tree(&mut store, &index).unwrap();
assert_eq!(store.writes(), 3);
```

これまでと同じく，テストリスト，図，実装の順に進める．
