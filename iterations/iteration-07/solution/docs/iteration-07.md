# Iteration 7：参照，`commit`，`branch`(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 7-1 準備

`Cargo.toml`は，Iteration 6の模範解答からパッケージ名だけを変えたものである．依存は変わらない．

## 7-2 文法と概念

課題の解答例である．

```rust
#[cfg(test)]
mod practice {
    use std::fs::{self, OpenOptions};
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    #[derive(Debug, PartialEq)]
    struct Even(u32);

    impl TryFrom<u32> for Even {
        type Error = String;

        fn try_from(value: u32) -> Result<Even, String> {
            if value.is_multiple_of(2) {
                Ok(Even(value))
            } else {
                Err(format!("{value} is odd"))
            }
        }
    }

    #[test]
    fn only_even_numbers_become_even() {
        assert_eq!(Even::try_from(4), Ok(Even(4)));
        assert_eq!(Even::try_from(3), Err(String::from("3 is odd")));
    }

    struct Marker {
        path: PathBuf,
    }

    impl Marker {
        fn new(path: &Path) -> Marker {
            fs::write(path, "").unwrap();
            Marker {
                path: path.to_path_buf(),
            }
        }
    }

    impl Drop for Marker {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    #[test]
    fn marker_file_is_removed_when_dropped() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("marker");
        {
            let _marker = Marker::new(&path);
            assert!(path.exists());
        }
        assert!(!path.exists());
    }

    struct Ticket {
        name: String,
    }

    impl Ticket {
        fn use_once(self) -> String {
            format!("used {}", self.name)
        }
    }

    #[test]
    fn ticket_is_consumed() {
        let ticket = Ticket {
            name: String::from("A"),
        };
        assert_eq!(ticket.use_once(), "used A");
    }

    #[test]
    fn create_new_fails_for_existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("x.lock");
        let open = || OpenOptions::new().write(true).create_new(true).open(&path);
        assert!(open().is_ok());
        assert_eq!(open().unwrap_err().kind(), ErrorKind::AlreadyExists);
    }

    #[test]
    fn as_deref_gives_default() {
        let given: Option<String> = Some(String::from("topic"));
        let missing: Option<String> = None;
        assert_eq!(given.as_deref().unwrap_or("HEAD"), "topic");
        assert_eq!(missing.as_deref().unwrap_or("HEAD"), "HEAD");
    }
}
```

- 1：`value % 2 == 0`と書くと，`cargo clippy`が`is_multiple_of`を使うよう指摘する．
- 2：変数名を`_marker`にした．`_`だけにすると，値はその場で片付けられ，ファイルはすぐに消える．`_`で始まる名前なら，スコープの終わりまで値が生きる．
- 3：2回目の呼び出しは，ノートと同じ`E0382`になる．

```text
error[E0382]: use of moved value: `ticket`
  --> src/lib.rs:94:20
   |
90 |         let ticket = Ticket {
   |             ------ move occurs because `ticket` has type `Ticket`, which does not implement the `Copy` trait
...
93 |         assert_eq!(ticket.use_once(), "used A");
   |                           ---------- `ticket` moved due to this method call
94 |         assert_eq!(ticket.use_once(), "used A");
   |                    ^^^^^^ value used here after move
```

## 7-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

- 参照名の不正な例は，1つのテストの中で配列にまとめ，`for`で確かめた．`assert!`の2つ目の引数で，失敗したときにどの名前かを表示する．
- `LockFile`の項目は，置き換える場合，捨てる場合，`.lock`がすでにある場合の3つと，親のディレクトリを作る場合である．`refs/heads/feature/login`のように，ブランチ名に`/`があると親のディレクトリが要る．
- コミットのIDは，`rgit`のリポジトリと本物の`git`のリポジトリで同じ操作をして比べた．2つ目のコミットまで比べると，親の扱いも確かめられる．
- ロックのテストは，`.lock`を手で作ってから`commit`し，エラーになることと，ブランチが進まないことを確かめる．

## 7-4 図

模範解答は[design/types.md](../design/types.md)にある．Iteration 6からの変更は次のとおりである．

- `refs`の名前空間に`RefName`と`Ref`を，`lockfile`の名前空間に`LockFile`を，`revision`の名前空間に`resolve`を描いた．
- `Ref`は`ObjectId`か`RefName`を持つので，両方への所有の矢印を描いた．
- `Repository`に参照のメソッドと`commit`を加え，`Ref`，`RefName`，`LockFile`，`Commit`への依存を描いた．`Index`から`LockFile`への依存も加えた．
- `cli_mod`から`revision_mod`と`RefName`への依存を描いた．
- `LockFile::commit`が`self`を受け取ることと，`Drop`の振る舞いは，図の下に書いた．

## 7-5 テスト駆動の実装

### `RefName`

```rust
#[test]
fn head_and_names_under_refs_are_valid() {
    assert_eq!(RefName::try_from("HEAD").unwrap().as_str(), "HEAD");
    assert!(RefName::try_from("refs/heads/main").is_ok());
    assert!(RefName::try_from("refs/heads/feature/login").is_ok());
}
```

最初は，`HEAD`か`refs/`で始まるかだけを調べて通した．不正な名前のテストを加えて，規則を1つずつ足した．

```rust
/// `refs/`で始まり，Gitの参照名の規則の一部を満たすか．
fn is_valid_ref_path(name: &str) -> bool {
    let forbidden = |ch: char| ch.is_ascii_control() || " ~^:?*[\\".contains(ch);
    name.starts_with("refs/")
        && !name.contains("..")
        && !name.contains(forbidden)
        && name
            .split('/')
            .all(|part| !part.is_empty() && !part.starts_with('.') && !part.ends_with(".lock"))
}
```

末尾の`/`(`refs/heads/`)は，`split('/')`の最後の要素が空になるので，空の要素の規則で弾かれる．

```rust
/// ブランチ名から`refs/heads/<name>`の参照名を作る．
pub fn branch(name: &str) -> Result<RefName, Error> {
    RefName::try_from(format!("refs/heads/{name}").as_str())
        .map_err(|_| Error::InvalidBranchName(name.to_string()))
}
```

ブランチ名の誤りは，参照名ではなくブランチ名でエラーを表示する．`map_err`でエラーを差し替えた．

### `LockFile`

```rust
#[test]
fn commit_replaces_file_and_removes_lock() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("main");
    fs::write(&path, "old\n").unwrap();
    let mut lock = LockFile::acquire(&path).unwrap();
    lock.write_all(b"new\n").unwrap();
    assert!(dir.path().join("main.lock").exists());
    assert_eq!(fs::read_to_string(&path).unwrap(), "old\n");
    lock.commit().unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "new\n");
    assert!(!dir.path().join("main.lock").exists());
}
```

```rust
/// `<path>.lock`を作る．すでにあれば，ほかの処理が更新中なのでエラーにする．
pub fn acquire(path: &Path) -> Result<LockFile, Error> {
    let mut lock_path = path.as_os_str().to_owned();
    lock_path.push(".lock");
    let lock_path = PathBuf::from(lock_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            return Err(Error::Locked(lock_path.display().to_string()));
        }
        Err(error) => return Err(error.into()),
    };
    Ok(LockFile {
        path: path.to_path_buf(),
        lock_path,
        file,
        committed: false,
    })
}

/// 書いた内容で，元のファイルを置き換える．
pub fn commit(mut self) -> Result<(), Error> {
    fs::rename(&self.lock_path, &self.path)?;
    self.committed = true;
    Ok(())
}
```

捨てる場合のテストで，`Drop`を加えた．

```rust
#[test]
fn dropping_without_commit_removes_lock_and_keeps_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("main");
    fs::write(&path, "old\n").unwrap();
    {
        let mut lock = LockFile::acquire(&path).unwrap();
        lock.write_all(b"new\n").unwrap();
    }
    assert_eq!(fs::read_to_string(&path).unwrap(), "old\n");
    assert!(!dir.path().join("main.lock").exists());
}
```

```rust
impl Drop for LockFile {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(&self.lock_path);
        }
    }
}
```

`Index::save`も`LockFile`で書く形に変えた．インデックスの既存のテストは，変えずに通る．

```rust
/// インデックスのファイルを，ロックファイルを通して置き換える．
pub fn save(&self, path: &Path) -> Result<(), Error> {
    let mut lock = LockFile::acquire(path)?;
    lock.write_all(&self.to_bytes())?;
    lock.commit()
}
```

### 参照の読み書き

```rust
/// 参照を読む．参照のファイルがなければ`None`を返す．
pub fn read_ref(&self, name: &RefName) -> Result<Option<Ref>, Error> {
    match fs::read_to_string(self.git_dir.join(name.as_str())) {
        Ok(text) => Ok(Some(Ref::parse(&text)?)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// シンボリック参照をたどり，最後に`Direct`の参照を持つ参照名を返す．
/// `HEAD`がまだコミットのないブランチを指していれば，そのブランチ名を返す．
pub fn final_ref_name(&self, name: &RefName) -> Result<RefName, Error> {
    let mut name = name.clone();
    for _ in 0..5 {
        match self.read_ref(&name)? {
            Some(Ref::Symbolic(target)) => name = target,
            Some(Ref::Direct(_)) | None => return Ok(name),
        }
    }
    Err(Error::InvalidRefName(name.to_string()))
}
```

`self.git_dir.join(name.as_str())`は，検査済みの`RefName`でだけ作れる．`..`を含む名前で`.git`の外を読むことはない．
`final_ref_name`は，コミットのないブランチでも，そのブランチ名を返す．最初のコミットで，どのブランチを作ればよいかが分かる．
`resolve_ref`は，`final_ref_name`の参照を読み，`Direct`ならそのIDを返す．`update_ref`は`LockFile`でIDと改行を書く．

### `revision::resolve`

```rust
/// リビジョンの指定(`HEAD`，ブランチ名，`refs/`で始まる参照名，4桁以上のID)を，コミットなどのIDにする．
pub fn resolve(repo: &Repository, rev: &str) -> Result<ObjectId, Error> {
    for candidate in [rev.to_string(), format!("refs/heads/{rev}")] {
        if let Ok(name) = RefName::try_from(candidate.as_str())
            && let Some(id) = repo.resolve_ref(&name)?
        {
            return Ok(id);
        }
    }
    if rev == "HEAD" {
        return Err(Error::ObjectNotFound(rev.to_string()));
    }
    repo.resolve_prefix(rev)
}
```

参照名として読めない指定(`6c04901`は`refs/`で始まらない)は，`try_from`の`Err`になって次の候補に進む．
本物のGitと同じく，ブランチ名を短縮したIDより先に調べる．`cat-file`，`ls-tree`，`commit-tree`も`resolve`を使う形に変えた．

### `commit`

`Repository::commit`は，コミットを書いて，`HEAD`のたどり着く参照を更新する．

```rust
/// コミットを書き込み，`HEAD`が指すブランチ(切り離された`HEAD`なら`HEAD`)をそのコミットに進める．
pub fn commit(&self, commit: Commit) -> Result<ObjectId, Error> {
    let id = self.write_object(ObjectKind::Commit, &commit.to_bytes())?;
    let target = self.final_ref_name(&RefName::head())?;
    self.update_ref(&target, id)?;
    Ok(id)
}
```

`cli`の`commit`は，親を`resolve_ref(&head)`で求め，あれば`parent`に加える．出力の形は，親の有無と`HEAD`のたどり着く参照名から作る．

```rust
let id = repo.commit(builder.build())?;
let target = repo.final_ref_name(&head)?;
let branch = target.branch_name().unwrap_or("detached HEAD");
let root = if parent.is_none() { " (root-commit)" } else { "" };
let summary = message.lines().next().unwrap_or("");
writeln!(out, "[{branch}{root} {}] {summary}", id.short())?;
```

### `branch`

一覧と作成は，`Command::Branch`の`name`が`None`か`Some`かで，`match`の腕を分けた．

```rust
Command::Branch { name: None, .. } => {
    let repo = Repository::discover(cwd)?;
    let current = repo.final_ref_name(&RefName::head())?;
    for branch in repo.branches()? {
        let marker = if current.branch_name() == Some(branch.as_str()) {
            '*'
        } else {
            ' '
        };
        writeln!(out, "{marker} {branch}")?;
    }
}
```

作成では，`start.as_deref().unwrap_or("HEAD")`で既定値を決め，`resolve`でIDにする．

## 7-6 振り返り

1. 模範解答は，`LockFile`を参照とインデックスの両方で使うので，`LockFile`だけの単体テストを独立させた．
2. `&str`を受け取る設計では，`update_ref`，`read_ref`，`resolve_ref`のそれぞれで検査が要り，どれかで忘れると`.git`の外のファイルを読み書きできてしまう．`RefName`なら，検査は`try_from`の1か所で，忘れようがない．
3. `.lock`のファイルが残り，以後のコミットがすべて失敗する．ほかの言語では`try`と`finally`や`defer`で消す処理を書く．`Drop`なら，`LockFile`を使うすべての場所で自動的に消える．
4. `&mut self`なら，`commit`のあとも`lock`を使えるので，置き換えたあとに`.lock`へ書き込むコードがコンパイルできてしまう．`self`を受け取ると，そのコードはコンパイルエラーになる．
5. 図に描いた型と関数は，コードと一致している．

## 7-7 発展課題

`Branch`に`-d`のフラグを加え，`delete: true`の腕を，作成の腕より前に置く．

```rust
Command::Branch {
    delete: true,
    name: Some(name),
    ..
} => {
    let repo = Repository::discover(cwd)?;
    let branch = RefName::branch(&name)?;
    if repo.final_ref_name(&RefName::head())? == branch {
        return Err(Error::CurrentBranch(name));
    }
    let Some(Ref::Direct(id)) = repo.read_ref(&branch)? else {
        return Err(Error::BranchNotFound(name));
    };
    repo.delete_ref(&branch)?;
    writeln!(out, "Deleted branch {name} (was {}).", id.short())?;
}
```

`let パターン = 式 else { … };`は，パターンに一致しなければ`else`の中を実行する．`else`の中は，`return`などで必ず抜ける必要がある．
`Repository::delete_ref`は，ほかの処理と同時に書き換えないように，ロックを取ってからファイルを消す．

```rust
/// 参照を消す．
pub fn delete_ref(&self, name: &RefName) -> Result<(), Error> {
    let lock = LockFile::acquire(&self.git_dir.join(name.as_str()))?;
    fs::remove_file(self.git_dir.join(name.as_str()))?;
    drop(lock);
    Ok(())
}
```

`drop(lock)`は，値をその場で片付ける．`commit`しないので，`Drop`が`.lock`のファイルを消す．
本物の`git`は，今のブランチを消そうとすると`cannot delete branch 'main' used by worktree at '<パス>'`と表示する．`rgit`は短いメッセージにした．
