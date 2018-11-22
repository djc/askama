extern crate askama_escape;
#[macro_use]
extern crate criterion;

use askama_escape::escape;
use criterion::Criterion;

criterion_main!(benches);
criterion_group!(benches, functions);

fn functions(c: &mut Criterion) {
    c.bench_function("toString 1 bytes", format_short);
    c.bench_function("No Escaping 1 bytes", no_escaping_short);
    c.bench_function("Escaping 1 bytes", escaping_short);
    c.bench_function("toString 10 bytes", format);
    c.bench_function("No Escaping 10 bytes", no_escaping);
    c.bench_function("Escaping 10 bytes", escaping);
    c.bench_function("toString 5 MB", format_long);
    c.bench_function("No Escaping 5 MB", no_escaping_long);
    c.bench_function("Escaping 5 MB", escaping_long);
}

static A: &str = "a";
static E: &str = "<";

fn escaping_short(b: &mut criterion::Bencher) {
    b.iter(|| escape(E).to_string());
}

fn no_escaping_short(b: &mut criterion::Bencher) {
    b.iter(|| {
        escape(A).to_string();
    });
}

fn format_short(b: &mut criterion::Bencher) {
    b.iter(|| A.to_string());
}

fn escaping(b: &mut criterion::Bencher) {
    // 10 bytes at 10% escape
    let string: &str = &[A, A, A, A, A, E, A, A, A, A, A].join("");

    b.iter(|| escape(string).to_string());
}

fn no_escaping(b: &mut criterion::Bencher) {
    let no_escape: &str = &A.repeat(10);

    b.iter(|| escape(no_escape).to_string());
}

fn format(b: &mut criterion::Bencher) {
    let string: &str = &A.repeat(10);

    b.iter(|| string.to_string());
}

fn escaping_long(b: &mut criterion::Bencher) {
    // 5 MB at 3.125% escape
    let string: &str = &[&A.repeat(15), E, &A.repeat(16)]
        .join("")
        .repeat(160 * 1024);

    b.iter(|| escape(string).to_string());
}

fn no_escaping_long(b: &mut criterion::Bencher) {
    let no_escape: &str = &A.repeat(5 * 1024 * 1024);

    b.iter(|| escape(no_escape).to_string());
}

fn format_long(b: &mut criterion::Bencher) {
    let string: &str = &A.repeat(5 * 1024 * 1024);

    b.iter(|| string.to_string());
}
