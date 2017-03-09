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

named!(pub template_expression<&[u8], TemplateExpression>,
       alt!(
           map!(comment, |()| TemplateExpression::Comment) |
           do_parse!(
               tag!("@:") >>
               name: rust_name >>
               args: delimited!(tag!("("),
                                separated_list!(tag!(", "), template_argument),
                                tag!(")")) >>
               (TemplateExpression::CallTemplate {
                   name: name,
                   args: args,
               })) |
           do_parse!(
               tag!("@for") >> spacelike >>
               name: rust_name >>
               spacelike >> tag!("in") >> spacelike >>
               expr: expression >> spacelike >> tag!("{") >> spacelike >>
               body: many0!(template_expression) >>
               spacelike >> tag!("}") >>
               (TemplateExpression::ForLoop {
                   name: name,
                   expr: expr,
                   body: body,
               })) |
           do_parse!(
               tag!("@if") >> spacelike >>
               expr: cond_expression >> spacelike >> tag!("{") >> spacelike >>
               body: many0!(template_expression) >> spacelike >>
               tag!("}") >>
               else_body: opt!(do_parse!(
                   spacelike >> tag!("else") >> spacelike >>
                   tag!("{") >>
                   else_body: many0!(template_expression) >>
                   tag!("}") >>
                   (else_body))) >>
               (TemplateExpression::IfBlock {
                   expr: expr,
                   body: body,
                   else_body: else_body,
               })) |
           map!(tag!("@{"),
                |_| TemplateExpression::Text { text: "{{".to_string() }) |
           map!(tag!("@}"),
                |_| TemplateExpression::Text { text: "}}".to_string() }) |
           map!(is_not!("@{}"),
                |text| TemplateExpression::Text {
                    text: from_utf8(text).unwrap().to_string()
                }) |
           map!(preceded!(tag!("@"), expression),
                |expr| TemplateExpression::Expression{ expr: expr })
       )
);

named!(template_argument<&[u8], TemplateArgument>,
       alt!(map!(delimited!(tag!("{"), many0!(template_expression), tag!("}")),
                 |body| TemplateArgument::Body(body)) |
            map!(expression, |expr| TemplateArgument::Rust(expr))));

named!(cond_expression<&[u8], String>,
       alt!(do_parse!(tag!("let") >> spacelike >>
                      lhs: expression >>
                      spacelike >> char!('=') >> spacelike >>
                      rhs: expression >>
                      (format!("let {} = {}", lhs, rhs))) |
            expression));
