use crate::expression::{
    comma_expressions, expr_in_braces, expr_inside_parens, expression,
    input_to_str, rust_name,
};
use crate::parseresult::PResult;
use crate::spacelike::{comment_tail, spacelike};
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{map, map_res, opt, recognize, value};
use nom::error::context;
use nom::multi::{many0, many_till, separated_list0};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
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
    MatchBlock {
        expr: String,
        arms: Vec<(String, Vec<TemplateExpression>)>,
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
            TemplateArgument::Rust(ref s) => out.write_str(s),
            TemplateArgument::Body(ref v) if v.is_empty() => {
                out.write_str("|_| Ok(())")
            }
            TemplateArgument::Body(ref v) => writeln!(
                out,
                "#[allow(clippy::used_underscore_binding)] |mut _ructe_out_| {{\n{}\nOk(())\n}}",
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
                format!("_ructe_out_.write_all(b{text:?})?;\n")
            }
            TemplateExpression::Text { ref text } => {
                format!("_ructe_out_.write_all({text:?}.as_bytes())?;\n")
            }
            TemplateExpression::Expression { ref expr } => {
                format!("{expr}.to_html(_ructe_out_.by_ref())?;\n")
            }
            TemplateExpression::ForLoop {
                ref name,
                ref expr,
                ref body,
            } => format!(
                "for {name} in {expr} {{\n{}}}\n",
                body.iter().map(|b| b.code()).format(""),
            ),
            TemplateExpression::IfBlock {
                ref expr,
                ref body,
                ref else_body,
            } => format!(
                "if {expr} {{\n{}}}{}\n",
                body.iter().map(|b| b.code()).format(""),
                match else_body.as_deref() {
                    Some([e @ TemplateExpression::IfBlock { .. }]) =>
                        format!(" else {}", e.code()),

                    Some(body) => format!(
                        " else {{\n{}}}",
                        body.iter().map(|b| b.code()).format(""),
                    ),
                    None => String::new(),
                }
            ),
            TemplateExpression::MatchBlock { ref expr, ref arms } => format!(
                "match {expr} {{{}}}\n",
                arms.iter().format_with("", |(expr, body), f| {
                    f(&format_args!(
                        "\n  {} => {{\n{}}}",
                        expr,
                        body.iter().map(|b| b.code()).format(""),
                    ))
                })
            ),
            TemplateExpression::CallTemplate { ref name, ref args } => {
                format!(
                    "{name}(_ructe_out_.by_ref(){})?;\n",
                    args.iter().format_with("", |arg, f| f(&format_args!(
                        ", {arg}"
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
            tag("("),
            terminated(alt((tag("if"), tag("for"), tag("match"))), tag(" ")),
            value(&b""[..], tag("")),
        )),
    ))(input)?
    {
        (i, Some(b":")) => map(
            pair(
                rust_name,
                delimited(
                    char('('),
                    separated_list0(
                        terminated(tag(","), spacelike),
                        template_argument,
                    ),
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
        (i, Some(b"if")) => if2(i),
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
                context("Error in loop block:", template_block),
            )),
            |(name, expr, body)| TemplateExpression::ForLoop {
                name,
                expr,
                body,
            },
        )(i),
        (i, Some(b"match")) => context(
            "Error in match expression:",
            map(
                tuple((
                    delimited(spacelike, expression, spacelike),
                    preceded(
                        char('{'),
                        map(
                            many_till(
                                context(
                                    "Error in match arm starting here:",
                                    pair(
                                        delimited(
                                            spacelike,
                                            map(expression, String::from),
                                            spacelike,
                                        ),
                                        preceded(
                                            terminated(tag("=>"), spacelike),
                                            template_block,
                                        ),
                                    ),
                                ),
                                preceded(spacelike, char('}')),
                            ),
                            |(arms, _end)| arms,
                        ),
                    ),
                )),
                |(expr, arms)| TemplateExpression::MatchBlock {
                    expr: expr.to_string(),
                    arms,
                },
            ),
        )(i),
        (i, Some(b"(")) => {
            map(terminated(expr_inside_parens, tag(")")), |expr| {
                TemplateExpression::Expression {
                    expr: format!("({expr})"),
                }
            })(i)
        }
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

fn if2(input: &[u8]) -> PResult<TemplateExpression> {
    context(
        "Error in conditional expression:",
        map(
            tuple((
                delimited(spacelike, cond_expression, spacelike),
                template_block,
                opt(preceded(
                    delimited(spacelike, tag("else"), spacelike),
                    alt((
                        preceded(tag("if"), map(if2, |e| vec![e])),
                        template_block,
                    )),
                )),
            )),
            |(expr, body, else_body)| TemplateExpression::IfBlock {
                expr,
                body,
                else_body,
            },
        ),
    )(input)
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
                    |(pre, args)| {
                        format!("{}({})", pre.map_or("", |_| "&"), args)
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
            delimited(
                char('{'),
                many0(template_expression),
                terminated(char('}'), spacelike),
            ),
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
            |(lhs, rhs)| format!("let {lhs} = {rhs}"),
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
    fn call_simple() {
        assert_eq!(
            template_expression(b"@foo()"),
            Ok((
                &b""[..],
                TemplateExpression::Expression {
                    expr: "foo()".to_string(),
                },
            ))
        )
    }

    /// Check that issue #53 stays fixed.
    #[test]
    fn call_empty_str() {
        assert_eq!(
            template_expression(b"@foo(\"\")"),
            Ok((
                &b""[..],
                TemplateExpression::Expression {
                    expr: "foo(\"\")".to_string(),
                },
            ))
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

    /// Check that issue #53 stays fixed.
    #[test]
    fn if_compare_empty_string() {
        // Note that x.is_empty() would be better in real code, but this and
        // other uses of empty strings in conditionals should be ok.
        assert_eq!(
            template_expression(b"@if x == \"\" { something }"),
            Ok((
                &b""[..],
                TemplateExpression::IfBlock {
                    expr: "x == \"\"".to_string(),
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
