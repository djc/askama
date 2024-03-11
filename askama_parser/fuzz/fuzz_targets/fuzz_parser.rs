#![no_main]
use askama_parser::*;
use libfuzzer_sys::fuzz_target;
use std::str;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    if let Ok(data) = str::from_utf8(data) {
        if let Ok(_) = Ast::from_str(data, None, &Syntax::default()) {}
    }
});
