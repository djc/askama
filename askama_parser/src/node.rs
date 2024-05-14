use std::borrow::Cow;
use std::str;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{
    complete, consumed, cut, eof, map, map_res, not, opt, peek, recognize, value,
};
use nom::error::{Error, ErrorKind};
use nom::error_position;
use nom::multi::{fold_many0, many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};

use crate::{ErrorContext, ParseResult, RcStr};

use super::{
    bool_lit, char_lit, filter, identifier, is_ws, keyword, num_lit, path_or_identifier, skip_till,
    str_lit, ws, Expr, Filter, PathOrIdentifier, State,
};

#[derive(Debug, PartialEq)]
pub enum Node {
    Lit(Lit),
    Comment(Comment),
    Expr(Ws, Expr),
    Call(Call),
    Let(Let),
    If(If),
    Match(Match),
    Loop(Box<Loop>),
    Extends(Extends),
    BlockDef(BlockDef),
    Include(Include),
    Import(Import),
    Macro(Macro),
    Raw(Raw),
    Break(Ws),
    Continue(Ws),
    FilterBlock(FilterBlock),
}

impl Node {
    pub(super) fn many(i: RcStr, s: &State<'_>) -> ParseResult<Vec<Self>> {
        complete(many0(alt((
            map(|i| Lit::parse(i, s), Self::Lit),
            map(|i| Comment::parse(i, s), Self::Comment),
            |i| Self::expr(i, s),
            |i| Self::parse(i, s),
        ))))(i)
    }

    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = delimited(
            |i| s.tag_block_start(i),
            alt((
                map(|i| Call::parse(i, s), Self::Call),
                map(|i| Let::parse(i, s), Self::Let),
                map(|i| If::parse(i, s), Self::If),
                map(|i| Loop::parse(i, s), |l| Self::Loop(Box::new(l))),
                map(|i| Match::parse(i, s), Self::Match),
                map(Extends::parse, Self::Extends),
                map(Include::parse, Self::Include),
                map(Import::parse, Self::Import),
                map(|i| BlockDef::parse(i, s), Self::BlockDef),
                map(|i| Macro::parse(i, s), Self::Macro),
                map(|i| Raw::parse(i, s), Self::Raw),
                |i| Self::r#break(i, s),
                |i| Self::r#continue(i, s),
                map(|i| FilterBlock::parse(i, s), Self::FilterBlock),
            )),
            cut(|i| s.tag_block_end(i)),
        );

        let (i, _) = s.nest(i)?;
        let result = p(i);
        s.leave();

        result
    }

    fn r#break(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = tuple((
            opt(Whitespace::parse),
            ws(keyword("break")),
            opt(Whitespace::parse),
        ));
        let (j, (pws, _, nws)) = p(i.clone())?;
        if !s.is_in_loop() {
            return Err(nom::Err::Failure(error_position!(i, ErrorKind::Tag)));
        }
        Ok((j, Self::Break(Ws(pws, nws))))
    }

    fn r#continue(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = tuple((
            opt(Whitespace::parse),
            ws(keyword("continue")),
            opt(Whitespace::parse),
        ));
        let (j, (pws, _, nws)) = p(i.clone())?;
        if !s.is_in_loop() {
            return Err(nom::Err::Failure(error_position!(i, ErrorKind::Tag)));
        }
        Ok((j, Self::Continue(Ws(pws, nws))))
    }

    fn expr(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = tuple((
            |i| s.tag_expr_start(i),
            cut(tuple((
                opt(Whitespace::parse),
                ws(|i| Expr::parse(i, s.level.get())),
                opt(Whitespace::parse),
                |i| s.tag_expr_end(i),
            ))),
        ));
        let (i, (_, (pws, expr, nws, _))) = p(i)?;
        Ok((i, Self::Expr(Ws(pws, nws), expr)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Placeholder,
    Name(RcStr),
    Tuple(Vec<RcStr>, Vec<Target>),
    Struct(Vec<RcStr>, Vec<(RcStr, Target)>),
    NumLit(RcStr),
    StrLit(RcStr),
    CharLit(RcStr),
    BoolLit(RcStr),
    Path(Vec<RcStr>),
    OrChain(Vec<Target>),
}

impl Target {
    /// Parses multiple targets with `or` separating them
    pub(super) fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        map(
            separated_list1(ws(tag("or")), |i| {
                let (i, _) = s.nest(i)?;
                let ret = Self::parse_one(i, s)?;
                s.leave();
                Ok(ret)
            }),
            |mut opts| match opts.len() {
                1 => opts.pop().unwrap(),
                _ => Self::OrChain(opts),
            },
        )(i)
    }

    /// Parses a single target without an `or`, unless it is wrapped in parentheses.
    fn parse_one(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut opt_opening_paren = map(opt(ws(char('('))), |o| o.is_some());
        let mut opt_closing_paren = map(opt(ws(char(')'))), |o| o.is_some());
        let mut opt_opening_brace = map(opt(ws(char('{'))), |o| o.is_some());

        let (i, lit) = opt(Self::lit)(i)?;
        if let Some(lit) = lit {
            return Ok((i, lit));
        }

        // match tuples and unused parentheses
        let (i, target_is_tuple) = opt_opening_paren(i)?;
        if target_is_tuple {
            let (i, is_empty_tuple) = opt_closing_paren(i)?;
            if is_empty_tuple {
                return Ok((i, Self::Tuple(Vec::new(), Vec::new())));
            }

            let (i, first_target) = Self::parse(i, s)?;
            let (i, is_unused_paren) = opt_closing_paren(i)?;
            if is_unused_paren {
                return Ok((i, first_target));
            }

            let mut targets = vec![first_target];
            let (i, _) = cut(tuple((
                fold_many0(
                    preceded(ws(char(',')), |i| Self::parse(i, s)),
                    || (),
                    |_, target| {
                        targets.push(target);
                    },
                ),
                opt(ws(char(','))),
                ws(cut(char(')'))),
            )))(i)?;
            return Ok((i, Self::Tuple(Vec::new(), targets)));
        }

        let path = |i| {
            map_res(path_or_identifier, |v| match v {
                PathOrIdentifier::Path(v) => Ok(v),
                PathOrIdentifier::Identifier(v) => Err(v),
            })(i)
        };

        // match structs
        let (i, path) = opt(path)(i)?;
        if let Some(path) = path {
            let i_before_matching_with = i.clone();
            let (i, _) = opt(ws(keyword("with")))(i)?;

            let (i, is_unnamed_struct) = opt_opening_paren(i)?;
            if is_unnamed_struct {
                let (i, targets) = alt((
                    map(char(')'), |_| Vec::new()),
                    terminated(
                        cut(separated_list1(ws(char(',')), |i| Self::parse(i, s))),
                        pair(opt(ws(char(','))), ws(cut(char(')')))),
                    ),
                ))(i)?;
                return Ok((i, Self::Tuple(path, targets)));
            }

            let (i, is_named_struct) = opt_opening_brace(i)?;
            if is_named_struct {
                let (i, targets) = alt((
                    map(char('}'), |_| Vec::new()),
                    terminated(
                        cut(separated_list1(ws(char(',')), |i| Self::named(i, s))),
                        pair(opt(ws(char(','))), ws(cut(char('}')))),
                    ),
                ))(i)?;
                return Ok((i, Self::Struct(path, targets)));
            }

            return Ok((i_before_matching_with, Self::Path(path)));
        }

        // neither literal nor struct nor path
        let (new_i, name) = identifier(i.clone())?;
        Ok((new_i, Self::verify_name(i, name)?))
    }

    fn lit(i: RcStr) -> ParseResult<Self> {
        alt((
            map(str_lit, Self::StrLit),
            map(char_lit, Self::CharLit),
            map(num_lit, Self::NumLit),
            map(bool_lit, Self::BoolLit),
        ))(i)
    }

    fn named(init_i: RcStr, s: &State<'_>) -> ParseResult<(RcStr, Self)> {
        let (i, (src, target)) = pair(
            identifier,
            opt(preceded(ws(char(':')), |i| Self::parse(i, s))),
        )(init_i.clone())?;

        let target = match target {
            Some(target) => target,
            None => Self::verify_name(init_i, src.clone())?,
        };

        Ok((i, (src, target)))
    }

    fn verify_name(input: RcStr, name: RcStr) -> Result<Self, nom::Err<ErrorContext>> {
        match name.as_str() {
            "self" | "writer" => Err(nom::Err::Failure(ErrorContext {
                input,
                message: Some(Cow::Owned(format!("Cannot use `{name}` as a name"))),
            })),
            _ => Ok(Self::Name(name)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct When {
    pub ws: Ws,
    pub target: Target,
    pub nodes: Vec<Node>,
}

impl When {
    fn r#match(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = tuple((
            |i| s.tag_block_start(i),
            opt(Whitespace::parse),
            ws(keyword("else")),
            cut(tuple((
                opt(Whitespace::parse),
                |i| s.tag_block_end(i),
                cut(|i| Node::many(i, s)),
            ))),
        ));
        let (i, (_, pws, _, (nws, _, nodes))) = p(i)?;
        Ok((
            i,
            Self {
                ws: Ws(pws, nws),
                target: Target::Placeholder,
                nodes,
            },
        ))
    }

    #[allow(clippy::self_named_constructors)]
    fn when(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = tuple((
            |i| s.tag_block_start(i),
            opt(Whitespace::parse),
            ws(keyword("when")),
            cut(tuple((
                ws(|i| Target::parse(i, s)),
                opt(Whitespace::parse),
                |i| s.tag_block_end(i),
                cut(|i| Node::many(i, s)),
            ))),
        ));
        let (i, (_, pws, _, (target, nws, _, nodes))) = p(i)?;
        Ok((
            i,
            Self {
                ws: Ws(pws, nws),
                target,
                nodes,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct Cond {
    pub ws: Ws,
    pub cond: Option<CondTest>,
    pub nodes: Vec<Node>,
}

impl Cond {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = tuple((
            |i| s.tag_block_start(i),
            opt(Whitespace::parse),
            ws(alt((keyword("else"), |i: RcStr| {
                keyword("elif")(i.clone())?;
                Err(nom::Err::Failure(ErrorContext {
                    input: i,
                    message: Some(Cow::Borrowed(
                        "unknown `elif` keyword; did you mean `else if`?",
                    )),
                }))
            }))),
            cut(tuple((
                opt(|i| CondTest::parse(i, s)),
                opt(Whitespace::parse),
                |i| s.tag_block_end(i),
                cut(|i| Node::many(i, s)),
            ))),
        ));
        let (i, (_, pws, _, (cond, nws, _, nodes))) = p(i)?;
        Ok((
            i,
            Self {
                ws: Ws(pws, nws),
                cond,
                nodes,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct CondTest {
    pub target: Option<Target>,
    pub expr: Expr,
}

impl CondTest {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = preceded(
            ws(keyword("if")),
            cut(tuple((
                opt(delimited(
                    ws(alt((keyword("let"), keyword("set")))),
                    ws(|i| Target::parse(i, s)),
                    ws(char('=')),
                )),
                ws(|i| Expr::parse(i, s.level.get())),
            ))),
        );
        let (i, (target, expr)) = p(i)?;
        Ok((i, Self { target, expr }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Whitespace {
    Preserve,
    Suppress,
    Minimize,
}

impl Whitespace {
    fn parse(i: RcStr) -> ParseResult<Self> {
        alt((
            value(Self::Preserve, char('+')),
            value(Self::Suppress, char('-')),
            value(Self::Minimize, char('~')),
        ))(i)
    }
}

#[derive(Debug, PartialEq)]
pub struct Loop {
    pub ws1: Ws,
    pub var: Target,
    pub iter: Expr,
    pub cond: Option<Expr>,
    pub body: Vec<Node>,
    pub ws2: Ws,
    pub else_nodes: Vec<Node>,
    pub ws3: Ws,
}

impl Loop {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        fn content(i: RcStr, s: &State<'_>) -> ParseResult<Vec<Node>> {
            s.enter_loop();
            let result = Node::many(i, s);
            s.leave_loop();
            result
        }

        let if_cond = preceded(
            ws(keyword("if")),
            cut(ws(|i| Expr::parse(i, s.level.get()))),
        );

        let else_block = |i| {
            let mut p = preceded(
                ws(keyword("else")),
                cut(tuple((
                    opt(Whitespace::parse),
                    delimited(
                        |i| s.tag_block_end(i),
                        |i| Node::many(i, s),
                        |i| s.tag_block_start(i),
                    ),
                    opt(Whitespace::parse),
                ))),
            );
            let (i, (pws, nodes, nws)) = p(i)?;
            Ok((i, (pws, nodes, nws)))
        };

        let mut p = tuple((
            opt(Whitespace::parse),
            ws(keyword("for")),
            cut(tuple((
                ws(|i| Target::parse(i, s)),
                ws(keyword("in")),
                cut(tuple((
                    ws(|i| Expr::parse(i, s.level.get())),
                    opt(if_cond),
                    opt(Whitespace::parse),
                    |i| s.tag_block_end(i),
                    cut(tuple((
                        |i| content(i, s),
                        cut(tuple((
                            |i| s.tag_block_start(i),
                            opt(Whitespace::parse),
                            opt(else_block),
                            ws(keyword("endfor")),
                            opt(Whitespace::parse),
                        ))),
                    ))),
                ))),
            ))),
        ));
        let (i, (pws1, _, (var, _, (iter, cond, nws1, _, (body, (_, pws2, else_block, _, nws2)))))) =
            p(i)?;
        let (nws3, else_block, pws3) = else_block.unwrap_or_default();
        Ok((
            i,
            Self {
                ws1: Ws(pws1, nws1),
                var,
                iter,
                cond,
                body,
                ws2: Ws(pws2, nws3),
                else_nodes: else_block,
                ws3: Ws(pws3, nws2),
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct Macro {
    pub ws1: Ws,
    pub name: RcStr,
    pub args: Vec<RcStr>,
    pub nodes: Vec<Node>,
    pub ws2: Ws,
}

impl Macro {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        fn parameters(i: RcStr) -> ParseResult<Vec<RcStr>> {
            delimited(
                ws(char('(')),
                separated_list0(char(','), ws(identifier)),
                tuple((opt(ws(char(','))), char(')'))),
            )(i)
        }

        let mut start = tuple((
            opt(Whitespace::parse),
            ws(keyword("macro")),
            cut(tuple((
                ws(identifier),
                opt(ws(parameters)),
                opt(Whitespace::parse),
                |i| s.tag_block_end(i),
            ))),
        ));
        let (i, (pws1, _, (name, params, nws1, _))) = start(i)?;

        let mut end = cut(tuple((
            |i| Node::many(i, s),
            cut(tuple((
                |i| s.tag_block_start(i),
                opt(Whitespace::parse),
                ws(keyword("endmacro")),
                cut(preceded(
                    opt(|before: RcStr| {
                        let (after, end_name) = ws(identifier)(before.clone())?;
                        check_end_name(before, after, name.clone(), end_name, "macro")
                    }),
                    opt(Whitespace::parse),
                )),
            ))),
        )));
        let (i, (contents, (_, pws2, _, nws2))) = end(i)?;

        if name == "super" {
            // TODO: yield a a better error message here
            return Err(ErrorContext::from_err(nom::Err::Failure(Error::new(
                i,
                ErrorKind::Fail,
            ))));
        }

        Ok((
            i,
            Self {
                ws1: Ws(pws1, nws1),
                name: name.clone(),
                args: params.unwrap_or_default(),
                nodes: contents,
                ws2: Ws(pws2, nws2),
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct FilterBlock {
    pub ws1: Ws,
    pub filters: Filter,
    pub nodes: Vec<Node>,
    pub ws2: Ws,
}

impl FilterBlock {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut start = tuple((
            opt(Whitespace::parse),
            ws(keyword("filter")),
            cut(tuple((
                ws(identifier),
                opt(|i| Expr::arguments(i, s.level.get(), false)),
                many0(|i| filter(i, s.level.get())),
                ws(|i| Ok((i, ()))),
                opt(Whitespace::parse),
                |i| s.tag_block_end(i),
            ))),
        ));
        let (i, (pws1, _, (filter_name, params, extra_filters, _, nws1, _))) = start(i)?;

        let mut filters = Filter {
            name: filter_name,
            arguments: params.unwrap_or_default(),
        };
        for (filter_name, args) in extra_filters {
            filters = Filter {
                name: filter_name,
                arguments: {
                    let mut args = args.unwrap_or_default();
                    args.insert(0, Expr::Filter(filters));
                    args
                },
            };
        }

        let mut end = cut(tuple((
            |i| Node::many(i, s),
            cut(tuple((
                |i| s.tag_block_start(i),
                opt(Whitespace::parse),
                ws(keyword("endfilter")),
                opt(Whitespace::parse),
            ))),
        )));
        let (i, (nodes, (_, pws2, _, nws2))) = end(i)?;

        Ok((
            i,
            Self {
                ws1: Ws(pws1, nws1),
                filters,
                nodes,
                ws2: Ws(pws2, nws2),
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct Import {
    pub ws: Ws,
    pub path: RcStr,
    pub scope: RcStr,
}

impl Import {
    fn parse(i: RcStr) -> ParseResult<Self> {
        let mut p = tuple((
            opt(Whitespace::parse),
            ws(keyword("import")),
            cut(tuple((
                ws(str_lit),
                ws(keyword("as")),
                cut(pair(ws(identifier), opt(Whitespace::parse))),
            ))),
        ));
        let (i, (pws, _, (path, _, (scope, nws)))) = p(i)?;
        Ok((
            i,
            Self {
                ws: Ws(pws, nws),
                path,
                scope,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct Call {
    pub ws: Ws,
    pub scope: Option<RcStr>,
    pub name: RcStr,
    pub args: Vec<Expr>,
}

impl Call {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = tuple((
            opt(Whitespace::parse),
            ws(keyword("call")),
            cut(tuple((
                opt(tuple((ws(identifier), ws(tag("::"))))),
                ws(identifier),
                opt(ws(|nested| Expr::arguments(nested, s.level.get(), true))),
                opt(Whitespace::parse),
            ))),
        ));
        let (i, (pws, _, (scope, name, args, nws))) = p(i)?;
        let scope = scope.map(|(scope, _)| scope);
        let args = args.unwrap_or_default();
        Ok((
            i,
            Self {
                ws: Ws(pws, nws),
                scope,
                name,
                args,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct Match {
    pub ws1: Ws,
    pub expr: Expr,
    pub arms: Vec<When>,
    pub ws2: Ws,
}

impl Match {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = tuple((
            opt(Whitespace::parse),
            ws(keyword("match")),
            cut(tuple((
                ws(|i| Expr::parse(i, s.level.get())),
                opt(Whitespace::parse),
                |i| s.tag_block_end(i),
                cut(tuple((
                    ws(many0(ws(value((), |i| Comment::parse(i, s))))),
                    many1(|i| When::when(i, s)),
                    cut(tuple((
                        opt(|i| When::r#match(i, s)),
                        cut(tuple((
                            ws(|i| s.tag_block_start(i)),
                            opt(Whitespace::parse),
                            ws(keyword("endmatch")),
                            opt(Whitespace::parse),
                        ))),
                    ))),
                ))),
            ))),
        ));
        let (i, (pws1, _, (expr, nws1, _, (_, arms, (else_arm, (_, pws2, _, nws2)))))) = p(i)?;

        let mut arms = arms;
        if let Some(arm) = else_arm {
            arms.push(arm);
        }

        Ok((
            i,
            Self {
                ws1: Ws(pws1, nws1),
                expr,
                arms,
                ws2: Ws(pws2, nws2),
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct BlockDef {
    pub ws1: Ws,
    pub name: RcStr,
    pub nodes: Vec<Node>,
    pub ws2: Ws,
}

impl BlockDef {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut start = tuple((
            opt(Whitespace::parse),
            ws(keyword("block")),
            cut(tuple((ws(identifier), opt(Whitespace::parse), |i| {
                s.tag_block_end(i)
            }))),
        ));
        let (i, (pws1, _, (name, nws1, _))) = start(i)?;

        let mut end = cut(tuple((
            |i| Node::many(i, s),
            cut(tuple((
                |i| s.tag_block_start(i),
                opt(Whitespace::parse),
                ws(keyword("endblock")),
                cut(tuple((
                    opt(|before: RcStr| {
                        let (after, end_name) = ws(identifier)(before.clone())?;
                        check_end_name(before, after, name.clone(), end_name, "block")
                    }),
                    opt(Whitespace::parse),
                ))),
            ))),
        )));
        let (i, (nodes, (_, pws2, _, (_, nws2)))) = end(i)?;

        Ok((
            i,
            BlockDef {
                ws1: Ws(pws1, nws1),
                name: name.clone(),
                nodes,
                ws2: Ws(pws2, nws2),
            },
        ))
    }
}

fn check_end_name(
    before: RcStr,
    after: RcStr,
    name: RcStr,
    end_name: RcStr,
    kind: &str,
) -> ParseResult {
    if name == end_name {
        return Ok((after, end_name));
    }
    let message = if name.is_empty() && !end_name.is_empty() {
        format!("unexpected name `{end_name}` in `end{kind}` tag for unnamed `{kind}`")
    } else {
        format!("expected name `{name}` in `end{kind}` tag, found `{end_name}`")
    };
    Err(nom::Err::Failure(ErrorContext {
        input: before,
        message: Some(Cow::Owned(message)),
    }))
}

#[derive(Debug, PartialEq)]
pub struct Lit {
    pub lws: RcStr,
    pub val: RcStr,
    pub rws: RcStr,
}

impl Lit {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let p_start = alt((
            tag(s.syntax.block_start),
            tag(s.syntax.comment_start),
            tag(s.syntax.expr_start),
        ));

        let (i, _) = not(eof)(i)?;
        let (i, content) = opt(recognize(skip_till(p_start)))(i)?;
        let (i, content) = match content {
            Some(rest) if rest.is_empty() => {
                // {block,comment,expr}_start follows immediately.
                return Err(nom::Err::Error(error_position!(i, ErrorKind::TakeUntil)));
            }
            Some(content) => (i, content),
            None => (RcStr::default(), i), // there is no {block,comment,expr}_start: take everything
        };
        Ok((i, Self::split_ws_parts(content)))
    }

    pub(crate) fn split_ws_parts(s: RcStr) -> Self {
        let trimmed_start = s.trim_start_matches(is_ws);
        let len_start = s.len() - trimmed_start.len();
        let trimmed = trimmed_start.trim_end_matches(is_ws);
        Self {
            lws: s.substr(..len_start),
            rws: trimmed_start.substr(trimmed.len()..),
            val: trimmed,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Raw {
    pub ws1: Ws,
    pub lit: Lit,
    pub ws2: Ws,
}

impl Raw {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let endraw = tuple((
            |i| s.tag_block_start(i),
            opt(Whitespace::parse),
            ws(keyword("endraw")),
            opt(Whitespace::parse),
            peek(|i| s.tag_block_end(i)),
        ));

        let mut p = tuple((
            opt(Whitespace::parse),
            ws(keyword("raw")),
            cut(tuple((
                opt(Whitespace::parse),
                |i| s.tag_block_end(i),
                consumed(skip_till(endraw)),
            ))),
        ));

        let (_, (pws1, _, (nws1, _, (contents, (i, (_, pws2, _, nws2, _)))))) = p(i)?;
        let lit = Lit::split_ws_parts(contents);
        let ws1 = Ws(pws1, nws1);
        let ws2 = Ws(pws2, nws2);
        Ok((i, Self { ws1, lit, ws2 }))
    }
}

#[derive(Debug, PartialEq)]
pub struct Let {
    pub ws: Ws,
    pub var: Target,
    pub val: Option<Expr>,
}

impl Let {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = tuple((
            opt(Whitespace::parse),
            ws(alt((keyword("let"), keyword("set")))),
            cut(tuple((
                ws(|i| Target::parse(i, s)),
                opt(preceded(
                    ws(char('=')),
                    ws(|i| Expr::parse(i, s.level.get())),
                )),
                opt(Whitespace::parse),
            ))),
        ));
        let (i, (pws, _, (var, val, nws))) = p(i)?;

        Ok((
            i,
            Let {
                ws: Ws(pws, nws),
                var,
                val,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct If {
    pub ws: Ws,
    pub branches: Vec<Cond>,
}

impl If {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        let mut p = tuple((
            opt(Whitespace::parse),
            |i| CondTest::parse(i, s),
            cut(tuple((
                opt(Whitespace::parse),
                |i| s.tag_block_end(i),
                cut(tuple((
                    |i| Node::many(i, s),
                    many0(|i| Cond::parse(i, s)),
                    cut(tuple((
                        |i| s.tag_block_start(i),
                        opt(Whitespace::parse),
                        ws(keyword("endif")),
                        opt(Whitespace::parse),
                    ))),
                ))),
            ))),
        ));

        let (i, (pws1, cond, (nws1, _, (nodes, elifs, (_, pws2, _, nws2))))) = p(i)?;
        let mut branches = vec![Cond {
            ws: Ws(pws1, nws1),
            cond: Some(cond),
            nodes,
        }];
        branches.extend(elifs);

        Ok((
            i,
            Self {
                ws: Ws(pws2, nws2),
                branches,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct Include {
    pub ws: Ws,
    pub path: RcStr,
}

impl Include {
    fn parse(i: RcStr) -> ParseResult<Self> {
        let mut p = tuple((
            opt(Whitespace::parse),
            ws(keyword("include")),
            cut(pair(ws(str_lit), opt(Whitespace::parse))),
        ));
        let (i, (pws, _, (path, nws))) = p(i)?;
        Ok((
            i,
            Self {
                ws: Ws(pws, nws),
                path,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct Extends {
    pub path: RcStr,
}

impl Extends {
    fn parse(i: RcStr) -> ParseResult<Self> {
        let start = i.clone();

        let (i, (pws, _, (path, nws))) = tuple((
            opt(Whitespace::parse),
            ws(keyword("extends")),
            cut(pair(ws(str_lit), opt(Whitespace::parse))),
        ))(i)?;
        match (pws, nws) {
            (None, None) => Ok((i, Self { path })),
            (_, _) => Err(nom::Err::Failure(ErrorContext {
                input: start,
                message: Some(Cow::Borrowed(
                    "whitespace control is not allowed on `extends`",
                )),
            })),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Comment {
    pub ws: Ws,
    pub content: RcStr,
}

impl Comment {
    fn parse(i: RcStr, s: &State<'_>) -> ParseResult<Self> {
        #[derive(Debug, Clone, Copy)]
        enum Tag {
            Open,
            Close,
        }

        fn tag(i: RcStr, s: &State<'_>) -> ParseResult<Tag> {
            alt((
                value(Tag::Open, |i| s.tag_comment_start(i)),
                value(Tag::Close, |i| s.tag_comment_end(i)),
            ))(i)
        }

        fn content(mut i: RcStr, s: &State<'_>) -> ParseResult<()> {
            let mut depth = 0usize;
            loop {
                let (_, (j, tag)) = skip_till(|i| tag(i, s))(i.clone())?;
                match tag {
                    Tag::Open => match depth.checked_add(1) {
                        Some(new_depth) => depth = new_depth,
                        None => {
                            return Err(nom::Err::Failure(ErrorContext {
                                input: i,
                                message: Some(Cow::Borrowed("too deeply nested comments")),
                            }))
                        }
                    },
                    Tag::Close => match depth.checked_sub(1) {
                        Some(new_depth) => depth = new_depth,
                        None => return Ok((j, ())),
                    },
                }
                i = j;
            }
        }

        let (i, (pws, content)) = pair(
            preceded(|i| s.tag_comment_start(i), opt(Whitespace::parse)),
            recognize(cut(|i| content(i, s))),
        )(i)?;

        let mut nws = None;
        if let Some(content) = content.as_str().strip_suffix(s.syntax.comment_end) {
            nws = match content.chars().last() {
                Some('-') => Some(Whitespace::Suppress),
                Some('+') => Some(Whitespace::Preserve),
                Some('~') => Some(Whitespace::Minimize),
                _ => None,
            }
        };

        Ok((
            i,
            Self {
                ws: Ws(pws, nws),
                content,
            },
        ))
    }
}

/// First field is "minus/plus sign was used on the left part of the item".
///
/// Second field is "minus/plus sign was used on the right part of the item".
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ws(pub Option<Whitespace>, pub Option<Whitespace>);
