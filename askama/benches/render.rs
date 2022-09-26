use std::convert::{TryFrom, TryInto};
use std::time::{Duration, Instant};

use askama::Template;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

criterion_main!(benches);
criterion_group!(benches, big_table, teams);

#[derive(Template)]
#[template(
    ext = "html",
    source = r#"<table>
{% for row in table %}
<tr>{% for col in row %}<td>{{ col|escape }}</td>{% endfor %}</tr>
{% endfor %}
</table>
"#
)]
struct BigTable {
    table: Vec<Vec<usize>>,
}

struct Team {
    name: String,
    score: u8,
}

#[derive(Template)]
#[template(
    ext = "html",
    source = r#"<html>
  <head>
    <title>{{ year }}</title>
  </head>
  <body>
    <h1>CSL {{ year }}</h1>
    <ul>
    {% for team in teams %}
      <li class="{% if loop.index0 == 0 %}champion{% endif %}">
      <b>{{ team.name }}</b>: {{ team.score }}
      </li>
    {% endfor %}
    </ul>
  </body>
</html>
"#
)]
struct Teams {
    year: u16,
    teams: Vec<Team>,
}

fn big_table(c: &mut Criterion) {
    c.bench_function("Big table", |b| {
        b.iter_custom(|size| {
            let mut nanos = 0;
            for i in 0..usize::try_from(size).unwrap() {
                let size = i.checked_mul(100).unwrap();

                let mut table = Vec::with_capacity(size);
                for _ in 0..size {
                    let mut inner = Vec::with_capacity(size);
                    for i in 0..size {
                        inner.push(i);
                    }
                    table.push(inner);
                }
                let ctx = BigTable {
                    table: black_box(table),
                };

                let start = Instant::now();
                let _ = black_box(ctx).render().unwrap();
                nanos += start.elapsed().as_nanos();
            }
            Duration::from_nanos(
                nanos
                    .try_into()
                    .expect("Did the benchmark take 584 years to finish?"),
            )
        })
    });
}

fn teams(c: &mut Criterion) {
    let teams = Teams {
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
    };

    c.bench_function("Teams", |b| {
        b.iter(|| {
            let _ = black_box(&teams).render().unwrap();
        })
    });
}
