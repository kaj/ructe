//! An example web service using ructe with the gotham framework.
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;
#[macro_use]
extern crate serde_derive;

mod ructe_response;

use gotham::http::response::create_response;
use gotham::router::builder::{
    build_simple_router, DefineSingleRoute, DrawRoutes,
};
use gotham::state::{FromState, State};
use hyper::header::Expires;
use hyper::{Response, StatusCode};
use ructe_response::RucteResponse;
use std::io::{self, Write};
use std::time::{Duration, SystemTime};
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
fn homepage(state: State) -> (State, Response) {
    // See the trait RucteResponse in ructe_response.rs for the html method.
    state.html(|o| page(o, &[("first", 3), ("second", 7), ("third", 2)]))
}

/// This method can be used as a "template tag", that is a method that
/// can be called directly from a template.
fn footer(out: &mut Write) -> io::Result<()> {
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
fn static_file(state: State) -> (State, Response) {
    let res = {
        let FilePath { ref name } = FilePath::borrow_from(&state);
        if let Some(data) = statics::StaticFile::get(&name) {
            create_response(
                &state,
                StatusCode::Ok,
                Some((data.content.to_vec(), data.mime.clone())),
            ).with_header(Expires((SystemTime::now() + FAR).into()))
        } else {
            println!("Static file {} not found", name);
            create_response(&state, StatusCode::NotFound, None)
        }
    };
    (state, res)
}

static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
