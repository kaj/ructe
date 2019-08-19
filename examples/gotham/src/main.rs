//! An example web service using ructe with the gotham framework.
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;
#[macro_use]
extern crate serde_derive;

mod ructe_response;

use gotham::router::builder::{
    build_simple_router, DefineSingleRoute, DrawRoutes,
};
use gotham::state::{FromState, State};
use hyper::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use hyper::{Body, Response, StatusCode};
use mime::TEXT_HTML;
use ructe_response::RucteResponse;
use std::io::{self, Write};
use templates::*;

/// The main routine starts a gotham server with a simple router
/// calling different handlers for some urls.
fn main() {
    let addr = "127.0.0.1:3000";
    println!("Starting server on http://{}/", addr);
    gotham::start(
        addr,
        build_simple_router(|route| {
            route.get("/").to(homepage);
            route
                .get("/static/:name")
                .with_path_extractor::<FilePath>()
                .to(static_file);
        }),
    )
}

/// A handler for the front page.
/// Simply render the page tempate with some arguments.
fn homepage(state: State) -> (State, Response<Body>) {
    // See the trait RucteResponse in ructe_response.rs for the html method.
    state.html(|o| page(o, &[("first", 3), ("second", 7), ("third", 2)]))
}

/// This method can be used as a "template tag", that is a method that
/// can be called directly from a template.
fn footer(out: &mut dyn Write) -> io::Result<()> {
    templates::footer(
        out,
        &[
            ("ructe", "https://crates.io/crates/ructe"),
            ("gotham", "https://gotham.rs/"),
        ],
    )
}

/// Gotham uses structs like this to extract arguments from the url.
/// In this case, the name of static file.
#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct FilePath {
    pub name: String,
}

/// Handler for static files.
/// The state will contain a FilePath.  The response from this view
/// should be the file data with a correct content type and a far
/// expires header (or a 404 if the file does not exist).
fn static_file(state: State) -> (State, Response<Body>) {
    let res = {
        let FilePath { ref name } = FilePath::borrow_from(&state);
        if let Some(data) = statics::StaticFile::get(&name) {
            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, data.mime.as_ref())
                .header(CACHE_CONTROL, "max-age: 31536000") // 1 year as seconds
                .body(data.content.into())
                .unwrap()
        } else {
            println!("Static file {} not found", name);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(CONTENT_TYPE, TEXT_HTML.as_ref())
                .body("not found".into())
                .unwrap()
        }
    };
    (state, res)
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
