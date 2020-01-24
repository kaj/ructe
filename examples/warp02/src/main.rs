//! An example web service using ructe with the warp framework.
use std::io::{self, Write};
use std::time::{Duration, SystemTime};
use templates::{statics::StaticFile, RenderRucte};
use warp::http::{Response, StatusCode};
use warp::{path, Filter, Rejection, Reply};

/// Main program: Set up routes and start server.
#[tokio::main]
async fn main() {
    env_logger::init();

    let routes = warp::get()
        .and(
            path::end()
                .and_then(home_page)
                .or(path("static").and(path::param()).and_then(static_file))
                .or(path("bad").and_then(bad_handler)),
        )
        .recover(customize_error);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

/// Home page handler; just render a template with some arguments.
async fn home_page() -> Result<impl Reply, Rejection> {
    Response::builder().html(|o| {
        templates::page(o, &[("first", 3), ("second", 7), ("third", 2)])
    })
}

#[derive(Debug)]
struct SomeError;
impl std::error::Error for SomeError {}
impl warp::reject::Reject for SomeError {}

impl std::fmt::Display for SomeError {
    fn fmt(&self, out: &mut std::fmt::Formatter) -> std::fmt::Result {
        out.write_str("Some error")
    }
}

/// A handler that always gives a server error.
async fn bad_handler() -> Result<StatusCode, Rejection> {
    Err(warp::reject::custom(SomeError))
}

/// This method can be used as a "template tag", i.e. a method that
/// can be called directly from a template.
fn footer(out: &mut dyn Write) -> io::Result<()> {
    templates::footer(
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
async fn static_file(name: String) -> Result<impl Reply, Rejection> {
    if let Some(data) = StaticFile::get(&name) {
        let _far_expires = SystemTime::now() + FAR;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", data.mime.as_ref())
            // TODO .header("expires", _far_expires)
            .body(data.content))
    } else {
        println!("Static file {} not found", name);
        Err(warp::reject::not_found())
    }
}

/// A duration to add to current time for a far expires header.
static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);

/// Create custom error pages.
async fn customize_error(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.is_not_found() {
            eprintln!("Got a 404: {:?}", err);
            // We have a custom 404 page!
            Response::builder().status(StatusCode::NOT_FOUND).html(|o| {
                templates::error(
                    o,
                    StatusCode::NOT_FOUND,
                    "The resource you requested could not be located.",
                )
            })
    } else {
        let code = StatusCode::INTERNAL_SERVER_ERROR; // FIXME
        eprintln!("Got a {}: {:?}", code.as_u16(), err);
        Response::builder()
           .status(code)
           .html(|o| templates::error(o, code, "Something went wrong."))
    }
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
