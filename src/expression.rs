use nom::{alpha, digit};
use std::str::from_utf8;

named!(pub expression<&[u8], String>,
       do_parse!(
           pre: map_res!(alt!(tag!("&") | tag!("*") | tag!("")), from_utf8) >>
           name: return_error!(err_str!("Expected rust expression"),
                               alt_complete!(rust_name |
                                             map_res!(digit, from_utf8) |
                                             quoted_string |
                                             expr_in_parens |
                                             expr_in_brackets)) >>
           post: fold_many0!(
               alt_complete!(
                   map!(preceded!(tag!("."), expression),
                        |expr| format!(".{}", expr)) |
                   map!(preceded!(tag!("::"), expression),
                        |expr| format!("::{}", expr)) |
                   map!(expr_in_parens, String::from) |
                   map!(expr_in_brackets, String::from) |
                   map!(preceded!(tag!("!"), expr_in_parens),
                        |expr| format!("!{}", expr)) |
                   map!(preceded!(tag!("!"), expr_in_brackets),
                        |expr| format!("!{}", expr))),
               String::new(),
               |mut acc: String, item: String| {
                   acc.push_str(&item);
                   acc
               }) >>
           (format!("{}{}{}", pre, name, post))));

named!(pub comma_expressions<&[u8], String>,
       map!(separated_list!(preceded!(tag!(","), many0!(tag!(" "))),
                            expression),
            |list: Vec<_>| list.join(", ")));

named!(
    pub rust_name<&[u8], &str>,
    map_res!(
        recognize!(
            pair!(alpha, opt!(is_a!("_0123456789abcdefghijklmnopqrstuvwxyz")))
        ),
        from_utf8
));

named!(
    expr_in_parens<&[u8], &str>,
    map_res!(
        recognize!(delimited!(
            tag!("("),
            many0!(alt!(
                value!((), is_not!("[]()\"/")) |
                value!((), expr_in_brackets) |
                value!((), expr_in_parens) |
                value!((), quoted_string) |
                value!((), rust_comment) |
                value!((), terminated!(tag!("/"), none_of!("*")))
            )),
            tag!(")")
        )),
        from_utf8
    )
);

named!(
    expr_in_brackets<&[u8], &str>,
    map_res!(
        recognize!(delimited!(
            tag!("["),
            many0!(alt!(
                value!((), is_not!("[]()\"/")) |
                value!((), expr_in_brackets) |
                value!((), expr_in_parens) |
                value!((), quoted_string) |
                value!((), rust_comment) |
                value!((), terminated!(tag!("/"), none_of!("*")))
            )),
            tag!("]")
        )),
        from_utf8
    )
);

named!(
    quoted_string<&[u8], &str>,
    map_res!(
        recognize!(delimited!(
            char!('"'),
            escaped!(is_not!("\"\\"), '\\', one_of!("\"\\")),
            char!('"')
        )),
        from_utf8
    )
);

named!(
    rust_comment,
    delimited!(
        tag!("/*"),
        recognize!(many0!(alt_complete!(
            is_not!("*") | preceded!(tag!("*"), not!(tag!("/")))
        ))),
        tag!("*/")
    )
);

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
        check_expr("(!foo.is_empty())");
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
    fn expression_str_paren() {
        check_expr("(\")\")");
    }
    #[test]
    fn expression_enum_variant() {
        check_expr("MyEnum::Variant.method()");
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
    #[test]
    fn expression_with_comment() {
        check_expr("(42 /* truly important number */)");
    }
    #[test]
    fn expression_with_comment_a() {
        check_expr("(42 /* \" */)");
    }
    #[test]
    fn expression_with_comment_b() {
        check_expr("(42 /* ) */)");
    }
    #[test]
    fn expression_arithemtic_in_parens() {
        check_expr("(2 + 3*4 - 5/2)");
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
            expression_error_message(b"(missing end"),
            ":   1:(missing end\n\
             :     ^ Expected rust expression\n\
             :   1:(missing end\n\
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
