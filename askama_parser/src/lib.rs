#![deny(unreachable_pub)]
#![deny(elided_lifetimes_in_paths)]

use std::borrow::Cow;
use std::cell::Cell;
use std::{fmt, str};

use nom::branch::alt;
use nom::bytes::complete::{escaped, is_not, tag, take_till};
use nom::character::complete::{anychar, char, one_of, satisfy};
use nom::combinator::{cut, eof, map, opt, recognize};
use nom::error::{Error, ErrorKind, FromExternalError};
use nom::multi::{many0_count, many1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{error_position, AsChar, InputTakeAtPosition};

pub mod expr;
pub use expr::Expr;
pub mod node;
pub use node::Node;
#[cfg(test)]
mod tests;

mod _parsed {
    use std::cmp::PartialEq;
    use std::{fmt, mem};

    use super::node::Node;
    use super::{Ast, ParseError, Syntax};

    #[derive(Default)]
    pub struct Parsed {
        // `source` must outlive `ast`, so `ast` must be declared before `source`
        ast: Ast<'static>,
        #[allow(dead_code)]
        source: String,
    }

    impl Parsed {
        pub fn new(source: String, syntax: &Syntax<'_>) -> Result<Self, ParseError> {
            // Self-referential borrowing: `self` will keep the source alive as `String`,
            // internally we will transmute it to `&'static str` to satisfy the compiler.
            // However, we only expose the nodes with a lifetime limited to `self`.
            let src = unsafe { mem::transmute::<&str, &'static str>(source.as_str()) };
            let ast = Ast::from_str(src, syntax)?;
            Ok(Self { ast, source })
        }

        // The return value's lifetime must be limited to `self` to uphold the unsafe invariant.
        pub fn nodes(&self) -> &[Node<'_>] {
            &self.ast.nodes
        }
    }

    impl fmt::Debug for Parsed {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Parsed")
                .field("nodes", &self.ast.nodes)
                .finish_non_exhaustive()
        }
    }

    impl PartialEq for Parsed {
        fn eq(&self, other: &Self) -> bool {
            self.ast.nodes == other.ast.nodes
        }
    }
}

pub use _parsed::Parsed;

#[derive(Debug, Default)]
pub struct Ast<'a> {
    nodes: Vec<Node<'a>>,
}

impl<'a> Ast<'a> {
    pub fn from_str(src: &'a str, syntax: &Syntax<'_>) -> Result<Self, ParseError> {
        let parse = |i: &'a str| Node::many(i, &State::new(syntax));
        let (input, message) = match terminated(parse, cut(eof))(src) {
            Ok(("", nodes)) => return Ok(Self { nodes }),
            Ok(_) => unreachable!("eof() is not eof?"),
            Err(
                nom::Err::Error(ErrorContext { input, message, .. })
                | nom::Err::Failure(ErrorContext { input, message, .. }),
            ) => (input, message),
            Err(nom::Err::Incomplete(_)) => return Err(ParseError("parsing incomplete".into())),
        };

        let offset = src.len() - input.len();
        let (source_before, source_after) = src.split_at(offset);

        let source_after = match source_after.char_indices().enumerate().take(41).last() {
            Some((40, (i, _))) => format!("{:?}...", &source_after[..i]),
            _ => format!("{source_after:?}"),
        };

        let (row, last_line) = source_before.lines().enumerate().last().unwrap_or_default();
        let column = last_line.chars().count();

        let msg = format!(
            "{}problems parsing template source at row {}, column {} near:\n{}",
            if let Some(message) = message {
                format!("{message}\n")
            } else {
                String::new()
            },
            row + 1,
            column,
            source_after,
        );

        Err(ParseError(msg))
    }

    pub fn nodes(&self) -> &[Node<'a>] {
        &self.nodes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(String);

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub(crate) type ParseResult<'a, T = &'a str> = Result<(&'a str, T), nom::Err<ErrorContext<'a>>>;

/// This type is used to handle `nom` errors and in particular to add custom error messages.
/// It used to generate `ParserError`.
///
/// It cannot be used to replace `ParseError` because it expects a generic, which would make
/// `askama`'s users experience less good (since this generic is only needed for `nom`).
#[derive(Debug)]
pub(crate) struct ErrorContext<'a> {
    pub(crate) input: &'a str,
    pub(crate) message: Option<Cow<'static, str>>,
}

impl<'a> nom::error::ParseError<&'a str> for ErrorContext<'a> {
    fn from_error_kind(input: &'a str, _code: ErrorKind) -> Self {
        Self {
            input,
            message: None,
        }
    }

    fn append(_: &'a str, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a, E: std::fmt::Display> FromExternalError<&'a str, E> for ErrorContext<'a> {
    fn from_external_error(input: &'a str, _kind: ErrorKind, e: E) -> Self {
        Self {
            input,
            message: Some(Cow::Owned(e.to_string())),
        }
    }
}

impl<'a> ErrorContext<'a> {
    pub(crate) fn from_err(error: nom::Err<Error<&'a str>>) -> nom::Err<Self> {
        match error {
            nom::Err::Incomplete(i) => nom::Err::Incomplete(i),
            nom::Err::Failure(Error { input, .. }) => nom::Err::Failure(Self {
                input,
                message: None,
            }),
            nom::Err::Error(Error { input, .. }) => nom::Err::Error(Self {
                input,
                message: None,
            }),
        }
    }
}

fn is_ws(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}

fn not_ws(c: char) -> bool {
    !is_ws(c)
}

fn ws<'a, O>(
    inner: impl FnMut(&'a str) -> ParseResult<'a, O>,
) -> impl FnMut(&'a str) -> ParseResult<'a, O> {
    delimited(take_till(not_ws), inner, take_till(not_ws))
}

/// Skips input until `end` was found, but does not consume it.
/// Returns tuple that would be returned when parsing `end`.
fn skip_till<'a, O>(
    end: impl FnMut(&'a str) -> ParseResult<'a, O>,
) -> impl FnMut(&'a str) -> ParseResult<'a, (&'a str, O)> {
    enum Next<O> {
        IsEnd(O),
        NotEnd(char),
    }
    let mut next = alt((map(end, Next::IsEnd), map(anychar, Next::NotEnd)));
    move |start: &'a str| {
        let mut i = start;
        loop {
            let (j, is_end) = next(i)?;
            match is_end {
                Next::IsEnd(lookahead) => return Ok((i, (j, lookahead))),
                Next::NotEnd(_) => i = j,
            }
        }
    }
}

fn keyword<'a>(k: &'a str) -> impl FnMut(&'a str) -> ParseResult<'_> {
    move |i: &'a str| -> ParseResult<'a> {
        let (j, v) = identifier(i)?;
        if k == v {
            Ok((j, v))
        } else {
            Err(nom::Err::Error(error_position!(i, ErrorKind::Tag)))
        }
    }
}

fn identifier(input: &str) -> ParseResult<'_> {
    fn start(s: &str) -> ParseResult<'_> {
        s.split_at_position1_complete(
            |c| !(c.is_alpha() || c == '_' || c >= '\u{0080}'),
            nom::error::ErrorKind::Alpha,
        )
    }

    fn tail(s: &str) -> ParseResult<'_> {
        s.split_at_position1_complete(
            |c| !(c.is_alphanum() || c == '_' || c >= '\u{0080}'),
            nom::error::ErrorKind::Alpha,
        )
    }

    recognize(pair(start, opt(tail)))(input)
}

fn bool_lit(i: &str) -> ParseResult<'_> {
    alt((keyword("false"), keyword("true")))(i)
}

fn num_lit(i: &str) -> ParseResult<'_> {
    let integer_suffix = |i| {
        alt((
            tag("i8"),
            tag("i16"),
            tag("i32"),
            tag("i64"),
            tag("i128"),
            tag("isize"),
            tag("u8"),
            tag("u16"),
            tag("u32"),
            tag("u64"),
            tag("u128"),
            tag("usize"),
        ))(i)
    };
    let float_suffix = |i| alt((tag("f32"), tag("f64")))(i);

    recognize(tuple((
        opt(char('-')),
        alt((
            recognize(tuple((
                char('0'),
                alt((
                    recognize(tuple((char('b'), separated_digits(2, false)))),
                    recognize(tuple((char('o'), separated_digits(8, false)))),
                    recognize(tuple((char('x'), separated_digits(16, false)))),
                )),
                opt(integer_suffix),
            ))),
            recognize(tuple((
                separated_digits(10, true),
                opt(alt((
                    integer_suffix,
                    float_suffix,
                    recognize(tuple((
                        opt(tuple((char('.'), separated_digits(10, true)))),
                        one_of("eE"),
                        opt(one_of("+-")),
                        separated_digits(10, false),
                        opt(float_suffix),
                    ))),
                    recognize(tuple((
                        char('.'),
                        separated_digits(10, true),
                        opt(float_suffix),
                    ))),
                ))),
            ))),
        )),
    )))(i)
}

/// Underscore separated digits of the given base, unless `start` is true this may start
/// with an underscore.
fn separated_digits(radix: u32, start: bool) -> impl Fn(&str) -> ParseResult<'_> {
    move |i| {
        recognize(tuple((
            |i| match start {
                true => Ok((i, 0)),
                false => many0_count(char('_'))(i),
            },
            satisfy(|ch| ch.is_digit(radix)),
            many0_count(satisfy(|ch| ch == '_' || ch.is_digit(radix))),
        )))(i)
    }
}

fn str_lit(i: &str) -> ParseResult<'_> {
    let (i, s) = delimited(
        char('"'),
        opt(escaped(is_not("\\\""), '\\', anychar)),
        char('"'),
    )(i)?;
    Ok((i, s.unwrap_or_default()))
}

fn char_lit(i: &str) -> ParseResult<'_> {
    let (i, s) = delimited(
        char('\''),
        opt(escaped(is_not("\\\'"), '\\', anychar)),
        char('\''),
    )(i)?;
    Ok((i, s.unwrap_or_default()))
}

enum PathOrIdentifier<'a> {
    Path(Vec<&'a str>),
    Identifier(&'a str),
}

fn path_or_identifier(i: &str) -> ParseResult<'_, PathOrIdentifier<'_>> {
    let root = ws(opt(tag("::")));
    let tail = opt(many1(preceded(ws(tag("::")), identifier)));

    let (i, (root, start, rest)) = tuple((root, identifier, tail))(i)?;
    let rest = rest.as_deref().unwrap_or_default();

    // The returned identifier can be assumed to be path if:
    // - it is an absolute path (starts with `::`), or
    // - it has multiple components (at least one `::`), or
    // - the first letter is uppercase
    match (root, start, rest) {
        (Some(_), start, tail) => {
            let mut path = Vec::with_capacity(2 + tail.len());
            path.push("");
            path.push(start);
            path.extend(rest);
            Ok((i, PathOrIdentifier::Path(path)))
        }
        (None, name, []) if name.chars().next().map_or(true, |c| c.is_lowercase()) => {
            Ok((i, PathOrIdentifier::Identifier(name)))
        }
        (None, start, tail) => {
            let mut path = Vec::with_capacity(1 + tail.len());
            path.push(start);
            path.extend(rest);
            Ok((i, PathOrIdentifier::Path(path)))
        }
    }
}

struct State<'a> {
    syntax: &'a Syntax<'a>,
    loop_depth: Cell<usize>,
    level: Cell<Level>,
}

impl<'a> State<'a> {
    fn new(syntax: &'a Syntax<'a>) -> State<'a> {
        State {
            syntax,
            loop_depth: Cell::new(0),
            level: Cell::new(Level::default()),
        }
    }

    fn nest<'b>(&self, i: &'b str) -> ParseResult<'b, ()> {
        let (_, level) = self.level.get().nest(i)?;
        self.level.set(level);
        Ok((i, ()))
    }

    fn leave(&self) {
        self.level.set(self.level.get().leave());
    }

    fn tag_block_start<'i>(&self, i: &'i str) -> ParseResult<'i> {
        tag(self.syntax.block_start)(i)
    }

    fn tag_block_end<'i>(&self, i: &'i str) -> ParseResult<'i> {
        tag(self.syntax.block_end)(i)
    }

    fn tag_comment_start<'i>(&self, i: &'i str) -> ParseResult<'i> {
        tag(self.syntax.comment_start)(i)
    }

    fn tag_comment_end<'i>(&self, i: &'i str) -> ParseResult<'i> {
        tag(self.syntax.comment_end)(i)
    }

    fn tag_expr_start<'i>(&self, i: &'i str) -> ParseResult<'i> {
        tag(self.syntax.expr_start)(i)
    }

    fn tag_expr_end<'i>(&self, i: &'i str) -> ParseResult<'i> {
        tag(self.syntax.expr_end)(i)
    }

    fn enter_loop(&self) {
        self.loop_depth.set(self.loop_depth.get() + 1);
    }

    fn leave_loop(&self) {
        self.loop_depth.set(self.loop_depth.get() - 1);
    }

    fn is_in_loop(&self) -> bool {
        self.loop_depth.get() > 0
    }
}

#[derive(Debug)]
pub struct Syntax<'a> {
    pub block_start: &'a str,
    pub block_end: &'a str,
    pub expr_start: &'a str,
    pub expr_end: &'a str,
    pub comment_start: &'a str,
    pub comment_end: &'a str,
}

impl Default for Syntax<'static> {
    fn default() -> Self {
        Self {
            block_start: "{%",
            block_end: "%}",
            expr_start: "{{",
            expr_end: "}}",
            comment_start: "{#",
            comment_end: "#}",
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Level(u8);

impl Level {
    fn nest(self, i: &str) -> ParseResult<'_, Level> {
        if self.0 >= Self::MAX_DEPTH {
            return Err(ErrorContext::from_err(nom::Err::Failure(error_position!(
                i,
                ErrorKind::TooLarge
            ))));
        }

        Ok((i, Level(self.0 + 1)))
    }

    fn leave(&self) -> Self {
        Level(self.0 - 1)
    }

    const MAX_DEPTH: u8 = 128;
}
