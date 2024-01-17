use std::borrow::Cow;
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
    char_lit, identifier, not_ws, num_lit, path_or_identifier, str_lit, ws, Level, PathOrIdentifier,
};
use crate::{ErrorContext, ParseResult};

macro_rules! expr_prec_layer {
    ( $name:ident, $inner:ident, $op:expr ) => {
        fn $name(i: &'a str, level: Level) -> ParseResult<'a, Self> {
            let (_, level) = level.nest(i)?;
            let (i, left) = Self::$inner(i, level)?;
            let (i, right) = many0(pair(
                ws(tag($op)),
                |i| Self::$inner(i, level),
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
        fn $name(i: &'a str, level: Level) -> ParseResult<'a, Self> {
            let (_, level) = level.nest(i)?;
            let (i, left) = Self::$inner(i, level)?;
            let (i, right) = many0(pair(
                ws(alt(($( tag($op) ),+,))),
                |i| Self::$inner(i, level),
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

#[derive(Clone, Debug, PartialEq)]
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
    NamedArgument(&'a str, Box<Expr<'a>>),
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
    pub(super) fn arguments(
        i: &'a str,
        level: Level,
        is_template_macro: bool,
    ) -> ParseResult<'a, Vec<Self>> {
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
                        if has_named_arguments && !matches!(expr, Self::NamedArgument(_, _)) {
                            Err(nom::Err::Failure(ErrorContext {
                                input: start,
                                message: Some(Cow::Borrowed(
                                    "named arguments must always be passed last",
                                )),
                            }))
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
    ) -> ParseResult<'a, Self> {
        if !is_template_macro {
            // If this is not a template macro, we don't want to parse named arguments so
            // we instead return an error which will allow to continue the parsing.
            return Err(nom::Err::Error(error_position!(i, ErrorKind::Alt)));
        }

        let (_, level) = level.nest(i)?;
        let (i, (argument, _, value)) =
            tuple((identifier, ws(char('=')), move |i| Self::parse(i, level)))(i)?;
        if named_arguments.insert(argument) {
            Ok((i, Self::NamedArgument(argument, Box::new(value))))
        } else {
            Err(nom::Err::Failure(ErrorContext {
                input: start,
                message: Some(Cow::Owned(format!(
                    "named argument `{argument}` was passed more than once"
                ))),
            }))
        }
    }

    pub(super) fn parse(i: &'a str, level: Level) -> ParseResult<'a, Self> {
        let (_, level) = level.nest(i)?;
        let range_right = move |i| {
            pair(
                ws(alt((tag("..="), tag("..")))),
                opt(move |i| Self::or(i, level)),
            )(i)
        };
        alt((
            map(range_right, |(op, right)| {
                Self::Range(op, None, right.map(Box::new))
            }),
            map(
                pair(move |i| Self::or(i, level), opt(range_right)),
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

    fn filtered(i: &'a str, level: Level) -> ParseResult<'a, Self> {
        let (_, level) = level.nest(i)?;
        #[allow(clippy::type_complexity)]
        fn filter(i: &str, level: Level) -> ParseResult<'_, (&str, Option<Vec<Expr<'_>>>)> {
            let (i, (_, fname, args)) = tuple((
                char('|'),
                ws(identifier),
                opt(|i| Expr::arguments(i, level, false)),
            ))(i)?;
            Ok((i, (fname, args)))
        }

        let (i, (obj, filters)) =
            tuple((|i| Self::prefix(i, level), many0(|i| filter(i, level))))(i)?;

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

    fn prefix(i: &'a str, mut level: Level) -> ParseResult<'a, Self> {
        let (_, nested) = level.nest(i)?;
        let (i, (ops, mut expr)) = pair(many0(ws(alt((tag("!"), tag("-"))))), |i| {
            Suffix::parse(i, nested)
        })(i)?;

        for op in ops.iter().rev() {
            // This is a rare place where we create recursion in the parsed AST
            // without recursing the parser call stack. However, this can lead
            // to stack overflows in drop glue when the AST is very deep.
            level = level.nest(i)?.1;
            expr = Self::Unary(op, Box::new(expr));
        }

        Ok((i, expr))
    }

    fn single(i: &'a str, level: Level) -> ParseResult<'a, Self> {
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

    fn group(i: &'a str, level: Level) -> ParseResult<'a, Self> {
        let (_, level) = level.nest(i)?;
        let (i, expr) = preceded(ws(char('(')), opt(|i| Self::parse(i, level)))(i)?;
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
            preceded(char(','), ws(|i| Self::parse(i, level))),
            || (),
            |_, expr| {
                exprs.push(expr);
            },
        )(i)?;
        let (i, _) = pair(ws(opt(char(','))), char(')'))(i)?;
        Ok((i, Self::Tuple(exprs)))
    }

    fn array(i: &'a str, level: Level) -> ParseResult<'a, Self> {
        let (_, level) = level.nest(i)?;
        preceded(
            ws(char('[')),
            cut(terminated(
                map(
                    separated_list0(char(','), ws(move |i| Self::parse(i, level))),
                    Self::Array,
                ),
                char(']'),
            )),
        )(i)
    }

    fn path_var_bool(i: &'a str) -> ParseResult<'a, Self> {
        map(path_or_identifier, |v| match v {
            PathOrIdentifier::Path(v) => Self::Path(v),
            PathOrIdentifier::Identifier(v @ "true") => Self::BoolLit(v),
            PathOrIdentifier::Identifier(v @ "false") => Self::BoolLit(v),
            PathOrIdentifier::Identifier(v) => Self::Var(v),
        })(i)
    }

    fn str(i: &'a str) -> ParseResult<'a, Self> {
        map(str_lit, Self::StrLit)(i)
    }

    fn num(i: &'a str) -> ParseResult<'a, Self> {
        map(num_lit, Self::NumLit)(i)
    }

    fn char(i: &'a str) -> ParseResult<'a, Self> {
        map(char_lit, Self::CharLit)(i)
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
    fn parse(i: &'a str, level: Level) -> ParseResult<'a, Expr<'a>> {
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
