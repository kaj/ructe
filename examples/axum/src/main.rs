use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use std::io::{self, Write};

use templates::statics::StaticFile;

#[macro_use]
mod axum_ructe;

/// Setup routes
fn app() -> Router {
    Router::new()
        .route("/", get(home_page))
        .route("/static/:filename", get(static_files))
        .route("/int/:n", get(take_int))
        .route("/bad", get(make_error))
        .fallback(handler_404)
}

/// Home page handler; just render a template with some arguments.
async fn home_page() -> impl IntoResponse {
    render!(
        templates::page_html,
        &[("first", 3), ("second", 7), ("third", 2)]
    )
}

/// Handler for static files.
/// Create a response from the file data with a correct content type
/// and a far expires header (or a 404 if the file does not exist).
async fn static_files(Path(filename): Path<String>) -> Response {
    match StaticFile::get(&filename) {
        Some(data) => {
            (
                [
                    (header::CONTENT_TYPE, data.mime.as_ref()),
                    (
                        header::CACHE_CONTROL,
                        // max age is 180 days (given in seconds)
                        "public, max_age=15552000, immutable",
                    ),
                ],
                data.content,
            )
                .into_response()
        }
        None => handler_404().await.into_response(),
    }
}

async fn take_int(Path(n): Path<usize>) -> impl IntoResponse {
    render!(
        templates::page_html,
        &[(&format!("number {}", n), 1 + n % 7)]
    )
}

/// This function always fail, to show an example of error handling.
async fn make_error() -> Result<impl IntoResponse, ExampleAppError> {
    let i = "three".parse()?;
    Ok(render!(templates::page_html, &[("first", i)]))
}

/// The error type that can be returned from resource handlers.
///
/// This needs to be convertible from any error types used with `?` in
/// handlers, and implement the axum [IntoResponse] trait.
#[derive(Debug)]
enum ExampleAppError {
    ParseInt(std::num::ParseIntError),
}
impl std::fmt::Display for ExampleAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseInt(e) => write!(f, "Bad integer: {e}"),
        }
    }
}
impl std::error::Error for ExampleAppError {}

impl From<std::num::ParseIntError> for ExampleAppError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}
impl IntoResponse for ExampleAppError {
    /// Handle the error by creating a response for it.
    ///
    /// This is also where the error is logged.  A real service may
    /// use `tracing` to add context to the logged error message.
    fn into_response(self) -> Response {
        log::error!("ISE: {self:?}");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Sorry, Something went wrong. This is probably not your fault.",
        )
        .into_response()
    }
}

/// This method can be used as a "template tag", i.e. a method that
/// can be called directly from a template.
fn footer(out: &mut impl Write) -> io::Result<()> {
    templates::footer_html(
        out,
        &[
            ("ructe", "https://crates.io/crates/ructe"),
            ("axum", "https://crates.io/crates/axum"),
        ],
    )
}

async fn handler_404() -> impl IntoResponse {
    error_response(
        StatusCode::NOT_FOUND,
        "The resource you requested can't be found.",
    )
}

fn error_response(
    status_code: StatusCode,
    message: &str,
) -> impl IntoResponse + '_ {
    (
        status_code,
        render!(templates::error_html, status_code, message),
    )
}

/// Start server
#[tokio::main]
async fn main() {
    env_logger::init();

    let listener =
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    log::info!("Listening on {}.", listener.local_addr().unwrap());
    axum::serve(listener, app().into_make_service())
        .await
        .unwrap()
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
