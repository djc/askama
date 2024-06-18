#![no_main]
use arbitrary::{Arbitrary, Unstructured};
use askama::filters::*;
use askama_escape::Html;
use libfuzzer_sys::fuzz_target;
use std::str::{self};

macro_rules! fuzz {
    ($name: ident, $func: ident) => {
        fn $name(data: &[u8]) {
            if let Ok(data) = str::from_utf8(data) {
                if let Ok(d) = $func(data) {
                    let _ = d.to_string();
                }
            }
        }
    };
    ($name: ident, $func: ident, $arg_type: ty) => {
        fn $name(data: &[u8]) {
            if let Some(adata) = get_arbitrary_data::<$arg_type>(data) {
                if let Ok(sdata) = str::from_utf8(data) {
                    if let Ok(d) = $func(sdata, adata) {
                        let _ = d.to_string();
                    }
                }
            }
        }
    };
}

fn get_arbitrary_data<'a, T>(data: &'a [u8]) -> Option<T>
where
    T: Arbitrary<'a>,
{
    T::arbitrary(&mut Unstructured::new(data)).ok()
}

fuzz!(fuzz_urlencode, urlencode);
fuzz!(fuzz_urlencode_strict, urlencode_strict);
fuzz!(fuzz_linebreaks, linebreaks);
fuzz!(fuzz_paragraphbreaks, paragraphbreaks);
fuzz!(fuzz_trim, trim);
fuzz!(fuzz_title, title);
fuzz!(fuzz_capitalize, capitalize);
fuzz!(fuzz_truncate, truncate, usize);
fuzz!(fuzz_indent, indent, usize);
fuzz!(fuzz_center, center, usize);

fn fuzz_escape(data: &[u8]) {
    if let Ok(data) = str::from_utf8(data) {
        let _ = askama_escape::escape(data, Html);
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let idx = data[0] % 11;
    let data = &data[1..];
    match idx {
        0 => fuzz_urlencode(data),
        1 => fuzz_urlencode_strict(data),
        2 => fuzz_linebreaks(data),
        3 => fuzz_paragraphbreaks(data),
        4 => fuzz_trim(data),
        5 => fuzz_title(data),
        6 => fuzz_capitalize(data),
        7 => fuzz_truncate(data),
        8 => fuzz_indent(data),
        9 => fuzz_escape(data),
        _ => fuzz_center(data),
    }
});
