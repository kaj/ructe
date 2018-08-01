use nom::multispace;

named!(pub spacelike<&[u8], ()>,
       map!(many0!(alt!(
           comment |
           map!(multispace, |_|()))),
            |_| ()));

named!(
    pub comment<&[u8], ()>,
    preceded!(tag!("@*"), comment_tail)
);

named!(
    pub comment_tail<&[u8], ()>,
    preceded!(
        many0!(alt!(
            value!((), is_not!("*")) |
            value!((), preceded!(tag!("*"), none_of!("@")))
        )),
        value!((), tag!("*@"))
    )
);

#[cfg(test)]
mod test {
    use nom::verbose_errors::Err;
    use nom::ErrorKind;
    use nom::IResult::{Done, Error};
    use spacelike::{comment, spacelike};

    #[test]
    fn comment1() {
        assert_eq!(comment(b"@* a simple comment *@"), Done(&b""[..], ()));
    }
    #[test]
    fn comment2() {
        let space_before = b" @* comment *@";
        assert_eq!(
            comment(space_before),
            Error(Err::Position(ErrorKind::Tag, &space_before[..]))
        )
    }
    #[test]
    fn comment3() {
        assert_eq!(
            comment(b"@* comment *@ & stuff"),
            Done(&b" & stuff"[..], ())
        );
    }
    #[test]
    fn comment4() {
        assert_eq!(
            comment(b"@* comment *@ and @* another *@"),
            Done(&b" and @* another *@"[..], ())
        );
    }
    #[test]
    fn comment5() {
        assert_eq!(
            comment(b"@* comment containing * and @ *@"),
            Done(&b""[..], ())
        );
    }
    #[test]
    fn comment6() {
        assert_eq!(
            comment(b"@*** peculiar comment ***@***"),
            Done(&b"***"[..], ())
        );
    }

    #[test]
    fn spacelike_empty() {
        assert_eq!(spacelike(b""), Done(&b""[..], ()));
    }
    #[test]
    fn spacelike_simple() {
        assert_eq!(spacelike(b" "), Done(&b""[..], ()));
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
            Done(&b"something else"[..], ())
        );
    }
}
