//! An example web service using ructe with the warp framework.
use std::io::{self, Write};
use std::time::{Duration, SystemTime};
use templates::{statics::StaticFile, RenderRucte};
use warp::http::response::Builder;
use warp::http::StatusCode;
use warp::reply::Response;
use warp::{path, Filter, Rejection, Reply};

/// Main program: Set up routes and start server.
#[tokio::main]
async fn main() {
    env_logger::init();

    let routes = warp::get()
        .and(
            path::end()
                .then(home_page)
                .map(wrap)
                .or(path("static")
                    .and(path::param())
                    .then(static_file)
                    .map(wrap))
                .or(path("arg")
                    .and(path::param())
                    .then(arg_handler)
                    .map(wrap)),
        )
        .recover(customize_error);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

type Result<T, E = MyError> = std::result::Result<T, E>;

/// An error response is also a response.
///
/// Until <https://github.com/seanmonstar/warp/pull/909> is merged, we
/// need to do this manually, with a `.map(wrap)` after the handlers
/// above.
fn wrap(result: Result<impl Reply, impl Reply>) -> Response {
    match result {
        Ok(reply) => reply.into_response(),
        Err(err) => err.into_response(),
    }
}

/// Home page handler; just render a template with some arguments.
async fn home_page() -> Result<impl Reply> {
    Ok(Builder::new().html(|o| {
        templates::page_html(o, &[("first", 3), ("second", 7), ("third", 2)])
    })?)
}

/// A handler with some error handling.
///
/// Depending on the argument, it either returns a result or an error
/// (that may be NotFound or BadRequest).
async fn arg_handler(what: String) -> Result<Response> {
    // Note: This parsing could be done by typing `what` as usize in the
    // function signature.  This is just an example for mapping an error.
    let n: usize = what.parse().map_err(|_| MyError::NotFound)?;
    let w = match n {
        0 => return Err(MyError::BadRequest),
        1 => "one",
        2 | 3 | 5 | 7 | 11 | 13 => "prime",
        4 | 6 | 8 | 10 | 12 | 14 => "even",
        9 | 15 => "odd",
        _ => return Err(MyError::BadRequest),
    };
    Ok(Builder::new()
        .html(|o| templates::page_html(o, &[("first", 0), (w, n)]))?)
}

/// This method can be used as a "template tag", i.e. a method that
/// can be called directly from a template.
fn footer(out: &mut impl Write) -> io::Result<()> {
    templates::footer_html(
        out,
        &[
            ("ructe", "https://crates.io/crates/ructe"),
            ("warp", "https://crates.io/crates/warp"),
        ],
    )
}

/// Handler for static files.
/// Create a response from the file data with a correct content type
/// and a far expires header (or a 404 if the file does not exist).
async fn static_file(name: String) -> Result<impl Reply> {
    if let Some(data) = StaticFile::get(&name) {
        let _far_expires = SystemTime::now() + FAR;
        Ok(Builder::new()
            .status(StatusCode::OK)
            .header("content-type", data.mime.as_ref())
            // TODO .header("expires", _far_expires)
            .body(data.content))
    } else {
        Err(MyError::NotFound)
    }
}

/// A duration to add to current time for a far expires header.
static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);

/// Convert some rejections to MyError
///
/// This enables "nice" error responses.
async fn customize_error(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.is_not_found() {
        Ok(MyError::NotFound)
    } else {
        // Could identify some other errors and make nice messages here
        // but warp makes that rather hard, so lets just keep the rejection here.
        // that way we at least get the correct status code.
        Err(err)
    }
}

#[derive(Debug)]
enum MyError {
    NotFound,
    BadRequest,
    InternalError,
}

impl std::error::Error for MyError {}

impl warp::reject::Reject for MyError {}

impl Reply for MyError {
    fn into_response(self) -> Response {
        match self {
            MyError::NotFound => {
                wrap(Builder::new().status(StatusCode::NOT_FOUND).html(|o| {
                    templates::error_html(
                        o,
                        StatusCode::NOT_FOUND,
                        "The resource you requested could not be located.",
                    )
                }))
            }
            MyError::BadRequest => {
                let code = StatusCode::BAD_REQUEST;
                wrap(Builder::new().status(code).html(|o| {
                    templates::error_html(o, code, "I won't do that.")
                }))
            }
            MyError::InternalError => {
                let code = StatusCode::INTERNAL_SERVER_ERROR;
                wrap(Builder::new().status(code).html(|o| {
                    templates::error_html(o, code, "Something went wrong.")
                }))
            }
        }
    }
}

impl std::fmt::Display for MyError {
    fn fmt(&self, out: &mut std::fmt::Formatter) -> std::fmt::Result {
        out.write_str("Some error")
    }
}

impl From<templates::RenderError> for MyError {
    fn from(err: templates::RenderError) -> MyError {
        log::error!("Failed to render: {:?}", err);
        MyError::InternalError
    }
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
