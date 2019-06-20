use warp::http::{header::CONTENT_TYPE, response::Builder, Response};
use warp::{reject::custom, Rejection};
use mime::TEXT_HTML_UTF_8;

/// Extension trait for [`response::Builder`] to simplify template rendering.
///
/// Render a template to a buffer, and use that buffer to complete a
/// `Response` from the builder.  Also set the content type of the
/// response to `TEXT_HTML_UTF_8`.
///
/// # Examples
///
/// Give a template `page`, that takes two arguments other than the
/// `Write` buffer, this will use the variables `title` and `body` and
/// render the template to a response.
///
/// ```
/// # extern crate warp;
/// # use std::io::{self, Write};
/// # use warp::http::Response;
/// # use ructe::templates::RenderRucte;
/// # fn page(o: &mut Write, _: u8, _: u8) -> io::Result<()> { Ok(()) }
/// # let (title, body) = (47, 11);
/// Response::builder().html(|o| page(o, title, body))
/// # ;
/// ```
///
/// Other builder methods can be called before calling the `html` method.
/// Here is an example that sets a cookie in the Response.
///
/// ```
/// # extern crate warp;
/// # use std::io::{self, Write};
/// # use warp::http::{header::SET_COOKIE, Response};
/// # use ructe::templates::RenderRucte;
/// # fn page(o: &mut Write, _: u8, _: u8) -> io::Result<()> { Ok(()) }
/// # let (title, body, value) = (47, 11, 14);
/// Response::builder()
///     .header(SET_COOKIE, format!("FOO={}, SameSite=Strict; HttpOnly", value))
///     .html(|o| page(o, title, body))
/// # ;
/// ```
///
/// [`response::Builder`]: ../../http/response/struct.Builder.html
pub trait RenderRucte {
    /// Render a template on the response builder.
    ///
    /// This is the main function of the trait.  Please see the trait documentation.
    fn html<F>(&mut self, f: F) -> Result<Response<Vec<u8>>, Rejection>
    where
        F: FnOnce(&mut Vec<u8>) -> io::Result<()>;
}

impl RenderRucte for Builder {
    fn html<F>(&mut self, f: F) -> Result<Response<Vec<u8>>, Rejection>
    where
        F: FnOnce(&mut Vec<u8>) -> io::Result<()>,
    {
        let mut buf = Vec::new();
        f(&mut buf).map_err(custom)?;
        self.header(CONTENT_TYPE, TEXT_HTML_UTF_8.as_ref())
            .body(buf)
            .map_err(custom)
    }
}
