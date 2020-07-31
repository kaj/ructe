pub trait Render {
    fn render<Call>(&mut self, call: Call) -> std::io::Result<()>
    where
        Call: FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>;

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

pub trait RenderBuilder {
    fn render<Call>(self, call: Call) -> tide::ResponseBuilder
    where
        Call: FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>;

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
