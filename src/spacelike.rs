use nom::{multispace};

named!(pub spacelike<&[u8], ()>,
       chain!(many0!(alt!(
           comment |
           chain!(multispace, ||()))),
              || ()));

named!(pub comment<&[u8], ()>,
       value!((), delimited!(tag!("@*"),
                             many0!(alt!(
                                 chain!(is_not!("*"), ||()) |
                                 chain!(tag!("*") ~ none_of!("@"), ||())
                                     )),
                             tag!("*@"))));

#[cfg(test)]
mod test {
    use nom::IResult::{Done, Error};
    use nom;
    use spacelike::comment;

    #[test]
    fn comment1() {
        assert_eq!(comment(b"@* a simple comment *@"), Done(&b""[..], ()));
    }
    #[test]
    fn comment2() {
        assert_eq!(comment(b" @* comment *@"),
                   Error(nom::Err::Position(nom::ErrorKind::Tag,
                                            &b" @* comment *@"[..])));
    }
    #[test]
    fn comment3() {
        assert_eq!(comment(b"@* comment *@ & stuff"), Done(&b" & stuff"[..], ()));
    }
    #[test]
    fn comment4() {
        assert_eq!(comment(b"@* comment *@ and @* another *@"),
                   Done(&b" and @* another *@"[..], ()));
    }
    #[test]
    fn comment5() {
        assert_eq!(comment(b"@* comment containing * and @ *@"),
                   Done(&b""[..], ()));
    }
    #[test]
    fn comment6() {
        assert_eq!(comment(b"@*** peculiar comment ***@***"),
                   Done(&b"***"[..], ()));
    }
}
