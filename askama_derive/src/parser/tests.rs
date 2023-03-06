use crate::parser::{Block, Expr, Lit, Node, Syntax, Tag, Target, Whitespace, Ws};

fn check_ws_split(s: &str, res: &(&str, &str, &str)) {
    let Lit { lws, val, rws } = super::split_ws_parts(s);
    assert_eq!(lws, res.0);
    assert_eq!(val, res.1);
    assert_eq!(rws, res.2);
}

#[test]
fn test_ws_splitter() {
    check_ws_split("", &("", "", ""));
    check_ws_split("a", &("", "a", ""));
    check_ws_split("\ta", &("\t", "a", ""));
    check_ws_split("b\n", &("", "b", "\n"));
    check_ws_split(" \t\r\n", &(" \t\r\n", "", ""));
}

#[test]
#[should_panic]
fn test_invalid_block() {
    super::parse("{% extend \"blah\" %}", &Syntax::default()).unwrap();
}

#[test]
fn test_parse_filter() {
    use Expr::*;
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{{ strvar|e }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Filter("e", vec![Var("strvar")])),
        )],
    );
    assert_eq!(
        super::parse("{{ 2|abs }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Filter("abs", vec![NumLit("2")])),
        )],
    );
    assert_eq!(
        super::parse("{{ -2|abs }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Filter("abs", vec![Unary("-", NumLit("2").into())])),
        )],
    );
    assert_eq!(
        super::parse("{{ (1 - 2)|abs }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Filter(
                "abs",
                vec![Group(
                    BinOp("-", NumLit("1").into(), NumLit("2").into()).into()
                )]
            )),
        )],
    );
}

#[test]
fn test_parse_numbers() {
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{{ 2 }}", &syntax).unwrap(),
        vec![Node::Tag(Ws(None, None), Tag::Expr(Expr::NumLit("2")))],
    );
    assert_eq!(
        super::parse("{{ 2.5 }}", &syntax).unwrap(),
        vec![Node::Tag(Ws(None, None), Tag::Expr(Expr::NumLit("2.5")))],
    );
}

#[test]
fn test_parse_var() {
    let s = Syntax::default();

    assert_eq!(
        super::parse("{{ foo }}", &s).unwrap(),
        vec![Node::Tag(Ws(None, None), Tag::Expr(Expr::Var("foo")))],
    );
    assert_eq!(
        super::parse("{{ foo_bar }}", &s).unwrap(),
        vec![Node::Tag(Ws(None, None), Tag::Expr(Expr::Var("foo_bar")))],
    );

    assert_eq!(
        super::parse("{{ none }}", &s).unwrap(),
        vec![Node::Tag(Ws(None, None), Tag::Expr(Expr::Var("none")))],
    );
}

#[test]
fn test_parse_const() {
    let s = Syntax::default();

    assert_eq!(
        super::parse("{{ FOO }}", &s).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Path(vec!["FOO"])),
        )],
    );
    assert_eq!(
        super::parse("{{ FOO_BAR }}", &s).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Path(vec!["FOO_BAR"])),
        )],
    );

    assert_eq!(
        super::parse("{{ NONE }}", &s).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Path(vec!["NONE"])),
        )],
    );
}

#[test]
fn test_parse_path() {
    let s = Syntax::default();

    assert_eq!(
        super::parse("{{ None }}", &s).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Path(vec!["None"])),
        )],
    );
    assert_eq!(
        super::parse("{{ Some(123) }}", &s).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Call(
                Box::new(Expr::Path(vec!["Some"])),
                vec![Expr::NumLit("123")],
            )),
        )],
    );

    assert_eq!(
        super::parse("{{ Ok(123) }}", &s).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Call(
                Box::new(Expr::Path(vec!["Ok"])),
                vec![Expr::NumLit("123")],
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ Err(123) }}", &s).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Call(
                Box::new(Expr::Path(vec!["Err"])),
                vec![Expr::NumLit("123")],
            )),
        )],
    );
}

#[test]
fn test_parse_var_call() {
    assert_eq!(
        super::parse("{{ function(\"123\", 3) }}", &Syntax::default()).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Call(
                Box::new(Expr::Var("function")),
                vec![Expr::StrLit("123"), Expr::NumLit("3")],
            )),
        )],
    );
}

#[test]
fn test_parse_path_call() {
    let s = Syntax::default();

    assert_eq!(
        super::parse("{{ Option::None }}", &s).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Path(vec!["Option", "None"])),
        )],
    );
    assert_eq!(
        super::parse("{{ Option::Some(123) }}", &s).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Call(
                Box::new(Expr::Path(vec!["Option", "Some"])),
                vec![Expr::NumLit("123")],
            )),
        )],
    );

    assert_eq!(
        super::parse("{{ self::function(\"123\", 3) }}", &s).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Call(
                Box::new(Expr::Path(vec!["self", "function"])),
                vec![Expr::StrLit("123"), Expr::NumLit("3")],
            )),
        )],
    );
}

#[test]
fn test_parse_root_path() {
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{{ std::string::String::new() }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Call(
                Box::new(Expr::Path(vec!["std", "string", "String", "new"])),
                vec![],
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ ::std::string::String::new() }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Expr::Call(
                Box::new(Expr::Path(vec!["", "std", "string", "String", "new"])),
                vec![],
            )),
        )],
    );
}

#[test]
fn change_delimiters_parse_filter() {
    let syntax = Syntax {
        expr_start: "{=",
        expr_end: "=}",
        ..Syntax::default()
    };

    super::parse("{= strvar|e =}", &syntax).unwrap();
}

#[test]
fn test_precedence() {
    use Expr::*;
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{{ a + b == c }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "==",
                BinOp("+", Var("a").into(), Var("b").into()).into(),
                Var("c").into(),
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ a + b * c - d / e }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "-",
                BinOp(
                    "+",
                    Var("a").into(),
                    BinOp("*", Var("b").into(), Var("c").into()).into(),
                )
                .into(),
                BinOp("/", Var("d").into(), Var("e").into()).into(),
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ a * (b + c) / -d }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "/",
                BinOp(
                    "*",
                    Var("a").into(),
                    Group(BinOp("+", Var("b").into(), Var("c").into()).into()).into()
                )
                .into(),
                Unary("-", Var("d").into()).into()
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ a || b && c || d && e }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "||",
                BinOp(
                    "||",
                    Var("a").into(),
                    BinOp("&&", Var("b").into(), Var("c").into()).into(),
                )
                .into(),
                BinOp("&&", Var("d").into(), Var("e").into()).into(),
            )),
        )],
    );
}

#[test]
fn test_associativity() {
    use Expr::*;
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{{ a + b + c }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "+",
                BinOp("+", Var("a").into(), Var("b").into()).into(),
                Var("c").into()
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ a * b * c }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "*",
                BinOp("*", Var("a").into(), Var("b").into()).into(),
                Var("c").into()
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ a && b && c }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "&&",
                BinOp("&&", Var("a").into(), Var("b").into()).into(),
                Var("c").into()
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ a + b - c + d }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "+",
                BinOp(
                    "-",
                    BinOp("+", Var("a").into(), Var("b").into()).into(),
                    Var("c").into()
                )
                .into(),
                Var("d").into()
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ a == b != c > d > e == f }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "==",
                BinOp(
                    ">",
                    BinOp(
                        ">",
                        BinOp(
                            "!=",
                            BinOp("==", Var("a").into(), Var("b").into()).into(),
                            Var("c").into()
                        )
                        .into(),
                        Var("d").into()
                    )
                    .into(),
                    Var("e").into()
                )
                .into(),
                Var("f").into()
            )),
        )],
    );
}

#[test]
fn test_odd_calls() {
    use Expr::*;
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{{ a[b](c) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Call(
                Box::new(Index(Box::new(Var("a")), Box::new(Var("b")))),
                vec![Var("c")],
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ (a + b)(c) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Call(
                Box::new(Group(Box::new(BinOp(
                    "+",
                    Box::new(Var("a")),
                    Box::new(Var("b"))
                )))),
                vec![Var("c")],
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ a + b(c) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "+",
                Box::new(Var("a")),
                Box::new(Call(Box::new(Var("b")), vec![Var("c")])),
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ (-a)(b) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Call(
                Box::new(Group(Box::new(Unary("-", Box::new(Var("a")))))),
                vec![Var("b")],
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ -a(b) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Unary(
                "-",
                Box::new(Call(Box::new(Var("a")), vec![Var("b")]))
            )),
        )],
    );
}

#[test]
fn test_parse_comments() {
    let s = &Syntax::default();

    assert_eq!(
        super::parse("{##}", s).unwrap(),
        vec![Node::Tag(Ws(None, None), Tag::Comment)],
    );
    assert_eq!(
        super::parse("{#- #}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Suppress), None),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{# -#}", s).unwrap(),
        vec![Node::Tag(
            Ws(None, Some(Whitespace::Suppress)),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{#--#}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Suppress), Some(Whitespace::Suppress)),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{#- foo\n bar -#}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Suppress), Some(Whitespace::Suppress)),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{#- foo\n {#- bar\n -#} baz -#}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Suppress), Some(Whitespace::Suppress)),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{#+ #}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Preserve), None),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{# +#}", s).unwrap(),
        vec![Node::Tag(
            Ws(None, Some(Whitespace::Preserve)),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{#++#}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Preserve), Some(Whitespace::Preserve)),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{#+ foo\n bar +#}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Preserve), Some(Whitespace::Preserve)),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{#+ foo\n {#+ bar\n +#} baz -+#}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Preserve), Some(Whitespace::Preserve)),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{#~ #}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Minimize), None),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{# ~#}", s).unwrap(),
        vec![Node::Tag(
            Ws(None, Some(Whitespace::Minimize)),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{#~~#}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Minimize), Some(Whitespace::Minimize)),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{#~ foo\n bar ~#}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Minimize), Some(Whitespace::Minimize)),
            Tag::Comment
        )],
    );
    assert_eq!(
        super::parse("{#~ foo\n {#~ bar\n ~#} baz -~#}", s).unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Minimize), Some(Whitespace::Minimize)),
            Tag::Comment
        )],
    );

    assert_eq!(
        super::parse("{# foo {# bar #} {# {# baz #} qux #} #}", s).unwrap(),
        vec![Node::Tag(Ws(None, None), Tag::Comment)],
    );
}

#[test]
fn test_parse_match() {
    use super::{Match, When};
    let syntax = Syntax::default();
    assert_eq!(
        super::parse(
            "{%+ match foo %}{% when Foo ~%}{%- else +%}{%~ endmatch %}",
            &syntax
        )
        .unwrap(),
        vec![Node::Tag(
            Ws(Some(Whitespace::Preserve), None),
            Tag::Match(Match {
                expr: Expr::Var("foo"),
                arms: vec![
                    When {
                        target: Target::Path(vec!["Foo"]),
                        block: Block::with_whitespace(Ws(
                            Some(Whitespace::Suppress),
                            Some(Whitespace::Minimize),
                        )),
                    },
                    When {
                        target: Target::Name("_"),
                        block: Block::with_whitespace(Ws(
                            Some(Whitespace::Minimize),
                            Some(Whitespace::Preserve),
                        )),
                    },
                ],
            }),
        )],
    );
}

#[test]
fn test_parse_tuple() {
    use super::Expr::*;
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{{ () }}", &syntax).unwrap(),
        vec![Node::Tag(Ws(None, None), Tag::Expr(Tuple(vec![])))],
    );
    assert_eq!(
        super::parse("{{ (1) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Group(Box::new(NumLit("1"))))
        )],
    );
    assert_eq!(
        super::parse("{{ (1,) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Tuple(vec![NumLit("1")]))
        )],
    );
    assert_eq!(
        super::parse("{{ (1, ) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Tuple(vec![NumLit("1")]))
        )],
    );
    assert_eq!(
        super::parse("{{ (1 ,) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Tuple(vec![NumLit("1")]))
        )],
    );
    assert_eq!(
        super::parse("{{ (1 , ) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Tuple(vec![NumLit("1")]))
        )],
    );
    assert_eq!(
        super::parse("{{ (1, 2) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Tuple(vec![NumLit("1"), NumLit("2")])),
        )],
    );
    assert_eq!(
        super::parse("{{ (1, 2,) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Tuple(vec![NumLit("1"), NumLit("2")])),
        )],
    );
    assert_eq!(
        super::parse("{{ (1, 2, 3) }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Tuple(vec![NumLit("1"), NumLit("2"), NumLit("3")])),
        )],
    );
    assert_eq!(
        super::parse("{{ ()|abs }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Filter("abs", vec![Tuple(vec![])])),
        )],
    );
    assert_eq!(
        super::parse("{{ () | abs }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp("|", Box::new(Tuple(vec![])), Box::new(Var("abs")))),
        )],
    );
    assert_eq!(
        super::parse("{{ (1)|abs }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Filter("abs", vec![Group(Box::new(NumLit("1")))])),
        )],
    );
    assert_eq!(
        super::parse("{{ (1) | abs }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "|",
                Box::new(Group(Box::new(NumLit("1")))),
                Box::new(Var("abs"))
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ (1,)|abs }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Filter("abs", vec![Tuple(vec![NumLit("1")])])),
        )],
    );
    assert_eq!(
        super::parse("{{ (1,) | abs }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "|",
                Box::new(Tuple(vec![NumLit("1")])),
                Box::new(Var("abs"))
            )),
        )],
    );
    assert_eq!(
        super::parse("{{ (1, 2)|abs }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(Filter("abs", vec![Tuple(vec![NumLit("1"), NumLit("2")])])),
        )],
    );
    assert_eq!(
        super::parse("{{ (1, 2) | abs }}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Expr(BinOp(
                "|",
                Box::new(Tuple(vec![NumLit("1"), NumLit("2")])),
                Box::new(Var("abs"))
            )),
        )],
    );
}

#[test]
fn test_parse_loop() {
    use super::{Expr, Loop, Target};
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{% for user in users +%}{%~ else -%}{%+ endfor %}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Loop(Loop {
                var: Target::Name("user"),
                iter: Expr::Var("users"),
                cond: None,
                body: Block::with_whitespace(Ws(
                    Some(Whitespace::Minimize),
                    Some(Whitespace::Preserve),
                )),
                else_block: Block::with_whitespace(Ws(
                    Some(Whitespace::Preserve),
                    Some(Whitespace::Suppress),
                )),
            }),
        )]
    );
    assert_eq!(
        super::parse("{% for user in users +%}{%~ endfor -%}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, Some(Whitespace::Suppress)),
            Tag::Loop(Loop {
                var: Target::Name("user"),
                iter: Expr::Var("users"),
                cond: None,
                body: Block::with_whitespace(Ws(
                    Some(Whitespace::Minimize),
                    Some(Whitespace::Preserve),
                )),
                else_block: Block::with_whitespace(Ws(None, None)),
            }),
        )]
    );
}

#[test]
fn test_missing_space_after_kw() {
    let syntax = Syntax::default();
    let err = super::parse("{%leta=b%}", &syntax).unwrap_err();
    assert_eq!(err, "unable to parse template:\n\n\"{%leta=b%}\"");
}

#[test]
fn test_parse_call_statement() {
    use super::Call;
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{% call foo(bar) %}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Call(Call {
                scope: None,
                name: "foo",
                args: vec![Expr::Var("bar"),],
            }),
        )],
    );
    assert_eq!(
        super::parse("{% call foo::bar() %}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, None),
            Tag::Call(Call {
                scope: Some("foo"),
                name: "bar",
                args: vec![],
            }),
        )],
    );
}

#[test]
fn test_parse_macro_statement() {
    use super::Macro;
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{% macro foo(bar) -%}{%~ endmacro +%}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, Some(Whitespace::Preserve)),
            Tag::Macro(Macro {
                name: "foo",
                args: vec!["bar"],
                block: Block::with_whitespace(Ws(
                    Some(Whitespace::Minimize),
                    Some(Whitespace::Suppress),
                )),
            }),
        )]
    );
}

#[test]
fn test_parse_raw_block() {
    use super::Raw;
    let syntax = Syntax::default();
    assert_eq!(
        super::parse(
            "{% raw -%}{% if condition %}{{ result }}{% endif %}{%~ endraw +%}",
            &syntax
        )
        .unwrap(),
        vec![Node::Tag(
            Ws(None, Some(Whitespace::Preserve)),
            Tag::Raw(Raw {
                lit: Lit {
                    lws: "",
                    val: "{% if condition %}{{ result }}{% endif %}",
                    rws: "",
                },
                ws: Ws(Some(Whitespace::Minimize), Some(Whitespace::Suppress)),
            }),
        )]
    );
}

#[test]
fn test_parse_block_def() {
    use super::BlockDef;
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{% block foo -%}{%~ endblock +%}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, Some(Whitespace::Preserve)),
            Tag::BlockDef(BlockDef {
                name: "foo",
                block: vec![],
                ws: Ws(Some(Whitespace::Minimize), Some(Whitespace::Suppress)),
            }),
        )],
    );
}

#[test]
fn test_parse_cond() {
    use super::{Cond, CondTest};
    let syntax = Syntax::default();
    assert_eq!(
        super::parse("{% if condition -%}{%~ endif +%}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, Some(Whitespace::Preserve)),
            Tag::Cond(vec![Cond {
                test: Some(CondTest {
                    expr: Expr::Var("condition"),
                    target: None,
                }),
                block: vec![],
                ws: Ws(Some(Whitespace::Minimize), Some(Whitespace::Suppress)),
            }]),
        )],
    );
    assert_eq!(
        super::parse("{% if let Some(val) = condition -%}{%~ endif +%}", &syntax).unwrap(),
        vec![Node::Tag(
            Ws(None, Some(Whitespace::Preserve)),
            Tag::Cond(vec![Cond {
                test: Some(CondTest {
                    expr: Expr::Var("condition"),
                    target: Some(Target::Tuple(vec!["Some"], vec![Target::Name("val")],)),
                }),
                block: vec![],
                ws: Ws(Some(Whitespace::Minimize), Some(Whitespace::Suppress)),
            }]),
        )],
    );
    assert_eq!(
        super::parse(
            "{% if condition -%}{%+ else if other -%}{%~ else %}{%~ endif +%}",
            &syntax
        )
        .unwrap(),
        vec![Node::Tag(
            Ws(None, Some(Whitespace::Preserve)),
            Tag::Cond(vec![
                Cond {
                    test: Some(CondTest {
                        expr: Expr::Var("condition"),
                        target: None,
                    }),
                    block: vec![],
                    ws: Ws(Some(Whitespace::Preserve), Some(Whitespace::Suppress)),
                },
                Cond {
                    test: Some(CondTest {
                        expr: Expr::Var("other"),
                        target: None,
                    }),
                    block: vec![],
                    ws: Ws(Some(Whitespace::Minimize), Some(Whitespace::Suppress)),
                },
                Cond {
                    test: None,
                    block: vec![],
                    ws: Ws(Some(Whitespace::Minimize), None),
                },
            ]),
        )],
    );
}
