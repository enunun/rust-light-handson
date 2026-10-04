# Iteration 6：`write-tree`と`commit-tree`(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 6-1 準備

`Cargo.toml`は，Iteration 5の模範解答からパッケージ名だけを変えたものである．依存は変わらない．

## 6-2 文法と概念

課題の解答例である．

```rust
#[cfg(test)]
mod practice {
    use std::cmp::Ordering;
    use std::collections::HashMap;
    use std::marker::PhantomData;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sorts_by_length_then_name() {
        let mut names = vec!["bb", "a", "ab", "c"];
        names.sort_by(|a, b| a.len().cmp(&b.len()).then(a.cmp(b)));
        assert_eq!(names, ["a", "c", "ab", "bb"]);
    }

    #[test]
    fn compares_with_virtual_slash() {
        let dir = "a".bytes().chain(Some(b'/'));
        let file = "a.txt".bytes().chain(None);
        assert_eq!(dir.cmp(file), Ordering::Greater);
        assert_eq!(true.then_some(b'/'), Some(b'/'));
        assert_eq!(false.then_some(b'/'), None);
    }

    struct Open;
    struct Closed;

    struct Door<S> {
        name: String,
        state: PhantomData<S>,
    }

    impl Door<Closed> {
        fn new(name: &str) -> Door<Closed> {
            Door {
                name: name.to_string(),
                state: PhantomData,
            }
        }

        fn open(self) -> Door<Open> {
            Door {
                name: self.name,
                state: PhantomData,
            }
        }
    }

    impl Door<Open> {
        fn walk_through(&self) -> String {
            format!("walked through {}", self.name)
        }
    }

    #[test]
    fn only_open_door_can_be_walked_through() {
        let door = Door::new("front").open();
        assert_eq!(door.walk_through(), "walked through front");
    }

    #[test]
    fn reads_environment_like_map() {
        let mut env = HashMap::new();
        env.insert(String::from("GIT_AUTHOR_NAME"), String::from("Alice"));
        assert_eq!(env.get("GIT_AUTHOR_NAME").cloned(), Some(String::from("Alice")));
        assert_eq!(env.get("GIT_AUTHOR_EMAIL"), None);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        assert!(now.as_secs() > 1_700_000_000);
    }

    #[test]
    fn splits_sign_from_offset() {
        assert_eq!("+0900".split_at_checked(1), Some(("+", "0900")));
        assert_eq!("".split_at_checked(1), None);
    }
}
```

課題3で`open()`を外すと，`Door<Closed>`には`walk_through`がないというエラーになる．

```text
error[E0599]: no method named `walk_through` found for struct `Door<Closed>` in the current scope
  --> src/lib.rs:72:25
   |
42 |     struct Door<S> {
   |     -------------- method `walk_through` not found for this struct
...
72 |         assert_eq!(door.walk_through(), "walked through front");
   |                         ^^^^^^^^^^^^ method not found in `Door<Closed>`
   |
   = note: the method was found for `Door<Open>`
```

## 6-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

- `tree`の並び順は，本物の`git`で確かめた`a-b`，`a.txt`，`a`の順を単体テストにした．結合テストでも，同じ名前のファイルを含む作業ディレクトリで`git write-tree`と比べる．
- `commit`は，署名，ビルダー，直列化，解析の順に並べた．直列化の期待値は，`git cat-file -p`の出力と同じ形である．
- ビルダーの「`tree`を呼ばないと`build`できない」ことはコンパイルエラーなので，テストにはできない．6-5でエラーを確かめた．
- `commit-tree`の結合テストは，`rgit`と本物の`git`に同じ環境変数を渡してIDを比べる．IDが一致すれば，内容の1バイトまで一致している．
- `cli::run`の引数が増えるので，既存のテストを補助関数経由にそろえる項目を加えた．

## 6-4 図

模範解答は[design/types.md](../design/types.md)にある．Iteration 5からの変更は次のとおりである．

- `commit`の名前空間に，`Signature`，`Commit`，`Missing`，`CommitBuilder~T, A, C~`，`commit_mod`(`parse_offset`)を描いた．
- `Commit`は`Signature`と`ObjectId`を持つ．`CommitBuilder`は`build`で`Commit`を作り，`Missing`を型引数に使う．
- ビルダーの型の変わり方(`tree`が`T`を`ObjectId`にする，など)は図で表せないので，図の下に書いた．
- `tree_mod`に`compare_entries`と`tree_bytes`を，`object_mod`に`hash_object`を，`Repository`に`write_object`と`write_tree`を加えた．
- `cli_mod`の`run`に`env`の引数を加え，`cli`から`commit`への依存を描いた．

## 6-5 テスト駆動の実装

### treeの並び順

```rust
#[test]
fn directory_is_ordered_as_if_followed_by_slash() {
    let dir = named(Mode::Directory, "a");
    let dash = named(Mode::File, "a-b");
    let dot = named(Mode::File, "a.txt");
    assert_eq!(compare_entries(&dash, &dir), Ordering::Less);
    assert_eq!(compare_entries(&dot, &dir), Ordering::Less);
    let file = named(Mode::File, "a");
    assert_eq!(compare_entries(&file, &dot), Ordering::Less);
}
```

1つ目の項目(`a.txt`と`b.txt`)は，名前だけを比べる`a.name.cmp(b.name)`で通る．このテストは，その形では失敗する．

```text
thread 'tree::tests::directory_is_ordered_as_if_followed_by_slash' (8277) panicked at src/tree.rs:287:9:
assertion `left == right` failed
  left: Greater
 right: Less
```

ディレクトリのときだけ`/`を後ろにつないで比べる．

```rust
/// treeのエントリーの並び順．名前のバイト順だが，ディレクトリは名前の後ろに`/`があるものとして比べる．
pub fn compare_entries(a: &TreeEntry, b: &TreeEntry) -> Ordering {
    let slash = |entry: &TreeEntry| (entry.mode == Mode::Directory).then_some(b'/');
    let a_key = a.name.bytes().chain(slash(a));
    let b_key = b.name.bytes().chain(slash(b));
    a_key.cmp(b_key)
}
```

ファイル`a`は`a.txt`より前のままである．名前が`a`で終わり，後ろに何もつながないので，短い方が小さい．

### `tree_bytes`

```rust
#[test]
fn tree_bytes_sorts_entries_and_parses_back() {
    let entries = vec![
        named(Mode::Directory, "a"),
        named(Mode::File, "a.txt"),
        named(Mode::Executable, "a-b"),
    ];
    let bytes = tree_bytes(entries);
    let parsed = parse_tree(&bytes).unwrap();
    let names: Vec<&str> = parsed.iter().map(|entry| entry.name).collect();
    assert_eq!(names, ["a-b", "a.txt", "a"]);
    assert_eq!(parsed[0].mode, Mode::Executable);
    assert_eq!(parsed[2].mode, Mode::Directory);
}
```

```rust
/// エントリーをGitの順に並べ，treeオブジェクトの内容のバイト列にする．
pub fn tree_bytes(mut entries: Vec<TreeEntry<'_>>) -> Vec<u8> {
    entries.sort_by(compare_entries);
    let mut bytes = Vec::new();
    for entry in &entries {
        bytes.extend_from_slice(format!("{:o} {}\0", entry.mode.bits(), entry.name).as_bytes());
        bytes.extend_from_slice(entry.id.as_bytes());
    }
    bytes
}
```

`{:o}`は8進数で書く．`0o40000`は`40000`になり，treeの中の表記と一致する．Iteration 4の`TryFrom<&[u8]>`で読み戻せることを，往復で確かめた．

### `write_tree`

種類を引数で受け取る`write_object`を作り，`write_blob`はそれを呼ぶだけの形にした．IDの計算も，種類を問わない`hash_object`を`object`に加えた．

```rust
/// データをblobオブジェクトとして書き込み，そのIDを返す．
pub fn write_blob(&self, data: &[u8]) -> Result<ObjectId, Error> {
    self.write_object(ObjectKind::Blob, data)
}
```

空のインデックスの項目は，最上位のtreeを1つ書くだけで通る．入れ子の項目を通すために，ディレクトリごとの再帰を加えた．

```rust
/// インデックスからtreeオブジェクトを作って書き込み，最上位のtreeのIDを返す．
pub fn write_tree(&self, index: &Index) -> Result<ObjectId, Error> {
    let entries: Vec<(&str, &IndexEntry)> = index
        .entries()
        .iter()
        .map(|(path, entry)| (path.as_str(), entry))
        .collect();
    self.write_subtree(&entries)
}

/// パスの順に並んだエントリーから，1つのディレクトリのtreeを書く．
/// パスは，このディレクトリからの相対パスである．
fn write_subtree(&self, entries: &[(&str, &IndexEntry)]) -> Result<ObjectId, Error> {
    let mut tree = Vec::new();
    let mut rest = entries;
    while let Some(&(path, entry)) = rest.first() {
        match path.split_once('/') {
            None => {
                tree.push(TreeEntry {
                    mode: entry.mode,
                    name: path,
                    id: entry.id,
                });
                rest = &rest[1..];
            }
            Some((dir, _)) => {
                let prefix = format!("{dir}/");
                let end = rest
                    .iter()
                    .position(|(path, _)| !path.starts_with(&prefix))
                    .unwrap_or(rest.len());
                let children: Vec<(&str, &IndexEntry)> = rest[..end]
                    .iter()
                    .map(|&(path, entry)| (&path[prefix.len()..], entry))
                    .collect();
                let id = self.write_subtree(&children)?;
                tree.push(TreeEntry {
                    mode: Mode::Directory,
                    name: dir,
                    id,
                });
                rest = &rest[end..];
            }
        }
    }
    self.write_object(ObjectKind::Tree, &tree_bytes(tree))
}
```

- `TreeEntry`の名前と子の列のパスは，どれもインデックスの`String`の一部を借りている．文字列を1つも複製せずにtreeを組み立てられる．
- `position`は，`prefix`で始まらない最初の位置を探す．見つからなければ，残りすべてがそのディレクトリの中である．
- `tree_bytes`が並べ替えるので，`write_subtree`はエントリーを順不同で`push`してよい．

### 署名

```rust
#[test]
fn signature_is_shown_with_offset() {
    assert_eq!(
        alice().to_string(),
        "Alice <alice@example.com> 1767225600 +0900"
    );
    let west = Signature {
        offset_minutes: -90,
        ..alice()
    };
    assert!(west.to_string().ends_with(" -0130"));
}
```

```rust
impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.offset_minutes < 0 { '-' } else { '+' };
        let offset = self.offset_minutes.abs();
        write!(
            f,
            "{} <{}> {} {sign}{:02}{:02}",
            self.name,
            self.email,
            self.time,
            offset / 60,
            offset % 60
        )
    }
}
```

ずれを分の数で持つと，`-0130`の時間と分を，割り算と余りで書ける．文字列のまま持つと，`+900`のような不正な値を持ててしまう．
解析は`split_once`で区切りの順に分け，ずれを`parse_offset`で読む．

```rust
/// `+0900`や`-0130`を，分の数にする．
pub fn parse_offset(text: &str) -> Option<i32> {
    let (sign, digits) = match text.split_at_checked(1)? {
        ("+", digits) => (1, digits),
        ("-", digits) => (-1, digits),
        _ => return None,
    };
    if digits.len() != 4 {
        return None;
    }
    let hours: i32 = digits[..2].parse().ok()?;
    let minutes: i32 = digits[2..].parse().ok()?;
    Some(sign * (hours * 60 + minutes))
}
```

`match`のパターンに，組と文字列のリテラルを組み合わせて書ける．

### ビルダー

```rust
#[test]
fn builder_makes_commit_with_parents_in_order() {
    let tree = hash_blob(b"tree");
    let first = hash_blob(b"first");
    let second = hash_blob(b"second");
    let commit = Commit::builder()
        .message("merge\n")
        .parent(first)
        .parent(second)
        .tree(tree)
        .committer(alice())
        .author(alice())
        .build();
    assert_eq!(commit.tree, tree);
    assert_eq!(commit.parents, [first, second]);
    assert_eq!(commit.message, "merge\n");
}
```

テストのIDは，どれも`hash_blob`で作った値である．ビルダーはIDの指すオブジェクトの種類を調べないので，どんなIDでも確かめられる．
ビルダーは，ノートのとおり`CommitBuilder<T, A, C>`と`Missing`で作った．`parent`と`message`は型を変えないので，`mut self`で受け取って書き換えて返す．

```rust
pub fn parent(mut self, parent: ObjectId) -> CommitBuilder<T, A, C> {
    self.parents.push(parent);
    self
}
```

`tree`を呼ばずに`build`を呼ぶと，コンパイルエラーになる([Rustのノート](../../../../docs/rust/iteration-06.md)にエラーの全文がある)．

### 直列化と解析

```rust
pub fn to_bytes(&self) -> Vec<u8> {
    let mut text = format!("tree {}\n", self.tree);
    for parent in &self.parents {
        text.push_str(&format!("parent {parent}\n"));
    }
    text.push_str(&format!("author {}\n", self.author));
    text.push_str(&format!("committer {}\n", self.committer));
    text.push('\n');
    text.push_str(&self.message);
    text.into_bytes()
}
```

解析は，`split_once("\n\n")`でヘッダーとメッセージに分け，ヘッダーを1行ずつ`match`で振り分ける．
`tree`，`author`，`committer`は，ヘッダーを読み終えてからそろっているかを調べるので，`Option`で持つ．

```rust
let mut tree = None;
let mut parents = Vec::new();
let mut author = None;
let mut committer = None;
for line in headers.lines() {
    let (key, value) = line.split_once(' ').ok_or_else(invalid)?;
    match key {
        "tree" => tree = Some(value.parse().map_err(|_| invalid())?),
        "parent" => parents.push(value.parse().map_err(|_| invalid())?),
        "author" => author = Some(Signature::parse(value)?),
        "committer" => committer = Some(Signature::parse(value)?),
        _ => return Err(Error::CorruptObject("unsupported commit header")),
    }
}
```

`value.parse()`の型は，`tree`が`Option<ObjectId>`であることから決まる．
往復のテストは，親と，空行を含むメッセージ(`second\n\nbody\n`)で確かめた．最初の`\n\n`だけで分けるので，メッセージの中の空行は残る．

### `commit-tree`

`cli::run`に`env: &HashMap<String, String>`を加えた．`main.rs`は`std::env::vars().collect()`で作って渡す．
環境変数から署名を作る関数は，`cli`の非公開の関数にした．

```rust
fn signature_from_env(env: &HashMap<String, String>, role: &str) -> Result<Signature, Error> {
    let var = |field: &str| {
        let name = format!("GIT_{role}_{field}");
        env.get(&name).cloned().ok_or(Error::MissingVariable(name))
    };
    let name = var("NAME")?;
    let email = var("EMAIL")?;
    let (time, offset_minutes) = match var("DATE") {
        Ok(date) => parse_date(&date).ok_or(Error::InvalidDate(date))?,
        Err(_) => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            (now.as_secs() as i64, 0)
        }
    };
    Ok(Signature {
        name,
        email,
        time,
        offset_minutes,
    })
}
```

`var`は，`role`と`env`を使うクロージャである．`GIT_AUTHOR_NAME`と`GIT_COMMITTER_NAME`を同じ関数で読める．

```rust
Command::CommitTree {
    tree,
    parents,
    message,
} => {
    let repo = Repository::discover(cwd)?;
    let tree = repo.resolve_prefix(&tree)?;
    let mut builder = Commit::builder()
        .tree(tree)
        .author(signature_from_env(env, "AUTHOR")?)
        .committer(signature_from_env(env, "COMMITTER")?)
        .message(&format!("{message}\n"));
    for parent in parents {
        builder = builder.parent(repo.resolve_prefix(&parent)?);
    }
    let commit = builder.build();
    let id = repo.write_object(ObjectKind::Commit, &commit.to_bytes())?;
    writeln!(out, "{id}")?;
}
```

`builder`は`let mut`で持ち，`parent`の結果を代入し直す．`parent`は型を変えないので，同じ変数に入れられる．

### 結合テストの補助関数

`tests/common/mod.rs`に，作者とコミッターの環境変数を作る`identity`を加え，`rgit`と`git`の両方に渡した．
`run`の引数が変わったので，`run`を直接呼んでいたエラーのテストは，補助関数`rgit_error`を使う形にそろえた．
`common`の関数は結合テストのファイルごとに使うものが違うので，ファイルの先頭に`#![allow(dead_code)]`を書き，使わない関数の警告を出さないようにした．

## 6-6 振り返り

1. 模範解答の`commit`の項目は，小さな部品(署名)から大きな部品(コミット)の順に並んでいる．
2. フィールドを直接書く方法は短いが，フィールドが増えるたびに，作るすべての場所を直す必要がある．ビルダーは，必要なものがそろっていることを保証する代わりに，型と`impl`が増える．
3. `Option`で持つ設計では，treeの指定忘れは実行したときに，`build`が返すエラーで見つかる．型状態パターンでは，コンパイルしたときに，書いた本人が見つける．
4. `run`の中で`std::env::var`を呼ぶと，テストはプロセスの環境変数を書き換える必要がある．テストは並行に実行されるので，テスト同士が干渉する．引数で渡せば，テストごとに別の環境変数を使える．
5. 図に描いた型と関数は，コードと一致している．

## 6-7 発展課題

`LsTree`の処理で，オブジェクトがコミットなら，`Commit::parse`でtreeのIDを取り出し，そのtreeを読み直す．

```rust
let (mut kind, mut content) = repo.read_object(id)?;
if kind == ObjectKind::Commit {
    let commit = Commit::parse(&content)?;
    (kind, content) = repo.read_object(commit.tree)?;
}
if kind != ObjectKind::Tree {
    return Err(Error::NotATree);
}
write_tree_entries(&content, out)?;
```

`(kind, content) = …`は，組の要素を既存の変数にまとめて代入する．
テストでは，コミットの`ls-tree`が，そのtreeの`ls-tree`と同じ出力になることを確かめる．
