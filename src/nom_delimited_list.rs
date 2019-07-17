//! This parser combinator is contributed back to nom:
//! https://github.com/Geal/nom/pull/997
use nom::error::ParseError;
use nom::IResult;

/// Alternates between to parsers delimited by two outer parsers to
/// produce a list of elements.
///
/// Similar to `delimited(pre, separated_list(sep, item), term)`, but
/// handles errors differently: Since this parser knows what is
/// supposed to terminate the list, it can report errors from the item
/// parser rather than assuming that the list has ended when item
/// parsing fails.
///
/// The list is allowed to be empty, and a trailing separator (before
/// the terminator) is allowed but not required.
///
/// # Arguments
///
/// * `pre` The opening parser.
/// * `item` Parses the elements of the list.
/// * `sep` Parses the separator between list elements.
/// * `term` The list-terminating parser.
///
/// # Example
///
/// ```
/// # extern crate nom;
/// # extern crate ructe;
/// # use ructe::nom_delimited_list::delimited_list;
/// # use nom::bytes::complete::{is_a, tag};
/// # use nom::error::ErrorKind;
/// # use nom::Err;
/// let parser = delimited_list(tag("("), is_a("abcde"), tag(","), tag(")"));
///
/// assert_eq!(parser("(a,b,c)"), Ok(("", vec!["a", "b", "c"])));
/// assert_eq!(parser("(a,b,c,)"), Ok(("", vec!["a", "b", "c"])));
/// assert_eq!(parser("()"), Ok(("", vec![])));
///
/// // This call returns the error from the terminator parser:
/// assert_eq!(parser("(a!)"), Err(Err::Error(("!)", ErrorKind::Tag))));
/// // This call returns the error from the item parser:
/// assert_eq!(parser("(a,!)"), Err(Err::Error(("!)", ErrorKind::IsA))));
/// ```
pub fn delimited_list<I, OP, OF, OS, OT, E: ParseError<I>, P, F, S, T>(
    pre: P,
    item: F,
    sep: S,
    term: T,
) -> impl Fn(I) -> IResult<I, Vec<OF>, E>
where
    I: Clone + PartialEq,
    P: Fn(I) -> IResult<I, OP, E>,
    F: Fn(I) -> IResult<I, OF, E>,
    S: Fn(I) -> IResult<I, OS, E>,
    T: Fn(I) -> IResult<I, OT, E>,
{
    move |input| {
        let (mut input, _) = pre(input)?;
        let mut list = Vec::new();
        loop {
            let (i, value) = match item(input.clone()) {
                Ok((i, value)) => (i, value),
                Err(item_error) => match term(input) {
                    Ok((i, _)) => return Ok((i, list)),
                    Err(_) => return Err(item_error),
                },
            };
            list.push(value);
            match sep(i.clone()) {
                Ok((i, _)) => input = i,
                Err(_) => {
                    input = i;
                    break;
                }
            }
        }
        let (input, _) = term(input)?;
        Ok((input, list))
    }
}
