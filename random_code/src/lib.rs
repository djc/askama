#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]
#![allow(clippy::type_complexity)]

//! This library generates arbitrary template code that might be deeply nested, and contains
//! (almost) all structures that may occur in askama templates.

mod expr;
mod node;
mod strings;

pub use expr::Expr;
pub use node::Node;
