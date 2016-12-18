use nom::eof;
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
                use ::templates::{{Html,ToHtml}};\n\
                {preamble}\n\
                pub fn {name}{type_args}(out: &mut Write{args})\n\
                -> io::Result<()> {type_spec}{{\n\
                {body}\
                Ok(())\n\
                }}\n",
               preamble = self.preamble
                   .iter()
                   .map(|l| format!("{};\n", l))
                   .collect::<String>(),
               name = name,
               type_args = self.args
                   .iter()
                   .filter(|a| a.as_str() == "content: Content")
                   .map(|_a| format!("<Content>"))
                   .collect::<String>(),
               args = self.args
                   .iter()
                   .map(|a| format!(", {}", a))
                   .collect::<String>(),
               type_spec = self.args
                   .iter()
                   .filter(|a| a.as_str() == "content: Content")
                   .map(|_a| {
                       format!("\nwhere Content: FnOnce(&mut Write) \
                                -> io::Result<()>")
                   })
                   .collect::<String>(),
               body = self.body
                   .iter()
                   .map(|b| b.code())
                   .collect::<String>())
    }
}

named!(pub template<&[u8], Template>,
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
