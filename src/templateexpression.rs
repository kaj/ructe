use expression::{
    comma_expressions, expr_in_braces, expression, input_to_str, rust_name,
};
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{map, map_res, opt, recognize, value};
use nom::error::context;
use nom::multi::{many0, many_till, separated_list};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use parseresult::PResult;
use spacelike::{comment_tail, spacelike};
use std::fmt::{self, Display};

#[derive(Debug, PartialEq, Eq)]
pub enum TemplateExpression {
    Comment,
    Text {
        text: String,
    },
    Expression {
        expr: String,
    },
    ForLoop {
        name: String,
        expr: String,
        body: Vec<TemplateExpression>,
    },
    IfBlock {
        expr: String,
        body: Vec<TemplateExpression>,
        else_body: Option<Vec<TemplateExpression>>,
    },
    CallTemplate {
        name: String,
        args: Vec<TemplateArgument>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum TemplateArgument {
    Rust(String),
    Body(Vec<TemplateExpression>),
}

impl Display for TemplateArgument {
    fn fmt(&self, out: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            TemplateArgument::Rust(ref s) => out.write_str(&s),
            TemplateArgument::Body(ref v) if v.is_empty() => {
                out.write_str("|_| Ok(())")
            }
            TemplateArgument::Body(ref v) => writeln!(
                out,
                "|mut out| {{\n{}\nOk(())\n}}",
                v.iter().map(|b| b.code()).format(""),
            ),
        }
    }
}

impl TemplateExpression {
    pub fn text(text: &str) -> Self {
        TemplateExpression::Text {
            text: text.to_string(),
        }
    }
    pub fn code(&self) -> String {
        match *self {
            TemplateExpression::Comment => String::new(),
            TemplateExpression::Text { ref text } if text.is_ascii() => {
                format!("out.write_all(b{:?})?;\n", text)
            }
            TemplateExpression::Text { ref text } => {
                format!("out.write_all({:?}.as_bytes())?;\n", text)
            }
            TemplateExpression::Expression { ref expr } => {
                format!("{}.to_html(&mut out)?;\n", expr)
            }
            TemplateExpression::ForLoop {
                ref name,
                ref expr,
                ref body,
            } => format!(
                "for {} in {} {{\n{}}}\n",
                name,
                expr,
                body.iter().map(|b| b.code()).format(""),
            ),
            TemplateExpression::IfBlock {
                ref expr,
                ref body,
                ref else_body,
            } => format!(
                "if {} {{\n{}}}{}\n",
                expr,
                body.iter().map(|b| b.code()).format(""),
                else_body.iter().format_with("", |body, f| f(&format_args!(
                    " else {{\n{}}}",
                    body.iter().map(|b| b.code()).format(""),
                ))),
            ),
            TemplateExpression::CallTemplate { ref name, ref args } => {
                format!(
                    "{}(&mut out{})?;\n",
                    name,
                    args.iter().format_with("", |arg, f| f(&format_args!(
                        ", {}",
                        arg
                    ))),
                )
            }
        }
    }
}

pub fn template_expression(input: &[u8]) -> PResult<TemplateExpression> {
    match opt(preceded(
        char('@'),
        alt((
            tag("*"),
            tag(":"),
            tag("@"),
            tag("{"),
            tag("}"),
            terminated(alt((tag("if"), tag("for"))), tag(" ")),
            value(&b""[..], tag("")),
        )),
    ))(input)?
    {
        (i, Some(b":")) => map(
            pair(
                rust_name,
                delimited(
                    char('('),
                    separated_list(tag(", "), template_argument),
                    char(')'),
                ),
            ),
            |(name, args)| TemplateExpression::CallTemplate {
                name: name.to_string(),
                args,
            },
        )(i),
        (i, Some(b"@")) => Ok((i, TemplateExpression::text("@"))),
        (i, Some(b"{")) => Ok((i, TemplateExpression::text("{"))),
        (i, Some(b"}")) => Ok((i, TemplateExpression::text("}"))),
        (i, Some(b"*")) => {
            map(comment_tail, |()| TemplateExpression::Comment)(i)
        }
        (i, Some(b"if")) => context(
            "Error in conditional expression:",
            map(
                tuple((
                    delimited(spacelike, cond_expression, spacelike),
                    template_block,
                    opt(preceded(
                        delimited(spacelike, tag("else"), spacelike),
                        template_block,
                    )),
                )),
                |(expr, body, else_body)| TemplateExpression::IfBlock {
                    expr,
                    body,
                    else_body,
                },
            ),
        )(i),
        (i, Some(b"for")) => map(
            tuple((
                for_variable,
                delimited(
                    terminated(
                        context("Expected \"in\"", tag("in")),
                        spacelike,
                    ),
                    context("Expected iterable expression", loop_expression),
                    spacelike,
                ),
                terminated(
                    context("Error in loop block:", template_block),
                    spacelike,
                ),
            )),
            |(name, expr, body)| TemplateExpression::ForLoop {
                name,
                expr: expr.to_string(),
                body,
            },
        )(i),
        (i, Some(b"")) => {
            map(expression, |expr| TemplateExpression::Expression {
                expr: expr.to_string(),
            })(i)
        }
        (_i, Some(_)) => unreachable!(),
        (i, None) => map(map_res(is_not("@{}"), input_to_str), |text| {
            TemplateExpression::Text {
                text: text.to_string(),
            }
        })(i),
    }
}

fn for_variable(input: &[u8]) -> PResult<String> {
    delimited(
        spacelike,
        context(
            "Expected loop variable name or destructuring tuple",
            alt((
                map(
                    map_res(
                        recognize(preceded(rust_name, opt(expr_in_braces))),
                        input_to_str,
                    ),
                    String::from,
                ),
                map(
                    pair(
                        opt(char('&')),
                        delimited(char('('), comma_expressions, char(')')),
                    ),
                    |(pre, args)| match pre {
                        Some(_) => format!("&({})", args),
                        None => format!("({})", args),
                    },
                ),
            )),
        ),
        spacelike,
    )(input)
}

fn template_block(input: &[u8]) -> PResult<Vec<TemplateExpression>> {
    preceded(
        char('{'),
        map(
            many_till(
                context(
                    "Error in expression starting here:",
                    template_expression,
                ),
                char('}'),
            ),
            |(block, _end)| block,
        ),
    )(input)
}

fn template_argument(input: &[u8]) -> PResult<TemplateArgument> {
    alt((
        map(
            delimited(char('{'), many0(template_expression), char('}')),
            TemplateArgument::Body,
        ),
        map(map(expression, String::from), TemplateArgument::Rust),
    ))(input)
}

fn cond_expression(input: &[u8]) -> PResult<String> {
    match opt(tag("let"))(input)? {
        (i, Some(b"let")) => map(
            pair(
                preceded(
                    spacelike,
                    context(
                        "Expected LHS expression in let binding",
                        expression,
                    ),
                ),
                preceded(
                    delimited(spacelike, char('='), spacelike),
                    context(
                        "Expected RHS expression in let binding",
                        expression,
                    ),
                ),
            ),
            |(lhs, rhs)| format!("let {} = {}", lhs, rhs),
        )(i),
        (_i, Some(_)) => unreachable!(),
        (i, None) => map(
            context("Expected expression", logic_expression),
            String::from,
        )(i),
    }
}

fn loop_expression(input: &[u8]) -> PResult<String> {
    map(
        map_res(
            recognize(terminated(
                expression,
                opt(preceded(
                    terminated(tag(".."), opt(char('='))),
                    expression,
                )),
            )),
            input_to_str,
        ),
        String::from,
    )(input)
}

fn logic_expression(input: &[u8]) -> PResult<&str> {
    map_res(
        recognize(tuple((
            opt(terminated(char('!'), spacelike)),
            expression,
            opt(pair(
                rel_operator,
                context("Expected expression", logic_expression),
            )),
        ))),
        input_to_str,
    )(input)
}

fn rel_operator(input: &[u8]) -> PResult<&str> {
    map_res(
        delimited(
            spacelike,
            context(
                "Expected relational operator",
                alt((
                    tag("!="),
                    tag("&&"),
                    tag("<="),
                    tag("<"),
                    tag("=="),
                    tag(">="),
                    tag(">"),
                    tag("||"),
                )),
            ),
            spacelike,
        ),
        input_to_str,
    )(input)
}

#[cfg(test)]
mod test {
    use super::super::parseresult::show_errors;
    use super::*;

    #[test]
    fn for_variable_simple() {
        assert_eq!(
            for_variable(b"foo").unwrap(),
            (&b""[..], "foo".to_string())
        )
    }

    #[test]
    fn for_variable_tuple() {
        assert_eq!(
            for_variable(b"(foo, bar)").unwrap(),
            (&b""[..], "(foo, bar)".to_string())
        )
    }

    #[test]
    fn for_variable_tuple_ref() {
        assert_eq!(
            for_variable(b"&(foo, bar)").unwrap(),
            (&b""[..], "&(foo, bar)".to_string())
        )
    }

    #[test]
    fn for_variable_struct() {
        assert_eq!(
            for_variable(b"MyStruct{foo, bar}").unwrap(),
            (&b""[..], "MyStruct{foo, bar}".to_string())
        )
    }

    #[test]
    fn if_boolean_var() {
        assert_eq!(
            template_expression(b"@if cond { something }"),
            Ok((
                &b""[..],
                TemplateExpression::IfBlock {
                    expr: "cond".to_string(),
                    body: vec![TemplateExpression::text(" something ")],
                    else_body: None,
                }
            ))
        )
    }

    #[test]
    fn if_let() {
        assert_eq!(
            template_expression(b"@if let Some(x) = x { something }"),
            Ok((
                &b""[..],
                TemplateExpression::IfBlock {
                    expr: "let Some(x) = x".to_string(),
                    body: vec![TemplateExpression::text(" something ")],
                    else_body: None,
                }
            ))
        )
    }

    #[test]
    fn if_let_2() {
        assert_eq!(
            template_expression(b"@if let Some((x, y)) = x { something }"),
            Ok((
                &b""[..],
                TemplateExpression::IfBlock {
                    expr: "let Some((x, y)) = x".to_string(),
                    body: vec![TemplateExpression::text(" something ")],
                    else_body: None,
                }
            ))
        )
    }

    #[test]
    fn if_let_3() {
        assert_eq!(
            template_expression(
                b"@if let Some(p) = Uri::borrow_from(&state) { something }"
            ),
            Ok((
                &b""[..],
                TemplateExpression::IfBlock {
                    expr: "let Some(p) = Uri::borrow_from(&state)"
                        .to_string(),
                    body: vec![TemplateExpression::text(" something ")],
                    else_body: None,
                }
            ))
        )
    }

    #[test]
    fn if_let_struct() {
        assert_eq!(
            template_expression(
                b"@if let Struct{x, y} = variable { something }"
            ),
            Ok((
                &b""[..],
                TemplateExpression::IfBlock {
                    expr: "let Struct{x, y} = variable".to_string(),
                    body: vec![TemplateExpression::text(" something ")],
                    else_body: None,
                }
            ))
        )
    }

    #[test]
    fn if_compare() {
        assert_eq!(
            template_expression(b"@if x == 17 { something }"),
            Ok((
                &b""[..],
                TemplateExpression::IfBlock {
                    expr: "x == 17".to_string(),
                    body: vec![TemplateExpression::text(" something ")],
                    else_body: None,
                }
            ))
        )
    }

    #[test]
    fn if_complex_logig() {
        assert_eq!(
            template_expression(b"@if x == 17 || y && z() { something }"),
            Ok((
                &b""[..],
                TemplateExpression::IfBlock {
                    expr: "x == 17 || y && z()".to_string(),
                    body: vec![TemplateExpression::text(" something ")],
                    else_body: None,
                }
            ))
        )
    }
    #[test]
    fn if_missing_conditional() {
        assert_eq!(
            expression_error(b"@if { oops }"),
            ":   1:@if { oops }\n\
             :         ^ Error in conditional expression:\n\
             :   1:@if { oops }\n\
             :         ^ Expected expression\n\
             :   1:@if { oops }\n\
             :         ^ Expected rust expression\n"
        )
    }

    #[test]
    fn if_bad_let() {
        assert_eq!(
            expression_error(b"@if let foo { oops }"),
            ":   1:@if let foo { oops }\n\
             :         ^ Error in conditional expression:\n\
             :   1:@if let foo { oops }\n\
             :                 ^ Expected \'=\'\n"
        )
    }

    #[test]
    fn for_in_struct() {
        assert_eq!(
            template_expression(
                b"@for Struct{x, y} in structs { something }"
            ),
            Ok((
                &b""[..],
                TemplateExpression::ForLoop {
                    name: "Struct{x, y}".to_string(),
                    expr: "structs".to_string(),
                    body: vec![TemplateExpression::text(" something ")],
                }
            ))
        )
    }

    #[test]
    fn for_missing_in() {
        // TODO The second part of this message isn't really helpful.
        assert_eq!(
            expression_error(b"@for what ever { hello }"),
            ":   1:@for what ever { hello }\n\
             :               ^ Expected \"in\"\n"
        )
    }

    fn expression_error(input: &[u8]) -> String {
        let mut buf = Vec::new();
        if let Err(error) = template_expression(input) {
            show_errors(&mut buf, input, &error, ":");
        }
        String::from_utf8(buf).unwrap()
    }
}
