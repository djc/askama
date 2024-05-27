use super::node::{Lit, Whitespace, Ws};
use super::{Ast, Expr, Filter, Node, Syntax, WithSpan};

impl<T> WithSpan<'static, T> {
    fn no_span(inner: T) -> Self {
        Self { inner, span: "" }
    }
}

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
    Ast::from_str("{% extend \"blah\" %}", None, &Syntax::default()).unwrap();
}

#[test]
fn test_parse_filter() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ strvar|e }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "e",
                arguments: vec![WithSpan::no_span(Expr::Var("strvar"))]
            })),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ 2|abs }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "abs",
                arguments: vec![WithSpan::no_span(Expr::NumLit("2"))]
            })),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ -2|abs }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "abs",
                arguments: vec![WithSpan::no_span(Expr::Unary(
                    "-",
                    WithSpan::no_span(Expr::NumLit("2")).into()
                ))]
            })),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1 - 2)|abs }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "abs",
                arguments: vec![WithSpan::no_span(Expr::Group(
                    WithSpan::no_span(Expr::BinOp(
                        "-",
                        WithSpan::no_span(Expr::NumLit("1")).into(),
                        WithSpan::no_span(Expr::NumLit("2")).into()
                    ))
                    .into()
                ))],
            })),
        )],
    );
}

#[test]
fn test_parse_numbers() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ 2 }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::NumLit("2"))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ 2.5 }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::NumLit("2.5"))
        )],
    );
}

#[test]
fn test_parse_var() {
    let s = Syntax::default();

    assert_eq!(
        Ast::from_str("{{ foo }}", None, &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Var("foo"))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ foo_bar }}", None, &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Var("foo_bar"))
        )],
    );

    assert_eq!(
        Ast::from_str("{{ none }}", None, &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Var("none"))
        )],
    );
}

#[test]
fn test_parse_const() {
    let s = Syntax::default();

    assert_eq!(
        Ast::from_str("{{ FOO }}", None, &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Path(vec!["FOO"]))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ FOO_BAR }}", None, &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Path(vec!["FOO_BAR"]))
        )],
    );

    assert_eq!(
        Ast::from_str("{{ NONE }}", None, &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Path(vec!["NONE"]))
        )],
    );
}

#[test]
fn test_parse_path() {
    let s = Syntax::default();

    assert_eq!(
        Ast::from_str("{{ None }}", None, &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Path(vec!["None"]))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ Some(123) }}", None, &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Call(
                Box::new(WithSpan::no_span(Expr::Path(vec!["Some"]))),
                vec![WithSpan::no_span(Expr::NumLit("123"))]
            )),
        )],
    );

    assert_eq!(
        Ast::from_str("{{ Ok(123) }}", None, &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Call(
                Box::new(WithSpan::no_span(Expr::Path(vec!["Ok"]))),
                vec![WithSpan::no_span(Expr::NumLit("123"))]
            )),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ Err(123) }}", None, &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Call(
                Box::new(WithSpan::no_span(Expr::Path(vec!["Err"]))),
                vec![WithSpan::no_span(Expr::NumLit("123"))]
            )),
        )],
    );
}

#[test]
fn test_parse_var_call() {
    assert_eq!(
        Ast::from_str("{{ function(\"123\", 3) }}", None, &Syntax::default())
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Call(
                Box::new(WithSpan::no_span(Expr::Var("function"))),
                vec![
                    WithSpan::no_span(Expr::StrLit("123")),
                    WithSpan::no_span(Expr::NumLit("3"))
                ]
            )),
        )],
    );
}

#[test]
fn test_parse_path_call() {
    let s = Syntax::default();

    assert_eq!(
        Ast::from_str("{{ Option::None }}", None, &s).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Path(vec!["Option", "None"]))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ Option::Some(123) }}", None, &s)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Call(
                Box::new(WithSpan::no_span(Expr::Path(vec!["Option", "Some"]))),
                vec![WithSpan::no_span(Expr::NumLit("123"))],
            ),)
        )],
    );

    assert_eq!(
        Ast::from_str("{{ self::function(\"123\", 3) }}", None, &s)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Call(
                Box::new(WithSpan::no_span(Expr::Path(vec!["self", "function"]))),
                vec![
                    WithSpan::no_span(Expr::StrLit("123")),
                    WithSpan::no_span(Expr::NumLit("3"))
                ],
            ),)
        )],
    );
}

#[test]
fn test_parse_root_path() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ std::string::String::new() }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Call(
                Box::new(WithSpan::no_span(Expr::Path(vec![
                    "std", "string", "String", "new"
                ]))),
                vec![]
            )),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ ::std::string::String::new() }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Call(
                Box::new(WithSpan::no_span(Expr::Path(vec![
                    "", "std", "string", "String", "new"
                ]))),
                vec![]
            )),
        )],
    );
}

#[test]
fn test_rust_macro() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ vec!(1, 2, 3) }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::RustMacro(vec!["vec"], "1, 2, 3")),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ alloc::vec!(1, 2, 3) }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::RustMacro(vec!["alloc", "vec"], "1, 2, 3")),
        )],
    );
    assert_eq!(
        Ast::from_str("{{a!()}}", None, &syntax).unwrap().nodes,
        [Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::RustMacro(vec!["a"], ""))
        )],
    );
    assert_eq!(
        Ast::from_str("{{a !()}}", None, &syntax).unwrap().nodes,
        [Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::RustMacro(vec!["a"], ""))
        )],
    );
    assert_eq!(
        Ast::from_str("{{a! ()}}", None, &syntax).unwrap().nodes,
        [Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::RustMacro(vec!["a"], ""))
        )],
    );
    assert_eq!(
        Ast::from_str("{{a ! ()}}", None, &syntax).unwrap().nodes,
        [Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::RustMacro(vec!["a"], ""))
        )],
    );
    assert_eq!(
        Ast::from_str("{{A!()}}", None, &syntax).unwrap().nodes,
        [Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::RustMacro(vec!["A"], ""))
        )],
    );
    assert_eq!(
        &*Ast::from_str("{{a.b.c!( hello )}}", None, &syntax)
            .unwrap_err()
            .to_string(),
        "failed to parse template source at row 1, column 7 near:\n\"!( hello )}}\"",
    );
}

#[test]
fn change_delimiters_parse_filter() {
    let syntax = Syntax {
        expr_start: "{=",
        expr_end: "=}",
        ..Syntax::default()
    };

    Ast::from_str("{= strvar|e =}", None, &syntax).unwrap();
}

#[test]
fn test_precedence() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ a + b == c }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "==",
                WithSpan::no_span(Expr::BinOp(
                    "+",
                    WithSpan::no_span(Expr::Var("a")).into(),
                    WithSpan::no_span(Expr::Var("b")).into()
                ))
                .into(),
                WithSpan::no_span(Expr::Var("c")).into()
            ),)
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a + b * c - d / e }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "-",
                WithSpan::no_span(Expr::BinOp(
                    "+",
                    WithSpan::no_span(Expr::Var("a")).into(),
                    WithSpan::no_span(Expr::BinOp(
                        "*",
                        WithSpan::no_span(Expr::Var("b")).into(),
                        WithSpan::no_span(Expr::Var("c")).into()
                    ))
                    .into()
                ))
                .into(),
                WithSpan::no_span(Expr::BinOp(
                    "/",
                    WithSpan::no_span(Expr::Var("d")).into(),
                    WithSpan::no_span(Expr::Var("e")).into()
                ))
                .into(),
            ))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a * (b + c) / -d }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "/",
                Box::new(WithSpan::no_span(Expr::BinOp(
                    "*",
                    Box::new(WithSpan::no_span(Expr::Var("a"))),
                    Box::new(WithSpan::no_span(Expr::Group(Box::new(WithSpan::no_span(
                        Expr::BinOp(
                            "+",
                            Box::new(WithSpan::no_span(Expr::Var("b"))),
                            Box::new(WithSpan::no_span(Expr::Var("c")))
                        )
                    )))))
                ))),
                Box::new(WithSpan::no_span(Expr::Unary(
                    "-",
                    Box::new(WithSpan::no_span(Expr::Var("d")))
                )))
            ))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a || b && c || d && e }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "||",
                Box::new(WithSpan::no_span(Expr::BinOp(
                    "||",
                    Box::new(WithSpan::no_span(Expr::Var("a"))),
                    Box::new(WithSpan::no_span(Expr::BinOp(
                        "&&",
                        Box::new(WithSpan::no_span(Expr::Var("b"))),
                        Box::new(WithSpan::no_span(Expr::Var("c")))
                    ))),
                ))),
                Box::new(WithSpan::no_span(Expr::BinOp(
                    "&&",
                    Box::new(WithSpan::no_span(Expr::Var("d"))),
                    Box::new(WithSpan::no_span(Expr::Var("e")))
                ))),
            ))
        )],
    );
}

#[test]
fn test_associativity() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ a + b + c }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "+",
                Box::new(WithSpan::no_span(Expr::BinOp(
                    "+",
                    Box::new(WithSpan::no_span(Expr::Var("a"))),
                    Box::new(WithSpan::no_span(Expr::Var("b")))
                ))),
                Box::new(WithSpan::no_span(Expr::Var("c")))
            ))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a * b * c }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "*",
                Box::new(WithSpan::no_span(Expr::BinOp(
                    "*",
                    Box::new(WithSpan::no_span(Expr::Var("a"))),
                    Box::new(WithSpan::no_span(Expr::Var("b")))
                ))),
                Box::new(WithSpan::no_span(Expr::Var("c")))
            ))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a && b && c }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "&&",
                Box::new(WithSpan::no_span(Expr::BinOp(
                    "&&",
                    Box::new(WithSpan::no_span(Expr::Var("a"))),
                    Box::new(WithSpan::no_span(Expr::Var("b")))
                ))),
                Box::new(WithSpan::no_span(Expr::Var("c")))
            ))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a + b - c + d }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "+",
                Box::new(WithSpan::no_span(Expr::BinOp(
                    "-",
                    Box::new(WithSpan::no_span(Expr::BinOp(
                        "+",
                        Box::new(WithSpan::no_span(Expr::Var("a"))),
                        Box::new(WithSpan::no_span(Expr::Var("b")))
                    ))),
                    Box::new(WithSpan::no_span(Expr::Var("c")))
                ))),
                Box::new(WithSpan::no_span(Expr::Var("d")))
            ))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a == b != c > d > e == f }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "==",
                Box::new(WithSpan::no_span(Expr::BinOp(
                    ">",
                    Box::new(WithSpan::no_span(Expr::BinOp(
                        ">",
                        Box::new(WithSpan::no_span(Expr::BinOp(
                            "!=",
                            Box::new(WithSpan::no_span(Expr::BinOp(
                                "==",
                                Box::new(WithSpan::no_span(Expr::Var("a"))),
                                Box::new(WithSpan::no_span(Expr::Var("b")))
                            ))),
                            Box::new(WithSpan::no_span(Expr::Var("c")))
                        ))),
                        Box::new(WithSpan::no_span(Expr::Var("d")))
                    ))),
                    Box::new(WithSpan::no_span(Expr::Var("e")))
                ))),
                Box::new(WithSpan::no_span(Expr::Var("f")))
            ))
        )],
    );
}

#[test]
fn test_odd_calls() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ a[b](c) }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Call(
                Box::new(WithSpan::no_span(Expr::Index(
                    Box::new(WithSpan::no_span(Expr::Var("a"))),
                    Box::new(WithSpan::no_span(Expr::Var("b")))
                ))),
                vec![WithSpan::no_span(Expr::Var("c"))],
            ),)
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (a + b)(c) }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Call(
                Box::new(WithSpan::no_span(Expr::Group(Box::new(WithSpan::no_span(
                    Expr::BinOp(
                        "+",
                        Box::new(WithSpan::no_span(Expr::Var("a"))),
                        Box::new(WithSpan::no_span(Expr::Var("b")))
                    )
                ))))),
                vec![WithSpan::no_span(Expr::Var("c"))],
            ),)
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a + b(c) }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "+",
                Box::new(WithSpan::no_span(Expr::Var("a"))),
                Box::new(WithSpan::no_span(Expr::Call(
                    Box::new(WithSpan::no_span(Expr::Var("b"))),
                    vec![WithSpan::no_span(Expr::Var("c"))]
                ))),
            )),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (-a)(b) }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Call(
                Box::new(WithSpan::no_span(Expr::Group(Box::new(WithSpan::no_span(
                    Expr::Unary("-", Box::new(WithSpan::no_span(Expr::Var("a"))))
                ))))),
                vec![WithSpan::no_span(Expr::Var("b"))],
            ),)
        )],
    );
    assert_eq!(
        Ast::from_str("{{ -a(b) }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Unary(
                "-",
                Box::new(WithSpan::no_span(Expr::Call(
                    Box::new(WithSpan::no_span(Expr::Var("a"))),
                    vec![WithSpan::no_span(Expr::Var("b"))]
                )))
            ),)
        )],
    );
    assert_eq!(
        Ast::from_str("{{ a(b)|c }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "c",
                arguments: vec![WithSpan::no_span(Expr::Call(
                    Box::new(WithSpan::no_span(Expr::Var("a"))),
                    vec![WithSpan::no_span(Expr::Var("b"))]
                ))]
            }),)
        )]
    );
    assert_eq!(
        Ast::from_str("{{ a(b)| c }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "c",
                arguments: vec![WithSpan::no_span(Expr::Call(
                    Box::new(WithSpan::no_span(Expr::Var("a"))),
                    vec![WithSpan::no_span(Expr::Var("b"))]
                ))]
            })),
        )]
    );
    assert_eq!(
        Ast::from_str("{{ a(b) |c }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "|",
                Box::new(WithSpan::no_span(Expr::Call(
                    Box::new(WithSpan::no_span(Expr::Var("a"))),
                    vec![WithSpan::no_span(Expr::Var("b"))]
                ))),
                Box::new(WithSpan::no_span(Expr::Var("c")))
            ),)
        )]
    );
}

#[test]
fn test_parse_comments() {
    fn one_comment_ws(source: &str, ws: Ws) {
        let s = &Syntax::default();
        let mut nodes = Ast::from_str(source, None, s).unwrap().nodes;
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
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ () }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Tuple(vec![]),)
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1) }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Group(Box::new(WithSpan::no_span(Expr::NumLit("1"))),))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1,) }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Tuple(vec![WithSpan::no_span(Expr::NumLit("1"))])),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1, ) }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Tuple(vec![WithSpan::no_span(Expr::NumLit("1"))])),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1 ,) }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Tuple(vec![WithSpan::no_span(Expr::NumLit("1"))])),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1 , ) }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Tuple(vec![WithSpan::no_span(Expr::NumLit("1"))])),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1, 2) }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Tuple(vec![
                WithSpan::no_span(Expr::NumLit("1")),
                WithSpan::no_span(Expr::NumLit("2"))
            ])),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1, 2,) }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Tuple(vec![
                WithSpan::no_span(Expr::NumLit("1")),
                WithSpan::no_span(Expr::NumLit("2"))
            ])),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1, 2, 3) }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Tuple(vec![
                WithSpan::no_span(Expr::NumLit("1")),
                WithSpan::no_span(Expr::NumLit("2")),
                WithSpan::no_span(Expr::NumLit("3"))
            ])),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ ()|abs }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "abs",
                arguments: vec![WithSpan::no_span(Expr::Tuple(vec![]))]
            })),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ () | abs }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "|",
                Box::new(WithSpan::no_span(Expr::Tuple(vec![]))),
                Box::new(WithSpan::no_span(Expr::Var("abs")))
            )),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1)|abs }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "abs",
                arguments: vec![WithSpan::no_span(Expr::Group(Box::new(WithSpan::no_span(
                    Expr::NumLit("1")
                ))))]
            })),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1) | abs }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "|",
                Box::new(WithSpan::no_span(Expr::Group(Box::new(WithSpan::no_span(
                    Expr::NumLit("1")
                ))))),
                Box::new(WithSpan::no_span(Expr::Var("abs")))
            )),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1,)|abs }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "abs",
                arguments: vec![WithSpan::no_span(Expr::Tuple(vec![WithSpan::no_span(
                    Expr::NumLit("1")
                )]))]
            })),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1,) | abs }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "|",
                Box::new(WithSpan::no_span(Expr::Tuple(vec![WithSpan::no_span(
                    Expr::NumLit("1")
                )]))),
                Box::new(WithSpan::no_span(Expr::Var("abs")))
            )),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1, 2)|abs }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "abs",
                arguments: vec![WithSpan::no_span(Expr::Tuple(vec![
                    WithSpan::no_span(Expr::NumLit("1")),
                    WithSpan::no_span(Expr::NumLit("2"))
                ]))]
            })),
        )],
    );
    assert_eq!(
        Ast::from_str("{{ (1, 2) | abs }}", None, &syntax)
            .unwrap()
            .nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "|",
                Box::new(WithSpan::no_span(Expr::Tuple(vec![
                    WithSpan::no_span(Expr::NumLit("1")),
                    WithSpan::no_span(Expr::NumLit("2"))
                ]))),
                Box::new(WithSpan::no_span(Expr::Var("abs")))
            )),
        )],
    );
}

#[test]
fn test_missing_space_after_kw() {
    let syntax = Syntax::default();
    let err = Ast::from_str("{%leta=b%}", None, &syntax).unwrap_err();
    assert!(matches!(
        &*err.to_string(),
        "failed to parse template source at row 1, column 0 near:\n\"{%leta=b%}\"",
    ));
}

#[test]
fn test_parse_array() {
    let syntax = Syntax::default();
    assert_eq!(
        Ast::from_str("{{ [] }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Array(vec![]))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [1] }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Array(vec![WithSpan::no_span(Expr::NumLit("1"))]))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [ 1] }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Array(vec![WithSpan::no_span(Expr::NumLit("1"))]))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [1 ] }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Array(vec![WithSpan::no_span(Expr::NumLit("1"))]))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [1,2] }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Array(vec![
                WithSpan::no_span(Expr::NumLit("1")),
                WithSpan::no_span(Expr::NumLit("2"))
            ]))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [1 ,2] }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Array(vec![
                WithSpan::no_span(Expr::NumLit("1")),
                WithSpan::no_span(Expr::NumLit("2"))
            ]))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [1, 2] }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Array(vec![
                WithSpan::no_span(Expr::NumLit("1")),
                WithSpan::no_span(Expr::NumLit("2"))
            ]))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [1,2 ] }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Array(vec![
                WithSpan::no_span(Expr::NumLit("1")),
                WithSpan::no_span(Expr::NumLit("2"))
            ]))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ []|foo }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "foo",
                arguments: vec![WithSpan::no_span(Expr::Array(vec![]))]
            }))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ []| foo }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::Filter(Filter {
                name: "foo",
                arguments: vec![WithSpan::no_span(Expr::Array(vec![]))]
            }))
        )],
    );
    assert_eq!(
        Ast::from_str("{{ [] |foo }}", None, &syntax).unwrap().nodes,
        vec![Node::Expr(
            Ws(None, None),
            WithSpan::no_span(Expr::BinOp(
                "|",
                Box::new(WithSpan::no_span(Expr::Array(vec![]))),
                Box::new(WithSpan::no_span(Expr::Var("foo")))
            )),
        )],
    );
}

#[test]
fn fuzzed_unicode_slice() {
    let d = "{eeuuu{b&{!!&{!!11{{
            0!(!1q҄א!)!!!!!!n!";
    assert!(Ast::from_str(d, None, &Syntax::default()).is_err());
}

#[test]
fn fuzzed_macro_no_end() {
    let s = "{%macro super%}{%endmacro";
    assert!(Ast::from_str(s, None, &Syntax::default()).is_err());
}

#[test]
fn fuzzed_target_recursion() {
    const TEMPLATE: &str = include_str!("../tests/target-recursion.txt");
    assert!(Ast::from_str(TEMPLATE, None, &Syntax::default()).is_err());
}

#[test]
fn fuzzed_unary_recursion() {
    const TEMPLATE: &str = include_str!("../tests/unary-recursion.txt");
    assert!(Ast::from_str(TEMPLATE, None, &Syntax::default()).is_err());
}

#[test]
fn fuzzed_comment_depth() {
    let (sender, receiver) = std::sync::mpsc::channel();
    let test = std::thread::spawn(move || {
        const TEMPLATE: &str = include_str!("../tests/comment-depth.txt");
        assert!(Ast::from_str(TEMPLATE, None, &Syntax::default()).is_ok());
        sender.send(()).unwrap();
    });
    receiver
        .recv_timeout(std::time::Duration::from_secs(3))
        .expect("timeout");
    test.join().unwrap();
}

#[test]
fn let_set() {
    assert_eq!(
        Ast::from_str("{% let a %}", None, &Syntax::default())
            .unwrap()
            .nodes(),
        Ast::from_str("{% set a %}", None, &Syntax::default())
            .unwrap()
            .nodes(),
    );
}
