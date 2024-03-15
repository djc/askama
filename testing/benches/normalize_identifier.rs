use std::collections::HashSet;
use std::iter::FusedIterator;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

criterion_main!(benches);
criterion_group!(benches, functions);

fn functions(c: &mut Criterion) {
    let words = Words::default().collect::<Vec<_>>();

    macro_rules! bench_function {
        ($($func:ident)*) => {
            for word in words.iter().collect::<HashSet<_>>() {
                let linear: &str = normalize_identifier_linear(word);
                $(
                    assert_eq!(linear, $func(word));
                )*
            }

            $(
                c.bench_function(stringify!($func), |b| {
                    b.iter(|| {
                        for s in &words {
                            black_box($func(black_box(s)));
                        }
                    });
                });
            )*
        };
    }

    bench_function! {
        normalize_identifier_linear
        normalize_identifier_linear_replacement_only
        normalize_identifier_bisect
        normalize_identifier_bisect_replacement_only
        normalize_identifier_linear_by_len
        normalize_identifier_linear_by_len_replacement_only
        normalize_identifier_bisect_by_len
        normalize_identifier_bisect_by_len_replacement_only
        normalize_identifier_phf
    }
}

fn normalize_identifier_linear(ident: &str) -> &str {
    static USE_RAW: [(&str, &str); 47] = [
        ("abstract", "r#abstract"),
        ("as", "r#as"),
        ("async", "r#async"),
        ("await", "r#await"),
        ("become", "r#become"),
        ("box", "r#box"),
        ("break", "r#break"),
        ("const", "r#const"),
        ("continue", "r#continue"),
        ("crate", "r#crate"),
        ("do", "r#do"),
        ("dyn", "r#dyn"),
        ("else", "r#else"),
        ("enum", "r#enum"),
        ("extern", "r#extern"),
        ("false", "r#false"),
        ("final", "r#final"),
        ("fn", "r#fn"),
        ("for", "r#for"),
        ("if", "r#if"),
        ("impl", "r#impl"),
        ("in", "r#in"),
        ("let", "r#let"),
        ("macro", "r#macro"),
        ("match", "r#match"),
        ("mod", "r#mod"),
        ("move", "r#move"),
        ("mut", "r#mut"),
        ("override", "r#override"),
        ("priv", "r#priv"),
        ("pub", "r#pub"),
        ("ref", "r#ref"),
        ("return", "r#return"),
        ("static", "r#static"),
        ("struct", "r#struct"),
        ("trait", "r#trait"),
        ("true", "r#true"),
        ("try", "r#try"),
        ("type", "r#type"),
        ("typeof", "r#typeof"),
        ("unsafe", "r#unsafe"),
        ("unsized", "r#unsized"),
        ("use", "r#use"),
        ("virtual", "r#virtual"),
        ("where", "r#where"),
        ("while", "r#while"),
        ("yield", "r#yield"),
    ];

    if let Some(word) = USE_RAW.iter().find(|x| x.0 == ident) {
        word.1
    } else {
        ident
    }
}

fn normalize_identifier_bisect(ident: &str) -> &str {
    static USE_RAW: [(&str, &str); 47] = [
        ("abstract", "r#abstract"),
        ("as", "r#as"),
        ("async", "r#async"),
        ("await", "r#await"),
        ("become", "r#become"),
        ("box", "r#box"),
        ("break", "r#break"),
        ("const", "r#const"),
        ("continue", "r#continue"),
        ("crate", "r#crate"),
        ("do", "r#do"),
        ("dyn", "r#dyn"),
        ("else", "r#else"),
        ("enum", "r#enum"),
        ("extern", "r#extern"),
        ("false", "r#false"),
        ("final", "r#final"),
        ("fn", "r#fn"),
        ("for", "r#for"),
        ("if", "r#if"),
        ("impl", "r#impl"),
        ("in", "r#in"),
        ("let", "r#let"),
        ("macro", "r#macro"),
        ("match", "r#match"),
        ("mod", "r#mod"),
        ("move", "r#move"),
        ("mut", "r#mut"),
        ("override", "r#override"),
        ("priv", "r#priv"),
        ("pub", "r#pub"),
        ("ref", "r#ref"),
        ("return", "r#return"),
        ("static", "r#static"),
        ("struct", "r#struct"),
        ("trait", "r#trait"),
        ("true", "r#true"),
        ("try", "r#try"),
        ("type", "r#type"),
        ("typeof", "r#typeof"),
        ("unsafe", "r#unsafe"),
        ("unsized", "r#unsized"),
        ("use", "r#use"),
        ("virtual", "r#virtual"),
        ("where", "r#where"),
        ("while", "r#while"),
        ("yield", "r#yield"),
    ];

    if let Ok(idx) = USE_RAW.binary_search_by(|(probe, _)| (*probe).cmp(ident)) {
        USE_RAW[idx].1
    } else {
        ident
    }
}

fn normalize_identifier_linear_replacement_only(ident: &str) -> &str {
    static USE_RAW: [&str; 47] = [
        "r#abstract",
        "r#as",
        "r#async",
        "r#await",
        "r#become",
        "r#box",
        "r#break",
        "r#const",
        "r#continue",
        "r#crate",
        "r#do",
        "r#dyn",
        "r#else",
        "r#enum",
        "r#extern",
        "r#false",
        "r#final",
        "r#fn",
        "r#for",
        "r#if",
        "r#impl",
        "r#in",
        "r#let",
        "r#macro",
        "r#match",
        "r#mod",
        "r#move",
        "r#mut",
        "r#override",
        "r#priv",
        "r#pub",
        "r#ref",
        "r#return",
        "r#static",
        "r#struct",
        "r#trait",
        "r#true",
        "r#try",
        "r#type",
        "r#typeof",
        "r#unsafe",
        "r#unsized",
        "r#use",
        "r#virtual",
        "r#where",
        "r#while",
        "r#yield",
    ];

    USE_RAW
        .iter()
        .find(|x| &x[2..] == ident)
        .copied()
        .unwrap_or(ident)
}

fn normalize_identifier_bisect_replacement_only(ident: &str) -> &str {
    static USE_RAW: [&str; 47] = [
        "r#abstract",
        "r#as",
        "r#async",
        "r#await",
        "r#become",
        "r#box",
        "r#break",
        "r#const",
        "r#continue",
        "r#crate",
        "r#do",
        "r#dyn",
        "r#else",
        "r#enum",
        "r#extern",
        "r#false",
        "r#final",
        "r#fn",
        "r#for",
        "r#if",
        "r#impl",
        "r#in",
        "r#let",
        "r#macro",
        "r#match",
        "r#mod",
        "r#move",
        "r#mut",
        "r#override",
        "r#priv",
        "r#pub",
        "r#ref",
        "r#return",
        "r#static",
        "r#struct",
        "r#trait",
        "r#true",
        "r#try",
        "r#type",
        "r#typeof",
        "r#unsafe",
        "r#unsized",
        "r#use",
        "r#virtual",
        "r#where",
        "r#while",
        "r#yield",
    ];

    if let Ok(idx) = USE_RAW.binary_search_by(|probe| probe[2..].cmp(ident)) {
        USE_RAW[idx]
    } else {
        ident
    }
}

fn normalize_identifier_linear_by_len(ident: &str) -> &str {
    const KW0: &[([u8; 8], [u8; 10])] = &[];
    const KW1: &[([u8; 8], [u8; 10])] = &[];
    const KW2: &[([u8; 8], [u8; 10])] = &[
        (*b"as______", *b"r#as______"),
        (*b"do______", *b"r#do______"),
        (*b"fn______", *b"r#fn______"),
        (*b"if______", *b"r#if______"),
        (*b"in______", *b"r#in______"),
    ];
    const KW3: &[([u8; 8], [u8; 10])] = &[
        (*b"box_____", *b"r#box_____"),
        (*b"dyn_____", *b"r#dyn_____"),
        (*b"for_____", *b"r#for_____"),
        (*b"let_____", *b"r#let_____"),
        (*b"mod_____", *b"r#mod_____"),
        (*b"mut_____", *b"r#mut_____"),
        (*b"pub_____", *b"r#pub_____"),
        (*b"ref_____", *b"r#ref_____"),
        (*b"try_____", *b"r#try_____"),
        (*b"use_____", *b"r#use_____"),
    ];
    const KW4: &[([u8; 8], [u8; 10])] = &[
        (*b"else____", *b"r#else____"),
        (*b"enum____", *b"r#enum____"),
        (*b"impl____", *b"r#impl____"),
        (*b"move____", *b"r#move____"),
        (*b"priv____", *b"r#priv____"),
        (*b"true____", *b"r#true____"),
        (*b"type____", *b"r#type____"),
    ];
    const KW5: &[([u8; 8], [u8; 10])] = &[
        (*b"async___", *b"r#async___"),
        (*b"await___", *b"r#await___"),
        (*b"break___", *b"r#break___"),
        (*b"const___", *b"r#const___"),
        (*b"crate___", *b"r#crate___"),
        (*b"false___", *b"r#false___"),
        (*b"final___", *b"r#final___"),
        (*b"macro___", *b"r#macro___"),
        (*b"match___", *b"r#match___"),
        (*b"trait___", *b"r#trait___"),
        (*b"where___", *b"r#where___"),
        (*b"while___", *b"r#while___"),
        (*b"yield___", *b"r#yield___"),
    ];
    const KW6: &[([u8; 8], [u8; 10])] = &[
        (*b"become__", *b"r#become__"),
        (*b"extern__", *b"r#extern__"),
        (*b"return__", *b"r#return__"),
        (*b"static__", *b"r#static__"),
        (*b"struct__", *b"r#struct__"),
        (*b"typeof__", *b"r#typeof__"),
        (*b"unsafe__", *b"r#unsafe__"),
    ];
    const KW7: &[([u8; 8], [u8; 10])] = &[
        (*b"unsized_", *b"r#unsized_"),
        (*b"virtual_", *b"r#virtual_"),
    ];
    const KW8: &[([u8; 8], [u8; 10])] = &[
        (*b"abstract", *b"r#abstract"),
        (*b"continue", *b"r#continue"),
        (*b"override", *b"r#override"),
    ];
    const KWS: &[&[([u8; 8], [u8; 10])]] = &[KW0, KW1, KW2, KW3, KW4, KW5, KW6, KW7, KW8];

    if ident.len() > 8 {
        return ident;
    }
    let kws = KWS[ident.len()];

    let mut padded_ident = [b'_'; 8];
    padded_ident[..ident.len()].copy_from_slice(ident.as_bytes());

    let replacement = match kws.iter().find(|(kw, _)| *kw == padded_ident) {
        Some((_, replacement)) => replacement,
        None => return ident,
    };

    // SAFETY: We know that the input byte slice is pure-ASCII.
    unsafe { std::str::from_utf8_unchecked(&replacement[..ident.len() + 2]) }
}

fn normalize_identifier_linear_by_len_replacement_only(ident: &str) -> &str {
    const KW0: &[[u8; 10]] = &[];
    const KW1: &[[u8; 10]] = &[];
    const KW2: &[[u8; 10]] = &[
        *b"r#as______",
        *b"r#do______",
        *b"r#fn______",
        *b"r#if______",
        *b"r#in______",
    ];
    const KW3: &[[u8; 10]] = &[
        *b"r#box_____",
        *b"r#dyn_____",
        *b"r#for_____",
        *b"r#let_____",
        *b"r#mod_____",
        *b"r#mut_____",
        *b"r#pub_____",
        *b"r#ref_____",
        *b"r#try_____",
        *b"r#use_____",
    ];
    const KW4: &[[u8; 10]] = &[
        *b"r#else____",
        *b"r#enum____",
        *b"r#impl____",
        *b"r#move____",
        *b"r#priv____",
        *b"r#true____",
        *b"r#type____",
    ];
    const KW5: &[[u8; 10]] = &[
        *b"r#async___",
        *b"r#await___",
        *b"r#break___",
        *b"r#const___",
        *b"r#crate___",
        *b"r#false___",
        *b"r#final___",
        *b"r#macro___",
        *b"r#match___",
        *b"r#trait___",
        *b"r#where___",
        *b"r#while___",
        *b"r#yield___",
    ];
    const KW6: &[[u8; 10]] = &[
        *b"r#become__",
        *b"r#extern__",
        *b"r#return__",
        *b"r#static__",
        *b"r#struct__",
        *b"r#typeof__",
        *b"r#unsafe__",
    ];
    const KW7: &[[u8; 10]] = &[*b"r#unsized_", *b"r#virtual_"];
    const KW8: &[[u8; 10]] = &[*b"r#abstract", *b"r#continue", *b"r#override"];
    const KWS: &[&[[u8; 10]]] = &[KW0, KW1, KW2, KW3, KW4, KW5, KW6, KW7, KW8];

    if ident.len() > 8 {
        return ident;
    }
    let kws = KWS[ident.len()];

    let mut padded_ident = [b'_'; 8];
    padded_ident[..ident.len()].copy_from_slice(ident.as_bytes());

    let replacement = match kws
        .iter()
        .find(|probe| padded_ident == <[u8; 8]>::try_from(&probe[2..]).unwrap())
    {
        Some(replacement) => replacement,
        None => return ident,
    };

    // SAFETY: We know that the input byte slice is pure-ASCII.
    unsafe { std::str::from_utf8_unchecked(&replacement[..ident.len() + 2]) }
}

fn normalize_identifier_bisect_by_len(ident: &str) -> &str {
    const KW0: &[([u8; 8], [u8; 10])] = &[];
    const KW1: &[([u8; 8], [u8; 10])] = &[];
    const KW2: &[([u8; 8], [u8; 10])] = &[
        (*b"as______", *b"r#as______"),
        (*b"do______", *b"r#do______"),
        (*b"fn______", *b"r#fn______"),
        (*b"if______", *b"r#if______"),
        (*b"in______", *b"r#in______"),
    ];
    const KW3: &[([u8; 8], [u8; 10])] = &[
        (*b"box_____", *b"r#box_____"),
        (*b"dyn_____", *b"r#dyn_____"),
        (*b"for_____", *b"r#for_____"),
        (*b"let_____", *b"r#let_____"),
        (*b"mod_____", *b"r#mod_____"),
        (*b"mut_____", *b"r#mut_____"),
        (*b"pub_____", *b"r#pub_____"),
        (*b"ref_____", *b"r#ref_____"),
        (*b"try_____", *b"r#try_____"),
        (*b"use_____", *b"r#use_____"),
    ];
    const KW4: &[([u8; 8], [u8; 10])] = &[
        (*b"else____", *b"r#else____"),
        (*b"enum____", *b"r#enum____"),
        (*b"impl____", *b"r#impl____"),
        (*b"move____", *b"r#move____"),
        (*b"priv____", *b"r#priv____"),
        (*b"true____", *b"r#true____"),
        (*b"type____", *b"r#type____"),
    ];
    const KW5: &[([u8; 8], [u8; 10])] = &[
        (*b"async___", *b"r#async___"),
        (*b"await___", *b"r#await___"),
        (*b"break___", *b"r#break___"),
        (*b"const___", *b"r#const___"),
        (*b"crate___", *b"r#crate___"),
        (*b"false___", *b"r#false___"),
        (*b"final___", *b"r#final___"),
        (*b"macro___", *b"r#macro___"),
        (*b"match___", *b"r#match___"),
        (*b"trait___", *b"r#trait___"),
        (*b"where___", *b"r#where___"),
        (*b"while___", *b"r#while___"),
        (*b"yield___", *b"r#yield___"),
    ];
    const KW6: &[([u8; 8], [u8; 10])] = &[
        (*b"become__", *b"r#become__"),
        (*b"extern__", *b"r#extern__"),
        (*b"return__", *b"r#return__"),
        (*b"static__", *b"r#static__"),
        (*b"struct__", *b"r#struct__"),
        (*b"typeof__", *b"r#typeof__"),
        (*b"unsafe__", *b"r#unsafe__"),
    ];
    const KW7: &[([u8; 8], [u8; 10])] = &[
        (*b"unsized_", *b"r#unsized_"),
        (*b"virtual_", *b"r#virtual_"),
    ];
    const KW8: &[([u8; 8], [u8; 10])] = &[
        (*b"abstract", *b"r#abstract"),
        (*b"continue", *b"r#continue"),
        (*b"override", *b"r#override"),
    ];
    const KWS: &[&[([u8; 8], [u8; 10])]] = &[KW0, KW1, KW2, KW3, KW4, KW5, KW6, KW7, KW8];

    if ident.len() > 8 {
        return ident;
    }
    let kws = KWS[ident.len()];

    let mut padded_ident = [b'_'; 8];
    padded_ident[..ident.len()].copy_from_slice(ident.as_bytes());

    if let Ok(idx) = kws.binary_search_by(|(probe, _)| probe.cmp(&padded_ident)) {
        // SAFETY: We know that the input byte slice is pure-ASCII.
        unsafe { std::str::from_utf8_unchecked(&kws[idx].1[..ident.len() + 2]) }
    } else {
        ident
    }
}

fn normalize_identifier_bisect_by_len_replacement_only(ident: &str) -> &str {
    const KW0: &[[u8; 10]] = &[];
    const KW1: &[[u8; 10]] = &[];
    const KW2: &[[u8; 10]] = &[
        *b"r#as______",
        *b"r#do______",
        *b"r#fn______",
        *b"r#if______",
        *b"r#in______",
    ];
    const KW3: &[[u8; 10]] = &[
        *b"r#box_____",
        *b"r#dyn_____",
        *b"r#for_____",
        *b"r#let_____",
        *b"r#mod_____",
        *b"r#mut_____",
        *b"r#pub_____",
        *b"r#ref_____",
        *b"r#try_____",
        *b"r#use_____",
    ];
    const KW4: &[[u8; 10]] = &[
        *b"r#else____",
        *b"r#enum____",
        *b"r#impl____",
        *b"r#move____",
        *b"r#priv____",
        *b"r#true____",
        *b"r#type____",
    ];
    const KW5: &[[u8; 10]] = &[
        *b"r#async___",
        *b"r#await___",
        *b"r#break___",
        *b"r#const___",
        *b"r#crate___",
        *b"r#false___",
        *b"r#final___",
        *b"r#macro___",
        *b"r#match___",
        *b"r#trait___",
        *b"r#where___",
        *b"r#while___",
        *b"r#yield___",
    ];
    const KW6: &[[u8; 10]] = &[
        *b"r#become__",
        *b"r#extern__",
        *b"r#return__",
        *b"r#static__",
        *b"r#struct__",
        *b"r#typeof__",
        *b"r#unsafe__",
    ];
    const KW7: &[[u8; 10]] = &[*b"r#unsized_", *b"r#virtual_"];
    const KW8: &[[u8; 10]] = &[*b"r#abstract", *b"r#continue", *b"r#override"];
    const KWS: &[&[[u8; 10]]] = &[KW0, KW1, KW2, KW3, KW4, KW5, KW6, KW7, KW8];

    if ident.len() > 8 {
        return ident;
    }
    let kws = KWS[ident.len()];

    let mut padded_ident = [b'_'; 8];
    padded_ident[..ident.len()].copy_from_slice(ident.as_bytes());

    let idx =
        kws.binary_search_by(|probe| <[u8; 8]>::try_from(&probe[2..]).unwrap().cmp(&padded_ident));
    match idx {
        Ok(idx) => unsafe { std::str::from_utf8_unchecked(&kws[idx][..ident.len() + 2]) },
        Err(_) => ident,
    }
}

fn normalize_identifier_phf(ident: &str) -> &str {
    static USE_RAW: phf::Map<&str, &str> = phf::phf_map! {
        "abstract" => "r#abstract",
        "as" => "r#as",
        "async" => "r#async",
        "await" => "r#await",
        "become" => "r#become",
        "box" => "r#box",
        "break" => "r#break",
        "const" => "r#const",
        "continue" => "r#continue",
        "crate" => "r#crate",
        "do" => "r#do",
        "dyn" => "r#dyn",
        "else" => "r#else",
        "enum" => "r#enum",
        "extern" => "r#extern",
        "false" => "r#false",
        "final" => "r#final",
        "fn" => "r#fn",
        "for" => "r#for",
        "if" => "r#if",
        "impl" => "r#impl",
        "in" => "r#in",
        "let" => "r#let",
        "macro" => "r#macro",
        "match" => "r#match",
        "mod" => "r#mod",
        "move" => "r#move",
        "mut" => "r#mut",
        "override" => "r#override",
        "priv" => "r#priv",
        "pub" => "r#pub",
        "ref" => "r#ref",
        "return" => "r#return",
        "static" => "r#static",
        "struct" => "r#struct",
        "trait" => "r#trait",
        "true" => "r#true",
        "try" => "r#try",
        "type" => "r#type",
        "typeof" => "r#typeof",
        "unsafe" => "r#unsafe",
        "unsized" => "r#unsized",
        "use" => "r#use",
        "virtual" => "r#virtual",
        "where" => "r#where",
        "while" => "r#while",
        "yield" => "r#yield",
    };

    USE_RAW.get(ident).copied().unwrap_or(ident)
}

struct Words(&'static str);

impl Default for Words {
    fn default() -> Self {
        Self(include_str!("../../askama_derive/src/generator.rs"))
    }
}

impl Iterator for Words {
    type Item = &'static str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut pos = self.0;
        loop {
            if pos.is_empty() {
                self.0 = "";
                return None;
            } else if matches!(pos.as_bytes()[0], b'_' | b'a'..=b'z' | b'A'..=b'Z') {
                break;
            }

            let mut chars = pos.chars();
            chars.next();
            pos = chars.as_str();
        }

        let start = pos;
        loop {
            if pos.is_empty() {
                self.0 = "";
                return Some(start);
            } else if !matches!(pos.as_bytes()[0], b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9') {
                self.0 = pos;
                return Some(&start[..start.len() - pos.len()]);
            }

            let mut chars = pos.chars();
            chars.next();
            pos = chars.as_str();
        }
    }
}

impl FusedIterator for Words {}
