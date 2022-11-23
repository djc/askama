use askama_parser::CompileError;
use askama_parser::config::Syntax;
use askama_parser::parser::{Loop, Node, Whitespace, Ws};

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

pub fn fmt(ast: &[Node], syn: &Syntax) -> Result<String, CompileError> { // TODO: need result?????
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

                    buf.push_str(&fmt(block, syn)?);
                }
                block_tag(&mut buf, syn, ws, |buf| buf.push_str("endif"));
            }
            Node::Match(lws, expr, interstitial, blocks, rws) => {
                block_tag(&mut buf, syn, lws, |buf| {
                    buf.push_str("match ");
                    expr_to_str(buf, expr);
                });

                buf.push_str(&fmt(interstitial, syn)?);

                for (ws, target, block) in blocks {
                    block_tag(&mut buf, syn, ws, |buf| {
                        buf.push_str("when ");
                        target_to_str(buf, target);
                    });
                    buf.push_str(&fmt(block, syn)?);
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

                buf.push_str(&fmt(body, syn)?);

                if !else_block.is_empty() {
                    block_tag(&mut buf, syn, ws2, |buf| { buf.push_str("else") });

                    buf.push_str(&fmt(else_block, syn)?);
                }

                block_tag(&mut buf, syn, ws3, |buf| { buf.push_str("endfor") });
            }
            Node::Extends(parent) => {
                let ws = &Ws(None, None);
                block_tag(&mut buf, syn, ws, |buf| {
                    buf.push_str("extends ");
                    expr_to_str(buf, parent);
                });
            }
            Node::BlockDef(lws, name, body, rws) => {
                block_tag(&mut buf, syn, lws, |buf| {
                    buf.push_str("block ");
                    buf.push_str(name);
                });
                buf.push_str(&fmt(body, syn)?);
                block_tag(&mut buf, syn, lws, |buf| {
                    buf.push_str("endblock");
                });
            }
            Node::Break(ws) => {
                block_tag(&mut buf, syn, ws, |buf| { buf.push_str("break") });
            }
            Node::Continue(ws) => {
                block_tag(&mut buf, syn, ws, |buf| { buf.push_str("continue") });
            }
            _ => panic!("unhandled node type! {:?}", node),
        }
    }

    Ok(buf)
}

fn target_to_str(buf: &mut String, target: &askama_parser::parser::Target) {
    use askama_parser::parser::Target::*;
    match target {
        Name(name) => buf.push_str(name),
        Tuple(path, elements) => {
            buf.push_str(&path.join("::"));
            buf.push_str(" with (");

            let mut print_comma = false;
            for element in elements {
                if print_comma {
                    buf.push_str(", ");
                } else {
                    print_comma = true;
                }

                target_to_str(buf, element);
            }

            buf.push(')');
        }
        StrLit(val) => {
            buf.push('"');
            buf.push_str(val); // TODO determine escaping
            buf.push('"');
        }
        Path(path) => buf.push_str(&path.join("::")),
        _ => panic!("unsupported target {:?}!", target),
    }
}

fn expr_to_str(buf: &mut String, expr: &askama_parser::parser::Expr) {
    use askama_parser::parser::Expr::*;
    match expr {
        BoolLit(s) | NumLit(s) | Var(s) => buf.push_str(s),
        StrLit(s) => {
            buf.push_str("\"");
            buf.push_str(s);
            buf.push_str("\"");
        }
        CharLit(s) => {
            buf.push_str("'");
            buf.push_str(s);
            buf.push_str("'");
        }
        Path(ss) => {
            buf.push_str(&ss.join("::"));
        }
        _ => panic!("unsupported expr!"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use askama_parser::config::Syntax;
    use askama_parser::parser::{parse, Expr, Ws};

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

        assert_eq!(" foobar\t", fmt(&node, &syn).expect("FMT"));
    }

    #[test]
    fn comment() {
        let syn = Syntax::default();
        let node = parse("foo{#+ empty -#}bar", &syn).expect("PARSE");

        assert_eq!("foo{#+ empty -#}bar", fmt(&node, &syn).expect("FMT"));
        assert_eq!("foo<!+ empty -!>bar", fmt(&node, &custom()).expect("FMT"));
    }

    #[test]
    fn expr() {
        let syn = Syntax::default();
        let node = parse("{{42}}", &syn).expect("PARSE");

        assert_eq!("{{ 42 }}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<: 42 :>", fmt(&node, &custom()).expect("FMT"));
    }

    fn test_expr(expr: Expr) {
        let syn = Syntax::default();
        let node = Node::Expr(Ws(None, None), expr);

        let str1 = fmt(&[node], &syn).expect("FMT1");
        let parsed = parse(&str1, &syn).expect("PARSE");
        let str2 = fmt(&parsed, &syn).expect("FMT1");
        assert_eq!(str1, str2);
    }

    #[test] fn expr_bool_lit() { test_expr(Expr::BoolLit("true")); }
    #[test] fn expr_num_lit() { test_expr(Expr::NumLit("42")); }
    #[test] fn expr_str_lit() { test_expr(Expr::StrLit("foo\\\"bar")); }
    #[test] fn expr_char_lit() { test_expr(Expr::CharLit("c")); }
    #[test] fn expr_var() { test_expr(Expr::Var("value")); }
    #[test] fn expr_path() { test_expr(Expr::Path(vec!["askama", "Template"])); }

    #[test]
    fn call() {
        let syn = Syntax::default();
        let node = parse("{% call scope::macro(1, 2, 3) %}", &syn).expect("PARSE");

        assert_eq!("{% call scope::macro(1, 2, 3) %}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? call scope::macro(1, 2, 3) ?>", fmt(&node, &custom()).expect("FMT"));
    }

    #[test]
    fn let_decl() {
        let syn = Syntax::default();
        let node = parse("{%let foo\t%}", &syn).expect("PARSE");

        assert_eq!("{% let foo %}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? let foo ?>", fmt(&node, &custom()).expect("FMT"));
    }

    #[test]
    fn let_() {
        let syn = Syntax::default();
        let node = parse("{%let foo\t=\n42%}", &syn).expect("PARSE");

        assert_eq!("{% let foo = 42 %}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? let foo = 42 ?>", fmt(&node, &custom()).expect("FMT"));
    }

    #[test]
    fn cond() {
        let syn = Syntax::default();
        let node = parse("{%if foo-%}bar{%-else\t-%}baz{%- endif\n%}", &syn).expect("PARSE");

        assert_eq!("{% if foo -%}bar{%- else -%}baz{%- endif %}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? if foo -?>bar<?- else -?>baz<?- endif ?>", fmt(&node, &custom()).expect("FMT"));
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
{% endmatch %}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? match item -?>
  <? when Some with (\"foo\") -?>
    Found literal foo
  <? when Some with (val) -?>
    Found <: val :>
  <? when None -?>
<? endmatch ?>", fmt(&node, &custom()).expect("FMT"));
    }

    #[test]
    fn loop_() {
        let syn = Syntax::default();
        let node = parse("{%for value in values-%}{{\tvalue\n}}{%endfor~%}", &syn).expect("PARSE");

        assert_eq!("{% for value in values -%}{{ value }}{% endfor ~%}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? for value in values -?><: value :><? endfor ~?>", fmt(&node, &custom()).expect("FMT"));
    }

    #[test]
    fn loop_cond() {
        let syn = Syntax::default();
        let node = parse("{%for value in values if true-%}{{\tvalue\n}}{%endfor~%}", &syn).expect("PARSE");

        assert_eq!("{% for value in values if true -%}{{ value }}{% endfor ~%}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? for value in values if true -?><: value :><? endfor ~?>", fmt(&node, &custom()).expect("FMT"));
    }

    #[test]
    fn loop_else() {
        let syn = Syntax::default();
        let node = parse("{%for value in values-%}{{\tvalue\n}}{%else%}NONE{%endfor~%}", &syn).expect("PARSE");

        assert_eq!("{% for value in values -%}{{ value }}{% else %}NONE{% endfor ~%}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? for value in values -?><: value :><? else ?>NONE<? endfor ~?>", fmt(&node, &custom()).expect("FMT"));
    }

    #[test]
    fn extends() {
        let syn = Syntax::default();
        let node = parse("{%extends \"base.html\"\t%}", &syn).expect("PARSE");

        assert_eq!("{% extends \"base.html\" %}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? extends \"base.html\" ?>", fmt(&node, &custom()).expect("FMT"));
    }

    #[test]
    fn block_def() {
        let syn = Syntax::default();
        let node = parse("{%block title\t%}Hi!{%endblock%}", &syn).expect("PARSE");

        assert_eq!("{% block title %}Hi!{% endblock %}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? block title ?>Hi!<? endblock ?>", fmt(&node, &custom()).expect("FMT"));
    }

    #[test]
    fn break_() {
        let syn = Syntax::default();
        let node = parse("{%for value in values-%}{%\tbreak\n%}{%endfor~%}", &syn).expect("PARSE");

        assert_eq!("{% for value in values -%}{% break %}{% endfor ~%}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? for value in values -?><? break ?><? endfor ~?>", fmt(&node, &custom()).expect("FMT"));
    }

    #[test]
    fn continue_() {
        let syn = Syntax::default();
        let node = parse("{%for value in values-%}{%\tcontinue\n%}{%endfor~%}", &syn).expect("PARSE");

        assert_eq!("{% for value in values -%}{% continue %}{% endfor ~%}", fmt(&node, &syn).expect("FMT"));
        assert_eq!("<? for value in values -?><? continue ?><? endfor ~?>", fmt(&node, &custom()).expect("FMT"));
    }
}
