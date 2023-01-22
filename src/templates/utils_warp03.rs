use mime::TEXT_HTML_UTF_8;
use std::error::Error;
use std::io;
use warp::http::{header::CONTENT_TYPE, response::Builder};
use warp::{reject::Reject, reply::Response, Reply};

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
    fn html<F>(self, f: F) -> Result<Response, RenderError>
    where
        F: FnOnce(&mut Vec<u8>) -> io::Result<()>;
}

impl RenderRucte for Builder {
    fn html<F>(self, f: F) -> Result<Response, RenderError>
    where
        F: FnOnce(&mut Vec<u8>) -> io::Result<()>,
    {
        let mut buf = Vec::new();
        f(&mut buf).map_err(RenderError::write)?;
        self.header(CONTENT_TYPE, TEXT_HTML_UTF_8.as_ref())
            .body(buf.into())
            .map_err(RenderError::build)
    }
}

/// Error type for [`RenderRucte::html`].
///
/// This type implements [`Error`] for common Rust error handling, but
/// also both [`Reply`] and [`Reject`] to facilitate use in warp filters
/// and handlers.
#[derive(Debug)]
pub struct RenderError {
    im: RenderErrorImpl,
}
impl RenderError {
    fn build(e: warp::http::Error) -> Self {
        RenderError { im: RenderErrorImpl::Build(e) }
    }
    fn write(e: std::io::Error) -> Self {
        RenderError { im: RenderErrorImpl::Write(e) }
    }
}

// make variants private
#[derive(Debug)]
enum RenderErrorImpl {
    Write(std::io::Error),
    Build(warp::http::Error),
}

impl Error for RenderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.im {
            RenderErrorImpl::Write(e) => Some(e),
            RenderErrorImpl::Build(e) => Some(e),
        }
    }
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, out: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.im {
            RenderErrorImpl::Write(_) => "Failed to write template",
            RenderErrorImpl::Build(_) => "Failed to build response",
        }.fmt(out)
    }
}

impl Reject for RenderError {}

impl Reply for RenderError {
    fn into_response(self) -> Response {
        Response::new(self.to_string().into())
    }
}
