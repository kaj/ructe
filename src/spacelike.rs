use crate::parseresult::PResult;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{multispace1, none_of};
use nom::combinator::value;
use nom::multi::many0;
use nom::sequence::preceded;
use nom::Parser as _;

pub fn spacelike(input: &[u8]) -> PResult<()> {
    value((), many0(alt((comment, value((), multispace1))))).parse(input)
}

pub fn comment(input: &[u8]) -> PResult<()> {
    preceded(tag("@*"), comment_tail).parse(input)
}

pub fn comment_tail(input: &[u8]) -> PResult<()> {
    preceded(
        many0(alt((
            value((), is_not("*")),
            value((), preceded(tag("*"), none_of("@"))),
        ))),
        value((), tag("*@")),
    )
    .parse(input)
}

#[cfg(test)]
mod test {
    use super::{comment, spacelike};
    use nom::{error::ErrorKind, Err};
    use nom_language::error::{VerboseError, VerboseErrorKind};

    #[test]
    fn comment1() {
        assert_eq!(comment(b"@* a simple comment *@"), Ok((&b""[..], ())));
    }
    #[test]
    fn comment2() {
        let space_before = b" @* comment *@";
        assert_eq!(
            comment(space_before),
            Err(Err::Error(VerboseError {
                errors: vec![(
                    &space_before[..],
                    VerboseErrorKind::Nom(ErrorKind::Tag),
                )],
            })),
        )
    }
    #[test]
    fn comment3() {
        assert_eq!(
            comment(b"@* comment *@ & stuff"),
            Ok((&b" & stuff"[..], ()))
        );
    }
    #[test]
    fn comment4() {
        assert_eq!(
            comment(b"@* comment *@ and @* another *@"),
            Ok((&b" and @* another *@"[..], ()))
        );
    }
    #[test]
    fn comment5() {
        assert_eq!(
            comment(b"@* comment containing * and @ *@"),
            Ok((&b""[..], ()))
        );
    }
    #[test]
    fn comment6() {
        assert_eq!(
            comment(b"@*** peculiar comment ***@***"),
            Ok((&b"***"[..], ()))
        );
    }

    #[test]
    fn spacelike_empty() {
        assert_eq!(spacelike(b""), Ok((&b""[..], ())));
    }
    #[test]
    fn spacelike_simple() {
        assert_eq!(spacelike(b" "), Ok((&b""[..], ())));
    }
    #[test]
    fn spacelike_long() {
        assert_eq!(
            spacelike(
                b"\n\
                  @* a comment on a line by itself *@\n\
                  \t\t   \n\n\r\n\
                  @*another comment*@    something else"
            ),
            Ok((&b"something else"[..], ()))
        );
    }
}
