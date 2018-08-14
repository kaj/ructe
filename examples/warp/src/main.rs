//! An example web service using ructe with the warp framework.
#![deny(warnings)]
extern crate env_logger;
extern crate warp;

use std::io::{self, Write};
use std::time::{Duration, SystemTime};
use templates::statics::StaticFile;
use warp::http::{Response, StatusCode};
use warp::{Filter, Rejection, Reply};

/// Main program: Set up routes and start server.
fn main() {
    env_logger::init();

    let index = warp::index().map(home_page);
    let statics = warp::path("static")
        .and(warp::path::param())
        .map(static_file);

    let route = warp::get2().and(index.or(statics)).recover(customize_error);
    warp::serve(route).run(([127, 0, 0, 1], 3030));
}

fn home_page() -> impl warp::Reply {
    use templates::page;
    let mut buf = Vec::new();
    match page(&mut buf, &[("first", 3), ("second", 7), ("third", 2)]) {
        Ok(()) => String::from_utf8(buf).unwrap(),
        Err(e) => {
            println!("Rendering failed: {}", e);
            "error".into()
        }
    }
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
fn static_file(name: String) -> Result<impl Reply, warp::http::Error> {
    if let Some(data) = StaticFile::get(&name) {
        let _far_expires = SystemTime::now() + FAR;
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", data.mime.as_ref())
            // TODO .header("expires", _far_expires)
            .body(data.content)
    } else {
        println!("Static file {} not found", name);
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "text/plain")
            .body(&b"not found"[..])
    }
}

/// A duration to add to current time for a far expires header.
static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);

fn customize_error(err: Rejection) -> Result<impl Reply, Rejection> {
    match err.status() {
        StatusCode::NOT_FOUND => {
            let mut buf = Vec::new();
            use templates::not_found;
            match not_found(&mut buf) {
                Ok(()) => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(buf)),
                Err(e) => {
                    println!("Rendering failed: {}", e);
                    Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(
                            b"you get a 404, and *you* get a 404...".to_vec(),
                        ))
                }
            }
            // We have a custom 404 page!
        }
        StatusCode::INTERNAL_SERVER_ERROR => {
            // Oh no, something is on fire!
            eprintln!("quick, page someone! fire!");
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(b":fire: this is fine".to_vec()))
        }
        _ => {
            // Don't customize these errors, just let warp do
            // the default! (or optionally a later filter could
            // customize these).
            Err(err)
        }
    }
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
