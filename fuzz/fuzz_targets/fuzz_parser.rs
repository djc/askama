#![no_main]
use askama_parser::*;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    if let Ok(data) = std::str::from_utf8(data) {
        let _ = Ast::from_str(data, None, &Syntax::default()).is_ok();
    }
});
