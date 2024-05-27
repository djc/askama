use std::collections::HashSet;
use std::str;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::char;
use nom::combinator::{cut, map, not, opt, peek, recognize};
use nom::error::ErrorKind;
use nom::error_position;
use nom::multi::{fold_many0, many0, separated_list0};
use nom::sequence::{pair, preceded, terminated, tuple};

use super::{
    char_lit, filter, identifier, not_ws, num_lit, path_or_identifier, str_lit, ws, Level,
    PathOrIdentifier,
};
use crate::{ErrorContext, ParseResult, WithSpan};

macro_rules! expr_prec_layer {
    ( $name:ident, $inner:ident, $op:expr ) => {
        fn $name(i: &'a str, level: Level) -> ParseResult<'a, WithSpan<'a, Self>> {
            let (_, level) = level.nest(i)?;
            let start = i;
            let (i, left) = Self::$inner(i, level)?;
            let (i, right) = many0(pair(
                ws(tag($op)),
                |i| Self::$inner(i, level),
            ))(i)?;
            Ok((
                i,
                right.into_iter().fold(left, |left, (op, right)| {
                    WithSpan::new(Self::BinOp(op, Box::new(left), Box::new(right)), start)
                }),
            ))
        }
    };
    ( $name:ident, $inner:ident, $( $op:expr ),+ ) => {
        fn $name(i: &'a str, level: Level) -> ParseResult<'a, WithSpan<'a, Self>> {
            let (_, level) = level.nest(i)?;
            let start = i;
            let (i, left) = Self::$inner(i, level)?;
            let (i, right) = many0(pair(
                ws(alt(($( tag($op) ),+,))),
                |i| Self::$inner(i, level),
            ))(i)?;
            Ok((
                i,
                right.into_iter().fold(left, |left, (op, right)| {
                    WithSpan::new(Self::BinOp(op, Box::new(left), Box::new(right)), start)
                }),
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<'a> {
    BoolLit(&'a str),
    NumLit(&'a str),
    StrLit(&'a str),
    CharLit(&'a str),
    Var(&'a str),
    Path(Vec<&'a str>),
    Array(Vec<WithSpan<'a, Expr<'a>>>),
    Attr(Box<WithSpan<'a, Expr<'a>>>, &'a str),
    Index(Box<WithSpan<'a, Expr<'a>>>, Box<WithSpan<'a, Expr<'a>>>),
    Filter(Filter<'a>),
    NamedArgument(&'a str, Box<WithSpan<'a, Expr<'a>>>),
    Unary(&'a str, Box<WithSpan<'a, Expr<'a>>>),
    BinOp(
        &'a str,
        Box<WithSpan<'a, Expr<'a>>>,
        Box<WithSpan<'a, Expr<'a>>>,
    ),
    Range(
        &'a str,
        Option<Box<WithSpan<'a, Expr<'a>>>>,
        Option<Box<WithSpan<'a, Expr<'a>>>>,
    ),
    Group(Box<WithSpan<'a, Expr<'a>>>),
    Tuple(Vec<WithSpan<'a, Expr<'a>>>),
    Call(Box<WithSpan<'a, Expr<'a>>>, Vec<WithSpan<'a, Expr<'a>>>),
    RustMacro(Vec<&'a str>, &'a str),
    Try(Box<WithSpan<'a, Expr<'a>>>),
    /// This variant should never be used directly. It is created when generating filter blocks.
    Generated(String),
}

impl<'a> Expr<'a> {
    pub(super) fn arguments(
        i: &'a str,
        level: Level,
        is_template_macro: bool,
    ) -> ParseResult<'a, Vec<WithSpan<'a, Self>>> {
        let (_, level) = level.nest(i)?;
        let mut named_arguments = HashSet::new();
        let start = i;

        preceded(
            ws(char('(')),
            cut(terminated(
                separated_list0(
                    char(','),
                    ws(move |i| {
                        // Needed to prevent borrowing it twice between this closure and the one
                        // calling `Self::named_arguments`.
                        let named_arguments = &mut named_arguments;
                        let has_named_arguments = !named_arguments.is_empty();

                        let (i, expr) = alt((
                            move |i| {
                                Self::named_argument(
                                    i,
                                    level,
                                    named_arguments,
                                    start,
                                    is_template_macro,
                                )
                            },
                            move |i| Self::parse(i, level),
                        ))(i)?;
                        if has_named_arguments && !matches!(*expr, Self::NamedArgument(_, _)) {
                            Err(nom::Err::Failure(ErrorContext::new(
                                "named arguments must always be passed last",
                                start,
                            )))
                        } else {
                            Ok((i, expr))
                        }
                    }),
                ),
                tuple((opt(ws(char(','))), char(')'))),
            )),
        )(i)
    }

    fn named_argument(
        i: &'a str,
        level: Level,
        named_arguments: &mut HashSet<&'a str>,
        start: &'a str,
        is_template_macro: bool,
    ) -> ParseResult<'a, WithSpan<'a, Self>> {
        if !is_template_macro {
            // If this is not a template macro, we don't want to parse named arguments so
            // we instead return an error which will allow to continue the parsing.
            return Err(nom::Err::Error(error_position!(i, ErrorKind::Alt)));
        }

        let (_, level) = level.nest(i)?;
        let (i, (argument, _, value)) =
            tuple((identifier, ws(char('=')), move |i| Self::parse(i, level)))(i)?;
        if named_arguments.insert(argument) {
            Ok((
                i,
                WithSpan::new(Self::NamedArgument(argument, Box::new(value)), start),
            ))
        } else {
            Err(nom::Err::Failure(ErrorContext::new(
                format!("named argument `{argument}` was passed more than once"),
                start,
            )))
        }
    }

    pub(super) fn parse(i: &'a str, level: Level) -> ParseResult<'a, WithSpan<'a, Self>> {
        let (_, level) = level.nest(i)?;
        let start = i;
        let range_right = move |i| {
            pair(
                ws(alt((tag("..="), tag("..")))),
                opt(move |i| Self::or(i, level)),
            )(i)
        };
        alt((
            map(range_right, |(op, right)| {
                WithSpan::new(Self::Range(op, None, right.map(Box::new)), start)
            }),
            map(
                pair(move |i| Self::or(i, level), opt(range_right)),
                |(left, right)| match right {
                    Some((op, right)) => WithSpan::new(
                        Self::Range(op, Some(Box::new(left)), right.map(Box::new)),
                        start,
                    ),
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

    fn filtered(i: &'a str, level: Level) -> ParseResult<'a, WithSpan<'a, Self>> {
        let (_, level) = level.nest(i)?;
        let start = i;
        let (i, (obj, filters)) =
            tuple((|i| Self::prefix(i, level), many0(|i| filter(i, level))))(i)?;

        let mut res = obj;
        for (fname, args) in filters {
            res = WithSpan::new(
                Self::Filter(Filter {
                    name: fname,
                    arguments: {
                        let mut args = args.unwrap_or_default();
                        args.insert(0, res);
                        args
                    },
                }),
                start,
            );
        }

        Ok((i, res))
    }

    fn prefix(i: &'a str, mut level: Level) -> ParseResult<'a, WithSpan<'a, Self>> {
        let (_, nested) = level.nest(i)?;
        let start = i;
        let (i, (ops, mut expr)) = pair(many0(ws(alt((tag("!"), tag("-"))))), |i| {
            Suffix::parse(i, nested)
        })(i)?;

        for op in ops.iter().rev() {
            // This is a rare place where we create recursion in the parsed AST
            // without recursing the parser call stack. However, this can lead
            // to stack overflows in drop glue when the AST is very deep.
            level = level.nest(i)?.1;
            expr = WithSpan::new(Self::Unary(op, Box::new(expr)), start);
        }

        Ok((i, expr))
    }

    fn single(i: &'a str, level: Level) -> ParseResult<'a, WithSpan<'a, Self>> {
        let (_, level) = level.nest(i)?;
        alt((
            Self::num,
            Self::str,
            Self::char,
            Self::path_var_bool,
            move |i| Self::array(i, level),
            move |i| Self::group(i, level),
        ))(i)
    }

    fn group(i: &'a str, level: Level) -> ParseResult<'a, WithSpan<'a, Self>> {
        let (_, level) = level.nest(i)?;
        let start = i;
        let (i, expr) = preceded(ws(char('(')), opt(|i| Self::parse(i, level)))(i)?;
        let expr = match expr {
            Some(expr) => expr,
            None => {
                let (i, _) = char(')')(i)?;
                return Ok((i, WithSpan::new(Self::Tuple(vec![]), start)));
            }
        };

        let (i, comma) = ws(opt(peek(char(','))))(i)?;
        if comma.is_none() {
            let (i, _) = char(')')(i)?;
            return Ok((i, WithSpan::new(Self::Group(Box::new(expr)), start)));
        }

        let mut exprs = vec![expr];
        let (i, _) = fold_many0(
            preceded(char(','), ws(|i| Self::parse(i, level))),
            || (),
            |_, expr| {
                exprs.push(expr);
            },
        )(i)?;
        let (i, _) = pair(ws(opt(char(','))), char(')'))(i)?;
        Ok((i, WithSpan::new(Self::Tuple(exprs), start)))
    }

    fn array(i: &'a str, level: Level) -> ParseResult<'a, WithSpan<'a, Self>> {
        let (_, level) = level.nest(i)?;
        let start = i;
        preceded(
            ws(char('[')),
            cut(terminated(
                map(
                    separated_list0(char(','), ws(move |i| Self::parse(i, level))),
                    |i| WithSpan::new(Self::Array(i), start),
                ),
                char(']'),
            )),
        )(i)
    }

    fn path_var_bool(i: &'a str) -> ParseResult<'a, WithSpan<'a, Self>> {
        let start = i;
        map(path_or_identifier, |v| match v {
            PathOrIdentifier::Path(v) => Self::Path(v),
            PathOrIdentifier::Identifier(v @ "true") => Self::BoolLit(v),
            PathOrIdentifier::Identifier(v @ "false") => Self::BoolLit(v),
            PathOrIdentifier::Identifier(v) => Self::Var(v),
        })(i)
        .map(|(i, expr)| (i, WithSpan::new(expr, start)))
    }

    fn str(i: &'a str) -> ParseResult<'a, WithSpan<'a, Self>> {
        let start = i;
        map(str_lit, |i| WithSpan::new(Self::StrLit(i), start))(i)
    }

    fn num(i: &'a str) -> ParseResult<'a, WithSpan<'a, Self>> {
        let start = i;
        map(num_lit, |i| WithSpan::new(Self::NumLit(i), start))(i)
    }

    fn char(i: &'a str) -> ParseResult<'a, WithSpan<'a, Self>> {
        let start = i;
        map(char_lit, |i| WithSpan::new(Self::CharLit(i), start))(i)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Filter<'a> {
    pub name: &'a str,
    pub arguments: Vec<WithSpan<'a, Expr<'a>>>,
}

enum Suffix<'a> {
    Attr(&'a str),
    Index(WithSpan<'a, Expr<'a>>),
    Call(Vec<WithSpan<'a, Expr<'a>>>),
    // The value is the arguments of the macro call.
    MacroCall(&'a str),
    Try,
}

impl<'a> Suffix<'a> {
    fn parse(i: &'a str, level: Level) -> ParseResult<'a, WithSpan<'a, Expr<'a>>> {
        let (_, level) = level.nest(i)?;
        let (mut i, mut expr) = Expr::single(i, level)?;
        loop {
            let (j, suffix) = opt(alt((
                Self::attr,
                |i| Self::index(i, level),
                |i| Self::call(i, level),
                Self::r#try,
                Self::r#macro,
            )))(i)?;

            match suffix {
                Some(Self::Attr(attr)) => expr = WithSpan::new(Expr::Attr(expr.into(), attr), i),
                Some(Self::Index(index)) => {
                    expr = WithSpan::new(Expr::Index(expr.into(), index.into()), i)
                }
                Some(Self::Call(args)) => expr = WithSpan::new(Expr::Call(expr.into(), args), i),
                Some(Self::Try) => expr = WithSpan::new(Expr::Try(expr.into()), i),
                Some(Self::MacroCall(args)) => match expr.inner {
                    Expr::Path(path) => expr = WithSpan::new(Expr::RustMacro(path, args), i),
                    Expr::Var(name) => expr = WithSpan::new(Expr::RustMacro(vec![name], args), i),
                    _ => return Err(nom::Err::Failure(error_position!(i, ErrorKind::Tag))),
                },
                None => break,
            }

            i = j;
        }
        Ok((i, expr))
    }

    fn r#macro(i: &'a str) -> ParseResult<'a, Self> {
        fn nested_parenthesis(input: &str) -> ParseResult<'_, ()> {
            let mut nested = 0;
            let mut last = 0;
            let mut in_str = false;
            let mut escaped = false;

            for (i, c) in input.char_indices() {
                if !(c == '(' || c == ')') || !in_str {
                    match c {
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

                if escaped && c != '\\' {
                    escaped = false;
                }
            }

            if nested == 0 {
                Ok((&input[last..], ()))
            } else {
                Err(nom::Err::Error(error_position!(
                    input,
                    ErrorKind::SeparatedNonEmptyList
                )))
            }
        }

        preceded(
            pair(ws(char('!')), char('(')),
            cut(terminated(
                map(recognize(nested_parenthesis), Self::MacroCall),
                char(')'),
            )),
        )(i)
    }

    fn attr(i: &'a str) -> ParseResult<'a, Self> {
        map(
            preceded(
                ws(pair(char('.'), not(char('.')))),
                cut(alt((num_lit, identifier))),
            ),
            Self::Attr,
        )(i)
    }

    fn index(i: &'a str, level: Level) -> ParseResult<'a, Self> {
        let (_, level) = level.nest(i)?;
        map(
            preceded(
                ws(char('[')),
                cut(terminated(ws(move |i| Expr::parse(i, level)), char(']'))),
            ),
            Self::Index,
        )(i)
    }

    fn call(i: &'a str, level: Level) -> ParseResult<'a, Self> {
        let (_, level) = level.nest(i)?;
        map(move |i| Expr::arguments(i, level, false), Self::Call)(i)
    }

    fn r#try(i: &'a str) -> ParseResult<'a, Self> {
        map(preceded(take_till(not_ws), char('?')), |_| Self::Try)(i)
    }
}
