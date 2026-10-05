# Iteration 11：`add`と`status`の並列化

このIterationでは，オブジェクトストアを複数のスレッドから同時に書き込める形に変える．
そのうえで，要素ごとの処理を複数のスレッドで行う関数を作り，`rgit add`と`rgit status`のファイルの読み込みとハッシュの計算を並列にする．

## 11-1 準備

このディレクトリ(`iterations/iteration-11/exercise`)に移動し，`cargo test`で，Iteration 10から引き継いだテストがすべて通ることを確かめる．

## 11-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-11.md)：スレッドを作る，`thread::scope`，`Arc`との比較，`Send`と`Sync`，親トレイト，`Mutex`と内部可変性，チャネル，アトミック変数，クロージャのトレイトと`where`，スレッドの数，スレッドの数と速さ
- [Gitのノート](../../../../docs/git/iteration-11.md)：並列にできる処理，同じオブジェクトを同時に書く，一時ファイルと名前の変更，`git fsck`

読み終えたら，`src/lib.rs`の末尾に`#[cfg(test)] mod practice`を作り，次の課題を確かめる．

1. 1から10までの`Vec`を前半と後半に分け，`thread::scope`の2つのスレッドでそれぞれの合計を求める．
2. `thread::spawn`に，関数の`Vec`を借りるクロージャを渡し，コンパイルエラーを確かめる．`move`を付けて直す．
3. `Mutex<i32>`を4つのスレッドで共有し，それぞれ100回ずつ1を足す．結果が400になることを確かめる．
4. 3を`thread::spawn`と`Arc`で書く．
5. 3の`Mutex`を`RefCell`に変え，コンパイルエラーを確かめる(確かめたら戻す)．
6. 3つのスレッドからチャネルで数を送り，受信側で集める．送信側を`drop`しないとどうなるかも確かめる．
7. `AtomicUsize`の`fetch_add`で，4つのスレッドに0から99までの番号を1回ずつ配る．
8. `thread::available_parallelism()`の値を確かめる．

確かめ終えたら，`mod practice`を消す．

## 11-3 テストリスト

### 要件

- `ObjectStore`の書き込みは`&self`で行い，トレイトに`Send + Sync`を求める(リファクタリング)．
  - `MemoryObjectStore`は`Mutex`で中身を守る．
  - `LooseObjectStore`は一時ファイルに書いてから名前を変え，同じオブジェクトを同時に書いても壊れないようにする．
  - `Repository`の`objects_mut`を消し，`add`と`commit`は`&self`で呼べるようにする．
- 要素ごとの処理を，指定した数までのスレッドで行う`map_parallel`を作る．結果は要素と同じ順に並び，スレッドの数によらず同じである．
- `rgit add`と`rgit status`は，ファイルの読み込みとハッシュの計算を複数のスレッドで行う．
  - スレッドの数は`--jobs <N>`(`-j <N>`)で指定する．省略すれば`std::thread::available_parallelism`の値を使う．
  - `rgit diff`は，作業ディレクトリのハッシュを`available_parallelism`の数のスレッドで計算する．
- 結果は，これまでと同じく本物の`git`と一致する．

### 使用例

```console
$ rgit add --jobs 8 .
$ rgit status --jobs 8
A  hello.txt
A  src/main.rs
```

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `store` | `ObjectStore`に`Send + Sync`を求め，`write`を`&self`にする．`MemoryObjectStore`は`Mutex<HashMap<…>>`を持つ．`LooseObjectStore`は一時ファイルに書いてから名前を変える |
| `parallel` | `pub fn map_parallel<T, R, F>(items: &[T], jobs: usize, f: F) -> Vec<R>`，`pub fn available_jobs() -> usize` |
| `repo` | `objects_mut`を消す．`add`はスレッドの数を受け取り，ファイルごとの処理を`map_parallel`で行う．`commit`は`&self`になる |
| `status` | `status`と`work_tree_files`はスレッドの数を受け取り，ハッシュの計算を`map_parallel`で行う |
| `patch` | `diff_work_tree`はスレッドの数を受け取る |
| `cli` | `add`と`status`の`--jobs` |

### 書くときに考えること

- リファクタリングでは，`&mut`で書き込んでいたテストを`&`に変える．振る舞いは変わらない．
- `map_parallel`のテストは，要素の順に結果が並ぶこと，スレッドの数が1，要素より多い，0の場合，空の要素，実際に複数のスレッドで動くことを確かめる．
- オブジェクトストアは，複数のスレッドから同じオブジェクトを同時に書いても，すべて成功し，読めることを確かめる．一時ファイルが残らないことも確かめる．
- 結合テストは，スレッドの数を変えても，`add`のあとのインデックスと`status`の出力が本物の`git`と一致することを確かめる．並列に書いたオブジェクトは`git fsck`で調べられる．

## 11-4 図

`design/types.md`を，このIterationの終わりの状態に更新する．

- `ObjectStore`の`write`から`&mut self`を消し，図の下に親トレイト`Send + Sync`を書く．`MemoryObjectStore`のフィールドを`Mutex`にする．
- `Repository`から`objects_mut`を消し，`add`と`commit`の`&mut self`を消す．
- 新しいモジュール`parallel`の名前空間を加え，使う側からの依存を描く．
- `add`と`status`の`--jobs`を表す型を`cli`に加える．

## 11-5 テスト駆動の実装

### 進め方

1. `ObjectStore::write`を`&self`に変え，コンパイルエラーを1つずつ直す．`MemoryObjectStore`は`Mutex`を使う．各段階で`cargo test`がすべて通ることを確かめる．
2. `LooseObjectStore`の書き込みを，一時ファイルと名前の変更に変える．同時に書くテストを先に書く．
3. `map_parallel`を，テストリストの順に作る．
4. `Repository::add`と`work_tree_files`を，`map_parallel`を使う形に変える．
5. `cli`に`--jobs`を加え，結合テストで確かめる．

### 実装のヒント

- `ObjectStore`に`Send + Sync`を求めると，`S: ObjectStore`の`&S`を複数のスレッドに渡せる．
- `map_parallel`は，`thread::scope`の中で`jobs`個のスレッドを作る．各スレッドは，`AtomicUsize`の`fetch_add`で次の要素の番号を受け取り，`(番号, 結果)`をチャネルで送る．受信側は，番号の位置に結果を置く．
- 結果を置く場所は，`Vec<Option<R>>`にしておくと，最後に`Option`を外して`Vec<R>`にできる．
- `thread::scope`の中で受信するなら，すべてのスレッドを作ったあとに元の送信側を`drop`する．
- スレッドの数は，`jobs.clamp(1, items.len().max(1))`で，1以上かつ要素の数以下にする．
- 一時ファイルの名前には，`std::process::id()`と，`static`の`AtomicUsize`で数えた番号を使う．
- `add`と`status`で同じ`--jobs`を使うには，clapの`Args`を導出した構造体を作り，`#[command(flatten)]`で両方のサブコマンドに入れる．

### 速度を測る

テストがすべて通ったら，スレッドの数を変えて`add`と`status`の速さを測る．

1. `cargo build --release`で最適化したビルドを作り，パスを`RGIT`に入れる．
2. Iteration 9と同じく，2000個のファイルを持つリポジトリ`/tmp/many`を作り，`git`でコミットする．
3. `status`を，スレッドの数を1，2，4に変えて測る：`hyperfine -N --warmup 3 -L jobs 1,2,4 "$RGIT status --jobs {jobs}"`
4. `add`を，測るたびにオブジェクトとインデックスを消してから測る：`hyperfine -N --runs 5 -L jobs 1,2,4 --prepare 'rm -rf .git/objects .git/index' "$RGIT add --jobs {jobs} ."`
   - `rgit`は，すでにあるオブジェクトを書き直さないので，消してから測る．
   - このあと，リポジトリはコミットのオブジェクトを失う．ほかの確かめには使わない．

5. スレッドが4つのときの速くなった倍率から，並列にできている部分の割合をアムダールの法則で求める．
6. `User`の時間(すべてのスレッドのCPUの時間の合計)が，スレッドの数でどう変わるかを見る．

## 11-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．
2. `map_parallel`は，次の要素を`AtomicUsize`で配る．要素をあらかじめ`jobs`個のまとまりに分けて各スレッドに渡す方法と比べて，ファイルの大きさが偏っているときにどちらが速いか．
3. `ObjectStore::write`を`&self`にした．`&mut self`のまま，スレッドごとに別のオブジェクトストアを持たせる方法と比べる．
4. `MemoryObjectStore`は1つの`Mutex`で`HashMap`全体を守る．多くのスレッドが同時に書くと何が起きるか．
5. 測った結果で，並列にしていない処理は何で，どれだけの時間を使っているか．8つのCPUを持つ計算機で`--jobs 8`にすると，何倍速くなると見込めるか．
6. 図と実装を見比べ，違うところがあれば図を直す．

## 11-7 発展課題

### エラーで止める

模範解答の`add`は，1つのファイルの読み込みに失敗しても，残りのファイルをすべて処理してからエラーを返す．
`f`が`Result`を返し，どれかが`Err`になったら，まだ始めていない要素を処理せずに`Err`を返す関数を作る．

```rust
pub fn try_map_parallel<T, R, E, F>(items: &[T], jobs: usize, f: F) -> Result<Vec<R>, E>
where
    T: Sync,
    R: Send,
    E: Send,
    F: Fn(&T) -> Result<R, E> + Sync,
```

`f`を呼んだ回数を`AtomicUsize`で数え，エラーのあとに残りの要素を処理していないことを単体テストで確かめる．
作った関数で，`Repository::add`と`work_tree_files`を書き直す．

### ファイルの状態でハッシュの計算を省く

Iteration 9で測ったとおり，`git status`は`rgit status`よりずっと速い．
`git`は，インデックスに記録したファイルの状態(大きさ，更新時刻など)が今のファイルと同じなら，ファイルを読まない([Iteration 9のGitのノート](../../../../docs/git/iteration-09.md))．
`rgit`の`work_tree_files`も，同じようにハッシュの計算を省く．

- `fs::metadata`から作った`Stat`が，インデックスのエントリーの`stat`と同じなら，エントリーのIDを使う．
- ただし，更新時刻がインデックスのファイルの更新時刻と同じか後のファイル(racyなファイル)は，読んでハッシュを計算する．インデックスを書いたのと同じ時刻のうちに，大きさを変えずに書き換えられた場合を見分けられないからである．
- インデックスのファイルの更新時刻は，`cli`で`fs::metadata`から求め，`status`と`work_tree_files`に渡す．インデックスがなければ`(0, 0)`とする．

単体テストでは，中身と違うIDを今のファイルの`Stat`とともにインデックスに入れる．
インデックスの時刻が十分に後なら記録したIDが使われて変更なしになり，`(0, 0)`ならファイルを読んで変更あり(`AM`)になることを確かめる．
結合テストが`git status`と一致したままであることを確かめてから，Iteration 9と同じく`hyperfine`で`git status --porcelain`と比べる．

これまでと同じく，テストリスト，図，実装の順に進める．
