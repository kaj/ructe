use nom::alpha;
use std::str::from_utf8;

named!(pub expression<&[u8], String>,
       chain!(pre: alt!(tag!("&") | tag!("!") | tag!("ref ") | tag!("")) ~
              name: alt!(rust_name |
                        chain!(char!('"') ~
                               text: is_not!("\"") ~ char!('"'),
                               || format!("\"{}\"",
                                          from_utf8(text).unwrap()))) ~
              post: fold_many0!(
                  alt_complete!(
                      chain!(tag!(".") ~ post: expression,
                             || format!(".{}", post)) |
                      chain!(tag!("(") ~ args: comma_expressions ~ tag!(")"),
                             || format!("({})", args)) |
                      chain!(tag!("[") ~ args: comma_expressions ~ tag!("]"),
                             || format!("[{}]", args)) |
                      chain!(tag!("!(") ~ args: comma_expressions ~ tag!(")"),
                             || format!("!({})", args)) |
                      chain!(tag!("![") ~ args: comma_expressions ~ tag!("]"),
                             || format!("![{}]", args))),
                  String::new(),
                  |mut acc: String, item: String| {
                      acc.push_str(&item);
                      acc
                  }),
              || format!("{}{}{}", from_utf8(pre).unwrap(), name, post)));

named!(comma_expressions<&[u8], String>,
       chain!(list: separated_list!(tag!(", "), expression),
              || list.join(", ")));

named!(pub rust_name<&[u8], String>,
       chain!(first: alpha ~
              rest: opt!(is_a!("_0123456789abcdefghijklmnopqrstuvwxyz")),
              || format!("{}{}",
                         from_utf8(first).unwrap(),
                         from_utf8(rest.unwrap_or(b"")).unwrap())));

#[cfg(test)]
mod test {
    use expression::expression;
    use nom;
    use nom::IResult::{Done, Error};
    use std::str::from_utf8;

    #[test]
    fn expressions() {
        // Proper expressions, each followed by two non-expression characters.
        for input in &[&b"foo  "[..],
                       &b"foo<x"[..],
                       &b"foo. "[..],
                       &b"foo! "[..],
                       &b"foo? "[..],
                       &b"x15  "[..],
                       &b"a_b_c  "[..],
                       &b"foo. "[..],
                       &b"foo.bar  "[..],
                       &b"boo.bar.baz##"[..],
                       &b"!foo.is_empty()  "[..],
                       &b"foo(x, a.b.c(), d)  "[..],
                       &b"foo(&\"x\").bar  "[..],
                       &b"foo().bar(x).baz, "[..]] {
            let i = input.len() - 2;
            assert_eq!(expression(*input),
                       Done(&input[i..],
                            from_utf8(&input[..i]).unwrap().to_string()));
        }
    }
    #[test]
    fn non_expressions() {
        // non-expressions
        // TODO See if I can get nom to produce more helpfull errors.
        for input in &[&b".foo"[..], &b" foo"[..], &b"()"[..]] {
            assert_eq!(expression(*input), Error(nom::ErrorKind::Alt));
        }
    }
}
