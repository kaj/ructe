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

impl<'a> Write for ToHtmlEscapingWriter<'a> {
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

impl<'a> ToHtmlEscapingWriter<'a> {
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

/// Adapter interface providing `join_html` method.
pub trait JoinHtml<I: Iterator> {
    /// Format the items of the given iterator, separated by `sep`.
    ///
    /// The formatting is done by a given template (or template-like function).
    ///
    /// # Examples
    ///
    /// ```
    /// use ructe::templates::{JoinHtml, Html};
    /// # fn main() -> std::io::Result<()> {
    /// assert_eq!(
    ///     [("Rasmus", "kaj"), ("Kalle", "karl")]
    ///         .iter()
    ///         .join_html(
    ///             |o, (name, user)| {
    ///                 write!(o, "<a href=\"/profile/{}\">{}</a>", user, name)
    ///             },
    ///             Html("<br/>\n"),
    ///         )
    ///         .to_buffer()?,
    ///     "<a href=\"/profile/kaj\">Rasmus</a><br/>\
    ///      \n<a href=\"/profile/karl\">Kalle</a>"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Note that the callback function is responsible for any html
    /// escaping of the argument.
    /// The closure with the write function above don't do any
    /// escaping, it worked fine only because the names and user-names
    /// in the example did not contain any characters requireing escaping.
    ///
    /// One nice way to get a function that handles escaping is to use
    /// a template function as the formatting callback.
    ///
    /// If the the following template is `link.rs.html`:
    /// ```ructe
    /// @((title, slug): &(&str, &str))
    /// <a href=\"/album/@slug\">@title</a>
    /// ```
    ///
    /// It can be used like this in rust code:
    /// ```
    /// # // Mock the above template
    /// # use std::io;
    /// # use ructe::templates::ToHtml;
    /// # fn link(o: &mut dyn io::Write, (title, slug): &(&str, &str)) -> io::Result<()> {
    /// #     o.write_all(b"<a href=\"/album/")?;
    /// #     slug.to_html(o)?;
    /// #     o.write_all(b"\">")?;
    /// #     title.to_html(o)?;
    /// #     o.write_all(b"</a>")
    /// # }
    /// use ructe::templates::{Html, JoinHtml};
    /// # fn main() -> std::io::Result<()> {
    /// assert_eq!(
    ///     [("Spirou & Fantasio", "spirou"), ("Tom & Jerry", "tom_jerry")]
    ///         .iter()
    ///         .join_html(link, Html("<br/>\n"))
    ///         .to_buffer()?,
    ///     "<a href=\"/album/spirou\">Spirou &amp; Fantasio</a><br/>\
    ///      \n<a href=\"/album/tom_jerry\">Tom &amp; Jerry</a>"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Or like this in a template, giving similar result:
    /// ```ructe
    /// @use super::{link, Html, JoinHtml};
    ///
    /// @(comics: &[(&str, &str)])
    /// <div class="containing markup">
    ///   @comics.iter().to_html(link, Html("<br/>"))
    /// </div>
    /// ```
    fn join_html<
        F: 'static + Fn(&mut dyn Write, I::Item) -> io::Result<()>,
        Sep: 'static + ToHtml,
    >(
        self,
        item_template: F,
        sep: Sep,
    ) -> Box<dyn ToHtml>;
}

/// Adapter interface providing `join_to_html` method.
pub trait JoinToHtml<Item: ToHtml, I: Iterator<Item = Item>> {
    /// Format the items of the given iterator, separated by `sep`.
    ///
    /// # Example
    ///
    /// ```
    /// use ructe::templates::JoinToHtml;
    /// # fn main() -> std::io::Result<()> {
    /// assert_eq!(
    ///     ["foo", "b<a", "baz"]
    ///         .iter()
    ///         .join_to_html(" & ")
    ///         .to_buffer()?,
    ///     "foo &amp; b&lt;a &amp; baz"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn join_to_html<Sep: 'static + ToHtml>(self, sep: Sep)
        -> Box<dyn ToHtml>;
}

impl<I: 'static + Iterator + Clone> JoinHtml<I> for I {
    fn join_html<
        F: 'static + Fn(&mut dyn Write, I::Item) -> io::Result<()>,
        Sep: 'static + ToHtml,
    >(
        self,
        item_template: F,
        sep: Sep,
    ) -> Box<dyn ToHtml> {
        Box::new(HtmlJoiner {
            items: self,
            f: item_template,
            sep,
        })
    }
}

impl<Item: ToHtml, Iter: 'static + Iterator<Item = Item> + Clone>
    JoinToHtml<Item, Iter> for Iter
{
    fn join_to_html<Sep: 'static + ToHtml>(
        self,
        sep: Sep,
    ) -> Box<dyn ToHtml> {
        Box::new(HtmlJoiner {
            items: self,
            f: |o, i| i.to_html(o),
            sep,
        })
    }
}

struct HtmlJoiner<
    Items: Iterator + Clone,
    F: Fn(&mut dyn Write, Items::Item) -> io::Result<()>,
    Sep: ToHtml,
> {
    items: Items,
    f: F,
    sep: Sep,
}

impl<
        Items: Iterator + Clone,
        F: Fn(&mut dyn Write, Items::Item) -> io::Result<()>,
        Sep: ToHtml,
    > ToHtml for HtmlJoiner<Items, F, Sep>
{
    fn to_html(&self, out: &mut dyn Write) -> io::Result<()> {
        let mut iter = self.items.clone();
        if let Some(first) = iter.next() {
            (self.f)(out, first)?;
        } else {
            return Ok(());
        }
        for item in iter {
            self.sep.to_html(out)?;
            (self.f)(out, item)?;
        }
        Ok(())
    }
}

#[test]
fn test_join_to_html() {
    assert_eq!(
        ["foo", "b<a", "baz"]
            .iter()
            .join_to_html(", ")
            .to_buffer()
            .unwrap(),
        "foo, b&lt;a, baz"
    )
}

#[test]
fn test_join_to_html_empty() {
    use std::iter::empty;
    assert_eq!(empty::<&str>().join_to_html(", ").to_buffer().unwrap(), "")
}

#[test]
fn test_join_html_empty() {
    use std::iter::empty;
    assert_eq!(
        empty::<&str>()
            .join_html(
                |_o, _s| panic!("The callback should never be called"),
                ", ",
            )
            .to_buffer()
            .unwrap(),
        ""
    )
}
