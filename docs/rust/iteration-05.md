# Iteration 5：バイナリーの形式，`BTreeMap`，再帰とクロージャ

Iteration 5では，インデックス`.git/index`を読み書きし，作業ディレクトリのファイルを登録する`add`を作る．
このノートでは，整数とバイト列の変換，`as`による整数の変換，バイト列を読み進める型，`BTreeMap`，Unixのファイルの情報，ディレクトリをたどる再帰，クロージャとイテレーターのメソッドを説明する．

## 整数とバイト列の変換

ファイルの形式では，整数を決まったバイト数とバイト順で書く．
`u32`は4バイトで，大きい桁のバイトから書く順をビッグエンディアン，小さい桁から書く順をリトルエンディアンと呼ぶ．

```rust
assert_eq!(0x1234_5678u32.to_be_bytes(), [0x12, 0x34, 0x56, 0x78]);
assert_eq!(0x1234_5678u32.to_le_bytes(), [0x78, 0x56, 0x34, 0x12]);
assert_eq!(u32::from_be_bytes([0, 0, 1, 0]), 256);
```

- `to_be_bytes()`は`[u8; 4]`を返す．`from_be_bytes`は`[u8; 4]`を受け取る．スライスから配列への変換には，Iteration 4の`try_into()`を使う．
- 数のリテラルの`_`は，読みやすくするための区切りで，値には影響しない．
- `0o100644`は8進数，`0x0fff`は16進数のリテラルである．

## `as`による整数の変換

`as`は，整数の型を変換する．大きい型から小さい型へ変換すると，入りきらない上位のビットが捨てられる．

```rust
let big: u64 = 0x1_0000_0002;
assert_eq!(big as u32, 2);
assert_eq!(300u32 as u8, 44);
```

インデックスの時刻や大きさの欄は32ビットなので，`u64`の値を`as u32`で切り詰めて書く．
値が範囲に収まることを確かめたいときは，`u32::try_from(big)`を使う．収まらなければ`Err`になる．
小さい型から大きい型への変換は値が変わらないので，`usize::from(flags)`のように`From`で書ける．

## バイト列の組み立て

| 書き方 | 動作 |
| --- | --- |
| `bytes.extend_from_slice(&x)` | スライスの中身を後ろに足す |
| `bytes.push(b)` | 1バイトを後ろに足す |
| `bytes.resize(n, 0)` | 長さを`n`にする．伸ばした部分は0で埋める |

`for value in [a, b, c] { … }`のように，配列をそのまま`for`で回せる．
同じ形の値をいくつも書くときに使える．

## バイト列を読み進める型

読む位置を進めながらバイト列を読むには，残りのスライスを持つ小さな型を作ると書きやすい．

```rust
struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        if self.rest.len() < n {
            return Err(Error::CorruptIndex("unexpected end"));
        }
        let (taken, rest) = self.rest.split_at(n);
        self.rest = rest;
        Ok(taken)
    }

    fn u32(&mut self) -> Result<u32, Error> {
        let bytes: [u8; 4] = self.take(4)?.try_into().unwrap();
        Ok(u32::from_be_bytes(bytes))
    }
}
```

`take`の戻り値の`&'a [u8]`は，`Reader`自身(`&mut self`)ではなく，元のバイト列から借りている．
`'a`を書かずに省略すると，戻り値は`&mut self`から借りたことになり，`take`の結果を持ったまま次の`take`を呼べなくなる．
ここでは省略の規則ではなく，`impl<'a>`で名前を付けたライフタイムを使う．

## `BTreeMap`

`BTreeMap<K, V>`は，キーと値の組を，キーの順に並べて持つ．
`for (key, value) in &map`や`map.keys()`は，キーの小さい順に返す．`String`のキーは，バイトの順に並ぶ．

```rust
use std::collections::BTreeMap;

let mut map = BTreeMap::new();
map.insert("src/main.rs", 2);
map.insert("hello.txt", 1);
let keys: Vec<&&str> = map.keys().collect();
assert_eq!(keys, [&"hello.txt", &"src/main.rs"]);
map.insert("hello.txt", 10); // 同じキーなら値を置き換える
assert_eq!(map["hello.txt"], 10);
```

| メソッド | 動作 |
| --- | --- |
| `insert(k, v)` | 組を加える．同じキーがあれば値を置き換える |
| `remove(&k)` | 組を除く |
| `get(&k)` | 値を`Option<&V>`で返す |
| `map[&k]` | 値を返す．キーがなければパニックになる |
| `len()`，`is_empty()` | 組の数，空かどうか |

`HashMap`も同じように使えるが，順序は決まらない．
インデックスはパスの順に並べて書く決まりなので，`BTreeMap`を使えば並べ替えが要らない．

## 構造体の既定値

`#[derive(Default)]`を付けると，すべてのフィールドを既定値(整数は0，`BTreeMap`は空)にした値を`Type::default()`で作れる．
構造体の更新構文`..`を使うと，一部のフィールドだけを指定し，残りをほかの値から写せる．

```rust
let stat = Stat {
    mtime: 1767225600,
    size: 6,
    ..Stat::default()
};
```

## Unixのファイルの情報

`fs::metadata(path)`は，ファイルの大きさ，時刻，権限などの情報(`Metadata`)を返す．
Unixに固有の情報は，トレイト`std::os::unix::fs::MetadataExt`を`use`すると読める．

| メソッド | 値 |
| --- | --- |
| `meta.mtime()`，`meta.mtime_nsec()` | 変更の時刻(秒と，その中のナノ秒) |
| `meta.ctime()`，`meta.ctime_nsec()` | 状態の変更の時刻 |
| `meta.dev()`，`meta.ino()` | デバイスとiノードの番号 |
| `meta.uid()`，`meta.gid()` | 所有者 |
| `meta.size()` | 大きさ |

権限は`meta.permissions().mode()`で読める(`std::os::unix::fs::PermissionsExt`)．
`mode & 0o111 != 0`なら，誰かが実行できるファイルである．
権限を変えるには，`fs::set_permissions(path, fs::Permissions::from_mode(0o755))`を使う．

## 再帰とディレクトリ

ディレクトリの下のファイルをすべて集めるには，ディレクトリなら自分自身を呼び直す再帰の関数を書く．

```rust
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
```

`fs::read_dir`の要素は`io::Result<DirEntry>`である．
`collect::<Result<Vec<_>, _>>()`は，すべてが`Ok`なら`Ok(Vec)`を，1つでも`Err`があれば最初の`Err`を返す．
`Result`の列を，`Vec`の`Result`に変えられる．`_`は，コンパイラーに推論させる型である．

`DirEntry`の`path()`はエントリーのパスを，`file_name()`は名前を返す．
`read_dir`の返す順序は決まっていない．順序が要るときは並べ替えるか，`BTreeMap`に入れる．

## クロージャとイテレーター

Iteration 3から使ってきたクロージャ`|引数| 式`は，周りの変数を使える無名の関数である．
イテレーターのメソッドに渡して，要素の選び方や変え方を書く．

```rust
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
```

| メソッド | 動作 |
| --- | --- |
| `iter()` | 要素への参照を順に返す |
| `filter(\|x\| 条件)` | 条件を満たす要素だけを残す |
| `map(\|x\| 式)` | 要素を式の値に変える |
| `cloned()` | 参照の要素を複製して，値の要素にする |
| `collect()` | 要素を集めて`Vec`などにする |

`filter`のクロージャは`prefix`を使っている．クロージャは，作られた場所の変数を借りて使える．
イテレーターのメソッドは，`collect`などで要素を取り出すまで何もしない．`map`や`filter`をつないでも，途中の`Vec`は作られない．

`ok_or_else(|| …)`や`map_err(|_| …)`もクロージャを受け取る．
`if let`の条件は，`&&`で`else if`にもつなげられる(`else if path.is_file() && let Some(file) = …`)．
