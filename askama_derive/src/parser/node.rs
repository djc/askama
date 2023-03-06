//! Template abstract syntax tree node types

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
    tag_expr_start, take_content, ws, Expr, State, Whitespace, Ws,
};

/// An Askama block.
///
/// This represents both the top-level block of a template and all sub-blocks of statement nodes.
#[derive(Debug, PartialEq)]
pub(crate) struct Block<'a> {
    /// The nodes within the block.
    pub(crate) nodes: Vec<Node<'a>>,
    /// Whitespace suppression for the inside of the block.
    pub(crate) ws: Ws,
}

impl<'a> Block<'a> {
    #[cfg(test)]
    pub(crate) fn with_whitespace(ws: Ws) -> Self {
        Block { nodes: vec![], ws }
    }
}

impl<'a> PartialEq<Vec<Node<'a>>> for Block<'a> {
    fn eq(&self, nodes: &Vec<Node<'a>>) -> bool {
        &self.nodes == nodes
    }
}

/// An Askama template abstract syntax tree node.
#[derive(Debug, PartialEq)]
pub(crate) enum Node<'a> {
    /// Literal text to output directly.
    Lit(Lit<'a>),
    /// An Askama tag, either a comment, expression, or statement.
    ///
    /// The `Ws` element represents whitespace suppression for the outside of the entire tag.
    Tag(Ws, Tag<'a>),
}

/// An Askama tag.
///
/// Tags come in three "flavors": comments, expressions, and statements.
/// With the default syntax, comments are tags wrapped in `{# #}`, expressions are
/// wrapped in `{{ }}`, and statements in `{% %}`.
///
/// Expression and comment tags are always self-closing.  Statement tags
/// may or may not have a matching end tag, depending on the type of statement.
/// Statements with child `Block`s always require an end tag.
#[derive(Debug, PartialEq)]
pub(crate) enum Tag<'a> {
    /// A block comment.
    ///
    /// ```ignore
    /// {# A Comment #}
    /// ```
    Comment,
    /// An expression, the result of which will be output.
    ///
    /// ```ignore
    /// {{ 25 / 6 - 4 }}
    /// ```
    Expr(Expr<'a>),
    /// A macro invocation.
    ///
    /// ```ignore
    /// {% call scope::heading(s) %}
    /// ```
    Call(Call<'a>),
    /// A variable declaration without an assignment.
    ///
    /// ```ignore
    /// {% let val %}
    /// ```
    LetDecl(Target<'a>),
    /// A variable assignment.
    ///
    /// ```ignore
    /// {% let val = "foo" %}
    /// ```
    Let(Target<'a>, Expr<'a>),
    /// An if-else block.
    ///
    /// ```ignore
    /// {% if users.len() == 0 %}
    ///   No users
    /// {% else if users.len() == 1 %}
    ///   1 user
    /// {% else %}
    ///   {{ users.len() }} users
    /// {% endif %}
    /// ```
    Cond(Vec<Cond<'a>>),
    /// A match block with several clauses.
    ///
    /// ```ignore
    /// {% match item %}
    ///   {% when Some with ("foo") %}
    ///     Found literal foo
    ///   {% when Some with (val) %}
    ///     Found {{ val }}
    ///   {% when None %}
    /// {% endmatch %}
    /// ```
    Match(Match<'a>),
    /// A for loop.
    ///
    /// ```ignore
    /// Users
    /// -----
    /// {% for user in users %}
    ///   - {{ user.name }}
    /// {% endfor %}
    /// ```
    Loop(Loop<'a>),
    /// A template inheritance declaration.
    ///
    /// ```ignore
    /// {% extends "base.html" %}
    /// ```
    Extends(&'a str),
    /// A block definition.
    ///
    /// ```ignore
    /// {% block title %}Index{% endblock %}
    /// ```
    BlockDef(BlockDef<'a>),
    /// Include the specified template file inline here.
    ///
    /// ```ignore
    /// {% include "item.html" %}
    /// ```
    Include(&'a str),
    /// Import macros from another template file.
    ///
    /// ```ignore
    /// {% import "macros.html" as scope %}
    /// ```
    Import(&'a str, &'a str),
    /// A macro declaration.
    ///
    /// ```ignore
    /// {% macro heading(arg) %}
    /// {{arg}}
    /// -------
    /// {% endmacro %}
    /// ```
    Macro(Macro<'a>),
    /// A raw block.
    ///
    /// ```ignore
    /// {% raw %}
    /// {{ this * is - not + an % expression }}
    /// {% endraw %}
    /// ```
    Raw(Raw<'a>),
    /// The break statement.
    Break,
    /// The continue statement.
    Continue,
    /// A call to render the parent block.
    ///
    /// ```ignore
    /// {% call super() %}
    /// ```
    Super,
}

/// The Askama equivalent of a Rust pattern, the target of a match or assignment.
#[derive(Debug, PartialEq)]
pub(crate) enum Target<'a> {
    /// Bind the value to a name.
    Name(&'a str),
    /// Destructure a tuple value.
    Tuple(Vec<&'a str>, Vec<Target<'a>>),
    /// Destructure a struct value.
    Struct(Vec<&'a str>, Vec<(&'a str, Target<'a>)>),
    /// Match a numeric literal.
    NumLit(&'a str),
    /// Match a string literal.
    StrLit(&'a str),
    /// Match a character literal.
    CharLit(&'a str),
    /// Match a boolean literal.
    BoolLit(&'a str),
    /// Match against a path.
    Path(Vec<&'a str>),
}

/// A literal bit of text to output directly.
#[derive(Debug, PartialEq)]
pub(crate) struct Lit<'a> {
    /// White space preceeding the text.
    pub(crate) lws: &'a str,
    /// The literal text itself.
    pub(crate) val: &'a str,
    /// White space following the text.
    pub(crate) rws: &'a str,
}

/// A raw block to output directly.
#[derive(Debug, PartialEq)]
pub(crate) struct Raw<'a> {
    /// The content of the raw block.
    pub(crate) lit: Lit<'a>,
    /// Whitespace suppression for the inside of the block.
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
    /// The target pattern to match.
    pub(crate) target: Target<'a>,
    /// Body of the match arm.
    pub(crate) block: Block<'a>,
}

/// A for loop syntax node.
#[derive(Debug, PartialEq)]
pub(crate) struct Loop<'a> {
    /// The variable of iteration within the loop.
    pub(crate) var: Target<'a>,
    /// The collection to iterate over.
    pub(crate) iter: Expr<'a>,
    /// An optional condition, which if it evaluates to false should skip that iteration.
    pub(crate) cond: Option<Expr<'a>>,
    /// The body of the loop.
    pub(crate) body: Block<'a>,
    /// The else block of the loop, invoked if the collection is empty.
    pub(crate) else_block: Block<'a>,
}

/// A macro definition.
#[derive(Debug, PartialEq)]
pub(crate) struct Macro<'a> {
    /// The name of the macro.
    pub(crate) name: &'a str,
    /// Names of each of the macro's parameters.
    pub(crate) args: Vec<&'a str>,
    /// The body of the macro.
    pub(crate) block: Block<'a>,
}

/// A block statement, either a definition or a reference.
#[derive(Debug, PartialEq)]
pub(crate) struct BlockDef<'a> {
    /// The name of the block.
    pub(crate) name: &'a str,
    /// The contents of the block.
    pub(crate) block: Block<'a>,
}

/// A single branch of a conditional statement.
#[derive(Debug, PartialEq)]
pub(crate) struct Cond<'a> {
    /// The test for this branch, or `None` for the `else` branch.
    pub(crate) test: Option<CondTest<'a>>,
    /// Body of this conditional branch.
    pub(crate) block: Block<'a>,
}

/// An if or if let condition.
#[derive(Debug, PartialEq)]
pub(crate) struct CondTest<'a> {
    /// For an if let, the assignment target.
    pub(crate) target: Option<Target<'a>>,
    /// The condition expression to evaluate.
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
    let ws = Ws::new(pws, nws);

    let tag = match name {
        "super" => Tag::Super,
        _ => {
            let scope = scope.map(|(scope, _)| scope);
            Tag::Call(Call { scope, name, args })
        }
    };
    Ok((i, Node::Tag(ws, tag)))
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
    let (i, (_, pws, _, (test, nws, _, nodes))) = p(i)?;
    let block = Block {
        nodes,
        ws: Ws::new(pws, nws),
    };
    Ok((i, Cond { test, block }))
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
    let (i, (pws1, cond, (nws1, _, (nodes, elifs, (_, pws2, _, nws2))))) = p(i)?;
    let block = Block {
        nodes,
        ws: Ws::new(pws1, nws1),
    };

    let mut res = vec![Cond {
        test: Some(cond),
        block,
    }];
    res.extend(elifs);

    let outer = Ws::new(pws1, nws2);

    let mut cursor = pws2;
    let mut idx = res.len() - 1;
    loop {
        std::mem::swap(&mut cursor, &mut res[idx].block.ws.flush);

        if idx == 0 {
            break;
        }
        idx -= 1;
    }

    Ok((i, Node::Tag(outer, Tag::Cond(res))))
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
    let (i, (_, pws, _, (nws, _, nodes))) = p(i)?;
    let block = Block {
        nodes,
        ws: Ws::new(pws, nws),
    };
    Ok((
        i,
        When {
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
    let (i, (_, pws, _, (target, nws, _, nodes))) = p(i)?;
    let block = Block {
        nodes,
        ws: Ws::new(pws, nws),
    };
    Ok((i, When { target, block }))
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

    let outer = Ws::new(pws1, nws2);

    let mut cursor = pws2;
    let mut idx = arms.len() - 1;
    loop {
        std::mem::swap(&mut cursor, &mut arms[idx].block.ws.flush);

        if idx == 0 {
            break;
        }
        idx -= 1;
    }

    Ok((i, Node::Tag(outer, Tag::Match(Match { expr, arms }))))
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
            Node::Tag(Ws::new(pws, nws), Tag::Let(var, val))
        } else {
            Node::Tag(Ws::new(pws, nws), Tag::LetDecl(var))
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
    let body = Block {
        nodes: body,
        ws: Ws::new(pws2, nws1),
    };
    let else_block = Block {
        nodes: else_block,
        ws: Ws::new(pws3, nws3),
    };
    Ok((
        i,
        Node::Tag(
            Ws::new(pws1, nws2),
            Tag::Loop(Loop {
                var,
                iter,
                cond,
                body,
                else_block,
            }),
        ),
    ))
}

fn block_extends(i: &str) -> IResult<&str, Node<'_>> {
    let (i, (_, name)) = tuple((ws(keyword("extends")), ws(str_lit)))(i)?;
    Ok((i, Node::Tag(Ws::new(None, None), Tag::Extends(name))))
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
    let (i, (nodes, (_, pws2, _, (_, nws2)))) = end(i)?;
    let block = Block {
        nodes,
        ws: Ws::new(pws2, nws1),
    };

    Ok((
        i,
        Node::Tag(Ws::new(pws1, nws2), Tag::BlockDef(BlockDef { name, block })),
    ))
}

fn block_include(i: &str) -> IResult<&str, Node<'_>> {
    let mut p = tuple((
        opt(expr_handle_ws),
        ws(keyword("include")),
        cut(pair(ws(str_lit), opt(expr_handle_ws))),
    ));
    let (i, (pws, _, (name, nws))) = p(i)?;
    Ok((i, Node::Tag(Ws::new(pws, nws), Tag::Include(name))))
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
    Ok((i, Node::Tag(Ws::new(pws, nws), Tag::Import(name, scope))))
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
    let (i, (nodes, (_, pws2, _, (_, nws2)))) = end(i)?;
    let block = Block {
        nodes,
        ws: Ws::new(pws2, nws1),
    };

    assert_ne!(name, "super", "invalid macro name 'super'");

    Ok((
        i,
        Node::Tag(
            Ws::new(pws1, nws2),
            Tag::Macro(Macro {
                name,
                args: params,
                block,
            }),
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
    let outer = Ws::new(pws1, nws2);
    let ws = Ws::new(pws2, nws1);
    Ok((i, Node::Tag(outer, Tag::Raw(Raw { lit, ws }))))
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
    Ok((j, Node::Tag(Ws::new(pws, nws), Tag::Break)))
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
    Ok((j, Node::Tag(Ws::new(pws, nws), Tag::Continue)))
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
    Ok((i, Node::Tag(Ws::new(pws, nws), Tag::Comment)))
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
    Ok((i, Node::Tag(Ws::new(pws, nws), Tag::Expr(expr))))
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
