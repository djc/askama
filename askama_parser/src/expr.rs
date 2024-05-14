use std::borrow::Cow;
use std::collections::HashSet;

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
use crate::{ErrorContext, ParseResult, RcStr};

macro_rules! expr_prec_layer {
    ( $name:ident, $inner:ident, $op:expr ) => {
        fn $name(i: RcStr, level: Level) -> ParseResult<Self> {
            let (i, level) = level.nest(i)?;
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
        fn $name(i: RcStr, level: Level) -> ParseResult<Self> {
            let (i, level) = level.nest(i)?;
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
pub enum Expr {
    BoolLit(RcStr),
    NumLit(RcStr),
    StrLit(RcStr),
    CharLit(RcStr),
    Var(RcStr),
    Path(Vec<RcStr>),
    Array(Vec<Expr>),
    Attr(Box<Expr>, RcStr),
    Index(Box<Expr>, Box<Expr>),
    Filter(Filter),
    NamedArgument(RcStr, Box<Expr>),
    Unary(RcStr, Box<Expr>),
    BinOp(RcStr, Box<Expr>, Box<Expr>),
    Range(RcStr, Option<Box<Expr>>, Option<Box<Expr>>),
    Group(Box<Expr>),
    Tuple(Vec<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    RustMacro(Vec<RcStr>, RcStr),
    Try(Box<Expr>),
    /// This variant should never be used directly. It is created when generating filter blocks.
    Generated(String),
}

impl Expr {
    pub(super) fn arguments(
        i: RcStr,
        level: Level,
        is_template_macro: bool,
    ) -> ParseResult<Vec<Self>> {
        let (i, level) = level.nest(i)?;
        let mut named_arguments = HashSet::new();
        let start = i.clone();

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
                            |i| {
                                Self::named_argument(
                                    i,
                                    level,
                                    named_arguments,
                                    start.clone(),
                                    is_template_macro,
                                )
                            },
                            move |i| Self::parse(i, level),
                        ))(i)?;
                        if has_named_arguments && !matches!(expr, Self::NamedArgument(_, _)) {
                            Err(nom::Err::Failure(ErrorContext {
                                input: start.clone(),
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
        i: RcStr,
        level: Level,
        named_arguments: &mut HashSet<RcStr>,
        start: RcStr,
        is_template_macro: bool,
    ) -> ParseResult<Self> {
        if !is_template_macro {
            // If this is not a template macro, we don't want to parse named arguments so
            // we instead return an error which will allow to continue the parsing.
            return Err(nom::Err::Error(error_position!(i, ErrorKind::Alt)));
        }

        let (i, level) = level.nest(i)?;
        let (i, (argument, _, value)) =
            tuple((identifier, ws(char('=')), move |i| Self::parse(i, level)))(i)?;
        if named_arguments.insert(argument.clone()) {
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

    pub(super) fn parse(i: RcStr, level: Level) -> ParseResult<Self> {
        let (i, level) = level.nest(i)?;
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

    fn filtered(i: RcStr, level: Level) -> ParseResult<Self> {
        let (i, level) = level.nest(i)?;
        let (i, (obj, filters)) =
            tuple((|i| Self::prefix(i, level), many0(|i| filter(i, level))))(i)?;

        let mut res = obj;
        for (fname, args) in filters {
            res = Self::Filter(Filter {
                name: fname,
                arguments: {
                    let mut args = args.unwrap_or_default();
                    args.insert(0, res);
                    args
                },
            });
        }

        Ok((i, res))
    }

    fn prefix(i: RcStr, mut level: Level) -> ParseResult<Self> {
        let (i, nested) = level.nest(i)?;
        let (i, (ops, mut expr)) = pair(many0(ws(alt((tag("!"), tag("-"))))), |i| {
            Suffix::parse(i, nested)
        })(i)?;

        for op in ops.into_iter().rev() {
            // This is a rare place where we create recursion in the parsed AST
            // without recursing the parser call stack. However, this can lead
            // to stack overflows in drop glue when the AST is very deep.
            level = level.nest(i.clone())?.1;
            expr = Self::Unary(op, Box::new(expr));
        }

        Ok((i, expr))
    }

    fn single(i: RcStr, level: Level) -> ParseResult<Self> {
        let (i, level) = level.nest(i)?;
        alt((
            Self::num,
            Self::str,
            Self::char,
            Self::path_var_bool,
            move |i| Self::array(i, level),
            move |i| Self::group(i, level),
        ))(i)
    }

    fn group(i: RcStr, level: Level) -> ParseResult<Self> {
        let (i, level) = level.nest(i)?;
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

    fn array(i: RcStr, level: Level) -> ParseResult<Self> {
        let (i, level) = level.nest(i)?;
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

    fn path_var_bool(i: RcStr) -> ParseResult<Self> {
        map(path_or_identifier, |v| match v {
            PathOrIdentifier::Path(v) => Self::Path(v),
            PathOrIdentifier::Identifier(v) if v == "true" => Self::BoolLit(v),
            PathOrIdentifier::Identifier(v) if v == "false" => Self::BoolLit(v),
            PathOrIdentifier::Identifier(v) => Self::Var(v),
        })(i)
    }

    fn str(i: RcStr) -> ParseResult<Self> {
        map(str_lit, Self::StrLit)(i)
    }

    fn num(i: RcStr) -> ParseResult<Self> {
        map(num_lit, Self::NumLit)(i)
    }

    fn char(i: RcStr) -> ParseResult<Self> {
        map(char_lit, Self::CharLit)(i)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    pub name: RcStr,
    pub arguments: Vec<Expr>,
}

enum Suffix {
    Attr(RcStr),
    Index(Expr),
    Call(Vec<Expr>),
    // The value is the arguments of the macro call.
    MacroCall(RcStr),
    Try,
}

impl Suffix {
    fn parse(i: RcStr, level: Level) -> ParseResult<Expr> {
        let (i, level) = level.nest(i)?;
        let (mut i, mut expr) = Expr::single(i, level)?;
        loop {
            let (j, suffix) = opt(alt((
                Self::attr,
                |i| Self::index(i, level),
                |i| Self::call(i, level),
                Self::r#try,
                Self::r#macro,
            )))(i.clone())?;

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

    fn r#macro(i: RcStr) -> ParseResult<Self> {
        fn nested_parenthesis(input: RcStr) -> ParseResult<()> {
            let mut nested = 0;
            let mut last = 0;
            let mut in_str = false;
            let mut escaped = false;

            for (i, c) in input.as_str().char_indices() {
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
                Ok((input.substr(last..), ()))
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

    fn attr(i: RcStr) -> ParseResult<Self> {
        map(
            preceded(
                ws(pair(char('.'), not(char('.')))),
                cut(alt((num_lit, identifier))),
            ),
            Self::Attr,
        )(i)
    }

    fn index(i: RcStr, level: Level) -> ParseResult<Self> {
        let (i, level) = level.nest(i)?;
        map(
            preceded(
                ws(char('[')),
                cut(terminated(ws(move |i| Expr::parse(i, level)), char(']'))),
            ),
            Self::Index,
        )(i)
    }

    fn call(i: RcStr, level: Level) -> ParseResult<Self> {
        let (i, level) = level.nest(i)?;
        map(move |i| Expr::arguments(i, level, false), Self::Call)(i)
    }

    fn r#try(i: RcStr) -> ParseResult<Self> {
        map(preceded(take_till(not_ws), char('?')), |_| Self::Try)(i)
    }
}
