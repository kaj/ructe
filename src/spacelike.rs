use nom::multispace;

named!(pub spacelike<&[u8], ()>,
       map!(many0!(alt!(
           comment |
           map!(multispace, |_|()))),
            |_| ()));

named!(pub comment<&[u8], ()>,
       value!((), delimited!(tag!("@*"),
                             many0!(alt!(
                                 map!(is_not!("*"), |_|()) |
                                 do_parse!(tag!("*") >> none_of!("@") >> ())
                                     )),
                             tag!("*@"))));

#[cfg(test)]
mod test {
    use nom::ErrorKind;
    use nom::IResult::{Done, Error};
    use nom::verbose_errors::Err;
    use spacelike::{comment, spacelike};

    #[test]
    fn comment1() {
        assert_eq!(
            comment(b"@* a simple comment *@"),
            Done(&b""[..], ())
        );
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
