use gotham::http::response::create_response;
use gotham::state::State;
use hyper::{Response, StatusCode};
use mime::TEXT_HTML_UTF_8;
use std::io::{self, Write};

pub trait RucteResponse: Sized {
    fn html<F>(self, do_render: F) -> (Self, Response)
    where
        F: FnOnce(&mut Write) -> io::Result<()>;
}

impl RucteResponse for State {
    fn html<F>(self, do_render: F) -> (Self, Response)
    where
        F: FnOnce(&mut Write) -> io::Result<()>,
    {
        let mut buf = Vec::new();
        let res = match do_render(&mut buf) {
            Ok(()) => create_response(
                &self,
                StatusCode::Ok,
                Some((buf, TEXT_HTML_UTF_8)),
            ),
            Err(e) => {
                println!("Rendering failed: {}", e);
                create_response(&self, StatusCode::InternalServerError, None)
            }
        };
        (self, res)
    }
}
