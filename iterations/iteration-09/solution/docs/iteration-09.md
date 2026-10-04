# Iteration 9：オブジェクトストアの抽象化と`status`(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 9-1 準備

`Cargo.toml`は，Iteration 8の模範解答からパッケージ名だけを変えたものである．依存は変わらない．

## 9-2 文法と概念

課題の解答例である．

```rust
#[cfg(test)]
mod practice {
    use std::collections::{BTreeMap, BTreeSet};

    trait Counter {
        fn count(&self, text: &str) -> usize;
    }

    struct Chars;
    struct Words;

    impl Counter for Chars {
        fn count(&self, text: &str) -> usize {
            text.chars().count()
        }
    }

    impl Counter for Words {
        fn count(&self, text: &str) -> usize {
            text.split_whitespace().count()
        }
    }

    fn total<C: Counter + ?Sized>(counter: &C, texts: &[&str]) -> usize {
        texts.iter().map(|text| counter.count(text)).sum()
    }

    #[test]
    fn generic_function_accepts_any_counter() {
        let texts = ["a b", "cde"];
        assert_eq!(total(&Chars, &texts), 6);
        assert_eq!(total(&Words, &texts), 3);
        let counter: &dyn Counter = &Words;
        assert_eq!(total(counter, &texts), 3);
    }

    #[test]
    fn boxed_trait_objects_of_different_types() {
        let counters: Vec<Box<dyn Counter>> = vec![Box::new(Chars), Box::new(Words)];
        let counts: Vec<usize> = counters.iter().map(|c| c.count("a b")).collect();
        assert_eq!(counts, [3, 2]);
    }

    #[derive(Debug, Default, PartialEq)]
    struct Settings {
        verbose: bool,
        jobs: usize,
        name: String,
    }

    #[test]
    fn default_fills_zero_values() {
        assert_eq!(
            Settings::default(),
            Settings {
                verbose: false,
                jobs: 0,
                name: String::new()
            }
        );
    }

    #[test]
    fn union_of_keys_in_order() {
        let a = BTreeMap::from([("b", 1), ("a", 1)]);
        let b = BTreeMap::from([("c", 2), ("a", 2)]);
        let keys: BTreeSet<&&str> = a.keys().chain(b.keys()).collect();
        assert_eq!(keys.into_iter().copied().collect::<Vec<_>>(), ["a", "b", "c"]);
    }

    #[test]
    fn map_or_gives_default() {
        assert_eq!(Some(3).map_or(0, |n| n * 2), 6);
        assert_eq!(None::<i32>.map_or(0, |n| n * 2), 0);
    }
}
```

- 2：`sum()`は，イテレーターの要素を足し合わせる．`total`は`?Sized`なので，`&dyn Counter`も受け取れる．`?Sized`を外すと，`counter`を渡す行がコンパイルエラーになる．
- 3：`Box::new(x)`は，値をヒープに置き，その所有権を持つ`Box<T>`を作る．`Box<dyn Counter>`にすれば，型の違う値を1つの`Vec`に入れられる．
- 5：`BTreeMap::from([…])`は，組の配列から表を作る．`BTreeSet`は，重なりを除いて順に並べる．

## 9-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

- リファクタリングの項目は，「移す」と「書き方を変える」に分けた．どちらも，振る舞いは変わらない．
- `store`は，2つの実装に同じ確認の関数`check_store`を使う．新しい実装を作ったときも，この関数を呼べば同じ振る舞いを確かめられる．
- `status`は，状態の組み合わせを1つずつ増やす順に並べた．`MM`の項目は，同じファイルの状態をYだけ`M`，Xだけ`M`，`MM`と変えていく1つのテストにした．
- 結合テストは，ノートの例と同じ操作を`rgit`で行い，`git status --porcelain -uall`と比べる．

## 9-4 図

模範解答は[design/types.md](../design/types.md)にある．Iteration 8からの変更は次のとおりである．

- `store`の名前空間に`ObjectStore`(`<<trait>>`)，`LooseObjectStore`，`MemoryObjectStore`を描き，実装の矢印でつないだ．
- `Repository`は`LooseObjectStore`を所有する．オブジェクトを読み書きするメソッドを消し，`objects`，`objects_mut`，`work_dir`を加えた．書き込むメソッドには`&mut self`を書いた．
- `tree_mod`に`write_tree`と`flatten_tree`を，`commit_mod`に`read_commit`を加え，`ObjectStore`への依存を描いた．
- `RevWalk~'s, S~`は`ObjectStore`を実装した値を借りる．関連の矢印の先を`Repository`から`ObjectStore`に変えた．
- `status`の名前空間に，`status_mod`，`Change`，`StatusEntry`を描いた．
- 型引数`S`の境界は図に書けないので，図の下に書いた．

## 9-5 テスト駆動の実装

### `ObjectStore`と`MemoryObjectStore`

`check_store`を先に書き，`MemoryObjectStore`で通した．

```rust
/// どのオブジェクトストアにも成り立つ振る舞いを確かめる．
fn check_store(store: &mut dyn ObjectStore) {
    let id = store.write(ObjectKind::Blob, b"hello\n").unwrap();
    assert_eq!(id, hash_blob(b"hello\n"));
    assert_eq!(
        store.read(id).unwrap(),
        (ObjectKind::Blob, b"hello\n".to_vec())
    );
    assert_eq!(store.find("ce01").unwrap(), [id]);
    assert_eq!(store.find("ffff").unwrap(), []);
    let missing = hash_blob(b"missing");
    assert!(matches!(
        store.read(missing),
        Err(Error::ObjectNotFound(name)) if name == missing.to_string()
    ));
}

#[test]
fn memory_store_reads_what_it_wrote() {
    check_store(&mut MemoryObjectStore::default());
}
```

```rust
impl ObjectStore for MemoryObjectStore {
    fn read(&self, id: ObjectId) -> Result<(ObjectKind, Vec<u8>), Error> {
        self.objects
            .get(&id)
            .cloned()
            .ok_or_else(|| Error::ObjectNotFound(id.to_string()))
    }

    fn write(&mut self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error> {
        let id = hash_object(kind, data);
        self.objects.insert(id, (kind, data.to_vec()));
        Ok(id)
    }

    fn find(&self, prefix: &str) -> Result<Vec<ObjectId>, Error> {
        let mut found: Vec<ObjectId> = self
            .objects
            .keys()
            .filter(|id| id.to_string().starts_with(prefix))
            .copied()
            .collect();
        found.sort();
        Ok(found)
    }
}
```

`ObjectId`は，Iteration 8で`Hash`を導出したので，そのまま`HashMap`のキーにできる．`copied()`は，`Copy`の要素の参照を値にする．

### `LooseObjectStore`

`Repository`の`write_object`，`read_object`，`object_path`を，ほとんどそのまま`LooseObjectStore`に移した．
`resolve_prefix`のディレクトリを読む部分は`find`にした．

```rust
fn find(&self, prefix: &str) -> Result<Vec<ObjectId>, Error> {
    let (dir_name, rest) = prefix.split_at(2);
    let dir = self.objects_dir.join(dir_name);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    for entry in fs::read_dir(dir)? {
        let name = entry?.file_name();
        if let Some(name) = name.to_str()
            && name.starts_with(rest)
            && let Ok(id) = format!("{dir_name}{name}").parse()
        {
            found.push(id);
        }
    }
    found.sort();
    Ok(found)
}
```

`check_store`を`LooseObjectStore`でも呼び，同じ振る舞いであることを確かめた．
`Repository::write_blob`のテストは，ファイルの形式を確かめる`LooseObjectStore`のテストとして移した．

### `Repository`

`Repository`は`LooseObjectStore`を持ち，`objects`と`objects_mut`で貸す．

```rust
pub struct Repository {
    work_dir: PathBuf,
    git_dir: PathBuf,
    objects: LooseObjectStore,
}
```

`resolve_prefix`は，桁数と文字を調べてから`find`を呼ぶ．

```rust
let mut found = self.objects.find(&prefix.to_ascii_lowercase())?;
match found.len() {
    0 => Err(not_found()),
    1 => Ok(found.remove(0)),
    _ => Err(Error::AmbiguousObject(prefix.to_string())),
}
```

オブジェクトのメソッドを消すと，`cli`の呼び出しがコンパイルエラーになる．エラーを1つずつ，`repo.objects().read(id)`や`repo.objects_mut().write(…)`に直した．
コンパイラーが直す場所をすべて教えてくれるので，リファクタリングの漏れがない．

### ジェネリックな関数に移す

`write_tree`は，`Repository`のメソッドから，オブジェクトストアを受け取る関数になった．中身は`self.write_object`を`store.write`に変えただけである．

```rust
pub fn write_tree<S: ObjectStore + ?Sized>(store: &mut S, index: &Index) -> Result<ObjectId, Error> {
    // Iteration 6と同じ
}
```

単体テストでは`TempDir`と`Repository`を使わず，組み立てたインデックスを`MemoryObjectStore`に書く．
`RevWalk`も，`Repository`の代わりに`&'s S`を持つ形にし，テストを`MemoryObjectStore`で書き直した．

### `flatten_tree`

```rust
#[test]
fn flatten_tree_lists_files_with_full_paths() {
    let mut store = MemoryObjectStore::default();
    let index = index_of(&[
        ("hello.txt", Mode::File, b"hello\n"),
        ("src/bin/tool.rs", Mode::Executable, b"fn main() {}\n"),
    ]);
    let root = write_tree(&mut store, &index).unwrap();
    let files = flatten_tree(&store, root).unwrap();
    let expected: BTreeMap<String, (Mode, ObjectId)> = index
        .entries()
        .iter()
        .map(|(path, entry)| (path.clone(), (entry.mode, entry.id)))
        .collect();
    assert_eq!(files, expected);
}
```

インデックスから書いたtreeを展開すると，元のインデックスと同じ表になる．往復で確かめた．
`flatten_tree`は，Iteration 4の発展課題の`ls-tree -r`と同じく，サブディレクトリで再帰する．

### `status`

単体テストの準備をまとめる`Fixture`を作り，最も単純な項目から書いた．

```rust
#[test]
fn modified_file_is_unstaged_then_staged() {
    let mut fixture = Fixture::new();
    fixture.write("hello.txt", "hello\n");
    fixture.add("hello.txt");
    fixture.commit();
    fixture.write("hello.txt", "hello\nworld\n");
    assert_eq!(fixture.status(), [" M hello.txt"]);
    fixture.add("hello.txt");
    assert_eq!(fixture.status(), ["M  hello.txt"]);
    fixture.write("hello.txt", "changed again\n");
    assert_eq!(fixture.status(), ["MM hello.txt"]);
}
```

`status`は，3つの表を作ってから比べる．

```rust
pub fn status<S: ObjectStore + ?Sized>(
    store: &S,
    head: Option<ObjectId>,
    index: &Index,
    work_dir: &Path,
) -> Result<Vec<StatusEntry>, Error> {
    let head_files = match head {
        Some(commit) => flatten_tree(store, read_commit(store, commit)?.tree)?,
        None => BTreeMap::new(),
    };
    let index_files: BTreeMap<String, (Mode, ObjectId)> = index
        .entries()
        .iter()
        .map(|(path, entry)| (path.clone(), (entry.mode, entry.id)))
        .collect();
    let work_files = work_tree_files(work_dir)?;

    let mut entries = Vec::new();
    let paths: BTreeSet<&String> = head_files.keys().chain(index_files.keys()).collect();
    for path in paths {
        let staged = compare(head_files.get(path), index_files.get(path));
        let unstaged = match index_files.get(path) {
            Some(_) => compare(index_files.get(path), work_files.get(path)),
            None => None,
        };
        if staged.is_some() || unstaged.is_some() {
            entries.push(StatusEntry::Changed {
                path: path.clone(),
                staged,
                unstaged,
            });
        }
    }
    for path in work_files.keys() {
        if !index_files.contains_key(path) {
            entries.push(StatusEntry::Untracked(path.clone()));
        }
    }
    Ok(entries)
}
```

- `unstaged`は，インデックスにあるファイルだけを比べる．インデックスにないファイルは，追跡していないファイルとして後で扱う．
- 追跡していないファイルを最後にまとめたので，`git`と同じく変更のあるファイルのあとに並ぶ．
- `StatusEntry`を，`path`と2つの`Option<Change>`を持つ`Changed`と，`Untracked`の`enum`にした．追跡していないファイルに`staged`や`unstaged`という意味のないフィールドを持たせずに済む．

作業ディレクトリのファイルのモードは，`add`と`status`で同じ規則にするため，`worktree::file_mode`にまとめた．

### `cli`

```rust
Command::Status => {
    let repo = Repository::discover(cwd)?;
    let index = Index::load(&repo.index_path())?;
    let head = repo.resolve_ref(&RefName::head())?;
    for entry in status(repo.objects(), head, &index, repo.work_dir())? {
        writeln!(out, "{entry}")?;
    }
}
```

## 9-6 振り返り

1. 模範解答のリファクタリングの項目は，既存のテストを消さずに移している．テストを移す先を決めることが，責務を移す先を決めることになっている．
2. トレイトのメソッドが増えると，新しいオブジェクトストアはすべてを実装する必要がある．`write_tree`は置き場所によらず同じ処理なので，トレイトの外の関数にすれば，実装する人は3つのメソッドだけを書けばよい．
3. `&mut dyn ObjectStore`にすると，`write_tree`の機械語は1つになり，呼び出しは実行時に実装を選ぶ．呼ぶ側のコードと振る舞いは変わらない．
4. ファイルを作らずにオブジェクトを読み書きできるので，一時ディレクトリの準備が要らない．その代わり，zlibの形式やファイルの置き場所は確かめられない．それは`LooseObjectStore`のテストと結合テストが受け持つ．
5. 図に描いた型と関係は，コードと一致している．

## 9-7 発展課題

`CountingStore<S>`は，`S: ObjectStore`のときに`ObjectStore`を実装する．
`read`と`find`は中身の`inner`にそのまま任せ，`write`だけ数を増やしてから任せる．

```rust
/// ほかのオブジェクトストアを包み，書き込んだ回数を数える．
pub struct CountingStore<S> {
    inner: S,
    writes: usize,
}

impl<S: ObjectStore> ObjectStore for CountingStore<S> {
    fn read(&self, id: ObjectId) -> Result<(ObjectKind, Vec<u8>), Error> {
        self.inner.read(id)
    }

    fn write(&mut self, kind: ObjectKind, data: &[u8]) -> Result<ObjectId, Error> {
        self.writes += 1;
        self.inner.write(kind, data)
    }

    fn find(&self, prefix: &str) -> Result<Vec<ObjectId>, Error> {
        self.inner.find(prefix)
    }
}
```

`impl<S: ObjectStore> ObjectStore for CountingStore<S>`は，「中身がオブジェクトストアなら，包んだものもオブジェクトストアである」ことを表す．
`write_tree`は`CountingStore`のことを知らないまま，同じように動く．
`hello.txt`，`src/main.rs`，`src/bin/tool.rs`のインデックスからは，最上位，`src`，`src/bin`の3つのtreeが書かれる．
