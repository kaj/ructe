use nom::ErrorKind;
use spacelike::spacelike;
use std::io::{self, Write};
use std::str::from_utf8;
use templateexpression::{TemplateExpression, template_expression};

#[derive(Debug, PartialEq, Eq)]
pub struct Template {
    preamble: Vec<String>,
    args: Vec<String>,
    body: Vec<TemplateExpression>,
}

impl Template {
    pub fn write_rust(&self, out: &mut Write, name: &str) -> io::Result<()> {
        write!(out,
               "use std::io::{{self, Write}};\n\
                #[allow(unused)]\n\
                use ::templates::{{Html,ToHtml}};\n")?;
        for l in &self.preamble {
            write!(out, "{};\n", l)?;
        }
        let type_args = if self.args.contains(&"content: Content".to_owned()) {
            ("<Content>",
             "\nwhere Content: FnOnce(&mut Write) \
              -> io::Result<()>")
        } else {
            ("", "")
        };
        write!(out,
               "\n\
                pub fn {name}{type_args}(out: &mut Write{args})\n\
                -> io::Result<()> {type_spec}{{\n\
                {body}\
                Ok(())\n\
                }}\n",
               name = name,
               type_args = type_args.0,
               args = self.args
            .iter()
            .map(|a| format!(", {}", a))
            .collect::<String>(),
               type_spec = type_args.1,
               body = self.body.iter().map(|b| b.code()).collect::<String>())
    }
}

named!(pub template<&[u8], Template>,
       do_parse!(
           spacelike >>
           preamble: many0!(do_parse!(tag!("@") >>
                                      code: is_not!(";()") >>
                                      tag!(";") >>
                                      spacelike >>
                                      (from_utf8(code).unwrap().to_string())
                                      )) >>
           tag!("@(") >>
           args: separated_list!(tag!(", "), formal_argument) >>
           tag!(")") >>
           spacelike >>
           body: add_return_error!(
               ErrorKind::Custom(1),
               many_till!(template_expression, end)) >>
           (Template { preamble: preamble, args: args, body: body.0 })
           ));

named!(end<&[u8], ()>,
       map!(eof!(), |_| ()));

// TODO Actually parse arguments!
named!(formal_argument<&[u8], String>,
       map!(is_not!(",)"),
            |raw| from_utf8(raw).unwrap().to_string()));
