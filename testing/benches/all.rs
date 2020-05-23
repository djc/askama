#[macro_use]
extern crate criterion;

use askama::Template;
use criterion::Criterion;

criterion_main!(benches);
criterion_group!(benches, functions);

fn functions(c: &mut Criterion) {
    c.bench_function("Big table (string)", |b| big_table_string(b, 100));
    c.bench_function("Big table (bytes)", |b| big_table_bytes(b, 100));
    c.bench_function("Teams (string)", teams_string);
    c.bench_function("Teams (bytes)", teams_bytes);
}

fn big_table_string(b: &mut criterion::Bencher, size: usize) {
    let ctx = big_table_build(size);
    b.iter(|| ctx.render().unwrap());
}
fn big_table_bytes(b: &mut criterion::Bencher, size: usize) {
    let ctx = big_table_build(size);
    b.iter(|| ctx.render_bytes().unwrap());
}
fn big_table_build(size: usize) -> BigTable {
    let mut table = Vec::with_capacity(size);
    for _ in 0..size {
        let mut inner = Vec::with_capacity(size);
        for i in 0..size {
            inner.push(i);
        }
        table.push(inner);
    }
    BigTable { table }
}

#[derive(Template)]
#[template(path = "big-table.html")]
struct BigTable {
    table: Vec<Vec<usize>>,
}

fn teams_string(b: &mut criterion::Bencher) {
    let teams = teams_build();
    b.iter(|| teams.render().unwrap());
}
fn teams_bytes(b: &mut criterion::Bencher) {
    let teams = teams_build();
    b.iter(|| teams.render_bytes().unwrap());
}
fn teams_build() -> Teams {
    Teams {
        year: 2015,
        teams: vec![
            Team {
                name: "Jiangsu".into(),
                score: 43,
            },
            Team {
                name: "Beijing".into(),
                score: 27,
            },
            Team {
                name: "Guangzhou".into(),
                score: 22,
            },
            Team {
                name: "Shandong".into(),
                score: 12,
            },
        ],
    }
}

#[derive(Template)]
#[template(path = "teams.html")]
struct Teams {
    year: u16,
    teams: Vec<Team>,
}

struct Team {
    name: String,
    score: u8,
}
