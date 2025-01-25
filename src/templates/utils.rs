use std::fmt::Display;
use std::io::{self, Write};

/// This trait should be implemented for any value that can be the
/// result of an expression in a template.
///
/// This trait decides how to format the given object as html.
/// There exists a default implementation for any `T: Display` that
/// formats the value using Display and then html-encodes the result.
pub trait ToHtml {
    /// Write self to `out`, which is in html representation.
    fn to_html(&self, out: &mut dyn Write) -> io::Result<()>;

    /// Write the HTML represention of this value to a buffer.
    ///
    /// This can be used for testing, and for short-cutting situations
    /// with complex ownership, since the resulting buffer gets owned
    /// by the caller.
    ///
    /// # Examples
    /// ```ignore
    /// # fn main() -> std::io::Result<()> {
    /// # use ructe::templates;
    /// use templates::ToHtml;
    /// assert_eq!(17.to_buffer()?, "17");
    /// assert_eq!("a < b".to_buffer()?, "a &lt; b");
    /// # Ok(())
    /// # }
    /// ```
    fn to_buffer(&self) -> io::Result<HtmlBuffer> {
        let mut buf = Vec::new();
        self.to_html(&mut buf)?;
        Ok(HtmlBuffer { buf })
    }
}

/// Return type for [`ToHtml::to_buffer`].
///
/// An opaque heap-allocated buffer containing a rendered HTML snippet.
pub struct HtmlBuffer {
    #[doc(hidden)]
    buf: Vec<u8>,
}

impl std::fmt::Debug for HtmlBuffer {
    fn fmt(&self, out: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(out, "HtmlBuffer({:?})", String::from_utf8_lossy(&self.buf))
    }
}

impl ToHtml for HtmlBuffer {
    fn to_html(&self, out: &mut dyn Write) -> io::Result<()> {
        out.write_all(&self.buf)
    }
}

impl AsRef<[u8]> for HtmlBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.buf
    }
}

impl PartialEq<&[u8]> for HtmlBuffer {
    fn eq(&self, other: &&[u8]) -> bool {
        &self.buf == other
    }
}
impl PartialEq<&str> for HtmlBuffer {
    fn eq(&self, other: &&str) -> bool {
        let other: &[u8] = other.as_ref();
        self.buf == other
    }
}

/// Wrapper object for data that should be outputted as raw html
/// (objects that may contain markup).
#[allow(dead_code)]
pub struct Html<T>(pub T);

impl<T: Display> ToHtml for Html<T> {
    #[inline]
    fn to_html(&self, out: &mut dyn Write) -> io::Result<()> {
        write!(out, "{}", self.0)
    }
}

impl<T: Display> ToHtml for T {
    #[inline]
    fn to_html(&self, out: &mut dyn Write) -> io::Result<()> {
        write!(ToHtmlEscapingWriter(out), "{self}")
    }
}

struct ToHtmlEscapingWriter<'a>(&'a mut dyn Write);

impl Write for ToHtmlEscapingWriter<'_> {
    #[inline]
    // This takes advantage of the fact that `write` doesn't have to write everything,
    // and the call will be retried with the rest of the data
    // (it is a part of `write_all`'s loop or similar.)
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        // quickly skip over data that doesn't need escaping
        let n = data
            .iter()
            .take_while(|&&c| {
                c != b'"' && c != b'&' && c != b'\'' && c != b'<' && c != b'>'
            })
            .count();
        if n > 0 {
            self.0.write(&data[0..n])
        } else {
            Self::write_one_byte_escaped(&mut self.0, data)
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl ToHtmlEscapingWriter<'_> {
    #[inline(never)]
    fn write_one_byte_escaped(
        out: &mut impl Write,
        data: &[u8],
    ) -> io::Result<usize> {
        let next = data.first();
        out.write_all(match next {
            Some(b'"') => b"&quot;",
            Some(b'&') => b"&amp;",
            Some(b'<') => b"&lt;",
            Some(b'>') => b"&gt;",
            None => return Ok(0),
            // we know this function is called only for chars that need escaping,
            // so we don't have to handle the "other" case (this one is for `'`)
            _ => b"&#39;",
        })?;
        Ok(1)
    }
}
