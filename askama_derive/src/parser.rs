// rustfmt doesn't do a very good job on nom parser invocations.
#![cfg_attr(rustfmt, rustfmt_skip)]

use nom;
use std::str;

use askama_shared::Syntax;

#[derive(Debug)]
pub enum Expr<'a> {
    NumLit(&'a str),
    StrLit(&'a str),
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
    MethodCall(Box<Expr<'a>>, &'a str, Vec<Expr<'a>>),
    RustMacro(&'a str, &'a str),
}

#[derive(Debug)]
pub enum MatchVariant<'a> {
    Path(Vec<&'a str>),
    Name(&'a str),
    NumLit(&'a str),
    StrLit(&'a str),
}

#[derive(Debug)]
pub enum MatchParameter<'a> {
    Name(&'a str),
    NumLit(&'a str),
    StrLit(&'a str),
}

#[derive(Debug)]
pub enum Target<'a> {
    Name(&'a str),
    Tuple(Vec<&'a str>),
}

#[derive(Clone, Copy, Debug)]
pub struct WS(pub bool, pub bool);

#[derive(Debug)]
pub struct Macro<'a> {
    pub ws1: WS,
    pub args: Vec<&'a str>,
    pub nodes: Vec<Node<'a>>,
    pub ws2: WS,
}

#[derive(Debug)]
pub enum Node<'a> {
    Lit(&'a str, &'a str, &'a str),
    Comment(WS),
    Expr(WS, Expr<'a>),
    Call(WS, Option<&'a str>, &'a str, Vec<Expr<'a>>),
    LetDecl(WS, Target<'a>),
    Let(WS, Target<'a>, Expr<'a>),
    Cond(Vec<(WS, Option<Expr<'a>>, Vec<Node<'a>>)>, WS),
    Match(WS, Expr<'a>, Option<&'a str>, Vec<When<'a>>, WS),
    Loop(WS, Target<'a>, Expr<'a>, Vec<Node<'a>>, WS),
    Extends(Expr<'a>),
    BlockDef(WS, &'a str, Vec<Node<'a>>, WS),
    Include(WS, &'a str),
    Import(WS, &'a str, &'a str),
    Macro(&'a str, Macro<'a>),
    Raw(WS, &'a str, WS),
}

pub type Cond<'a> = (WS, Option<Expr<'a>>, Vec<Node<'a>>);
pub type When<'a> = (
    WS,
    Option<MatchVariant<'a>>,
    MatchParameters<'a>,
    Vec<Node<'a>>,
);

#[derive(Debug)]
pub enum MatchParameters<'a> {
    Simple(Vec<MatchParameter<'a>>),
    Named(Vec<(&'a str, Option<MatchParameter<'a>>)>),
}

impl<'a> Default for MatchParameters<'a> {
    fn default() -> Self {
        MatchParameters::Simple(vec![])
    }
}

type Input<'a> = nom::types::CompleteByteSlice<'a>;
#[allow(non_snake_case)]
fn Input(input: &[u8]) -> Input {
    nom::types::CompleteByteSlice(input)
}

fn split_ws_parts(s: &[u8]) -> Node {
    if s.is_empty() {
        let rs = str::from_utf8(&s).unwrap();
        return Node::Lit(rs, rs, rs);
    }
    let is_ws = |c: &u8| *c != b' ' && *c != b'\t' && *c != b'\r' && *c != b'\n';
    let start = s.iter().position(&is_ws);
    let res = if start.is_none() {
        (s, &s[0..0], &s[0..0])
    } else {
        let start = start.unwrap();
        let end = s.iter().rposition(&is_ws);
        if end.is_none() {
            (&s[..start], &s[start..], &s[0..0])
        } else {
            let end = end.unwrap();
            (&s[..start], &s[start..=end], &s[end + 1..])
        }
    };
    Node::Lit(
        str::from_utf8(res.0).unwrap(),
        str::from_utf8(res.1).unwrap(),
        str::from_utf8(res.2).unwrap(),
    )
}

#[derive(Debug)]
enum ContentState {
    Any,
    Brace(usize),
    End(usize),
}

fn take_content<'a>(
    i: Input<'a>,
    s: &'a Syntax<'a>,
) -> Result<(Input<'a>, Node<'a>), nom::Err<Input<'a>>> {
    use crate::parser::ContentState::*;
    let bs = s.block_start.as_bytes()[0];
    let be = s.block_start.as_bytes()[1];
    let cs = s.comment_start.as_bytes()[0];
    let ce = s.comment_start.as_bytes()[1];
    let es = s.expr_start.as_bytes()[0];
    let ee = s.expr_start.as_bytes()[1];

    let mut state = Any;
    for (idx, c) in i.iter().enumerate() {
        state = match state {
            Any => {
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
            End(_) => panic!("cannot happen"),
        };
        if let End(_) = state {
            break;
        }
    }
    match state {
        Any | Brace(_) => Ok((Input(&i[..0]), split_ws_parts(i.0))),
        End(0) => Err(nom::Err::Error(error_position!(
            i,
            nom::ErrorKind::Custom(0)
        ))),
        End(start) => Ok((Input(&i[start..]), split_ws_parts(&i[..start]))),
    }
}

fn identifier(input: Input) -> Result<(Input, &str), nom::Err<Input>> {
    if !nom::is_alphabetic(input[0]) && input[0] != b'_' && !non_ascii(input[0]) {
        return Err(nom::Err::Error(error_position!(
            input,
            nom::ErrorKind::Custom(0)
        )));
    }
    for (i, ch) in input.iter().enumerate() {
        if i == 0 || nom::is_alphanumeric(*ch) || *ch == b'_' || non_ascii(*ch) {
            continue;
        }
        return Ok((Input(&input[i..]), str::from_utf8(&input[..i]).unwrap()));
    }
    Ok((Input(&input[1..]), str::from_utf8(&input[..1]).unwrap()))
}

#[inline]
fn non_ascii(chr: u8) -> bool {
    chr >= 0x80 && chr <= 0xFD
}

named!(num_lit<Input, &str>, map!(nom::digit,
    |s| str::from_utf8(s.0).unwrap()
));

named!(expr_num_lit<Input, Expr>, map!(num_lit,
    |s| Expr::NumLit(s)
));

named!(expr_array_lit<Input, Expr>,
    delimited!(
        ws!(tag!("[")),
        map!(separated_nonempty_list!(
            ws!(tag!(",")),
            expr_any
        ), |arr| Expr::Array(arr)),
        ws!(tag!("]"))
    )
);

named!(variant_num_lit<Input, MatchVariant>, map!(num_lit,
    |s| MatchVariant::NumLit(s)
));

named!(param_num_lit<Input, MatchParameter>, map!(num_lit,
    |s| MatchParameter::NumLit(s)
));

named!(expr_str_lit<Input, Expr>, map!(
    delimited!(char!('"'), take_until!("\""), char!('"')),
    |s| Expr::StrLit(str::from_utf8(&s).unwrap())
));

named!(variant_str_lit<Input, MatchVariant>, map!(
    delimited!(char!('"'), is_not!("\""), char!('"')),
    |s| MatchVariant::StrLit(str::from_utf8(&s).unwrap())
));

named!(param_str_lit<Input, MatchParameter>, map!(
    delimited!(char!('"'), is_not!("\""), char!('"')),
    |s| MatchParameter::StrLit(str::from_utf8(&s).unwrap())
));

named!(expr_var<Input, Expr>, map!(identifier,
    |s| Expr::Var(s))
);

named!(expr_path<Input, Expr>, do_parse!(
    start: call!(identifier) >>
    tag!("::") >>
    rest: separated_nonempty_list!(tag!("::"), identifier) >>
    ({
        let mut path = vec![start];
        path.extend(rest);
        Expr::Path(path)
    })
));

named!(variant_path<Input, MatchVariant>,
    map!(
        separated_nonempty_list!(tag!("::"), identifier),
        |path| MatchVariant::Path(path)
    )
);

named!(target_single<Input, Target>, map!(identifier,
    |s| Target::Name(s)
));

named!(target_tuple<Input, Target>, delimited!(
    tag!("("),
    do_parse!(
        res: separated_list!(tag!(","), ws!(identifier)) >>
        opt!(ws!(tag!(","))) >>
        (Target::Tuple(res))
    ),
    tag!(")")
));

named!(variant_name<Input, MatchVariant>, map!(identifier,
    |s| MatchVariant::Name(s)
));

named!(param_name<Input, MatchParameter>, map!(identifier,
    |s| MatchParameter::Name(s)
));

named!(arguments<Input, Vec<Expr>>, delimited!(
    tag!("("),
    separated_list!(tag!(","), ws!(expr_any)),
    tag!(")")
));

named!(macro_arguments<Input, &str>,
    delimited!(char!('('), nested_parenthesis, char!(')'))
);

fn nested_parenthesis(i: Input) -> Result<(Input, &str), nom::Err<Input>> {
    let mut nested = 0;
    let mut last = 0;
    let mut in_str = false;
    let mut escaped = false;

    for (i, b) in i.iter().enumerate() {
        if !(*b == b'(' || *b == b')') || !in_str {
            match *b {
                b'(' => {
                    nested += 1
                },
                b')' => {
                    if nested == 0 {
                        last = i;
                        break;
                    }
                    nested -= 1;
                },
                b'"' => {
                    if in_str {
                        if !escaped {
                            in_str = false;
                        }
                    } else {
                        in_str = true;
                    }
                },
                b'\\' => {
                    escaped = !escaped;
                },
                _ => (),
            }
        }

        if escaped && *b != b'\\' {
            escaped = false;
        }
    }

    if nested == 0 {
        Ok((Input(&i[last..]), str::from_utf8(&i[..last]).unwrap()))
    } else {
        Err(nom::Err::Error(error_position!(
            i,
            nom::ErrorKind::Custom(0)
        )))
    }
}

named!(parameters<Input, Vec<&str>>, delimited!(
    tag!("("),
    separated_list!(tag!(","), ws!(identifier)),
    tag!(")")
));

named!(with_parameters<Input, MatchParameters>, do_parse!(
    tag!("with") >>
    value: alt!(match_simple_parameters | match_named_parameters) >>
    (value)
));

named!(match_simple_parameters<Input, MatchParameters>, delimited!(
    ws!(tag!("(")),
    map!(separated_list!(tag!(","), ws!(match_parameter)), |mps| MatchParameters::Simple(mps)),
    tag!(")")
));

named!(match_named_parameters<Input, MatchParameters>, delimited!(
    ws!(tag!("{")),
    map!(separated_list!(tag!(","), ws!(match_named_parameter)), |mps| MatchParameters::Named(mps)),
    tag!("}")
));

named!(expr_group<Input, Expr>, map!(
    delimited!(char!('('), expr_any, char!(')')),
    |s| Expr::Group(Box::new(s))
));

named!(expr_single<Input, Expr>, alt!(
    expr_num_lit |
    expr_str_lit |
    expr_path |
    expr_rust_macro |
    expr_array_lit |
    expr_var |
    expr_group
));

named!(match_variant<Input, MatchVariant>, alt!(
    variant_path |
    variant_name |
    variant_num_lit |
    variant_str_lit
));

named!(match_parameter<Input, MatchParameter>, alt!(
    param_name |
    param_num_lit |
    param_str_lit
));

named!(match_named_parameter<Input, (&str, Option<MatchParameter>)>, do_parse!(
    name: identifier >>
    param: opt!(do_parse!(
        ws!(tag!(":")) >>
        param: match_parameter >>
        (param)
    )) >>
    ((name, param))
));

named!(attr<Input, (&str, Option<Vec<Expr>>)>, do_parse!(
    tag!(".") >>
    attr: alt!(num_lit | identifier) >>
    args: opt!(arguments) >>
    (attr, args)
));

named!(expr_attr<Input, Expr>, do_parse!(
    obj: expr_single >>
    attrs: many0!(attr) >>
    ({
        let mut res = obj;
        for (aname, args) in attrs {
            res = if args.is_some() {
                Expr::MethodCall(Box::new(res), aname, args.unwrap())
            } else {
                Expr::Attr(Box::new(res), aname)
            };
        }
        res
    })
));

named!(expr_index<Input, Expr>, do_parse!(
    obj: expr_attr >>
    key: opt!(do_parse!(
        ws!(tag!("[")) >>
        key: expr_any >>
        ws!(tag!("]")) >>
        (key)
    )) >>
    (match key {
        Some(key) => Expr::Index(Box::new(obj), Box::new(key)),
        None => obj,
    })
));

named!(filter<Input, (&str, Option<Vec<Expr>>)>, do_parse!(
    tag!("|") >>
    fname: identifier >>
    args: opt!(arguments) >>
    (fname, args)
));

named!(expr_filtered<Input, Expr>, do_parse!(
    obj: expr_index >>
    filters: many0!(filter) >>
    ({
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
       res
    })
));

named!(expr_unary<Input, Expr>, do_parse!(
    op: opt!(alt!(tag!("!") | tag!("-"))) >>
    expr: expr_filtered >>
    (match op {
        Some(op) => Expr::Unary(str::from_utf8(op.0).unwrap(), Box::new(expr)),
        None => expr,
    })
));

named!(expr_rust_macro<Input, Expr>, do_parse!(
    mname: identifier >>
    tag!("!") >>
    args: macro_arguments >>
    (Expr::RustMacro(mname, args))
));

macro_rules! expr_prec_layer {
    ( $name:ident, $inner:ident, $( $op:expr ),* ) => {
        named!($name<Input, Expr>, do_parse!(
            left: $inner >>
            op_and_right: opt!(pair!(ws!(alt!($( tag!($op) )|*)), expr_any)) >>
            (match op_and_right {
                Some((op, right)) => Expr::BinOp(
                    str::from_utf8(op.0).unwrap(), Box::new(left), Box::new(right)
                ),
                None => left,
            })
        ));
    }
}

expr_prec_layer!(expr_muldivmod, expr_unary, "*", "/", "%");
expr_prec_layer!(expr_addsub, expr_muldivmod, "+", "-");
expr_prec_layer!(expr_shifts, expr_addsub, ">>", "<<");
expr_prec_layer!(expr_band, expr_shifts, "&");
expr_prec_layer!(expr_bxor, expr_band, "^");
expr_prec_layer!(expr_bor, expr_bxor, "|");
expr_prec_layer!(expr_compare, expr_bor, "==", "!=", ">=", ">", "<=", "<");
expr_prec_layer!(expr_and, expr_compare, "&&");
expr_prec_layer!(expr_or, expr_and, "||");

named!(range_right<Input, Expr>, do_parse!(
    ws!(tag!("..")) >>
    incl: opt!(ws!(tag!("="))) >>
    right: opt!(expr_or) >>
    (Expr::Range(if incl.is_some() { "..=" } else { ".." }, None, right.map(Box::new)))
));

named!(expr_any<Input, Expr>, alt!(
    range_right |
    do_parse!(
        left: expr_or >>
        rest: range_right >> (match rest {
            Expr::Range(op, _, right) => Expr::Range(op, Some(Box::new(left)), right),
            _ => unreachable!(),
        })
    ) |
    expr_or
));

named_args!(expr_node<'a>(s: &'a Syntax<'a>) <Input<'a>, Node<'a>>, do_parse!(
    call!(tag_expr_start, s) >>
    pws: opt!(tag!("-")) >>
    expr: ws!(expr_any) >>
    nws: opt!(tag!("-")) >>
    call!(tag_expr_end, s) >>
    (Node::Expr(WS(pws.is_some(), nws.is_some()), expr))
));

named!(block_call<Input, Node>, do_parse!(
    pws: opt!(tag!("-")) >>
    ws!(tag!("call")) >>
    scope: opt!(do_parse!(
        scope: ws!(identifier) >>
        ws!(tag!("::")) >>
        (scope)
    )) >>
    name: ws!(identifier) >>
    args: ws!(arguments) >>
    nws: opt!(tag!("-")) >>
    (Node::Call(WS(pws.is_some(), nws.is_some()), scope, name, args))
));

named!(cond_if<Input, Expr>, do_parse!(
    ws!(tag!("if")) >>
    cond: ws!(expr_any) >>
    (cond)
));

named_args!(cond_block<'a>(s: &'a Syntax<'a>) <Input<'a>, Cond<'a>>, do_parse!(
    call!(tag_block_start, s) >>
    pws: opt!(tag!("-")) >>
    ws!(tag!("else")) >>
    cond: opt!(cond_if) >>
    nws: opt!(tag!("-")) >>
    call!(tag_block_end, s) >>
    block: call!(parse_template, s) >>
    (WS(pws.is_some(), nws.is_some()), cond, block)
));

named_args!(block_if<'a>(s: &'a Syntax<'a>) <Input<'a>, Node<'a>>, do_parse!(
    pws1: opt!(tag!("-")) >>
    cond: ws!(cond_if) >>
    nws1: opt!(tag!("-")) >>
    call!(tag_block_end, s) >>
    block: call!(parse_template, s) >>
    elifs: many0!(call!(cond_block, s)) >>
    call!(tag_block_start, s) >>
    pws2: opt!(tag!("-")) >>
    ws!(tag!("endif")) >>
    nws2: opt!(tag!("-")) >>
    ({
       let mut res = Vec::new();
       res.push((WS(pws1.is_some(), nws1.is_some()), Some(cond), block));
       res.extend(elifs);
       Node::Cond(res, WS(pws2.is_some(), nws2.is_some()))
    })
));

named_args!(match_else_block<'a>(s: &'a Syntax<'a>) <Input<'a>, When<'a>>, do_parse!(
    call!(tag_block_start, s) >>
    pws: opt!(tag!("-")) >>
    ws!(tag!("else")) >>
    nws: opt!(tag!("-")) >>
    call!(tag_block_end, s) >>
    block: call!(parse_template, s) >>
    (WS(pws.is_some(), nws.is_some()), None, MatchParameters::Simple(vec![]), block)
));

named_args!(when_block<'a>(s: &'a Syntax<'a>) <Input<'a>, When<'a>>, do_parse!(
    call!(tag_block_start, s) >>
    pws: opt!(tag!("-")) >>
    ws!(tag!("when")) >>
    variant: ws!(match_variant) >>
    params: opt!(ws!(with_parameters)) >>
    nws: opt!(tag!("-")) >>
    call!(tag_block_end, s) >>
    block: call!(parse_template, s) >>
    (WS(pws.is_some(), nws.is_some()), Some(variant), params.unwrap_or_default(), block)
));

named_args!(block_match<'a>(s: &'a Syntax<'a>) <Input<'a>, Node<'a>>, do_parse!(
    pws1: opt!(tag!("-")) >>
    ws!(tag!("match")) >>
    expr: ws!(expr_any) >>
    nws1: opt!(tag!("-")) >>
    call!(tag_block_end, s) >>
    inter: opt!(call!(take_content, s)) >>
    arms: many1!(call!(when_block, s)) >>
    else_arm: opt!(call!(match_else_block, s)) >>
    ws!(call!(tag_block_start, s)) >>
    pws2: opt!(tag!("-")) >>
    ws!(tag!("endmatch")) >>
    nws2: opt!(tag!("-")) >>
    ({
        let mut arms = arms;
        if let Some(arm) = else_arm {
            arms.push(arm);
        }
        let inter = match inter {
            Some(Node::Lit(lws, val, rws)) => {
                assert!(val.is_empty(),
                        "only whitespace allowed between match and first when, found {}", val);
                assert!(rws.is_empty(),
                        "only whitespace allowed between match and first when, found {}", rws);
                Some(lws)
            },
            None => None,
            _ => panic!("only literals allowed between match and first when"),
        };
        Node::Match(
            WS(pws1.is_some(), nws1.is_some()),
            expr,
            inter,
            arms,
            WS(pws2.is_some(), nws2.is_some()),
        )
    })
));

named!(block_let<Input, Node>, do_parse!(
    pws: opt!(tag!("-")) >>
    ws!(tag!("let")) >>
    var: ws!(alt!(target_single | target_tuple)) >>
    val: opt!(do_parse!(
        ws!(tag!("=")) >>
        val: ws!(expr_any) >>
        (val)
    )) >>
    nws: opt!(tag!("-")) >>
    (if val.is_some() {
        Node::Let(WS(pws.is_some(), nws.is_some()), var, val.unwrap())
    } else {
        Node::LetDecl(WS(pws.is_some(), nws.is_some()), var)
    })
));

named_args!(block_for<'a>(s: &'a Syntax<'a>) <Input<'a>, Node<'a>>, do_parse!(
    pws1: opt!(tag!("-")) >>
    ws!(tag!("for")) >>
    var: ws!(alt!(target_single | target_tuple)) >>
    ws!(tag!("in")) >>
    iter: ws!(expr_any) >>
    nws1: opt!(tag!("-")) >>
    call!(tag_block_end, s) >>
    block: call!(parse_template, s) >>
    call!(tag_block_start, s) >>
    pws2: opt!(tag!("-")) >>
    ws!(tag!("endfor")) >>
    nws2: opt!(tag!("-")) >>
    (Node::Loop(WS(pws1.is_some(), nws1.is_some()),
                var, iter, block,
                WS(pws2.is_some(), nws2.is_some())))
));

named!(block_extends<Input, Node>, do_parse!(
    ws!(tag!("extends")) >>
    name: ws!(expr_str_lit) >>
    (Node::Extends(name))
));

named_args!(block_block<'a>(s: &'a Syntax<'a>) <Input<'a>, Node<'a>>, do_parse!(
    pws1: opt!(tag!("-")) >>
    ws!(tag!("block")) >>
    name: ws!(identifier) >>
    nws1: opt!(tag!("-")) >>
    call!(tag_block_end, s) >>
    contents: call!(parse_template, s) >>
    call!(tag_block_start, s) >>
    pws2: opt!(tag!("-")) >>
    ws!(tag!("endblock")) >>
    opt!(ws!(tag!(name))) >>
    nws2: opt!(tag!("-")) >>
    (Node::BlockDef(WS(pws1.is_some(), nws1.is_some()),
                    name, contents,
                    WS(pws2.is_some(), nws2.is_some())))
));

named!(block_include<Input, Node>, do_parse!(
    pws: opt!(tag!("-")) >>
    ws!(tag!("include")) >>
    name: ws!(expr_str_lit) >>
    nws: opt!(tag!("-")) >>
    (Node::Include(WS(pws.is_some(), nws.is_some()), match name {
        Expr::StrLit(s) => s,
        _ => panic!("include path must be a string literal"),
    }))
));

named!(block_import<Input, Node>, do_parse!(
    pws: opt!(tag!("-")) >>
    ws!(tag!("import")) >>
    name: ws!(expr_str_lit) >>
    ws!(tag!("as")) >>
    scope: ws!(identifier) >>
    nws: opt!(tag!("-")) >>
    (Node::Import(WS(pws.is_some(), nws.is_some()), match name {
        Expr::StrLit(s) => s,
        _ => panic!("import path must be a string literal"),
    }, scope))
));

named_args!(block_macro<'a>(s: &'a Syntax<'a>) <Input<'a>, Node<'a>>, do_parse!(
    pws1: opt!(tag!("-")) >>
    ws!(tag!("macro")) >>
    name: ws!(identifier) >>
    params: ws!(parameters) >>
    nws1: opt!(tag!("-")) >>
    call!(tag_block_end, s) >>
    contents: call!(parse_template, s) >>
    call!(tag_block_start, s) >>
    pws2: opt!(tag!("-")) >>
    ws!(tag!("endmacro")) >>
    nws2: opt!(tag!("-")) >>
    ({
        if name == "super" {
            panic!("invalid macro name 'super'");
        }
        Node::Macro(
            name,
            Macro {
                ws1: WS(pws1.is_some(), nws1.is_some()),
                args: params,
                nodes: contents,
                ws2: WS(pws2.is_some(), nws2.is_some())
            }
        )
    })
));

named_args!(block_raw<'a>(s: &'a Syntax<'a>) <Input<'a>, Node<'a>>, do_parse!(
    pws1: opt!(tag!("-")) >>
    ws!(tag!("raw")) >>
    nws1: opt!(tag!("-")) >>
    call!(tag_block_end, s) >>
    contents: take_until!("{% endraw %}") >>
    call!(tag_block_start, s) >>
    pws2: opt!(tag!("-")) >>
    ws!(tag!("endraw")) >>
    nws2: opt!(tag!("-")) >>
    ({
        let str_contents = str::from_utf8(&contents).unwrap();
        (Node::Raw(WS(pws1.is_some(), nws1.is_some()),
                   str_contents,
                   WS(pws2.is_some(), nws2.is_some())))
    })
));

named_args!(block_node<'a>(s: &'a Syntax<'a>) <Input<'a>, Node<'a>>, do_parse!(
    call!(tag_block_start, s) >>
    contents: alt!(
        block_call |
        block_let |
        call!(block_if, s) |
        call!(block_for, s) |
        call!(block_match, s) |
        block_extends |
        block_include |
        block_import |
        call!(block_block, s) |
        call!(block_macro, s) |
        call!(block_raw, s)
    ) >>
    call!(tag_block_end, s) >>
    (contents)
));

named_args!(block_comment<'a>(s: &'a Syntax<'a>) <Input<'a>, Node<'a>>, do_parse!(
    call!(tag_comment_start, s)  >>
    pws: opt!(tag!("-")) >>
    inner: take_until_s!(s.comment_end) >>
    call!(tag_comment_end, s) >>
    (Node::Comment(WS(pws.is_some(), inner.len() > 1 && inner[inner.len() - 1] == b'-')))
));

named_args!(parse_template<'a>(s: &'a Syntax<'a>)<Input<'a>, Vec<Node<'a>>>, many0!(alt!(
    call!(take_content, s) |
    call!(block_comment, s) |
    call!(expr_node, s) |
    call!(block_node, s)
)));

named_args!(tag_block_start<'a>(s: &'a Syntax<'a>) <Input<'a>, Input<'a>>, tag!(s.block_start));
named_args!(tag_block_end<'a>(s: &'a Syntax<'a>) <Input<'a>, Input<'a>>, tag!(s.block_end));
named_args!(tag_comment_start<'a>(s: &'a Syntax<'a>) <Input<'a>, Input<'a>>, tag!(s.comment_start));
named_args!(tag_comment_end<'a>(s: &'a Syntax<'a>) <Input<'a>, Input<'a>>, tag!(s.comment_end));
named_args!(tag_expr_start<'a>(s: &'a Syntax<'a>) <Input<'a>, Input<'a>>, tag!(s.expr_start));
named_args!(tag_expr_end<'a>(s: &'a Syntax<'a>) <Input<'a>, Input<'a>>, tag!(s.expr_end));

pub fn parse<'a>(src: &'a str, syntax: &'a Syntax<'a>) -> Vec<Node<'a>> {
    match parse_template(Input(src.as_bytes()), syntax) {
        Ok((left, res)) => {
            if !left.is_empty() {
                let s = str::from_utf8(left.0).unwrap();
                panic!("unable to parse template:\n\n{:?}", s);
            } else {
                res
            }
        }
        Err(nom::Err::Error(err)) => panic!("problems parsing template source: {:?}", err),
        Err(nom::Err::Failure(err)) => panic!("problems parsing template source: {:?}", err),
        Err(nom::Err::Incomplete(_)) => panic!("parsing incomplete"),
    }
}

#[cfg(test)]
mod tests {
    use askama_shared::Syntax;

    fn check_ws_split(s: &str, res: &(&str, &str, &str)) {
        let node = super::split_ws_parts(s.as_bytes());
        match node {
            super::Node::Lit(lws, s, rws) => {
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
        super::parse("{% extend \"blah\" %}", &Syntax::default());
    }

    #[test]
    fn test_parse_filter() {
        super::parse("{{ strvar|e }}", &Syntax::default());
    }

    #[test]
    fn change_delimiters_parse_filter() {
        let syntax = Syntax {
            expr_start: "{~",
            expr_end: "~}",
            ..Syntax::default()
        };

        super::parse("{~ strvar|e ~}", &syntax);
    }
}
