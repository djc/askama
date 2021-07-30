#![allow(clippy::type_complexity)]

use std::fmt;
use std::ops::Range;

use askama::Template;

#[derive(Template)]
#[template(path = "for.html")]
struct ForTemplate<'a> {
    strings: Vec<&'a str>,
    tuple_strings: Vec<(&'a str, &'a str)>,
}

#[test]
fn test_for() {
    let s = ForTemplate {
        strings: vec!["A", "alfa", "1"],
        tuple_strings: vec![("B", "beta")],
    };
    assert_eq!(
        s.render().unwrap(),
        "0. A (first)\n1. alfa\n2. 1\n\n0. B,beta (first)\n"
    );
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
    assert_eq!(
        s.render().unwrap(),
        "0. A2 (first)\n1. alfa4\n2. 16 (last)\n"
    );
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
    assert_eq!(
        s.render().unwrap(),
        "foo (first)\nfoo (last)\nbar\nbar\nfoo\nbar\nbar\n"
    );
}

#[derive(Template)]
#[template(source = "{% for i in [1, 2, 3] %}{{ i }}{% endfor %}", ext = "txt")]
struct ForArrayTemplate;

#[test]
fn test_for_array() {
    let t = ForArrayTemplate;
    assert_eq!(t.render().unwrap(), "123");
}

#[derive(Template)]
#[template(
    source = "{% for i in [1, 2, 3].iter() %}{{ i }}{% endfor %}",
    ext = "txt"
)]
struct ForMethodCallTemplate;

#[test]
fn test_for_method_call() {
    let t = ForMethodCallTemplate;
    assert_eq!(t.render().unwrap(), "123");
}

#[derive(Template)]
#[template(
    source = "{% for i in ::std::iter::repeat(\"a\").take(5) %}{{ i }}{% endfor %}",
    ext = "txt"
)]
struct ForPathCallTemplate;

#[test]
fn test_for_path_call() {
    assert_eq!(ForPathCallTemplate.render().unwrap(), "aaaaa");
}

#[derive(Template)]
#[template(
    source = "{% for i in [1, 2, 3, 4, 5][3..] %}{{ i }}{% endfor %}",
    ext = "txt"
)]
struct ForIndexTemplate;

#[test]
fn test_for_index() {
    let t = ForIndexTemplate;
    assert_eq!(t.render().unwrap(), "45");
}

#[derive(Template)]
#[template(
    source = "{% for (i, j) in (0..10).zip(10..20).zip(30..40) %}{{ i.0 }} {{ i.1 }} {{ j }} {% endfor %}",
    ext = "txt"
)]
struct ForZipRangesTemplate;

#[test]
fn test_for_zip_ranges() {
    let t = ForZipRangesTemplate;
    assert_eq!(
        t.render().unwrap(),
        "0 10 30 1 11 31 2 12 32 3 13 33 4 14 34 5 15 35 6 16 36 7 17 37 8 18 38 9 19 39 "
    );
}

struct ForVecAttrVec {
    iterable: Vec<i32>,
}

#[derive(Template)]
#[template(
    source = "{% for x in v %}{% for y in x.iterable %}{{ y }} {% endfor %}{% endfor %}",
    ext = "txt"
)]
struct ForVecAttrVecTemplate {
    v: Vec<ForVecAttrVec>,
}

#[test]
fn test_for_vec_attr_vec() {
    let t = ForVecAttrVecTemplate {
        v: vec![
            ForVecAttrVec {
                iterable: vec![1, 2],
            },
            ForVecAttrVec {
                iterable: vec![3, 4],
            },
            ForVecAttrVec {
                iterable: vec![5, 6],
            },
        ],
    };
    assert_eq!(t.render().unwrap(), "1 2 3 4 5 6 ");
}

struct ForVecAttrSlice {
    iterable: &'static [i32],
}

#[derive(Template)]
#[template(
    source = "{% for x in v %}{% for y in x.iterable %}{{ y }} {% endfor %}{% endfor %}",
    ext = "txt"
)]
struct ForVecAttrSliceTemplate {
    v: Vec<ForVecAttrSlice>,
}

#[test]
fn test_for_vec_attr_slice() {
    let t = ForVecAttrSliceTemplate {
        v: vec![
            ForVecAttrSlice { iterable: &[1, 2] },
            ForVecAttrSlice { iterable: &[3, 4] },
            ForVecAttrSlice { iterable: &[5, 6] },
        ],
    };
    assert_eq!(t.render().unwrap(), "1 2 3 4 5 6 ");
}

struct ForVecAttrRange {
    iterable: Range<usize>,
}

#[derive(Template)]
#[template(
    source = "{% for x in v %}{% for y in x.iterable.clone() %}{{ y }} {% endfor %}{% endfor %}",
    ext = "txt"
)]
struct ForVecAttrRangeTemplate {
    v: Vec<ForVecAttrRange>,
}

#[test]
fn test_for_vec_attr_range() {
    let t = ForVecAttrRangeTemplate {
        v: vec![
            ForVecAttrRange { iterable: 1..3 },
            ForVecAttrRange { iterable: 3..5 },
            ForVecAttrRange { iterable: 5..7 },
        ],
    };
    assert_eq!(t.render().unwrap(), "1 2 3 4 5 6 ");
}

#[derive(Template)]
#[template(
    source = "{% for v in v %}{% let v = v %}{% for v in v.iterable %}{% let v = v %}{{ v }} {% endfor %}{% endfor %}",
    ext = "txt"
)]
struct ForVecAttrSliceShadowingTemplate {
    v: Vec<ForVecAttrSlice>,
}

#[test]
fn test_for_vec_attr_slice_shadowing() {
    let t = ForVecAttrSliceShadowingTemplate {
        v: vec![
            ForVecAttrSlice { iterable: &[1, 2] },
            ForVecAttrSlice { iterable: &[3, 4] },
            ForVecAttrSlice { iterable: &[5, 6] },
        ],
    };
    assert_eq!(t.render().unwrap(), "1 2 3 4 5 6 ");
}

struct NotClonable<T: fmt::Display>(T);

impl<T: fmt::Display> fmt::Display for NotClonable<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Template)]
#[template(
    source = "{% for (((a,), b), c) in v %}{{a}}{{b}}{{c}}-{% endfor %}",
    ext = "txt"
)]
struct ForDestructoringRefTupleTemplate<'a> {
    v: &'a [(((char,), NotClonable<char>), &'a char)],
}

#[test]
fn test_for_destructoring_ref_tuple() {
    let v = [
        ((('a',), NotClonable('b')), &'c'),
        ((('d',), NotClonable('e')), &'f'),
        ((('g',), NotClonable('h')), &'i'),
    ];
    let t = ForDestructoringRefTupleTemplate { v: &v };
    assert_eq!(t.render().unwrap(), "abc-def-ghi-");
}

#[derive(Template)]
#[template(
    source = "{% for (((a,), b), c) in v %}{{a}}{{b}}{{c}}-{% endfor %}",
    ext = "txt"
)]
struct ForDestructoringTupleTemplate<'a, const N: usize> {
    v: [(((char,), NotClonable<char>), &'a char); N],
}

#[test]
fn test_for_destructoring_tuple() {
    let t = ForDestructoringTupleTemplate {
        v: [
            ((('a',), NotClonable('b')), &'c'),
            ((('d',), NotClonable('e')), &'f'),
            ((('g',), NotClonable('h')), &'i'),
        ],
    };
    assert_eq!(t.render().unwrap(), "abc-def-ghi-");
}

#[derive(Template)]
#[template(
    source = "{% for (i, msg) in messages.iter().enumerate() %}{{i}}={{msg}}-{% endfor %}",
    ext = "txt"
)]
struct ForEnumerateTemplate<'a> {
    messages: &'a [&'a str],
}

#[test]
fn test_for_enumerate() {
    let t = ForEnumerateTemplate {
        messages: &["hello", "world", "!"],
    };
    assert_eq!(t.render().unwrap(), "0=hello-1=world-2=!-");
}

#[derive(Template)]
#[template(
    source = "{% for v in values.iter() %}x{{v}}{% if matches!(v, x if *x==3) %}{% break %}{% endif %}y{% endfor %}",
    ext = "txt"
)]
struct Break<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_break() {
    let t = Break {
        values: &[1, 2, 3, 4, 5],
    };
    assert_eq!(t.render().unwrap(), "x1yx2yx3");

    let t = Break {
        values: &[1, 2, 4, 5],
    };
    assert_eq!(t.render().unwrap(), "x1yx2yx4yx5y");
}

#[derive(Template)]
#[template(
    source = "{% for v in values %}x{{v}}{% if matches!(v, x if *x==3) %}{% continue %}{% endif %}y{% endfor %}",
    ext = "txt"
)]
struct Continue<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_continue() {
    let t = Continue {
        values: &[1, 2, 3, 4, 5],
    };
    assert_eq!(t.render().unwrap(), "x1yx2yx3x4yx5y");

    let t = Continue {
        values: &[1, 2, 4, 5],
    };
    assert_eq!(t.render().unwrap(), "x1yx2yx4yx5y");
}

#[derive(Template)]
#[template(path = "for-break-continue.html")]
struct BreakContinue<'a> {
    values: &'a [i32],
}

#[test]
fn test_loop_break_continue() {
    let t = BreakContinue {
        values: &[1, 2, 3, 4, 5],
    };
    assert_eq!(t.render().unwrap(), "x1yx2yx3yx4yx5y");

    let t = BreakContinue {
        values: &[1, 2, 3, 10, 4, 5],
    };
    assert_eq!(t.render().unwrap(), "x1yx2yx3yx10");

    let t = BreakContinue {
        values: &[1, 2, 3, 11, 4, 5],
    };
    assert_eq!(t.render().unwrap(), "x1yx2yx3yx11x4yx5y");
}
