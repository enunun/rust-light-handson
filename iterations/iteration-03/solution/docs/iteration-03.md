# Iteration 3：`cat-file`(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 3-1 準備

`Cargo.toml`は，Iteration 2の模範解答からパッケージ名だけを変えたものである．依存は変わらない．

## 3-2 文法と概念

課題の解答例である．

```rust
#[cfg(test)]
mod practice {
    #[test]
    fn splits_at_first_nul() {
        let data = b"blob 6\0hello\n";
        let nul = data.iter().position(|byte| *byte == 0).unwrap();
        assert_eq!(nul, 6);
        let (head, rest) = data.split_at(nul);
        assert_eq!(head, b"blob 6");
        assert_eq!(rest, b"\0hello\n");
    }

    fn after_nul(data: &[u8]) -> Option<&[u8]> {
        let nul = data.iter().position(|byte| *byte == 0)?;
        Some(&data[nul + 1..])
    }

    #[test]
    fn returns_bytes_after_nul() {
        assert_eq!(after_nul(b"blob 6\0hello\n"), Some(&b"hello\n"[..]));
        assert_eq!(after_nul(b"blob"), None);
    }

    fn content(data: &[u8]) -> Vec<u8> {
        let copy = data.to_vec();
        copy[1..].to_vec()
    }

    #[test]
    fn content_is_copied() {
        assert_eq!(content(b"xabc"), b"abc");
    }

    #[test]
    fn converts_errors() {
        assert_eq!("x".parse::<u32>().map_err(|_| "bad"), Err("bad"));
        assert_eq!(None::<u32>.ok_or("none"), Err("none"));
    }
}
```

- 2：`Option`を返す関数の中では，`Option`に`?`を使える．`None`ならその場で`None`を返す．期待値の`&b"hello\n"[..]`は，配列への参照をスライスにしたものである．
- 3：`Vec<u8>`を返せば，呼び出す側が中身を所有するので，関数の中の値を借りずに済む．

## 3-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

- `object`は，種類の変換，`encode`，`parse_header`の順に並べた．`parse_header`は種類の変換を使う．
- ヘッダーの壊れ方は，NULがない場合と大きさが合わない場合の2つを確かめた．知らない種類は，`ObjectKind`の`FromStr`のテストで確かめた．
- `resolve_prefix`は，見つかる場合を1つのテストにまとめ，4桁，大文字の7桁，40桁を並べた．どれも同じ1つのIDになる．
- 結合テストは，本物の`git`が書いたオブジェクトを読む．`rgit`の書き込みと読み取りが同じ誤りをしていても，本物の`git`との比較で見つかる．

## 3-4 図

模範解答は[design/types.md](../design/types.md)にある．Iteration 2からの変更は次のとおりである．

- `object`に`ObjectKind`を加え，`object_mod`の関数を`encode`，`parse_header`，`hash_blob`にした．
- `Repository`に`read_object`と`resolve_prefix`を加え，`ObjectKind`への依存を描いた．
- `read_object`と`parse_header`の戻り値はタプルを含むので，`Result`とだけ書き，正確な型を図の下に書いた．
- `cli`に`CatFileMode`を加え，`Command`が所有する関係を描いた．
- `Error`に3つの列挙子を加えた．`ObjectKind`の`FromStr`は`Error`を返すので，`object_mod`から`Error`への依存を描いた．

## 3-5 テスト駆動の実装

### `ObjectKind`

```rust
#[test]
fn kinds_are_shown_and_parsed_by_name() {
    assert_eq!(ObjectKind::Commit.to_string(), "commit");
    assert_eq!("tree".parse::<ObjectKind>().unwrap(), ObjectKind::Tree);
}

#[test]
fn unknown_kind_is_corrupt() {
    let result = "tag".parse::<ObjectKind>();
    assert!(matches!(
        result,
        Err(Error::CorruptObject("unknown object type"))
    ));
}
```

```rust
/// オブジェクトの種類．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ObjectKind::Blob => "blob",
            ObjectKind::Tree => "tree",
            ObjectKind::Commit => "commit",
        };
        f.write_str(name)
    }
}

impl FromStr for ObjectKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<ObjectKind, Error> {
        match s {
            "blob" => Ok(ObjectKind::Blob),
            "tree" => Ok(ObjectKind::Tree),
            "commit" => Ok(ObjectKind::Commit),
            _ => Err(Error::CorruptObject("unknown object type")),
        }
    }
}
```

`Error`に`CorruptObject(&'static str)`を加えた．`matches!`のパターンには，文字列のリテラルも書ける．
`ObjectKind`は`Copy`にした．データを持たない列挙子だけの`enum`は小さいので，値で渡せば足りる．

### `encode`

`blob_bytes`のテストを`encode`のテストに書き換える．

```rust
#[test]
fn encode_starts_with_kind_and_size() {
    assert_eq!(encode(ObjectKind::Blob, b"hello\n"), b"blob 6\0hello\n");
}
```

```rust
/// オブジェクトのバイト列(`<種類> <大きさ>\0<内容>`)を作る．
pub fn encode(kind: ObjectKind, data: &[u8]) -> Vec<u8> {
    let mut bytes = format!("{kind} {}\0", data.len()).into_bytes();
    bytes.extend_from_slice(data);
    bytes
}
```

`hash_blob`と`Repository::write_blob`の`blob_bytes(data)`を，`encode(ObjectKind::Blob, data)`に置き換えた．

### `parse_header`

```rust
#[test]
fn parse_header_splits_kind_and_content() {
    let (kind, content) = parse_header(b"blob 6\0hello\n").unwrap();
    assert_eq!(kind, ObjectKind::Blob);
    assert_eq!(content, b"hello\n");
}
```

最初は，大きさを読んで捨てる形で通した．

```rust
/// オブジェクトのバイト列を，種類と内容に分ける．内容は`data`の一部を借りて返す．
pub fn parse_header(data: &[u8]) -> Result<(ObjectKind, &[u8]), Error> {
    let nul = data
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(Error::CorruptObject("missing header"))?;
    let (header, rest) = data.split_at(nul);
    let content = &rest[1..];
    let header =
        std::str::from_utf8(header).map_err(|_| Error::CorruptObject("invalid header"))?;
    let (kind, size) = header
        .split_once(' ')
        .ok_or(Error::CorruptObject("invalid header"))?;
    let kind: ObjectKind = kind.parse()?;
    let _ = size;
    Ok((kind, content))
}
```

`kind.parse()?`の`?`は，`FromStr`の`Err`が`Error`なので，変換せずにそのまま返す．
NULのない場合のテストは，`ok_or`があるので，この形で通る．

```rust
#[test]
fn header_without_nul_is_corrupt() {
    let result = parse_header(b"blob 6");
    assert!(matches!(result, Err(Error::CorruptObject("missing header"))));
}

#[test]
fn size_must_match_content() {
    let result = parse_header(b"blob 5\0hello\n");
    assert!(matches!(result, Err(Error::CorruptObject("size mismatch"))));
}
```

大きさのテストは失敗する．

```text
thread 'object::tests::size_must_match_content' (15529) panicked at src/object.rs:117:9:
assertion failed: matches!(result, Err(Error::CorruptObject("size mismatch")))
```

`let _ = size;`を，大きさを数にして比べる処理に置き換える．

```rust
let size: usize = size
    .parse()
    .map_err(|_| Error::CorruptObject("invalid size"))?;
if size != content.len() {
    return Err(Error::CorruptObject("size mismatch"));
}
```

### `read_object`

```rust
#[test]
fn read_object_returns_kind_and_content_of_written_blob() {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let id = repo.write_blob(b"hello\n").unwrap();
    let (kind, content) = repo.read_object(id).unwrap();
    assert_eq!(kind, ObjectKind::Blob);
    assert_eq!(content, b"hello\n");
}

#[test]
fn read_object_reports_missing_object() {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let id = hash_blob(b"hello\n");
    let result = repo.read_object(id);
    assert!(matches!(result, Err(Error::ObjectNotFound(name)) if name == id.to_string()));
}
```

```rust
/// オブジェクトを読み，種類と内容を返す．
pub fn read_object(&self, id: ObjectId) -> Result<(ObjectKind, Vec<u8>), Error> {
    let file = match fs::File::open(self.object_path(id)) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(Error::ObjectNotFound(id.to_string()));
        }
        Err(error) => return Err(error.into()),
    };
    let mut data = Vec::new();
    ZlibDecoder::new(file).read_to_end(&mut data)?;
    let (kind, content) = parse_header(&data)?;
    Ok((kind, content.to_vec()))
}
```

`content`は`data`を借りているので，そのまま返すと`data`より長く使うことになり，コンパイルエラーになる．`to_vec()`で複製して返す．
2つ目のテストの`matches!`のガード(`if name == …`)は，エラーが含むIDも確かめる．

### `resolve_prefix`

```rust
#[test]
fn resolve_prefix_finds_unique_object() {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let id = repo.write_blob(b"hello\n").unwrap();
    assert_eq!(repo.resolve_prefix("ce01").unwrap(), id);
    assert_eq!(repo.resolve_prefix("CE01362").unwrap(), id);
    assert_eq!(repo.resolve_prefix(&id.to_string()).unwrap(), id);
}
```

最初の形は，ディレクトリの中で前方一致するファイルを1つ探すだけである．

```rust
/// 4桁以上40桁以下の16進数を，それで始まるただ1つのオブジェクトのIDにする．
pub fn resolve_prefix(&self, prefix: &str) -> Result<ObjectId, Error> {
    let not_found = || Error::ObjectNotFound(prefix.to_string());
    let prefix_lower = prefix.to_ascii_lowercase();
    let (dir_name, rest) = prefix_lower.split_at(2);
    let dir = self.git_dir.join("objects").join(dir_name);
    let mut found = Vec::new();
    for entry in fs::read_dir(dir)? {
        let name = entry?.file_name();
        if let Some(name) = name.to_str()
            && name.starts_with(rest)
        {
            found.push(format!("{dir_name}{name}"));
        }
    }
    found[0].parse().map_err(|_| not_found())
}
```

`found[0].parse()`の結果は`Result<ObjectId, ParseObjectIdError>`なので，`map_err`で`Error`に変える．
続く3つのテストで，桁数の検査，ディレクトリがない場合，0個と2個以上の場合を加えた．

```rust
let is_hex = prefix.chars().all(|ch| ch.is_ascii_hexdigit());
if prefix.len() < 4 || prefix.len() > 40 || !is_hex {
    return Err(not_found());
}
// …
if !dir.is_dir() {
    return Err(not_found());
}
// …
match found.len() {
    0 => Err(not_found()),
    1 => found[0].parse().map_err(|_| not_found()),
    _ => Err(Error::AmbiguousObject(prefix.to_string())),
}
```

16進数の検査がないと，`../..`のような文字列でディレクトリの外を読もうとする．
`match`で数を分けると，`found[0]`は要素が1つのときだけ読むので，空の`Vec`の添字でパニックにならない．

### `cat-file`

`tests/cat_file.rs`に，本物の`git`でオブジェクトを書く補助関数を置き，`-t`，`-s`，`-p`の順にテストを加えた．

```rust
/// 本物の`git`でリポジトリを作り，`hello.txt`をオブジェクトとして書く．
fn repository_with_hello() -> TempDir {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "-q"]);
    fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
    git(dir.path(), &["hash-object", "-w", "hello.txt"]);
    dir
}
```

`TempDir`を返すと，所有権が呼び出し元に移り，テストの終わりまでディレクトリが残る．

`cli.rs`にフラグの構造体と列挙子を加え，`match`に腕を足す．

```rust
/// `cat-file`で表示するもの．どれか1つだけを指定する．
#[derive(Args)]
#[group(required = true, multiple = false)]
struct CatFileMode {
    /// Show the object type
    #[arg(short = 't')]
    kind: bool,
    /// Show the object size
    #[arg(short = 's')]
    size: bool,
    /// Show the object content
    #[arg(short = 'p')]
    pretty: bool,
}
```

```rust
Command::CatFile { mode, object } => {
    let repo = Repository::discover(cwd)?;
    let id = repo.resolve_prefix(&object)?;
    let (kind, content) = repo.read_object(id)?;
    if mode.kind {
        writeln!(out, "{kind}")?;
    } else if mode.size {
        writeln!(out, "{}", content.len())?;
    } else {
        out.write_all(&content)?;
    }
}
```

グループが「どれか1つ」を保証するので，`-t`と`-s`のどちらでもなければ`-p`である．`pretty`のフィールドは読まない．
commitのテストは`git cat-file`の出力と比べるので，コミットの日時が実行のたびに変わっても通る．

## 3-6 振り返り

1. 模範解答のエラーの項目は，テストを書くたびに実装が1つ増える順に並んでいる．
2. スライスを返すと，呼び出す側は，元のバイト列を手放す前に内容を使い終えるか，複製する必要がある．`Vec<u8>`を返す設計なら制約はないが，毎回複製が起きる．
3. 理由を文字列で持つと，テストは`matches!`で文字列まで比べられる．列挙子を分ければ，綴りの誤りをコンパイラーが見つけるが，`Error`の列挙子が増える．
4. 本物の`git`が書いたものを読むテストは，`rgit`の読み取りが本物の形式に合っていることを保証する．`rgit`同士のテストは，書き込みと読み取りが互いに合っていることだけを保証する．
5. 図に描いた型と関数は，コードと一致している．

## 3-7 発展課題

`AmbiguousObject`を，短縮形と候補を持つ列挙子にする．
thiserrorの`#[error]`には，フィールドを使う式を書ける．

```rust
#[error("short object ID {prefix} is ambiguous{}", candidate_hints(candidates))]
AmbiguousObject {
    prefix: String,
    candidates: Vec<String>,
},
```

```rust
/// 曖昧な短縮形の候補を，1行に1つの`hint:`の行にする．
fn candidate_hints(candidates: &[String]) -> String {
    let mut hints = String::from("\nhint: The candidates are:");
    for candidate in candidates {
        hints.push_str(&format!("\nhint:   {candidate}"));
    }
    hints
}
```

`resolve_prefix`では，見つかったIDを`sort`で並べ，先頭の7桁を候補にする．

```rust
_ => {
    found.sort();
    let mut candidates = Vec::new();
    for hex in &found {
        candidates.push(hex[..7].to_string());
    }
    Err(Error::AmbiguousObject {
        prefix: prefix.to_string(),
        candidates,
    })
}
```

`main.rs`は`fatal: {error}`と表示するので，1行目にだけ`fatal:`が付く．
