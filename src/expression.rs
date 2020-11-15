use crate::parseresult::PResult;
use nom::branch::alt;
use nom::bytes::complete::{escaped, is_a, is_not, tag};
use nom::character::complete::{alpha1, char, digit1, none_of, one_of};
use nom::combinator::{map, map_res, not, opt, recognize, value};
use nom::error::context; //, VerboseError};
use nom::multi::{fold_many0, many0, separated_list};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use std::str::{from_utf8, Utf8Error};

pub fn expression(input: &[u8]) -> PResult<&str> {
    map_res(
        recognize(context(
            "Expected rust expression",
            tuple((
                map_res(alt((tag("&"), tag("*"), tag(""))), input_to_str),
                alt((
                    rust_name,
                    map_res(digit1, input_to_str),
                    quoted_string,
                    expr_in_parens,
                    expr_in_brackets,
                )),
                fold_many0(
                    alt((
                        preceded(context("separator", tag(".")), expression),
                        preceded(tag("::"), expression),
                        expr_in_parens,
                        expr_in_braces,
                        expr_in_brackets,
                        preceded(tag("!"), expr_in_parens),
                        preceded(tag("!"), expr_in_brackets),
                    )),
                    (),
                    |_, _| (),
                ),
            )),
        )),
        input_to_str,
    )(input)
}

pub fn input_to_str(s: &[u8]) -> Result<&str, Utf8Error> {
    from_utf8(&s)
}

pub fn comma_expressions(input: &[u8]) -> PResult<String> {
    map(
        separated_list(preceded(tag(","), many0(tag(" "))), expression),
        |list: Vec<_>| list.join(", "),
    )(input)
}

pub fn rust_name(input: &[u8]) -> PResult<&str> {
    map_res(
        recognize(pair(
            alt((tag("_"), alpha1)),
            opt(is_a("_0123456789abcdefghijklmnopqrstuvwxyz")),
        )),
        input_to_str,
    )(input)
}

fn expr_in_parens(input: &[u8]) -> PResult<&str> {
    map_res(
        recognize(delimited(tag("("), expr_inside_parens, tag(")"))),
        input_to_str,
    )(input)
}

fn expr_in_brackets(input: &[u8]) -> PResult<&str> {
    map_res(
        recognize(delimited(
            tag("["),
            many0(alt((
                value((), is_not("[]()\"/")),
                value((), expr_in_brackets),
                value((), expr_in_braces),
                value((), expr_in_parens),
                value((), quoted_string),
                value((), rust_comment),
                value((), terminated(tag("/"), none_of("*"))),
            ))),
            tag("]"),
        )),
        input_to_str,
    )(input)
}

pub fn expr_in_braces(input: &[u8]) -> PResult<&str> {
    map_res(
        recognize(delimited(
            tag("{"),
            many0(alt((
                value((), is_not("{}[]()\"/")),
                value((), expr_in_brackets),
                value((), expr_in_braces),
                value((), expr_in_parens),
                value((), quoted_string),
                value((), rust_comment),
                value((), terminated(tag("/"), none_of("*"))),
            ))),
            tag("}"),
        )),
        input_to_str,
    )(input)
}

pub fn expr_inside_parens(input: &[u8]) -> PResult<&str> {
    map_res(
        recognize(many0(alt((
            value((), is_not("{}[]()\"/")),
            value((), expr_in_braces),
            value((), expr_in_brackets),
            value((), expr_in_parens),
            value((), quoted_string),
            value((), rust_comment),
            value((), terminated(tag("/"), none_of("*"))),
        )))),
        input_to_str,
    )(input)
}

pub fn quoted_string(input: &[u8]) -> PResult<&str> {
    map_res(
        recognize(delimited(
            char('"'),
            opt(escaped(is_not("\"\\"), '\\', one_of("'\"\\nrt0xu"))),
            char('"'),
        )),
        input_to_str,
    )(input)
}

pub fn rust_comment(input: &[u8]) -> PResult<&[u8]> {
    delimited(
        tag("/*"),
        recognize(many0(alt((
            is_not("*"),
            terminated(tag("*"), not(tag("/"))),
        )))),
        tag("*/"),
    )(input)
}

#[cfg(test)]
mod test {
    use super::expression;

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
    fn expression_str_quoted() {
        check_expr("\"line 1\\nline\\t2\"");
    }
    #[test]
    fn expression_str_quoted_unicode() {
        check_expr("\"Snowman: \\u{2603}\"");
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
        assert_eq!(expression(expr.as_bytes()), Ok((&b""[..], expr)));
    }

    #[test]
    fn non_expression_a() {
        assert_eq!(
            expression_error_message(b".foo"),
            ":   1:.foo\n\
             :     ^ Expected rust expression\n"
        );
    }
    #[test]
    fn non_expression_b() {
        assert_eq!(
            expression_error_message(b" foo"),
            ":   1: foo\n\
             :     ^ Expected rust expression\n"
        );
    }
    #[test]
    fn non_expression_c() {
        assert_eq!(
            expression_error_message(b"(missing end"),
            ":   1:(missing end\n\
             :     ^ Expected rust expression\n"
        );
    }
    fn expression_error_message(input: &[u8]) -> String {
        use crate::parseresult::show_errors;
        let mut buf = Vec::new();
        if let Err(error) = expression(input) {
            show_errors(&mut buf, input, &error, ":");
        }
        String::from_utf8(buf).unwrap()
    }
}
