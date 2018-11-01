#[macro_use]
extern crate askama;

use askama::Template;
use std::ops::Range;

#[derive(Template)]
#[template(path = "for.html")]
struct ForTemplate<'a> {
    strings: Vec<&'a str>,
}

#[test]
fn test_for() {
    let s = ForTemplate {
        strings: vec!["A", "alfa", "1"],
    };
    assert_eq!(s.render().unwrap(), "0. A (first)\n1. alfa\n2. 1\n");
}

#[derive(Template)]
#[template(path = "nested-for.html")]
struct NestedForTemplate<'a> {
    seqs: Vec<&'a [&'a str]>,
}

#[test]
fn test_nested_for() {
    let alpha = vec!["a", "b", "c"];
    let numbers = vec!["one", "two"];
    let s = NestedForTemplate {
        seqs: vec![&alpha, &numbers],
    };
    assert_eq!(s.render().unwrap(), "1\n  0a1b2c2\n  0one1two");
}

#[derive(Template)]
#[template(path = "precedence-for.html")]
struct PrecedenceTemplate<'a> {
    strings: Vec<&'a str>,
}

#[test]
fn test_precedence_for() {
    let s = PrecedenceTemplate {
        strings: vec!["A", "alfa", "1"],
    };
    assert_eq!(s.render().unwrap(), "0. A2 (first)\n1. alfa4\n2. 16\n");
}

#[derive(Template)]
#[template(path = "for-range.html")]
struct ForRangeTemplate {
    init: i32,
    end: i32,
}

#[test]
fn test_for_range() {
    let s = ForRangeTemplate { init: -1, end: 1 };
    assert_eq!(s.render().unwrap(), "foo\nfoo\nbar\nbar\nfoo\nbar\nbar\n");
}

#[derive(Template)]
#[template(path = "for-not-borrow.html")]
struct ForNotBorrowTemplate {
    range: Range<usize>,
}

#[test]
fn test_for_not_borrow() {
    let s = ForNotBorrowTemplate { range: 0..10 };
    assert_eq!(
        s.render().unwrap(),
        "0. 9 (first)\n1. 8\n2. 7\n3. 6\n4. 5\n5. 4\n6. 3\n7. 2\n8. 1\n9. 0\n"
    );
}
