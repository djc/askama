use askama_parser::CompileError;
use askama_parser::config::Syntax;
use askama_parser::parser::{Loop, Node, Whitespace};

pub fn ws_to_char(ws: &Whitespace) -> char {
    match ws {
        Whitespace::Preserve => '+',
        Whitespace::Suppress => '-',
        Whitespace::Minimize => '~',
    }
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
            Node::Comment(ws, text) => {
                buf.push_str(&syn.comment_start);
                ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(text);
                ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(&syn.comment_end);
            }
            Node::Expr(ws, expr) => {
                buf.push_str(&syn.expr_start);
                ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push(' ');
                expr_to_str(&mut buf, expr);
                buf.push(' ');
                ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(&syn.expr_end);
            }
            // TODO: Node::Call
            Node::LetDecl(ws, target) => {
                buf.push_str(&syn.block_start);
                ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(" let ");
                target_to_str(&mut buf, target);
                buf.push(' ');
                ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(&syn.block_end);
            }
            Node::Let(ws, target, expr) => {
                buf.push_str(&syn.block_start);
                ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(" let ");
                target_to_str(&mut buf, target);
                buf.push_str(" = ");
                expr_to_str(&mut buf, expr);
                buf.push(' ');
                ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(&syn.block_end);
            }
            Node::Cond(blocks, ws) => {
                let mut print_else = false;
                for (bws, cond, block) in blocks {
                    buf.push_str(&syn.block_start);
                    bws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                    if print_else {
                        buf.push_str(" else");
                    } else {
                        print_else = true;
                    }
                    buf.push(' ');
                    if let Some(test) = cond {
                        buf.push_str("if ");
                        if let Some(target) = &test.target {
                            buf.push_str("let ");
                            target_to_str(&mut buf, target);
                            buf.push_str(" = ");
                        }
                        expr_to_str(&mut buf, &test.expr);
                        buf.push(' ');
                    }
                    bws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                    buf.push_str(&syn.block_end);

                    // check the ws allows for this
                    //buf.push_str("\n  ");

                    buf.push_str(&fmt(block, syn)?);

                    // check the ws allows for this
                    //buf.push_str("\n");
                }
                buf.push_str(&syn.block_start);
                ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(" endif ");
                ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(&syn.block_end);
            }
            Node::Match(lws, expr, interstitial, blocks, rws) => {
                buf.push_str(&syn.block_start);
                lws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(" match ");
                expr_to_str(&mut buf, expr);
                buf.push(' ');
                lws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(&syn.block_end);

                buf.push_str(&fmt(interstitial, syn)?);

                for (ws, target, block) in blocks {
                    buf.push_str(&syn.block_start);
                    ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                    buf.push_str(" when ");
                    target_to_str(&mut buf, target);
                    buf.push(' ');
                    ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                    buf.push_str(&syn.block_end);
                    buf.push_str(&fmt(block, syn)?);
                }

                buf.push_str(&syn.block_start);
                rws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(" endmatch ");
                rws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(&syn.block_end);
            }
            Node::Loop(Loop { ws1, var, iter, cond, body, ws2, else_block, ws3 }) => {
                buf.push_str(&syn.block_start);
                ws1.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(" for ");
                target_to_str(&mut buf, var);
                buf.push_str(" in ");
                expr_to_str(&mut buf, iter);

                if let Some(cond) = cond {
                    buf.push_str(" if ");
                    expr_to_str(&mut buf, cond);
                }

                buf.push(' ');
                ws1.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(&syn.block_end);

                buf.push_str(&fmt(body, syn)?);

                if !else_block.is_empty() {
                    buf.push_str(&syn.block_start);
                    ws2.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                    buf.push_str(" else ");
                    ws2.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                    buf.push_str(&syn.block_end);

                    buf.push_str(&fmt(else_block, syn)?);
                }

                buf.push_str(&syn.block_start);
                ws3.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(" endfor ");
                ws3.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(&syn.block_end);
            }
            Node::Break(ws) => {
                buf.push_str(&syn.block_start);
                ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(" break ");
                ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(&syn.block_end);
            }
            Node::Continue(ws) => {
                buf.push_str(&syn.block_start);
                ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(" continue ");
                ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(&syn.block_end);
            }
            _ => panic!("boo"),
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
