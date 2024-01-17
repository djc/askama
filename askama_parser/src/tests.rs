use super::node::{Lit, Whitespace, Ws};
use super::{Ast, Expr, Node, Syntax};

fn check_ws_split(s: &str, res: &(&str, &str, &str)) {
    let Lit { lws, val, rws } = Lit::split_ws_parts(s);
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
    Ast::from_str("{% extend \"blah\" %}", &Syntax::default()).unwrap();
}

#[test]
fn test_parse_filter() {
    use Expr::*;
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ strvar|e }}", &syntax).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Filter("e", vec![Var("strvar")]),)],
    );
    assert_eq!(
        Ast::from_str("{{ 2|abs }}", &syntax).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Filter("abs", vec![NumLit("2")]),)],
    );
    assert_eq!(
        Ast::from_str("{{ -2|abs }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Filter("abs", vec![Unary("-", NumLit("2").into())]),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1 - 2)|abs }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Filter(
                "abs",
                vec![Group(
                    BinOp("-", NumLit("1").into(), NumLit("2").into()).into()
                )]
            ),
        )],
    );
}

#[test]
fn test_parse_numbers() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ 2 }}", &syntax).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Expr::NumLit("2"),)],
    );
    assert_eq!(
        Ast::from_str("{{ 2.5 }}", &syntax).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Expr::NumLit("2.5"),)],
    );
}

#[test]
fn test_parse_var() {
    let s = Syntax::default();

    assert_eq!(
        Ast::from_str("{{ foo }}", &s).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Expr::Var("foo"))],
    );
    assert_eq!(
        Ast::from_str("{{ foo_bar }}", &s).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Expr::Var("foo_bar"))],
    );

    assert_eq!(
        Ast::from_str("{{ none }}", &s).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Expr::Var("none"))],
    );
}

#[test]
fn test_parse_const() {
    let s = Syntax::default();

    assert_eq!(
        Ast::from_str("{{ FOO }}", &s).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Expr::Path(vec!["FOO"]))],
    );
    assert_eq!(
        Ast::from_str("{{ FOO_BAR }}", &s).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Expr::Path(vec!["FOO_BAR"]))],
    );

    assert_eq!(
        Ast::from_str("{{ NONE }}", &s).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Expr::Path(vec!["NONE"]))],
    );
}

#[test]
fn test_parse_path() {
    let s = Syntax::default();

    assert_eq!(
        Ast::from_str("{{ None }}", &s).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Expr::Path(vec!["None"]))],
    );
    assert_eq!(
        Ast::from_str("{{ Some(123) }}", &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Call(
                Box::new(Expr::Path(vec!["Some"])),
                vec![Expr::NumLit("123")]
            ),
        )],
    );

    assert_eq!(
        Ast::from_str("{{ Ok(123) }}", &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Call(Box::new(Expr::Path(vec!["Ok"])), vec![Expr::NumLit("123")]),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ Err(123) }}", &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Call(Box::new(Expr::Path(vec!["Err"])), vec![Expr::NumLit("123")]),
        )],
    );
}

#[test]
fn test_parse_var_call() {
    assert_eq!(
        Ast::from_str("{{ function(\"123\", 3) }}", &Syntax::default())
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Call(
                Box::new(Expr::Var("function")),
                vec![Expr::StrLit("123"), Expr::NumLit("3")]
            ),
        )],
    );
}

#[test]
fn test_parse_path_call() {
    let s = Syntax::default();

    assert_eq!(
        Ast::from_str("{{ Option::None }}", &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Path(vec!["Option", "None"])
        )],
    );
    assert_eq!(
        Ast::from_str("{{ Option::Some(123) }}", &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Call(
                Box::new(Expr::Path(vec!["Option", "Some"])),
                vec![Expr::NumLit("123")],
            ),
        )],
    );

    assert_eq!(
        Ast::from_str("{{ self::function(\"123\", 3) }}", &s)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Call(
                Box::new(Expr::Path(vec!["self", "function"])),
                vec![Expr::StrLit("123"), Expr::NumLit("3")],
            ),
        )],
    );
}

#[test]
fn test_parse_root_path() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ std::string::String::new() }}", &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Call(
                Box::new(Expr::Path(vec!["std", "string", "String", "new"])),
                vec![]
            ),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ ::std::string::String::new() }}", &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Call(
                Box::new(Expr::Path(vec!["", "std", "string", "String", "new"])),
                vec![]
            ),
        )],
    );
}

#[test]
fn test_rust_macro() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ vec!(1, 2, 3) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::RustMacro(vec!["vec"], "1, 2, 3",),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ alloc::vec!(1, 2, 3) }}", &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::RustMacro(vec!["alloc", "vec"], "1, 2, 3",),
        )],
    );
    assert_eq!(
        Ast::from_str("{{a!()}}", &syntax).unwrap().nodes,
        [Node::Expr(Ws(None, None), Expr::RustMacro(vec!["a"], ""))],
    );
    assert_eq!(
        Ast::from_str("{{a !()}}", &syntax).unwrap().nodes,
        [Node::Expr(Ws(None, None), Expr::RustMacro(vec!["a"], ""))],
    );
    assert_eq!(
        Ast::from_str("{{a! ()}}", &syntax).unwrap().nodes,
        [Node::Expr(Ws(None, None), Expr::RustMacro(vec!["a"], ""))],
    );
    assert_eq!(
        Ast::from_str("{{a ! ()}}", &syntax).unwrap().nodes,
        [Node::Expr(Ws(None, None), Expr::RustMacro(vec!["a"], ""))],
    );
    assert_eq!(
        Ast::from_str("{{A!()}}", &syntax).unwrap().nodes,
        [Node::Expr(Ws(None, None), Expr::RustMacro(vec!["A"], ""),)],
    );
    assert_eq!(
        &*Ast::from_str("{{a.b.c!( hello )}}", &syntax)
            .unwrap_err()
            .to_string(),
        "problems parsing template source at row 1, column 7 near:\n\"!( hello )}}\"",
    );
}

#[test]
fn change_delimiters_parse_filter() {
    let syntax = Syntax {
        expr_start: "{=",
        expr_end: "=}",
        ..Syntax::default()
    };

    Ast::from_str("{= strvar|e =}", &syntax).unwrap();
}

#[test]
fn test_precedence() {
    use Expr::*;
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ a + b == c }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "==",
                BinOp("+", Var("a").into(), Var("b").into()).into(),
                Var("c").into(),
            )
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a + b * c - d / e }}", &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "-",
                BinOp(
                    "+",
                    Var("a").into(),
                    BinOp("*", Var("b").into(), Var("c").into()).into(),
                )
                .into(),
                BinOp("/", Var("d").into(), Var("e").into()).into(),
            )
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a * (b + c) / -d }}", &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "/",
                BinOp(
                    "*",
                    Var("a").into(),
                    Group(BinOp("+", Var("b").into(), Var("c").into()).into()).into()
                )
                .into(),
                Unary("-", Var("d").into()).into()
            )
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a || b && c || d && e }}", &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "||",
                BinOp(
                    "||",
                    Var("a").into(),
                    BinOp("&&", Var("b").into(), Var("c").into()).into(),
                )
                .into(),
                BinOp("&&", Var("d").into(), Var("e").into()).into(),
            )
        )],
    );
}

#[test]
fn test_associativity() {
    use Expr::*;
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ a + b + c }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "+",
                BinOp("+", Var("a").into(), Var("b").into()).into(),
                Var("c").into()
            )
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a * b * c }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "*",
                BinOp("*", Var("a").into(), Var("b").into()).into(),
                Var("c").into()
            )
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a && b && c }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "&&",
                BinOp("&&", Var("a").into(), Var("b").into()).into(),
                Var("c").into()
            )
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a + b - c + d }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "+",
                BinOp(
                    "-",
                    BinOp("+", Var("a").into(), Var("b").into()).into(),
                    Var("c").into()
                )
                .into(),
                Var("d").into()
            )
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a == b != c > d > e == f }}", &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
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
            )
        )],
    );
}

#[test]
fn test_odd_calls() {
    use Expr::*;
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ a[b](c) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Call(
                Box::new(Index(Box::new(Var("a")), Box::new(Var("b")))),
                vec![Var("c")],
            ),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (a + b)(c) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Call(
                Box::new(Group(Box::new(BinOp(
                    "+",
                    Box::new(Var("a")),
                    Box::new(Var("b"))
                )))),
                vec![Var("c")],
            ),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a + b(c) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "+",
                Box::new(Var("a")),
                Box::new(Call(Box::new(Var("b")), vec![Var("c")])),
            ),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (-a)(b) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Call(
                Box::new(Group(Box::new(Unary("-", Box::new(Var("a")))))),
                vec![Var("b")],
            ),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ -a(b) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Unary("-", Box::new(Call(Box::new(Var("a")), vec![Var("b")]))),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a(b)|c }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Filter("c", vec![Call(Box::new(Var("a")), vec![Var("b")])]),
        )]
    );
    assert_eq!(
        Ast::from_str("{{ a(b)| c }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Filter("c", vec![Call(Box::new(Var("a")), vec![Var("b")])]),
        )]
    );
    assert_eq!(
        Ast::from_str("{{ a(b) |c }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "|",
                Box::new(Call(Box::new(Var("a")), vec![Var("b")])),
                Box::new(Var("c"))
            ),
        )]
    );
}

#[test]
fn test_parse_comments() {
    fn one_comment_ws(source: &str, ws: Ws) {
        let s = &Syntax::default();
        let mut nodes = Ast::from_str(source, s).unwrap().nodes;
        assert_eq!(nodes.len(), 1, "expected to parse one node");
        match nodes.pop().unwrap() {
            Node::Comment(comment) => assert_eq!(comment.ws, ws),
            node => panic!("expected a comment not, but parsed {:?}", node),
        }
    }

    one_comment_ws("{##}", Ws(None, None));
    one_comment_ws("{#- #}", Ws(Some(Whitespace::Suppress), None));
    one_comment_ws("{# -#}", Ws(None, Some(Whitespace::Suppress)));
    one_comment_ws(
        "{#--#}",
        Ws(Some(Whitespace::Suppress), Some(Whitespace::Suppress)),
    );
    one_comment_ws(
        "{#- foo\n bar -#}",
        Ws(Some(Whitespace::Suppress), Some(Whitespace::Suppress)),
    );
    one_comment_ws(
        "{#- foo\n {#- bar\n -#} baz -#}",
        Ws(Some(Whitespace::Suppress), Some(Whitespace::Suppress)),
    );
    one_comment_ws("{#+ #}", Ws(Some(Whitespace::Preserve), None));
    one_comment_ws("{# +#}", Ws(None, Some(Whitespace::Preserve)));
    one_comment_ws(
        "{#++#}",
        Ws(Some(Whitespace::Preserve), Some(Whitespace::Preserve)),
    );
    one_comment_ws(
        "{#+ foo\n bar +#}",
        Ws(Some(Whitespace::Preserve), Some(Whitespace::Preserve)),
    );
    one_comment_ws(
        "{#+ foo\n {#+ bar\n +#} baz -+#}",
        Ws(Some(Whitespace::Preserve), Some(Whitespace::Preserve)),
    );
    one_comment_ws("{#~ #}", Ws(Some(Whitespace::Minimize), None));
    one_comment_ws("{# ~#}", Ws(None, Some(Whitespace::Minimize)));
    one_comment_ws(
        "{#~~#}",
        Ws(Some(Whitespace::Minimize), Some(Whitespace::Minimize)),
    );
    one_comment_ws(
        "{#~ foo\n bar ~#}",
        Ws(Some(Whitespace::Minimize), Some(Whitespace::Minimize)),
    );
    one_comment_ws(
        "{#~ foo\n {#~ bar\n ~#} baz -~#}",
        Ws(Some(Whitespace::Minimize), Some(Whitespace::Minimize)),
    );

    one_comment_ws("{# foo {# bar #} {# {# baz #} qux #} #}", Ws(None, None));
}

#[test]
fn test_parse_tuple() {
    use super::expr::Expr::*;
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ () }}", &syntax).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Tuple(vec![]),)],
    );
    assert_eq!(
        Ast::from_str("{{ (1) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Group(Box::new(NumLit("1"))),)],
    );
    assert_eq!(
        Ast::from_str("{{ (1,) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Tuple(vec![NumLit("1")]),)],
    );
    assert_eq!(
        Ast::from_str("{{ (1, ) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Tuple(vec![NumLit("1")]),)],
    );
    assert_eq!(
        Ast::from_str("{{ (1 ,) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Tuple(vec![NumLit("1")]),)],
    );
    assert_eq!(
        Ast::from_str("{{ (1 , ) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Tuple(vec![NumLit("1")]),)],
    );
    assert_eq!(
        Ast::from_str("{{ (1, 2) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Tuple(vec![NumLit("1"), NumLit("2")]),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1, 2,) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Tuple(vec![NumLit("1"), NumLit("2")]),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1, 2, 3) }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Tuple(vec![NumLit("1"), NumLit("2"), NumLit("3")]),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ ()|abs }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Filter("abs", vec![Tuple(vec![])]),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ () | abs }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp("|", Box::new(Tuple(vec![])), Box::new(Var("abs"))),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1)|abs }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Filter("abs", vec![Group(Box::new(NumLit("1")))]),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1) | abs }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "|",
                Box::new(Group(Box::new(NumLit("1")))),
                Box::new(Var("abs"))
            ),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1,)|abs }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Filter("abs", vec![Tuple(vec![NumLit("1")])]),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1,) | abs }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "|",
                Box::new(Tuple(vec![NumLit("1")])),
                Box::new(Var("abs"))
            ),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1, 2)|abs }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Filter("abs", vec![Tuple(vec![NumLit("1"), NumLit("2")])]),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1, 2) | abs }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            BinOp(
                "|",
                Box::new(Tuple(vec![NumLit("1"), NumLit("2")])),
                Box::new(Var("abs"))
            ),
        )],
    );
}

#[test]
fn test_missing_space_after_kw() {
    let syntax = Syntax::default();
    let err = Ast::from_str("{%leta=b%}", &syntax).unwrap_err();
    assert!(matches!(
        &*err.to_string(),
        "problems parsing template source at row 1, column 0 near:\n\"{%leta=b%}\"",
    ));
}

#[test]
fn test_parse_array() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ [] }}", &syntax).unwrap().nodes,
        vec![Node::Expr(Ws(None, None), Expr::Array(vec![]))],
    );
    assert_eq!(
        Ast::from_str("{{ [1] }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Array(vec![Expr::NumLit("1")])
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [ 1] }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Array(vec![Expr::NumLit("1")])
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [1 ] }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Array(vec![Expr::NumLit("1")])
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [1,2] }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Array(vec![Expr::NumLit("1"), Expr::NumLit("2")])
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [1 ,2] }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Array(vec![Expr::NumLit("1"), Expr::NumLit("2")])
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [1, 2] }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Array(vec![Expr::NumLit("1"), Expr::NumLit("2")])
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [1,2 ] }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Array(vec![Expr::NumLit("1"), Expr::NumLit("2")])
        )],
    );
    assert_eq!(
        Ast::from_str("{{ []|foo }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Filter("foo", vec![Expr::Array(vec![])])
        )],
    );
    assert_eq!(
        Ast::from_str("{{ []| foo }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::Filter("foo", vec![Expr::Array(vec![])])
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [] |foo }}", &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            Expr::BinOp(
                "|",
                Box::new(Expr::Array(vec![])),
                Box::new(Expr::Var("foo"))
            ),
        )],
    );
}

#[test]
fn fuzzed_unicode_slice() {
    let d = "{eeuuu{b&{!!&{!!11{{
            0!(!1q҄א!)!!!!!!n!";
    assert!(Ast::from_str(d, &Syntax::default()).is_err());
}

#[test]
fn fuzzed_macro_no_end() {
    let s = "{%macro super%}{%endmacro";
    assert!(Ast::from_str(s, &Syntax::default()).is_err());
}

#[test]
fn fuzzed_target_recursion() {
    const TEMPLATE: &str = include_str!("../tests/target-recursion.txt");
    assert!(Ast::from_str(TEMPLATE, &Syntax::default()).is_err());
}

#[test]
fn fuzzed_unary_recursion() {
    const TEMPLATE: &str = include_str!("../tests/unary-recursion.txt");
    assert!(Ast::from_str(TEMPLATE, &Syntax::default()).is_err());
}
