use std::fmt::{Display, Formatter, Write};

use arbitrary::{Arbitrary, Unstructured};

use super::strings::{Ident, Printable, PrintableNoGrouping, Space, TypeName};

#[derive(Debug, Arbitrary)]
pub struct Expr(TopLevel);

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Arbitrary)]
enum TopLevel {
    ExprInner(ExprInner),
    Cmp(Cmp),
    Filter(Box<Filter>),
}

impl Display for TopLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExprInner(v) => v.fmt(f),
            Self::Cmp(v) => v.fmt(f),
            Self::Filter(v) => v.fmt(f),
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

impl Display for Cmp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.left.fmt(f)?;
        self.spaces[0].fmt(f)?;
        self.op.fmt(f)?;
        self.spaces[1].fmt(f)?;
        self.right.fmt(f)?;
        Ok(())
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

impl Display for CmpOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Ge => ">=",
            Self::Gt => ">",
            Self::Le => "<=",
            Self::Lt => "<",
        })
    }
}

#[derive(Debug, Arbitrary)]
struct Filter {
    space: Space,
    expr: ExprInner,
    filter: Ident,
    args: Option<([Space; 3], Vec<([Space; 2], ExprInner)>)>,
}

impl Display for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.expr.fmt(f)?;
        f.write_char('|')?;
        self.space.fmt(f)?;
        self.filter.fmt(f)?;
        if let Some((spaces, args)) = &self.args {
            spaces[0].fmt(f)?;
            f.write_char('(')?;
            spaces[1].fmt(f)?;
            for (idx, (spaces, expr)) in args.iter().enumerate() {
                if idx > 0 {
                    f.write_char(',')?;
                    spaces[0].fmt(f)?;
                }
                expr.fmt(f)?;
                spaces[1].fmt(f)?;
            }
            spaces[2].fmt(f)?;
            f.write_char(')')?;
        }
        Ok(())
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

impl Display for ExprInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unary(v) => v.fmt(f),
            Self::Binary(v) => v.fmt(f),
            // TODO: Self::Range(v) => v.fmt(f),
            Self::Suffix(v) => v.fmt(f),
            Self::Literal(v) => v.fmt(f),
            Self::Collection(v) => v.fmt(f),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Unary {
    space: Space,
    op: UnaryOp,
    expr: ExprInner,
}

impl Display for Unary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.op.fmt(f)?;
        self.space.fmt(f)?;
        self.expr.fmt(f)?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
enum UnaryOp {
    Not,
    Minus,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Self::Not => '-',
            Self::Minus => '!',
        })
    }
}

#[derive(Debug, Arbitrary)]
struct Binary {
    op: BinOp,
    spaces: [Space; 2],
    left: ExprInner,
    right: ExprInner,
}

impl Display for Binary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.left.fmt(f)?;
        self.spaces[0].fmt(f)?;
        self.op.fmt(f)?;
        self.spaces[1].fmt(f)?;
        self.right.fmt(f)?;
        Ok(())
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

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
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
        })
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
            expr.fmt(f)?;
            space.fmt(f)?;
        }
        self.op.fmt(f)?;
        if let Some((space, expr)) = &self.right {
            space.fmt(f)?;
            expr.fmt(f)?;
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
        f.write_str(match self {
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

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(v) => write!(f, "{v:?}"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => v.fmt(f),
            Self::String(v) => write!(f, "{:?}", &v.0),
            Self::Char(v) => write!(f, "{v:?}"),
            Self::Var(v) => v.fmt(f),
            Self::Path(v) => v.fmt(f),
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

impl Display for Suffix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Attr(v) => v.fmt(f),
            Self::Index(v) => v.fmt(f),
            Self::Call(v) => v.fmt(f),
            Self::Try(v) => v.fmt(f),
            Self::Macro(v) => v.fmt(f),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Attr {
    spaces: [Space; 2],
    obj: ExprInner,
    field: Ident,
}

impl Display for Attr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.obj.fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_char('.')?;
        self.spaces[1].fmt(f)?;
        self.field.fmt(f)?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Index {
    spaces: [Space; 3],
    obj: ExprInner,
    index: ExprInner,
}

impl Display for Index {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.obj.fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_char('[')?;
        self.spaces[1].fmt(f)?;
        self.index.fmt(f)?;
        self.spaces[2].fmt(f)?;
        f.write_char(']')?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Call {
    spaces: [Space; 3],
    callee: ExprInner,
    args: Vec<([Space; 2], ExprInner)>,
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.callee.fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_char('(')?;
        self.spaces[1].fmt(f)?;
        for (idx, (spaces, expr)) in self.args.iter().enumerate() {
            if idx > 0 {
                f.write_char(',')?;
                spaces[0].fmt(f)?;
            }
            expr.fmt(f)?;
            spaces[1].fmt(f)?;
        }
        self.spaces[2].fmt(f)?;
        f.write_char(')')?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Try {
    space: Space,
    expr: ExprInner,
}

impl Display for Try {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.expr.fmt(f)?;
        self.space.fmt(f)?;
        f.write_char('?')?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Macro {
    spaces: [Space; 4],
    callee: Ident,
    content: PrintableNoGrouping,
}

impl Display for Macro {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.callee.fmt(f)?;
        self.spaces[0].fmt(f)?;
        f.write_char('!')?;
        self.spaces[1].fmt(f)?;
        f.write_char('(')?;
        self.spaces[2].fmt(f)?;
        self.content.fmt(f)?;
        self.spaces[3].fmt(f)?;
        f.write_char(')')?;
        Ok(())
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

impl Display for Float {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Arbitrary)]
enum Collection {
    Array(Array),
    Group(Group),
    Tuple(Tuple),
}

impl Display for Collection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(v) => v.fmt(f),
            Self::Group(v) => v.fmt(f),
            Self::Tuple(v) => v.fmt(f),
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Array {
    spaces: [Space; 2],
    items: Vec<([Space; 2], ExprInner)>,
}

impl Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        self.spaces[0].fmt(f)?;
        for (idx, (spaces, expr)) in self.items.iter().enumerate() {
            if idx > 0 {
                f.write_char(',')?;
                spaces[0].fmt(f)?;
            }
            expr.fmt(f)?;
            spaces[1].fmt(f)?;
        }
        self.spaces[1].fmt(f)?;
        f.write_char(']')?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Group {
    spaces: [Space; 2],
    item: ExprInner,
}

impl Display for Group {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('(')?;
        self.spaces[0].fmt(f)?;
        self.item.fmt(f)?;
        self.spaces[1].fmt(f)?;
        f.write_char(')')?;
        Ok(())
    }
}

#[derive(Debug, Arbitrary)]
struct Tuple {
    spaces: [Space; 2],
    items: Vec<([Space; 2], ExprInner)>,
}

impl Display for Tuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('(')?;
        self.spaces[0].fmt(f)?;
        for (idx, (spaces, expr)) in self.items.iter().enumerate() {
            if idx > 0 {
                f.write_char(',')?;
                spaces[0].fmt(f)?;
            }
            expr.fmt(f)?;
            spaces[1].fmt(f)?;
        }
        self.spaces[1].fmt(f)?;
        f.write_char(')')?;
        Ok(())
    }
}
