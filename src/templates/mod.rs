//! The module containing your generated template code will also
//! contain everything from here.
//!
//! The name `ructe::templates` should never be used.  Instead, you
//! should use the module templates created when compiling your
//! templates.
//! If you include the generated `templates.rs` in your `main.rs` (or
//! `lib.rs` in a library crate), this module will be
//! `crate::templates`.

mod utils;
pub use self::utils::*;

#[cfg(feature = "mime03")]
use mime::Mime;

#[cfg(feature = "mime02")]
/// Documentation mock.  The real Mime type comes from the `mime` crate.
pub type Mime = u8; // mock

/// A static file has a name (so its url can be recognized) and the
/// actual file contents.
///
/// The content-type (mime type) of the file is available as a
/// static field when building ructe with the `mime03` feature or
/// as the return value of a method when building ructe with the
/// `mime02` feature (in `mime` version 0.2.x, a Mime cannot be
/// defined as a part of a const static value.
pub struct StaticFile {
    /// The actual static file contents.
    pub content: &'static [u8],
    /// The file name as used in a url, including a short (48 bits
    /// as 8 base64 characters) hash of the content, to enable
    /// long-time caching of static resourses in the clients.
    pub name: &'static str,
    /// The Mime type of this static file, as defined in the mime
    /// crate version 0.3.x.
    #[cfg(feature = "mime03")]
    pub mime: &'static Mime,
}

impl StaticFile {
    /// Get the mime type of this static file.
    ///
    /// Currently, this method parses a (static) string every time.
    /// A future release of `mime` may support statically created
    /// `Mime` structs, which will make this nicer.
    #[allow(unused)]
    #[cfg(feature = "mime02")]
    pub fn mime(&self) -> Mime {
        unimplemented!()
    }
}

#[test]
fn encoded() {
    let mut buf = Vec::new();
    "a < b\0\n".to_html(&mut buf).unwrap();
    assert_eq!(b"a &lt; b\0\n", &buf[..]);

    let mut buf = Vec::new();
    "'b".to_html(&mut buf).unwrap();
    assert_eq!(b"&#39;b", &buf[..]);

    let mut buf = Vec::new();
    "xxxxx>&".to_html(&mut buf).unwrap();
    assert_eq!(b"xxxxx&gt;&amp;", &buf[..]);
}

#[test]
fn encoded_empty() {
    let mut buf = Vec::new();
    "".to_html(&mut buf).unwrap();
    "".to_html(&mut buf).unwrap();
    "".to_html(&mut buf).unwrap();
    assert_eq!(b"", &buf[..]);
}

#[test]
fn double_encoded() {
    let mut buf = Vec::new();
    "&amp;".to_html(&mut buf).unwrap();
    "&lt;".to_html(&mut buf).unwrap();
    assert_eq!(b"&amp;amp;&amp;lt;", &buf[..]);
}

#[test]
fn encoded_only() {
    let mut buf = Vec::new();
    "&&&&&&&&&&&&&&&&".to_html(&mut buf).unwrap();
    assert_eq!(b"&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;" as &[u8], &buf[..]);

    let mut buf = Vec::new();
    "''''''''''''''".to_html(&mut buf).unwrap();
    assert_eq!(b"&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;" as &[u8], &buf[..]);
}

#[test]
fn raw_html() {
    let mut buf = Vec::new();
    Html("a<b>c</b>").to_html(&mut buf).unwrap();
    assert_eq!(b"a<b>c</b>", &buf[..]);
}
