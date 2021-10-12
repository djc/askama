use std::cell::Cell;
use std::str;

use nom::branch::alt;
use nom::bytes::complete::{escaped, is_not, tag, take_till, take_until};
use nom::character::complete::{anychar, char, digit1};
use nom::combinator::{complete, cut, map, opt, recognize, value};
use nom::error::Error;
use nom::multi::{fold_many0, many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{self, IResult, error_position};

use crate::{CompileError, Syntax};

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Lit(&'a str, &'a str, &'a str),
    Comment(Ws),
    Expr(Ws, Expr<'a>),
    Call(Ws, Option<&'a str>, &'a str, Vec<Expr<'a>>),
    LetDecl(Ws, Target<'a>),
    Let(Ws, Target<'a>, Expr<'a>),
    Cond(Vec<Cond<'a>>, Ws),
    Match(Ws, Expr<'a>, Vec<When<'a>>, Ws),
    Loop(Ws, Target<'a>, Expr<'a>, Vec<Node<'a>>, Ws),
    Extends(Expr<'a>),
    BlockDef(Ws, &'a str, Vec<Node<'a>>, Ws),
    Include(Ws, &'a str),
    Import(Ws, &'a str, &'a str),
    Macro(&'a str, Macro<'a>),
    Raw(Ws, &'a str, Ws),
    Break(Ws),
    Continue(Ws),
}

#[derive(Debug, PartialEq)]
pub enum Expr<'a> {
    BoolLit(&'a str),
    NumLit(&'a str),
    StrLit(&'a str),
    CharLit(&'a str),
    Var(&'a str),
    VarCall(&'a str, Vec<Expr<'a>>),
    Path(Vec<&'a str>),
    PathCall(Vec<&'a str>, Vec<Expr<'a>>),
    Array(Vec<Expr<'a>>),
    Attr(Box<Expr<'a>>, &'a str),
    Index(Box<Expr<'a>>, Box<Expr<'a>>),
    Filter(&'a str, Vec<Expr<'a>>),
    Unary(&'a str, Box<Expr<'a>>),
    BinOp(&'a str, Box<Expr<'a>>, Box<Expr<'a>>),
    Range(&'a str, Option<Box<Expr<'a>>>, Option<Box<Expr<'a>>>),
    Group(Box<Expr<'a>>),
    MethodCall(Box<Expr<'a>>, &'a str, Vec<Expr<'a>>),
    RustMacro(&'a str, &'a str),
}

impl Expr<'_> {
    /// Returns `true` if enough assumptions can be made,
    /// to determine that `self` is copyable.
    pub fn is_copyable(&self) -> bool {
        self.is_copyable_within_op(false)
    }

    fn is_copyable_within_op(&self, within_op: bool) -> bool {
        use Expr::*;
        match self {
            BoolLit(_) | NumLit(_) | StrLit(_) | CharLit(_) => true,
            Unary(.., expr) => expr.is_copyable_within_op(true),
            BinOp(_, lhs, rhs) => {
                lhs.is_copyable_within_op(true) && rhs.is_copyable_within_op(true)
            }
            Range(..) => true,
            // The result of a call likely doesn't need to be borrowed,
            // as in that case the call is more likely to return a
            // reference in the first place then.
            VarCall(..) | Path(..) | PathCall(..) | MethodCall(..) => true,
            // If the `expr` is within a `Unary` or `BinOp` then
            // an assumption can be made that the operand is copy.
            // If not, then the value is moved and adding `.clone()`
            // will solve that issue. However, if the operand is
            // implicitly borrowed, then it's likely not even possible
            // to get the template to compile.
            _ => within_op && self.is_attr_self(),
        }
    }

    /// Returns `true` if this is an `Attr` where the `obj` is `"self"`.
    pub fn is_attr_self(&self) -> bool {
        match self {
            Expr::Attr(obj, _) if matches!(obj.as_ref(), Expr::Var("self")) => true,
            Expr::Attr(obj, _) if matches!(obj.as_ref(), Expr::Attr(..)) => obj.is_attr_self(),
            _ => false,
        }
    }
}

pub type When<'a> = (Ws, Target<'a>, Vec<Node<'a>>);

#[derive(Debug, PartialEq)]
pub struct Macro<'a> {
    pub ws1: Ws,
    pub args: Vec<&'a str>,
    pub nodes: Vec<Node<'a>>,
    pub ws2: Ws,
}

#[derive(Debug, PartialEq)]
pub enum Target<'a> {
    Name(&'a str),
    Tuple(Vec<&'a str>, Vec<Target<'a>>),
    Struct(Vec<&'a str>, Vec<(&'a str, Target<'a>)>),
    NumLit(&'a str),
    StrLit(&'a str),
    CharLit(&'a str),
    BoolLit(&'a str),
    Path(Vec<&'a str>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ws(pub bool, pub bool);

pub type Cond<'a> = (Ws, Option<CondTest<'a>>, Vec<Node<'a>>);

#[derive(Debug, PartialEq)]
pub struct CondTest<'a> {
    pub target: Option<Target<'a>>,
    pub expr: Expr<'a>,
}

fn is_ws(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}

fn not_ws(c: u8) -> bool {
    !is_ws(c as char)
}

fn ws<'a, O>(
    inner: impl FnMut(&'a [u8]) -> IResult<&'a [u8], O>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O> {
    delimited(take_till(not_ws), inner, take_till(not_ws))
}

fn split_ws_parts(s: &[u8]) -> Node<'_> {
    let s = str::from_utf8(s).unwrap();
    let trimmed_start = s.trim_start_matches(is_ws);
    let len_start = s.len() - trimmed_start.len();
    let trimmed = trimmed_start.trim_end_matches(is_ws);
    Node::Lit(&s[..len_start], trimmed, &trimmed_start[trimmed.len()..])
}

#[derive(Debug)]
enum ContentState {
    Start,
    Any,
    Brace(usize),
    End(usize),
}

struct State<'a> {
    syntax: &'a Syntax<'a>,
    loop_depth: Cell<usize>,
}

fn take_content<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    use crate::parser::ContentState::*;
    let bs = s.syntax.block_start.as_bytes()[0];
    let be = s.syntax.block_start.as_bytes()[1];
    let cs = s.syntax.comment_start.as_bytes()[0];
    let ce = s.syntax.comment_start.as_bytes()[1];
    let es = s.syntax.expr_start.as_bytes()[0];
    let ee = s.syntax.expr_start.as_bytes()[1];

    let mut state = Start;
    for (idx, c) in i.iter().enumerate() {
        state = match state {
            Start | Any => {
                if *c == bs || *c == es || *c == cs {
                    Brace(idx)
                } else {
                    Any
                }
            }
            Brace(start) => {
                if *c == be || *c == ee || *c == ce {
                    End(start)
                } else {
                    Any
                }
            }
            End(_) => unreachable!(),
        };
        if let End(_) = state {
            break;
        }
    }

    match state {
        Any | Brace(_) => Ok((&i[..0], split_ws_parts(i))),
        Start | End(0) => Err(nom::Err::Error(error_position!(
            i,
            nom::error::ErrorKind::TakeUntil
        ))),
        End(start) => Ok((&i[start..], split_ws_parts(&i[..start]))),
    }
}

fn identifier(input: &[u8]) -> IResult<&[u8], &str> {
    if !nom::character::is_alphabetic(input[0]) && input[0] != b'_' && !non_ascii(input[0]) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::AlphaNumeric,
        )));
    }
    for (i, ch) in input.iter().enumerate() {
        if i == 0 || nom::character::is_alphanumeric(*ch) || *ch == b'_' || non_ascii(*ch) {
            continue;
        }
        return Ok((&input[i..], str::from_utf8(&input[..i]).unwrap()));
    }
    Ok((&input[1..], str::from_utf8(&input[..1]).unwrap()))
}

#[inline]
fn non_ascii(chr: u8) -> bool {
    (0x80..=0xFD).contains(&chr)
}

fn bool_lit(i: &[u8]) -> IResult<&[u8], &str> {
    map(alt((tag("false"), tag("true"))), |s| {
        str::from_utf8(s).unwrap()
    })(i)
}

fn expr_bool_lit(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    map(bool_lit, Expr::BoolLit)(i)
}

fn variant_bool_lit(i: &[u8]) -> IResult<&[u8], Target<'_>> {
    map(bool_lit, Target::BoolLit)(i)
}

fn num_lit(i: &[u8]) -> IResult<&[u8], &str> {
    map(recognize(pair(digit1, opt(pair(char('.'), digit1)))), |s| {
        str::from_utf8(s).unwrap()
    })(i)
}

fn expr_num_lit(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    map(num_lit, Expr::NumLit)(i)
}

fn expr_array_lit(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    delimited(
        ws(char('[')),
        map(separated_list1(ws(char(',')), expr_any), Expr::Array),
        ws(char(']')),
    )(i)
}

fn variant_num_lit(i: &[u8]) -> IResult<&[u8], Target<'_>> {
    map(num_lit, Target::NumLit)(i)
}

fn str_lit(i: &[u8]) -> IResult<&[u8], &str> {
    map(
        delimited(
            char('\"'),
            opt(escaped(is_not("\\\""), '\\', anychar)),
            char('\"'),
        ),
        |s| s.map(|s| str::from_utf8(s).unwrap()).unwrap_or(""),
    )(i)
}

fn expr_str_lit(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    map(str_lit, Expr::StrLit)(i)
}

fn variant_str_lit(i: &[u8]) -> IResult<&[u8], Target<'_>> {
    map(str_lit, Target::StrLit)(i)
}

fn char_lit(i: &[u8]) -> IResult<&[u8], &str> {
    map(
        delimited(
            char('\''),
            opt(escaped(is_not("\\\'"), '\\', anychar)),
            char('\''),
        ),
        |s| s.map(|s| str::from_utf8(s).unwrap()).unwrap_or(""),
    )(i)
}

fn expr_char_lit(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    map(char_lit, Expr::CharLit)(i)
}

fn variant_char_lit(i: &[u8]) -> IResult<&[u8], Target<'_>> {
    map(char_lit, Target::CharLit)(i)
}

fn expr_var(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    map(identifier, Expr::Var)(i)
}

fn expr_var_call(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    let (i, (s, args)) = tuple((ws(identifier), arguments))(i)?;
    Ok((i, Expr::VarCall(s, args)))
}

fn path(i: &[u8]) -> IResult<&[u8], Vec<&str>> {
    let root = opt(value("", ws(tag("::"))));
    let tail = separated_list1(ws(tag("::")), identifier);

    match tuple((root, identifier, ws(tag("::")), tail))(i) {
        Ok((i, (root, start, _, rest))) => {
            let mut path = Vec::new();
            path.extend(root);
            path.push(start);
            path.extend(rest);
            Ok((i, path))
        }
        Err(err) => {
            if let Ok((i, name)) = identifier(i) {
                // The returned identifier can be assumed to be path if:
                // - Contains both a lowercase and uppercase character, i.e. a type name like `None`
                // - Doesn't contain any lowercase characters, i.e. it's a constant
                // In short, if it contains any uppercase characters it's a path.
                if name.contains(char::is_uppercase) {
                    return Ok((i, vec![name]));
                }
            }

            // If `identifier()` fails then just return the original error
            Err(err)
        }
    }
}

fn expr_path(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    let (i, path) = path(i)?;
    Ok((i, Expr::Path(path)))
}

fn expr_path_call(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    let (i, (path, args)) = tuple((ws(path), arguments))(i)?;
    Ok((i, Expr::PathCall(path, args)))
}

fn named_target(i: &[u8]) -> IResult<&[u8], (&str, Target<'_>)> {
    let (i, (src, target)) = pair(identifier, opt(preceded(ws(char(':')), target)))(i)?;
    Ok((i, (src, target.unwrap_or(Target::Name(src)))))
}

fn variant_lit(i: &[u8]) -> IResult<&[u8], Target<'_>> {
    alt((
        variant_str_lit,
        variant_char_lit,
        variant_num_lit,
        variant_bool_lit,
    ))(i)
}

fn target(i: &[u8]) -> IResult<&[u8], Target<'_>> {
    let mut opt_opening_paren = map(opt(ws(char('('))), |o| o.is_some());
    let mut opt_closing_paren = map(opt(ws(char(')'))), |o| o.is_some());
    let mut opt_opening_brace = map(opt(ws(char('{'))), |o| o.is_some());

    let (i, lit) = opt(variant_lit)(i)?;
    if let Some(lit) = lit {
        return Ok((i, lit));
    }

    // match tuples and unused parentheses
    let (i, target_is_tuple) = opt_opening_paren(i)?;
    if target_is_tuple {
        let (i, is_empty_tuple) = opt_closing_paren(i)?;
        if is_empty_tuple {
            return Ok((i, Target::Tuple(Vec::new(), Vec::new())));
        }

        let (i, first_target) = target(i)?;
        let (i, is_unused_paren) = opt_closing_paren(i)?;
        if is_unused_paren {
            return Ok((i, first_target));
        }

        let mut targets = vec![first_target];
        let (i, _) = cut(tuple((
            fold_many0(
                preceded(ws(char(',')), target),
                || (),
                |_, target| {
                    targets.push(target);
                },
            ),
            opt(ws(char(','))),
            ws(cut(char(')'))),
        )))(i)?;
        return Ok((i, Target::Tuple(Vec::new(), targets)));
    }

    // match structs
    let (i, path) = opt(path)(i)?;
    if let Some(path) = path {
        let i_before_matching_with = i;
        let (i, _) = opt(ws(tag("with")))(i)?;

        let (i, is_unnamed_struct) = opt_opening_paren(i)?;
        if is_unnamed_struct {
            let (i, targets) = alt((
                map(char(')'), |_| Vec::new()),
                terminated(
                    cut(separated_list1(ws(char(',')), target)),
                    pair(opt(ws(char(','))), ws(cut(char(')')))),
                ),
            ))(i)?;
            return Ok((i, Target::Tuple(path, targets)));
        }

        let (i, is_named_struct) = opt_opening_brace(i)?;
        if is_named_struct {
            let (i, targets) = alt((
                map(char('}'), |_| Vec::new()),
                terminated(
                    cut(separated_list1(ws(char(',')), named_target)),
                    pair(opt(ws(char(','))), ws(cut(char('}')))),
                ),
            ))(i)?;
            return Ok((i, Target::Struct(path, targets)));
        }

        return Ok((i_before_matching_with, Target::Path(path)));
    }

    // neither literal nor struct nor path
    map(identifier, Target::Name)(i)
}

fn arguments(i: &[u8]) -> IResult<&[u8], Vec<Expr<'_>>> {
    delimited(
        ws(char('(')),
        separated_list0(char(','), ws(expr_any)),
        ws(char(')')),
    )(i)
}

fn macro_arguments(i: &[u8]) -> IResult<&[u8], &str> {
    delimited(char('('), nested_parenthesis, char(')'))(i)
}

fn nested_parenthesis(i: &[u8]) -> IResult<&[u8], &str> {
    let mut nested = 0;
    let mut last = 0;
    let mut in_str = false;
    let mut escaped = false;

    for (i, b) in i.iter().enumerate() {
        if !(*b == b'(' || *b == b')') || !in_str {
            match *b {
                b'(' => nested += 1,
                b')' => {
                    if nested == 0 {
                        last = i;
                        break;
                    }
                    nested -= 1;
                }
                b'"' => {
                    if in_str {
                        if !escaped {
                            in_str = false;
                        }
                    } else {
                        in_str = true;
                    }
                }
                b'\\' => {
                    escaped = !escaped;
                }
                _ => (),
            }
        }

        if escaped && *b != b'\\' {
            escaped = false;
        }
    }

    if nested == 0 {
        Ok((&i[last..], str::from_utf8(&i[..last]).unwrap()))
    } else {
        Err(nom::Err::Error(error_position!(
            i,
            nom::error::ErrorKind::SeparatedNonEmptyList
        )))
    }
}

fn parameters(i: &[u8]) -> IResult<&[u8], Vec<&str>> {
    delimited(
        ws(char('(')),
        separated_list0(char(','), ws(identifier)),
        ws(char(')')),
    )(i)
}

fn expr_group(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    map(delimited(ws(char('(')), expr_any, ws(char(')'))), |s| {
        Expr::Group(Box::new(s))
    })(i)
}

fn expr_single(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    alt((
        expr_bool_lit,
        expr_num_lit,
        expr_str_lit,
        expr_char_lit,
        expr_path_call,
        expr_path,
        expr_rust_macro,
        expr_array_lit,
        expr_var_call,
        expr_var,
        expr_group,
    ))(i)
}

fn attr(i: &[u8]) -> IResult<&[u8], (&str, Option<Vec<Expr<'_>>>)> {
    let (i, (_, attr, args)) = tuple((
        ws(char('.')),
        alt((num_lit, identifier)),
        ws(opt(arguments)),
    ))(i)?;
    Ok((i, (attr, args)))
}

fn expr_attr(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    let (i, (obj, attrs)) = tuple((expr_single, many0(attr)))(i)?;

    let mut res = obj;
    for (aname, args) in attrs {
        res = if let Some(args) = args {
            Expr::MethodCall(Box::new(res), aname, args)
        } else {
            Expr::Attr(Box::new(res), aname)
        };
    }

    Ok((i, res))
}

fn expr_index(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    let key = opt(tuple((ws(char('[')), expr_any, ws(char(']')))));
    let (i, (obj, key)) = tuple((expr_attr, key))(i)?;
    let key = key.map(|(_, key, _)| key);

    Ok((
        i,
        match key {
            Some(key) => Expr::Index(Box::new(obj), Box::new(key)),
            None => obj,
        },
    ))
}

fn filter(i: &[u8]) -> IResult<&[u8], (&str, Option<Vec<Expr<'_>>>)> {
    let (i, (_, fname, args)) = tuple((char('|'), ws(identifier), opt(arguments)))(i)?;
    Ok((i, (fname, args)))
}

fn expr_filtered(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    let (i, (obj, filters)) = tuple((expr_unary, many0(filter)))(i)?;

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

fn expr_unary(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    let (i, (op, expr)) = tuple((opt(alt((ws(tag("!")), ws(tag("-"))))), expr_index))(i)?;
    Ok((
        i,
        match op {
            Some(op) => Expr::Unary(str::from_utf8(op).unwrap(), Box::new(expr)),
            None => expr,
        },
    ))
}

fn expr_rust_macro(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    let (i, (mname, _, args)) = tuple((identifier, char('!'), macro_arguments))(i)?;
    Ok((i, Expr::RustMacro(mname, args)))
}

macro_rules! expr_prec_layer {
    ( $name:ident, $inner:ident, $op:expr ) => {
        fn $name(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
            let (i, left) = $inner(i)?;
            let (i, right) = many0(pair(
                ws(tag($op)),
                $inner,
            ))(i)?;
            Ok((
                i,
                right.into_iter().fold(left, |left, (op, right)| {
                    Expr::BinOp(str::from_utf8(op).unwrap(), Box::new(left), Box::new(right))
                }),
            ))
        }
    };
    ( $name:ident, $inner:ident, $( $op:expr ),+ ) => {
        fn $name(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
            let (i, left) = $inner(i)?;
            let (i, right) = many0(pair(
                ws(alt(($( tag($op) ),*,))),
                $inner,
            ))(i)?;
            Ok((
                i,
                right.into_iter().fold(left, |left, (op, right)| {
                    Expr::BinOp(str::from_utf8(op).unwrap(), Box::new(left), Box::new(right))
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

fn range_right(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    let (i, (_, incl, right)) = tuple((ws(tag("..")), opt(ws(char('='))), opt(expr_or)))(i)?;
    Ok((
        i,
        Expr::Range(
            if incl.is_some() { "..=" } else { ".." },
            None,
            right.map(Box::new),
        ),
    ))
}

fn expr_any(i: &[u8]) -> IResult<&[u8], Expr<'_>> {
    let compound = map(tuple((expr_or, range_right)), |(left, rest)| match rest {
        Expr::Range(op, _, right) => Expr::Range(op, Some(Box::new(left)), right),
        _ => unreachable!(),
    });
    alt((range_right, compound, expr_or))(i)
}

fn expr_node<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    let mut p = tuple((
        |i| tag_expr_start(i, s),
        cut(tuple((opt(char('-')), ws(expr_any), opt(char('-')), |i| {
            tag_expr_end(i, s)
        }))),
    ));
    let (i, (_, (pws, expr, nws, _))) = p(i)?;
    Ok((i, Node::Expr(Ws(pws.is_some(), nws.is_some()), expr)))
}

fn block_call(i: &[u8]) -> IResult<&[u8], Node<'_>> {
    let mut p = tuple((
        opt(char('-')),
        ws(tag("call")),
        cut(tuple((
            opt(tuple((ws(identifier), ws(tag("::"))))),
            ws(identifier),
            ws(arguments),
            opt(char('-')),
        ))),
    ));
    let (i, (pws, _, (scope, name, args, nws))) = p(i)?;
    let scope = scope.map(|(scope, _)| scope);
    Ok((
        i,
        Node::Call(Ws(pws.is_some(), nws.is_some()), scope, name, args),
    ))
}

fn cond_if(i: &[u8]) -> IResult<&[u8], CondTest<'_>> {
    let mut p = preceded(
        ws(tag("if")),
        cut(tuple((
            opt(delimited(
                ws(alt((tag("let"), tag("set")))),
                ws(target),
                ws(char('=')),
            )),
            ws(expr_any),
        ))),
    );
    let (i, (target, expr)) = p(i)?;
    Ok((i, CondTest { target, expr }))
}

fn cond_block<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Cond<'a>> {
    let mut p = tuple((
        |i| tag_block_start(i, s),
        opt(char('-')),
        ws(tag("else")),
        cut(tuple((
            opt(cond_if),
            opt(char('-')),
            |i| tag_block_end(i, s),
            cut(|i| parse_template(i, s)),
        ))),
    ));
    let (i, (_, pws, _, (cond, nws, _, block))) = p(i)?;
    Ok((i, (Ws(pws.is_some(), nws.is_some()), cond, block)))
}

fn block_if<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    let mut p = tuple((
        opt(char('-')),
        cond_if,
        cut(tuple((
            opt(char('-')),
            |i| tag_block_end(i, s),
            cut(tuple((
                |i| parse_template(i, s),
                many0(|i| cond_block(i, s)),
                cut(tuple((
                    |i| tag_block_start(i, s),
                    opt(char('-')),
                    ws(tag("endif")),
                    opt(char('-')),
                ))),
            ))),
        ))),
    ));
    let (i, (pws1, cond, (nws1, _, (block, elifs, (_, pws2, _, nws2))))) = p(i)?;

    let mut res = vec![(Ws(pws1.is_some(), nws1.is_some()), Some(cond), block)];
    res.extend(elifs);
    Ok((i, Node::Cond(res, Ws(pws2.is_some(), nws2.is_some()))))
}

fn match_else_block<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], When<'a>> {
    let mut p = tuple((
        |i| tag_block_start(i, s),
        opt(char('-')),
        ws(tag("else")),
        cut(tuple((
            opt(char('-')),
            |i| tag_block_end(i, s),
            cut(|i| parse_template(i, s)),
        ))),
    ));
    let (i, (_, pws, _, (nws, _, block))) = p(i)?;
    Ok((
        i,
        (Ws(pws.is_some(), nws.is_some()), Target::Name("_"), block),
    ))
}

fn when_block<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], When<'a>> {
    let mut p = tuple((
        |i| tag_block_start(i, s),
        opt(char('-')),
        ws(tag("when")),
        cut(tuple((
            ws(target),
            opt(char('-')),
            |i| tag_block_end(i, s),
            cut(|i| parse_template(i, s)),
        ))),
    ));
    let (i, (_, pws, _, (target, nws, _, block))) = p(i)?;
    Ok((i, (Ws(pws.is_some(), nws.is_some()), target, block)))
}

fn block_match<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    let mut p = tuple((
        opt(char('-')),
        ws(tag("match")),
        cut(tuple((
            ws(expr_any),
            opt(char('-')),
            |i| tag_block_end(i, s),
            cut(tuple((
                opt(|i| take_content(i, s)),
                many1(|i| when_block(i, s)),
                cut(tuple((
                    opt(|i| match_else_block(i, s)),
                    cut(tuple((
                        ws(|i| tag_block_start(i, s)),
                        opt(char('-')),
                        ws(tag("endmatch")),
                        opt(char('-')),
                    ))),
                ))),
            ))),
        ))),
    ));
    let (i, (pws1, _, (expr, nws1, _, (inter, arms, (else_arm, (_, pws2, _, nws2)))))) = p(i)?;

    let mut arms = arms;
    if let Some(arm) = else_arm {
        arms.push(arm);
    }

    match inter {
        Some(Node::Lit(_, val, rws)) => {
            assert!(
                val.is_empty(),
                "only whitespace allowed between match and first when, found {}",
                val
            );
            assert!(
                rws.is_empty(),
                "only whitespace allowed between match and first when, found {}",
                rws
            );
        }
        None => {}
        _ => panic!("only literals allowed between match and first when"),
    }

    Ok((
        i,
        Node::Match(
            Ws(pws1.is_some(), nws1.is_some()),
            expr,
            arms,
            Ws(pws2.is_some(), nws2.is_some()),
        ),
    ))
}

fn block_let(i: &[u8]) -> IResult<&[u8], Node<'_>> {
    let mut p = tuple((
        opt(char('-')),
        ws(alt((tag("let"), tag("set")))),
        cut(tuple((
            ws(target),
            opt(tuple((ws(char('=')), ws(expr_any)))),
            opt(char('-')),
        ))),
    ));
    let (i, (pws, _, (var, val, nws))) = p(i)?;

    Ok((
        i,
        if let Some((_, val)) = val {
            Node::Let(Ws(pws.is_some(), nws.is_some()), var, val)
        } else {
            Node::LetDecl(Ws(pws.is_some(), nws.is_some()), var)
        },
    ))
}

fn parse_loop_content<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Vec<Node<'a>>> {
    s.loop_depth.set(s.loop_depth.get() + 1);
    let (i, node) = parse_template(i, s)?;
    s.loop_depth.set(s.loop_depth.get() - 1);
    Ok((i, node))
}

fn block_for<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    let mut p = tuple((
        opt(char('-')),
        ws(tag("for")),
        cut(tuple((
            ws(target),
            ws(tag("in")),
            cut(tuple((
                ws(expr_any),
                opt(char('-')),
                |i| tag_block_end(i, s),
                cut(tuple((
                    |i| parse_loop_content(i, s),
                    cut(tuple((
                        |i| tag_block_start(i, s),
                        opt(char('-')),
                        ws(tag("endfor")),
                        opt(char('-')),
                    ))),
                ))),
            ))),
        ))),
    ));
    let (i, (pws1, _, (var, _, (iter, nws1, _, (block, (_, pws2, _, nws2)))))) = p(i)?;
    Ok((
        i,
        Node::Loop(
            Ws(pws1.is_some(), nws1.is_some()),
            var,
            iter,
            block,
            Ws(pws2.is_some(), nws2.is_some()),
        ),
    ))
}

fn block_extends(i: &[u8]) -> IResult<&[u8], Node<'_>> {
    let (i, (_, name)) = tuple((ws(tag("extends")), ws(expr_str_lit)))(i)?;
    Ok((i, Node::Extends(name)))
}

fn block_block<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    let mut start = tuple((
        opt(char('-')),
        ws(tag("block")),
        cut(tuple((ws(identifier), opt(char('-')), |i| {
            tag_block_end(i, s)
        }))),
    ));
    let (i, (pws1, _, (name, nws1, _))) = start(i)?;

    let mut end = cut(tuple((
        |i| parse_template(i, s),
        cut(tuple((
            |i| tag_block_start(i, s),
            opt(char('-')),
            ws(tag("endblock")),
            cut(tuple((opt(ws(tag(name))), opt(char('-'))))),
        ))),
    )));
    let (i, (contents, (_, pws2, _, (_, nws2)))) = end(i)?;

    Ok((
        i,
        Node::BlockDef(
            Ws(pws1.is_some(), nws1.is_some()),
            name,
            contents,
            Ws(pws2.is_some(), nws2.is_some()),
        ),
    ))
}

fn block_include(i: &[u8]) -> IResult<&[u8], Node<'_>> {
    let mut p = tuple((
        opt(char('-')),
        ws(tag("include")),
        cut(pair(ws(expr_str_lit), opt(char('-')))),
    ));
    let (i, (pws, _, (name, nws))) = p(i)?;
    Ok((
        i,
        Node::Include(
            Ws(pws.is_some(), nws.is_some()),
            match name {
                Expr::StrLit(s) => s,
                _ => panic!("include path must be a string literal"),
            },
        ),
    ))
}

fn block_import(i: &[u8]) -> IResult<&[u8], Node<'_>> {
    let mut p = tuple((
        opt(char('-')),
        ws(tag("import")),
        cut(tuple((
            ws(expr_str_lit),
            ws(tag("as")),
            cut(pair(ws(identifier), opt(char('-')))),
        ))),
    ));
    let (i, (pws, _, (name, _, (scope, nws)))) = p(i)?;
    Ok((
        i,
        Node::Import(
            Ws(pws.is_some(), nws.is_some()),
            match name {
                Expr::StrLit(s) => s,
                _ => panic!("import path must be a string literal"),
            },
            scope,
        ),
    ))
}

fn block_macro<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    let mut p = tuple((
        opt(char('-')),
        ws(tag("macro")),
        cut(tuple((
            ws(identifier),
            ws(parameters),
            opt(char('-')),
            |i| tag_block_end(i, s),
            cut(tuple((
                |i| parse_template(i, s),
                cut(tuple((
                    |i| tag_block_start(i, s),
                    opt(char('-')),
                    ws(tag("endmacro")),
                    opt(char('-')),
                ))),
            ))),
        ))),
    ));

    let (i, (pws1, _, (name, params, nws1, _, (contents, (_, pws2, _, nws2))))) = p(i)?;
    assert_ne!(name, "super", "invalid macro name 'super'");

    Ok((
        i,
        Node::Macro(
            name,
            Macro {
                ws1: Ws(pws1.is_some(), nws1.is_some()),
                args: params,
                nodes: contents,
                ws2: Ws(pws2.is_some(), nws2.is_some()),
            },
        ),
    ))
}

fn block_raw<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    let mut p = tuple((
        opt(char('-')),
        ws(tag("raw")),
        cut(tuple((
            opt(char('-')),
            |i| tag_block_end(i, s),
            take_until("{% endraw %}"),
            |i| tag_block_start(i, s),
            opt(char('-')),
            ws(tag("endraw")),
            opt(char('-')),
        ))),
    ));

    let (i, (pws1, _, (nws1, _, contents, _, pws2, _, nws2))) = p(i)?;
    let str_contents = str::from_utf8(contents).unwrap();
    Ok((
        i,
        Node::Raw(
            Ws(pws1.is_some(), nws1.is_some()),
            str_contents,
            Ws(pws2.is_some(), nws2.is_some()),
        ),
    ))
}

fn break_statement<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    let mut p = tuple((opt(char('-')), ws(tag("break")), opt(char('-'))));
    let (j, (pws, _, nws)) = p(i)?;
    if s.loop_depth.get() == 0 {
        return Err(nom::Err::Failure(error_position!(
            i,
            nom::error::ErrorKind::Tag
        )));
    }
    Ok((j, Node::Break(Ws(pws.is_some(), nws.is_some()))))
}

fn continue_statement<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    let mut p = tuple((opt(char('-')), ws(tag("continue")), opt(char('-'))));
    let (j, (pws, _, nws)) = p(i)?;
    if s.loop_depth.get() == 0 {
        return Err(nom::Err::Failure(error_position!(
            i,
            nom::error::ErrorKind::Tag
        )));
    }
    Ok((j, Node::Continue(Ws(pws.is_some(), nws.is_some()))))
}

fn block_node<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    let mut p = tuple((
        |i| tag_block_start(i, s),
        alt((
            block_call,
            block_let,
            |i| block_if(i, s),
            |i| block_for(i, s),
            |i| block_match(i, s),
            block_extends,
            block_include,
            block_import,
            |i| block_block(i, s),
            |i| block_macro(i, s),
            |i| block_raw(i, s),
            |i| break_statement(i, s),
            |i| continue_statement(i, s),
        )),
        cut(|i| tag_block_end(i, s)),
    ));
    let (i, (_, contents, _)) = p(i)?;
    Ok((i, contents))
}

fn block_comment_body<'a>(mut i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], &'a [u8]> {
    let mut level = 0;
    loop {
        let (end, tail) = take_until(s.syntax.comment_end)(i)?;
        match take_until::<_, _, Error<_>>(s.syntax.comment_start)(i) {
            Ok((start, _)) if start.as_ptr() < end.as_ptr() => {
                level += 1;
                i = &start[2..];
            }
            _ if level > 0 => {
                level -= 1;
                i = &end[2..];
            }
            _ => return Ok((end, tail)),
        }
    }
}

fn block_comment<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Node<'a>> {
    let mut p = tuple((
        |i| tag_comment_start(i, s),
        cut(tuple((
            opt(char('-')),
            |i| block_comment_body(i, s),
            |i| tag_comment_end(i, s),
        ))),
    ));
    let (i, (_, (pws, tail, _))) = p(i)?;
    Ok((i, Node::Comment(Ws(pws.is_some(), tail.ends_with(b"-")))))
}

fn parse_template<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], Vec<Node<'a>>> {
    many0(alt((
        complete(|i| take_content(i, s)),
        complete(|i| block_comment(i, s)),
        complete(|i| expr_node(i, s)),
        complete(|i| block_node(i, s)),
    )))(i)
}

fn tag_block_start<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], &'a [u8]> {
    tag(s.syntax.block_start)(i)
}
fn tag_block_end<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], &'a [u8]> {
    tag(s.syntax.block_end)(i)
}
fn tag_comment_start<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], &'a [u8]> {
    tag(s.syntax.comment_start)(i)
}
fn tag_comment_end<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], &'a [u8]> {
    tag(s.syntax.comment_end)(i)
}
fn tag_expr_start<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], &'a [u8]> {
    tag(s.syntax.expr_start)(i)
}
fn tag_expr_end<'a>(i: &'a [u8], s: &State<'_>) -> IResult<&'a [u8], &'a [u8]> {
    tag(s.syntax.expr_end)(i)
}

pub fn parse<'a>(src: &'a str, syntax: &'a Syntax<'a>) -> Result<Vec<Node<'a>>, CompileError> {
    let state = State {
        syntax,
        loop_depth: Cell::new(0),
    };
    match parse_template(src.as_bytes(), &state) {
        Ok((left, res)) => {
            if !left.is_empty() {
                let s = str::from_utf8(left).unwrap();
                Err(format!("unable to parse template:\n\n{:?}", s).into())
            } else {
                Ok(res)
            }
        }

        Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
            let nom::error::Error { input, .. } = err;
            let offset = src.len() - input.len();
            let (source_before, source_after) = src.split_at(offset);

            let source_after = match source_after.char_indices().enumerate().take(41).last() {
                Some((40, (i, _))) => format!("{:?}...", &source_after[..i]),
                _ => format!("{:?}", source_after),
            };

            let (row, last_line) = source_before.lines().enumerate().last().unwrap();
            let column = last_line.chars().count();

            let msg = format!(
                "problems parsing template source at row {}, column {} near:\n{}",
                row + 1,
                column,
                source_after,
            );
            Err(msg.into())
        }

        Err(nom::Err::Incomplete(_)) => Err("parsing incomplete".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{Expr, Node, Ws};
    use crate::Syntax;

    fn check_ws_split(s: &str, res: &(&str, &str, &str)) {
        match super::split_ws_parts(s.as_bytes()) {
            Node::Lit(lws, s, rws) => {
                assert_eq!(lws, res.0);
                assert_eq!(s, res.1);
                assert_eq!(rws, res.2);
            }
            _ => {
                panic!("fail");
            }
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

    #[test]
    #[should_panic]
    fn test_invalid_block() {
        super::parse("{% extend \"blah\" %}", &Syntax::default()).unwrap();
    }

    #[test]
    fn test_parse_filter() {
        use Expr::*;
        let syntax = Syntax::default();
        assert_eq!(
            super::parse("{{ strvar|e }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Filter("e", vec![Var("strvar")]),
            )],
        );
        assert_eq!(
            super::parse("{{ 2|abs }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Filter("abs", vec![NumLit("2")]),
            )],
        );
        assert_eq!(
            super::parse("{{ -2|abs }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Filter("abs", vec![Unary("-", NumLit("2").into())]),
            )],
        );
        assert_eq!(
            super::parse("{{ (1 - 2)|abs }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Filter(
                    "abs",
                    vec![Group(
                        BinOp("-", NumLit("1").into(), NumLit("2").into()).into()
                    )]
                ),
            )],
        );
    }

    #[test]
    fn test_parse_numbers() {
        let syntax = Syntax::default();
        assert_eq!(
            super::parse("{{ 2 }}", &syntax).unwrap(),
            vec![Node::Expr(Ws(false, false), Expr::NumLit("2"),)],
        );
        assert_eq!(
            super::parse("{{ 2.5 }}", &syntax).unwrap(),
            vec![Node::Expr(Ws(false, false), Expr::NumLit("2.5"),)],
        );
    }

    #[test]
    fn test_parse_var() {
        let s = Syntax::default();

        assert_eq!(
            super::parse("{{ foo }}", &s).unwrap(),
            vec![Node::Expr(Ws(false, false), Expr::Var("foo"))],
        );
        assert_eq!(
            super::parse("{{ foo_bar }}", &s).unwrap(),
            vec![Node::Expr(Ws(false, false), Expr::Var("foo_bar"))],
        );

        assert_eq!(
            super::parse("{{ none }}", &s).unwrap(),
            vec![Node::Expr(Ws(false, false), Expr::Var("none"))],
        );
    }

    #[test]
    fn test_parse_const() {
        let s = Syntax::default();

        assert_eq!(
            super::parse("{{ FOO }}", &s).unwrap(),
            vec![Node::Expr(Ws(false, false), Expr::Path(vec!["FOO"]))],
        );
        assert_eq!(
            super::parse("{{ FOO_BAR }}", &s).unwrap(),
            vec![Node::Expr(Ws(false, false), Expr::Path(vec!["FOO_BAR"]))],
        );

        assert_eq!(
            super::parse("{{ NONE }}", &s).unwrap(),
            vec![Node::Expr(Ws(false, false), Expr::Path(vec!["NONE"]))],
        );
    }

    #[test]
    fn test_parse_path() {
        let s = Syntax::default();

        assert_eq!(
            super::parse("{{ None }}", &s).unwrap(),
            vec![Node::Expr(Ws(false, false), Expr::Path(vec!["None"]))],
        );
        assert_eq!(
            super::parse("{{ Some(123) }}", &s).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Expr::PathCall(vec!["Some"], vec![Expr::NumLit("123")],),
            )],
        );

        assert_eq!(
            super::parse("{{ Ok(123) }}", &s).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Expr::PathCall(vec!["Ok"], vec![Expr::NumLit("123")],),
            )],
        );
        assert_eq!(
            super::parse("{{ Err(123) }}", &s).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Expr::PathCall(vec!["Err"], vec![Expr::NumLit("123")],),
            )],
        );
    }

    #[test]
    fn test_parse_var_call() {
        assert_eq!(
            super::parse("{{ function(\"123\", 3) }}", &Syntax::default()).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Expr::VarCall("function", vec![Expr::StrLit("123"), Expr::NumLit("3")]),
            )],
        );
    }

    #[test]
    fn test_parse_path_call() {
        let s = Syntax::default();

        assert_eq!(
            super::parse("{{ Option::None }}", &s).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Expr::Path(vec!["Option", "None"])
            )],
        );
        assert_eq!(
            super::parse("{{ Option::Some(123) }}", &s).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Expr::PathCall(vec!["Option", "Some"], vec![Expr::NumLit("123")],),
            )],
        );

        assert_eq!(
            super::parse("{{ self::function(\"123\", 3) }}", &s).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Expr::PathCall(
                    vec!["self", "function"],
                    vec![Expr::StrLit("123"), Expr::NumLit("3")],
                ),
            )],
        );
    }

    #[test]
    fn test_parse_root_path() {
        let syntax = Syntax::default();
        assert_eq!(
            super::parse("{{ std::string::String::new() }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Expr::PathCall(vec!["std", "string", "String", "new"], vec![]),
            )],
        );
        assert_eq!(
            super::parse("{{ ::std::string::String::new() }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                Expr::PathCall(vec!["", "std", "string", "String", "new"], vec![]),
            )],
        );
    }

    #[test]
    fn change_delimiters_parse_filter() {
        let syntax = Syntax {
            expr_start: "{~",
            expr_end: "~}",
            ..Syntax::default()
        };

        super::parse("{~ strvar|e ~}", &syntax).unwrap();
    }

    #[test]
    fn test_precedence() {
        use Expr::*;
        let syntax = Syntax::default();
        assert_eq!(
            super::parse("{{ a + b == c }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                BinOp(
                    "==",
                    BinOp("+", Var("a").into(), Var("b").into()).into(),
                    Var("c").into(),
                )
            )],
        );
        assert_eq!(
            super::parse("{{ a + b * c - d / e }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                BinOp(
                    "-",
                    BinOp(
                        "+",
                        Var("a").into(),
                        BinOp("*", Var("b").into(), Var("c").into()).into(),
                    )
                    .into(),
                    BinOp("/", Var("d").into(), Var("e").into()).into(),
                )
            )],
        );
        assert_eq!(
            super::parse("{{ a * (b + c) / -d }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                BinOp(
                    "/",
                    BinOp(
                        "*",
                        Var("a").into(),
                        Group(BinOp("+", Var("b").into(), Var("c").into()).into()).into()
                    )
                    .into(),
                    Unary("-", Var("d").into()).into()
                )
            )],
        );
        assert_eq!(
            super::parse("{{ a || b && c || d && e }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                BinOp(
                    "||",
                    BinOp(
                        "||",
                        Var("a").into(),
                        BinOp("&&", Var("b").into(), Var("c").into()).into(),
                    )
                    .into(),
                    BinOp("&&", Var("d").into(), Var("e").into()).into(),
                )
            )],
        );
    }

    #[test]
    fn test_associativity() {
        use Expr::*;
        let syntax = Syntax::default();
        assert_eq!(
            super::parse("{{ a + b + c }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                BinOp(
                    "+",
                    BinOp("+", Var("a").into(), Var("b").into()).into(),
                    Var("c").into()
                )
            )],
        );
        assert_eq!(
            super::parse("{{ a * b * c }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                BinOp(
                    "*",
                    BinOp("*", Var("a").into(), Var("b").into()).into(),
                    Var("c").into()
                )
            )],
        );
        assert_eq!(
            super::parse("{{ a && b && c }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                BinOp(
                    "&&",
                    BinOp("&&", Var("a").into(), Var("b").into()).into(),
                    Var("c").into()
                )
            )],
        );
        assert_eq!(
            super::parse("{{ a + b - c + d }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                BinOp(
                    "+",
                    BinOp(
                        "-",
                        BinOp("+", Var("a").into(), Var("b").into()).into(),
                        Var("c").into()
                    )
                    .into(),
                    Var("d").into()
                )
            )],
        );
        assert_eq!(
            super::parse("{{ a == b != c > d > e == f }}", &syntax).unwrap(),
            vec![Node::Expr(
                Ws(false, false),
                BinOp(
                    "==",
                    BinOp(
                        ">",
                        BinOp(
                            ">",
                            BinOp(
                                "!=",
                                BinOp("==", Var("a").into(), Var("b").into()).into(),
                                Var("c").into()
                            )
                            .into(),
                            Var("d").into()
                        )
                        .into(),
                        Var("e").into()
                    )
                    .into(),
                    Var("f").into()
                )
            )],
        );
    }

    #[test]
    fn test_parse_comments() {
        let s = &Syntax::default();

        assert_eq!(
            super::parse("{##}", s).unwrap(),
            vec![Node::Comment(Ws(false, false))],
        );
        assert_eq!(
            super::parse("{#- #}", s).unwrap(),
            vec![Node::Comment(Ws(true, false))],
        );
        assert_eq!(
            super::parse("{# -#}", s).unwrap(),
            vec![Node::Comment(Ws(false, true))],
        );
        assert_eq!(
            super::parse("{#--#}", s).unwrap(),
            vec![Node::Comment(Ws(true, true))],
        );

        assert_eq!(
            super::parse("{#- foo\n bar -#}", s).unwrap(),
            vec![Node::Comment(Ws(true, true))],
        );
        assert_eq!(
            super::parse("{#- foo\n {#- bar\n -#} baz -#}", s).unwrap(),
            vec![Node::Comment(Ws(true, true))],
        );
        assert_eq!(
            super::parse("{# foo {# bar #} {# {# baz #} qux #} #}", s).unwrap(),
            vec![Node::Comment(Ws(false, false))],
        );
    }
}
