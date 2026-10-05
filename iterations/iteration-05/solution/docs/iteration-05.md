# Iteration 5：インデックス，`add`，`ls-files`(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 5-1 準備

`Cargo.toml`は，Iteration 4の模範解答からパッケージ名だけを変えたものである．依存は変わらない．

## 5-2 文法と概念

課題の解答例である．

```rust
#[cfg(test)]
mod practice {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    #[test]
    fn big_endian_bytes() {
        assert_eq!(2u32.to_be_bytes(), [0, 0, 0, 2]);
        assert_eq!(0x1234_5678u32.to_be_bytes(), [0x12, 0x34, 0x56, 0x78]);
        assert_eq!(u32::from_be_bytes([0, 0, 1, 0]), 256);
        assert_eq!(0x1234_5678u32.to_le_bytes(), [0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn btree_map_iterates_in_key_order() {
        let mut map = BTreeMap::new();
        map.insert("src/main.rs", 2);
        map.insert("hello.txt", 1);
        map.insert("src/lib.rs", 3);
        let keys: Vec<&&str> = map.keys().collect();
        assert_eq!(keys, [&"hello.txt", &"src/lib.rs", &"src/main.rs"]);
        map.insert("hello.txt", 10);
        assert_eq!(map["hello.txt"], 10);
    }

    fn count_files(dir: &Path) -> std::io::Result<usize> {
        let mut count = 0;
        let entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                count += count_files(&path)?;
            } else {
                count += 1;
            }
        }
        Ok(count)
    }

    #[test]
    fn counts_files_recursively() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        fs::write(dir.path().join("x.txt"), "").unwrap();
        fs::write(dir.path().join("a/y.txt"), "").unwrap();
        fs::write(dir.path().join("a/b/z.txt"), "").unwrap();
        assert_eq!(count_files(dir.path()).unwrap(), 3);
    }

    #[test]
    fn filters_paths_with_closure() {
        let paths = [
            String::from("hello.txt"),
            String::from("src/lib.rs"),
            String::from("src/main.rs"),
        ];
        let prefix = "src/";
        let under: Vec<String> = paths
            .iter()
            .filter(|path| path.starts_with(prefix))
            .cloned()
            .collect();
        assert_eq!(under, ["src/lib.rs", "src/main.rs"]);
    }

    fn padded_entry_len(path_len: usize) -> usize {
        let len = 62 + path_len;
        len + 8 - len % 8
    }

    #[test]
    fn entries_are_padded_with_one_to_eight_nuls() {
        assert_eq!(padded_entry_len(9), 72);
        assert_eq!(padded_entry_len(11), 80);
        assert_eq!(padded_entry_len(10), 80);
    }
}
```

- 2：`keys()`の要素は`&&str`(キー`&str`への参照)である．同じキーで`insert`すると，値が置き換わる．
- 4：配列の要素を`vec!`で`Vec`にしなくても，配列のまま`iter()`を呼べる．`vec!`で作ると，`cargo clippy`が不要な`vec!`だと指摘する．
- 5：パスが10バイトなら固定部と合わせてちょうど72バイトになるが，NULを8個足して80バイトにする．パスの後ろに，必ず1個以上のNULがある．

## 5-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

- `index`は，空のインデックス，1エントリーのバイト配置，3エントリーの往復，壊れたチェックサムの順に並べた．
- 1エントリーのテストは，バイト列の長さとパスの位置を直接確かめる．往復のテストは，読みと書きが互いに合っていることを確かめる．両方が本物の`git`と合っていることは，結合テストで確かめる．
- `worktree`と`repo`は，ファイルシステムを使う単体テストである．`TempDir`に作業ディレクトリを作って確かめる．
- 結合テストは，`rgit`が書いたインデックスを本物の`git`が読むテストと，その逆のテストを両方含む．

## 5-4 図

模範解答は[design/types.md](../design/types.md)にある．Iteration 4からの変更は次のとおりである．

- `index`の名前空間を加え，`Index`，`IndexEntry`，`Stat`，`Reader~'a~`を描いた．`Index`は`IndexEntry`を，`IndexEntry`は`Stat`，`Mode`，`ObjectId`を値として持つ．
- `worktree`の名前空間に，2つの公開関数を書いた．
- `Repository`に`work_dir`，`index_path`，`add`を，`Mode`に`bits`と`TryFrom~u32~`を，`ObjectId`に`as_bytes`を加えた．
- `Repository`から`Index`と`worktree_mod`への依存，`cli_mod`から`Index`への依存(`ls-files`)を描いた．

## 5-5 テスト駆動の実装

### モードと数

`tree.rs`に，インデックスの数との変換を加えた．`Mode::try_from`は`&[u8]`と`u32`の2つになる．

```rust
/// インデックスに書く数としてのモード(`0o100644`など)．
pub fn bits(self) -> u32 {
    match self {
        Mode::File => 0o100644,
        Mode::Executable => 0o100755,
        Mode::Symlink => 0o120000,
        Mode::Directory => 0o40000,
        Mode::Submodule => 0o160000,
    }
}
```

### 空のインデックス

```rust
#[test]
fn empty_index_has_header_and_checksum() {
    let bytes = Index::default().to_bytes();
    assert_eq!(&bytes[..12], b"DIRC\0\0\0\x02\0\0\0\0");
    assert_eq!(bytes.len(), 12 + 20);
    assert_eq!(&bytes[12..], Sha1::digest(&bytes[..12]).as_slice());
}
```

`Index`に`#[derive(Default)]`を付け，ヘッダーとチェックサムだけを書く`to_bytes`を作った．

```rust
pub fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"DIRC");
    bytes.extend_from_slice(&2u32.to_be_bytes());
    bytes.extend_from_slice(&(self.entries.len() as u32).to_be_bytes());
    let checksum = Sha1::digest(&bytes);
    bytes.extend_from_slice(&checksum);
    bytes
}
```

### 1エントリーのバイト配置

```rust
fn entry(mode: Mode, data: &[u8]) -> IndexEntry {
    IndexEntry {
        stat: Stat {
            mtime: 1767225600,
            size: data.len() as u32,
            ..Stat::default()
        },
        mode,
        id: hash_blob(data),
    }
}

#[test]
fn entry_is_padded_to_multiple_of_eight() {
    let mut index = Index::default();
    index.insert("hello.txt".to_string(), entry(Mode::File, b"hello\n"));
    let bytes = index.to_bytes();
    // 62バイトの固定部と9バイトのパスの後ろに，1個のNULを足して72バイトにする．
    assert_eq!(bytes.len(), 12 + 72 + 20);
    assert_eq!(&bytes[12 + 62..12 + 71], b"hello.txt");
    assert_eq!(bytes[12 + 71], 0);
}
```

エントリーを書くループを`to_bytes`に加えた．`Stat`は`Copy`なので，`let stat = entry.stat;`で取り出せる．

```rust
for (path, entry) in &self.entries {
    let entry_start = bytes.len();
    let stat = entry.stat;
    for value in [
        stat.ctime,
        stat.ctime_nsec,
        stat.mtime,
        stat.mtime_nsec,
        stat.dev,
        stat.ino,
        entry.mode.bits(),
        stat.uid,
        stat.gid,
        stat.size,
    ] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes.extend_from_slice(entry.id.as_bytes());
    let flags = path.len().min(0x0fff) as u16;
    bytes.extend_from_slice(&flags.to_be_bytes());
    bytes.extend_from_slice(path.as_bytes());
    let written = bytes.len() - entry_start;
    bytes.resize(bytes.len() + 8 - written % 8, 0);
}
```

IDのバイト列を書くために，`ObjectId`に`as_bytes`を加えた．フラグの長さは12ビットなので，`min(0x0fff)`で上限を付ける．

### 往復

```rust
#[test]
fn entries_are_written_in_path_order_and_read_back() {
    let mut index = Index::default();
    index.insert("src/main.rs".to_string(), entry(Mode::File, b"fn main() {}\n"));
    index.insert("run.sh".to_string(), entry(Mode::Executable, b"echo\n"));
    index.insert("a.txt".to_string(), entry(Mode::File, b""));
    let parsed = Index::parse(&index.to_bytes()).unwrap();
    let paths: Vec<&String> = parsed.entries().keys().collect();
    assert_eq!(paths, ["a.txt", "run.sh", "src/main.rs"]);
    assert_eq!(parsed, index);
}
```

`assert_eq!(parsed, index)`のために，`Index`，`IndexEntry`，`Stat`に`PartialEq`と`Debug`を導出した．
`parse`は，`Reader`で先頭から読む．

```rust
let mut reader = Reader { rest: body };
if reader.take(4)? != b"DIRC" {
    return Err(Error::CorruptIndex("bad signature"));
}
if reader.u32()? != 2 {
    return Err(Error::CorruptIndex("unsupported version"));
}
let count = reader.u32()?;
let mut index = Index::default();
for _ in 0..count {
    let entry_start = reader.rest.len();
    let ctime = reader.u32()?;
    // … mtime，dev，ino，モード，uid，gid，大きさを順に読む
    let id: [u8; 20] = reader.take(20)?.try_into().unwrap();
    let flags = reader.u16()?;
    let name_len = usize::from(flags & 0x0fff);
    let path = std::str::from_utf8(reader.take(name_len)?)
        .map_err(|_| Error::CorruptIndex("path is not UTF-8"))?;
    // エントリーの長さが8の倍数になるまで，1〜8個のNULで埋めてある．
    let read = entry_start - reader.rest.len();
    reader.take(8 - read % 8)?;
    index.insert(path.to_string(), IndexEntry { stat, mode, id: ObjectId::from_bytes(id) });
}
```

`for _ in 0..count`の`_`は，回数だけを使い，番号を使わないことを表す．
エントリーの数だけ読んだら，残り(拡張)は読まずに終える．

### チェックサム

```rust
#[test]
fn broken_checksum_is_an_error() {
    let mut bytes = Index::default().to_bytes();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    assert!(matches!(
        Index::parse(&bytes),
        Err(Error::CorruptIndex("checksum mismatch"))
    ));
}
```

チェックサムを確かめる前の`parse`は，壊れたバイト列も読めてしまう．

```text
thread 'index::tests::broken_checksum_is_an_error' (30282) panicked at src/index.rs:262:9:
assertion failed: matches!(Index::parse(&bytes), Err(Error::CorruptIndex("checksum mismatch")))
```

先に末尾の20バイトを切り離して比べる．`^= 0xff`は，ビットを反転して最後のバイトを壊す．

```rust
let (body, checksum) = data.split_at(data.len() - 20);
if Sha1::digest(body).as_slice() != checksum {
    return Err(Error::CorruptIndex("checksum mismatch"));
}
```

### ファイルからの読み書き

`load`は，ファイルがなければ空のインデックスを返す．`git init`の直後には`.git/index`がないからである．

```rust
pub fn load(path: &Path) -> Result<Index, Error> {
    match fs::read(path) {
        Ok(data) => Index::parse(&data),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Index::default()),
        Err(error) => Err(error.into()),
    }
}
```

### `list_files`と`relative_path`

```rust
pub fn list_files(work_dir: &Path, dir: &Path) -> Result<Vec<String>, Error> {
    let mut files = Vec::new();
    collect_files(work_dir, dir, &mut files)?;
    Ok(files)
}

fn collect_files(work_dir: &Path, path: &Path, files: &mut Vec<String>) -> Result<(), Error> {
    if path.is_dir() {
        let entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        for entry in entries {
            if entry.file_name() != ".git" {
                collect_files(work_dir, &entry.path(), files)?;
            }
        }
    } else if path.is_file()
        && let Some(file) = relative_path(work_dir, path)
    {
        files.push(file);
    }
    Ok(())
}
```

再帰する補助関数は，結果を入れる`Vec`を`&mut`で受け取る．呼び出しのたびに`Vec`を作って連結せずに済む．
`read_dir`の順序は決まらないので，テストでは`sort()`してから比べた．インデックスに入れれば，`BTreeMap`がパスの順に並べる．

```rust
/// `path`を，`work_dir`からの`/`区切りのパスにする．`work_dir`の外のパスなら`None`を返す．
pub fn relative_path(work_dir: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(work_dir).ok()?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component.as_os_str().to_str() {
            Some("..") | None => return None,
            Some(".") => {}
            Some(part) => parts.push(part),
        }
    }
    Some(parts.join("/"))
}
```

`ok()`で`Result`を`Option`に変え，`?`で`None`を返す．`parts.join("/")`は，`&str`の列を`/`でつないだ`String`を作る．

### `Repository::add`

単体テストは，ディレクトリの指定，実行可能なファイル，消えたファイル，一致しないパスの順に加えた．
最終的な形は次のとおりである．

```rust
pub fn add(&self, cwd: &Path, pathspecs: &[PathBuf]) -> Result<(), Error> {
    let mut index = Index::load(&self.index_path())?;
    for spec in pathspecs {
        let spec_name = spec.display().to_string();
        let path = cwd.join(spec);
        let prefix = relative_path(&self.work_dir, &path)
            .ok_or_else(|| Error::OutsideRepository(spec_name.clone()))?;
        let files = list_files(&self.work_dir, &path)?;
        let tracked: Vec<String> = index
            .entries()
            .keys()
            .filter(|tracked| is_under(tracked, &prefix))
            .cloned()
            .collect();
        if files.is_empty() && tracked.is_empty() {
            return Err(Error::PathspecNotMatched(spec_name));
        }
        for tracked in tracked {
            if !files.contains(&tracked) {
                index.remove(&tracked);
            }
        }
        for file in files {
            let full_path = self.work_dir.join(&file);
            let metadata = fs::metadata(&full_path)?;
            let id = self.write_blob(&fs::read(&full_path)?)?;
            let mode = if metadata.permissions().mode() & 0o111 != 0 {
                Mode::Executable
            } else {
                Mode::File
            };
            let stat = Stat::from_metadata(&metadata);
            index.insert(file, IndexEntry { stat, mode, id });
        }
    }
    index.save(&self.index_path())
}
```

- `tracked`は，`index`から借りたキーを`cloned()`で複製して集める．借りたままだと，あとの`index.remove`で`index`を書き換えられない．
- `is_under`は，`src`と`srcs/a`を区別するため，`prefix`に`/`を足して比べる．
- 消えたファイルを除く処理は，ファイルを登録する前に行う．登録したパスを`files`から探して除いてしまわないためである．

### `ls-files`

```rust
Command::LsFiles { stage } => {
    let repo = Repository::discover(cwd)?;
    let index = Index::load(&repo.index_path())?;
    for (path, entry) in index.entries() {
        if stage {
            writeln!(out, "{} {} 0\t{path}", entry.mode, entry.id)?;
        } else {
            writeln!(out, "{path}")?;
        }
    }
}
```

`Add`の`paths`には`#[arg(required = true)]`を付け，パスのない`rgit add`を引数の誤りにした．
結合テストの`git status --porcelain`は，`rgit add`で書いたインデックスを本物の`git`が読み，2つのファイルがステージされていると表示することを確かめる．

## 5-6 振り返り

1. 模範解答は，インデックスの形式を3段階(空，1エントリー，往復)で確かめた．空のインデックスで，ヘッダーとチェックサムだけを先に固められる．
2. `Vec`で持つと，`add`で同じパスを探して置き換える処理と，`to_bytes`の前の並べ替えが要る．`BTreeMap`は，その両方を引き受ける．
3. `entries`を非公開にすると，エントリーの追加と削除は`insert`と`remove`を通る．並び順やパスの形の決まりを，`Index`の中で守れる．
4. 往復のテストは，読みと書きのどちらかの誤りを見つけるが，両方が同じように誤っていると見つけられない．バイト列を直接確かめるテストや本物の`git`との比較は，その誤りも見つける．
5. 図に描いた型と関数は，コードと一致している．

## 5-7 発展課題

`Rm`のサブコマンドに`--cached`と`paths`を持たせ，`Repository`に`remove_cached`を加える．

```rust
/// 指定したパスの下のファイルを，インデックスから除く．作業ディレクトリのファイルは変えない．
pub fn remove_cached(&self, cwd: &Path, pathspecs: &[PathBuf]) -> Result<(), Error> {
    let mut index = Index::load(&self.index_path())?;
    for spec in pathspecs {
        let spec_name = spec.display().to_string();
        let prefix = relative_path(&self.work_dir, &cwd.join(spec))
            .ok_or_else(|| Error::OutsideRepository(spec_name.clone()))?;
        let tracked: Vec<String> = index
            .entries()
            .keys()
            .filter(|tracked| is_under(tracked, &prefix))
            .cloned()
            .collect();
        if tracked.is_empty() {
            return Err(Error::PathspecNotMatched(spec_name));
        }
        for path in tracked {
            index.remove(&path);
        }
    }
    index.save(&self.index_path())
}
```

`--cached`は`#[arg(long, required = true)]`にし，作業ディレクトリのファイルを消す`git rm`の動作とは区別した．
結合テストでは，`rm --cached src`のあとで`ls-files`から`src/main.rs`が消え，ファイルは残っていることを確かめる．
