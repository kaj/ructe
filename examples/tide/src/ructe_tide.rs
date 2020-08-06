//! These traits and impl may be included in the tide feature of ructe
//! in a future release.
//!
//! Comments welcome at
//! [kaj/ructe#79](https://github.com/kaj/ructe/issues/79).

/// Add `render` and `render_html` methods to [`tide::Response`].
///
/// [`tide::Response`]: ../../tide/struct.Response.html
pub trait Render {
    /// Render a template to the body of self.
    ///
    /// The `Call` takes a `Write` target as only argument, other
    /// arguments will typically be moved or borrowed into a closure
    /// that is used as `call`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tide::Response;
    /// # use ructe_tide::ructe_tide::Render;
    /// # use std::io::{self, Write};
    /// # // Mock template:
    /// # fn page(o: impl Write, c: &str, n: u8) -> io::Result<()> {
    /// #     Ok(())
    /// # }
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = "something";
    /// let other = 17;
    /// let mut result = Response::new(200);
    /// result.render(|o| page(o, content, other))?;
    /// # Ok(())
    /// # }
    /// ```
    fn render<Call>(&mut self, call: Call) -> std::io::Result<()>
    where
        Call: FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>;

    /// Render a template to the html body of self.
    ///
    /// just like `render`, excep it also sets the content-type of the
    /// respons to [`HTML`].
    ///
    /// [`HTML`]: ../../tide/http/mime/constant.HTML.html
    fn render_html<Call>(&mut self, call: Call) -> std::io::Result<()>
    where
        Call: FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>;
}

impl Render for tide::Response {
    fn render<Call>(&mut self, call: Call) -> std::io::Result<()>
    where
        Call: FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
    {
        let mut buf = Vec::new();
        call(&mut buf)?;
        self.set_body(buf);
        Ok(())
    }

    fn render_html<Call>(&mut self, call: Call) -> std::io::Result<()>
    where
        Call: FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
    {
        self.render(call)?;
        self.set_content_type(tide::http::mime::HTML);
        Ok(())
    }
}

/// Add `render` and `render_html` methods to [`tide::ResponseBuilder`].
///
/// [`tide::Response`]: ../../tide/struct.ResponseBuilder.html
pub trait RenderBuilder {
    /// Render a template to the body of self.
    ///
    /// The `Call` takes a `Write` target as only argument, other
    /// arguments will typically be moved or borrowed into a closure
    /// that is used as `call`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tide::Response;
    /// # use ructe_tide::ructe_tide::RenderBuilder;
    /// # use std::io::{self, Write};
    /// # // Mock template:
    /// # fn page(o: impl Write, c: &str, n: u8) -> io::Result<()> {
    /// #     Ok(())
    /// # }
    /// let content = "something";
    /// let other = 17;
    /// Response::builder(200)
    ///     .render(|o| page(o, content, other))
    ///     .build()
    /// # ;
    /// ```
    fn render<Call>(self, call: Call) -> tide::ResponseBuilder
    where
        Call: FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>;

    /// Render a template to the html body of self.
    ///
    /// just like `render`, excep it also sets the content-type of the
    /// respons to [`HTML`].
    ///
    /// [`HTML`]: ../../tide/http/mime/constant.HTML.html
    fn render_html<Call>(self, call: Call) -> tide::ResponseBuilder
    where
        Call: FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>;
}

impl RenderBuilder for tide::ResponseBuilder {
    fn render<Call>(self, call: Call) -> tide::ResponseBuilder
    where
        Call: FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
    {
        let mut buf = Vec::new();
        match call(&mut buf) {
            Ok(()) => self.body(buf),
            Err(e) => {
                // NOTE: A tide::Response may contain an Error, but there
                // seem to be no way of setting that in a ResponseBuilder,
                // so I just log the error and return a builder for a
                // generic internal server error.
                tide::log::error!("Failed to render response: {}", e);
                tide::Response::builder(500)
            }
        }
    }

    fn render_html<Call>(self, call: Call) -> tide::ResponseBuilder
    where
        Call: FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
    {
        self.content_type(tide::http::mime::HTML).render(call)
    }
}
