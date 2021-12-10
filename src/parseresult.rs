use nom::error::{VerboseError, VerboseErrorKind};
use nom::{Err, IResult};
use std::io::Write;
use std::str::from_utf8;

/// Parser result, with verbose error.
pub type PResult<'a, O> = IResult<&'a [u8], O, VerboseError<&'a [u8]>>;

pub fn show_errors(
    out: &mut impl Write,
    buf: &[u8],
    error: &Err<VerboseError<&[u8]>>,
    prefix: &str,
) {
    match error {
        Err::Failure(VerboseError { ref errors })
        | Err::Error(VerboseError { ref errors }) => {
            for (rest, err) in errors.iter().rev() {
                if let Some(message) = get_message(err) {
                    let pos = buf.len() - rest.len();
                    show_error(out, buf, pos, &message, prefix);
                }
            }
        }
        Err::Incomplete(needed) => {
            let msg = format!("Incomplete: {:?}", needed);
            show_error(out, buf, 0, &msg, prefix);
        }
    }
}

fn get_message(err: &VerboseErrorKind) -> Option<String> {
    match err {
        VerboseErrorKind::Context(msg) => Some((*msg).into()),
        VerboseErrorKind::Char(ch) => Some(format!("Expected {:?}", ch)),
        VerboseErrorKind::Nom(_err) => None,
    }
}

fn show_error(
    out: &mut impl Write,
    buf: &[u8],
    pos: usize,
    msg: &str,
    prefix: &str,
) {
    let mut line_start = buf[0..pos].rsplitn(2, |c| *c == b'\n');
    let _ = line_start.next();
    let line_start = line_start.next().map_or(0, |bytes| bytes.len() + 1);
    let line = buf[line_start..]
        .splitn(2, |c| *c == b'\n')
        .next()
        .and_then(|s| from_utf8(s).ok())
        .unwrap_or("(Failed to display line)");
    let line_no = bytecount::count(&buf[..line_start], b'\n') + 1;
    let pos_in_line =
        from_utf8(&buf[line_start..pos]).unwrap().chars().count() + 1;
    writeln!(
        out,
        "{prefix}{:>4}:{}\n\
         {prefix}     {:>pos$} {}",
        line_no,
        line,
        "^",
        msg,
        pos = pos_in_line,
        prefix = prefix,
    )
    .unwrap();
}
