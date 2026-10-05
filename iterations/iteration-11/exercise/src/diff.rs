use std::fmt;

/// 2つの列`a`と`b`の間の編集の1つ．数は列の添字である．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edit {
    /// `Equal(i, j)`は，`a[i]`と`b[j]`が等しいことを表す．
    Equal(usize, usize),
    /// `Delete(i)`は，`a[i]`を消すことを表す．
    Delete(usize),
    /// `Insert(j)`は，`b[j]`を挿入することを表す．
    Insert(usize),
}

/// `a`を`b`にする最短の編集を，Myersのアルゴリズムで求める．
/// 編集は`a`と`b`の先頭から順に並ぶ．
pub fn diff<T: PartialEq>(a: &[T], b: &[T]) -> Vec<Edit> {
    let n = a.len() as isize;
    let m = b.len() as isize;
    let max = n + m;
    // `v[slot(k, max)]`は，対角線kの上で，これまでに最も遠くまで進んだxである．
    let mut v = vec![0; 2 * max as usize + 3];
    let mut trace = Vec::new();
    for d in 0..=max {
        // 段階dで読むのは，対角線-d - 1からd + 1までの値だけである．その範囲だけを記録する．
        trace.push(v[slot(-d - 1, max)..=slot(d + 1, max)].to_vec());
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

/// 対角線`-max - 1`から`max + 1`までの値を並べた配列で，対角線kの値を置く添字．
fn slot(k: isize, max: isize) -> usize {
    (k + max + 1) as usize
}

/// 段階dで対角線kに来るとき，対角線k + 1から下へ進む(挿入する)か．
/// 偽なら，対角線k - 1から右へ進む(削除する)．
fn goes_down(v: &[isize], k: isize, d: isize, max: isize) -> bool {
    k == -d || (k != d && v[slot(k - 1, max)] < v[slot(k + 1, max)])
}

/// `diff`が記録した各段階の`v`を，終点から始点へたどって編集を集める．
/// `trace[d]`は対角線`-d - 1`から`d + 1`までの値を持つので，添字は`slot(k, d)`で求める．
fn backtrack(trace: &[Vec<isize>], n: isize, m: isize) -> Vec<Edit> {
    let mut edits = Vec::new();
    let (mut x, mut y) = (n, m);
    for (d, v) in trace.iter().enumerate().rev() {
        let d = d as isize;
        let k = x - y;
        let prev_k = if goes_down(v, k, d, d) { k + 1 } else { k - 1 };
        let prev_x = v[slot(prev_k, d)];
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

/// 近くの変更を，前後の文脈とともにまとめたもの．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    /// 古い列で，ハンクの始まる添字．
    pub old_start: usize,
    /// ハンクに含まれる古い列の要素の数．
    pub old_len: usize,
    /// 新しい列で，ハンクの始まる添字．
    pub new_start: usize,
    /// ハンクに含まれる新しい列の要素の数．
    pub new_len: usize,
    pub edits: Vec<Edit>,
}

/// unified形式の見出し`@@ -<開始>,<行数> +<開始>,<行数> @@`を書く．
impl fmt::Display for Hunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "@@ -{} +{} @@",
            Range(self.old_start, self.old_len),
            Range(self.new_start, self.new_len)
        )
    }
}

/// 見出しの中の行の範囲．行番号は1から数える．
struct Range(usize, usize);

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            // 空の範囲は，その直前の行の番号で表す．
            Range(start, 0) => write!(f, "{start},0"),
            Range(start, 1) => write!(f, "{}", start + 1),
            Range(start, len) => write!(f, "{},{len}", start + 1),
        }
    }
}

/// 編集の列から，変更の前後`context`個の一致を含むハンクを作る．
/// 間の一致が`2 * context`個以下の変更は，1つのハンクにまとめる．
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

/// `edits[start..end]`を1つのハンクにする．
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

#[cfg(test)]
mod tests {
    use super::*;
    use Edit::{Delete, Equal, Insert};

    /// 編集を`a`に当てはめて，`b`ができることを確かめる．
    fn apply(a: &[char], b: &[char], edits: &[Edit]) -> Vec<char> {
        edits
            .iter()
            .filter_map(|edit| match *edit {
                Equal(i, j) => {
                    assert_eq!(a[i], b[j]);
                    Some(a[i])
                }
                Delete(_) => None,
                Insert(j) => Some(b[j]),
            })
            .collect()
    }

    fn chars(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    #[test]
    fn same_sequences_are_all_equal() {
        assert_eq!(diff(&[1, 2], &[1, 2]), [Equal(0, 0), Equal(1, 1)]);
    }

    #[test]
    fn empty_sequences_have_no_edits() {
        assert_eq!(diff::<u8>(&[], &[]), []);
    }

    #[test]
    fn from_empty_is_all_inserts_and_to_empty_is_all_deletes() {
        assert_eq!(diff(&[], &["a", "b"]), [Insert(0), Insert(1)]);
        assert_eq!(diff(&["a", "b"], &[]), [Delete(0), Delete(1)]);
    }

    #[test]
    fn delete_comes_before_insert() {
        assert_eq!(
            diff(&["a", "b", "c"], &["a", "c", "d"]),
            [Equal(0, 0), Delete(1), Equal(2, 1), Insert(2)]
        );
        assert_eq!(diff(&["a"], &["b"]), [Delete(0), Insert(0)]);
    }

    #[test]
    fn edits_are_shortest_and_rebuild_the_new_sequence() {
        let (a, b) = (chars("ABCABBA"), chars("CBABAC"));
        let edits = diff(&a, &b);
        let changes = edits.iter().filter(|edit| !matches!(edit, Equal(..)));
        assert_eq!(changes.count(), 5);
        assert_eq!(apply(&a, &b, &edits), b);
    }

    #[test]
    fn one_change_makes_one_hunk_with_context() {
        let a: Vec<u32> = (1..=10).collect();
        let mut b = a.clone();
        b[4] = 50;
        let hunks = hunks(&diff(&a, &b), 3);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].to_string(), "@@ -2,7 +2,7 @@");
        assert_eq!(hunks[0].edits.len(), 8);
    }

    #[test]
    fn hunks_merge_when_gap_is_at_most_twice_the_context() {
        let a: Vec<u32> = (1..=20).collect();
        let mut near = a.clone();
        near[2] = 30;
        near[9] = 100;
        assert_eq!(hunks(&diff(&a, &near), 3).len(), 1);
        let mut far = a.clone();
        far[2] = 30;
        far[10] = 110;
        let headers: Vec<String> = hunks(&diff(&a, &far), 3)
            .iter()
            .map(|hunk| hunk.to_string())
            .collect();
        assert_eq!(headers, ["@@ -1,6 +1,6 @@", "@@ -8,7 +8,7 @@"]);
    }

    #[test]
    fn header_omits_length_one_and_uses_previous_line_for_empty_range() {
        assert_eq!(hunks(&diff(&[], &["a"]), 3)[0].to_string(), "@@ -0,0 +1 @@");
        assert_eq!(
            hunks(&diff(&["a", "b"], &["a"]), 3)[0].to_string(),
            "@@ -1,2 +1 @@"
        );
    }

    #[test]
    fn no_changes_make_no_hunks() {
        assert_eq!(hunks(&diff(&[1, 2], &[1, 2]), 3), []);
    }
}
