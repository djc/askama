use std::env::temp_dir;
use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::time::{Duration, Instant};

use arbitrary::{Arbitrary, Unstructured};
use askama_parser::{Ast, ParseError, Syntax};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use random_code::Node;

fn by_min_size(c: &mut Criterion) {
    let syntax = &Syntax::default();
    let mut source = String::new();
    for count in [100, 1000, 10_000, 100_000] {
        c.bench_function(&format!("{count} bytes"), |b| {
            b.iter_custom(|i| {
                let mut total = Duration::default();
                for i in 0..i {
                    fill_at_least(&mut source, i, count);
                    let source = source.as_str();

                    let start = Instant::now();
                    if let Err(err) = Ast::from_str(black_box(source), syntax) {
                        failure(err, i, source);
                    }
                    total += start.elapsed();
                }
                total
            });
        });
    }
}

fn fill_at_least(source: &mut String, i: u64, count: usize) {
    source.clear();

    let i = i.to_ne_bytes();
    // front part of SHA-256's IV
    let mut rng = Xoshiro256PlusPlus::from_seed([
        0x42, 0x8a, 0x2f, 0x98, 0x71, 0x37, 0x44, 0x91, 0xb5, 0xc0, 0xfb, 0xcf, 0xe9, 0xb5, 0xdb,
        0xa5, 0x39, 0x56, 0xc2, 0x5b, 0x59, 0xf1, 0x11, 0xf1, i[0], i[1], i[2], i[3], i[4], i[5],
        i[6], i[7],
    ]);
    let mut unstructured_data = vec![0_u8; 1 << 12];
    while source.len() < count {
        rng.fill_bytes(&mut unstructured_data);
        let mut u = Unstructured::new(&unstructured_data);
        if let Ok(node) = Node::arbitrary(&mut u) {
            write!(source, "{node}").expect("should be able to dump Node");
        }
    }
}

#[cold]
fn failure(err: ParseError, seed: u64, source: &str) -> ! {
    let path = temp_dir().join("failed-askama-bench.jinja2");
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .expect("should be able to write to temp dir")
        .write_all(source.as_bytes())
        .expect("should be able to write to temp dir");
    panic!(
        "\nCould not parse source for seed #{}:\n{}\n\nDumped source into: {:?}",
        seed, err, path,
    );
}

criterion_group!(benches, by_min_size);
criterion_main!(benches);
