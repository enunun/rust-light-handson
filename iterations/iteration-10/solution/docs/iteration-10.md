# Iteration 10：`diff`(解説)

演習の各手順の模範解答と，その考え方を説明する．

## 10-1 準備

`Cargo.toml`は，Iteration 9の模範解答からパッケージ名だけを変えたものである．依存は変わらない．

## 10-2 文法と概念

課題の解答例である．

```rust
#[cfg(test)]
mod practice {
    use std::fmt::Write;

    fn count_equal<T: PartialEq>(a: &[T], b: &[T]) -> usize {
        a.iter().zip(b).filter(|(x, y)| x == y).count()
    }

    #[test]
    fn generic_function_works_for_any_comparable_type() {
        assert_eq!(count_equal(&[1, 2, 3], &[1, 5, 3]), 2);
        assert_eq!(count_equal(&["a", "b"], &["a", "c"]), 1);
    }

    #[derive(Debug, PartialEq)]
    enum Step {
        Keep(char),
        Skip,
    }

    #[test]
    fn matches_checks_a_pattern() {
        let steps = [Step::Keep('a'), Step::Skip, Step::Keep('b')];
        let kept = steps
            .iter()
            .filter(|step| matches!(step, Step::Keep(_)))
            .count();
        assert_eq!(kept, 2);
    }

    #[test]
    fn signed_and_unsigned_arithmetic() {
        let (x, y): (usize, usize) = (2, 5);
        let k = x as isize - y as isize;
        assert_eq!(k, -3);
        assert_eq!(x.saturating_sub(y), 0);
        assert_eq!(x.checked_sub(y), None);
    }

    #[test]
    fn split_inclusive_keeps_newlines() {
        let with_newline: Vec<&str> = "a\nb\n".split_inclusive('\n').collect();
        let without_newline: Vec<&str> = "a\nb".split_inclusive('\n').collect();
        assert_eq!(with_newline, ["a\n", "b\n"]);
        assert_eq!(without_newline, ["a\n", "b"]);
        assert_eq!(
            "a\nb\n".lines().collect::<Vec<_>>(),
            "a\nb".lines().collect::<Vec<_>>()
        );
    }

    #[test]
    fn write_builds_a_string() {
        let mut out = String::new();
        for n in 1..=3 {
            write!(out, "{n}").unwrap();
        }
        writeln!(out, "!").unwrap();
        assert_eq!(out, "123!\n");
    }

    #[test]
    fn chunk_by_groups_neighbors() {
        let numbers = [1, 2, 3, 7, 8, 10];
        let groups: Vec<&[i32]> = numbers.chunk_by(|a, b| b - a == 1).collect();
        assert_eq!(groups, [&[1, 2, 3][..], &[7, 8], &[10]]);
    }

    fn parse_optional(text: Option<&str>) -> Result<Option<i32>, std::num::ParseIntError> {
        text.map(|text| text.parse::<i32>()).transpose()
    }

    #[test]
    fn transpose_swaps_option_and_result() {
        assert_eq!(parse_optional(None), Ok(None));
        assert_eq!(parse_optional(Some("42")), Ok(Some(42)));
        assert!(parse_optional(Some("x")).is_err());
    }
}
```

- 1：`zip`は，2つのイテレーターの要素を組にして返す．短いほうが終わると止まる．`: PartialEq`を消すと，`==`のところでコンパイルエラーになる．コンパイラーは，境界を加える場所まで示す．

```console
$ cargo test
error[E0369]: binary operation `==` cannot be applied to type `&&T`
  --> src/lib.rs:26:43
   |
26 |         a.iter().zip(b).filter(|(x, y)| x == y).count()
   |                                         - ^^ - &&T
   |                                         |
   |                                         &&T
   |
help: consider restricting type parameter `T` with trait `PartialEq`
   |
25 |     fn count_equal<T: std::cmp::PartialEq>(a: &[T], b: &[T]) -> usize {
   |                     +++++++++++++++++++++

For more information about this error, try `rustc --explain E0369`.
error: could not compile `rgit` (lib test) due to 1 previous error
```

- 3：`checked_sub`は，結果が負になるとき`None`を返す．`saturating_sub`は0で止まる．
- 4：`lines()`では，2つの文字列が同じ行の列になる．末尾の改行の有無は`split_inclusive`でしか区別できない．
- 6：比べる型をそろえるため，最初の要素だけ`[..]`でスライスにしている．残りの要素は，それに合わせて変換される．
- 7：`parse`の結果は`Result`なので，`map`のあとは`Option<Result<i32, _>>`になる．`transpose`で`Result<Option<i32>, _>`にする．

## 10-3 テストリスト

模範解答のテストリストは[TESTLIST.md](../TESTLIST.md)にある．

- `diff`は，空の列から始め，ノートの例(`[a, b, c]`と`[a, c, d]`)を経て，Myersの論文の例(`ABCABBA`と`CBABAC`)に進む．論文の例は最短の編集が複数あるので，編集の列そのものではなく，変更の数と，当てはめた結果を確かめる．
- `hunks`は，間の一致が6つ(まとまる)と7つ(分かれる)の境目を両方試す．
- `patch`の項目は，`git`で試した出力の規則を1つずつテストにした．
- 結合テストは，いろいろな変更を1つの作業ディレクトリにまとめて加え，`git diff`と比べる．同じ長さの編集が複数ある変更は含めていない．

## 10-4 図

模範解答は[design/types.md](../design/types.md)にある．Iteration 9からの変更は次のとおりである．

- `diff`の名前空間に，`diff_mod`，`Edit`，`Hunk`，見出しの範囲を書く非公開の`Range`を描いた．
- `patch`の名前空間に，`patch_mod`と`FileVersion`を描き，`diff_mod`，`status_mod`，`ObjectStore`への依存を描いた．
- `status_mod`に，表を作る3つの関数と`compare`を加えた．
- `Command`に`Diff`を加え，`cli_mod`から`patch_mod`への依存を描いた．
- `diff`の型引数`T`の境界と，戻り値の表の型は，図の下に書いた．

## 10-5 テスト駆動の実装

### `diff`

最初のテストは，同じ列と空の列である．同じ列なら，`d = 0`のとき斜めに進むだけで終点に着く．

```rust
#[test]
fn same_sequences_are_all_equal() {
    assert_eq!(diff(&[1, 2], &[1, 2]), [Equal(0, 0), Equal(1, 1)]);
}
```

削除と挿入の混ざるテストを加えてから，Myersのアルゴリズムを書いた．

```rust
pub fn diff<T: PartialEq>(a: &[T], b: &[T]) -> Vec<Edit> {
    let n = a.len() as isize;
    let m = b.len() as isize;
    let max = n + m;
    // `v[slot(k, max)]`は，対角線kの上で，これまでに最も遠くまで進んだxである．
    let mut v = vec![0; 2 * max as usize + 3];
    let mut trace = Vec::new();
    for d in 0..=max {
        trace.push(v.clone());
        for k in (-d..=d).step_by(2) {
            let mut x = if goes_down(&v, k, d, max) {
                v[slot(k + 1, max)]
            } else {
                v[slot(k - 1, max)] + 1
            };
            let mut y = x - k;
            while x < n && y < m && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }
            v[slot(k, max)] = x;
            if x >= n && y >= m {
                return backtrack(&trace, n, m);
            }
        }
    }
    unreachable!("d = n + mまでに必ず終点に着く")
}

/// 段階dで対角線kに来るとき，対角線k + 1から下へ進む(挿入する)か．
/// 偽なら，対角線k - 1から右へ進む(削除する)．
fn goes_down(v: &[isize], k: isize, d: isize, max: isize) -> bool {
    k == -d || (k != d && v[slot(k - 1, max)] < v[slot(k + 1, max)])
}
```

- `k == -d`なら，左の対角線はないので，上の対角線(`k + 1`)から下へ進むしかない．`k == d`なら，右から来るしかない．どちらでもなければ，遠くまで進んだほうから来る．
- `v`の長さは，`k`が`-max - 1`から`max + 1`までの添字を持てる`2 * max + 3`にした．`d = 0`でも`v[slot(1, max)]`を読むので，両端に1つずつ余裕が要る．
- 進む向きを決める規則は，`diff`と`backtrack`の両方で使うので，関数`goes_down`にまとめた．

`backtrack`は，記録した`v`を新しい段階から順に見て，どの対角線から来たかを`goes_down`で求める．
来た点まで斜めに戻る間は`Equal`を，最後の1歩は`Insert`か`Delete`を集め，最後に`reverse`する．

```rust
fn backtrack(trace: &[Vec<isize>], n: isize, m: isize) -> Vec<Edit> {
    let max = n + m;
    let mut edits = Vec::new();
    let (mut x, mut y) = (n, m);
    for (d, v) in trace.iter().enumerate().rev() {
        let d = d as isize;
        let k = x - y;
        let prev_k = if goes_down(v, k, d, max) {
            k + 1
        } else {
            k - 1
        };
        let prev_x = v[slot(prev_k, max)];
        let prev_y = prev_x - prev_k;
        while x > prev_x && y > prev_y {
            x -= 1;
            y -= 1;
            edits.push(Edit::Equal(x as usize, y as usize));
        }
        if d > 0 {
            if x == prev_x {
                edits.push(Edit::Insert(prev_y as usize));
            } else {
                edits.push(Edit::Delete(prev_x as usize));
            }
        }
        x = prev_x;
        y = prev_y;
    }
    edits.reverse();
    edits
}
```

`ABCABBA`と`CBABAC`のテストでは，編集を当てはめる補助の関数`apply`を作った．`Equal(i, j)`では`a[i]`と`b[j]`が等しいことも確かめる．

### `hunks`

変更のある編集の位置を集め，`chunk_by`で近いものをまとめる．

```rust
pub fn hunks(edits: &[Edit], context: usize) -> Vec<Hunk> {
    let changes: Vec<usize> = edits
        .iter()
        .enumerate()
        .filter(|(_, edit)| !matches!(edit, Edit::Equal(..)))
        .map(|(i, _)| i)
        .collect();
    changes
        .chunk_by(|prev, next| next - prev - 1 <= 2 * context)
        .map(|group| {
            let start = group[0].saturating_sub(context);
            let end = (group[group.len() - 1] + context + 1).min(edits.len());
            hunk(edits, start, end)
        })
        .collect()
}
```

`next - prev - 1`は，2つの変更の間にある一致の数である．`changes`は増える順なので，この引き算は負にならない．
ハンクの開始の行は，ハンクより前の編集のうち，古い側と新しい側を使うものの数である．

```rust
fn hunk(edits: &[Edit], start: usize, end: usize) -> Hunk {
    let uses_old = |edit: &&Edit| !matches!(edit, Edit::Insert(_));
    let uses_new = |edit: &&Edit| !matches!(edit, Edit::Delete(_));
    let (before, inside) = (&edits[..start], &edits[start..end]);
    Hunk {
        old_start: before.iter().filter(uses_old).count(),
        old_len: inside.iter().filter(uses_old).count(),
        new_start: before.iter().filter(uses_new).count(),
        new_len: inside.iter().filter(uses_new).count(),
        edits: inside.to_vec(),
    }
}
```

`filter`のクロージャーは要素の参照を受け取る．要素が`&Edit`なので，引数は`&&Edit`になる．
`Hunk`の`Display`は，ノートの`Range`で見出しを書く．`old_start`は0から数える添字のまま持ち，表示するときに1を足す．

### `file_patch`

最初のテストは，ノートの`hello.txt`の例である．期待する文字列は，`git diff`の出力をそのまま書いた．

```rust
#[test]
fn modified_file_has_index_line_and_hunk() {
    let patch = file_patch(
        "hello.txt",
        Some(&version("hello\n")),
        Some(&version("hello\nworld\n")),
    );
    assert_eq!(
        patch,
        "diff --git a/hello.txt b/hello.txt\n\
         index ce01362..94954ab 100644\n\
         --- a/hello.txt\n\
         +++ b/hello.txt\n\
         @@ -1 +1,2 @@\n \
         hello\n\
         +world\n"
    );
}
```

文字列リテラルの行末の`\`は，改行と次の行の先頭の空白を取り除く．期待する出力を行ごとに並べて書ける．
文脈の行の先頭の空白は取り除かれてしまうので，`@@ -1 +1,2 @@\n \`のように，`\`の前に置いた．

追加，削除，モード，バイナリーのテストを1つずつ加え，ヘッダーの部分を`match`で分けた．

```rust
match (old, new) {
    (None, None) => return String::new(),
    (None, Some(new)) => {
        writeln!(out, "new file mode {}", new.mode).unwrap();
        writeln!(out, "index 0000000..{}", new.id.short()).unwrap();
    }
    (Some(old), None) => {
        writeln!(out, "deleted file mode {}", old.mode).unwrap();
        writeln!(out, "index {}..0000000", old.id.short()).unwrap();
    }
    (Some(old), Some(new)) if old.mode != new.mode => {
        writeln!(out, "old mode {}", old.mode).unwrap();
        writeln!(out, "new mode {}", new.mode).unwrap();
        if old.id == new.id {
            return out;
        }
        writeln!(out, "index {}..{}", old.id.short(), new.id.short()).unwrap();
    }
    (Some(old), Some(new)) => {
        let (old_id, new_id) = (old.id.short(), new.id.short());
        writeln!(out, "index {old_id}..{new_id} {}", old.mode).unwrap();
    }
}
```

`if old.mode != new.mode`は，パターンに条件を加える(Iteration 9の`compare`と同じ)．モードの違いを先に扱い，最後の腕はモードが同じ場合になる．

ない側は，中身を空のバイト列として扱う．`&[][..]`は，空の配列をスライスにしたものである．

```rust
let old_content = old.map_or(&[][..], |old| &old.content);
```

### `diff_work_tree`と`diff_cached`

`status`の中で表を作っていた部分を，`head_files`，`index_files`，`work_tree_files`の3つの関数に分けた．`status`のテストはそのまま通る．
`patch`は，2つの表を比べる`diff_tables`を，`git diff`と`git diff --cached`の両方で使う．

```rust
let old_version = old
    .get(path)
    .map(|&(mode, id)| read_version(store, mode, id, None))
    .transpose()?;
```

`|&(mode, id)|`は，表の値の参照`&(Mode, ObjectId)`を，パターンで分解して受け取る．`Mode`と`ObjectId`は`Copy`なので，値として取り出せる．
作業ディレクトリの側は，表のIDではなくファイルを読む．IDは`work_tree_files`がハッシュを計算したものなので，`index`の行に使える．

`git diff`は，インデックスにないファイルを表示しない．`diff_work_tree`は，作業ディレクトリの表から，インデックスにないファイルを除いてから比べる．

### `cli`

```rust
Command::Diff { cached } => {
    let repo = Repository::discover(cwd)?;
    let index = Index::load(&repo.index_path())?;
    let patch = if cached {
        let head = repo.resolve_ref(&RefName::head())?;
        diff_cached(repo.objects(), head, &index)?
    } else {
        diff_work_tree(repo.objects(), &index, repo.work_dir())?
    };
    out.write_all(patch.as_bytes())?;
}
```

`cli.rs`は`std::io::Write`を使うので，`patch.rs`だけが`std::fmt::Write`を使う．モジュールを分けたので，2つの`Write`がぶつからない．

## 10-6 振り返り

1. 模範解答は，`git`で試した出力の規則を，1つの規則に1つのテストで書いている．どの規則が壊れたかが，テストの名前でわかる．
2. `&[&str]`だけを受け取る関数にすると，テストにも行の配列が要る．ジェネリックにしたので，数や文字の短い列でテストを書け，単語の差分などにも使える．
3. 添字を持つ`Edit`は`Copy`で，元の列の寿命に縛られない．参照を持つ`Edit<'a, T>`なら，表示するときに元の列を引かずに済むが，`Edit`を使う型すべてにライフタイムと型引数が付く．
4. `String`を返すと，テストは`assert_eq!`で比べるだけでよい．`impl io::Write`に書くと大きな差分でもメモリーを使わずに済むが，テストでは`Vec<u8>`に書いて文字列に戻す手間が増える．
5. 図に描いた型と関係は，コードと一致している．

## 10-7 発展課題

`Command::Diff`に引数を加え，文脈の行数を`diff_work_tree`，`diff_cached`，`diff_tables`，`file_patch`に渡す．
定数`CONTEXT`は要らなくなる．

```rust
Diff {
    /// Show changes between HEAD and the index instead
    #[arg(long)]
    cached: bool,
    /// Number of context lines
    #[arg(short = 'U', long = "unified", default_value_t = 3)]
    unified: usize,
},
```

`default_value_t`は，引数を省いたときの値である．`-U1`のように，短い名前の直後に値を続けて書ける．

```rust
#[test]
fn unified_option_changes_context_like_git() {
    // 英字で始まる行があると，gitは見出しに関数名を付ける．数の行だけのファイルで比べる．
    let dir = TempDir::new().unwrap();
    rgit(dir.path(), &["init"]);
    fs::write(dir.path().join("numbers.txt"), numbers()).unwrap();
    rgit(dir.path(), &["add", "."]);
    let changed = numbers().replacen("5\n", "50\n", 1).replacen("12\n", "", 1);
    fs::write(dir.path().join("numbers.txt"), changed + "21\n").unwrap();
    for unified in ["-U0", "-U1", "-U3", "-U5"] {
        assert_eq!(
            rgit(dir.path(), &["diff", unified]),
            git(dir.path(), &["diff", unified])
        );
    }
}
```

このテストの変更を`-U0`で表示すると，次のようになる．

```console
$ rgit diff -U0
diff --git a/numbers.txt b/numbers.txt
index 0ff3bbb..9905cb3 100644
--- a/numbers.txt
+++ b/numbers.txt
@@ -5 +5 @@
-5
+50
@@ -12 +11,0 @@
-12
@@ -20,0 +20 @@
+21
```

隣り合わない変更は，すべて別のハンクになる．削除だけのハンクは新しい側の，挿入だけのハンクは古い側の行数が0になり，開始には直前の行の番号を書く．
