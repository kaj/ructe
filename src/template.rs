use crate::expression::{input_to_str, rust_name};
use crate::parseresult::PResult;
use crate::spacelike::spacelike;
use crate::templateexpression::{template_expression, TemplateExpression};
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::{char, multispace0};
use nom::combinator::{map, map_res, opt, recognize};
use nom::error::context;
use nom::multi::{many0, many_till, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use std::io::{self, Write};

#[derive(Debug, PartialEq, Eq)]
pub struct Template {
    preamble: Vec<String>,
    type_args: String,
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
             pub fn {name}<{ta}{ta_sep}W>(_ructe_out_: &mut W{args}) -> io::Result<()>\n\
             where W: Write {{\n\
             {body}\
             Ok(())\n\
             }}",
            name = name,
            ta = self.type_args,
            ta_sep = if self.type_args.is_empty() { "" } else { ", " },
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
            context("expected '@('...')' template declaration.", tag("@")),
            opt(delimited(
                terminated(tag("<"), multispace0),
                context(
                    "expected type argument or '>'",
                    map_res(
                        recognize(separated_list1(
                            terminated(tag(","), multispace0),
                            context(
                                "expected lifetime declaration",
                                preceded(tag("'"), rust_name),
                            ),
                        )),
                        input_to_str,
                    ),
                ),
                tag(">"),
            )),
            delimited(
                context(
                    "expected '('...')' template arguments declaration.",
                    terminated(tag("("), multispace0),
                ),
                separated_list0(
                    terminated(tag(","), multispace0),
                    context(
                        "expected formal argument",
                        map(formal_argument, String::from),
                    ),
                ),
                context(
                    "expected ',' or ')'.",
                    delimited(multispace0, tag(")"), spacelike),
                ),
            ),
            many_till(
                context(
                    "Error in expression starting here:",
                    template_expression,
                ),
                end_of_file,
            ),
        )),
        |((), preamble, _, type_args, args, body)| Template {
            preamble,
            type_args: type_args.map(String::from).unwrap_or_default(),
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
            opt(delimited(spacelike, tag("'"), rust_name)),
            delimited(
                spacelike,
                alt((tag("impl"), tag("dyn"), tag(""))),
                spacelike,
            ),
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
    map(
        terminated(
            separated_list0(preceded(tag(","), multispace0), type_expression),
            opt(preceded(tag(","), multispace0)),
        ),
        |_| (),
    )(input)
}

#[cfg(test)]
mod test {
    use super::type_expression;

    #[test]
    fn tuple() {
        check_type_expr("(Foo, Bar)");
    }

    #[test]
    fn unspaced_tuple() {
        check_type_expr("(Foo,Bar)");
    }

    #[test]
    fn tuple_with_trailing() {
        check_type_expr("(Foo,Bar,)");
    }

    #[test]
    fn generic() {
        check_type_expr("HashMap<Foo, Bar>");
    }

    #[test]
    fn unspaced_generic() {
        check_type_expr("HashMap<Foo,Bar>");
    }

    #[test]
    fn generic_with_trailing() {
        check_type_expr("Vec<Foo,>");
    }

    fn check_type_expr(expr: &str) {
        assert_eq!(type_expression(expr.as_bytes()), Ok((&b""[..], ())));
    }
}
