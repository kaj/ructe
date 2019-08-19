use gotham::state::State;
use hyper::http::header::CONTENT_TYPE;
use hyper::{Body, Response, StatusCode};
use mime::TEXT_HTML_UTF_8;
use std::io::{self, Write};

pub trait RucteResponse: Sized {
    fn html<F>(self, do_render: F) -> (Self, Response<Body>)
    where
        F: FnOnce(&mut dyn Write) -> io::Result<()>;
}

impl RucteResponse for State {
    fn html<F>(self, do_render: F) -> (Self, Response<Body>)
    where
        F: FnOnce(&mut dyn Write) -> io::Result<()>,
    {
        let mut buf = Vec::new();
        let res = match do_render(&mut buf) {
            Ok(()) => Response::builder()
                .header(CONTENT_TYPE, TEXT_HTML_UTF_8.as_ref())
                .body(buf.into())
                .unwrap(),
            Err(e) => {
                println!("Rendering failed: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(CONTENT_TYPE, TEXT_HTML_UTF_8.as_ref())
                    .body(buf.into())
                    .unwrap()
            }
        };
        (self, res)
    }
}
