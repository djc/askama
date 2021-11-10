use askama::Template;

#[derive(Template)]
#[template(
    source = "{% for v in values %}{{ v }}{% else %}empty{% endfor %}",
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

#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{% else %}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim00<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim00() {
    let t = LoopElseTrim00 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1\t b");

    let t = LoopElseTrim00 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX\n b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{% else %}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim01<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim01() {
    let t = LoopElseTrim01 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1\t b");

    let t = LoopElseTrim01 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX\n b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{% else %}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim02<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim02() {
    let t = LoopElseTrim02 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1\t b");

    let t = LoopElseTrim02 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX\n b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{% else %}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim03<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim03() {
    let t = LoopElseTrim03 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1\t b");

    let t = LoopElseTrim03 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX\n b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{%-else %}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim04<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim04() {
    let t = LoopElseTrim04 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1 b");

    let t = LoopElseTrim04 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX\n b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{%-else %}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim05<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim05() {
    let t = LoopElseTrim05 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1 b");

    let t = LoopElseTrim05 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX\n b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{%-else %}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim06<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim06() {
    let t = LoopElseTrim06 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1 b");

    let t = LoopElseTrim06 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX\n b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{%-else %}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim07<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim07() {
    let t = LoopElseTrim07 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1 b");

    let t = LoopElseTrim07 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX\n b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{% else-%}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim08<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim08() {
    let t = LoopElseTrim08 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1\t b");

    let t = LoopElseTrim08 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X\n b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{% else-%}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim09<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim09() {
    let t = LoopElseTrim09 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1\t b");

    let t = LoopElseTrim09 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX\n b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{% else-%}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim10<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim10() {
    let t = LoopElseTrim10 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1\t b");

    let t = LoopElseTrim10 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X\n b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{% else-%}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim11<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim11() {
    let t = LoopElseTrim11 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1\t b");

    let t = LoopElseTrim11 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX\n b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{%-else-%}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim12<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim12() {
    let t = LoopElseTrim12 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1 b");

    let t = LoopElseTrim12 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X\n b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{%-else-%}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim13<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim13() {
    let t = LoopElseTrim13 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1 b");

    let t = LoopElseTrim13 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX\n b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{%-else-%}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim14<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim14() {
    let t = LoopElseTrim14 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1 b");

    let t = LoopElseTrim14 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X\n b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{%-else-%}\nX\n{% endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim15<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim15() {
    let t = LoopElseTrim15 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1 b");

    let t = LoopElseTrim15 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX\n b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{% else %}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim16<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim16() {
    let t = LoopElseTrim16 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1\t b");

    let t = LoopElseTrim16 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{% else %}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim17<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim17() {
    let t = LoopElseTrim17 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1\t b");

    let t = LoopElseTrim17 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{% else %}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim18<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim18() {
    let t = LoopElseTrim18 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1\t b");

    let t = LoopElseTrim18 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{% else %}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim19<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim19() {
    let t = LoopElseTrim19 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1\t b");

    let t = LoopElseTrim19 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{%-else %}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim20<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim20() {
    let t = LoopElseTrim20 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1 b");

    let t = LoopElseTrim20 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{%-else %}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim21<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim21() {
    let t = LoopElseTrim21 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1 b");

    let t = LoopElseTrim21 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{%-else %}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim22<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim22() {
    let t = LoopElseTrim22 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1 b");

    let t = LoopElseTrim22 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{%-else %}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim23<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim23() {
    let t = LoopElseTrim23 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1 b");

    let t = LoopElseTrim23 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{% else-%}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim24<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim24() {
    let t = LoopElseTrim24 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1\t b");

    let t = LoopElseTrim24 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{% else-%}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim25<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim25() {
    let t = LoopElseTrim25 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1\t b");

    let t = LoopElseTrim25 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{% else-%}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim26<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim26() {
    let t = LoopElseTrim26 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1\t b");

    let t = LoopElseTrim26 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{% else-%}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim27<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim27() {
    let t = LoopElseTrim27 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1\t b");

    let t = LoopElseTrim27 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{%-else-%}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim28<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim28() {
    let t = LoopElseTrim28 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1 b");

    let t = LoopElseTrim28 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{%-else-%}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim29<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim29() {
    let t = LoopElseTrim29 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1 b");

    let t = LoopElseTrim29 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{%-else-%}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim30<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim30() {
    let t = LoopElseTrim30 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1 b");

    let t = LoopElseTrim30 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X b");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{%-else-%}\nX\n{%-endfor %} b",
    ext = "txt"
)]
struct LoopElseTrim31<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim31() {
    let t = LoopElseTrim31 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1 b");

    let t = LoopElseTrim31 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX b");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{% else %}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim32<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim32() {
    let t = LoopElseTrim32 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1\tb");

    let t = LoopElseTrim32 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX\nb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{% else %}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim33<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim33() {
    let t = LoopElseTrim33 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1\tb");

    let t = LoopElseTrim33 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX\nb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{% else %}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim34<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim34() {
    let t = LoopElseTrim34 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1\tb");

    let t = LoopElseTrim34 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX\nb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{% else %}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim35<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim35() {
    let t = LoopElseTrim35 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1\tb");

    let t = LoopElseTrim35 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX\nb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{%-else %}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim36<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim36() {
    let t = LoopElseTrim36 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1b");

    let t = LoopElseTrim36 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX\nb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{%-else %}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim37<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim37() {
    let t = LoopElseTrim37 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1b");

    let t = LoopElseTrim37 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX\nb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{%-else %}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim38<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim38() {
    let t = LoopElseTrim38 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1b");

    let t = LoopElseTrim38 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nX\nb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{%-else %}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim39<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim39() {
    let t = LoopElseTrim39 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1b");

    let t = LoopElseTrim39 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nX\nb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{% else-%}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim40<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim40() {
    let t = LoopElseTrim40 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1\tb");

    let t = LoopElseTrim40 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X\nb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{% else-%}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim41<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim41() {
    let t = LoopElseTrim41 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1\tb");

    let t = LoopElseTrim41 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX\nb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{% else-%}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim42<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim42() {
    let t = LoopElseTrim42 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1\tb");

    let t = LoopElseTrim42 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X\nb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{% else-%}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim43<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim43() {
    let t = LoopElseTrim43 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1\tb");

    let t = LoopElseTrim43 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX\nb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{%-else-%}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim44<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim44() {
    let t = LoopElseTrim44 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1b");

    let t = LoopElseTrim44 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X\nb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{%-else-%}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim45<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim45() {
    let t = LoopElseTrim45 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1b");

    let t = LoopElseTrim45 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX\nb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{%-else-%}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim46<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim46() {
    let t = LoopElseTrim46 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1b");

    let t = LoopElseTrim46 { values: &[] };
    assert_eq!(t.render().unwrap(), "a X\nb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{%-else-%}\nX\n{% endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim47<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim47() {
    let t = LoopElseTrim47 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1b");

    let t = LoopElseTrim47 { values: &[] };
    assert_eq!(t.render().unwrap(), "aX\nb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{% else %}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim48<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim48() {
    let t = LoopElseTrim48 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1\tb");

    let t = LoopElseTrim48 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nXb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{% else %}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim49<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim49() {
    let t = LoopElseTrim49 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1\tb");

    let t = LoopElseTrim49 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nXb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{% else %}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim50<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim50() {
    let t = LoopElseTrim50 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1\tb");

    let t = LoopElseTrim50 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nXb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{% else %}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim51<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim51() {
    let t = LoopElseTrim51 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1\tb");

    let t = LoopElseTrim51 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nXb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{%-else %}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim52<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim52() {
    let t = LoopElseTrim52 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1b");

    let t = LoopElseTrim52 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nXb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{%-else %}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim53<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim53() {
    let t = LoopElseTrim53 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1b");

    let t = LoopElseTrim53 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nXb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{%-else %}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim54<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim54() {
    let t = LoopElseTrim54 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1b");

    let t = LoopElseTrim54 { values: &[] };
    assert_eq!(t.render().unwrap(), "a \nXb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{%-else %}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim55<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim55() {
    let t = LoopElseTrim55 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1b");

    let t = LoopElseTrim55 { values: &[] };
    assert_eq!(t.render().unwrap(), "a\nXb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{% else-%}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim56<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim56() {
    let t = LoopElseTrim56 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1\tb");

    let t = LoopElseTrim56 { values: &[] };
    assert_eq!(t.render().unwrap(), "a Xb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{% else-%}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim57<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim57() {
    let t = LoopElseTrim57 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1\tb");

    let t = LoopElseTrim57 { values: &[] };
    assert_eq!(t.render().unwrap(), "aXb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{% else-%}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim58<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim58() {
    let t = LoopElseTrim58 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1\tb");

    let t = LoopElseTrim58 { values: &[] };
    assert_eq!(t.render().unwrap(), "a Xb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{% else-%}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim59<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim59() {
    let t = LoopElseTrim59 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1\tb");

    let t = LoopElseTrim59 { values: &[] };
    assert_eq!(t.render().unwrap(), "aXb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values %}\t{{v}}\t{%-else-%}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim60<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim60() {
    let t = LoopElseTrim60 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a \t1b");

    let t = LoopElseTrim60 { values: &[] };
    assert_eq!(t.render().unwrap(), "a Xb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values %}\t{{v}}\t{%-else-%}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim61<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim61() {
    let t = LoopElseTrim61 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a\t1b");

    let t = LoopElseTrim61 { values: &[] };
    assert_eq!(t.render().unwrap(), "aXb");
}
#[derive(Template)]
#[template(
    source = "a {% for v in values-%}\t{{v}}\t{%-else-%}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim62<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim62() {
    let t = LoopElseTrim62 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a 1b");

    let t = LoopElseTrim62 { values: &[] };
    assert_eq!(t.render().unwrap(), "a Xb");
}
#[derive(Template)]
#[template(
    source = "a {%-for v in values-%}\t{{v}}\t{%-else-%}\nX\n{%-endfor-%} b",
    ext = "txt"
)]
struct LoopElseTrim63<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_else_trim63() {
    let t = LoopElseTrim63 { values: &[1] };
    assert_eq!(t.render().unwrap(), "a1b");

    let t = LoopElseTrim63 { values: &[] };
    assert_eq!(t.render().unwrap(), "aXb");
}
