use nom::{self, IResult};
use std::str;

pub enum Expr<'a> {
    StrLit(&'a str),
    Var(&'a [u8]),
    Filter(&'a str, Box<Expr<'a>>),
    Compare(&'a str, Box<Expr<'a>>, Box<Expr<'a>>),
}

pub enum Target<'a> {
    Name(&'a [u8]),
}

pub enum Node<'a> {
    Lit(&'a [u8]),
    Expr(Expr<'a>),
    Cond(Vec<(Option<Expr<'a>>, Vec<Node<'a>>)>),
    Loop(Target<'a>, Expr<'a>, Vec<Node<'a>>),
    Extends(Expr<'a>),
    BlockDef(&'a str, Vec<Node<'a>>),
    Block(&'a str),
}

pub type Cond<'a> = (Option<Expr<'a>>, Vec<Node<'a>>);

fn take_content(i: &[u8]) -> IResult<&[u8], Node> {
    if i.len() < 1 || i[0] == b'{' {
        return IResult::Error(error_position!(nom::ErrorKind::TakeUntil, i));
    }
    for (j, c) in i.iter().enumerate() {
        if *c == b'{' {
            if i.len() < j + 2 {
                return IResult::Done(&i[..0], Node::Lit(&i[..]));
            } else if i[j + 1] == b'{' || i[j + 1] == b'%' {
                return IResult::Done(&i[j..], Node::Lit(&i[..j]));
            }
        }
    }
    IResult::Done(&i[..0], Node::Lit(&i[..]))
}

named!(expr_str_lit<Expr>, map!(
    delimited!(char!('"'), is_not!("\""), char!('"')),
    |s| Expr::StrLit(str::from_utf8(s).unwrap())));

named!(expr_var<Expr>, map!(nom::alphanumeric, Expr::Var));

named!(target_single<Target>, map!(nom::alphanumeric, Target::Name));

fn expr_filtered(i: &[u8]) -> IResult<&[u8], Expr> {
    let (mut left, mut expr) = match expr_var(i) {
        IResult::Error(err) => { return IResult::Error(err); },
        IResult::Incomplete(needed) => { return IResult::Incomplete(needed); },
        IResult::Done(left, res) => (left, res),
    };
    while left[0] == b'|' {
        match nom::alphanumeric(&left[1..]) {
            IResult::Error(err) => {
                return IResult::Error(err);
            },
            IResult::Incomplete(needed) => {
                return IResult::Incomplete(needed);
            },
            IResult::Done(new_left, res) => {
                left = new_left;
                expr = Expr::Filter(str::from_utf8(res).unwrap(), Box::new(expr));
            },
        };
    }
    return IResult::Done(left, expr);
}

named!(expr_compare<Expr>, do_parse!(
    left: expr_filtered >>
    op: ws!(alt!(
        tag_s!("==") | tag_s!("!=") |
        tag_s!(">=") | tag_s!(">") |
        tag_s!("<=") | tag_s!("<")
    )) >>
    right: expr_filtered >>
    (Expr::Compare(str::from_utf8(op).unwrap(),
                   Box::new(left), Box::new(right)))));

named!(expr_any<Expr>, alt!(
    expr_compare |
    expr_filtered |
    expr_str_lit));

named!(expr_node<Node>, map!(
    delimited!(tag_s!("{{"), ws!(expr_any), tag_s!("}}")),
    Node::Expr));

named!(cond_if<Expr>, do_parse!(
    ws!(tag_s!("if")) >>
    cond: ws!(expr_any) >>
    (cond)));

named!(cond_block<Cond>, do_parse!(
    tag_s!("{%") >>
    ws!(tag_s!("else")) >>
    cond: opt!(cond_if) >>
    tag_s!("%}") >>
    block: parse_template >>
    (cond, block)));

named!(block_if<Node>, do_parse!(
    tag_s!("{%") >>
    cond: ws!(cond_if) >>
    tag_s!("%}") >>
    block: parse_template >>
    elifs: many0!(cond_block) >>
    tag_s!("{%") >>
    ws!(tag_s!("endif")) >>
    tag_s!("%}") >>
    ({
        let mut res = Vec::new();
        res.push((Some(cond), block));
        res.extend(elifs);
        Node::Cond(res)
    })));

named!(block_for<Node>, do_parse!(
    tag_s!("{%") >>
    ws!(tag_s!("for")) >>
    var: ws!(target_single) >>
    ws!(tag_s!("in")) >>
    iter: ws!(expr_any) >>
    tag_s!("%}") >>
    block: parse_template >>
    tag_s!("{%") >>
    ws!(tag_s!("endfor")) >>
    tag_s!("%}") >>
    (Node::Loop(var, iter, block))));

named!(block_extends<Node>, do_parse!(
    tag_s!("{%") >>
    ws!(tag_s!("extends")) >>
    name: ws!(expr_str_lit) >>
    tag_s!("%}") >>
    (Node::Extends(name))));

named!(block_block<Node>, do_parse!(
    tag_s!("{%") >>
    ws!(tag_s!("block")) >>
    name: ws!(nom::alphanumeric) >>
    tag_s!("%}") >>
    contents: parse_template >>
    tag_s!("{%") >>
    ws!(tag_s!("endblock")) >>
    tag_s!("%}") >>
    (Node::BlockDef(str::from_utf8(name).unwrap(), contents))));

named!(parse_template<Vec<Node<'a>>>, many0!(alt!(
    take_content |
    expr_node |
    block_if |
    block_for |
    block_extends |
    block_block)));

pub fn parse<'a>(src: &'a str) -> Vec<Node<'a>> {
    match parse_template(src.as_bytes()) {
        IResult::Done(_, res) => res,
        IResult::Error(err) => panic!("problems parsing template source: {}", err),
        IResult::Incomplete(_) => panic!("parsing incomplete"),
    }
}
