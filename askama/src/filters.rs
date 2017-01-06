extern crate htmlescape;

use std::fmt;

pub fn e(s: &fmt::Display) -> String {
    let s = format!("{}", s);
    htmlescape::encode_minimal(&s)
}
