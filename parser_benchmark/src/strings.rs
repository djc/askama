use arbitrary::{Arbitrary, Unstructured};

use crate::ToSource;

#[derive(Debug)]
pub(crate) struct Printable(pub(crate) String);

const _: () = {
    impl ToSource for Printable {
        fn write_into(&self, buf: &mut String) {
            buf.push_str(&self.0);
        }
    }

    type Data = (
        PrintableDelim,
        Option<[PrintableChar; 3]>,
        Vec<PrintableChar>,
        PrintableDelim,
    );

    impl<'a> Arbitrary<'a> for Printable {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let (start, mid, extra, end) = Data::arbitrary(u)?;
            let mid = match &mid {
                Some(mid) => mid.as_slice(),
                None => &[],
            };
            let mut result = String::with_capacity(2 + mid.len() + extra.len());
            result.push(start.0 as char);
            for s in [mid, extra.as_slice()] {
                for &PrintableChar(c) in s {
                    if matches!(c, b'{' | b'%' | b'#') && result.ends_with('{') {
                        result.push(' ');
                    }
                    result.push(c as char);
                }
            }
            if matches!(end.0, b'{' | b'%' | b'#') && result.ends_with('{') {
                result.push(' ');
            }
            result.push(end.0 as char);
            if result.ends_with('{') {
                result.push('.');
            }
            Ok(Self(result))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            Data::size_hint(depth)
        }
    }

    struct PrintableDelim(u8);

    impl<'a> Arbitrary<'a> for PrintableDelim {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            const LOWER: &[u8] = &[
                b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd',
                b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r',
                b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'A', b'B', b'C', b'D', b'E', b'F',
                b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
                b'U', b'V', b'W', b'X', b'Y', b'Z', b'!', b'"', b'$', b'%', b'&', b'\'', b'(',
                b')', b'*', b'+', b',', b'-', b'.', b'/', b':', b';', b'<', b'=', b'>', b'?', b'@',
                b'[', b'\\', b']', b'^', b'_', b'`', b'{', b'|', b'}', b'~',
            ];
            Ok(Self(*u.choose(LOWER)?))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            usize::size_hint(depth)
        }
    }

    struct PrintableChar(u8);

    impl<'a> Arbitrary<'a> for PrintableChar {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            const LOWER: &[u8] = &[
                b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd',
                b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r',
                b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'A', b'B', b'C', b'D', b'E', b'F',
                b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
                b'U', b'V', b'W', b'X', b'Y', b'Z', b'!', b'"', b'$', b'%', b'&', b'\'', b'(',
                b')', b'*', b'+', b',', b'-', b'.', b'/', b':', b';', b'<', b'=', b'>', b'?', b'@',
                b'[', b'\\', b']', b'^', b'_', b'`', b'{', b'|', b'}', b'~', b' ', b'\t', b'\n',
            ];
            Ok(Self(*u.choose(LOWER)?))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            usize::size_hint(depth)
        }
    }
};

#[derive(Debug)]
pub(crate) struct PrintableNoGrouping(pub(crate) String);

const _: () = {
    impl ToSource for PrintableNoGrouping {
        fn write_into(&self, buf: &mut String) {
            buf.push_str(&self.0);
        }
    }

    type Data = (
        PrintableDelim,
        Option<[PrintableChar; 3]>,
        Vec<PrintableChar>,
        PrintableDelim,
    );

    impl<'a> Arbitrary<'a> for PrintableNoGrouping {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let (start, mid, extra, end) = Data::arbitrary(u)?;
            let mid = match &mid {
                Some(mid) => mid.as_slice(),
                None => &[],
            };
            let mut result = String::with_capacity(2 + mid.len() + extra.len());
            result.push(start.0 as char);
            for s in [mid, extra.as_slice()] {
                for &PrintableChar(c) in s {
                    result.push(c as char);
                }
            }
            result.push(end.0 as char);
            Ok(Self(result))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            Data::size_hint(depth)
        }
    }

    struct PrintableDelim(u8);

    impl<'a> Arbitrary<'a> for PrintableDelim {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            const LOWER: &[u8] = &[
                b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd',
                b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r',
                b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'A', b'B', b'C', b'D', b'E', b'F',
                b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
                b'U', b'V', b'W', b'X', b'Y', b'Z', b'!', b'$', b'%', b'&', b'*', b'+', b',', b'-',
                b'/', b':', b';', b'<', b'=', b'>', b'?', b'@', b'^', b'_', b'`', b'|', b'~',
            ];
            Ok(Self(*u.choose(LOWER)?))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            usize::size_hint(depth)
        }
    }

    struct PrintableChar(u8);

    impl<'a> Arbitrary<'a> for PrintableChar {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            const LOWER: &[u8] = &[
                b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd',
                b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r',
                b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'A', b'B', b'C', b'D', b'E', b'F',
                b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
                b'U', b'V', b'W', b'X', b'Y', b'Z', b'!', b'$', b'%', b'&', b'*', b'+', b',', b'-',
                b'/', b':', b';', b'<', b'=', b'>', b'?', b'@', b'^', b'_', b'`', b'|', b'~', b' ',
                b'\t', b'\n',
            ];
            Ok(Self(*u.choose(LOWER)?))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            usize::size_hint(depth)
        }
    }
};

#[derive(Debug)]
pub(crate) struct Space(pub(crate) String);

const _: () = {
    impl ToSource for Space {
        fn write_into(&self, buf: &mut String) {
            buf.push_str(&self.0);
        }
    }

    type Data = Option<Vec<SpaceChar>>;

    impl<'a> Arbitrary<'a> for Space {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let spaces = match Data::arbitrary(u)? {
                Some(spaces) if !spaces.is_empty() => spaces,
                _ => return Ok(Self("".to_owned())),
            };
            let mut result = String::with_capacity(spaces.len());
            for SpaceChar(c) in spaces {
                result.push(c as char);
            }
            Ok(Self(result))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            Data::size_hint(depth)
        }
    }

    struct SpaceChar(u8);

    impl<'a> Arbitrary<'a> for SpaceChar {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            const LOWER: &[u8] = &[b'\n', b'\n', b' '];
            Ok(Self(*u.choose(LOWER)?))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            u8::size_hint(depth)
        }
    }
};

#[derive(Debug)]
pub(crate) struct Ident(pub(crate) String);

const _: () = {
    impl ToSource for Ident {
        fn write_into(&self, buf: &mut String) {
            buf.push_str(&self.0);
        }
    }

    const KEYWORDS: &[&str] = &[
        "abstract",
        "as",
        "become",
        "box",
        "break",
        "const",
        "continue",
        "crate",
        "do",
        "dyn",
        "else",
        "enum",
        "extern",
        "false",
        "final",
        "fn",
        "for",
        "if",
        "impl",
        "in",
        "let",
        "loop",
        "macro_rules",
        "macro",
        "match",
        "mod",
        "move",
        "mut",
        "override",
        "priv",
        "pub",
        "ref",
        "return",
        "self",
        "Self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "try",
        "type",
        "typeof",
        "union",
        "unsafe",
        "unsized",
        "use",
        "virtual",
        "where",
        "while",
        "yield",
    ];

    type Data = (IdentStart, Option<[IdentTail; 3]>, Vec<IdentTail>);

    impl<'a> Arbitrary<'a> for Ident {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let (start, mid, extra) = Data::arbitrary(u)?;
            let mid = match &mid {
                Some(mid) => mid.as_slice(),
                None => &[],
            };
            let mut result = String::with_capacity(1 + mid.len() + extra.len());
            result.push(start.0 as char);
            for s in [mid, extra.as_slice()] {
                for &IdentTail(c) in s {
                    result.push(c as char);
                }
            }
            if result.len() >= 2 && KEYWORDS.binary_search(&result.as_str()).is_ok() {
                result.replace_range(1..=1, "_");
            }
            Ok(Self(result))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            Data::size_hint(depth)
        }
    }

    struct IdentStart(u8);

    impl<'a> Arbitrary<'a> for IdentStart {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            const LOWER: &[u8] = &[
                b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
                b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
            ];
            Ok(Self(*u.choose(LOWER)?))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            usize::size_hint(depth)
        }
    }

    struct IdentTail(u8);

    impl<'a> Arbitrary<'a> for IdentTail {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            const ANY: &[u8] = &[
                b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
                b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'0', b'1',
                b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'_',
            ];
            Ok(Self(*u.choose(ANY)?))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            usize::size_hint(depth)
        }
    }
};

#[derive(Debug)]
pub(crate) struct TypeName(pub(crate) Vec<([Space; 2], String)>);

const _: () = {
    impl ToSource for TypeName {
        fn write_into(&self, buf: &mut String) {
            for (idx, (spaces, name)) in self.0.iter().enumerate() {
                if idx > 0 {
                    spaces[0].write_into(buf);
                    buf.push_str("::");
                    spaces[1].write_into(buf);
                }
                buf.push_str(name);
            }
        }
    }

    type Data = (
        Option<([Space; 2], ([Space; 2], Ident))>,
        Vec<([Space; 2], Ident)>,
        [Space; 2],
        Name,
    );

    impl<'a> Arbitrary<'a> for TypeName {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let (absolute, path, spaces, name) = Data::arbitrary(u)?;
            let mut result: Vec<([Space; 2], String)> =
                Vec::with_capacity(1 + path.len() + 2 * absolute.is_some() as usize);
            if let Some((empty_spaces, (spaces, ident))) = absolute {
                result.push((empty_spaces, "".to_owned()));
                result.push((spaces, ident.0));
            }
            for (spaces, Ident(name)) in path {
                result.push((spaces, name));
            }
            result.push((spaces, name.0));
            Ok(Self(result))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            Data::size_hint(depth)
        }
    }

    #[derive(Debug)]
    struct Name(String);

    impl<'a> Arbitrary<'a> for Name {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let (start, tail) = <(NameStart, Vec<NameTail>)>::arbitrary(u)?;
            let mut result = String::with_capacity(1 + tail.len());
            result.push(start.0 as char);
            for NameTail(c) in tail {
                result.push(c as char);
            }
            Ok(Self(result))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            <(NameStart, Vec<NameTail>)>::size_hint(depth)
        }
    }

    struct NameStart(u8);

    impl<'a> Arbitrary<'a> for NameStart {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            const LOWER: &[u8] = &[
                b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N',
                b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z',
            ];
            Ok(Self(*u.choose(LOWER)?))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            usize::size_hint(depth)
        }
    }

    struct NameTail(u8);

    impl<'a> Arbitrary<'a> for NameTail {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            const ANY: &[u8] = &[
                b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
                b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'A', b'B',
                b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
                b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'0', b'1', b'2', b'3',
                b'4', b'5', b'6', b'7', b'8', b'9', b'_',
            ];
            Ok(Self(*u.choose(ANY)?))
        }

        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            usize::size_hint(depth)
        }
    }
};
