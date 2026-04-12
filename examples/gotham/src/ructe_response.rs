use gotham::helpers::http::Body;
use gotham::helpers::http::response::create_response;
use gotham::http::{Response, StatusCode};
use gotham::state::State;
use mime::TEXT_HTML_UTF_8;
use std::io;

pub trait RucteResponse: Sized {
    fn html<F>(self, do_render: F) -> (Self, Response<Body>)
    where
        F: FnOnce(&mut Vec<u8>) -> io::Result<()>;
}

impl RucteResponse for State {
    fn html<F>(self, do_render: F) -> (Self, Response<Body>)
    where
        F: FnOnce(&mut Vec<u8>) -> io::Result<()>,
    {
        let mut buf = Vec::new();
        let res = match do_render(&mut buf) {
            Ok(()) => {
                create_response(&self, StatusCode::OK, TEXT_HTML_UTF_8, buf)
            }
            Err(e) => {
                println!("Rendering failed: {}", e);
                create_response(
                    &self,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    TEXT_HTML_UTF_8,
                    buf,
                )
            }
        };
        (self, res)
    }
}
