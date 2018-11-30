use nom::types::CompleteByteSlice as Input;
use nom::{alpha, digit};
use std::str::{from_utf8, Utf8Error};

named!(
    pub expression<Input, &str>,
    map_res!(
        recognize!(tuple!(
            map_res!(alt!(tag!("&") | tag!("*") | tag!("")), input_to_str),
            add_return_error!(
                err_str!("Expected rust expression"),
                alt_complete!(
                    rust_name |
                    map_res!(digit, input_to_str) |
                    quoted_string |
                    expr_in_parens |
                    expr_in_brackets
                )
            ),
            fold_many0!(
                alt_complete!(
                    preceded!(tag!("."), expression) |
                    preceded!(tag!("::"), expression) |
                    expr_in_parens |
                    expr_in_braces |
                    expr_in_brackets |
                    preceded!(tag!("!"), expr_in_parens) |
                    preceded!(tag!("!"), expr_in_brackets)),
                (),
                |_, _| ()
            )
        )),
        input_to_str
    )
);

pub fn input_to_str(s: Input) -> Result<&str, Utf8Error> {
    from_utf8(&s)
}

named!(pub comma_expressions<Input, String>,
       map!(separated_list!(preceded!(tag!(","), many0!(tag!(" "))),
                            expression),
            |list: Vec<_>| list.join(", ")));

named!(
    pub rust_name<Input, &str>,
    map_res!(
        recognize!(
            pair!(alt!(tag!("_") | alpha),
                  opt!(is_a!("_0123456789abcdefghijklmnopqrstuvwxyz")))
        ),
        input_to_str
));

named!(
    expr_in_parens<Input, &str>,
    map_res!(
        recognize!(delimited!(
            tag!("("),
            many0!(alt!(
                value!((), is_not!("[]()\"/")) |
                value!((), expr_in_braces) |
                value!((), expr_in_brackets) |
                value!((), expr_in_parens) |
                value!((), quoted_string) |
                value!((), rust_comment) |
                value!((), terminated!(tag!("/"), none_of!("*")))
            )),
            tag!(")")
        )),
        input_to_str
    )
);

named!(
    expr_in_brackets<Input, &str>,
    map_res!(
        recognize!(delimited!(
            tag!("["),
            many0!(alt!(
                value!((), is_not!("[]()\"/")) |
                value!((), expr_in_brackets) |
                value!((), expr_in_braces) |
                value!((), expr_in_parens) |
                value!((), quoted_string) |
                value!((), rust_comment) |
                value!((), terminated!(tag!("/"), none_of!("*")))
            )),
            tag!("]")
        )),
        input_to_str
    )
);

named!(
    pub expr_in_braces<Input, &str>,
    map_res!(
        recognize!(delimited!(
            tag!("{"),
            many0!(alt!(
                value!((), is_not!("{}[]()\"/")) |
                value!((), expr_in_brackets) |
                value!((), expr_in_parens) |
                value!((), quoted_string) |
                value!((), rust_comment) |
                value!((), terminated!(tag!("/"), none_of!("*")))
            )),
            tag!("}")
        )),
        input_to_str
    )
);

named!(
    quoted_string<Input, &str>,
    map_res!(
        recognize!(delimited!(
            char!('"'),
            escaped!(is_not!("\"\\"), '\\', one_of!("\"\\")),
            char!('"')
        )),
        input_to_str
    )
);

named!(
    rust_comment<Input, Input>,
    delimited!(
        tag!("/*"),
        recognize!(many0!(alt_complete!(
            is_not!("*") | terminated!(tag!("*"), not!(tag!("/")))
        ))),
        tag!("*/")
    )
);

#[cfg(test)]
mod test {
    use expression::expression;
    use nom::types::CompleteByteSlice as Input;

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
        assert_eq!(
            expression(Input(expr.as_bytes())),
            Ok((Input(&b""[..]), expr))
        );
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
        show_errors(&mut buf, input, expression(Input(input)), ":");
        String::from_utf8(buf).unwrap()
    }
}
