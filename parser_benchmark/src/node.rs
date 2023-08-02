use std::fmt::Write;

use arbitrary::Arbitrary;

use super::expr::Expr;
use super::strings::{Ident, Printable, PrintableNoGrouping, Space, TypeName};
use super::ToSource;

#[derive(Debug, Arbitrary)]
pub(super) struct Node(NodeInner);

impl ToSource for Node {
    fn write_into(&self, buf: &mut String) {
        self.0.write_into(buf);
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

impl ToSource for NodeInner {
    fn write_into(&self, buf: &mut String) {
        match self {
            Self::Text(v) => v.write_into(buf),
            Self::Comment(v) => v.write_into(buf),
            Self::NodeExpr(v) => v.write_into(buf),
            Self::Call(v) => v.write_into(buf),
            Self::Let(v) => v.write_into(buf),
            Self::If(v) => v.write_into(buf),
            Self::Match(v) => v.write_into(buf),
            Self::For(v) => v.write_into(buf),
            Self::Extends(v) => v.write_into(buf),
            Self::BlockDef(v) => v.write_into(buf),
            Self::Include(v) => v.write_into(buf),
            Self::Import(v) => v.write_into(buf),
            Self::Macro(v) => v.write_into(buf),
            Self::Raw(v) => v.write_into(buf),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Text {
    spaces: [Space; 2],
    content: Printable,
}

impl ToSource for Text {
    fn write_into(&self, buf: &mut String) {
        self.spaces[0].write_into(buf);
        self.content.write_into(buf);
        self.spaces[1].write_into(buf);
    }
}

#[derive(Debug, Arbitrary)]
struct Comment {
    ws: [Whitespace; 2],
    spaces: [Space; 2],
    expr: NodeInner,
}

impl ToSource for Comment {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{#");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        self.expr.write_into(buf);
        self.spaces[1].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("#}");
    }
}

#[derive(Debug, Arbitrary)]
struct NodeExpr {
    ws: [Whitespace; 2],
    spaces: [Space; 2],
    expr: Expr,
}

impl ToSource for NodeExpr {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{{");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        self.expr.write_into(buf);
        self.spaces[1].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("}}");
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

impl ToSource for Call {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push_str("call ");
        self.spaces[1].write_into(buf);
        if let Some((scope, spaces)) = &self.scope {
            scope.write_into(buf);
            spaces[0].write_into(buf);
            buf.push_str("::");
            spaces[1].write_into(buf);
        }
        self.name.write_into(buf);
        self.spaces[2].write_into(buf);
        buf.push('(');
        self.spaces[3].write_into(buf);
        if !self.args.is_empty() {
            for (expr, spaces) in &self.args {
                spaces[0].write_into(buf);
                expr.write_into(buf);
                spaces[1].write_into(buf);
                buf.push(',')
            }
            buf.pop();
        }
        self.spaces[4].write_into(buf);
        buf.push(')');
        self.spaces[5].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");
    }
}

#[derive(Debug, Arbitrary)]
struct Let {
    ws: [Whitespace; 2],
    spaces: [Space; 3],
    var: Target,
    val: Option<([Space; 2], Expr)>,
}

impl ToSource for Let {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push_str("let ");
        self.spaces[1].write_into(buf);
        self.var.write_into(buf);
        if let Some((spaces, val)) = &self.val {
            spaces[0].write_into(buf);
            buf.push('=');
            spaces[1].write_into(buf);
            val.write_into(buf);
        }
        self.spaces[2].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");
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

impl ToSource for If {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.first.write_into(buf);

        for (space, else_if) in &self.else_ifs {
            buf.push_str("else ");
            space.write_into(buf);
            else_if.write_into(buf);
        }

        if let Some((spaces, ws, nodes)) = &self.r#else {
            buf.push_str("else ");
            spaces[0].write_into(buf);
            ws[0].write_into(buf);
            buf.push_str("%}");
            for (space, node) in nodes {
                space.write_into(buf);
                node.write_into(buf);
            }
            spaces[1].write_into(buf);
            buf.push_str("{%");
            ws[1].write_into(buf);
            spaces[2].write_into(buf);
        }

        buf.push_str("endif");
        self.spaces[0].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");
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

impl ToSource for ElseIf {
    fn write_into(&self, buf: &mut String) {
        self.spaces[0].write_into(buf);
        buf.push_str("if ");
        self.spaces[0].write_into(buf);
        if let Some((spaces, target)) = &self.target {
            buf.push_str("let ");
            spaces[0].write_into(buf);
            target.write_into(buf);
            spaces[1].write_into(buf);
            buf.push('=');
            spaces[2].write_into(buf);
        }
        self.expr.write_into(buf);
        self.spaces[1].write_into(buf);
        self.ws[0].write_into(buf);
        buf.push_str("%}");
        for (space, node) in &self.nodes {
            space.write_into(buf);
            node.write_into(buf);
        }
        self.spaces[2].write_into(buf);
        buf.push_str("{%");
        self.ws[1].write_into(buf);
        self.spaces[3].write_into(buf);
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

impl ToSource for Match {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push_str("match ");
        self.spaces[1].write_into(buf);
        self.expr.write_into(buf);
        self.spaces[2].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");
        self.spaces[4].write_into(buf);

        for (space, comment) in &self.comments {
            comment.write_into(buf);
            space.write_into(buf);
        }

        self.when.write_into(buf);
        for alt in &self.alternatives {
            alt.write_into(buf);
        }

        if let Some((ws, spaces, body)) = &self.r#else {
            buf.push_str("{%");
            ws[0].write_into(buf);
            spaces[0].write_into(buf);
            buf.push_str("else");
            spaces[1].write_into(buf);
            ws[1].write_into(buf);
            buf.push_str("%}");
            spaces[2].write_into(buf);
            for (space, body) in body {
                body.write_into(buf);
                space.write_into(buf);
            }
        }

        buf.push_str("{%");
        self.ws[2].write_into(buf);
        self.spaces[5].write_into(buf);
        buf.push_str("endmatch");
        self.spaces[6].write_into(buf);
        self.ws[3].write_into(buf);
        buf.push_str("%}");
    }
}

#[derive(Debug, Arbitrary)]
struct When {
    ws: [Whitespace; 2],
    spaces: [Space; 4],
    target: Target,
    nodes: Vec<(Space, NodeInner)>,
}

impl ToSource for When {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push_str("when ");
        self.spaces[1].write_into(buf);
        self.target.write_into(buf);
        self.spaces[2].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");
        self.spaces[3].write_into(buf);
        for (space, node) in &self.nodes {
            node.write_into(buf);
            space.write_into(buf);
        }
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

impl ToSource for For {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push_str("for ");
        self.spaces[1].write_into(buf);
        self.var.write_into(buf);
        self.spaces[2].write_into(buf);
        buf.push_str(" in ");
        self.spaces[3].write_into(buf);
        self.val.write_into(buf);
        self.spaces[4].write_into(buf);
        if let Some((spaces, expr)) = &self.cond {
            buf.push_str(" if ");
            spaces[0].write_into(buf);
            expr.write_into(buf);
            spaces[1].write_into(buf);
        }
        self.spaces[5].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");

        self.spaces[6].write_into(buf);
        for (space, node) in &self.body {
            node.write_into(buf);
            space.write_into(buf);
        }

        if let Some((ws, spaces, body)) = &self.r#else {
            buf.push_str("{%");
            ws[0].write_into(buf);
            spaces[0].write_into(buf);
            buf.push_str("else");
            spaces[1].write_into(buf);
            ws[1].write_into(buf);
            buf.push_str("%}");

            spaces[2].write_into(buf);
            for (space, node) in body {
                node.write_into(buf);
                space.write_into(buf);
            }
        }

        buf.push_str("{%");
        self.ws[2].write_into(buf);
        self.spaces[7].write_into(buf);
        buf.push_str("endfor");
        self.spaces[8].write_into(buf);
        self.ws[3].write_into(buf);
        buf.push_str("%}");
    }
}

#[derive(Debug, Arbitrary)]
struct Extends {
    spaces: [Space; 3],
    path: FilePath,
}

impl ToSource for Extends {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.spaces[0].write_into(buf);
        buf.push_str("extends ");
        self.spaces[1].write_into(buf);
        self.path.write_into(buf);
        self.spaces[2].write_into(buf);
        buf.push_str("%}");
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

impl ToSource for BlockDef {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push_str("block ");
        self.spaces[1].write_into(buf);
        self.name.write_into(buf);
        self.spaces[2].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");

        for (space, node) in &self.nodes {
            space.write_into(buf);
            node.write_into(buf);
        }

        self.spaces[3].write_into(buf);
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.spaces[4].write_into(buf);
        buf.push_str("endblock");
        if let Some(space) = &self.repeat_name {
            buf.push(' ');
            space.write_into(buf);
            self.name.write_into(buf);
        }
        self.spaces[5].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");
    }
}

#[derive(Debug, Arbitrary)]
struct Include {
    ws: [Whitespace; 2],
    spaces: [Space; 3],
    path: FilePath,
}

impl ToSource for Include {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push_str("include");
        self.spaces[1].write_into(buf);
        self.path.write_into(buf);
        self.spaces[2].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");
    }
}

#[derive(Debug, Arbitrary)]
struct Import {
    ws: [Whitespace; 2],
    spaces: [Space; 5],
    path: FilePath,
    name: Ident,
}

impl ToSource for Import {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push_str("import");
        self.spaces[1].write_into(buf);
        self.path.write_into(buf);
        self.spaces[2].write_into(buf);
        buf.push_str("as ");
        self.spaces[3].write_into(buf);
        self.name.write_into(buf);
        self.spaces[4].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");
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

impl ToSource for Macro {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push_str("macro ");
        self.spaces[1].write_into(buf);
        self.name.write_into(buf);
        if let Some((spaces, args)) = &self.args {
            spaces[0].write_into(buf);
            buf.push('(');
            if !args.is_empty() {
                for (spaces, name) in args {
                    spaces[0].write_into(buf);
                    name.write_into(buf);
                    spaces[1].write_into(buf);
                    buf.push(',')
                }
                buf.pop();
            }
            spaces[1].write_into(buf);
            buf.push(')');
        }
        self.spaces[2].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");

        for (space, node) in &self.nodes {
            space.write_into(buf);
            node.write_into(buf);
        }

        self.spaces[3].write_into(buf);
        buf.push_str("{%");
        self.ws[2].write_into(buf);
        self.spaces[4].write_into(buf);
        buf.push_str("endmacro");
        if let Some(space) = &self.repeat_name {
            space.write_into(buf);
            buf.push(' ');
            self.name.write_into(buf);
        }
        self.spaces[5].write_into(buf);
        self.ws[3].write_into(buf);
        buf.push_str("%}");
    }
}

#[derive(Debug, Arbitrary)]
struct Raw {
    ws: [Whitespace; 4],
    spaces: [Space; 6],
    inner: PrintableNoGrouping,
}

impl ToSource for Raw {
    fn write_into(&self, buf: &mut String) {
        buf.push_str("{%");
        self.ws[0].write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push_str("raw ");
        self.spaces[1].write_into(buf);
        self.ws[1].write_into(buf);
        buf.push_str("%}");

        self.spaces[2].write_into(buf);
        self.inner.write_into(buf);
        self.spaces[3].write_into(buf);

        buf.push_str("{%");
        self.ws[2].write_into(buf);
        self.spaces[4].write_into(buf);
        buf.push_str("endraw");
        self.spaces[5].write_into(buf);
        self.ws[3].write_into(buf);
        buf.push_str("%}");
    }
}

#[derive(Debug, Arbitrary)]
enum Target {
    Name(Ident),
    Tuple(Tuple),
    Struct(Struct),
    Lit(Lit),
}

impl ToSource for Target {
    fn write_into(&self, buf: &mut String) {
        match self {
            Self::Name(name) => name.write_into(buf),
            Self::Tuple(t) => t.write_into(buf),
            Self::Struct(s) => s.write_into(buf),
            Self::Lit(l) => l.write_into(buf),
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

impl ToSource for Lit {
    fn write_into(&self, buf: &mut String) {
        match self {
            Self::Int(v) => write!(buf, "{v}").unwrap(),
            Self::Float(v) => write!(buf, "{v}").unwrap(),
            Self::String(v) => write!(buf, "{:?}", &v.0).unwrap(),
            Self::Char(v) => write!(buf, "{v:?}").unwrap(),
            Self::Bool(v) => write!(buf, "{v:?}").unwrap(),
            Self::Path(v) => v.write_into(buf),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Tuple {
    spaces: [Space; 2],
    type_name: Option<(Space, TypeName)>,
    fields: Vec<([Space; 2], Ident)>,
}

impl ToSource for Tuple {
    fn write_into(&self, buf: &mut String) {
        if let Some((space, type_name)) = &self.type_name {
            type_name.write_into(buf);
            space.write_into(buf);
        }
        buf.push('(');
        self.spaces[0].write_into(buf);
        if !self.fields.is_empty() {
            for (spaces, ident) in &self.fields {
                spaces[0].write_into(buf);
                ident.write_into(buf);
                spaces[1].write_into(buf);
                buf.push(',');
            }
            buf.pop();
        }
        self.spaces[1].write_into(buf);
        buf.push(')');
    }
}

#[derive(Debug, Arbitrary)]
struct Struct {
    spaces: [Space; 2],
    type_name: TypeName,
    fields: Vec<([Space; 2], Ident, Option<([Space; 2], Target)>)>,
}

impl ToSource for Struct {
    fn write_into(&self, buf: &mut String) {
        self.type_name.write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push('{');
        if !self.fields.is_empty() {
            for (spaces, ident, target) in &self.fields {
                spaces[0].write_into(buf);
                ident.write_into(buf);
                if let Some((spaces, target)) = target {
                    spaces[0].write_into(buf);
                    buf.push(':');
                    spaces[1].write_into(buf);
                    target.write_into(buf);
                }
                spaces[1].write_into(buf);
                buf.push(',');
            }
            buf.pop();
        }
        self.spaces[1].write_into(buf);
        buf.push('}');
    }
}

#[derive(Debug, Arbitrary)]
enum Whitespace {
    Default,
    Preserve,
    Suppress,
    Minimize,
}

impl ToSource for Whitespace {
    fn write_into(&self, buf: &mut String) {
        match self {
            Self::Default => (),
            Self::Preserve => buf.push('+'),
            Self::Suppress => buf.push('-'),
            Self::Minimize => buf.push('~'),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct FilePath(Printable);

impl ToSource for FilePath {
    fn write_into(&self, buf: &mut String) {
        write!(buf, "{:?}", &self.0 .0).unwrap();
    }
}
