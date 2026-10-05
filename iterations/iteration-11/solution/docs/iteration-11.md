# Iteration 11：`add`と`status`の並列化(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 11-1 準備

`Cargo.toml`は，Iteration 10の模範解答からパッケージ名だけを変えたものである．依存は変わらない．スレッド，`Mutex`，チャネル，アトミック変数は，どれも標準ライブラリにある．

## 11-2 文法と概念

課題の解答例である．

```rust
#[cfg(test)]
mod practice {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn scoped_threads_borrow_local_data() {
        let numbers: Vec<i32> = (1..=10).collect();
        let (left, right) = numbers.split_at(5);
        let (a, b) = thread::scope(|scope| {
            let a = scope.spawn(|| left.iter().sum::<i32>());
            let b = scope.spawn(|| right.iter().sum::<i32>());
            (a.join().unwrap(), b.join().unwrap())
        });
        assert_eq!(a + b, 55);
    }

    #[test]
    fn spawn_needs_owned_data() {
        let numbers: Vec<i32> = (1..=3).collect();
        let handle = thread::spawn(move || numbers.iter().sum::<i32>());
        assert_eq!(handle.join().unwrap(), 6);
    }

    #[test]
    fn mutex_counts_from_many_threads() {
        let count = Mutex::new(0);
        thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    for _ in 0..100 {
                        *count.lock().unwrap() += 1;
                    }
                });
            }
        });
        assert_eq!(*count.lock().unwrap(), 400);
    }

    #[test]
    fn arc_shares_ownership_with_spawned_threads() {
        let count = Arc::new(Mutex::new(0));
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let count = Arc::clone(&count);
                thread::spawn(move || *count.lock().unwrap() += 1)
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(*count.lock().unwrap(), 4);
    }

    #[test]
    fn channel_collects_results() {
        let (sender, receiver) = mpsc::channel();
        thread::scope(|scope| {
            for n in 0..3 {
                let sender = sender.clone();
                scope.spawn(move || sender.send(n * 10).unwrap());
            }
        });
        drop(sender);
        let mut received: Vec<i32> = receiver.iter().collect();
        received.sort();
        assert_eq!(received, [0, 10, 20]);
    }

    #[test]
    fn atomic_hands_out_each_number_once() {
        let next = AtomicUsize::new(0);
        let taken = Mutex::new(Vec::new());
        thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    for _ in 0..25 {
                        let n = next.fetch_add(1, Ordering::Relaxed);
                        taken.lock().unwrap().push(n);
                    }
                });
            }
        });
        let mut taken = taken.into_inner().unwrap();
        taken.sort();
        assert_eq!(taken, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn available_parallelism_is_positive() {
        let jobs = thread::available_parallelism().unwrap().get();
        assert!(jobs >= 1);
    }
}
```

- 2：`move`を付けないと，次のコンパイルエラーになる．コンパイラーは`move`を付ける直し方を示す．

```console
$ cargo test
error[E0373]: closure may outlive the current function, but it borrows `numbers`, which is owned by the current function
  --> src/lib.rs:32:36
   |
32 |         let handle = thread::spawn(|| numbers.iter().sum::<i32>());
   |                                    ^^ ------- `numbers` is borrowed here
   |                                    |
   |                                    may outlive borrowed value `numbers`
   |
note: function requires argument type to outlive `'static`
  --> src/lib.rs:32:22
   |
32 |         let handle = thread::spawn(|| numbers.iter().sum::<i32>());
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: to force the closure to take ownership of `numbers` (and any other referenced variables), use the `move` keyword
   |
32 |         let handle = thread::spawn(move || numbers.iter().sum::<i32>());
   |                                    ++++

For more information about this error, try `rustc --explain E0373`.
error: could not compile `rgit` (lib test) due to 1 previous error
```

- 3：`*count.lock().unwrap() += 1`の`MutexGuard`は，その文の終わりで片付けられ，ロックが外れる．ロックを持つ時間が短いので，ほかのスレッドを長く待たせない．
- 4：`Arc::clone(&count)`は，中の値を複製せず，参照の数を1つ増やす．`move`で各スレッドに1つずつ渡す．
- 5：`RefCell`に変えると，ノートと同じ`RefCell<i32>` cannot be shared between threads safelyのエラーになる．
- 6：`drop(sender)`を消すと，`receiver.iter()`が送信側の残りを待ち続け，テストが終わらなくなる．
- 7：`into_inner()`は，`Mutex`を消費して中の値を取り出す．もうほかのスレッドが使わないので，ロックは要らない．

## 11-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

- リファクタリングの項目は，`&mut`を`&`に変えるだけで，振る舞いは変わらない．
- `parallel`は，順序，スレッドの数の境界(0，1，要素より多い)，空の入力，実際に複数のスレッドで動くこと，の順に並べた．
- 並行の不具合は，たまにしか起きないことがある．同じオブジェクトを書くテストは，8つのスレッドで同時に書き，すべての結果を確かめる．
- 結合テストは，スレッドの数を変えて同じ操作をし，本物の`git`と比べる．`add`のテストでは，`git fsck`でオブジェクトが壊れていないことも確かめる．

## 11-4 図

模範解答は[design/types.md](../design/types.md)にある．Iteration 10からの変更は次のとおりである．

- `ObjectStore`の`write`から`&mut self`を消した．親トレイトの`Send + Sync`は図に書けないので，図の下に書いた．
- `MemoryObjectStore`の`objects`を`Mutex`にした．
- `Repository`から`objects_mut`を消し，`add`と`commit`の`&mut self`を消した．`add`にスレッドの数を加え，非公開のメソッド`write_blob`を描いた．
- `parallel`の名前空間に`parallel_mod`を描き，`cli_mod`，`Repository`，`status_mod`からの依存を描いた．
- `cli`に`Jobs`を加え，`Command`の`Add`と`Status`が持つことを描いた．

## 11-5 テスト駆動の実装

### `ObjectStore::write`を`&self`にする

トレイトと各実装の`write`を`&self`に変えると，`MemoryObjectStore`の`write`がコンパイルエラーになる．

```console
$ cargo build
error[E0596]: cannot borrow `self.objects` as mutable, as it is behind a `&` reference
   --> src/store.rs:111:9
    |
111 |         self.objects.insert(id, (kind, data.to_vec()));
    |         ^^^^^^^^^^^^ `self` is a `&` reference, so it cannot be borrowed as mutable
    |
help: consider changing this to be a mutable reference in the `impl` method and the `trait` definition
    |
 20 ~     fn write(&mut self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error>;
 21 |
...
108 |
109 ~     fn write(&mut self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error> {
    |

For more information about this error, try `rustc --explain E0596`.
error: could not compile `rgit` (lib) due to 1 previous error
```

コンパイラーは`&mut self`に戻す直し方を示すが，ここでは`HashMap`を`Mutex`で包む．

```rust
pub struct MemoryObjectStore {
    objects: Mutex<HashMap<ObjectId, (ObjectKind, Vec<u8>)>>,
}
```

`read`と`find`も`self.objects.lock().unwrap()`を通して読む．
`LooseObjectStore`は，フィールドを変えずにファイルを書くので，`&self`のまま書ける．
トレイトに`Send + Sync`を加えると，`MemoryObjectStore`と`LooseObjectStore`がそれを満たすかをコンパイラーが調べる．`Mutex`と`PathBuf`はどちらも`Send`かつ`Sync`なので，そのまま通る．

最後に，`Repository::objects_mut`を消す．呼んでいた場所がコンパイルエラーになるので，`objects()`に直し，不要になった`mut`を消した．

### 一時ファイルと名前の変更

先に，8つのスレッドで同じオブジェクトを書くテストを書いた．

```rust
#[test]
fn loose_store_accepts_same_object_from_many_threads() {
    let dir = TempDir::new().unwrap();
    let store = LooseObjectStore::new(dir.path());
    let ids: Vec<ObjectId> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| store.write(ObjectKind::Blob, b"hello\n").unwrap()))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert!(ids.iter().all(|&id| id == hash_blob(b"hello\n")));
    assert_eq!(store.read(ids[0]).unwrap().1, b"hello\n");
    // 一時ファイルは残らない．
    let names: Vec<_> = fs::read_dir(dir.path().join("ce"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, ["013625030ba8dba906f756967f9e9ca394464a"]);
}
```

`write`は，一時ファイルに書いてから名前を変える．

```rust
let dir = path.parent().unwrap();
fs::create_dir_all(dir)?;
// 書きかけのファイルを読まれないよう，一時ファイルに書いてから名前を変える．
let temp_path = dir.join(temp_name());
fs::write(&temp_path, compressed)?;
fs::rename(&temp_path, &path)?;
```

`fs::create_dir_all`は，ほかのスレッドが先にディレクトリを作っていてもエラーにしない．同時に呼んでも問題ない．

### `map_parallel`

最初のテストは，結果の順序である．スレッドの数を変えても，`map`と同じ結果になることを確かめる．

```rust
#[test]
fn results_keep_the_order_of_items() {
    let items: Vec<u64> = (0..100).collect();
    for jobs in [1, 2, 3, 8] {
        let squares = map_parallel(&items, jobs, |n| n * n);
        assert_eq!(squares, items.iter().map(|n| n * n).collect::<Vec<_>>());
    }
}
```

```rust
pub fn map_parallel<T, R, F>(items: &[T], jobs: usize, f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync,
{
    let workers = jobs.clamp(1, items.len().max(1));
    let next = AtomicUsize::new(0);
    let (sender, receiver) = mpsc::channel();
    let mut results: Vec<Option<R>> = items.iter().map(|_| None).collect();
    thread::scope(|scope| {
        for _ in 0..workers {
            let sender = sender.clone();
            let (next, f) = (&next, &f);
            scope.spawn(move || {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(item) = items.get(i) else {
                        break;
                    };
                    sender.send((i, f(item))).unwrap();
                }
            });
        }
        // 自分の送信側を捨てる．すべてのスレッドが終わると送信側がなくなり，受信のループが終わる．
        drop(sender);
        for (i, result) in receiver {
            results[i] = Some(result);
        }
    });
    results
        .into_iter()
        .map(|result| result.expect("どの要素も1回ずつ処理される"))
        .collect()
}
```

- 各スレッドには，自分の送信側を`move`で渡す．`next`と`f`は参照にしてから`move`するので，すべてのスレッドで同じものを共有する．
- `items.get(i)`は，範囲の外なら`None`を返す．`let … else`(Iteration 8)で，要素がなくなったらループを抜ける．
- 受信は，`scope`の中の元のスレッドで行う．スレッドが結果を送るそばから受け取るので，結果がチャネルにたまり続けない．
- `Vec<Option<R>>`は`vec![None; n]`では作れない(`R`が`Clone`とは限らない)．`items.iter().map(|_| None)`で作った．

`T`，`R`，`F`のどの境界を外しても，`scope.spawn`のところで「`R` cannot be sent between threads safely」のようなコンパイルエラーになる．エラーの文が，足りない境界を示す．

### `add`と`status`

`Repository::add`は，ファイルごとの処理を`write_blob`に分け，`map_parallel`で呼ぶ．

```rust
let entries = map_parallel(&files, jobs, |file| self.write_blob(file));
for (file, entry) in files.into_iter().zip(entries) {
    index.insert(file, entry?);
}
```

クロージャは`&self`を借りる．`Repository`のフィールドはどれも`Sync`なので，`Repository`も`Sync`になり，複数のスレッドで共有できる．
インデックスへの登録は，結果を受け取ったあとに1つのスレッドで行う．`Index`を`Mutex`で守る必要はない．

`work_tree_files`も同じ形にした．クロージャの戻り値の型を`-> Result<(Mode, ObjectId), Error>`と書いて，中で`?`を使えるようにした．

```rust
let hashed = map_parallel(&paths, jobs, |path| -> Result<(Mode, ObjectId), Error> {
    let full_path = work_dir.join(path);
    let mode = file_mode(&fs::metadata(&full_path)?);
    Ok((mode, hash_blob(&fs::read(&full_path)?)))
});
```

### `cli`

`add`と`status`で同じ`--jobs`を使うため，`Args`を導出した`Jobs`を作り，両方のサブコマンドに`#[command(flatten)]`で入れた．

```rust
/// ファイルを処理するスレッドの数．
#[derive(Args)]
struct Jobs {
    /// Number of threads to read and hash files (all CPUs by default)
    #[arg(short = 'j', long = "jobs")]
    jobs: Option<usize>,
}

impl Jobs {
    /// 指定がなければ，この計算機で同時に動かせるスレッドの数を使う．
    fn count(&self) -> usize {
        self.jobs.unwrap_or_else(available_jobs)
    }
}
```

`unwrap_or_else(available_jobs)`は，`None`のときだけ`available_jobs`を呼ぶ．関数の名前は，引数のないクロージャとして渡せる．

```console
$ rgit add --help
Add file contents to the index

Usage: rgit add [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...  Files or directories to add

Options:
  -j, --jobs <JOBS>  Number of threads to read and hash files (all CPUs by default)
  -h, --help         Print help
```

### 速度を測る

Iteration 9と同じ2000個のファイル(合わせて約80MB)のリポジトリで，4つのCPUを持つDev Containerで測った結果である(値は計算機によって変わる)．

```console
$ hyperfine -N --warmup 3 -L jobs 1,2,4 "$RGIT status --jobs {jobs}"
Benchmark 1: /workspaces/iterations/iteration-11/exercise/target/release/rgit status --jobs 1
  Time (mean ± σ):     272.2 ms ±  38.1 ms    [User: 196.1 ms, System: 82.0 ms]
  Range (min … max):   228.0 ms … 339.2 ms    10 runs
 
Benchmark 2: /workspaces/iterations/iteration-11/exercise/target/release/rgit status --jobs 2
  Time (mean ± σ):     137.5 ms ±   7.9 ms    [User: 187.0 ms, System: 72.8 ms]
  Range (min … max):   122.7 ms … 158.1 ms    22 runs
 
Benchmark 3: /workspaces/iterations/iteration-11/exercise/target/release/rgit status --jobs 4
  Time (mean ± σ):      76.8 ms ±   4.5 ms    [User: 189.2 ms, System: 55.5 ms]
  Range (min … max):    69.6 ms …  87.8 ms    38 runs
 
Summary
  /workspaces/iterations/iteration-11/exercise/target/release/rgit status --jobs 4 ran
    1.79 ± 0.15 times faster than /workspaces/iterations/iteration-11/exercise/target/release/rgit status --jobs 2
    3.54 ± 0.54 times faster than /workspaces/iterations/iteration-11/exercise/target/release/rgit status --jobs 1
$ hyperfine -N --runs 5 -L jobs 1,2,4 --prepare 'rm -rf .git/objects .git/index' "$RGIT add --jobs {jobs} ."
Benchmark 1: /workspaces/iterations/iteration-11/exercise/target/release/rgit add --jobs 1 .
  Time (mean ± σ):      4.673 s ±  0.345 s    [User: 3.232 s, System: 1.420 s]
  Range (min … max):    4.290 s …  5.044 s    5 runs
 
Benchmark 2: /workspaces/iterations/iteration-11/exercise/target/release/rgit add --jobs 2 .
  Time (mean ± σ):      2.547 s ±  0.156 s    [User: 3.334 s, System: 1.698 s]
  Range (min … max):    2.308 s …  2.696 s    5 runs
 
Benchmark 3: /workspaces/iterations/iteration-11/exercise/target/release/rgit add --jobs 4 .
  Time (mean ± σ):      1.461 s ±  0.022 s    [User: 3.378 s, System: 2.177 s]
  Range (min … max):    1.435 s …  1.483 s    5 runs
 
Summary
  /workspaces/iterations/iteration-11/exercise/target/release/rgit add --jobs 4 . ran
    1.74 ± 0.11 times faster than /workspaces/iterations/iteration-11/exercise/target/release/rgit add --jobs 2 .
    3.20 ± 0.24 times faster than /workspaces/iterations/iteration-11/exercise/target/release/rgit add --jobs 1 .
```

| スレッドの数 | `status` | `add` |
| --- | --- | --- |
| 1 | 272ms | 4.67秒 |
| 2 | 138ms(1.98倍) | 2.55秒(1.83倍) |
| 4 | 77ms(3.54倍) | 1.46秒(3.20倍) |

- `add`の時間の多くは，zlibの圧縮である．ファイルごとの処理が独立しているので，スレッドの数にほぼ比例して速くなる．
- `User`の時間は，スレッドの数によらずほぼ同じである．仕事の量は変わらず，同時に進めているだけだからである．`add`の`System`の時間は，スレッドを増やすと増える．多くのスレッドが同時にファイルを作るので，OSの中で待ち合わせの処理が増えると考えられる．
- アムダールの法則の式に，4つのスレッドで3.20倍を当てはめると，`add`の`p`は約0.92になる．並列にしていない処理(ディレクトリの走査，インデックスの読み書き，プロセスの起動)が，1つのスレッドのときの時間の約8%を使っている．`status`では約4%である．

## 11-6 振り返り

1. 模範解答は，並行の振る舞いを「スレッドの数を変えても結果が同じ」という形で確かめている．順序や回数のように，スレッドの数によらず決まる性質をテストにした．
2. あらかじめ分ける方法では，大きなファイルが1つのまとまりに集まると，そのスレッドだけが遅れて終わる．`AtomicUsize`で1つずつ配る方法では，空いたスレッドが次の要素を取るので，偏りの影響を受けにくい．
3. スレッドごとに別のオブジェクトストアを持たせると，ロックは要らない．その代わり，書いたオブジェクトを最後に1つへまとめる手間がかかる．ディスクのオブジェクトストアは，フィールドを変えずに書けるので，`&self`にするほうが自然である．
4. 1つのスレッドが書いている間，ほかのスレッドは`lock`で待つ．書き込みは`HashMap`への挿入だけで短いので，`MemoryObjectStore`では問題になりにくい．待ちが問題になるなら，ハッシュの計算や圧縮のような重い処理をロックの外で行う．
5. 並列にしていないのは，ディレクトリの走査，インデックスの読み書き，プロセスの起動などで，`add`では1つのスレッドのときの時間の約8%である．`p = 0.92`なら，8つのスレッドで`1 / (0.08 + 0.92 / 8)`の約5.1倍と見込める．スレッドをいくら増やしても，約12倍を超えない．
6. 図に描いた型と関係は，コードと一致している．

## 11-7 発展課題

### エラーで止める

`try_map_parallel`は，`map_parallel`を使って書ける．エラーが起きたことを`AtomicBool`で伝え，以後の要素は`None`にして飛ばす．

```rust
pub fn try_map_parallel<T, R, E, F>(items: &[T], jobs: usize, f: F) -> Result<Vec<R>, E>
where
    T: Sync,
    R: Send,
    E: Send,
    F: Fn(&T) -> Result<R, E> + Sync,
{
    let failed = AtomicBool::new(false);
    let results = map_parallel(items, jobs, |item| {
        if failed.load(Ordering::Relaxed) {
            return None;
        }
        let result = f(item);
        if result.is_err() {
            failed.store(true, Ordering::Relaxed);
        }
        Some(result)
    });
    // 飛ばした要素(None)があるのは，Errがあったときだけである．
    results.into_iter().flatten().collect()
}
```

- `flatten`は，`Option`の`None`を除き，`Some`の中身を取り出す．
- `Result`の列を`collect`で`Result<Vec<R>, E>`にすると，最初の`Err`を返す．飛ばした要素があるのは`Err`があったときだけなので，結果が欠けた`Ok`は返らない．

```rust
#[test]
fn try_map_parallel_stops_after_error() {
    let items: Vec<u64> = (0..1000).collect();
    let calls = AtomicUsize::new(0);
    let result = try_map_parallel(&items, 4, |&n| {
        calls.fetch_add(1, Ordering::Relaxed);
        if n == 10 { Err(n) } else { Ok(n) }
    });
    assert_eq!(result, Err(10));
    assert!(calls.load(Ordering::Relaxed) < 100);
}
```

エラーのあとも，すでに要素を取ったスレッドはその要素を処理するので，呼んだ回数は11回ちょうどとは限らない．テストでは，1000回よりずっと少ないことを確かめる．

`add`と`work_tree_files`は，`zip`で`?`を使う必要がなくなる．

```rust
let entries = try_map_parallel(&files, jobs, |file| self.write_blob(file))?;
for (file, entry) in files.into_iter().zip(entries) {
    index.insert(file, entry);
}
```

### ファイルの状態でハッシュの計算を省く

`work_tree_files`は，ファイルの`fs::metadata`を先に取り，記録と同じなら読まずに済ませる．

```rust
pub fn work_tree_files(
    work_dir: &Path,
    index: &Index,
    index_time: (u32, u32),
    jobs: usize,
) -> Result<BTreeMap<String, (Mode, ObjectId)>, Error> {
    let paths = list_files(work_dir, work_dir)?;
    let hashed = map_parallel(&paths, jobs, |path| -> Result<(Mode, ObjectId), Error> {
        let full_path = work_dir.join(path);
        let meta = fs::metadata(&full_path)?;
        let mode = file_mode(&meta);
        if let Some(id) = cached_id(index, path, &meta, index_time) {
            return Ok((mode, id));
        }
        Ok((mode, hash_blob(&fs::read(&full_path)?)))
    });
    paths
        .into_iter()
        .zip(hashed)
        .map(|(path, file)| Ok((path, file?)))
        .collect()
}

/// ファイルの状態がインデックスの記録と同じなら，記録したIDを返す．
/// インデックスを書いた時刻(`index_time`)と同じか後に変わったファイルは，
/// 同じ時刻のうちに書き換えられたかもしれないので，`None`を返して読ませる．
fn cached_id(
    index: &Index,
    path: &str,
    meta: &fs::Metadata,
    index_time: (u32, u32),
) -> Option<ObjectId> {
    let entry = index.entries().get(path)?;
    let stat = Stat::from_metadata(meta);
    let racy = (stat.mtime, stat.mtime_nsec) >= index_time;
    (entry.stat == stat && !racy).then_some(entry.id)
}
```

- `Stat::from_metadata`は，`add`がインデックスに記録するときと同じ関数である．記録と今の状態を同じ形で比べられる．
- `(stat.mtime, stat.mtime_nsec) >= index_time`は，タプルの比較で，秒が同じならナノ秒を比べる．
- `bool::then_some`は，`true`なら`Some(値)`を，`false`なら`None`を返す．

インデックスの時刻は`cli`で求め，`status`と`diff_work_tree`に引数で渡す．

```rust
/// インデックスのファイルの更新時刻(秒，ナノ秒)．インデックスがなければ`(0, 0)`を返す．
fn index_time(path: &Path) -> Result<(u32, u32), Error> {
    match fs::metadata(path) {
        Ok(meta) => Ok((meta.mtime() as u32, meta.mtime_nsec() as u32)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok((0, 0)),
        Err(error) => Err(error.into()),
    }
}
```

単体テストでは，中身と違うIDを，今のファイルの状態とともにインデックスに入れた．
記録が使われたかどうかが，`status`の結果の違いでわかる．

```rust
#[test]
fn files_with_recorded_stat_are_not_read() {
    let mut fixture = Fixture::new();
    fixture.write("hello.txt", "hello\n");
    // 中身と違うIDを，今のファイルの状態とともに記録する．
    let meta = fs::metadata(fixture.dir.path().join("hello.txt")).unwrap();
    let entry = IndexEntry {
        stat: Stat::from_metadata(&meta),
        mode: Mode::File,
        id: hash_blob(b"other\n"),
    };
    fixture.index.insert(String::from("hello.txt"), entry);
    let entries = |index_time| {
        status(
            &fixture.store,
            None,
            &fixture.index,
            index_time,
            fixture.dir.path(),
            2,
        )
        .unwrap()
        .iter()
        .map(|entry| entry.to_string())
        .collect::<Vec<_>>()
    };
    // インデックスがファイルより後に書かれたなら，ファイルを読まずに記録したIDを使う．
    assert_eq!(entries((u32::MAX, 0)), ["A  hello.txt"]);
    // ファイルがインデックスと同じ時刻か後に変わったなら(racy)，読んでハッシュを計算する．
    assert_eq!(entries((0, 0)), ["AM hello.txt"]);
}
```

`status`の結合テストは，`git status --porcelain`と一致したまま通る．
演習と同じリポジトリで，`git status`と比べた結果である．

```console
$ hyperfine -N --warmup 3 "$RGIT status" 'git status --porcelain'
Benchmark 1: /workspaces/iterations/iteration-11/exercise/target/release/rgit status
  Time (mean ± σ):      31.8 ms ±   7.1 ms    [User: 13.9 ms, System: 27.3 ms]
  Range (min … max):    18.9 ms …  51.5 ms    97 runs
 
Benchmark 2: git status --porcelain
  Time (mean ± σ):       6.8 ms ±   1.3 ms    [User: 2.9 ms, System: 6.3 ms]
  Range (min … max):     5.0 ms …  19.2 ms    553 runs
 
  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.
 
Summary
  git status --porcelain ran
    4.70 ± 1.39 times faster than /workspaces/iterations/iteration-11/exercise/target/release/rgit status
```

- 4つのスレッドでハッシュを計算していたときの77msから，32msになった．
- 残りの時間は，インデックスとHEADのツリーの読み込み，ディレクトリの走査，2000個のファイルの`metadata`である．`System`の時間が`User`より長いのは，ファイルの状態を調べるOSの処理が多いからである．
- `git`は，インデックスの読み込みやディレクトリの走査にも多くの工夫を重ねていて，さらに約5倍速い．
