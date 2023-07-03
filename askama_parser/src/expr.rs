use std::str;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::char;
use nom::combinator::{cut, map, not, opt, peek, recognize};
use nom::error::ErrorKind;
use nom::multi::{fold_many0, many0, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{error_position, IResult};

use super::{bool_lit, char_lit, identifier, not_ws, num_lit, path, str_lit, ws};

macro_rules! expr_prec_layer {
    ( $name:ident, $inner:ident, $op:expr ) => {
        fn $name(i: &'a str) -> IResult<&'a str, Self> {
            let (i, left) = Self::$inner(i)?;
            let (i, right) = many0(pair(
                ws(tag($op)),
                Self::$inner,
            ))(i)?;
            Ok((
                i,
                right.into_iter().fold(left, |left, (op, right)| {
                    Self::BinOp(op, Box::new(left), Box::new(right))
                }),
            ))
        }
    };
    ( $name:ident, $inner:ident, $( $op:expr ),+ ) => {
        fn $name(i: &'a str) -> IResult<&'a str, Self> {
            let (i, left) = Self::$inner(i)?;
            let (i, right) = many0(pair(
                ws(alt(($( tag($op) ),+,))),
                Self::$inner,
            ))(i)?;
            Ok((
                i,
                right.into_iter().fold(left, |left, (op, right)| {
                    Self::BinOp(op, Box::new(left), Box::new(right))
                }),
            ))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr<'a> {
    BoolLit(&'a str),
    NumLit(&'a str),
    StrLit(&'a str),
    CharLit(&'a str),
    Var(&'a str),
    Path(Vec<&'a str>),
    Array(Vec<Expr<'a>>),
    Attr(Box<Expr<'a>>, &'a str),
    Index(Box<Expr<'a>>, Box<Expr<'a>>),
    Filter(&'a str, Vec<Expr<'a>>),
    Unary(&'a str, Box<Expr<'a>>),
    BinOp(&'a str, Box<Expr<'a>>, Box<Expr<'a>>),
    Range(&'a str, Option<Box<Expr<'a>>>, Option<Box<Expr<'a>>>),
    Group(Box<Expr<'a>>),
    Tuple(Vec<Expr<'a>>),
    Call(Box<Expr<'a>>, Vec<Expr<'a>>),
    RustMacro(Vec<&'a str>, &'a str),
    Try(Box<Expr<'a>>),
}

impl<'a> Expr<'a> {
    pub(super) fn arguments(i: &'a str) -> IResult<&'a str, Vec<Self>> {
        delimited(
            ws(char('(')),
            separated_list0(char(','), ws(Self::parse)),
            ws(char(')')),
        )(i)
    }

    pub(super) fn parse(i: &'a str) -> IResult<&'a str, Self> {
        let range_right = |i| pair(ws(alt((tag("..="), tag("..")))), opt(Self::or))(i);
        alt((
            map(range_right, |(op, right)| {
                Self::Range(op, None, right.map(Box::new))
            }),
            map(
                pair(Self::or, opt(range_right)),
                |(left, right)| match right {
                    Some((op, right)) => Self::Range(op, Some(Box::new(left)), right.map(Box::new)),
                    None => left,
                },
            ),
        ))(i)
    }

    expr_prec_layer!(or, and, "||");
    expr_prec_layer!(and, compare, "&&");
    expr_prec_layer!(compare, bor, "==", "!=", ">=", ">", "<=", "<");
    expr_prec_layer!(bor, bxor, "|");
    expr_prec_layer!(bxor, band, "^");
    expr_prec_layer!(band, shifts, "&");
    expr_prec_layer!(shifts, addsub, ">>", "<<");
    expr_prec_layer!(addsub, muldivmod, "+", "-");
    expr_prec_layer!(muldivmod, filtered, "*", "/", "%");

    fn filtered(i: &'a str) -> IResult<&'a str, Self> {
        let (i, (obj, filters)) = tuple((Self::prefix, many0(filter)))(i)?;

        let mut res = obj;
        for (fname, args) in filters {
            res = Self::Filter(fname, {
                let mut args = match args {
                    Some(inner) => inner,
                    None => Vec::new(),
                };
                args.insert(0, res);
                args
            });
        }

        Ok((i, res))
    }

    fn prefix(i: &'a str) -> IResult<&'a str, Self> {
        let (i, (ops, mut expr)) = pair(many0(ws(alt((tag("!"), tag("-"))))), Suffix::parse)(i)?;
        for op in ops.iter().rev() {
            expr = Self::Unary(op, Box::new(expr));
        }
        Ok((i, expr))
    }

    fn single(i: &'a str) -> IResult<&'a str, Self> {
        alt((
            Self::bool,
            Self::num,
            Self::str,
            Self::char,
            Self::path,
            Self::array,
            Self::var,
            Self::group,
        ))(i)
    }

    fn group(i: &'a str) -> IResult<&'a str, Self> {
        let (i, expr) = preceded(ws(char('(')), opt(Self::parse))(i)?;
        let expr = match expr {
            Some(expr) => expr,
            None => {
                let (i, _) = char(')')(i)?;
                return Ok((i, Self::Tuple(vec![])));
            }
        };

        let (i, comma) = ws(opt(peek(char(','))))(i)?;
        if comma.is_none() {
            let (i, _) = char(')')(i)?;
            return Ok((i, Self::Group(Box::new(expr))));
        }

        let mut exprs = vec![expr];
        let (i, _) = fold_many0(
            preceded(char(','), ws(Self::parse)),
            || (),
            |_, expr| {
                exprs.push(expr);
            },
        )(i)?;
        let (i, _) = pair(ws(opt(char(','))), char(')'))(i)?;
        Ok((i, Self::Tuple(exprs)))
    }

    fn array(i: &'a str) -> IResult<&'a str, Self> {
        delimited(
            ws(char('[')),
            map(separated_list1(ws(char(',')), Self::parse), Self::Array),
            ws(char(']')),
        )(i)
    }

    fn path(i: &'a str) -> IResult<&'a str, Self> {
        let (i, path) = path(i)?;
        Ok((i, Self::Path(path)))
    }

    fn var(i: &'a str) -> IResult<&'a str, Self> {
        map(identifier, Self::Var)(i)
    }

    fn str(i: &'a str) -> IResult<&'a str, Self> {
        map(str_lit, Self::StrLit)(i)
    }

    fn num(i: &'a str) -> IResult<&'a str, Self> {
        map(num_lit, Self::NumLit)(i)
    }

    fn char(i: &'a str) -> IResult<&'a str, Self> {
        map(char_lit, Self::CharLit)(i)
    }

    fn bool(i: &'a str) -> IResult<&'a str, Self> {
        map(bool_lit, Self::BoolLit)(i)
    }
}

enum Suffix<'a> {
    Attr(&'a str),
    Index(Expr<'a>),
    Call(Vec<Expr<'a>>),
    // The value is the arguments of the macro call.
    MacroCall(&'a str),
    Try,
}

impl<'a> Suffix<'a> {
    fn parse(i: &'a str) -> IResult<&'a str, Expr<'a>> {
        let (mut i, mut expr) = Expr::single(i)?;
        loop {
            let (j, suffix) = opt(alt((
                Self::attr,
                Self::index,
                Self::call,
                Self::r#try,
                Self::r#macro,
            )))(i)?;

            match suffix {
                Some(Self::Attr(attr)) => expr = Expr::Attr(expr.into(), attr),
                Some(Self::Index(index)) => expr = Expr::Index(expr.into(), index.into()),
                Some(Self::Call(args)) => expr = Expr::Call(expr.into(), args),
                Some(Self::Try) => expr = Expr::Try(expr.into()),
                Some(Self::MacroCall(args)) => match expr {
                    Expr::Path(path) => expr = Expr::RustMacro(path, args),
                    Expr::Var(name) => expr = Expr::RustMacro(vec![name], args),
                    _ => return Err(nom::Err::Failure(error_position!(i, ErrorKind::Tag))),
                },
                None => break,
            }

            i = j;
        }
        Ok((i, expr))
    }

    fn r#macro(i: &'a str) -> IResult<&'a str, Self> {
        preceded(
            pair(ws(char('!')), char('(')),
            cut(terminated(
                map(recognize(nested_parenthesis), Self::MacroCall),
                char(')'),
            )),
        )(i)
    }

    fn attr(i: &'a str) -> IResult<&'a str, Self> {
        map(
            preceded(
                ws(pair(char('.'), not(char('.')))),
                cut(alt((num_lit, identifier))),
            ),
            Self::Attr,
        )(i)
    }

    fn index(i: &'a str) -> IResult<&'a str, Self> {
        map(
            preceded(ws(char('[')), cut(terminated(Expr::parse, ws(char(']'))))),
            Self::Index,
        )(i)
    }

    fn call(i: &'a str) -> IResult<&'a str, Self> {
        map(Expr::arguments, Self::Call)(i)
    }

    fn r#try(i: &'a str) -> IResult<&'a str, Self> {
        map(preceded(take_till(not_ws), char('?')), |_| Self::Try)(i)
    }
}

fn nested_parenthesis(i: &str) -> IResult<&str, ()> {
    let mut nested = 0;
    let mut last = 0;
    let mut in_str = false;
    let mut escaped = false;

    for (i, b) in i.chars().enumerate() {
        if !(b == '(' || b == ')') || !in_str {
            match b {
                '(' => nested += 1,
                ')' => {
                    if nested == 0 {
                        last = i;
                        break;
                    }
                    nested -= 1;
                }
                '"' => {
                    if in_str {
                        if !escaped {
                            in_str = false;
                        }
                    } else {
                        in_str = true;
                    }
                }
                '\\' => {
                    escaped = !escaped;
                }
                _ => (),
            }
        }

        if escaped && b != '\\' {
            escaped = false;
        }
    }

    if nested == 0 {
        Ok((&i[last..], ()))
    } else {
        Err(nom::Err::Error(error_position!(
            i,
            ErrorKind::SeparatedNonEmptyList
        )))
    }
}

fn filter(i: &str) -> IResult<&str, (&str, Option<Vec<Expr<'_>>>)> {
    let (i, (_, fname, args)) = tuple((char('|'), ws(identifier), opt(Expr::arguments)))(i)?;
    Ok((i, (fname, args)))
}
