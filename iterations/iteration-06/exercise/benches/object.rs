use std::hint::black_box;
use std::io::Write;

use criterion::{Criterion, criterion_group, criterion_main};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use rgit::hash_blob;

/// 1MiBのテキスト(数を1行ずつ並べたもの)．
fn sample() -> Vec<u8> {
    (0..)
        .flat_map(|i| format!("{i}\n").into_bytes())
        .take(1 << 20)
        .collect()
}

/// zlibで圧縮する．`Repository::write_blob`は`Compression::default()`を使う．
fn compress(data: &[u8], level: Compression) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), level);
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

fn object(c: &mut Criterion) {
    let data = sample();
    c.bench_function("hash_blob", |b| b.iter(|| hash_blob(black_box(&data))));
    c.bench_function("compress_default", |b| {
        b.iter(|| compress(black_box(&data), Compression::default()))
    });
    c.bench_function("compress_fast", |b| {
        b.iter(|| compress(black_box(&data), Compression::fast()))
    });
}

criterion_group!(benches, object);
criterion_main!(benches);
