use expression::{comma_expressions, expression, rust_name};
use spacelike::{comment, spacelike};
use std::fmt::{self, Display};
use std::str::from_utf8;

/// Copied from nom, but fixed for
/// https://github.com/Geal/nom/issues/463
///
/// This should be removed when a fix for that is released from nom.
macro_rules! my_many_till(
  ($i:expr, $submac1:ident!( $($args1:tt)* ), $submac2:ident!( $($args2:tt)* ))
        => (
    {
      use nom::InputLength;
      use nom::ErrorKind;
      use nom::IResult;
      use nom::Needed;

      let ret;
      let mut res   = ::std::vec::Vec::new();
      let mut input = $i;

      loop {
        match $submac2!(input, $($args2)*) {
          IResult::Done(i, o) => {
            ret = IResult::Done(i, (res, o));
            break;
          },
          _                           => {
            match $submac1!(input, $($args1)*) {
              IResult::Error(err)                            => {
                ret = IResult::Error(error_node_position!(ErrorKind::ManyTill,
                                                          input,
                                                          err));
                break;
              },
              IResult::Incomplete(Needed::Unknown) => {
                ret = IResult::Incomplete(Needed::Unknown);
                break;
              },
              IResult::Incomplete(Needed::Size(i)) => {
                let size = i + ($i).input_len() - input.input_len();
                ret = IResult::Incomplete(Needed::Size(size));
                break;
              },
              IResult::Done(i, o)                          => {
                // loop trip must always consume (otherwise infinite loops)
                if i == input {
                  ret = IResult::Error(error_position!(ErrorKind::ManyTill,
                                                       input));
                  break;
                }

                res.push(o);
                input = i;
              },
            }
          },
        }
      }

      ret
    }
  );
  ($i:expr, $f:expr, $g: expr) => (
    my_many_till!($i, call!($f), call!($g));
  );
);

#[derive(Debug, PartialEq, Eq)]
pub enum TemplateExpression {
    Comment,
    Text { text: String },
    Raw { text: String },
    Expression { expr: String },
    ForLoop { name: String, expr: String, body: Vec<TemplateExpression> },
    IfBlock {
        expr: String,
        body: Vec<TemplateExpression>,
        else_body: Option<Vec<TemplateExpression>>,
    },
    CallTemplate { name: String, args: Vec<TemplateArgument> },
}

#[derive(Debug, PartialEq, Eq)]
pub enum TemplateArgument {
    Rust(String),
    Body(Vec<TemplateExpression>),
}

impl Display for TemplateArgument {
    fn fmt(&self, out: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            TemplateArgument::Rust(ref s) => write!(out, "{}", s),
            TemplateArgument::Body(ref v) => {
                write!(out,
                       "|out| {{\n{}\nOk(())\n}}\n",
                       v.iter().map(|b| b.code()).collect::<String>())
            }
        }
    }
}

impl TemplateExpression {
    pub fn code(&self) -> String {
        match *self {
            TemplateExpression::Comment => String::new(),
            TemplateExpression::Text { ref text } => {
                format!("write!(out, {:?})?;\n", text)
            }
            TemplateExpression::Raw { ref text } => text.to_owned(),
            TemplateExpression::Expression { ref expr } => {
                format!("{}.to_html(out)?;\n", expr)
            }
            TemplateExpression::ForLoop { ref name, ref expr, ref body } => {
                format!("for {} in {} {{\n{}}}\n",
                        name,
                        expr,
                        body.iter().map(|b| b.code()).collect::<String>())
            }
            TemplateExpression::IfBlock {
                ref expr,
                ref body,
                ref else_body,
            } => {
                format!("if {} {{\n{}}}{}\n",
                        expr,
                        body.iter().map(|b| b.code()).collect::<String>(),
                        else_body
                            .iter()
                            .map(|b| {
                                     format!(" else {{\n{}}}",
                                             b.iter()
                                                 .map(|b| b.code())
                                                 .collect::<String>())
                                 })
                            .collect::<String>())
            }
            TemplateExpression::CallTemplate { ref name, ref args } => {
                format!("{}(out{})?;\n",
                        name,
                        args.iter()
                            .map(|b| format!(", {}", b))
                            .collect::<String>())
            }
        }
    }
}

named!(pub template_expression<&[u8], TemplateExpression>,
       add_return_error!(
           err_str!("In expression starting here"),
           switch!(
               opt!(preceded!(tag!("@"),
                              alt!(tag!(":") | tag!("{") | tag!("}") | tag!("@") |
                                   terminated!(
                                       alt!(tag!("if") |
                                            tag!("for")),
                                       tag!(" "))))),
               Some(b":") => do_parse!(
                   name: rust_name >>
                   args: delimited!(tag!("("),
                                    separated_list!(tag!(", "),
                                                    template_argument),
                                    tag!(")")) >>
                   (TemplateExpression::CallTemplate {
                       name: name,
                       args: args,
                   })) |
               Some(b"{") => value!(TemplateExpression::Text {
                   text: "{{".to_string()
               }) |
               Some(b"}") => value!(TemplateExpression::Text {
                   text: "}}".to_string()
               }) |
               Some(b"@") => do_parse!(
                   text: take_until_and_consume!("\n") >>
                   (TemplateExpression::Raw { text: from_utf8(text).unwrap().to_string() + "\n" })
               ) |
               Some(b"if") => return_error!(
                   err_str!("Error in conditional expression:"),
                   do_parse!(
                   spacelike >>
                   expr: cond_expression >> spacelike >>
                   body: template_block >>
                   else_body: opt!(complete!(do_parse!(
                       spacelike >> tag!("else") >> spacelike >>
                       else_body: template_block >>
                       (else_body)))) >>
                   (TemplateExpression::IfBlock {
                       expr: expr,
                       body: body,
                       else_body: else_body,
                   }))) |
               Some(b"for") => do_parse!(
                   spacelike >>
                   name: return_error!(
                       err_str!("Expected loop variable name \
                                 or destructuring tuple"),
                       alt!(rust_name |
                            do_parse!(pre: opt!(char!('&')) >>
                                      tag!("(") >>
                                      args: comma_expressions >>
                                      tag!(")") >>
                                      (format!("{}({})",
                                               pre.unwrap_or(' '),
                                               args)))
                            )) >>
                   spacelike >>
                   return_error!(err_str!("Expected \"in\""), tag!("in")) >>
                   spacelike >>
                   expr: return_error!(err_str!("Expected iterable expression"),
                                       expression) >>
                   spacelike >>
                   body: return_error!(err_str!("Error in loop block:"),
                                       template_block) >> spacelike >>
                   (TemplateExpression::ForLoop {
                       name: name,
                       expr: expr,
                       body: body,
                   })) |
               None => alt!(
                   map!(comment, |()| TemplateExpression::Comment) |
                   map!(is_not!("@{}"),
                        |text| TemplateExpression::Text {
                            text: from_utf8(text).unwrap().to_string()
                        }) |
                   map!(preceded!(tag!("@"), expression),
                        |expr| TemplateExpression::Expression{ expr: expr })
                       )))
       );

named!(template_block<&[u8], Vec<TemplateExpression>>,
       do_parse!(return_error!(err_str!("Expected \"{\""), char!('{')) >>
                 body: my_many_till!(
                     return_error!(
                         err_str!("Error in expression starting here:"),
                         template_expression),
                     char!('}')) >>
                 (body.0)));

named!(template_argument<&[u8], TemplateArgument>,
       alt!(map!(delimited!(tag!("{"), many0!(template_expression), tag!("}")),
                 |body| TemplateArgument::Body(body)) |
            map!(expression, |expr| TemplateArgument::Rust(expr))));

named!(cond_expression<&[u8], String>,
       switch!(opt!(tag!("let")),
               Some(b"let") => do_parse!(
                   spacelike >>
                   lhs: return_error!(
                       err_str!("Expected LHS expression in let binding"),
                       expression) >>
                   spacelike >>
                   return_error!(err_str!("Expected \"=\""), char!('=')) >>
                   spacelike >>
                   rhs: return_error!(
                       err_str!("Expected RHS expression in let binding"),
                       expression) >>
                   (format!("let {} = {}", lhs, rhs))) |
               None => do_parse!(
                   a: return_error!(err_str!("Expected expression"),
                                    expression) >>
                   b: opt!(do_parse!(spacelike >>
                                     op: alt!(tag!("==") | tag!("!=") |
                                              tag!(">=") | tag!(">") |
                                              tag!("<=") | tag!("<")) >>
                                     spacelike >>
                                     rhs: expression >>
                                     (from_utf8(op).unwrap(), rhs))) >>
                   (if let Some((op, rhs)) = b {
                       format!("{} {} {}", a, op, rhs)
                   } else {
                       a
                   }))));

#[cfg(test)]
mod test {
    use super::*;
    use super::super::show_errors;
    use nom::IResult;

    #[test]
    fn if_boolean_var() {
        assert_eq!(template_expression(b"@if cond { something }"),
                   IResult::Done(&b""[..],
                                 TemplateExpression::IfBlock {
                                     expr: "cond".to_string(),
                                     body: vec![TemplateExpression::Text {
                                                    text: " something "
                                                        .to_string(),
                                                }],
                                     else_body: None,
                                 }))
    }

    #[test]
    fn if_let() {
        assert_eq!(template_expression(b"@if let Some(x) = x { something }"),
                   IResult::Done(&b""[..],
                                 TemplateExpression::IfBlock {
                                     expr: "let Some(x) = x".to_string(),
                                     body: vec![TemplateExpression::Text {
                                                    text: " something "
                                                        .to_string(),
                                                }],
                                     else_body: None,
                                 }))
    }

    #[test]
    fn if_compare() {
        assert_eq!(template_expression(b"@if x == 17 { something }"),
                   IResult::Done(&b""[..],
                                 TemplateExpression::IfBlock {
                                     expr: "x == 17".to_string(),
                                     body: vec![TemplateExpression::Text {
                                                    text: " something "
                                                        .to_string(),
                                                }],
                                     else_body: None,
                                 }))
    }

    #[test]
    fn if_missing_conditional() {
        assert_eq!(expression_error(b"@if { oops }"),
                   ":   1:@if { oops }\n\
                    :         ^ Error in conditional expression:\n\
                    :   1:@if { oops }\n\
                    :         ^ Expected expression\n\
                    :   1:@if { oops }\n\
                    :         ^ Expected rust expression\n\
                    :   1:@if { oops }\n\
                    :         ^ Alt\n")
    }

    #[test]
    fn if_bad_let() {
        assert_eq!(expression_error(b"@if let foo { oops }"),
                   ":   1:@if let foo { oops }\n\
                    :         ^ Error in conditional expression:\n\
                    :   1:@if let foo { oops }\n\
                    :                 ^ Expected \"=\"\n\
                    :   1:@if let foo { oops }\n\
                    :                 ^ Char\n")
    }

    #[test]
    fn for_missig_in() {
        // TODO The second part of this message isn't really helpful.
        assert_eq!(expression_error(b"@for what ever { hello }"),
                   ":   1:@for what ever { hello }\n\
                    :               ^ Expected \"in\"\n\
                    :   1:@for what ever { hello }\n\
                    :               ^ Tag\n")
    }

    fn expression_error(input: &[u8]) -> String {
        let mut buf = Vec::new();
        show_errors(&mut buf, input, template_expression(input), ":");
        String::from_utf8(buf).unwrap()
    }
}
