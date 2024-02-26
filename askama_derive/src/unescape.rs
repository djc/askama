// This code is a simplified version of the `rustc_lexer` unescape.

use crate::CompileError;
use proc_macro::Literal;
use std::str::Chars;

fn scan_unicode(chars: &mut Chars<'_>) -> Result<char, CompileError> {
    // We've parsed '\u', now we have to parse '{..}'.

    if chars.next() != Some('{') {
        return Err("expected { after \\u".into());
    }

    // First character must be a hexadecimal digit.
    let mut value: u32 = match chars
        .next()
        .ok_or_else(|| CompileError::from("unclosed `\\u`"))?
    {
        '}' => return Err("invalid empty unicode escape `\\u{}`".into()),
        c => c.to_digit(16).ok_or_else(|| {
            CompileError::from(format!("unexpected non hex character after \\u: `{c}`"))
        })?,
    };
    let mut n_digits = 1;

    // First character is valid, now parse the rest of the number
    // and closing brace.
    loop {
        match chars.next() {
            None => return Err("unclosed unicode escape".into()),
            Some('_') => continue,
            Some('}') => {
                break std::char::from_u32(value).ok_or_else(|| {
                    CompileError::from(format!(
                        "character code {value:x} is not a valid unicode character"
                    ))
                });
            }
            Some(c) => {
                n_digits += 1;
                let digit: u32 = c.to_digit(16).ok_or_else(|| {
                    CompileError::from(format!("unexpected non hex character after `\\u`: `{c}`"))
                })?;
                if n_digits > 6 {
                    return Err("overlong unicode escape (must have at most 6 hex digits)".into());
                }
                value = value * 16 + digit;
            }
        };
    }
}

fn scan_escape(chars: &mut Chars<'_>) -> Result<char, CompileError> {
    // Previous character was '\\', unescape what follows.
    let res = match chars.next().ok_or("expected a character after `\\`")? {
        '"' => b'"',
        'n' => b'\n',
        'r' => b'\r',
        't' => b'\t',
        '\\' => b'\\',
        '\'' => b'\'',
        '0' => b'\0',

        'x' => {
            // Parse hexadecimal character code.

            let hi = chars
                .next()
                .ok_or_else(|| CompileError::from("too short \\x escape"))?;
            let hi = hi
                .to_digit(16)
                .ok_or_else(|| CompileError::from("unexpected non-hex character after \\x"))?;

            let lo = chars
                .next()
                .ok_or_else(|| CompileError::from("too short \\x escape"))?;
            let lo = lo
                .to_digit(16)
                .ok_or_else(|| CompileError::from("unexpected non-hex character after \\x"))?;

            let value = hi * 16 + lo;

            value as u8
        }

        'u' => return scan_unicode(chars).map(Into::into),
        c => return Err(format!("unknown espace `\\{c}`").into()),
    };
    Ok(res.into())
}

fn skip_ascii_whitespace(chars: &mut Chars<'_>) {
    let tail = chars.as_str();
    let first_non_space = tail
        .bytes()
        .position(|b| b != b' ' && b != b'\t' && b != b'\n' && b != b'\r')
        .unwrap_or(tail.len());
    let tail = &tail[first_non_space..];
    *chars = tail.chars();
}

fn get_str(src: &str) -> Result<String, CompileError> {
    let mut chars = src.chars();
    let mut output = String::with_capacity(src.len());

    while let Some(c) = chars.next() {
        let res = match c {
            '\\' => {
                match chars.clone().next() {
                    Some('\n') => {
                        // Rust language specification requires us to skip whitespaces
                        // if unescaped '\' character is followed by '\n'.
                        // For details see [Rust language reference]
                        // (https://doc.rust-lang.org/reference/tokens.html#string-literals).
                        skip_ascii_whitespace(&mut chars);
                        continue;
                    }
                    _ => scan_escape(&mut chars)?,
                }
            }
            '"' => return Err("unexpected `\"` character".into()),
            '\r' => return Err("unexpected `\r` character".into()),
            c => c,
        };
        output.push(res);
    }
    Ok(output)
}

pub(crate) fn get_str_literal(lit: Literal, ident: &str) -> Result<String, CompileError> {
    let lit = lit.to_string();

    match lit.bytes().next() {
        Some(b'"') => get_str(&lit[1..lit.len() - 1]),
        Some(b'r') => {
            let mut bytes = lit[1..].bytes();
            let mut pounds = 0;
            while let Some(b'#') = bytes.next() {
                pounds += 1;
            }
            let lit = &lit[pounds + 1..];
            if lit.bytes().next() != Some(b'"') {
                return Err(
                    format!("template `{ident}` must be a string literal, found `{lit}`").into(),
                );
            }
            // If the string wasn't closed, we wouldn't be here in the first place...
            let close = lit.rfind('"').unwrap();
            if !lit[close + 1..].bytes().all(|c| c == b'#') || lit[close + 1..].len() != pounds {
                return Err(
                    format!("template `{ident}` must be a string literal, found `{lit}`").into(),
                );
            }
            Ok(lit[1..close].to_string())
        }
        _ => Err(format!("template `{ident}` must be a string literal").into()),
    }
}
