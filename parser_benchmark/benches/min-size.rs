use std::env::temp_dir;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parser::{Ast, ParseError, Syntax};
use parser_benchmark::min_size;

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

fn by_min_size(c: &mut Criterion) {
    let syntax = &Syntax::default();
    for count in [100, 1000, 10_000, 100_000] {
        c.bench_function(&format!("{count} bytes"), |b| {
            b.iter_custom(|i| {
                let mut total = Duration::default();
                for seed in 0..i {
                    let source = min_size(seed.into(), count);
                    let source = source.as_str();

                    let start = Instant::now();
                    if let Err(err) = Ast::from_str(black_box(source), syntax) {
                        failure(err, seed, source);
                    }
                    total += start.elapsed();
                }
                total
            });
        });
    }
}

criterion_group!(benches, by_min_size);
criterion_main!(benches);
