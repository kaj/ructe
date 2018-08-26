use expression::{input_to_str, rust_name};
use itertools::Itertools;
use nom::types::CompleteByteSlice as Input;
use spacelike::spacelike;
use std::io::{self, Write};
use templateexpression::{template_expression, TemplateExpression};

#[derive(Debug, PartialEq, Eq)]
pub struct Template {
    preamble: Vec<String>,
    args: Vec<String>,
    body: Vec<TemplateExpression>,
}

impl Template {
    pub fn write_rust(&self, out: &mut Write, name: &str) -> io::Result<()> {
        out.write_all(
            b"use std::io::{self, Write};\n\
             #[cfg_attr(feature=\"cargo-clippy\", \
             allow(useless_attribute))]\n\
             #[allow(unused)]\n\
             use ::templates::{Html,ToHtml};\n",
        )?;
        for l in &self.preamble {
            writeln!(out, "{};", l)?;
        }
        let type_args = if self.args.contains(&"content: Content".to_owned())
        {
            (
                "<Content>",
                "\nwhere Content: FnOnce(&mut Write) -> io::Result<()>",
            )
        } else {
            ("", "")
        };
        writeln!(
            out,
            "\n\
             pub fn {name}{type_args}(out: &mut Write{args})\n\
             -> io::Result<()> {type_spec}{{\n\
             {body}\
             Ok(())\n\
             }}",
            name = name,
            type_args = type_args.0,
            args = self
                .args
                .iter()
                .format_with("", |arg, f| f(&format_args!(", {}", arg))),
            type_spec = type_args.1,
            body = self.body.iter().map(|b| b.code()).format(""),
        )
    }
}

named!(
    pub template<Input, Template>,
    map!(
        tuple!(
            spacelike,
            many0!(map!(
                delimited!(
                    tag!("@"),
                    map_res!(is_not!(";()"), input_to_str),
                    terminated!(tag!(";"), spacelike)
                ),
                String::from
            )),
            delimited!(
                tag!("@("),
                separated_list!(tag!(", "), map!(formal_argument, String::from)),
                terminated!(tag!(")"), spacelike)
            ),
            many_till!(
                return_error!(
                    err_str!("Error in expression starting here:"),
                    template_expression),
                call!(end_of_file))
            ),
        |((), preamble, args, body)| Template { preamble, args, body: body.0 }
    )
);

named!(end_of_file<Input, ()>,
       value!((), eof!()));

named!(formal_argument<Input, &str>,
       map_res!(recognize!(do_parse!(rust_name >> spacelike >>
                            char!(':') >> spacelike >>
                            type_expression >>
                                 ())),
            input_to_str));

named!(type_expression<Input, ()>,
       do_parse!(
           alt!(tag!("&") | tag!("")) >>
           return_error!(err_str!("Expected rust type expression"),
                         alt!(map!(rust_name, |_| ()) |
                              do_parse!(tag!("[") >> type_expression >>
                                        tag!("]") >>
                                        ()) |
                              do_parse!(tag!("(") >> comma_type_expressions >>
                                        tag!(")") >>
                                        ()))) >>
           opt!(do_parse!(tag!("<") >> comma_type_expressions >> tag!(">") >>
                          ())) >>
           ()));

named!(pub comma_type_expressions<Input, ()>,
       map!(separated_list!(tag!(", "), type_expression), |_| ()));
