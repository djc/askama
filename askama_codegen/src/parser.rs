use nom::{self, IResult};

fn take_content(i: &[u8]) -> IResult<&[u8], &[u8]> {
    if i.len() < 1 || i[0] == b'{' {
        return IResult::Error(error_position!(nom::ErrorKind::TakeUntil, i));
    }
    for (j, c) in i.iter().enumerate() {
        if *c == b'{' {
            if i.len() < j + 2 {
                return IResult::Done(&i[..0], &i[..]);
            } else if i[j + 1] == '{' as u8 {
                return IResult::Done(&i[j..], &i[..j]);
            } else if i[j + 1] == '%' as u8 {
                return IResult::Done(&i[j..], &i[..j]);
            }
        }
    }
    IResult::Done(&i[..0], &i[..])
}

named!(var_expr, delimited!(tag!("{{"), take_until!("}}"), tag!("}}")));

named!(parse_template< Vec<&[u8]> >, many1!(alt!(take_content | var_expr)));

pub fn parse<'a>(src: &'a str) -> Vec<&'a [u8]> {
    match parse_template(src.as_bytes()) {
        IResult::Done(_, res) => res,
        _ => panic!("problems parsing template source"),
    }
}
