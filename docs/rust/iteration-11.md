# Iteration 11：スレッドと共有

Iteration 11では，`add`と`status`のファイルの処理を複数のスレッドで行う．
このノートでは，スレッドの作り方，スレッドの間で値を渡すときの`Send`と`Sync`，`&self`のまま中身を変える`Mutex`，チャネル，アトミック変数を説明する．

## スレッドを作る

`std::thread::spawn`は，クロージャを新しいスレッドで実行する．戻り値の`JoinHandle`の`join`で，終わるのを待って結果を受け取る．

```rust
let numbers: Vec<i32> = (1..=3).collect();
let handle = thread::spawn(move || numbers.iter().sum::<i32>());
assert_eq!(handle.join().unwrap(), 6);
```

`spawn`したスレッドは，呼び出した関数が終わったあとも動き続けることがある．
そのため，クロージャは関数の変数を借りられず，`move`で所有権ごと渡す必要がある．`move`を付けないと，次のコンパイルエラーになる．

```console
error[E0373]: closure may outlive the current function, but it borrows `numbers`, which is owned by the current function
```

### `thread::scope`

`thread::scope`の中で`spawn`したスレッドは，`scope`を抜ける前に必ず終わる．
そのため，関数の変数を借りたまま使える．

```rust
let numbers: Vec<i32> = (1..=10).collect();
let (left, right) = numbers.split_at(5);
let (a, b) = thread::scope(|scope| {
    let a = scope.spawn(|| left.iter().sum::<i32>());
    let b = scope.spawn(|| right.iter().sum::<i32>());
    (a.join().unwrap(), b.join().unwrap())
});
assert_eq!(a + b, 55);
```

`rgit`は，ファイルのパスの一覧やオブジェクトストアを借りたまま，複数のスレッドで処理する．`thread::scope`を使う．

### `Arc`との比較

`spawn`で複数のスレッドに同じ値を渡すには，`Arc<T>`(参照の数を数える共有ポインター)で包み，`Arc::clone`をスレッドごとに`move`する．
最後の`Arc`がなくなると，中の値が片付けられる．

```rust
let count = Arc::new(Mutex::new(0));
let handles: Vec<_> = (0..4)
    .map(|_| {
        let count = Arc::clone(&count);
        thread::spawn(move || *count.lock().unwrap() += 1)
    })
    .collect();
```

`thread::scope`なら，`Arc`を使わずに普通の参照で共有できる．スレッドが関数の中で終わる処理には，`thread::scope`のほうが簡単である．

## `Send`と`Sync`

コンパイラーは，スレッドの間で安全に渡せる値かを，2つのトレイトで調べる．

| トレイト | 意味 | 実装していない例 |
| --- | --- | --- |
| `Send` | 値を別のスレッドに移してよい | `Rc<T>` |
| `Sync` | 参照`&T`を複数のスレッドで共有してよい | `RefCell<T>`，`Cell<T>` |

どちらも，型の中身から自動で決まる．すべてのフィールドが`Send`なら，構造体も`Send`である．
`spawn`は，クロージャとその中で使う値に`Send`を求める．`&T`を別のスレッドに渡せるのは，`T`が`Sync`のときだけである．

`RefCell`(実行時に借用を調べる型)は，借用の数を数えるのに，スレッドの間で守られない普通の数を使う．
`RefCell`を複数のスレッドで共有しようとすると，コンパイルエラーになる．

```rust
let count = RefCell::new(0);
thread::scope(|scope| {
    for _ in 0..4 {
        scope.spawn(|| *count.borrow_mut() += 1);
    }
});
```

```console
error[E0277]: `RefCell<i32>` cannot be shared between threads safely
  --> src/lib.rs:35:29
   |
35 |                 scope.spawn(|| *count.borrow_mut() += 1);
   |                       ----- ^^^^^^^^^^^^^^^^^^^^^^^^^^^ `RefCell<i32>` cannot be shared between threads safely
   |                       |
   |                       required by a bound introduced by this call
   |
   = help: the trait `std::marker::Sync` is not implemented for `RefCell<i32>`
   = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` instead
   = note: required for `&RefCell<i32>` to implement `Send`
```

データの競合(複数のスレッドが同時に同じ値を書き換えること)は，実行してから見つけるのが難しい．Rustは，それをコンパイルの時点で見つける．

### 親トレイト

トレイトの定義で`trait ObjectStore: Send + Sync`と書くと，`ObjectStore`を実装する型は`Send`と`Sync`も満たす必要がある．
`Send + Sync`を親トレイトと呼ぶ．

```rust
pub trait ObjectStore: Send + Sync {
    fn read(&self, id: ObjectId) -> Result<(ObjectKind, Vec<u8>), Error>;
    fn write(&self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error>;
    fn find(&self, prefix: &str) -> Result<Vec<ObjectId>, Error>;
}
```

`S: ObjectStore`を受け取るどの関数でも，`&S`を複数のスレッドで共有できる．`dyn ObjectStore`も`Sync`になる．

## `Mutex`と内部可変性

`&self`のメソッドは，ふつうはフィールドを変えられない．
`ObjectStore::write`を`&self`にすると，`MemoryObjectStore`の`HashMap`に入れるところがコンパイルエラーになる．

```console
error[E0596]: cannot borrow `self.objects` as mutable, as it is behind a `&` reference
```

`Mutex<T>`は，中の値への可変の参照を，一度に1つのスレッドにだけ貸す．
`lock()`は，ほかのスレッドが使い終わるまで待ってから，中の値を指す`MutexGuard`を返す．`MutexGuard`が片付けられると，ロックが外れる(Iteration 7の`Drop`)．

```rust
pub struct MemoryObjectStore {
    objects: Mutex<HashMap<ObjectId, (ObjectKind, Vec<u8>)>>,
}

fn write(&self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error> {
    let id = hash_object(kind, data);
    let mut objects = self.objects.lock().unwrap();
    objects.insert(id, (kind, data.to_vec()));
    Ok(id)
}
```

`&self`(共有の参照)から中身を変えられることを，内部可変性と呼ぶ．`RefCell`は1つのスレッドの中の，`Mutex`は複数のスレッドの間の内部可変性である．
`lock()`は`Result`を返す．ロックを持ったスレッドがパニックすると，中の値が中途半端な状態で残ることがあるからである．`rgit`では`unwrap`する．

`ObjectStore::write`が`&self`になったので，`Repository`は`objects_mut`を持たなくてよい．書き込む処理も`let repo`で済む．

## チャネル

`std::sync::mpsc::channel`は，スレッドの間で値を送る送信側(`Sender`)と受信側(`Receiver`)の組を作る．
送信側は`clone`して複数のスレッドに渡せる(mpscは「複数の送信側と1つの受信側」の略)．

```rust
let (sender, receiver) = mpsc::channel();
thread::scope(|scope| {
    for n in 0..3 {
        let sender = sender.clone();
        scope.spawn(move || sender.send(n * 10).unwrap());
    }
});
drop(sender);
let mut received: Vec<i32> = receiver.iter().collect();
```

受信側の`for`や`iter`は，すべての送信側が片付けられるまで，次の値を待ち続ける．
元の`sender`を`drop`で捨てないと，受信のループが終わらず，プログラムが止まったままになる．

## アトミック変数

`AtomicUsize`は，複数のスレッドから同時に読み書きしても壊れない整数である．`&self`のまま値を変えられる．
`fetch_add(1, …)`は，1を足し，足す前の値を返す．この2つの操作の間に，ほかのスレッドが割り込むことはない．

```rust
let next = AtomicUsize::new(0);
let i = next.fetch_add(1, Ordering::Relaxed);
```

複数のスレッドが`fetch_add`を呼ぶと，それぞれが違う番号を受け取る．`rgit`は，次に処理する要素の番号を配るのに使う．
`Ordering::Relaxed`は，この数そのものだけを正しく扱えばよいときの指定である．ほかのメモリーの読み書きとの順序を保証したいときは，別の`Ordering`を使う．

`static`の`AtomicUsize`は，プログラム全体で1つの数になる．一時ファイルの名前の番号に使う．

```rust
fn temp_name() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("tmp_obj_{}_{n}", process::id())
}
```

## クロージャのトレイトと`where`

関数を引数に取る型引数には，クロージャのトレイトで境界を付ける．

| トレイト | 呼べる回数 | 捕まえた値 |
| --- | --- | --- |
| `FnOnce` | 1回 | 消費してよい |
| `FnMut` | 何回でも | 変えてよい |
| `Fn` | 何回でも | 読むだけ |

`map_parallel`の`f`は，複数のスレッドから同時に何回も呼ばれる．`Fn`で，さらに`&F`を共有するために`Sync`が要る．
境界が長いときは，`where`で関数のシグネチャの後ろにまとめて書ける．

```rust
pub fn map_parallel<T, R, F>(items: &[T], jobs: usize, f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync,
{
    // …
}
```

- `T: Sync`：各スレッドが`&T`(要素への参照)を受け取る．
- `R: Send`：結果をチャネルで別のスレッドに送る．
- `F: Fn(&T) -> R + Sync`：各スレッドが`&F`を共有して呼ぶ．

## スレッドの数

`thread::available_parallelism()`は，この計算機で同時に動かせるスレッドの数の目安を`io::Result<NonZeroUsize>`で返す．
`NonZeroUsize`は0にならない`usize`で，`get()`で`usize`にする．

```rust
pub fn available_jobs() -> usize {
    thread::available_parallelism().map_or(1, NonZeroUsize::get)
}
```
