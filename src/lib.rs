#[macro_use]
extern crate nom;

use nom::{alpha, alphanumeric, multispace, eof};
use nom::IResult::*;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::str::from_utf8;

#[derive(Debug, PartialEq, Eq)]
struct Template {
    preamble: Vec<String>,
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
           preamble: many0!(chain!(tag!("@") ~
                                   code: is_not!(";()") ~
                                   tag!(";") ~
                                   spacelike,
                                   ||from_utf8(code).unwrap().to_string()
                                   )) ~
           tag!("@(") ~
           args: separated_list!(tag!(", "), formal_argument) ~
           tag!(")") ~
           spacelike ~
           body: many0!(template_expression) ~
           eof,
           || { Template { preamble: preamble, args: args, body: body } }
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
               expr: expression,
               || TemplateExpression::Expression{ expr: expr }
           )
       )
);


named!(expression<&[u8], String>,
       alt!(
           chain!(pre: rust_name ~
                  char!('.') ~
                  post: expression,
                  || format!("{}.{}", pre, post)) |
           rust_name
               ));

#[test]
fn test_expression() {
    // Proper expressions, each followed by two non-expression characters.
    for input in &[&b"foo  "[..],
                   &b"foo<x"[..],
                   &b"foo!!"[..],
                   &b"x15  "[..],
                   &b"foo. "[..],
                   &b"foo.bar  "[..],
                   &b"boo.bar.baz##"[..]] {
        let i = input.len() - 2;
        assert_eq!(expression(*input),
                   Done(&input[i..],
                        from_utf8(&input[..i]).unwrap().to_string()));
    }
    // non-expressions
    for input in &[&b".foo"[..], &b" foo"[..], &b"()"[..]] {
        assert_eq!(expression(*input),
                   Error(nom::Err::Position(nom::ErrorKind::Alt, &input[..])));
    }
}

named!(rust_name<&[u8], String>,
       chain!(first: alpha ~
              rest: opt!(alphanumeric),
              || format!("{}{}",
                         from_utf8(first).unwrap(),
                         from_utf8(rest.unwrap_or(b"")).unwrap())));

named!(spacelike<&[u8], ()>,
       chain!(many0!(alt!(
           comment |
           chain!(multispace, ||()))),
              || ()));

named!(comment<&[u8], ()>,
       value!((), delimited!(tag!("@*"),
                             many0!(alt!(
                                 chain!(is_not!("*"), ||()) |
                                 chain!(tag!("*") ~ none_of!("@"), ||())
                                     )),
                             tag!("*@"))));

#[test]
fn test_comment() {
    assert_eq!(comment(b"@* a simple comment *@"), Done(&b""[..], ()));
}
#[test]
fn test_comment2() {
    assert_eq!(comment(b" @* comment *@"),
               Error(nom::Err::Position(nom::ErrorKind::Tag,
                                        &b" @* comment *@"[..])));
}
#[test]
fn test_comment3() {
    assert_eq!(comment(b"@* comment *@ & stuff"), Done(&b" & stuff"[..], ()));
}
#[test]
fn test_comment4() {
    assert_eq!(comment(b"@* comment *@ and @* another *@"),
               Done(&b" and @* another *@"[..], ()));
}
#[test]
fn test_comment5() {
    assert_eq!(comment(b"@* comment containing * and @ *@"),
               Done(&b""[..], ()));
}
#[test]
fn test_comment6() {
    assert_eq!(comment(b"@*** peculiar comment ***@***"),
               Done(&b"***"[..], ()));
}

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
                       "{preamble}\n\
                        pub fn {name}(out: &mut Write{args}) \
                        -> io::Result<()> {{\n\
                        {body}\n\
                        Ok(())\n\
                        }}",
                       preamble = tpl.preamble
                            .iter()
                            .map(|l| format!("{};\n", l))
                            .collect::<String>(),
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
