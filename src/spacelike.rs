use nom::multispace;
use nom::types::CompleteByteSlice as Input;

named!(pub spacelike<Input, ()>,
       map!(many0!(alt!(
           comment |
           map!(multispace, |_|()))),
            |_| ()));

named!(
    pub comment<Input, ()>,
    preceded!(tag!("@*"), comment_tail)
);

named!(
    pub comment_tail<Input, ()>,
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
    use nom::types::CompleteByteSlice as Input;
    use nom::{Context, Err, ErrorKind};
    use spacelike::{comment, spacelike};

    #[test]
    fn comment1() {
        assert_eq!(
            comment(Input(b"@* a simple comment *@")),
            Ok((Input(&b""[..]), ()))
        );
    }
    #[test]
    fn comment2() {
        let space_before = Input(b" @* comment *@");
        assert_eq!(
            comment(space_before),
            Err(Err::Error(Context::Code(space_before, ErrorKind::Tag)))
        )
    }
    #[test]
    fn comment3() {
        assert_eq!(
            comment(Input(b"@* comment *@ & stuff")),
            Ok((Input(&b" & stuff"[..]), ()))
        );
    }
    #[test]
    fn comment4() {
        assert_eq!(
            comment(Input(b"@* comment *@ and @* another *@")),
            Ok((Input(&b" and @* another *@"[..]), ()))
        );
    }
    #[test]
    fn comment5() {
        assert_eq!(
            comment(Input(b"@* comment containing * and @ *@")),
            Ok((Input(&b""[..]), ()))
        );
    }
    #[test]
    fn comment6() {
        assert_eq!(
            comment(Input(b"@*** peculiar comment ***@***")),
            Ok((Input(&b"***"[..]), ()))
        );
    }

    #[test]
    fn spacelike_empty() {
        assert_eq!(spacelike(Input(b"")), Ok((Input(&b""[..]), ())));
    }
    #[test]
    fn spacelike_simple() {
        assert_eq!(spacelike(Input(b" ")), Ok((Input(&b""[..]), ())));
    }
    #[test]
    fn spacelike_long() {
        assert_eq!(
            spacelike(Input(
                b"\n\
                  @* a comment on a line by itself *@\n\
                  \t\t   \n\n\r\n\
                  @*another comment*@    something else"
            )),
            Ok((Input(&b"something else"[..]), ()))
        );
    }
}
