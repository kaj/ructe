use crate::expression::{input_to_str, rust_name};
use crate::parseresult::PResult;
use crate::spacelike::spacelike;
use crate::templateexpression::{template_expression, TemplateExpression};
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::{char, multispace0};
use nom::combinator::{map, map_res, opt, recognize, value};
use nom::error::context;
use nom::multi::{many0, many_till, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, terminated};
use nom::Parser as _;
use std::fmt::Write;

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
    ) -> std::fmt::Result {
        out.write_str(
            "use std::io::{self, Write};\n\
             #[allow(clippy::useless_attribute, unused)]\n\
             use super::{Html,ToHtml};\n",
        )?;
        for line in &self.preamble {
            writeln!(out, "{line};")?;
        }
        writeln!(
            out,
            "\n\
             #[allow(clippy::used_underscore_binding)]\n\
             pub fn {name}<{ta}{ta_sep}W>(\
             \n  #[allow(unused_mut)] mut _ructe_out_: W,",
            name = name,
            ta = self.type_args,
            ta_sep = if self.type_args.is_empty() { "" } else { ", " },
        )?;
        for arg in &self.args {
            writeln!(
                out,
                "  {},",
                arg.replace(
                    " Content",
                    " impl FnOnce(&mut W) -> io::Result<()>"
                )
            )?;
        }
        writeln!(
            out,
            ") -> io::Result<()>\n\
             where W: Write {{",
        )?;
        for b in &self.body {
            b.write_code(out)?;
        }
        writeln!(out, "Ok(())\n}}")?;
        Ok(())
    }
}

pub fn template(input: &[u8]) -> PResult<Template> {
    map(
        (
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
        ),
        |((), preamble, _, type_args, args, body)| Template {
            preamble,
            type_args: type_args.map(String::from).unwrap_or_default(),
            args,
            body: body.0,
        },
    )
    .parse(input)
}

fn end_of_file(input: &[u8]) -> PResult<()> {
    if input.is_empty() {
        Ok((input, ()))
    } else {
        use nom_language::error::{VerboseError, VerboseErrorKind};
        Err(nom::Err::Error(VerboseError {
            errors: vec![(input, VerboseErrorKind::Context("end of file"))],
        }))
    }
}

fn formal_argument(input: &[u8]) -> PResult<&str> {
    map_res(
        recognize((
            rust_name,
            spacelike,
            char(':'),
            spacelike,
            type_expression,
        )),
        input_to_str,
    )
    .parse(input)
}

fn type_expression(input: &[u8]) -> PResult<()> {
    value(
        (),
        (
            alt((tag("&"), tag(""))),
            opt(lifetime),
            delimited(
                spacelike,
                alt((tag("impl"), tag("dyn"), tag(""))),
                spacelike,
            ),
            context(
                "Expected rust type expression",
                alt((
                    value((), rust_name),
                    delimited(tag("["), value((), type_expression), tag("]")),
                    delimited(
                        tag("("),
                        value((), comma_type_expressions),
                        tag(")"),
                    ),
                )),
            ),
            opt(delimited(tag("<"), comma_type_expressions, tag(">"))),
        ),
    )
    .parse(input)
}

pub fn comma_type_expressions(input: &[u8]) -> PResult<()> {
    value(
        (),
        terminated(
            separated_list0(
                preceded(tag(","), multispace0),
                alt((type_expression, lifetime)),
            ),
            opt(preceded(tag(","), multispace0)),
        ),
    )
    .parse(input)
}

fn lifetime(input: &[u8]) -> PResult<()> {
    delimited(spacelike, value((), tag("'")), rust_name).parse(input)
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

    #[test]
    fn generic_with_lifetime() {
        check_type_expr("SomeTypeWithRef<'a>");
    }

    #[test]
    fn generic_with_anonymous_lifetime() {
        check_type_expr("SomeTypeWithRef<'_>");
    }

    #[test]
    fn multiword_constant() {
        check_type_expr("ONE_TWO_THREE");
    }

    fn check_type_expr(expr: &str) {
        assert_eq!(type_expression(expr.as_bytes()), Ok((&b""[..], ())));
    }
}
