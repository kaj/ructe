use axum::response::{Html, IntoResponse};

macro_rules! render {
    ($template:path) => {{
        use axum_ructe::Render;
        Render(|o| $template(o))
    }};
    ($template:path, $($arg:expr),*) => {{
        use axum_ructe::Render;
        Render(move |o| $template(o, $($arg),*))
    }}
}

pub struct Render<T: FnOnce(&mut Vec<u8>) -> std::io::Result<()>>(pub T);

impl<T: FnOnce(&mut Vec<u8>) -> std::io::Result<()>> IntoResponse
    for Render<T>
{
    fn into_response(self) -> axum::response::Response {
        let mut buf = Vec::new();
        match self.0(&mut buf) {
            Ok(()) => Html(buf).into_response(),
            Err(_e) => {
                // TODO: logging
                "Render failed".into_response()
            }
        }
    }
}
