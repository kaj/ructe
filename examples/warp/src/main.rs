//! An example web service using ructe with the warp framework.
#![deny(warnings)]
extern crate env_logger;
extern crate warp;

use std::io::{self, Write};
use std::time::{Duration, SystemTime};
use templates::statics::StaticFile;
use warp::http::{Response, StatusCode};
use warp::{path, reject, Filter, Rejection, Reply};

/// Main program: Set up routes and start server.
fn main() {
    env_logger::init();

    let index = warp::index().and_then(home_page);
    let statics = path("static").and(path::param()).and_then(static_file);
    let bad = path("bad").and_then(bad_handler);

    let route = warp::get2()
        .and(index.or(statics).or(bad))
        .recover(customize_error);
    warp::serve(route).run(([127, 0, 0, 1], 3030));
}

fn home_page() -> Result<impl Reply, Rejection> {
    use templates::page;
    let mut buf = Vec::new();
    page(&mut buf, &[("first", 3), ("second", 7), ("third", 2)])
        .map_err(|_| reject::server_error())?;
    Ok(String::from_utf8(buf).unwrap())
}

fn bad_handler() -> Result<StatusCode, Rejection> {
    Err(reject::server_error())
}

/// This method can be used as a "template tag", that is a method that
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
/// The state will contain a FilePath.  The response from this view
/// should be the file data with a correct content type and a far
/// expires header (or a 404 if the file does not exist).
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
        Err(reject::not_found())
    }
}

/// A duration to add to current time for a far expires header.
static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);

fn customize_error(err: Rejection) -> Result<impl Reply, Rejection> {
    match err.status() {
        StatusCode::NOT_FOUND => {
            eprintln!("Got a 404: {:?}", err);
            // We have a custom 404 page!
            let mut buf = Vec::new();
            templates::error(
                &mut buf,
                StatusCode::NOT_FOUND,
                "The resource you requested could not be located.",
            ).map_err(|_| err)?;
            Ok(Response::builder().status(StatusCode::NOT_FOUND).body(buf))
        }
        code => {
            eprintln!("Got a {}: {:?}", code.as_u16(), err);
            let mut buf = Vec::new();
            templates::error(&mut buf, code, "Something went wrong.")
                .map_err(|_| err)?;
            Ok(Response::builder().status(code).body(buf))
        }
    }
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
