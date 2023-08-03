#![deny(unreachable_pub)]
#![deny(elided_lifetimes_in_paths)]

use askama_parser::{Ast, Syntax};
use honggfuzz::fuzz;
use random_code::Node;

fn main() {
    loop {
        fuzz!(|node: Node| {
            dbg!(&node);
            let source = node.to_string();
            Ast::from_str(&source, &Syntax::default()).unwrap();
        });
    }
}
