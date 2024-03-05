use std::fmt::{Display, Formatter, Write};

use arbitrary::Arbitrary;

use super::expr::Expr;
use super::strings::{Ident, Printable, PrintableNoGrouping, Space, TypeName};

#[derive(Debug, Arbitrary)]
pub struct Node(NodeInner);

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Arbitrary)]
enum NodeInner {
    Text(Box<Text>),
    Comment(Box<Comment>),
    NodeExpr(Box<NodeExpr>),
    Call(Box<Call>),
    Let(Box<Let>),
    If(Box<If>),
    Match(Box<Match>),
    For(Box<For>),
    Extends(Box<Extends>),
    BlockDef(Box<BlockDef>),
    Include(Box<Include>),
    Import(Box<Import>),
    Macro(Box<Macro>),
    Raw(Box<Raw>),
    // TODO: Continue
    // TODO: Break
}

impl Display for NodeInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(v) => v.fmt(f),
            Self::Comment(v) => v.fmt(f),
            Self::NodeExpr(v) => v.fmt(f),
            Self::Call(v) => v.fmt(f),
            Self::Let(v) => v.fmt(f),
            Self::If(v) => v.fmt(f),
            Self::Match(v) => v.fmt(f),
            Self::For(v) => v.fmt(f),
            Self::Extends(v) => v.fmt(f),
            Self::BlockDef(v) => v.fmt(f),
            Self::Include(v) => v.fmt(f),
            Self::Import(v) => v.fmt(f),
            Self::Macro(v) => v.fmt(f),
            Self::Raw(v) => v.fmt(f),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Text {
    spaces: [Space; 2],
    content: Printable,
}

impl Display for Text {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.spaces[0].fmt(f)?;
        self.content.fmt(f)?;
        self.spaces[1].fmt(f)?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Comment {
    ws: [Whitespace; 2],
    spaces: [Space; 2],
    expr: NodeInner,
}

impl Display for Comment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{#")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        self.expr.fmt(f)?;
        self.spaces[1].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("#}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct NodeExpr {
    ws: [Whitespace; 2],
    spaces: [Space; 2],
    expr: Expr,
}

impl Display for NodeExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{{")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        self.expr.fmt(f)?;
        self.spaces[1].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("}}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Call {
    ws: [Whitespace; 2],
    spaces: [Space; 6],
    scope: Option<(Ident, [Space; 2])>,
    name: Ident,
    args: Vec<(Expr, [Space; 2])>,
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_str("call ")?;
        self.spaces[1].fmt(f)?;
        if let Some((scope, spaces)) = &self.scope {
            scope.fmt(f)?;
            spaces[0].fmt(f)?;
            f.write_str("::")?;
            spaces[1].fmt(f)?;
        }
        self.name.fmt(f)?;
        self.spaces[2].fmt(f)?;
        f.write_char('(')?;
        self.spaces[3].fmt(f)?;
        for (idx, (expr, spaces)) in self.args.iter().enumerate() {
            if idx > 0 {
                f.write_char(',')?;
                spaces[0].fmt(f)?;
            }
            expr.fmt(f)?;
            spaces[1].fmt(f)?;
        }
        self.spaces[4].fmt(f)?;
        f.write_char(')')?;
        self.spaces[5].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Let {
    ws: [Whitespace; 2],
    spaces: [Space; 3],
    var: Target,
    val: Option<([Space; 2], Expr)>,
}

impl Display for Let {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_str("let ")?;
        self.spaces[1].fmt(f)?;
        self.var.fmt(f)?;
        if let Some((spaces, val)) = &self.val {
            spaces[0].fmt(f)?;
            f.write_char('=')?;
            spaces[1].fmt(f)?;
            val.fmt(f)?;
        }
        self.spaces[2].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct If {
    ws: [Whitespace; 2],
    spaces: [Space; 1],
    first: ElseIf,
    else_ifs: Vec<(Space, ElseIf)>,
    r#else: Option<([Space; 3], [Whitespace; 2], Vec<(Space, NodeInner)>)>,
}

impl Display for If {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.first.fmt(f)?;

        for (space, else_if) in &self.else_ifs {
            f.write_str("else ")?;
            space.fmt(f)?;
            else_if.fmt(f)?;
        }

        if let Some((spaces, ws, nodes)) = &self.r#else {
            f.write_str("else ")?;
            spaces[0].fmt(f)?;
            ws[0].fmt(f)?;
            f.write_str("%}")?;
            for (space, node) in nodes {
                space.fmt(f)?;
                node.fmt(f)?;
            }
            spaces[1].fmt(f)?;
            f.write_str("{%")?;
            ws[1].fmt(f)?;
            spaces[2].fmt(f)?;
        }

        f.write_str("endif")?;
        self.spaces[0].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct ElseIf {
    ws: [Whitespace; 2],
    spaces: [Space; 4],
    target: Option<([Space; 3], Target)>,
    expr: Expr,
    nodes: Vec<(Space, NodeInner)>,
}

impl Display for ElseIf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.spaces[0].fmt(f)?;
        f.write_str("if ")?;
        self.spaces[0].fmt(f)?;
        if let Some((spaces, target)) = &self.target {
            f.write_str("let ")?;
            spaces[0].fmt(f)?;
            target.fmt(f)?;
            spaces[1].fmt(f)?;
            f.write_char('=')?;
            spaces[2].fmt(f)?;
        }
        self.expr.fmt(f)?;
        self.spaces[1].fmt(f)?;
        self.ws[0].fmt(f)?;
        f.write_str("%}")?;
        for (space, node) in &self.nodes {
            space.fmt(f)?;
            node.fmt(f)?;
        }
        self.spaces[2].fmt(f)?;
        f.write_str("{%")?;
        self.ws[1].fmt(f)?;
        self.spaces[3].fmt(f)?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Match {
    ws: [Whitespace; 4],
    spaces: [Space; 7],
    expr: Expr,
    comments: Vec<(Space, Comment)>,
    when: When,
    alternatives: Vec<When>,
    r#else: Option<([Whitespace; 2], [Space; 3], Vec<(Space, NodeInner)>)>,
}

impl Display for Match {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_str("match ")?;
        self.spaces[1].fmt(f)?;
        self.expr.fmt(f)?;
        self.spaces[2].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;
        self.spaces[4].fmt(f)?;

        for (space, comment) in &self.comments {
            comment.fmt(f)?;
            space.fmt(f)?;
        }

        self.when.fmt(f)?;
        for alt in &self.alternatives {
            alt.fmt(f)?;
        }

        if let Some((ws, spaces, body)) = &self.r#else {
            f.write_str("{%")?;
            ws[0].fmt(f)?;
            spaces[0].fmt(f)?;
            f.write_str("else")?;
            spaces[1].fmt(f)?;
            ws[1].fmt(f)?;
            f.write_str("%}")?;
            spaces[2].fmt(f)?;
            for (space, body) in body {
                body.fmt(f)?;
                space.fmt(f)?;
            }
        }

        f.write_str("{%")?;
        self.ws[2].fmt(f)?;
        self.spaces[5].fmt(f)?;
        f.write_str("endmatch")?;
        self.spaces[6].fmt(f)?;
        self.ws[3].fmt(f)?;
        f.write_str("%}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct When {
    ws: [Whitespace; 2],
    spaces: [Space; 4],
    target: Target,
    nodes: Vec<(Space, NodeInner)>,
}

impl Display for When {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_str("when ")?;
        self.spaces[1].fmt(f)?;
        self.target.fmt(f)?;
        self.spaces[2].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;
        self.spaces[3].fmt(f)?;
        for (space, node) in &self.nodes {
            node.fmt(f)?;
            space.fmt(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct For {
    ws: [Whitespace; 4],
    spaces: [Space; 9],
    var: Target,
    val: Expr,
    cond: Option<([Space; 2], Expr)>,
    body: Vec<(Space, NodeInner)>,
    r#else: Option<([Whitespace; 2], [Space; 3], Vec<(Space, Expr)>)>,
}

impl Display for For {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_str("for ")?;
        self.spaces[1].fmt(f)?;
        self.var.fmt(f)?;
        self.spaces[2].fmt(f)?;
        f.write_str(" in ")?;
        self.spaces[3].fmt(f)?;
        self.val.fmt(f)?;
        self.spaces[4].fmt(f)?;
        if let Some((spaces, expr)) = &self.cond {
            f.write_str(" if ")?;
            spaces[0].fmt(f)?;
            expr.fmt(f)?;
            spaces[1].fmt(f)?;
        }
        self.spaces[5].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;

        self.spaces[6].fmt(f)?;
        for (space, node) in &self.body {
            node.fmt(f)?;
            space.fmt(f)?;
        }

        if let Some((ws, spaces, body)) = &self.r#else {
            f.write_str("{%")?;
            ws[0].fmt(f)?;
            spaces[0].fmt(f)?;
            f.write_str("else")?;
            spaces[1].fmt(f)?;
            ws[1].fmt(f)?;
            f.write_str("%}")?;

            spaces[2].fmt(f)?;
            for (space, node) in body {
                node.fmt(f)?;
                space.fmt(f)?;
            }
        }

        f.write_str("{%")?;
        self.ws[2].fmt(f)?;
        self.spaces[7].fmt(f)?;
        f.write_str("endfor")?;
        self.spaces[8].fmt(f)?;
        self.ws[3].fmt(f)?;
        f.write_str("%}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Extends {
    spaces: [Space; 3],
    path: FilePath,
}

impl Display for Extends {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.spaces[0].fmt(f)?;
        f.write_str("extends ")?;
        self.spaces[1].fmt(f)?;
        self.path.fmt(f)?;
        self.spaces[2].fmt(f)?;
        f.write_str("%}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct BlockDef {
    ws: [Whitespace; 4],
    spaces: [Space; 6],
    name: Ident,
    repeat_name: Option<Space>,
    nodes: Vec<(Space, NodeInner)>,
}

impl Display for BlockDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_str("block ")?;
        self.spaces[1].fmt(f)?;
        self.name.fmt(f)?;
        self.spaces[2].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;

        for (space, node) in &self.nodes {
            space.fmt(f)?;
            node.fmt(f)?;
        }

        self.spaces[3].fmt(f)?;
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.spaces[4].fmt(f)?;
        f.write_str("endblock")?;
        if let Some(space) = &self.repeat_name {
            f.write_char(' ')?;
            space.fmt(f)?;
            self.name.fmt(f)?;
        }
        self.spaces[5].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Include {
    ws: [Whitespace; 2],
    spaces: [Space; 3],
    path: FilePath,
}

impl Display for Include {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_str("include")?;
        self.spaces[1].fmt(f)?;
        self.path.fmt(f)?;
        self.spaces[2].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Import {
    ws: [Whitespace; 2],
    spaces: [Space; 5],
    path: FilePath,
    name: Ident,
}

impl Display for Import {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_str("import")?;
        self.spaces[1].fmt(f)?;
        self.path.fmt(f)?;
        self.spaces[2].fmt(f)?;
        f.write_str("as ")?;
        self.spaces[3].fmt(f)?;
        self.name.fmt(f)?;
        self.spaces[4].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Macro {
    ws: [Whitespace; 4],
    spaces: [Space; 6],
    name: Ident,
    repeat_name: Option<Space>,
    args: Option<([Space; 2], Vec<([Space; 2], Ident)>)>,
    nodes: Vec<(Space, NodeInner)>,
}

impl Display for Macro {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_str("macro ")?;
        self.spaces[1].fmt(f)?;
        self.name.fmt(f)?;
        if let Some((spaces, args)) = &self.args {
            spaces[0].fmt(f)?;
            f.write_char('(')?;
            for (idx, (spaces, name)) in args.iter().enumerate() {
                if idx > 0 {
                    f.write_char(',')?;
                    spaces[0].fmt(f)?;
                }
                name.fmt(f)?;
                spaces[1].fmt(f)?;
            }
            spaces[1].fmt(f)?;
            f.write_char(')')?;
        }
        self.spaces[2].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;

        for (space, node) in &self.nodes {
            space.fmt(f)?;
            node.fmt(f)?;
        }

        self.spaces[3].fmt(f)?;
        f.write_str("{%")?;
        self.ws[2].fmt(f)?;
        self.spaces[4].fmt(f)?;
        f.write_str("endmacro")?;
        if let Some(space) = &self.repeat_name {
            space.fmt(f)?;
            f.write_char(' ')?;
            self.name.fmt(f)?;
        }
        self.spaces[5].fmt(f)?;
        self.ws[3].fmt(f)?;
        f.write_str("%}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Raw {
    ws: [Whitespace; 4],
    spaces: [Space; 6],
    inner: PrintableNoGrouping,
}

impl Display for Raw {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{%")?;
        self.ws[0].fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_str("raw ")?;
        self.spaces[1].fmt(f)?;
        self.ws[1].fmt(f)?;
        f.write_str("%}")?;

        self.spaces[2].fmt(f)?;
        self.inner.fmt(f)?;
        self.spaces[3].fmt(f)?;

        f.write_str("{%")?;
        self.ws[2].fmt(f)?;
        self.spaces[4].fmt(f)?;
        f.write_str("endraw")?;
        self.spaces[5].fmt(f)?;
        self.ws[3].fmt(f)?;
        f.write_str("%}")?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
enum Target {
    Name(Ident),
    Tuple(Tuple),
    Struct(Struct),
    Lit(Lit),
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(name) => name.fmt(f),
            Self::Tuple(t) => t.fmt(f),
            Self::Struct(s) => s.fmt(f),
            Self::Lit(l) => l.fmt(f),
        }
    }
}

#[derive(Debug, Arbitrary)]
enum Lit {
    Int(isize),
    Float(f32),
    String(Printable),
    Char(char),
    Bool(bool),
    Path(TypeName),
}

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "{:?}", &v.0),
            Self::Char(v) => write!(f, "{v:?}"),
            Self::Bool(v) => write!(f, "{v:?}"),
            Self::Path(v) => v.fmt(f),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Tuple {
    spaces: [Space; 2],
    type_name: Option<(Space, TypeName)>,
    fields: Vec<([Space; 2], Ident)>,
}

impl Display for Tuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some((space, type_name)) = &self.type_name {
            type_name.fmt(f)?;
            space.fmt(f)?;
        }
        f.write_char('(')?;
        self.spaces[0].fmt(f)?;
        for (idx, (spaces, ident)) in self.fields.iter().enumerate() {
            if idx > 0 {
                f.write_char(',')?;
                spaces[0].fmt(f)?;
            }
            ident.fmt(f)?;
            spaces[1].fmt(f)?;
        }
        self.spaces[1].fmt(f)?;
        f.write_char(')')?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Struct {
    spaces: [Space; 2],
    type_name: TypeName,
    fields: Vec<([Space; 2], Ident, Option<([Space; 2], Target)>)>,
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.type_name.fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_char('{')?;
        for (idx, (spaces, ident, target)) in self.fields.iter().enumerate() {
            if idx > 0 {
                f.write_char(',')?;
                spaces[0].fmt(f)?;
            }
            ident.fmt(f)?;
            spaces[1].fmt(f)?;
            if let Some((spaces, target)) = target {
                spaces[0].fmt(f)?;
                f.write_char(':')?;
                spaces[1].fmt(f)?;
                target.fmt(f)?;
            }
        }
        self.spaces[1].fmt(f)?;
        f.write_char('}')?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
enum Whitespace {
    Default,
    Preserve,
    Suppress,
    Minimize,
}

impl Display for Whitespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => Ok(()),
            Self::Preserve => f.write_char('+'),
            Self::Suppress => f.write_char('-'),
            Self::Minimize => f.write_char('~'),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct FilePath(Printable);

impl Display for FilePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.0 .0)
    }
}
