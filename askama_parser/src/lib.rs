#![deny(unreachable_pub)]
#![deny(elided_lifetimes_in_paths)]

use std::borrow::Cow;
use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use std::{fmt, str};

use nom::branch::alt;
use nom::bytes::complete::{escaped, is_not, tag, take_till, take_while_m_n};
use nom::character::complete::{anychar, char, one_of, satisfy};
use nom::combinator::{cut, eof, map, opt, recognize};
use nom::error::{Error, ErrorKind, FromExternalError};
use nom::multi::{many0_count, many1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{error_position, AsChar, InputTakeAtPosition};

pub use rc_str::RcStr;

pub mod expr;
pub use expr::{Expr, Filter};
pub mod node;
pub use node::Node;
mod rc_str;

#[derive(Debug, Default)]
pub struct Ast {
    nodes: Vec<Node>,
}

impl Ast {
    /// If `file_path` is `None`, it means the `source` is an inline template. Therefore, if
    /// a parsing error occurs, we won't display the path as it wouldn't be useful.
    pub fn new(
        src: RcStr,
        file_path: Option<Rc<Path>>,
        syntax: &Syntax<'_>,
    ) -> Result<Self, ParseError> {
        let parse = |i: RcStr| Node::many(i, &State::new(syntax));
        let (input, message) = match terminated(parse, cut(eof))(src.clone()) {
            Ok((rest, nodes)) if rest.is_empty() => return Ok(Self { nodes }),
            Ok(_) => unreachable!("eof() is not eof?"),
            Err(
                nom::Err::Error(ErrorContext { input, message, .. })
                | nom::Err::Failure(ErrorContext { input, message, .. }),
            ) => (input, message),
            Err(nom::Err::Incomplete(_)) => return Err(ParseError("parsing incomplete".into())),
        };

        let offset = src.len() - input.len();
        let (source_before, source_after) = src.split_at(offset);

        let source_after = match source_after
            .as_str()
            .char_indices()
            .enumerate()
            .take(41)
            .last()
        {
            Some((40, (i, _))) => format!("{:?}...", &source_after.as_str()[..i]),
            _ => format!("{source_after:?}"),
        };

        let (row, last_line) = source_before
            .as_str()
            .lines()
            .enumerate()
            .last()
            .unwrap_or_default();
        let column = last_line.chars().count();

        let file_info = file_path.and_then(|file_path| {
            let cwd = std::env::current_dir().ok()?;
            Some((cwd, file_path))
        });
        let message = message
            .map(|message| format!("{message}\n"))
            .unwrap_or_default();
        let error_msg = if let Some((cwd, file_path)) = file_info {
            format!(
                "{message}failed to parse template source\n  --> {path}:{row}:{column}\n{source_after}",
                path = strip_common(&cwd, &file_path),
                row = row + 1,
            )
        } else {
            format!(
                "{message}failed to parse template source at row {}, column {column} near:\n{source_after}",
                row + 1,
            )
        };

        Err(ParseError(error_msg))
    }

    pub fn nodes(&self) -> &[Node] {
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

pub(crate) type ParseErr = nom::Err<ErrorContext>;
pub(crate) type ParseResult<T = RcStr> = Result<(RcStr, T), ParseErr>;

/// This type is used to handle `nom` errors and in particular to add custom error messages.
/// It used to generate `ParserError`.
///
/// It cannot be used to replace `ParseError` because it expects a generic, which would make
/// `askama`'s users experience less good (since this generic is only needed for `nom`).
#[derive(Debug)]
pub(crate) struct ErrorContext {
    pub(crate) input: RcStr,
    pub(crate) message: Option<Cow<'static, str>>,
}

impl nom::error::ParseError<RcStr> for ErrorContext {
    fn from_error_kind(input: RcStr, _code: ErrorKind) -> Self {
        Self {
            input,
            message: None,
        }
    }

    fn append(_: RcStr, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<E: std::fmt::Display> FromExternalError<RcStr, E> for ErrorContext {
    fn from_external_error(input: RcStr, _kind: ErrorKind, e: E) -> Self {
        Self {
            input,
            message: Some(Cow::Owned(e.to_string())),
        }
    }
}

impl ErrorContext {
    pub(crate) fn from_err(error: nom::Err<Error<RcStr>>) -> nom::Err<Self> {
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

fn ws<O>(inner: impl FnMut(RcStr) -> ParseResult<O>) -> impl FnMut(RcStr) -> ParseResult<O> {
    delimited(take_till(not_ws), inner, take_till(not_ws))
}

/// Skips input until `end` was found, but does not consume it.
/// Returns tuple that would be returned when parsing `end`.
fn skip_till<O>(
    end: impl FnMut(RcStr) -> ParseResult<O>,
) -> impl FnMut(RcStr) -> ParseResult<(RcStr, O)> {
    enum Next<O> {
        IsEnd(O),
        NotEnd,
    }
    let mut next = alt((map(end, Next::IsEnd), map(anychar, |_| Next::NotEnd)));
    move |start: RcStr| {
        let mut i = start;
        loop {
            let (j, is_end) = next(i.clone())?;
            match is_end {
                Next::IsEnd(lookahead) => return Ok((i, (j, lookahead))),
                Next::NotEnd => i = j,
            }
        }
    }
}

fn keyword(k: &str) -> impl FnMut(RcStr) -> ParseResult + '_ {
    move |i: RcStr| -> ParseResult {
        let (j, v) = identifier(i.clone())?;
        if v == k {
            Ok((j, v))
        } else {
            Err(nom::Err::Error(error_position!(i, ErrorKind::Tag)))
        }
    }
}

fn identifier(input: RcStr) -> ParseResult {
    fn start(s: RcStr) -> ParseResult {
        s.split_at_position1_complete(
            |c| !(c.is_alpha() || c == '_' || c >= '\u{0080}'),
            nom::error::ErrorKind::Alpha,
        )
    }

    fn tail(s: RcStr) -> ParseResult {
        s.split_at_position1_complete(
            |c| !(c.is_alphanum() || c == '_' || c >= '\u{0080}'),
            nom::error::ErrorKind::Alpha,
        )
    }

    recognize(pair(start, opt(tail)))(input)
}

fn bool_lit(i: RcStr) -> ParseResult {
    alt((keyword("false"), keyword("true")))(i)
}

fn num_lit(i: RcStr) -> ParseResult {
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
fn separated_digits(radix: u32, start: bool) -> impl Fn(RcStr) -> ParseResult {
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

fn str_lit(i: RcStr) -> ParseResult {
    let (i, s) = delimited(
        char('"'),
        opt(escaped(is_not("\\\""), '\\', anychar)),
        char('"'),
    )(i)?;
    Ok((i, s.unwrap_or_default()))
}

// Information about allowed character escapes is available at:
// <https://doc.rust-lang.org/reference/tokens.html#character-literals>.
fn char_lit(i: RcStr) -> ParseResult {
    let start = i.clone();
    let (i, s) = delimited(
        char('\''),
        opt(escaped(is_not("\\\'"), '\\', anychar)),
        char('\''),
    )(i)?;
    let Some(s) = s else {
        return Err(nom::Err::Failure(ErrorContext {
            input: start,
            // Same error as rustc.
            message: Some(Cow::Borrowed("empty character literal")),
        }));
    };
    let c = match Char::parse(s.clone()) {
        Ok((rest, c)) if rest.is_empty() => c,
        _ => {
            return Err(nom::Err::Failure(ErrorContext {
                input: start,
                message: Some(Cow::Borrowed("invalid character")),
            }))
        }
    };
    let (nb, max_value, err1, err2) = match c {
        Char::Literal | Char::Escaped => return Ok((i, s)),
        Char::AsciiEscape(nb) => (
            nb,
            // `0x7F` is the maximum value for a `\x` escaped character.
            0x7F,
            "invalid character in ascii escape",
            "must be a character in the range [\\x00-\\x7f]",
        ),
        Char::UnicodeEscape(nb) => (
            nb,
            // `0x10FFFF` is the maximum value for a `\u` escaped character.
            0x10FFFF,
            "invalid character in unicode escape",
            "unicode escape must be at most 10FFFF",
        ),
    };

    let Ok(nb) = u32::from_str_radix(nb.as_str(), 16) else {
        return Err(nom::Err::Failure(ErrorContext {
            input: start,
            message: Some(Cow::Borrowed(err1)),
        }));
    };
    if nb > max_value {
        return Err(nom::Err::Failure(ErrorContext {
            input: start,
            message: Some(Cow::Borrowed(err2)),
        }));
    }
    Ok((i, s))
}

/// Represents the different kinds of char declarations:
enum Char {
    /// Any character that is not escaped.
    Literal,
    /// An escaped character (like `\n`) which doesn't require any extra check.
    Escaped,
    /// Ascii escape (like `\x12`).
    AsciiEscape(RcStr),
    /// Unicode escape (like `\u{12}`).
    UnicodeEscape(RcStr),
}

impl Char {
    fn parse(i: RcStr) -> ParseResult<Self> {
        if i.as_str().chars().count() == 1 {
            return Ok((RcStr::default(), Self::Literal));
        }
        map(
            tuple((
                char('\\'),
                alt((
                    map(char('n'), |_| Self::Escaped),
                    map(char('r'), |_| Self::Escaped),
                    map(char('t'), |_| Self::Escaped),
                    map(char('\\'), |_| Self::Escaped),
                    map(char('0'), |_| Self::Escaped),
                    map(char('\''), |_| Self::Escaped),
                    // Not useful but supported by rust.
                    map(char('"'), |_| Self::Escaped),
                    map(
                        tuple((
                            char('x'),
                            take_while_m_n(2, 2, |c: char| c.is_ascii_hexdigit()),
                        )),
                        |(_, s)| Self::AsciiEscape(s),
                    ),
                    map(
                        tuple((
                            tag("u{"),
                            take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit()),
                            char('}'),
                        )),
                        |(_, s, _)| Self::UnicodeEscape(s),
                    ),
                )),
            )),
            |(_, ch)| ch,
        )(i)
    }
}

enum PathOrIdentifier {
    Path(Vec<RcStr>),
    Identifier(RcStr),
}

fn path_or_identifier(i: RcStr) -> ParseResult<PathOrIdentifier> {
    let root = ws(opt(tag("::")));
    let tail = opt(many1(preceded(ws(tag("::")), identifier)));

    let (i, (root, start, rest)) = tuple((root, identifier, tail))(i)?;
    let rest = rest.unwrap_or_default();

    // The returned identifier can be assumed to be path if:
    // - it is an absolute path (starts with `::`), or
    // - it has multiple components (at least one `::`), or
    // - the first letter is uppercase
    match (root, start, rest) {
        (Some(_), start, tail) => {
            let mut path = Vec::with_capacity(2 + tail.len());
            path.push(RcStr::default());
            path.push(start);
            path.extend(tail);
            Ok((i, PathOrIdentifier::Path(path)))
        }
        (None, name, tail)
            if tail.is_empty()
                && name
                    .as_str()
                    .chars()
                    .next()
                    .map_or(true, |c| c.is_lowercase()) =>
        {
            Ok((i, PathOrIdentifier::Identifier(name)))
        }
        (None, start, tail) => {
            let mut path = Vec::with_capacity(1 + tail.len());
            path.push(start);
            path.extend(tail);
            Ok((i, PathOrIdentifier::Path(path)))
        }
    }
}

struct State<'a> {
    syntax: &'a Syntax<'a>,
    loop_depth: Cell<usize>,
    level: Cell<Level>,
}

impl State<'_> {
    fn new<'a>(syntax: &'a Syntax<'a>) -> State<'a> {
        State {
            syntax,
            loop_depth: Cell::new(0),
            level: Cell::new(Level::default()),
        }
    }

    fn nest(&self, i: RcStr) -> ParseResult<()> {
        let (_, level) = self.level.get().nest(i.clone())?;
        self.level.set(level);
        Ok((i, ()))
    }

    fn leave(&self) {
        self.level.set(self.level.get().leave());
    }

    fn tag_block_start(&self, i: RcStr) -> ParseResult {
        tag(self.syntax.block_start)(i)
    }

    fn tag_block_end(&self, i: RcStr) -> ParseResult {
        tag(self.syntax.block_end)(i)
    }

    fn tag_comment_start(&self, i: RcStr) -> ParseResult {
        tag(self.syntax.comment_start)(i)
    }

    fn tag_comment_end(&self, i: RcStr) -> ParseResult {
        tag(self.syntax.comment_end)(i)
    }

    fn tag_expr_start(&self, i: RcStr) -> ParseResult {
        tag(self.syntax.expr_start)(i)
    }

    fn tag_expr_end(&self, i: RcStr) -> ParseResult {
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
    fn nest(self, i: RcStr) -> ParseResult<Level> {
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

fn filter(i: RcStr, level: Level) -> ParseResult<(RcStr, Option<Vec<Expr>>)> {
    let (i, (_, fname, args)) = tuple((
        char('|'),
        ws(identifier),
        opt(|i| Expr::arguments(i, level, false)),
    ))(i)?;
    Ok((i, (fname, args)))
}

/// Returns the common parts of two paths.
///
/// The goal of this function is to reduce the path length based on the `base` argument
/// (generally the path where the program is running into). For example:
///
/// ```text
/// current dir: /a/b/c
/// path:        /a/b/c/d/e.txt
/// ```
///
/// `strip_common` will return `d/e.txt`.
fn strip_common(base: &Path, path: &Path) -> String {
    let path = match path.canonicalize() {
        Ok(path) => path,
        Err(_) => return path.display().to_string(),
    };
    let mut components_iter = path.components().peekable();

    for current_path_component in base.components() {
        let Some(path_component) = components_iter.peek() else {
            return path.display().to_string();
        };
        if current_path_component != *path_component {
            break;
        }
        components_iter.next();
    }
    let path_parts = components_iter
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>();
    if path_parts.is_empty() {
        path.display().to_string()
    } else {
        path_parts.join("/")
    }
}
