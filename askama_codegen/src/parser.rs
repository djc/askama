use nom::{self, IResult};

pub enum Expr<'a> {
     Var(&'a [u8]),
}

pub enum Node<'a> {
    Lit(&'a [u8]),
    Expr(Expr<'a>),
}

fn take_content(i: &[u8]) -> IResult<&[u8], Node> {
    if i.len() < 1 || i[0] == b'{' {
        return IResult::Error(error_position!(nom::ErrorKind::TakeUntil, i));
    }
    for (j, c) in i.iter().enumerate() {
        if *c == b'{' {
            if i.len() < j + 2 {
                return IResult::Done(&i[..0], Node::Lit(&i[..]));
            } else if i[j + 1] == '{' as u8 {
                return IResult::Done(&i[j..], Node::Lit(&i[..j]));
            } else if i[j + 1] == '%' as u8 {
                return IResult::Done(&i[j..], Node::Lit(&i[..j]));
            }
        }
    }
    IResult::Done(&i[..0], Node::Lit(&i[..]))
}

named!(expr_var<Expr>, map!(ws!(nom::alphanumeric), Expr::Var));

named!(any_expr<Expr>, delimited!(tag!("{{"), expr_var, tag!("}}")));

named!(expr_node<Node>, map!(any_expr, Node::Expr));

named!(parse_template< Vec<Node> >, many1!(alt!(take_content | expr_node)));

pub fn parse<'a>(src: &'a str) -> Vec<Node> {
    match parse_template(src.as_bytes()) {
        IResult::Done(_, res) => res,
        _ => panic!("problems parsing template source"),
    }
}
