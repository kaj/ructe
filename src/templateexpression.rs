use expression::{expression, rust_name};
use spacelike::{comment, spacelike};
use std::fmt::{self, Display};
use std::str::from_utf8;

#[derive(Debug, PartialEq, Eq)]
pub enum TemplateExpression {
    Comment,
    Text { text: String },
    Expression { expr: String },
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
                format!("try!(write!(out, {:?}));\n", text)
            }
            TemplateExpression::Expression { ref expr } => {
                format!("try!({}.to_html(out));\n", expr)
            }
            TemplateExpression::ForLoop { ref name, ref expr, ref body } => {
                format!("for {} in {} {{\n{}}}\n",
                        name,
                        expr,
                        body.iter().map(|b| b.code()).collect::<String>())
            }
            TemplateExpression::IfBlock { ref expr,
                                          ref body,
                                          ref else_body } => {
                format!("if {} {{\n{}}}{}\n",
                        expr,
                        body.iter().map(|b| b.code()).collect::<String>(),
                        else_body.iter()
                            .map(|ref b| {
                                     format!(" else {{\n{}}}",
                                             b.iter()
                                                 .map(|b| b.code())
                                                 .collect::<String>())
                                 })
                            .collect::<String>())
            }
            TemplateExpression::CallTemplate { ref name, ref args } => {
                format!("try!({}(out{}));\n",
                        name,
                        args.iter()
                            .map(|b| format!(", {}", b))
                            .collect::<String>())
            }
        }
    }
}
use nom::ErrorKind;

named!(pub template_expression<&[u8], TemplateExpression>,
       add_return_error!(
           ErrorKind::Custom(3),
           switch!(
               opt!(preceded!(tag!("@"),
                              alt!(tag!(":") | tag!("{") | tag!("}") |
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
               Some(b"if") => add_return_error!(
                   ErrorKind::Custom(4),
                   do_parse!(
                   spacelike >>
                   expr: cond_expression >> spacelike >>
                   body: template_block >>
                   else_body: opt!(do_parse!(
                       spacelike >> tag!("else") >> spacelike >>
                       else_body: template_block >>
                       (else_body))) >>
                   (TemplateExpression::IfBlock {
                       expr: expr,
                       body: body,
                       else_body: else_body,
                   }))) |
               Some(b"for") => add_return_error!(
                   ErrorKind::Custom(8),
                   do_parse!(
                   spacelike >>
                   name: rust_name >>
                   spacelike >> tag!("in") >> spacelike >>
                   expr: expression >> spacelike >>
                   body: template_block >> spacelike >>
                   (TemplateExpression::ForLoop {
                       name: name,
                       expr: expr,
                       body: body,
                   }))) |
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
       add_return_error!(
           ErrorKind::Custom(9),
           do_parse!(
               tag!("{") >>
               spacelike >>
               body: return_error!(
                   ErrorKind::Custom(11),
                   many_till!(template_expression, block_end)) >>
               (body.0))));

named!(block_end<&[u8], ()>,
       value!((), tag!("}")));

named!(template_argument<&[u8], TemplateArgument>,
       alt!(map!(delimited!(tag!("{"), many0!(template_expression), tag!("}")),
                 |body| TemplateArgument::Body(body)) |
            map!(expression, |expr| TemplateArgument::Rust(expr))));

named!(cond_expression<&[u8], String>,
       add_return_error!(
           ErrorKind::Custom(7),
           alt!(do_parse!(tag!("let") >> spacelike >>
                          lhs: expression >>
                          spacelike >> char!('=') >> spacelike >>
                          rhs: expression >>
                          (format!("let {} = {}", lhs, rhs))) |
                expression)));

#[cfg(test)]
mod test {
    use super::template_expression;
    use nom::ErrorKind;
    use nom::IResult::Error;
    use nom::verbose_errors::Err;

    #[test]
    fn if_missing_conditional() {
        let t = b"@if { oops }";
        assert_eq!(template_expression(t),
                   Error(Err::NodePosition(
                       ErrorKind::Custom(3), &t[..],
                       Box::new(Err::NodePosition(
                           ErrorKind::Switch, &t[..],
                           Box::new(Err::NodePosition(
                               ErrorKind::Custom(7), &t[4..],
                               Box::new(Err::Position(
                                   ErrorKind::Alt, &t[4..])))))))))
    }
}
