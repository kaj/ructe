//! An example web service using ructe with the gotham framework.
mod ructe_response;

use crate::ructe_response::RucteResponse;
use crate::templates::*;
use gotham::StartError;
use gotham::helpers::http::Body;
use gotham::helpers::http::response::create_response;
use gotham::http::header::CACHE_CONTROL;
use gotham::http::{Response, StatusCode};
use gotham::router::builder::{
    DefineSingleRoute, DrawRoutes, build_simple_router,
};
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};
use mime::TEXT_HTML;
use serde::Deserialize;
use std::io::{self, Write};

/// The main routine starts a gotham server with a simple router
/// calling different handlers for some urls.
fn main() -> Result<(), StartError> {
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
    state.html(|o| page_html(o, &[("first", 3), ("second", 7), ("third", 2)]))
}

/// This method can be used as a "template tag", that is a method that
/// can be called directly from a template.
fn footer(out: &mut impl Write) -> io::Result<()> {
    footer_html(
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
        let FilePath { name } = FilePath::borrow_from(&state);
        if let Some(data) = statics::StaticFile::get(name) {
            let mut response = create_response(
                &state,
                StatusCode::OK,
                data.mime.clone(),
                data.content,
            );
            response.headers_mut().append(
                CACHE_CONTROL,
                "max-age: 31536000".try_into().unwrap(),
            ); // 1 year as seconds
            response
        } else {
            println!("Static file {name} not found");
            create_response(
                &state,
                StatusCode::NOT_FOUND,
                TEXT_HTML,
                "not found".as_bytes(),
            )
        }
    };
    (state, res)
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
