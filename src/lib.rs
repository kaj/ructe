#[macro_use]
extern crate nom;

use nom::{alphanumeric, multispace, eof};
use nom::IResult::*;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::str::from_utf8;

#[derive(Debug, PartialEq, Eq)]
struct Template {
    args: Vec<String>,
    body: Vec<TemplateExpression>,
}

#[derive(Debug, PartialEq, Eq)]
enum TemplateExpression {
    Comment,
    Text { text: String },
    Expression { expr: String },
}

impl TemplateExpression {
    fn code(&self) -> String {
        match *self {
            TemplateExpression::Comment => String::new(),
            TemplateExpression::Text { ref text } => {
                format!("try!(write!(out, \"{}\"));\n", text)
            }
            TemplateExpression::Expression { ref expr } => {
                format!("try!(out.write_all(&try!(encode_html(&{}))));\n", expr)
            }
        }
    }
}

named!(template<&[u8], Template>,
       chain!(
           spacelike ~
           tag!("@(") ~
           args: separated_list!(tag!(", "), formal_argument) ~
           tag!(")") ~
           spacelike ~
           body: many0!(template_expression) ~
           eof,
           || { Template { args: args, body: body } }
           )
);

// TODO Actually parse arguments!
named!(formal_argument<&[u8], String>,
       chain!(
           raw: is_not!(",)"),
           || from_utf8(raw).unwrap().to_string()
               )
       );

named!(template_expression<&[u8], TemplateExpression>,
       alt!(
           chain!(
               comment,
               || TemplateExpression::Comment
               ) |
           chain!(
               text: is_not!("@"),
               || TemplateExpression::Text {
                   text: from_utf8(text).unwrap().to_string()
               }) |
           chain!(
               tag!("@") ~
               expr: alphanumeric,
               || TemplateExpression::Expression {
                   expr: from_utf8(expr).unwrap().to_string()
               }
           )
       )
);

named!(spacelike<&[u8], ()>,
       chain!(many0!(alt!(
           comment |
           chain!(multispace, ||()))),
              || ()));

named!(comment<&[u8], ()>,
       chain!(tag!("@*") ~
              is_not!("*") ~
              tag!("*@"),
              || ()));

pub fn compile_templates(indir: &Path,
                         outdir: &Path,
                         names: &[&str])
                         -> io::Result<()> {
    File::create(outdir.join("templates.rs")).and_then(|mut f| {
        try!(write!(f, "mod templates {{\n\
                        use std::io::{{self, Write}};\n\
                        use std::fmt::Display;\n"));
        for name in names {
            let path = indir.join(format!("{}.rs.html", name));
            let mut input = try!(File::open(&path));
            let mut buf = Vec::new();
            try!(input.read_to_end(&mut buf));
            let tpl = match template(&buf) {
                Done(_, t) => t,
                Error(err) => {
                    panic!("Template parse error in {:?}: {}", path, err)
                }
                Incomplete(needed) => {
                    panic!("Failed to parse template {:?}: {:?} needed",
                               path, needed)
                }
            };
            try!(write!(f,
                       "pub fn {name}(out: &mut Write{args}) \
                        -> io::Result<()> {{\n\
                        {body}\n\
                        Ok(())\n\
                        }}",
                       name = name,
                       args = tpl.args
                            .iter()
                            .map(|a| format!(", {}", a))
                            .collect::<String>(),
                       body = tpl.body
                            .iter()
                            .map(|b| b.code())
                            .collect::<String>()));
        }
        write!(f, "fn encode_html(arg: &Display) \
                       -> io::Result<Vec<u8>> {{\n\
                       let mut buf = Vec::new();\n\
                       try!(write!(buf, \"{{}}\", arg));\n\
                       Ok(buf.into_iter().fold(Vec::new(), |mut v, c| {{\n\
                       match c {{\n\
                       b'<' => v.extend_from_slice(b\"&lt;\"),\n\
                       b'>' => v.extend_from_slice(b\"&gt;\"),\n\
                       b'&' => v.extend_from_slice(b\"&amp;\"),\n\
                       c => v.push(c),\n\
                       }};\n\
                       v\n\
                       }}))\n\
                       }}\n\
                       }}\n")
    })
}
