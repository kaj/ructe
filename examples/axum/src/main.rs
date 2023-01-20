use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router, Server, TypedHeader,
};
use headers::{ContentType, Expires};

use std::{
    io::{self, Write},
    time::{Duration, SystemTime},
};

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
        templates::page,
        &[("first", 3), ("second", 7), ("third", 2)]
    )
}

/// Handler for static files.
/// Create a response from the file data with a correct content type
/// and a far expires header (or a 404 if the file does not exist).
async fn static_files(Path(filename): Path<String>) -> impl IntoResponse {
    /// A duration to add to current time for a far expires header.
    static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);
    match StaticFile::get(&filename) {
        Some(data) => {
            let far_expires = SystemTime::now() + FAR;
            (
                TypedHeader(ContentType::from(data.mime.clone())),
                TypedHeader(Expires::from(far_expires)),
                data.content,
            )
                .into_response()
        }
        None => handler_404().await.into_response(),
    }
}

async fn take_int(payload: Option<Path<usize>>) -> Response {
    if let Some(Path(n)) = payload {
        render!(templates::page, &[(&format!("number {}", n), 1 + n % 7)])
            .into_response()
    } else {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Sorry, Something went wrong. This is probably not your fault.",
        )
        .into_response()
    }
}

async fn make_error() -> Result<impl IntoResponse, ExampleAppError> {
    let i = "three".parse()?;
    Ok(render!(templates::page, &[("first", i)]))
}

/// The error type that can be returned from resource handlers.
///
/// This needs to be convertible from any error types used with `?` in
/// handlers, and implement the actix ResponseError type.
#[derive(Debug)]
enum ExampleAppError {
    ParseInt(std::num::ParseIntError),
}
impl std::fmt::Display for ExampleAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for ExampleAppError {}

impl From<std::num::ParseIntError> for ExampleAppError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}
impl IntoResponse for ExampleAppError {
    fn into_response(self) -> Response {
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
    templates::footer(
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
    (status_code, render!(templates::error, status_code, message))
}

/// Start server
#[tokio::main]
async fn main() {
    env_logger::init();
    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app().into_make_service())
        .await
        .unwrap()
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
