# Iteration 10：ジェネリックなアルゴリズムと文字列の組み立て

Iteration 10では，2つの列の差分を求める関数と，差分をunified形式の文字列にする関数を作る．
このノートでは，要素の型を問わないジェネリックな関数，操作を表す`enum`，`usize`と`isize`の使い分け，文字列の分け方と組み立て方，スライスをまとまりに分ける`chunk_by`，`Option::transpose`を説明する．

## 要素の型を問わない関数

Iteration 9の`S: ObjectStore`は，「オブジェクトストアならどの型でもよい」という境界だった．
差分のアルゴリズムが要素に求めるのは，2つの要素が等しいかを比べることだけである．
`==`で比べられることを表す標準のトレイトは`PartialEq`である．

```rust
pub fn diff<T: PartialEq>(a: &[T], b: &[T]) -> Vec<Edit> {
    // …
    while x < n && y < m && a[x as usize] == b[y as usize] {
        // …
    }
    // …
}
```

`T: PartialEq`と書いたので，`diff`は行(`&str`)の列にも，数の列にも，文字の列にも使える．

```rust
diff(&["a", "b", "c"], &["a", "c", "d"]);
diff(&[1, 2, 3], &[1, 3]);
diff(&['A', 'B'], &['B']);
```

境界を書かないと，関数の中で`==`を使ったところがコンパイルエラーになる．
コンパイラーは，`T`について，境界に書いたトレイトのメソッドと演算しか使わせない．
差分の単体テストは，ファイルを作らずに，数や文字の列で書ける．

`PartialEq`のほかに，よく使う境界には次のものがある．

| トレイト | できること |
| --- | --- |
| `PartialEq` | `==`と`!=`で比べる |
| `Ord` | 大小を比べる，並べ替える(`sort`) |
| `Hash` | `HashMap`のキーにする(`Eq`とともに使う) |
| `Clone` | `clone()`で複製する |
| `Display` | `{}`で表示する |

## 操作を表す`enum`

差分の結果は，「一致」「削除」「挿入」の操作の列である．
`enum`の各バリアントに，その操作に要る添字だけを持たせる．

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edit {
    /// `Equal(i, j)`は，`a[i]`と`b[j]`が等しいことを表す．
    Equal(usize, usize),
    /// `Delete(i)`は，`a[i]`を消すことを表す．
    Delete(usize),
    /// `Insert(j)`は，`b[j]`を挿入することを表す．
    Insert(usize),
}
```

`Edit`は要素そのものではなく添字を持つので，要素の型`T`に依存しない．`diff`の戻り値は`Vec<Edit>`で，要素を複製しない．
呼び出し側は，添字で元の列を引いて表示する．

```rust
let (marker, line) = match edit {
    Edit::Equal(i, _) => (' ', old_lines[i]),
    Edit::Delete(i) => ('-', old_lines[i]),
    Edit::Insert(j) => ('+', new_lines[j]),
};
```

### `matches!`

`matches!(値, パターン)`は，値がパターンに合えば`true`を返すマクロである．`match`で`true`と`false`を返す代わりに使う．

```rust
let is_change = !matches!(edit, Edit::Equal(..));
```

`Edit::Equal(..)`の`..`は，バリアントの中身をすべて無視する．

## `usize`と`isize`

添字と長さは`usize`(符号なし)である．Myersのアルゴリズムの対角線`k = x - y`は負にもなるので，`isize`(符号あり)で計算する．
`as`で型を変える．

```rust
let n = a.len() as isize;
let k = x - y;
a[x as usize]
```

`as`は，値が収まらないときも黙って変換する(負の`isize`を`usize`にすると大きな数になる)．`as`で変換する前に，値の範囲を確かめておく．

`usize`の引き算は，結果が負になるとき，デバッグビルドではパニックになる．負になりうる引き算には，0で止まる`saturating_sub`を使う．

```rust
let start = group[0].saturating_sub(context);
```

負の`k`を`Vec`の添字に使うことはできない．`k`に一定の数を足して，0以上の添字にずらす．

```rust
/// 対角線kの値を置く`v`の添字．
fn slot(k: isize, max: isize) -> usize {
    (k + max + 1) as usize
}
```

`(-d..=d).step_by(2)`は，`-d`から`d`までを2つおきに回す．`..=`は終わりを含む範囲である．

### `unreachable!`

ループを抜けずに終わることがないとわかっている場所には，`unreachable!`を書く．
もしそこに来たら，パニックになる．関数の戻り値の型を満たすためにも使える．

```rust
for d in 0..=max {
    // 終点に着いたらreturnする
}
unreachable!("d = n + mまでに必ず終点に着く")
```

## 文字列を行に分ける

`str::lines`は，行の終わりの改行を取り除く．最後の行に改行があったかどうかがわからなくなる．
`split_inclusive('\n')`は，改行を行に含めたまま分ける．

```rust
let text = "a\nb";
let lines: Vec<&str> = text.split_inclusive('\n').collect();
assert_eq!(lines, ["a\n", "b"]);
```

最後の要素が`'\n'`で終わらなければ，ファイルの末尾に改行がない．
`"abc"`と`"abc\n"`は違う行として比べられる．

ファイルの中身は`Vec<u8>`である．`String::from_utf8_lossy`は，UTF-8として読めないバイトを`�`に置き換えて文字列にする．
戻り値の`Cow<str>`は，置き換えが要らなければ元のバイト列を借り，要れば新しい`String`を持つ．`&`を付けると`&str`として使える．

## 文字列を組み立てる

`format!`は，毎回新しい`String`を作る．1つの`String`に少しずつ書き足すには，`write!`と`writeln!`を使う．

```rust
use std::fmt::Write;

let mut out = String::new();
writeln!(out, "diff --git a/{path} b/{path}").unwrap();
write!(out, "{marker}{line}").unwrap();
```

`write!`は，`std::fmt::Write`トレイトを実装した値に書く．`String`は`fmt::Write`を実装している．
`std::io::Write`(ファイルや標準出力)と名前が同じなので，同じファイルで両方を使うときは，`use std::fmt::Write as _;`のようにして名前をぶつけない．
`String`への書き込みは失敗しないので，結果の`Result`は`unwrap`してよい．

### タプル構造体の`Display`

ハンクの見出しの範囲は，行数によって書き方が変わる．タプル構造体`Range`に`Display`を実装し，`match`で場合を分ける．

```rust
struct Range(usize, usize);

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Range(start, 0) => write!(f, "{start},0"),
            Range(start, 1) => write!(f, "{}", start + 1),
            Range(start, len) => write!(f, "{},{len}", start + 1),
        }
    }
}
```

パターンの`0`や`1`は，その値に合うときだけ選ばれる．上から順に試すので，最後の`Range(start, len)`は残りすべてに合う．

## まとまりに分ける`chunk_by`

`slice::chunk_by(関数)`は，隣り合う2つの要素に関数を呼び，`true`のあいだを1つのまとまりにする．
戻り値は，まとまりのスライス(`&[T]`)を順に返すイテレーターである．

```rust
let numbers = [1, 2, 3, 7, 8, 10];
let groups: Vec<&[i32]> = numbers.chunk_by(|a, b| b - a == 1).collect();
assert_eq!(groups, [&[1, 2, 3][..], &[7, 8], &[10]]);
```

ハンクを作るときは，変更のある編集の位置を，間が近いものどうしでまとめる．

```rust
changes.chunk_by(|prev, next| next - prev - 1 <= 2 * context)
```

## `Option::transpose`

`transpose`は，`Option<Result<T, E>>`と`Result<Option<T>, E>`の内側と外側を入れ替える．`?`と組み合わせると，「あれば読み，読めなければエラーを返す」を1行で書ける．

```rust
let old_version: Option<FileVersion> = old
    .get(path)
    .map(|&(mode, id)| read_version(store, mode, id, None))
    .transpose()?;
```

| `map`の結果 | `transpose`の結果 | `?`のあと |
| --- | --- | --- |
| `None` | `Ok(None)` | `None` |
| `Some(Ok(v))` | `Ok(Some(v))` | `Some(v)` |
| `Some(Err(e))` | `Err(e)` | 関数から`e`を返す |
