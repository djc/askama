#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]
#![allow(clippy::type_complexity)]

mod expr;
mod node;
mod strings;

pub use expr::Expr;
pub use node::Node;
