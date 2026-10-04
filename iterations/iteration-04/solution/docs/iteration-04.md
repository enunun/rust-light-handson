# Iteration 4：ツリーの読み取りと`ls-tree`(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 4-1 準備

`Cargo.toml`は，Iteration 3の模範解答からパッケージ名だけを変えたものである．依存は変わらない．

## 4-2 文法と概念

課題の解答例である．

```rust
#[cfg(test)]
mod practice {
    struct Word<'a> {
        text: &'a str,
    }

    fn first_word(s: &str) -> Word<'_> {
        let end = s.find(' ').unwrap_or(s.len());
        Word { text: &s[..end] }
    }

    #[test]
    fn first_word_borrows_from_input() {
        let line = String::from("100644 hello.txt");
        let word = first_word(&line);
        assert_eq!(word.text, "100644");
    }

    #[derive(Debug, PartialEq)]
    enum Bit {
        Zero,
        One,
    }

    impl TryFrom<u8> for Bit {
        type Error = String;

        fn try_from(value: u8) -> Result<Bit, String> {
            match value {
                0 => Ok(Bit::Zero),
                1 => Ok(Bit::One),
                _ => Err(format!("{value} is not a bit")),
            }
        }
    }

    #[test]
    fn bits_from_u8() {
        assert_eq!(Bit::try_from(1), Ok(Bit::One));
        assert_eq!(Bit::try_from(2), Err(String::from("2 is not a bit")));
        let zero: Result<Bit, String> = 0u8.try_into();
        assert_eq!(zero, Ok(Bit::Zero));
    }

    #[test]
    fn slices_become_arrays_only_with_right_length() {
        let bytes = [1u8, 2, 3, 4, 5];
        let four: [u8; 4] = bytes[..4].try_into().unwrap();
        assert_eq!(four, [1, 2, 3, 4]);
        let three: Result<[u8; 4], _> = bytes[..3].try_into();
        assert!(three.is_err());
    }
}
```

- 1：`find(' ')`は最初の空白の位置を`Option<usize>`で返す．空白がなければ全体を1語とするため，`unwrap_or(s.len())`にした．
- 2：ノートと同じ`E0597`(`does not live long enough`)になる．`Word`が借りている`String`が，内側の`{ }`の終わりで片付けられるからである．
- 3：`TryFrom`を実装すると，`try_into`も使える．`try_into`の変換先は，受け取る変数の型から決まる．

## 4-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

- `Mode`の変換，表示，`parse_tree`の順に並べた．`parse_tree`は`Mode::try_from`を使う．
- `parse_tree`は，空，2エントリー，壊れた形の順に並べた．2エントリーのテストは，1つ目を読んだあとに`rest`を正しく進めているかを確かめる．
- 結合テストは，本物の`git`が作った，4つのモードを含むtreeを使う．`ls-tree`と`cat-file -p`の出力を，`git`の出力と比べる．

## 4-4 図

模範解答は[design/types.md](../design/types.md)にある．Iteration 3からの変更は次のとおりである．

- `tree`の名前空間を加え，`Mode`，`TreeEntry~'a~`，`tree_mod`(`parse_tree`)を描いた．
- `TreeEntry`は`Mode`と`ObjectId`を値として持つので，所有の矢印を描いた．名前は`&'a str`で借りているので，フィールドの型にライフタイムを書いた．
- `Mode::kind`は`ObjectKind`を返すので，`Mode`から`ObjectKind`への依存を描いた．
- `parse_tree`の戻り値は入れ子の型引数を含むので，`Result`とだけ書き，正確な型を図の下に書いた．
- `cli`から`tree_mod`への依存と，`Error`の`NotATree`を加えた．

## 4-5 テスト駆動の実装

### `Mode`

```rust
#[test]
fn modes_are_read_from_tree_digits() {
    assert_eq!(Mode::try_from(&b"100644"[..]).unwrap(), Mode::File);
    assert_eq!(Mode::try_from(&b"100755"[..]).unwrap(), Mode::Executable);
    assert_eq!(Mode::try_from(&b"120000"[..]).unwrap(), Mode::Symlink);
    assert_eq!(Mode::try_from(&b"40000"[..]).unwrap(), Mode::Directory);
    assert_eq!(Mode::try_from(&b"160000"[..]).unwrap(), Mode::Submodule);
}
```

`&b"100644"[..]`は，`&[u8; 6]`をスライス`&[u8]`にする．`TryFrom<&[u8]>`を実装したので，配列への参照のままでは型が合わない．

```rust
/// treeの中のモードの表記(ディレクトリは`40000`)から作る．
impl TryFrom<&[u8]> for Mode {
    type Error = Error;

    fn try_from(digits: &[u8]) -> Result<Mode, Error> {
        match digits {
            b"100644" => Ok(Mode::File),
            b"100755" => Ok(Mode::Executable),
            b"120000" => Ok(Mode::Symlink),
            b"40000" => Ok(Mode::Directory),
            b"160000" => Ok(Mode::Submodule),
            _ => Err(Error::CorruptObject("unknown mode")),
        }
    }
}
```

`match`はすべての値を扱う必要があるので，最初の項目を通す時点で`_`の腕も書く．そのため，知らない表記の項目も同時に通る．

表示と種類のテストで，`Display`と`kind`を加えた．

```rust
impl Mode {
    /// このモードのエントリーが指すオブジェクトの種類．
    pub fn kind(self) -> ObjectKind {
        match self {
            Mode::Directory => ObjectKind::Tree,
            Mode::Submodule => ObjectKind::Commit,
            Mode::File | Mode::Executable | Mode::Symlink => ObjectKind::Blob,
        }
    }
}
```

`Mode::File | Mode::Executable | Mode::Symlink`は，3つのどれかに一致するパターンである．
`kind`は`self`を値で受け取る．`Mode`は`Copy`なので，呼んだあとも元の値を使える．

### `parse_tree`

テストの中に，エントリーのバイト列を作る補助関数を書いた．

```rust
/// treeの内容の1つのエントリーのバイト列を作る．
fn entry_bytes(mode: &str, name: &str, id: ObjectId) -> Vec<u8> {
    let mut bytes = format!("{mode} {name}\0").into_bytes();
    let hex = id.to_string();
    for i in 0..20 {
        bytes.push(u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap());
    }
    bytes
}
```

`ObjectId`の20バイトはモジュールの外から読めないので，16進数の表示から2桁ずつ戻した．

空のtreeは，空の`Vec`を返す仮実装で通した．2エントリーのテストで，`rest`を読み進めるループを書いた．

```rust
/// treeオブジェクトの内容を，エントリーの列にする．
pub fn parse_tree(data: &[u8]) -> Result<Vec<TreeEntry<'_>>, Error> {
    let corrupt = || Error::CorruptObject("invalid tree entry");
    let mut entries = Vec::new();
    let mut rest = data;
    while !rest.is_empty() {
        let space = rest
            .iter()
            .position(|byte| *byte == b' ')
            .ok_or_else(corrupt)?;
        let mode = Mode::try_from(&rest[..space])?;
        rest = &rest[space + 1..];

        let nul = rest
            .iter()
            .position(|byte| *byte == 0)
            .ok_or_else(corrupt)?;
        let name = std::str::from_utf8(&rest[..nul]).map_err(|_| corrupt())?;
        rest = &rest[nul + 1..];

        let (id, after) = rest.split_at(20);
        let id: [u8; 20] = id.try_into().unwrap();
        rest = after;

        entries.push(TreeEntry {
            mode,
            name,
            id: ObjectId::from_bytes(id),
        });
    }
    Ok(entries)
}
```

`name`は`rest`の一部，つまり`data`の一部を借りている．`TreeEntry<'_>`の`'_`は，その借用が`data`から来ることを表す．

IDが足りないテストは，`split_at`がパニックになって失敗する．

```text
thread 'tree::tests::truncated_id_is_corrupt' (21868) panicked at src/tree.rs:105:32:
mid > len
```

`split_at`の前に長さを調べる．

```rust
if rest.len() < 20 {
    return Err(corrupt());
}
```

UTF-8でない名前の項目は，`from_utf8`と`map_err`があるので，そのまま通る．

### 表示

```rust
/// `<モード> <種類> <ID>\t<名前>`の形で表示する．
impl fmt::Display for TreeEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}\t{}",
            self.mode,
            self.mode.kind(),
            self.id,
            self.name
        )
    }
}
```

### `ls-tree`と`cat-file -p`

`cli.rs`に，エントリーを書く補助関数を加えた．

```rust
/// treeの内容を解析し，エントリーを1行ずつ書く．
fn write_tree_entries(content: &[u8], out: &mut impl Write) -> Result<(), Error> {
    for entry in parse_tree(content)? {
        writeln!(out, "{entry}")?;
    }
    Ok(())
}
```

`cat-file -p`は，種類がtreeのときだけこの関数を使う．`ls-tree`は，treeでなければ`NotATree`を返す．

```rust
Command::LsTree { tree } => {
    let repo = Repository::discover(cwd)?;
    let id = repo.resolve_prefix(&tree)?;
    let (kind, content) = repo.read_object(id)?;
    if kind != ObjectKind::Tree {
        return Err(Error::NotATree);
    }
    write_tree_entries(&content, out)?;
}
```

`content`は`Vec<u8>`で，`write_tree_entries`に貸している間だけエントリーが使われる．
RustOwlで`content`を見ると，生きている範囲が`write_tree_entries`の呼び出しまで続くことが分かる．

## 4-6 振り返り

1. 模範解答は，壊れたtreeの項目として，IDが足りない場合を選んだ．長さを調べ忘れると，エラーではなくパニックになる．
2. `name`を`String`にすると，`parse_tree`は名前ごとに`to_string()`を呼び，`TreeEntry`のライフタイム引数がなくなる．テストの期待値も`String::from("src")`になる．今の`rgit`はエントリーをすぐに表示して捨てるので，借用で足りる．
3. 違いは`Mode`の`TryFrom`(読むとき)と`Display`(表示するとき)に閉じ込めた．ほかのコードは`Mode::Directory`だけを扱い，2つの表記を知らなくてよい．
4. 手で組み立てると，壊れた形のように，本物の`git`では作れないバイト列を確かめられる．本物の`git`で作ったtreeは，実際の形式と一致していることを確かめられる．
5. 図に描いた型と関数は，コードと一致している．

## 4-7 発展課題

`LsTree`に`-r`のフラグを加え，サブディレクトリのtreeを読んで再帰的に書く．

```rust
/// treeの中のblobを，サブディレクトリもたどって`<ディレクトリ>/<名前>`の形で書く．
fn write_tree_recursive(
    repo: &Repository,
    content: &[u8],
    prefix: &str,
    out: &mut impl Write,
) -> Result<(), Error> {
    for entry in parse_tree(content)? {
        let path = format!("{prefix}{}", entry.name);
        if entry.mode == Mode::Directory {
            let (_, sub) = repo.read_object(entry.id)?;
            write_tree_recursive(repo, &sub, &format!("{path}/"), out)?;
        } else {
            writeln!(
                out,
                "{} {} {}\t{path}",
                entry.mode,
                entry.mode.kind(),
                entry.id
            )?;
        }
    }
    Ok(())
}
```

サブディレクトリの内容`sub`は，ループの1回分の中で読み，再帰呼び出しに貸す．`entries`の名前は親の`content`を借りているので，`sub`と混ざらない．
テストは，`git ls-tree -r`の出力と比べる．
