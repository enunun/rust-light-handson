use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rgit::diff::diff;

/// 0から`n - 1`までの数を1つずつ並べた行．
fn lines(n: usize) -> Vec<String> {
    (0..n).map(|i| i.to_string()).collect()
}

/// `a`の行のうち`changes`個を，等間隔に書き換える．1行の書き換えは，削除と挿入の2つの編集になる．
fn changed(a: &[String], changes: usize) -> Vec<String> {
    let step = a.len() / changes;
    let mut b = a.to_vec();
    for i in 0..changes {
        b[i * step] = format!("changed {i}");
    }
    b
}

/// 書き換える行の数を10に固定し，行の数を変える．
fn by_length(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff/length");
    for n in [1_000, 10_000, 100_000] {
        let a = lines(n);
        let b = changed(&a, 10);
        group.bench_with_input(BenchmarkId::from_parameter(n), &b, |bench, b| {
            bench.iter(|| diff(black_box(&a), black_box(b)))
        });
    }
    group.finish();
}

/// 行の数を20000に固定し，書き換える行の数を変える．
fn by_changes(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff/changes");
    let a = lines(20_000);
    for changes in [10, 100, 1_000] {
        let b = changed(&a, changes);
        group.bench_with_input(BenchmarkId::from_parameter(changes), &b, |bench, b| {
            bench.iter(|| diff(black_box(&a), black_box(b)))
        });
    }
    group.finish();
}

criterion_group!(benches, by_length, by_changes);
criterion_main!(benches);
