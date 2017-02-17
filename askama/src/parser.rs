use nom::{self, alphanumeric, IResult};
use std::str;

#[derive(Debug)]
pub enum Expr<'a> {
    NumLit(&'a str),
    StrLit(&'a str),
    Var(&'a str),
    Filter(&'a str, Box<Expr<'a>>),
    BinOp(&'a str, Box<Expr<'a>>, Box<Expr<'a>>),
}

#[derive(Debug)]
pub enum Target<'a> {
    Name(&'a str),
}

#[derive(Clone, Copy, Debug)]
pub struct WS(pub bool, pub bool);

#[derive(Debug)]
pub enum Node<'a> {
    Lit(&'a str, &'a str, &'a str),
    Expr(WS, Expr<'a>),
    Cond(Vec<(WS, Option<Expr<'a>>, Vec<Node<'a>>)>, WS),
    Loop(WS, Target<'a>, Expr<'a>, Vec<Node<'a>>, WS),
    Extends(Expr<'a>),
    BlockDef(WS, &'a str, Vec<Node<'a>>, WS),
    Block(WS, &'a str, WS),
}

pub type Cond<'a> = (WS, Option<Expr<'a>>, Vec<Node<'a>>);

fn split_ws_parts(s: &[u8]) -> Node {
    if s.is_empty() {
        let rs = str::from_utf8(s).unwrap();
        return Node::Lit(rs, rs, rs);
    }
    let is_ws = |c: &u8| {
        *c != b' ' && *c != b'\t' && *c != b'\r' && *c != b'\n'
    };
    let start = s.iter().position(&is_ws);
    let res = if start.is_none() {
            (s, &s[0..0], &s[0..0])
        } else {
            let start = start.unwrap();
            let end = s.iter().rposition(&is_ws);
            if end.is_none() {
                (&s[..start], &s[start..], &s[0..0])
            } else {
                let end = end.unwrap();
                (&s[..start], &s[start..end + 1], &s[end + 1..])
            }
        };
    Node::Lit(str::from_utf8(res.0).unwrap(),
              str::from_utf8(res.1).unwrap(),
              str::from_utf8(res.2).unwrap())
}

fn take_content(i: &[u8]) -> IResult<&[u8], Node> {
    if i.len() < 1 || i[0] == b'{' {
        return IResult::Error(error_position!(nom::ErrorKind::TakeUntil, i));
    }
    for (j, c) in i.iter().enumerate() {
        if *c == b'{' {
            if i.len() < j + 2 {
                return IResult::Done(&i[..0], split_ws_parts(&i[..]));
            } else if i[j + 1] == b'{' || i[j + 1] == b'%' {
                return IResult::Done(&i[j..], split_ws_parts(&i[..j]));
            }
        }
    }
    IResult::Done(&i[..0], split_ws_parts(&i[..]))
}

named!(expr_num_lit<Expr>, map!(nom::digit,
    |s| Expr::NumLit(str::from_utf8(s).unwrap())
));

named!(expr_str_lit<Expr>, map!(
    delimited!(char!('"'), is_not!("\""), char!('"')),
    |s| Expr::StrLit(str::from_utf8(s).unwrap())
));

named!(expr_var<Expr>, map!(alphanumeric,
    |s| Expr::Var(str::from_utf8(s).unwrap())
));

named!(target_single<Target>, map!(alphanumeric,
    |s| Target::Name(str::from_utf8(s).unwrap())
));

named!(expr_single<Expr>, alt!(
    expr_num_lit |
    expr_str_lit |
    expr_var
));

named!(filter, do_parse!(
    tag_s!("|") >>
    fname: alphanumeric >>
    (fname)
));

named!(expr_filtered<Expr>, do_parse!(
    obj: expr_single >>
    filters: many0!(filter) >>
    ({
       let mut res = obj;
       for f in filters {
           let fname = str::from_utf8(f).unwrap();
           res = Expr::Filter(fname, Box::new(res));
       }
       res
    })
));

macro_rules! expr_prec_layer {
    ( $name:ident, $inner:ident, $( $op:expr ),* ) => {
        named!($name<Expr>, alt!(
            do_parse!(
                left: $inner >>
                op: ws!(alt!($( tag_s!($op) )|*)) >>
                right: $inner >>
                (Expr::BinOp(str::from_utf8(op).unwrap(),
                             Box::new(left), Box::new(right)))
            ) | $inner
        ));
    }
}

expr_prec_layer!(expr_muldivmod, expr_filtered, "*", "/", "%");
expr_prec_layer!(expr_addsub, expr_muldivmod, "+", "-");
expr_prec_layer!(expr_shifts, expr_addsub, ">>", "<<");
expr_prec_layer!(expr_band, expr_shifts, "&");
expr_prec_layer!(expr_bxor, expr_band, "^");
expr_prec_layer!(expr_bor, expr_bxor, "|");
expr_prec_layer!(expr_compare, expr_bor,
    "==", "!=", ">=", ">", "<=", "<"
);
expr_prec_layer!(expr_and, expr_compare, "&&");
expr_prec_layer!(expr_any, expr_and, "||");

named!(expr_node<Node>, do_parse!(
    tag_s!("{{") >>
    pws: opt!(tag_s!("-")) >>
    expr: ws!(expr_any) >>
    nws: opt!(tag_s!("-")) >>
    tag_s!("}}") >>
    (Node::Expr(WS(pws.is_some(), nws.is_some()), expr))
));

named!(cond_if<Expr>, do_parse!(
    ws!(tag_s!("if")) >>
    cond: ws!(expr_any) >>
    (cond)
));

named!(cond_block<Cond>, do_parse!(
    tag_s!("{%") >>
    pws: opt!(tag_s!("-")) >>
    ws!(tag_s!("else")) >>
    cond: opt!(cond_if) >>
    nws: opt!(tag_s!("-")) >>
    tag_s!("%}") >>
    block: parse_template >>
    (WS(pws.is_some(), nws.is_some()), cond, block)
));

named!(block_if<Node>, do_parse!(
    tag_s!("{%") >>
    pws1: opt!(tag_s!("-")) >>
    cond: ws!(cond_if) >>
    nws1: opt!(tag_s!("-")) >>
    tag_s!("%}") >>
    block: parse_template >>
    elifs: many0!(cond_block) >>
    tag_s!("{%") >>
    pws2: opt!(tag_s!("-")) >>
    ws!(tag_s!("endif")) >>
    nws2: opt!(tag_s!("-")) >>
    tag_s!("%}") >>
    ({
       let mut res = Vec::new();
       res.push((WS(pws1.is_some(), nws1.is_some()), Some(cond), block));
       res.extend(elifs);
       Node::Cond(res, WS(pws2.is_some(), nws2.is_some()))
    })
));

named!(block_for<Node>, do_parse!(
    tag_s!("{%") >>
    pws1: opt!(tag_s!("-")) >>
    ws!(tag_s!("for")) >>
    var: ws!(target_single) >>
    ws!(tag_s!("in")) >>
    iter: ws!(expr_any) >>
    nws1: opt!(tag_s!("-")) >>
    tag_s!("%}") >>
    block: parse_template >>
    tag_s!("{%") >>
    pws2: opt!(tag_s!("-")) >>
    ws!(tag_s!("endfor")) >>
    nws2: opt!(tag_s!("-")) >>
    tag_s!("%}") >>
    (Node::Loop(WS(pws1.is_some(), nws1.is_some()),
                var, iter, block,
                WS(pws2.is_some(), pws2.is_some())))
));

named!(block_extends<Node>, do_parse!(
    tag_s!("{%") >>
    ws!(tag_s!("extends")) >>
    name: ws!(expr_str_lit) >>
    tag_s!("%}") >>
    (Node::Extends(name))
));

named!(block_block<Node>, do_parse!(
    tag_s!("{%") >>
    pws1: opt!(tag_s!("-")) >>
    ws!(tag_s!("block")) >>
    name: ws!(alphanumeric) >>
    nws1: opt!(tag_s!("-")) >>
    tag_s!("%}") >>
    contents: parse_template >>
    tag_s!("{%") >>
    pws2: opt!(tag_s!("-")) >>
    ws!(tag_s!("endblock")) >>
    nws2: opt!(tag_s!("-")) >>
    tag_s!("%}") >>
    (Node::BlockDef(WS(pws1.is_some(), nws1.is_some()),
                    str::from_utf8(name).unwrap(), contents,
                    WS(pws2.is_some(), pws2.is_some())))
));

named!(parse_template<Vec<Node<'a>>>, many0!(alt!(
    take_content |
    expr_node |
    block_if |
    block_for |
    block_extends |
    block_block
)));

pub fn parse(src: &str) -> Vec<Node> {
    match parse_template(src.as_bytes()) {
        IResult::Done(_, res) => res,
        IResult::Error(err) => panic!("problems parsing template source: {}", err),
        IResult::Incomplete(_) => panic!("parsing incomplete"),
    }
}

#[cfg(test)]
mod tests {
    fn check_ws_split(s: &str, res: &(&str, &str, &str)) {
        let node = super::split_ws_parts(s.as_bytes());
        match node {
            super::Node::Lit(lws, s, rws) => {
                assert_eq!(lws, res.0);
                assert_eq!(s, res.1);
                assert_eq!(rws, res.2);
            },
            _ => { panic!("fail"); },
        }
    }
    #[test]
    fn test_ws_splitter() {
        check_ws_split("", &("", "", ""));
        check_ws_split("a", &("", "a", ""));
        check_ws_split("\ta", &("\t", "a", ""));
        check_ws_split("b\n", &("", "b", "\n"));
        check_ws_split(" \t\r\n", &(" \t\r\n", "", ""));
    }
}
