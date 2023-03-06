use std::str;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::char;
use nom::combinator::{complete, consumed, cut, map, opt, peek, value};
use nom::error::{Error, ErrorKind};
use nom::multi::{fold_many0, many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{error_position, IResult};

use super::{
    bool_lit, char_lit, identifier, keyword, num_lit, path, skip_till, split_ws_parts, str_lit,
    tag_block_end, tag_block_start, tag_comment_end, tag_comment_start, tag_expr_end,
    tag_expr_start, take_content, ws, Expr, State,
};

#[derive(Debug, PartialEq)]
pub(crate) enum Node<'a> {
    Lit(Lit<'a>),
    Tag(Ws, Tag<'a>),
    LetDecl(Ws, Target<'a>),
    Let(Ws, Target<'a>, Expr<'a>),
    Cond(Ws, Vec<Cond<'a>>),
    Match(Ws, Match<'a>),
    Loop(Ws, Loop<'a>),
    Extends(&'a str),
    BlockDef(Ws, BlockDef<'a>),
    Include(Ws, &'a str),
    Import(Ws, &'a str, &'a str),
    Macro(Ws, Macro<'a>),
    Raw(Ws, Raw<'a>),
    Break(Ws),
    Continue(Ws),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Tag<'a> {
    Comment,
    Expr(Expr<'a>),
    Call(Call<'a>),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Target<'a> {
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
pub(crate) enum Whitespace {
    Preserve,
    Suppress,
    Minimize,
}

/// A literal bit of text to output directly.
#[derive(Debug, PartialEq)]
pub(crate) struct Lit<'a> {
    pub(crate) lws: &'a str,
    pub(crate) val: &'a str,
    pub(crate) rws: &'a str,
}

/// A raw block to output directly.
#[derive(Debug, PartialEq)]
pub(crate) struct Raw<'a> {
    pub(crate) lit: Lit<'a>,
    pub(crate) ws: Ws,
}

/// A macro call statement.
#[derive(Debug, PartialEq)]
pub(crate) struct Call<'a> {
    /// If the macro is imported, the scope name.
    pub(crate) scope: Option<&'a str>,
    /// The name of the macro to call.
    pub(crate) name: &'a str,
    /// The arguments to the macro.
    pub(crate) args: Vec<Expr<'a>>,
}

/// A match statement.
#[derive(Debug, PartialEq)]
pub(crate) struct Match<'a> {
    /// The expression to match against.
    pub(crate) expr: Expr<'a>,
    /// Each of the match arms, with a pattern and a body.
    pub(crate) arms: Vec<When<'a>>,
}

/// A single arm of a match statement.
#[derive(Debug, PartialEq)]
pub(crate) struct When<'a> {
    pub(crate) ws: Ws,
    /// The target pattern to match.
    pub(crate) target: Target<'a>,
    /// Body of the match arm.
    pub(crate) block: Vec<Node<'a>>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Loop<'a> {
    pub(crate) var: Target<'a>,
    pub(crate) iter: Expr<'a>,
    pub(crate) cond: Option<Expr<'a>>,
    pub(crate) body: Vec<Node<'a>>,
    pub(crate) body_ws: Ws,
    pub(crate) else_block: Vec<Node<'a>>,
    pub(crate) else_ws: Ws,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Macro<'a> {
    pub(crate) name: &'a str,
    pub(crate) args: Vec<&'a str>,
    pub(crate) nodes: Vec<Node<'a>>,
    pub(crate) ws: Ws,
}

/// A block statement, either a definition or a reference.
#[derive(Debug, PartialEq)]
pub(crate) struct BlockDef<'a> {
    pub(crate) name: &'a str,
    pub(crate) block: Vec<Node<'a>>,
    pub(crate) ws: Ws,
}

/// First field is "minus/plus sign was used on the left part of the item".
///
/// Second field is "minus/plus sign was used on the right part of the item".
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Ws(pub(crate) Option<Whitespace>, pub(crate) Option<Whitespace>);

/// A single branch of a conditional statement.
#[derive(Debug, PartialEq)]
pub(crate) struct Cond<'a> {
    pub(crate) ws: Ws,
    /// The test for this branch, or `None` for the `else` branch.
    pub(crate) test: Option<CondTest<'a>>,
    /// Body of this conditional branch.
    pub(crate) block: Vec<Node<'a>>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct CondTest<'a> {
    pub(crate) target: Option<Target<'a>>,
    pub(crate) expr: Expr<'a>,
}

impl Node<'_> {
    pub(super) fn parse<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Vec<Node<'a>>> {
        parse_template(i, s)
    }
}

impl Target<'_> {
    pub(super) fn parse(i: &str) -> IResult<&str, Target<'_>> {
        target(i)
    }
}

fn expr_handle_ws(i: &str) -> IResult<&str, Whitespace> {
    alt((char('-'), char('+'), char('~')))(i).map(|(s, r)| (s, Whitespace::from(r)))
}

fn parameters(i: &str) -> IResult<&str, Vec<&str>> {
    delimited(
        ws(char('(')),
        separated_list0(char(','), ws(identifier)),
        ws(char(')')),
    )(i)
}

fn block_call(i: &str) -> IResult<&str, Node<'_>> {
    let mut p = tuple((
        opt(expr_handle_ws),
        ws(keyword("call")),
        cut(tuple((
            opt(tuple((ws(identifier), ws(tag("::"))))),
            ws(identifier),
            ws(Expr::parse_arguments),
            opt(expr_handle_ws),
        ))),
    ));
    let (i, (pws, _, (scope, name, args, nws))) = p(i)?;
    let ws = Ws(pws, nws);
    let scope = scope.map(|(scope, _)| scope);
    Ok((i, Node::Tag(ws, Tag::Call(Call { scope, name, args }))))
}

fn cond_if(i: &str) -> IResult<&str, CondTest<'_>> {
    let mut p = preceded(
        ws(keyword("if")),
        cut(tuple((
            opt(delimited(
                ws(alt((keyword("let"), keyword("set")))),
                ws(Target::parse),
                ws(char('=')),
            )),
            ws(Expr::parse),
        ))),
    );
    let (i, (target, expr)) = p(i)?;
    Ok((i, CondTest { target, expr }))
}

fn cond_block<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Cond<'a>> {
    let mut p = tuple((
        |i| tag_block_start(i, s),
        opt(expr_handle_ws),
        ws(keyword("else")),
        cut(tuple((
            opt(cond_if),
            opt(expr_handle_ws),
            |i| tag_block_end(i, s),
            cut(|i| parse_template(i, s)),
        ))),
    ));
    let (i, (_, pws, _, (test, nws, _, block))) = p(i)?;
    Ok((
        i,
        Cond {
            ws: Ws(pws, nws),
            test,
            block,
        },
    ))
}

fn block_if<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Node<'a>> {
    let mut p = tuple((
        opt(expr_handle_ws),
        cond_if,
        cut(tuple((
            opt(expr_handle_ws),
            |i| tag_block_end(i, s),
            cut(tuple((
                |i| parse_template(i, s),
                many0(|i| cond_block(i, s)),
                cut(tuple((
                    |i| tag_block_start(i, s),
                    opt(expr_handle_ws),
                    ws(keyword("endif")),
                    opt(expr_handle_ws),
                ))),
            ))),
        ))),
    ));
    let (i, (pws1, cond, (nws1, _, (block, elifs, (_, pws2, _, nws2))))) = p(i)?;

    let mut res = vec![Cond {
        ws: Ws(pws1, nws1),
        test: Some(cond),
        block,
    }];
    res.extend(elifs);

    let outer = Ws(pws1, nws2);

    let mut cursor = pws2;
    let mut idx = res.len() - 1;
    loop {
        std::mem::swap(&mut cursor, &mut res[idx].ws.0);

        if idx == 0 {
            break;
        }
        idx -= 1;
    }

    Ok((i, Node::Cond(outer, res)))
}

fn match_else_block<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, When<'a>> {
    let mut p = tuple((
        |i| tag_block_start(i, s),
        opt(expr_handle_ws),
        ws(keyword("else")),
        cut(tuple((
            opt(expr_handle_ws),
            |i| tag_block_end(i, s),
            cut(|i| parse_template(i, s)),
        ))),
    ));
    let (i, (_, pws, _, (nws, _, block))) = p(i)?;
    Ok((
        i,
        When {
            ws: Ws(pws, nws),
            target: Target::Name("_"),
            block,
        },
    ))
}

fn when_block<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, When<'a>> {
    let mut p = tuple((
        |i| tag_block_start(i, s),
        opt(expr_handle_ws),
        ws(keyword("when")),
        cut(tuple((
            ws(Target::parse),
            opt(expr_handle_ws),
            |i| tag_block_end(i, s),
            cut(|i| parse_template(i, s)),
        ))),
    ));
    let (i, (_, pws, _, (target, nws, _, block))) = p(i)?;
    Ok((
        i,
        When {
            ws: Ws(pws, nws),
            target,
            block,
        },
    ))
}

fn block_match<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Node<'a>> {
    let mut p = tuple((
        opt(expr_handle_ws),
        ws(keyword("match")),
        cut(tuple((
            ws(Expr::parse),
            opt(expr_handle_ws),
            |i| tag_block_end(i, s),
            cut(tuple((
                ws(many0(ws(value((), |i| block_comment(i, s))))),
                many1(|i| when_block(i, s)),
                cut(tuple((
                    opt(|i| match_else_block(i, s)),
                    cut(tuple((
                        ws(|i| tag_block_start(i, s)),
                        opt(expr_handle_ws),
                        ws(keyword("endmatch")),
                        opt(expr_handle_ws),
                    ))),
                ))),
            ))),
        ))),
    ));
    let (i, (pws1, _, (expr, _, _, (_, arms, (else_arm, (_, pws2, _, nws2)))))) = p(i)?;

    let mut arms = arms;
    if let Some(arm) = else_arm {
        arms.push(arm);
    }

    let outer = Ws(pws1, nws2);

    let mut cursor = pws2;
    let mut idx = arms.len() - 1;
    loop {
        std::mem::swap(&mut cursor, &mut arms[idx].ws.0);

        if idx == 0 {
            break;
        }
        idx -= 1;
    }

    Ok((i, Node::Match(outer, Match { expr, arms })))
}

fn block_let(i: &str) -> IResult<&str, Node<'_>> {
    let mut p = tuple((
        opt(expr_handle_ws),
        ws(alt((keyword("let"), keyword("set")))),
        cut(tuple((
            ws(Target::parse),
            opt(tuple((ws(char('=')), ws(Expr::parse)))),
            opt(expr_handle_ws),
        ))),
    ));
    let (i, (pws, _, (var, val, nws))) = p(i)?;

    Ok((
        i,
        if let Some((_, val)) = val {
            Node::Let(Ws(pws, nws), var, val)
        } else {
            Node::LetDecl(Ws(pws, nws), var)
        },
    ))
}

fn parse_loop_content<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Vec<Node<'a>>> {
    s.enter_loop();
    let result = parse_template(i, s);
    s.leave_loop();
    result
}

fn block_for<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Node<'a>> {
    let if_cond = preceded(ws(keyword("if")), cut(ws(Expr::parse)));
    let else_block = |i| {
        let mut p = preceded(
            ws(keyword("else")),
            cut(tuple((
                opt(expr_handle_ws),
                delimited(
                    |i| tag_block_end(i, s),
                    |i| parse_template(i, s),
                    |i| tag_block_start(i, s),
                ),
                opt(expr_handle_ws),
            ))),
        );
        let (i, (pws, nodes, nws)) = p(i)?;
        Ok((i, (pws, nodes, nws)))
    };
    let mut p = tuple((
        opt(expr_handle_ws),
        ws(keyword("for")),
        cut(tuple((
            ws(Target::parse),
            ws(keyword("in")),
            cut(tuple((
                ws(Expr::parse),
                opt(if_cond),
                opt(expr_handle_ws),
                |i| tag_block_end(i, s),
                cut(tuple((
                    |i| parse_loop_content(i, s),
                    cut(tuple((
                        |i| tag_block_start(i, s),
                        opt(expr_handle_ws),
                        opt(else_block),
                        ws(keyword("endfor")),
                        opt(expr_handle_ws),
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
        Node::Loop(
            Ws(pws1, nws2),
            Loop {
                var,
                iter,
                cond,
                body,
                body_ws: Ws(pws2, nws1),
                else_block,
                else_ws: Ws(pws3, nws3),
            },
        ),
    ))
}

fn block_extends(i: &str) -> IResult<&str, Node<'_>> {
    let (i, (_, name)) = tuple((ws(keyword("extends")), ws(str_lit)))(i)?;
    Ok((i, Node::Extends(name)))
}

fn block_block<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Node<'a>> {
    let mut start = tuple((
        opt(expr_handle_ws),
        ws(keyword("block")),
        cut(tuple((ws(identifier), opt(expr_handle_ws), |i| {
            tag_block_end(i, s)
        }))),
    ));
    let (i, (pws1, _, (name, nws1, _))) = start(i)?;

    let mut end = cut(tuple((
        |i| parse_template(i, s),
        cut(tuple((
            |i| tag_block_start(i, s),
            opt(expr_handle_ws),
            ws(keyword("endblock")),
            cut(tuple((opt(ws(keyword(name))), opt(expr_handle_ws)))),
        ))),
    )));
    let (i, (block, (_, pws2, _, (_, nws2)))) = end(i)?;

    Ok((
        i,
        Node::BlockDef(
            Ws(pws1, nws2),
            BlockDef {
                name,
                block,
                ws: Ws(pws2, nws1),
            },
        ),
    ))
}

fn block_include(i: &str) -> IResult<&str, Node<'_>> {
    let mut p = tuple((
        opt(expr_handle_ws),
        ws(keyword("include")),
        cut(pair(ws(str_lit), opt(expr_handle_ws))),
    ));
    let (i, (pws, _, (name, nws))) = p(i)?;
    Ok((i, Node::Include(Ws(pws, nws), name)))
}

fn block_import(i: &str) -> IResult<&str, Node<'_>> {
    let mut p = tuple((
        opt(expr_handle_ws),
        ws(keyword("import")),
        cut(tuple((
            ws(str_lit),
            ws(keyword("as")),
            cut(pair(ws(identifier), opt(expr_handle_ws))),
        ))),
    ));
    let (i, (pws, _, (name, _, (scope, nws)))) = p(i)?;
    Ok((i, Node::Import(Ws(pws, nws), name, scope)))
}

fn block_macro<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Node<'a>> {
    let mut start = tuple((
        opt(expr_handle_ws),
        ws(keyword("macro")),
        cut(tuple((
            ws(identifier),
            ws(parameters),
            opt(expr_handle_ws),
            |i| tag_block_end(i, s),
        ))),
    ));
    let (i, (pws1, _, (name, params, nws1, _))) = start(i)?;

    let mut end = cut(tuple((
        |i| parse_template(i, s),
        cut(tuple((
            |i| tag_block_start(i, s),
            opt(expr_handle_ws),
            ws(keyword("endmacro")),
            cut(tuple((opt(ws(keyword(name))), opt(expr_handle_ws)))),
        ))),
    )));
    let (i, (contents, (_, pws2, _, (_, nws2)))) = end(i)?;

    assert_ne!(name, "super", "invalid macro name 'super'");

    Ok((
        i,
        Node::Macro(
            Ws(pws1, nws2),
            Macro {
                name,
                args: params,
                nodes: contents,
                ws: Ws(pws2, nws1),
            },
        ),
    ))
}

fn block_raw<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Node<'a>> {
    let endraw = tuple((
        |i| tag_block_start(i, s),
        opt(expr_handle_ws),
        ws(keyword("endraw")),
        opt(expr_handle_ws),
        peek(|i| tag_block_end(i, s)),
    ));

    let mut p = tuple((
        opt(expr_handle_ws),
        ws(keyword("raw")),
        cut(tuple((
            opt(expr_handle_ws),
            |i| tag_block_end(i, s),
            consumed(skip_till(endraw)),
        ))),
    ));

    let (_, (pws1, _, (nws1, _, (contents, (i, (_, pws2, _, nws2, _)))))) = p(i)?;
    let lit = split_ws_parts(contents);
    let outer = Ws(pws1, nws2);
    let ws = Ws(pws2, nws1);
    Ok((i, Node::Raw(outer, Raw { lit, ws })))
}

fn break_statement<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Node<'a>> {
    let mut p = tuple((
        opt(expr_handle_ws),
        ws(keyword("break")),
        opt(expr_handle_ws),
    ));
    let (j, (pws, _, nws)) = p(i)?;
    if !s.is_in_loop() {
        return Err(nom::Err::Failure(error_position!(i, ErrorKind::Tag)));
    }
    Ok((j, Node::Break(Ws(pws, nws))))
}

fn continue_statement<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Node<'a>> {
    let mut p = tuple((
        opt(expr_handle_ws),
        ws(keyword("continue")),
        opt(expr_handle_ws),
    ));
    let (j, (pws, _, nws)) = p(i)?;
    if !s.is_in_loop() {
        return Err(nom::Err::Failure(error_position!(i, ErrorKind::Tag)));
    }
    Ok((j, Node::Continue(Ws(pws, nws))))
}

fn block_node<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Node<'a>> {
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

fn block_comment_body<'a>(mut i: &'a str, s: &State<'_>) -> IResult<&'a str, &'a str> {
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

fn block_comment<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Node<'a>> {
    let mut p = tuple((
        |i| tag_comment_start(i, s),
        cut(tuple((
            opt(expr_handle_ws),
            |i| block_comment_body(i, s),
            |i| tag_comment_end(i, s),
        ))),
    ));
    let (i, (_, (pws, tail, _))) = p(i)?;
    let nws = if tail.ends_with('-') {
        Some(Whitespace::Suppress)
    } else if tail.ends_with('+') {
        Some(Whitespace::Preserve)
    } else if tail.ends_with('~') {
        Some(Whitespace::Minimize)
    } else {
        None
    };
    Ok((i, Node::Tag(Ws(pws, nws), Tag::Comment)))
}

fn expr_node<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Node<'a>> {
    let mut p = tuple((
        |i| tag_expr_start(i, s),
        cut(tuple((
            opt(expr_handle_ws),
            ws(Expr::parse),
            opt(expr_handle_ws),
            |i| tag_expr_end(i, s),
        ))),
    ));
    let (i, (_, (pws, expr, nws, _))) = p(i)?;
    Ok((i, Node::Tag(Ws(pws, nws), Tag::Expr(expr))))
}

fn parse_template<'a>(i: &'a str, s: &State<'_>) -> IResult<&'a str, Vec<Node<'a>>> {
    many0(alt((
        complete(|i| take_content(i, s)),
        complete(|i| block_comment(i, s)),
        complete(|i| expr_node(i, s)),
        complete(|i| block_node(i, s)),
    )))(i)
}

fn variant_lit(i: &str) -> IResult<&str, Target<'_>> {
    alt((
        map(str_lit, Target::StrLit),
        map(char_lit, Target::CharLit),
        map(num_lit, Target::NumLit),
        map(bool_lit, Target::BoolLit),
    ))(i)
}

fn target(i: &str) -> IResult<&str, Target<'_>> {
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
        let (i, _) = opt(ws(keyword("with")))(i)?;

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

fn named_target(i: &str) -> IResult<&str, (&str, Target<'_>)> {
    let (i, (src, target)) = pair(identifier, opt(preceded(ws(char(':')), target)))(i)?;
    Ok((i, (src, target.unwrap_or(Target::Name(src)))))
}
