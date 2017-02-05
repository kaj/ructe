use nom::alpha;
use std::str::from_utf8;

named!(pub expression<&[u8], String>,
       do_parse!(
           pre: alt!(tag!("&") | tag!("!") | tag!("ref ") | tag!("")) >>
           name: alt!(rust_name |
                      do_parse!(char!('"') >>
                                text: escaped!(is_not!("\"\\"),
                                               '\\', one_of!("\"\\")) >>
                                char!('"') >>
                                (format!("\"{}\"",
                                         from_utf8(text).unwrap()))) |
                      do_parse!(tag!("[") >> args: comma_expressions >>
                                tag!("]") >>
                                (format!("[{}]", args)))) >>
           post: fold_many0!(
               alt_complete!(
                   do_parse!(tag!(".") >> post: expression >>
                             (format!(".{}", post))) |
                   do_parse!(tag!("(") >> args: comma_expressions >>
                             tag!(")") >>
                             (format!("({})", args))) |
                   do_parse!(tag!("[") >> args: comma_expressions >>
                             tag!("]") >>
                             (format!("[{}]", args))) |
                   do_parse!(tag!("!(") >> args: comma_expressions >>
                             tag!(")") >>
                             (format!("!({})", args))) |
                   do_parse!(tag!("![") >> args: comma_expressions >>
                             tag!("]") >>
                             (format!("![{}]", args)))),
               String::new(),
               |mut acc: String, item: String| {
                   acc.push_str(&item);
                   acc
               }) >>
           (format!("{}{}{}", from_utf8(pre).unwrap(), name, post))));

named!(comma_expressions<&[u8], String>,
       map!(separated_list!(tag!(", "), expression),
            |list: Vec<_>| list.join(", ")));

named!(pub rust_name<&[u8], String>,
       do_parse!(first: alpha >>
                 rest: opt!(is_a!("_0123456789abcdefghijklmnopqrstuvwxyz")) >>
                 (format!("{}{}",
                          from_utf8(first).unwrap(),
                          from_utf8(rest.unwrap_or(b"")).unwrap()))));

#[cfg(test)]
mod test {
    use expression::expression;
    use nom;
    use nom::IResult::{Done, Error};

    #[test]
    fn expression_1() {
        check_expr("foo");
    }
    #[test]
    fn expression_2() {
        check_expr("x15");
    }
    #[test]
    fn expression_3() {
        check_expr("a_b_c");
    }
    #[test]
    fn expression_4() {
        check_expr("foo.bar");
    }
    #[test]
    fn expression_5() {
        check_expr("foo.bar.baz");
    }
    #[test]
    fn expression_6() {
        check_expr("!foo.is_empty()");
    }
    #[test]
    fn expression_7() {
        check_expr("foo(x, a.b.c(), d)");
    }
    #[test]
    fn expression_8() {
        check_expr("foo(&\"x\").bar");
    }
    #[test]
    fn expression_9() {
        check_expr("foo().bar(x).baz");
    }
    #[test]
    fn expression_str() {
        check_expr("\"foo\"");
    }
    #[test]
    fn expression_str_with_escaped_quotes() {
        check_expr("\"Hello \\\"world\\\"\"");
    }
    #[test]
    fn expression_slice() {
        check_expr("&[foo, bar]");
    }
    #[test]
    fn expression_slice_empty() {
        check_expr("&[]");
    }

    fn check_expr(expr: &str) {
        for post in &[" ", ", ", "! ", "? ", "<a>", "##", ". ", "\"", "'"] {
            let input = format!("{}{}", expr, post);
            assert_eq!(expression(input.as_bytes()),
                       Done(post.as_bytes(), expr.to_string()));
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
