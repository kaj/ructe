use expression::{input_to_str, rust_name};
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{map, map_res, opt, recognize};
use nom::error::context;
use nom::multi::{many0, many_till, separated_list};
use nom::sequence::{delimited, terminated, tuple};
use parseresult::PResult;
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
    pub fn write_rust(
        &self,
        out: &mut impl Write,
        name: &str,
    ) -> io::Result<()> {
        out.write_all(
            b"use std::io::{self, Write};\n\
             #[allow(renamed_and_removed_lints)]\n\
             #[cfg_attr(feature=\"cargo-clippy\", \
             allow(useless_attribute))]\n\
             #[allow(unused)]\n\
             use super::{Html,ToHtml};\n",
        )?;
        for l in &self.preamble {
            writeln!(out, "{};", l)?;
        }
        writeln!(
            out,
            "\n\
             pub fn {name}<W: Write>(mut out: W{args}) -> io::Result<()> {{\n\
             {body}\
             Ok(())\n\
             }}",
            name = name,
            args =
                self.args.iter().format_with("", |arg, f| f(&format_args!(
                    ", {}",
                    arg.replace(
                        " Content",
                        " impl FnOnce(&mut W) -> io::Result<()>"
                    )
                ))),
            body = self.body.iter().map(|b| b.code()).format(""),
        )
    }
}

pub fn template(input: &[u8]) -> PResult<Template> {
    map(
        tuple((
            spacelike,
            many0(map(
                delimited(
                    tag("@"),
                    map_res(is_not(";()"), input_to_str),
                    terminated(tag(";"), spacelike),
                ),
                String::from,
            )),
            delimited(
                tag("@("),
                separated_list(tag(", "), map(formal_argument, String::from)),
                terminated(tag(")"), spacelike),
            ),
            many_till(
                context(
                    "Error in expression starting here:",
                    template_expression,
                ),
                end_of_file,
            ),
        )),
        |((), preamble, args, body)| Template {
            preamble,
            args,
            body: body.0,
        },
    )(input)
}

fn end_of_file(input: &[u8]) -> PResult<()> {
    if input.is_empty() {
        Ok((input, ()))
    } else {
        use nom::error::{VerboseError, VerboseErrorKind};
        Err(nom::Err::Error(VerboseError {
            errors: vec![(input, VerboseErrorKind::Context("end of file"))],
        }))
    }
}

fn formal_argument(input: &[u8]) -> PResult<&str> {
    map_res(
        recognize(tuple((
            rust_name,
            spacelike,
            char(':'),
            spacelike,
            type_expression,
        ))),
        input_to_str,
    )(input)
}

fn type_expression(input: &[u8]) -> PResult<()> {
    map(
        tuple((
            alt((tag("&"), tag(""))),
            context(
                "Expected rust type expression",
                alt((
                    map(rust_name, |_| ()),
                    map(
                        delimited(tag("["), type_expression, tag("]")),
                        |_| (),
                    ),
                    map(
                        delimited(tag("("), comma_type_expressions, tag(")")),
                        |_| (),
                    ),
                )),
            ),
            opt(delimited(tag("<"), comma_type_expressions, tag(">"))),
        )),
        |_| (),
    )(input)
}

pub fn comma_type_expressions(input: &[u8]) -> PResult<()> {
    map(separated_list(tag(", "), type_expression), |_| ())(input)
}
