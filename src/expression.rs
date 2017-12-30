use nom::{alpha, digit};
use std::str::from_utf8;

named!(pub expression<&[u8], String>,
       do_parse!(
           pre: alt!(tag!("&") | tag!("!") | tag!("*") | tag!("ref ") |
                     tag!("")) >>
           name: return_error!(err_str!("Expected rust expression"),
                              alt!(rust_name |
                      map!(digit, |d| from_utf8(d).unwrap().to_string()) |
                      do_parse!(char!('"') >>
                                text: escaped!(is_not!("\"\\"),
                                               '\\', one_of!("\"\\")) >>
                                char!('"') >>
                                (format!("\"{}\"",
                                         from_utf8(text).unwrap()))) |
                      do_parse!(tag!("(") >> args: comma_expressions >>
                                tag!(")") >>
                                (format!("({})", args))) |
                      do_parse!(tag!("[") >> args: comma_expressions >>
                                tag!("]") >>
                                (format!("[{}]", args))))) >>
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

named!(pub comma_expressions<&[u8], String>,
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
    use nom::IResult::Done;

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
    #[test]
    fn expression_number() {
        check_expr("42");
    }

    fn check_expr(expr: &str) {
        for post in &[" ", ", ", "! ", "? ", "<a>", "##", ". ", "\"", "'"] {
            assert_eq!(
                expression(format!("{}{}", expr, post).as_bytes()),
                Done(post.as_bytes(), expr.to_string())
            );
        }
    }

    #[test]
    fn non_expression_a() {
        assert_eq!(
            expression_error_message(b".foo"),
            ":   1:.foo\n\
             :     ^ Expected rust expression\n\
             :   1:.foo\n\
             :     ^ Alt\n"
        );
    }
    #[test]
    fn non_expression_b() {
        assert_eq!(
            expression_error_message(b" foo"),
            ":   1: foo\n\
             :     ^ Expected rust expression\n\
             :   1: foo\n\
             :     ^ Alt\n"
        );
    }
    #[test]
    fn non_expression_c() {
        assert_eq!(
            expression_error_message(b"(+)"),
            ":   1:(+)\n\
             :     ^ Expected rust expression\n\
             :   1:(+)\n\
             :     ^ Alt\n"
        );
    }
    fn expression_error_message(input: &[u8]) -> String {
        use super::super::show_errors;
        let mut buf = Vec::new();
        show_errors(&mut buf, input, expression(input), ":");
        String::from_utf8(buf).unwrap()
    }
}
