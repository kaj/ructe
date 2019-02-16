//! An example web service using ructe with the warp framework.
extern crate env_logger;
extern crate mime;
extern crate warp;

mod render_ructe;

use render_ructe::RenderRucte;
use std::io::{self, Write};
use std::time::{Duration, SystemTime};
use templates::statics::StaticFile;
use warp::http::{Response, StatusCode};
use warp::{path, Filter, Rejection, Reply};

/// Main program: Set up routes and start server.
fn main() {
    env_logger::init();

    let routes = warp::get2()
        .and(
            path::end()
                .and_then(home_page)
                .or(path("static").and(path::param()).and_then(static_file))
                .or(path("bad").and_then(bad_handler)),
        )
        .recover(customize_error);
    warp::serve(routes).run(([127, 0, 0, 1], 3030));
}

/// Home page handler; just render a template with some arguments.
fn home_page() -> Result<impl Reply, Rejection> {
    Response::builder().html(|o| {
        templates::page(o, &[("first", 3), ("second", 7), ("third", 2)])
    })
}

/// A handler that always gives a server error.
fn bad_handler() -> Result<StatusCode, Rejection> {
    Err(warp::reject::custom("bad handler"))
}

/// This method can be used as a "template tag", i.e. a method that
/// can be called directly from a template.
fn footer(out: &mut Write) -> io::Result<()> {
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
fn static_file(name: String) -> Result<impl Reply, Rejection> {
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
fn customize_error(err: Rejection) -> Result<impl Reply, Rejection> {
    match err.status() {
        StatusCode::NOT_FOUND => {
            eprintln!("Got a 404: {:?}", err);
            // We have a custom 404 page!
            Response::builder().status(StatusCode::NOT_FOUND).html(|o| {
                templates::error(
                    o,
                    StatusCode::NOT_FOUND,
                    "The resource you requested could not be located.",
                )
            })
        }
        code => {
            eprintln!("Got a {}: {:?}", code.as_u16(), err);
            Response::builder()
                .status(code)
                .html(|o| templates::error(o, code, "Something went wrong."))
        }
    }
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
