//! An example web service using ructe with the iron framework.
#[macro_use]
extern crate mime;
extern crate iron;
extern crate router;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::io::{self, Write};

/// The main routine creates a request router, adds a route for static
/// files and one for the front page of the server.
/// Then it starts a server, listening on localhost:3000.
fn main() {
    let mut router = Router::new();
    router.get("/", frontpage, "index");
    router.get("/static/:name", static_file, "static_file");
    let server = Iron::new(router).http("localhost:3000").unwrap();
    println!("Listening on http://{}/", server.socket);
}

/// A handler for the front page of the server.
/// Simple render a template with some arguments and return a response
/// with the resulting html.
fn frontpage(_: &mut Request) -> IronResult<Response> {
    let mut buf = Vec::new();
    templates::page_html(
        &mut buf,
        &[("serious", 3), ("hard", 7), ("final", 3)],
    )
    .expect("render template");
    Ok(Response::with((
        status::Ok,
        mime!(Text / Html; Charset=Utf8),
        buf,
    )))
}

/// This method can be used as a "template tag", that is a method that
/// can be called directly from a template.
fn footer(out: &mut impl Write) -> io::Result<()> {
    templates::footer_html(
        out,
        &[
            ("ructe", "https://crates.io/crates/ructe"),
            ("iron", "https://crates.io/crates/iron"),
        ],
    )
}

/// A handler for static files.
/// The request should have the parameters `name` and `ext` from the route.
/// If those match an existing file, serve it, with its correct
/// content type.
/// Otherwise return a 404 result.
fn static_file(req: &mut Request) -> IronResult<Response> {
    // Extract the requested file name from the router
    let router = req.extensions.get::<Router>().expect("router");
    let name = router.find("name").expect("name");
    // If the static files exists, serve it
    if let Some(data) = templates::statics::StaticFile::get(name) {
        Ok(Response::with((status::Ok, data.mime(), data.content)))
    } else {
        println!("Static file {} not found", name);
        Ok(Response::with((
            status::NotFound,
            mime!(Text / Plain),
            "not found",
        )))
    }
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
