use askama_parser::config::Syntax;
use askama_parser::parser::{Loop, Macro, Node, Whitespace, Ws};

pub fn ws_to_char(ws: &Whitespace) -> char {
    match ws {
        Whitespace::Preserve => '+',
        Whitespace::Suppress => '-',
        Whitespace::Minimize => '~',
    }
}

fn block_tag<F: FnOnce(&mut String)>(buf: &mut String, syn: &Syntax, ws: &Ws, f: F) {
    structured(buf, &syn.block_start, &syn.block_end, true, ws, f);
}

fn structured<F: FnOnce(&mut String)>(buf: &mut String, open: &str, close: &str, padding: bool, ws: &Ws, f: F) {
    buf.push_str(open);
    ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
    if padding { buf.push(' '); }
    f(buf);
    if padding { buf.push(' '); }
    ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
    buf.push_str(close);
}

pub fn fmt(ast: &[Node], syn: &Syntax) -> String {
    let mut buf = String::new();

    for node in ast {
        match node {
            Node::Lit(lws, val, rws) => {
                buf.push_str(lws);
                buf.push_str(val);
                buf.push_str(rws);
            }
            Node::Comment(ws, text) => structured(&mut buf, &syn.comment_start, &syn.comment_end, false, ws, |buf| buf.push_str(text)),
            Node::Expr(ws, expr) => structured(&mut buf, &syn.expr_start, &syn.expr_end, true, ws, |buf| expr_to_str(buf, expr)),
            Node::Call(ws, scope, name, args) => block_tag(&mut buf, syn, ws, |buf| {
                buf.push_str("call ");
                if let Some(scope) = scope {
                    buf.push_str(scope);
                    buf.push_str("::");
                }
                buf.push_str(name);
                buf.push('(');
                let mut first = true;
                for arg in args {
                    if first {
                        first = false;
                    } else {
                        buf.push_str(", ");
                    }

                    expr_to_str(buf, arg);
                }
                buf.push(')');
            }),
            Node::LetDecl(ws, target) => block_tag(&mut buf, syn, ws, |buf| {
                buf.push_str("let ");
                target_to_str(buf, target);
            }),
            Node::Let(ws, target, expr) => block_tag(&mut buf, syn, ws, |buf| {
                buf.push_str("let ");
                target_to_str(buf, target);
                buf.push_str(" = ");
                expr_to_str(buf, expr);
            }),
            Node::Cond(blocks, ws) => {
                let mut print_else = false;
                for (bws, cond, block) in blocks {
                    block_tag(&mut buf, syn, bws, |buf| {
                        if print_else {
                            buf.push_str("else");
                        }
                        if let Some(test) = cond {
                            if print_else {
                                buf.push(' ');
                            }
                            buf.push_str("if ");
                            if let Some(target) = &test.target {
                                buf.push_str("let ");
                                target_to_str(buf, target);
                                buf.push_str(" = ");
                            }
                            expr_to_str(buf, &test.expr);
                        }
                        if !print_else {
                            print_else = true;
                        }
                    });

                    buf.push_str(&fmt(block, syn));
                }
                block_tag(&mut buf, syn, ws, |buf| buf.push_str("endif"));
            }
            Node::Match(lws, expr, interstitial, blocks, rws) => {
                block_tag(&mut buf, syn, lws, |buf| {
                    buf.push_str("match ");
                    expr_to_str(buf, expr);
                });

                buf.push_str(&fmt(interstitial, syn));

                for (ws, target, block) in blocks {
                    block_tag(&mut buf, syn, ws, |buf| {
                        buf.push_str("when ");
                        target_to_str(buf, target);
                    });
                    buf.push_str(&fmt(block, syn));
                }

                block_tag(&mut buf, syn, rws, |buf| buf.push_str("endmatch"));
            }
            Node::Loop(Loop { ws1, var, iter, cond, body, ws2, else_block, ws3 }) => {
                block_tag(&mut buf, syn, ws1, |buf| {
                    buf.push_str("for ");
                    target_to_str(buf, var);
                    buf.push_str(" in ");
                    expr_to_str(buf, iter);

                    if let Some(cond) = cond {
                        buf.push_str(" if ");
                        expr_to_str(buf, cond);
                    }
                });

                buf.push_str(&fmt(body, syn));

                if !else_block.is_empty() {
                    block_tag(&mut buf, syn, ws2, |buf| { buf.push_str("else") });

                    buf.push_str(&fmt(else_block, syn));
                }

                block_tag(&mut buf, syn, ws3, |buf| { buf.push_str("endfor") });
            }
            Node::Extends(parent) => {
                let ws = &Ws(None, None);
                block_tag(&mut buf, syn, ws, |buf| {
                    buf.push_str("extends ");
                    strlit_to_str(buf, parent);
                });
            }
            Node::BlockDef(lws, name, body, rws) => {
                block_tag(&mut buf, syn, lws, |buf| {
                    buf.push_str("block ");
                    buf.push_str(name);
                });
                buf.push_str(&fmt(body, syn));
                block_tag(&mut buf, syn, rws, |buf| {
                    buf.push_str("endblock");
                });
            }
            Node::Include(ws, name) => {
                block_tag(&mut buf, syn, ws, |buf| {
                    buf.push_str("include ");
                    strlit_to_str(buf, name);
                });
            }
            Node::Import(ws, name, alias) => {
                block_tag(&mut buf, syn, ws, |buf| {
                    buf.push_str("import ");
                    strlit_to_str(buf, name);
                    buf.push_str(" as ");
                    buf.push_str(alias);
                });
            }
            Node::Macro(name, Macro { ws1, args, nodes, ws2 }) => {
                block_tag(&mut buf, syn, ws1, |buf| {
                    buf.push_str("macro ");
                    buf.push_str(name);
                    buf.push('(');
                    let mut first = true;
                    for arg in args {
                        if first {
                            first = false;
                        } else {
                            buf.push_str(", ");
                        }
                        buf.push_str(arg);
                    }
                    buf.push(')');
                });
                buf.push_str(&fmt(nodes, syn));
                block_tag(&mut buf, syn, ws2, |buf| {
                    buf.push_str("endmacro");
                });
            }
            Node::Raw(ws1, lws, val, rws, ws2) => {
                block_tag(&mut buf, syn, ws1, |buf| { buf.push_str("raw") });
                buf.push_str(lws);
                buf.push_str(val);
                buf.push_str(rws);
                block_tag(&mut buf, syn, ws2, |buf| { buf.push_str("endraw") });
            }
            Node::Break(ws) => {
                block_tag(&mut buf, syn, ws, |buf| { buf.push_str("break") });
            }
            Node::Continue(ws) => {
                block_tag(&mut buf, syn, ws, |buf| { buf.push_str("continue") });
            }
        }
    }

    buf
}

fn target_to_str(buf: &mut String, target: &askama_parser::parser::Target) {
    use askama_parser::parser::Target::*;
    match target {
        Name(name) => buf.push_str(name),
        Tuple(path, elements) => {
            if !path.is_empty() {
                buf.push_str(&path.join("::"));
                buf.push_str(" with ");
            }

            buf.push('(');

            let mut print_comma = false;
            for element in elements {
                if print_comma {
                    buf.push_str(", ");
                } else {
                    print_comma = true;
                }

                target_to_str(buf, element);
            }

            if elements.len() == 1 && path.is_empty() {
                buf.push(',');
            }

            buf.push(')');
        }
        Struct(path, fields) => {
            buf.push_str(&path.join("::"));
            buf.push_str(" with { ");
            let mut first = true;
            for field in fields {
                if first {
                    first = false;
                } else {
                    buf.push_str(", ");
                }
                buf.push_str(field.0);
                if let askama_parser::parser::Target::Name(n) = field.1 {
                    if n != field.0 {
                        buf.push_str(": ");
                        target_to_str(buf, &field.1);
                    }
                }
            }
            buf.push_str(" }");
        }
        NumLit(val) => {
            buf.push_str(val);
        }
        StrLit(val) => {
            buf.push('"');
            buf.push_str(val); // TODO determine escaping
            buf.push('"');
        }
        CharLit(val) => {
            buf.push('\'');
            buf.push_str(val);
            buf.push('\'');
        }
        BoolLit(val) => {
            buf.push_str(val);
        }
        Path(path) => buf.push_str(&path.join("::")),
    }
}

fn expr_to_str(buf: &mut String, expr: &askama_parser::parser::Expr) {
    use askama_parser::parser::Expr::*;
    match expr {
        BoolLit(s) | NumLit(s) | Var(s) => buf.push_str(s),
        StrLit(s) => {
            strlit_to_str(buf, s);
        }
        CharLit(s) => {
            buf.push_str("'");
            buf.push_str(s);
            buf.push_str("'");
        }
        Path(ss) => {
            buf.push_str(&ss.join("::"));
        }
        Array(exprs) => {
            buf.push('[');
            let mut first = true;
            for el in exprs {
                if first {
                    first = false;
                } else {
                    buf.push_str(", ");
                }
                expr_to_str(buf, el);
            }
            buf.push(']');
        }
        Attr(expr, field) => {
            expr_to_str(buf, expr);
            buf.push('.');
            buf.push_str(field);
        }
        Index(expr, idx) => {
            expr_to_str(buf, expr);
            buf.push('[');
            expr_to_str(buf, idx);
            buf.push(']');
        }
        Filter(name, args) => {
            assert!(!args.is_empty());
            expr_to_str(buf, &args[0]);
            buf.push('|');
            buf.push_str(name);
            if args.len() > 1 {
                buf.push('(');
                let mut first = true;
                for arg in args.iter().skip(1) {
                    if first {
                        first = false;
                    } else {
                        buf.push_str(", ");
                    }
                    expr_to_str(buf, arg);
                }
                buf.push(')');
            }
        }
        Unary(op, arg) => {
            buf.push_str(op);
            expr_to_str(buf, arg);
        }
        BinOp(op, lhs, rhs) => {
            expr_to_str(buf, lhs);
            buf.push(' ');
            buf.push_str(op);
            buf.push(' ');
            expr_to_str(buf, rhs);
        }
        Range(op, lhs, rhs) => {
            // TODO: Rust ranges are notorious for needing parens.... same here???
            if let Some(lhs) = lhs {
                expr_to_str(buf, lhs);
            }
            buf.push_str(op);
            if let Some(rhs) = rhs {
                expr_to_str(buf, rhs);
            }
        }
        Group(expr) => {
            buf.push('(');
            expr_to_str(buf, expr);
            buf.push(')');
        }
        Tuple(els) => {
            buf.push('(');
            let mut first = true;
            for el in els {
                if first {
                    first = false;
                } else {
                    buf.push_str(", ");
                }
                expr_to_str(buf, el);
            }

            if els.len() == 1 {
                buf.push(',');
            }

            buf.push(')');
        }
        Call(callee, args) => {
            expr_to_str(buf, callee);
            buf.push('(');
            let mut first = true;
            for arg in args {
                if first {
                    first = false;
                } else {
                    buf.push_str(", ");
                }
                expr_to_str(buf, arg);
            }
            buf.push(')');
        }
        RustMacro(name, input) => {
            buf.push_str(name);
            buf.push('!');
            buf.push('(');
            buf.push_str(input);
            buf.push(')');
        }
        Try(expr) => {
            expr_to_str(buf, expr);
            buf.push('?');
        }
    }
}

fn strlit_to_str(buf: &mut String, s: &str) {
    buf.push_str("\"");
    buf.push_str(s);
    buf.push_str("\"");
}

#[cfg(test)]
mod tests {
    use super::*;

    use askama_parser::config::Syntax;
    use askama_parser::parser::{parse, Expr, Target, Ws};

    fn custom() -> Syntax {
        Syntax {
            block_start: "<?".into(),
            block_end: "?>".into(),
            comment_start: "<!".into(),
            comment_end: "!>".into(),
            expr_start: "<:".into(),
            expr_end: ":>".into(),
        }
    }

    #[test]
    fn lit() {
        let syn = Syntax::default();
        let node = parse(" foobar\t", &syn).expect("PARSE");

        assert_eq!(" foobar\t", fmt(&node, &syn));
    }

    #[test]
    fn comment() {
        let syn = Syntax::default();
        let node = parse("foo{#+ empty -#}bar", &syn).expect("PARSE");

        assert_eq!("foo{#+ empty -#}bar", fmt(&node, &syn));
        assert_eq!("foo<!+ empty -!>bar", fmt(&node, &custom()));
    }

    #[test]
    fn expr() {
        let syn = Syntax::default();
        let node = parse("{{42}}", &syn).expect("PARSE");

        assert_eq!("{{ 42 }}", fmt(&node, &syn));
        assert_eq!("<: 42 :>", fmt(&node, &custom()));
    }

    fn test_target(expected: &str, target: Target) {
        let syn = Syntax::default();
        let node = Node::Let(Ws(None, None), target, Expr::Var("val"));

        let str1 = fmt(&[node], &syn);
        assert_eq!(str1, format!("{{% let {} = val %}}", expected));

        let parsed = parse(&str1, &syn).expect("PARSE");
        let str2 = fmt(&parsed, &syn);
        assert_eq!(str1, str2);
    }

    #[test] fn target_name() { test_target("foo", Target::Name("foo")); }
    #[test] fn target_tuple_unit() { test_target("()", Target::Tuple(vec![], vec![])); }
    #[test] fn target_tuple_anon() { test_target("(a,)", Target::Tuple(vec![], vec![Target::Name("a")])); }
    #[test] fn target_tuple_named() { test_target("Some with (val)", Target::Tuple(
        vec!["Some"],
        vec![Target::Name("val")],
    )); }
    #[test] fn target_struct() { test_target("Color with { r, g: lime, b }", Target::Struct(
        vec!["Color"],
        vec![("r", Target::Name("r")), ("g", Target::Name("lime")), ("b", Target::Name("b"))],
    )); }
    #[test] fn target_numlit() { test_target("42", Target::NumLit("42")); }
    #[test] fn target_strlit() { test_target("\"foo\\\"bar\"", Target::StrLit("foo\\\"bar")); }
    #[test] fn target_charlit() { test_target("'.'", Target::CharLit(".")); }
    #[test] fn target_boollit() { test_target("false", Target::BoolLit("false")); }
    #[test] fn target_path() { test_target("foo::bar", Target::Path(vec!["foo", "bar"])); }

    fn test_expr(expected: &str, expr: Expr) {
        let syn = Syntax::default();
        let node = Node::Expr(Ws(None, None), expr);

        let str1 = fmt(&[node], &syn);
        assert_eq!(str1, format!("{{{{ {} }}}}", expected));

        let parsed = parse(&str1, &syn).expect("PARSE");
        let str2 = fmt(&parsed, &syn);
        assert_eq!(str1, str2);
    }

    #[test] fn expr_bool_lit() { test_expr("true", Expr::BoolLit("true")); }
    #[test] fn expr_num_lit() { test_expr("42", Expr::NumLit("42")); }
    #[test] fn expr_str_lit() { test_expr("\"foo\\\"bar\"", Expr::StrLit("foo\\\"bar")); }
    #[test] fn expr_char_lit() { test_expr("'c'", Expr::CharLit("c")); }
    #[test] fn expr_var() { test_expr("value", Expr::Var("value")); }
    #[test] fn expr_path() { test_expr("askama::Template", Expr::Path(vec!["askama", "Template"])); }
    #[test] fn expr_array() { test_expr("[1, 2]", Expr::Array(vec![
        Expr::NumLit("1"),
        Expr::NumLit("2"),
    ])); }
    #[test] fn expr_attr() { test_expr("obj.field", Expr::Attr(Box::new(Expr::Var("obj")), "field")); }
    #[test] fn expr_index() { test_expr("arr[idx]", Expr::Index(
        Box::new(Expr::Var("arr")),
        Box::new(Expr::Var("idx")),
    )); }
    #[test] fn expr_filter() { test_expr("input|filter(\"arg\")", Expr::Filter("filter", vec![
        Expr::Var("input"),
        Expr::StrLit("arg"),
    ])); }
    #[test] fn expr_unary() { test_expr("-42", Expr::Unary("-", Box::new(Expr::NumLit("42")))); }
    #[test] fn expr_binop() { test_expr("1 + 2", Expr::BinOp(
        "+",
        Box::new(Expr::NumLit("1")),
        Box::new(Expr::NumLit("2")),
    )); }
    #[test] fn expr_range_oo() { test_expr("..", Expr::Range("..", None, None)); }
    #[test] fn expr_range_co() { test_expr("1..", Expr::Range("..", Some(Box::new(Expr::NumLit("1"))), None)); }
    #[test] fn expr_range_oc() { test_expr("..1", Expr::Range("..", None, Some(Box::new(Expr::NumLit("1"))))); }
    #[test] fn expr_range_right() { test_expr("..=1", Expr::Range("..=", None, Some(Box::new(Expr::NumLit("1"))))); }
    #[test] fn expr_group() { test_expr("(var)", Expr::Group(Box::new(Expr::Var("var")))); }
    #[test] fn expr_tuple_one() { test_expr("(var,)", Expr::Tuple(vec![Expr::Var("var")])); }
    #[test] fn expr_tuple_two() { test_expr("(a, b)", Expr::Tuple(vec![
        Expr::Var("a"),
        Expr::Var("b"),
    ])); }
    #[test] fn expr_call() { test_expr("foo(bar, baz)", Expr::Call(
        Box::new(Expr::Var("foo")),
        vec![
            Expr::Var("bar"),
            Expr::Var("baz"),
        ],
    )); }
    #[test] fn rust_macro() { test_expr("do!(+#15 I$ 4@3)", Expr::RustMacro("do", "+#15 I$ 4@3")); }
    #[test] fn try_() { test_expr("maybe?", Expr::Try(Box::new(Expr::Var("maybe")))); }

    #[test]
    fn call() {
        let syn = Syntax::default();
        let node = parse("{% call scope::macro(1, 2, 3) %}", &syn).expect("PARSE");

        assert_eq!("{% call scope::macro(1, 2, 3) %}", fmt(&node, &syn));
        assert_eq!("<? call scope::macro(1, 2, 3) ?>", fmt(&node, &custom()));
    }

    #[test]
    fn let_decl() {
        let syn = Syntax::default();
        let node = parse("{%let foo\t%}", &syn).expect("PARSE");

        assert_eq!("{% let foo %}", fmt(&node, &syn));
        assert_eq!("<? let foo ?>", fmt(&node, &custom()));
    }

    #[test]
    fn let_() {
        let syn = Syntax::default();
        let node = parse("{%let foo\t=\n42%}", &syn).expect("PARSE");

        assert_eq!("{% let foo = 42 %}", fmt(&node, &syn));
        assert_eq!("<? let foo = 42 ?>", fmt(&node, &custom()));
    }

    #[test]
    fn cond() {
        let syn = Syntax::default();
        let node = parse("{%if foo-%}bar{%-else\t-%}baz{%- endif\n%}", &syn).expect("PARSE");

        assert_eq!("{% if foo -%}bar{%- else -%}baz{%- endif %}", fmt(&node, &syn));
        assert_eq!("<? if foo -?>bar<?- else -?>baz<?- endif ?>", fmt(&node, &custom()));
    }

    #[test]
    fn match_() {
        let syn = Syntax::default();
        let node = parse("{%match item-%}
  {%  when Some
  with\t (\t \"foo\"  )\t-%}
    Found literal foo
  {% when Some with (val) -%}
    Found {{ val }}
  {% when None -%}
{% endmatch\n%}", &syn).expect("PARSE");

        assert_eq!("{% match item -%}
  {% when Some with (\"foo\") -%}
    Found literal foo
  {% when Some with (val) -%}
    Found {{ val }}
  {% when None -%}
{% endmatch %}", fmt(&node, &syn));
        assert_eq!("<? match item -?>
  <? when Some with (\"foo\") -?>
    Found literal foo
  <? when Some with (val) -?>
    Found <: val :>
  <? when None -?>
<? endmatch ?>", fmt(&node, &custom()));
    }

    #[test]
    fn loop_() {
        let syn = Syntax::default();
        let node = parse("{%for value in values-%}{{\tvalue\n}}{%endfor~%}", &syn).expect("PARSE");

        assert_eq!("{% for value in values -%}{{ value }}{% endfor ~%}", fmt(&node, &syn));
        assert_eq!("<? for value in values -?><: value :><? endfor ~?>", fmt(&node, &custom()));
    }

    #[test]
    fn loop_cond() {
        let syn = Syntax::default();
        let node = parse("{%for value in values if true-%}{{\tvalue\n}}{%endfor~%}", &syn).expect("PARSE");

        assert_eq!("{% for value in values if true -%}{{ value }}{% endfor ~%}", fmt(&node, &syn));
        assert_eq!("<? for value in values if true -?><: value :><? endfor ~?>", fmt(&node, &custom()));
    }

    #[test]
    fn loop_else() {
        let syn = Syntax::default();
        let node = parse("{%for value in values-%}{{\tvalue\n}}{%else%}NONE{%endfor~%}", &syn).expect("PARSE");

        assert_eq!("{% for value in values -%}{{ value }}{% else %}NONE{% endfor ~%}", fmt(&node, &syn));
        assert_eq!("<? for value in values -?><: value :><? else ?>NONE<? endfor ~?>", fmt(&node, &custom()));
    }

    #[test]
    fn extends() {
        let syn = Syntax::default();
        let node = parse("{%extends \"base.html\"\t%}", &syn).expect("PARSE");

        assert_eq!("{% extends \"base.html\" %}", fmt(&node, &syn));
        assert_eq!("<? extends \"base.html\" ?>", fmt(&node, &custom()));
    }

    #[test]
    fn block_def() {
        let syn = Syntax::default();
        let node = parse("{%block title\t%}Hi!{%endblock%}", &syn).expect("PARSE");

        assert_eq!("{% block title %}Hi!{% endblock %}", fmt(&node, &syn));
        assert_eq!("<? block title ?>Hi!<? endblock ?>", fmt(&node, &custom()));
    }

    #[test]
    fn include() {
        let syn = Syntax::default();
        let node = parse("{%include \"item.html\"\t%}", &syn).expect("PARSE");

        assert_eq!("{% include \"item.html\" %}", fmt(&node, &syn));
        assert_eq!("<? include \"item.html\" ?>", fmt(&node, &custom()));
    }

    #[test]
    fn import() {
        let syn = Syntax::default();
        let node = parse("{%import \"macros.html\" as mod\t%}", &syn).expect("PARSE");

        assert_eq!("{% import \"macros.html\" as mod %}", fmt(&node, &syn));
        assert_eq!("<? import \"macros.html\" as mod ?>", fmt(&node, &custom()));
    }

    #[test]
    fn macro_() {
        let syn = Syntax::default();
        let node = parse("{%macro heading(arg)\t%}<h1>{{arg}}</h1>{%endmacro%}", &syn).expect("PARSE");

        assert_eq!("{% macro heading(arg) %}<h1>{{ arg }}</h1>{% endmacro %}", fmt(&node, &syn));
        assert_eq!("<? macro heading(arg) ?><h1><: arg :></h1><? endmacro ?>", fmt(&node, &custom()));
    }

    #[test]
    fn raw() {
        let syn = Syntax::default();
        let node = parse("{%raw\t%}\n{{\twhat}}{%endraw%}", &syn).expect("PARSE");

        assert_eq!("{% raw %}\n{{\twhat}}{% endraw %}", fmt(&node, &syn));
        assert_eq!("<? raw ?>\n{{\twhat}}<? endraw ?>", fmt(&node, &custom()));
    }

    #[test]
    fn break_() {
        let syn = Syntax::default();
        let node = parse("{%for value in values-%}{%\tbreak\n%}{%endfor~%}", &syn).expect("PARSE");

        assert_eq!("{% for value in values -%}{% break %}{% endfor ~%}", fmt(&node, &syn));
        assert_eq!("<? for value in values -?><? break ?><? endfor ~?>", fmt(&node, &custom()));
    }

    #[test]
    fn continue_() {
        let syn = Syntax::default();
        let node = parse("{%for value in values-%}{%\tcontinue\n%}{%endfor~%}", &syn).expect("PARSE");

        assert_eq!("{% for value in values -%}{% continue %}{% endfor ~%}", fmt(&node, &syn));
        assert_eq!("<? for value in values -?><? continue ?><? endfor ~?>", fmt(&node, &custom()));
    }
}
