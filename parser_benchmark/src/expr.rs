use std::fmt::Write;

use arbitrary::{Arbitrary, Unstructured};

use super::strings::{Ident, Printable, PrintableNoGrouping, Space, TypeName};
use super::ToSource;

#[derive(Debug, Arbitrary)]
pub(super) struct Expr(TopLevel);

impl ToSource for Expr {
    fn write_into(&self, buf: &mut String) {
        self.0.write_into(buf);
    }
}

#[derive(Debug, Arbitrary)]
enum TopLevel {
    ExprInner(ExprInner),
    Cmp(Cmp),
    Filter(Box<Filter>),
}

impl ToSource for TopLevel {
    fn write_into(&self, buf: &mut String) {
        match self {
            Self::ExprInner(v) => v.write_into(buf),
            Self::Cmp(v) => v.write_into(buf),
            Self::Filter(v) => v.write_into(buf),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Cmp {
    op: CmpOp,
    spaces: [Space; 2],
    left: ExprInner,
    right: ExprInner,
}

impl ToSource for Cmp {
    fn write_into(&self, buf: &mut String) {
        self.left.write_into(buf);
        self.spaces[0].write_into(buf);
        self.op.write_into(buf);
        self.spaces[1].write_into(buf);
        self.right.write_into(buf);
    }
}

#[derive(Debug, Arbitrary)]
enum CmpOp {
    Eq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,
}

impl ToSource for CmpOp {
    fn write_into(&self, buf: &mut String) {
        buf.push_str(match self {
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Ge => ">=",
            Self::Gt => ">",
            Self::Le => "<=",
            Self::Lt => "<",
        });
    }
}

#[derive(Debug, Arbitrary)]
struct Filter {
    space: Space,
    expr: ExprInner,
    filter: Ident,
    args: Option<([Space; 3], Vec<([Space; 2], ExprInner)>)>,
}

impl ToSource for Filter {
    fn write_into(&self, buf: &mut String) {
        self.expr.write_into(buf);
        buf.push('|');
        self.space.write_into(buf);
        self.filter.write_into(buf);
        if let Some((spaces, args)) = &self.args {
            spaces[0].write_into(buf);
            buf.push('(');
            spaces[1].write_into(buf);
            if !args.is_empty() {
                for (spaces, expr) in args {
                    spaces[0].write_into(buf);
                    expr.write_into(buf);
                    spaces[1].write_into(buf);
                    buf.push(',');
                }
                buf.pop();
            }
            spaces[2].write_into(buf);
            buf.push(')');
        }
    }
}

#[derive(Debug, Arbitrary)]
enum ExprInner {
    Unary(Box<Unary>),
    Binary(Box<Binary>),
    // TODO: Range(Box<Range>),
    Literal(Box<Literal>),
    Collection(Box<Collection>),
    Suffix(Suffix),
}

impl ToSource for ExprInner {
    fn write_into(&self, buf: &mut String) {
        match self {
            Self::Unary(v) => v.write_into(buf),
            Self::Binary(v) => v.write_into(buf),
            // TODO: Self::Range(v) => v.write_into(buf),
            Self::Suffix(v) => v.write_into(buf),
            Self::Literal(v) => v.write_into(buf),
            Self::Collection(v) => v.write_into(buf),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Unary {
    space: Space,
    op: UnaryOp,
    expr: ExprInner,
}

impl ToSource for Unary {
    fn write_into(&self, buf: &mut String) {
        self.op.write_into(buf);
        self.space.write_into(buf);
        self.expr.write_into(buf);
    }
}

#[derive(Debug, Arbitrary)]
enum UnaryOp {
    Not,
    Minus,
}

impl ToSource for UnaryOp {
    fn write_into(&self, buf: &mut String) {
        buf.push(match self {
            Self::Not => '-',
            Self::Minus => '!',
        });
    }
}

#[derive(Debug, Arbitrary)]
struct Binary {
    op: BinOp,
    spaces: [Space; 2],
    left: ExprInner,
    right: ExprInner,
}

impl ToSource for Binary {
    fn write_into(&self, buf: &mut String) {
        self.left.write_into(buf);
        self.spaces[0].write_into(buf);
        self.op.write_into(buf);
        self.spaces[1].write_into(buf);
        self.right.write_into(buf);
    }
}

#[derive(Debug, Arbitrary)]
enum BinOp {
    And,
    Or,
    Bor,
    Bxor,
    Band,
    Shl,
    Shr,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl ToSource for BinOp {
    fn write_into(&self, buf: &mut String) {
        buf.push_str(match self {
            Self::And => "&&",
            Self::Or => "||",
            Self::Bor => " |",
            Self::Bxor => "^",
            Self::Band => "&",
            Self::Shl => "<<",
            Self::Shr => ">>",
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Mod => "%",
        });
    }
}

// TODO: FIXME: generates invalid code
/*
#[derive(Debug, Arbitrary)]
struct Range {
    op: RangeOp,
    left: Option<(Space, ExprInner)>,
    right: Option<(Space, ExprInner)>,
}

impl ToSource for Range {
    fn write_into(&self, buf: &mut String) {
        if let Some((space, expr)) = &self.left {
            expr.write_into(buf);
            space.write_into(buf);
        }
        self.op.write_into(buf);
        if let Some((space, expr)) = &self.right {
            space.write_into(buf);
            expr.write_into(buf);
        }
    }
}

#[derive(Debug, Arbitrary)]
enum RangeOp {
    Open,
    Closed,
}

impl ToSource for RangeOp {
    fn write_into(&self, buf: &mut String) {
        buf.push_str(match self {
            Self::Open => "..",
            Self::Closed => "..=",
        });
    }
}
*/

#[derive(Debug, Arbitrary)]
enum Literal {
    Bool(bool),
    Int(isize),
    Float(Float),
    String(Printable),
    Char(char),
    Var(Ident),
    Path(TypeName),
}

impl ToSource for Literal {
    fn write_into(&self, buf: &mut String) {
        match self {
            Self::Bool(v) => write!(buf, "{v:?}").unwrap(),
            Self::Int(v) => write!(buf, "{v}").unwrap(),
            Self::Float(v) => v.write_into(buf),
            Self::String(v) => write!(buf, "{:?}", &v.0).unwrap(),
            Self::Char(v) => write!(buf, "{v:?}").unwrap(),
            Self::Var(v) => v.write_into(buf),
            Self::Path(v) => v.write_into(buf),
        }
    }
}

#[derive(Debug, Arbitrary)]
enum Suffix {
    Attr(Box<Attr>),
    Index(Box<Index>),
    Call(Box<Call>),
    Try(Box<Try>),
    Macro(Box<Macro>),
}

impl ToSource for Suffix {
    fn write_into(&self, buf: &mut String) {
        match self {
            Self::Attr(v) => v.write_into(buf),
            Self::Index(v) => v.write_into(buf),
            Self::Call(v) => v.write_into(buf),
            Self::Try(v) => v.write_into(buf),
            Self::Macro(v) => v.write_into(buf),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Attr {
    spaces: [Space; 2],
    obj: ExprInner,
    field: Ident,
}

impl ToSource for Attr {
    fn write_into(&self, buf: &mut String) {
        self.obj.write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push('.');
        self.spaces[1].write_into(buf);
        self.field.write_into(buf);
    }
}

#[derive(Debug, Arbitrary)]
struct Index {
    spaces: [Space; 3],
    obj: ExprInner,
    index: ExprInner,
}

impl ToSource for Index {
    fn write_into(&self, buf: &mut String) {
        self.obj.write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push('[');
        self.spaces[1].write_into(buf);
        self.index.write_into(buf);
        self.spaces[2].write_into(buf);
        buf.push(']');
    }
}

#[derive(Debug, Arbitrary)]
struct Call {
    spaces: [Space; 3],
    callee: ExprInner,
    args: Vec<([Space; 2], ExprInner)>,
}

impl ToSource for Call {
    fn write_into(&self, buf: &mut String) {
        self.callee.write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push('(');
        self.spaces[1].write_into(buf);
        if !self.args.is_empty() {
            for (spaces, expr) in &self.args {
                spaces[0].write_into(buf);
                expr.write_into(buf);
                spaces[1].write_into(buf);
                buf.push(',');
            }
            buf.pop();
        }
        self.spaces[2].write_into(buf);
        buf.push(')');
    }
}

#[derive(Debug, Arbitrary)]
struct Try {
    space: Space,
    expr: ExprInner,
}

impl ToSource for Try {
    fn write_into(&self, buf: &mut String) {
        self.expr.write_into(buf);
        self.space.write_into(buf);
        buf.push('?');
    }
}

#[derive(Debug, Arbitrary)]
struct Macro {
    spaces: [Space; 4],
    callee: Ident,
    content: PrintableNoGrouping,
}

impl ToSource for Macro {
    fn write_into(&self, buf: &mut String) {
        self.callee.write_into(buf);
        self.spaces[0].write_into(buf);
        buf.push('!');
        self.spaces[1].write_into(buf);
        buf.push('(');
        self.spaces[2].write_into(buf);
        self.content.write_into(buf);
        self.spaces[3].write_into(buf);
        buf.push(')');
    }
}

#[derive(Debug)]
struct Float(f32);

impl<'a> Arbitrary<'a> for Float {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let num = u.int_in_range(-9999999..=9999999)?;
        let denom = u.int_in_range(1..=9999999)?;
        Ok(Self(num as f32 / denom as f32))
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <[i32; 2]>::size_hint(depth)
    }
}

impl ToSource for Float {
    fn write_into(&self, buf: &mut String) {
        write!(buf, "{}", self.0).unwrap();
    }
}

#[derive(Debug, Arbitrary)]
enum Collection {
    Array(Array),
    Group(Group),
    Tuple(Tuple),
}

impl ToSource for Collection {
    fn write_into(&self, buf: &mut String) {
        match self {
            Self::Array(v) => v.write_into(buf),
            Self::Group(v) => v.write_into(buf),
            Self::Tuple(v) => v.write_into(buf),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Array {
    spaces: [Space; 2],
    items: Vec<([Space; 2], ExprInner)>,
}

impl ToSource for Array {
    fn write_into(&self, buf: &mut String) {
        buf.push('[');
        self.spaces[0].write_into(buf);
        if !self.items.is_empty() {
            for (spaces, expr) in &self.items {
                spaces[0].write_into(buf);
                expr.write_into(buf);
                spaces[1].write_into(buf);
                buf.push(',');
            }
            buf.pop();
        }
        self.spaces[1].write_into(buf);
        buf.push(']');
    }
}

#[derive(Debug, Arbitrary)]
struct Group {
    spaces: [Space; 2],
    item: ExprInner,
}

impl ToSource for Group {
    fn write_into(&self, buf: &mut String) {
        buf.push('(');
        self.spaces[0].write_into(buf);
        self.item.write_into(buf);
        self.spaces[1].write_into(buf);
        buf.push(')');
    }
}

#[derive(Debug, Arbitrary)]
struct Tuple {
    spaces: [Space; 2],
    items: Vec<([Space; 2], ExprInner)>,
}

impl ToSource for Tuple {
    fn write_into(&self, buf: &mut String) {
        buf.push('(');
        self.spaces[0].write_into(buf);
        if !self.items.is_empty() {
            for (spaces, expr) in &self.items {
                spaces[0].write_into(buf);
                expr.write_into(buf);
                spaces[1].write_into(buf);
                buf.push(',');
            }
            buf.pop();
        }
        self.spaces[1].write_into(buf);
        buf.push(')');
    }
}
