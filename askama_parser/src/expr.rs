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

impl Expr<'_> {
    pub(super) fn parse(i: &str) -> IResult<&str, Expr<'_>> {
        expr_any(i)
    }

    pub(super) fn parse_arguments(i: &str) -> IResult<&str, Vec<Expr<'_>>> {
        arguments(i)
    }
}

fn expr_bool_lit(i: &str) -> IResult<&str, Expr<'_>> {
    map(bool_lit, Expr::BoolLit)(i)
}

fn expr_num_lit(i: &str) -> IResult<&str, Expr<'_>> {
    map(num_lit, Expr::NumLit)(i)
}

fn expr_array_lit(i: &str) -> IResult<&str, Expr<'_>> {
    delimited(
        ws(char('[')),
        map(separated_list1(ws(char(',')), expr_any), Expr::Array),
        ws(char(']')),
    )(i)
}

fn expr_str_lit(i: &str) -> IResult<&str, Expr<'_>> {
    map(str_lit, Expr::StrLit)(i)
}

fn expr_char_lit(i: &str) -> IResult<&str, Expr<'_>> {
    map(char_lit, Expr::CharLit)(i)
}

fn expr_var(i: &str) -> IResult<&str, Expr<'_>> {
    map(identifier, Expr::Var)(i)
}

fn expr_path(i: &str) -> IResult<&str, Expr<'_>> {
    let (i, path) = path(i)?;
    Ok((i, Expr::Path(path)))
}

fn expr_group(i: &str) -> IResult<&str, Expr<'_>> {
    let (i, expr) = preceded(ws(char('(')), opt(expr_any))(i)?;
    let expr = match expr {
        Some(expr) => expr,
        None => {
            let (i, _) = char(')')(i)?;
            return Ok((i, Expr::Tuple(vec![])));
        }
    };

    let (i, comma) = ws(opt(peek(char(','))))(i)?;
    if comma.is_none() {
        let (i, _) = char(')')(i)?;
        return Ok((i, Expr::Group(Box::new(expr))));
    }

    let mut exprs = vec![expr];
    let (i, _) = fold_many0(
        preceded(char(','), ws(expr_any)),
        || (),
        |_, expr| {
            exprs.push(expr);
        },
    )(i)?;
    let (i, _) = pair(ws(opt(char(','))), char(')'))(i)?;
    Ok((i, Expr::Tuple(exprs)))
}

fn expr_single(i: &str) -> IResult<&str, Expr<'_>> {
    alt((
        expr_bool_lit,
        expr_num_lit,
        expr_str_lit,
        expr_char_lit,
        expr_path,
        expr_array_lit,
        expr_var,
        expr_group,
    ))(i)
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
        let (mut i, mut expr) = expr_single(i)?;
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
            preceded(ws(char('[')), cut(terminated(expr_any, ws(char(']'))))),
            Self::Index,
        )(i)
    }

    fn call(i: &'a str) -> IResult<&'a str, Self> {
        map(arguments, Self::Call)(i)
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
    let (i, (_, fname, args)) = tuple((char('|'), ws(identifier), opt(arguments)))(i)?;
    Ok((i, (fname, args)))
}

fn expr_filtered(i: &str) -> IResult<&str, Expr<'_>> {
    let (i, (obj, filters)) = tuple((expr_prefix, many0(filter)))(i)?;

    let mut res = obj;
    for (fname, args) in filters {
        res = Expr::Filter(fname, {
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

fn expr_prefix(i: &str) -> IResult<&str, Expr<'_>> {
    let (i, (ops, mut expr)) = pair(many0(ws(alt((tag("!"), tag("-"))))), Suffix::parse)(i)?;
    for op in ops.iter().rev() {
        expr = Expr::Unary(op, Box::new(expr));
    }
    Ok((i, expr))
}

macro_rules! expr_prec_layer {
    ( $name:ident, $inner:ident, $op:expr ) => {
        fn $name(i: &str) -> IResult<&str, Expr<'_>> {
            let (i, left) = $inner(i)?;
            let (i, right) = many0(pair(
                ws(tag($op)),
                $inner,
            ))(i)?;
            Ok((
                i,
                right.into_iter().fold(left, |left, (op, right)| {
                    Expr::BinOp(op, Box::new(left), Box::new(right))
                }),
            ))
        }
    };
    ( $name:ident, $inner:ident, $( $op:expr ),+ ) => {
        fn $name(i: &str) -> IResult<&str, Expr<'_>> {
            let (i, left) = $inner(i)?;
            let (i, right) = many0(pair(
                ws(alt(($( tag($op) ),+,))),
                $inner,
            ))(i)?;
            Ok((
                i,
                right.into_iter().fold(left, |left, (op, right)| {
                    Expr::BinOp(op, Box::new(left), Box::new(right))
                }),
            ))
        }
    }
}

expr_prec_layer!(expr_muldivmod, expr_filtered, "*", "/", "%");
expr_prec_layer!(expr_addsub, expr_muldivmod, "+", "-");
expr_prec_layer!(expr_shifts, expr_addsub, ">>", "<<");
expr_prec_layer!(expr_band, expr_shifts, "&");
expr_prec_layer!(expr_bxor, expr_band, "^");
expr_prec_layer!(expr_bor, expr_bxor, "|");
expr_prec_layer!(expr_compare, expr_bor, "==", "!=", ">=", ">", "<=", "<");
expr_prec_layer!(expr_and, expr_compare, "&&");
expr_prec_layer!(expr_or, expr_and, "||");

fn expr_any(i: &str) -> IResult<&str, Expr<'_>> {
    let range_right = |i| pair(ws(alt((tag("..="), tag("..")))), opt(expr_or))(i);
    alt((
        map(range_right, |(op, right)| {
            Expr::Range(op, None, right.map(Box::new))
        }),
        map(
            pair(expr_or, opt(range_right)),
            |(left, right)| match right {
                Some((op, right)) => Expr::Range(op, Some(Box::new(left)), right.map(Box::new)),
                None => left,
            },
        ),
    ))(i)
}

fn arguments(i: &str) -> IResult<&str, Vec<Expr<'_>>> {
    delimited(
        ws(char('(')),
        separated_list0(char(','), ws(expr_any)),
        ws(char(')')),
    )(i)
}
