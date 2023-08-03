#![deny(unreachable_pub)]
#![deny(elided_lifetimes_in_paths)]
#![no_main]

use askama_parser::{Ast, Syntax};
use libfuzzer_sys::fuzz_target;
use random_code::Node;

fuzz_target!(|node: Node| {
    dbg!(&node);
    let source = node.to_string();
    Ast::from_str(&source, &Syntax::default()).unwrap();
});
