# Iteration 9：トレイトの設計とジェネリクス

Iteration 9では，オブジェクトの読み書きをトレイト`ObjectStore`にまとめ，ディスクに書く実装とメモリーに持つ実装を作る．
このノートでは，トレイトの定義と実装，トレイト境界を持つジェネリクス，`?Sized`，トレイトオブジェクト(`dyn`)との比較，トレイトを使ったテストの差し替えを説明する．

## トレイトを定義する

これまで`Display`や`Iterator`など，標準のトレイトを実装してきた．トレイトは自分でも定義できる．
トレイトには，それを実装する型が持つべきメソッドのシグネチャを書く．

```rust
pub trait ObjectStore {
    fn read(&self, id: ObjectId) -> Result<(ObjectKind, Vec<u8>), Error>;
    fn write(&mut self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error>;
    fn find(&self, prefix: &str) -> Result<Vec<ObjectId>, Error>;
}
```

`impl トレイト for 型 { … }`で，型にトレイトを実装する．

```rust
#[derive(Debug, Default)]
pub struct MemoryObjectStore {
    objects: HashMap<ObjectId, (ObjectKind, Vec<u8>)>,
}

impl ObjectStore for MemoryObjectStore {
    fn write(&mut self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error> {
        let id = hash_object(kind, data);
        self.objects.insert(id, (kind, data.to_vec()));
        Ok(id)
    }
    // read，find
}
```

トレイトのメソッドを呼ぶには，トレイトを`use`する(Iteration 0の`Digest`と同じ)．

## 何をトレイトにするか

`rgit`の処理でオブジェクトの置き場所に依存するのは，読む・書く・IDの先頭で探すの3つだけである．
treeを書く処理や，コミットのグラフをたどる処理は，この3つさえあれば，置き場所を知らなくてよい．

- トレイトには，置き場所ごとに実装が変わる最小限の操作だけを入れる．
- treeの組み立てやコミットの解析のように，どの置き場所でも同じ処理は，トレイトを使う関数として外に書く．

トレイトのメソッドが少ないほど，新しい実装(パックファイル，ネットワークの向こうのリポジトリなど)を作りやすい．

## トレイト境界を持つジェネリクス

関数の型引数に`S: ObjectStore`と書くと，「`ObjectStore`を実装した任意の型`S`」を受け取れる．これをトレイト境界と呼ぶ．

```rust
pub fn read_commit<S: ObjectStore + ?Sized>(store: &S, id: ObjectId) -> Result<Commit, Error> {
    match store.read(id)? {
        (ObjectKind::Commit, content) => Commit::parse(&content),
        _ => Err(Error::NotACommit(id.to_string())),
    }
}
```

`read_commit`は，`LooseObjectStore`と`MemoryObjectStore`のどちらでも使える．
コンパイラーは，呼び出しで使われた型ごとに`read_commit`の機械語を作る(単相化)．実行時の速さは，型を直接書いた関数と変わらない．

構造体の型引数にも，トレイト境界を付けられる．

```rust
pub struct RevWalk<'s, S: ObjectStore + ?Sized> {
    store: &'s S,
    queue: BinaryHeap<(i64, ObjectId)>,
    seen: HashSet<ObjectId>,
}

impl<S: ObjectStore + ?Sized> Iterator for RevWalk<'_, S> {
    // …
}
```

### `?Sized`

既定の型引数は，大きさがコンパイル時に決まる型(`Sized`)だけを受け取る．
`?Sized`は，その制限を外す．`dyn ObjectStore`(次の節)のように大きさの決まらない型も，`&S`の形で受け取れるようになる．
参照(`&S`，`&mut S`)でしか受け取らない型引数には，`?Sized`を付けておくと使える場面が広がる．

## トレイトオブジェクト

`&dyn ObjectStore`は，「`ObjectStore`を実装した何かへの参照」という1つの型である．
どの型の値かは実行時に決まり，メソッドの呼び出しは，実行時に表を引いて実装を選ぶ(動的ディスパッチ)．

```rust
/// どのオブジェクトストアにも成り立つ振る舞いを確かめる．
fn check_store(store: &mut dyn ObjectStore) {
    let id = store.write(ObjectKind::Blob, b"hello\n").unwrap();
    assert_eq!(id, hash_blob(b"hello\n"));
    // …
}

check_store(&mut MemoryObjectStore::default());
check_store(&mut LooseObjectStore::new(dir.path()));
```

| | ジェネリクス(`S: ObjectStore`) | トレイトオブジェクト(`dyn ObjectStore`) |
| --- | --- | --- |
| 型が決まるとき | コンパイル時 | 実行時 |
| メソッドの呼び出し | 直接呼ぶ(静的ディスパッチ) | 表を引いて呼ぶ(動的ディスパッチ) |
| 機械語 | 型ごとに作られる | 1つだけ |
| 1つの`Vec`に違う型を入れる | できない | `Vec<Box<dyn ObjectStore>>`でできる |

`rgit`は，ふだんはジェネリクスを使い，テストで同じ確認を2つの実装に行うところでトレイトオブジェクトを使う．
`?Sized`を付けたジェネリックな関数は，`&dyn ObjectStore`も受け取れる．

## テストのための差し替え

`MemoryObjectStore`は，ファイルを読み書きしないので，一時ディレクトリが要らず，速い．
treeを書く処理やコミットのグラフをたどる処理の単体テストは，`MemoryObjectStore`で書ける．

```rust
let mut store = MemoryObjectStore::default();
let root = write_tree(&mut store, &index).unwrap();
let files = flatten_tree(&store, root).unwrap();
```

`LooseObjectStore`のテストは，ファイルの形式(zlib，置き場所)を確かめるものだけに絞れる．

## `Repository`がオブジェクトストアを貸す

`Repository`は`LooseObjectStore`をフィールドに持ち，メソッドで貸す．

```rust
pub fn objects(&self) -> &LooseObjectStore {
    &self.objects
}

pub fn objects_mut(&mut self) -> &mut LooseObjectStore {
    &mut self.objects
}
```

`write`は`&mut self`を受け取るので，書き込む側は`objects_mut()`を使い，`Repository`も`let mut repo`で持つ．
読むだけの処理は`objects()`で済む．書き込むかどうかが，型で区別される．

## 2つの表の突き合わせ

`status`は，HEAD，インデックス，作業ディレクトリの表(`BTreeMap<String, (Mode, ObjectId)>`)を比べる．
両方の表のパスを`chain`でつなぎ，`BTreeSet`に集めると，重なりを除いたパスをパスの順に回せる．

```rust
let paths: BTreeSet<&String> = head_files.keys().chain(index_files.keys()).collect();
```

1つのパスの古い状態と新しい状態は，`Option`の組を`match`で分けて比べる．

```rust
fn compare(old: Option<&(Mode, ObjectId)>, new: Option<&(Mode, ObjectId)>) -> Option<Change> {
    match (old, new) {
        (None, None) => None,
        (None, Some(_)) => Some(Change::Added),
        (Some(_), None) => Some(Change::Deleted),
        (Some(old), Some(new)) if old != new => Some(Change::Modified),
        (Some(_), Some(_)) => None,
    }
}
```

4つの組み合わせをすべて書くので，扱い忘れはコンパイラーが見つける．

`Option::map_or(既定値, 関数)`は，`Some(x)`なら`関数(x)`を，`None`なら既定値を返す．

```rust
let code = |change: &Option<Change>| change.map_or(' ', Change::code);
```

## テストの準備をまとめる構造体

`status`のテストでは，作業ディレクトリ，インデックス，オブジェクトストア，HEADを毎回用意する．
これらを1つの構造体にまとめ，ファイルを書く・`add`する・コミットするといったメソッドを持たせると，テストが短くなる．

```rust
struct Fixture {
    dir: TempDir,
    store: MemoryObjectStore,
    index: Index,
    head: Option<ObjectId>,
}
```

`dir`の`TempDir`は，`Fixture`と一緒に片付けられて消える．
