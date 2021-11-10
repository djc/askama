#!/usr/bin/env python3

print(r'''use askama::Template;

#[derive(Template)]
#[template(
    source = "{% for v in values %}{{v}}{% else %}empty{% endfor %}",
    ext = "txt"
)]
struct ForElse<'a> {
    values: &'a [i32],
}

#[test]
fn test_for_else() {
    let t = ForElse { values: &[1, 2, 3] };
    assert_eq!(t.render().unwrap(), "123");

    let t = ForElse { values: &[] };
    assert_eq!(t.render().unwrap(), "empty");
}
''')

for i in range(2**6):
    a = '-' if i & 2**0 else ' '
    b = '-' if i & 2**1 else ' '
    c = '-' if i & 2**2 else ' '
    d = '-' if i & 2**3 else ' '
    e = '-' if i & 2**4 else ' '
    f = '-' if i & 2**5 else ' '
    source = fr'a {{%{a}for v in values{b}%}}\t{{{{v}}}}\t{{%{c}else{d}%}}\nX\n{{%{e}endfor{f}%}} b'

    a = '' if i & 2**0 else r' '
    b = '' if i & 2**1 else r'\t'
    c = '' if i & 2**2 else r'\t'
    d = '' if i & 2**3 else r'\n'
    e = '' if i & 2**4 else r'\n'
    f = '' if i & 2**5 else r' '
    some = f'a{a}{b}1{c}{f}b'
    none = f'a{a}{d}X{e}{f}b'

    print(f'''#[derive(Template)]
#[template(
    source = "{source}",
    ext = "txt"
)]
struct LoopElseTrim{i:02}<'a> {{
    values: &'a [i32],
}}

#[test]
fn test_loop_else_trim{i:02}() {{
    let t = LoopElseTrim{i:02} {{ values: &[1] }};
    assert_eq!(t.render().unwrap(), "{some}");

    let t = LoopElseTrim{i:02} {{ values: &[] }};
    assert_eq!(t.render().unwrap(), "{none}");
}}''')
