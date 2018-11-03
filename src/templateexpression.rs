use expression::{comma_expressions, expression, input_to_str, rust_name};
use itertools::Itertools;
use nom::types::CompleteByteSlice as Input;
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
                "|out| {{\n{}\nOk(())\n}}",
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
                format!("{}.to_html(out)?;\n", expr)
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
                    "{}(out{})?;\n",
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

named!(
    pub template_expression<Input, TemplateExpression>,
    add_return_error!(
        err_str!("In expression starting here"),
        switch!(
            opt!(preceded!(tag!("@"),
                           alt!(tag!("*") | tag!(":") | tag!("@") |
                                tag!("{") | tag!("}") |
                                terminated!(
                                    alt!(tag!("if") | tag!("for")),
                                    tag!(" ")) |
                                value!(Input(&b""[..]))))),
            Some(Input(b":")) => map!(
                pair!(rust_name,
                      delimited!(tag!("("),
                                 separated_list!(tag!(", "), template_argument),
                                 tag!(")"))),
                |(name, args)| TemplateExpression::CallTemplate {
                    name: name.to_string(),
                    args,
                }) |
            Some(Input(b"@")) => value!(TemplateExpression::text("@")) |
            Some(Input(b"{")) => value!(TemplateExpression::text("{")) |
            Some(Input(b"}")) => value!(TemplateExpression::text("}")) |
            Some(Input(b"*")) => map!(comment_tail, |()| TemplateExpression::Comment) |
            Some(Input(b"if")) => return_error!(
                err_str!("Error in conditional expression:"),
                map!(
                    tuple!(
                        delimited!(spacelike, cond_expression, spacelike),
                        template_block,
                        opt!(complete!(preceded!(
                            delimited!(spacelike, tag!("else"), spacelike),
                            template_block
                        )))
                    ),
                    |(expr, body, else_body)| TemplateExpression::IfBlock {
                        expr,
                        body,
                        else_body,
                    })) |
            Some(Input(b"for")) => map!(
                tuple!(
                    delimited!(
                        spacelike,
                        return_error!(
                            err_str!("Expected loop variable name \
                                      or destructuring tuple"),
                            alt!(map!(rust_name, String::from) |
                                 map!(
                                     pair!(
                                         opt!(char!('&')),
                                         delimited!(tag!("("),
                                                    comma_expressions,
                                                    tag!(")"))
                                     ),
                                     |(pre, args)| match pre {
                                         Some(_) => format!("&({})", args),
                                         None => format!("({})", args)
                                     }
                                 ))),
                        spacelike),
                    delimited!(
                        terminated!(return_error!(err_str!("Expected \"in\""),
                                                  tag!("in")),
                                    spacelike),
                        return_error!(err_str!("Expected iterable expression"),
                                      loop_expression),
                        spacelike),
                    terminated!(
                        return_error!(err_str!("Error in loop block:"),
                                      template_block),
                        spacelike)),
                |(name, expr, body)| TemplateExpression::ForLoop {
                    name,
                    expr: expr.to_string(),
                    body,
                }) |
            Some(Input(b"")) => map!(
                expression,
                |expr| TemplateExpression::Expression{ expr: expr.to_string() }
            ) |
            None => alt!(
                map!(map_res!(is_not!("@{}"), input_to_str),
                     |text| TemplateExpression::Text {
                         text: text.to_string()
                     })
            )
    ))
);

named!(template_block<Input, Vec<TemplateExpression>>,
       preceded!(
           return_error!(err_str!("Expected \"{\""), char!('{')),
           map!(
               many_till!(
                   return_error!(
                       err_str!("Error in expression starting here:"),
                       template_expression),
                   char!('}')),
               |(block, _end)| block
)));

named!(template_argument<Input, TemplateArgument>,
       alt!(map!(delimited!(tag!("{"), many0!(template_expression), tag!("}")),
                 TemplateArgument::Body) |
            map!(map!(expression, String::from), TemplateArgument::Rust)));

named!(
    cond_expression<Input, String>,
    switch!(
        opt!(tag!("let")),
        Some(Input(b"let")) => map!(
            pair!(
                preceded!(spacelike,
                          return_error!(
                              err_str!("Expected LHS expression in let binding"),
                              expression)),
                preceded!(
                    delimited!(
                        spacelike,
                        return_error!(err_str!("Expected \"=\""), char!('=')),
                        spacelike),
                    return_error!(
                        err_str!("Expected RHS expression in let binding"),
                        expression))),
            |(lhs, rhs)| format!("let {} = {}", lhs, rhs)) |
        None => map!(
            return_error!(err_str!("Expected expression"), logic_expression),
            String::from
        )
    )
);

named!(
    loop_expression<Input, String>,
    map!(
        map_res!(
            recognize!(
                terminated!(
                    expression,
                    opt!(
                        preceded!(
                            terminated!(tag!(".."), opt!(tag!("="))),
                            expression)))),
            input_to_str),
        String::from)
);

named!(
    logic_expression<Input, &str>,
    map_res!(
        recognize!(tuple!(
            opt!(terminated!(tag!("!"), spacelike)),
            expression,
            opt!(pair!(
                rel_operator,
                return_error!(
                    err_str!("Expected expression"),
                    logic_expression
                )))
        )),
        input_to_str
    )
);

named!(rel_operator<Input, &str>,
       map_res!(
           delimited!(
               spacelike,
               alt!(tag_s!("==") | tag_s!("!=") | tag_s!(">=") |
                    tag_s!(">") | tag_s!("<=") | tag_s!("<") |
                    tag_s!("||") | tag_s!("&&")),
               spacelike),
           input_to_str
       )
);

#[cfg(test)]
mod test {
    use super::super::show_errors;
    use super::*;
    use nom::types::CompleteByteSlice as Input;

    #[test]
    fn if_boolean_var() {
        assert_eq!(
            template_expression(Input(b"@if cond { something }")),
            Ok((
                Input(&b""[..]),
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
            template_expression(Input(b"@if let Some(x) = x { something }")),
            Ok((
                Input(&b""[..]),
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
            template_expression(Input(
                b"@if let Some((x, y)) = x { something }"
            )),
            Ok((
                Input(&b""[..]),
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
            template_expression(Input(
                b"@if let Some(p) = Uri::borrow_from(&state) { something }"
            )),
            Ok((
                Input(&b""[..]),
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
    fn if_compare() {
        assert_eq!(
            template_expression(Input(b"@if x == 17 { something }")),
            Ok((
                Input(&b""[..]),
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
            template_expression(Input(
                b"@if x == 17 || y && z() { something }"
            )),
            Ok((
                Input(&b""[..]),
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
             :         ^ Expected rust expression\n\
             :   1:@if { oops }\n\
             :         ^ Alt\n"
        )
    }

    #[test]
    fn if_bad_let() {
        assert_eq!(
            expression_error(b"@if let foo { oops }"),
            ":   1:@if let foo { oops }\n\
             :         ^ Error in conditional expression:\n\
             :   1:@if let foo { oops }\n\
             :                 ^ Expected \"=\"\n\
             :   1:@if let foo { oops }\n\
             :                 ^ Char\n"
        )
    }

    #[test]
    fn for_missing_in() {
        // TODO The second part of this message isn't really helpful.
        assert_eq!(
            expression_error(b"@for what ever { hello }"),
            ":   1:@for what ever { hello }\n\
             :     ^ In expression starting here\n\
             :   1:@for what ever { hello }\n\
             :               ^ Expected \"in\"\n\
             :   1:@for what ever { hello }\n\
             :               ^ Tag\n"
        )
    }

    fn expression_error(input: &[u8]) -> String {
        let mut buf = Vec::new();
        show_errors(&mut buf, input, template_expression(Input(input)), ":");
        String::from_utf8(buf).unwrap()
    }
}
