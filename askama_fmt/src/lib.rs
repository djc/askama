use askama_parser::CompileError;
use askama_parser::parser::{Node, Whitespace};

pub fn ws_to_char(ws: &Whitespace) -> char {
    match ws {
        Whitespace::Preserve => '+',
        Whitespace::Suppress => '-',
        Whitespace::Minimize => '~',
    }
}

pub fn fmt(ast: &[Node]) -> Result<String, CompileError> { // TODO: need result?????
    let mut buf = String::new();

    for node in ast {
        match node {
            Node::Lit(lws, val, rws) => {
                buf.push_str(lws);
                buf.push_str(val);
                buf.push_str(rws);
            }
            Node::Comment(ws, text) => {
                buf.push_str("{#");
                ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(text);
                ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str("#}");
            }
            Node::Expr(ws, expr) => {
                buf.push_str("{{");
                ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push(' ');
                expr_to_str(&mut buf, expr);
                buf.push(' ');
                ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str("}}");
            }
            // TODO: Node::Call
            Node::LetDecl(ws, target) => {
                buf.push_str("{%");
                ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(" let ");
                target_to_str(&mut buf, target);
                buf.push(' ');
                ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str("%}");
            }
            Node::Let(ws, target, expr) => {
                buf.push_str("{%");
                ws.0.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str(" let ");
                target_to_str(&mut buf, target);
                buf.push_str(" = ");
                expr_to_str(&mut buf, expr);
                buf.push(' ');
                ws.1.iter().map(ws_to_char).for_each(|c| buf.push(c));
                buf.push_str("%}");
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
        _ => panic!("unsupported target!"),
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

    #[test]
    fn lit() {
        let syn = Syntax::default();
        let node = parse(" foobar\t", &syn).expect("PARSE");

        let result = fmt(&node).expect("EVAL");

        assert_eq!(" foobar\t", result);
    }

    #[test]
    fn comment() {
        let syn = Syntax::default();
        let node = parse("foo{#+ empty -#}bar", &syn).expect("PARSE");

        let result = fmt(&node).expect("EVAL");

        assert_eq!("foo{#+ empty -#}bar", result);
    }

    #[test]
    fn expr() {
        let syn = Syntax::default();
        let node = parse("{{42}}", &syn).expect("PARSE");

        let result = fmt(&node).expect("EVAL");

        assert_eq!("{{ 42 }}", result);
    }

    fn test_expr(expr: Expr) {
        let node = Node::Expr(Ws(None, None), expr);

        let str1 = fmt(&[node]).expect("FMT1");
        let syn = Syntax::default();
        let parsed = parse(&str1, &syn).expect("PARSE");
        let str2 = fmt(&parsed).expect("FMT1");
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

        let result = fmt(&node).expect("EVAL");

        assert_eq!("{% let foo %}", result);
    }

    #[test]
    fn let_() {
        let syn = Syntax::default();
        let node = parse("{%let foo\t=\n42%}", &syn).expect("PARSE");

        let result = fmt(&node).expect("EVAL");

        assert_eq!("{% let foo = 42 %}", result);
    }
}
