//! An example web service using ructe with the nickel framework.
extern crate hyper;
extern crate mime;
extern crate nickel;
extern crate time;

use hyper::header::{ContentType, Expires, HttpDate};
use nickel::status::StatusCode;
use nickel::{Halt, HttpRouter, MiddlewareResult, Nickel, Request, Response};
use std::io::{self, Write};
use time::{now, Duration};

/// The main routine creates a Nickel server, adds a route for static
/// files and one for the front page of the server, and then runs the
/// server.
fn main() {
    let mut server = Nickel::new();
    server.get("/static/:name.:ext", static_file);
    server.get("/", page);
    server.listen("127.0.0.1:6767").expect("listen");
}

/// A handler for static files.
/// The request should have the parameters `name` and `ext` from the route.
/// If those match an existing file, serve it, with its correct
/// content type and a far expires header.
/// Otherwise return a 404 result.
fn static_file<'mw>(
    req: &mut Request,
    mut res: Response<'mw>,
) -> MiddlewareResult<'mw> {
    if let (Some(name), Some(ext)) = (req.param("name"), req.param("ext")) {
        use templates::statics::StaticFile;
        if let Some(s) = StaticFile::get(&format!("{}.{}", name, ext)) {
            res.set(ContentType(s.mime()));
            res.set(Expires(HttpDate(now() + Duration::days(300))));
            return res.send(s.content);
        }
    }
    res.error(StatusCode::NotFound, "Not found")
}

/// A handler for the front page of the server.
/// Simple render a template with some arguments.
fn page<'mw>(
    _req: &mut Request,
    res: Response<'mw>,
) -> MiddlewareResult<'mw> {
    use templates::page;
    render(res, |o| page(o, &[("silly", 4), ("long", 7), ("final", 3)]))
}

fn render<F>(res: Response, do_render: F) -> MiddlewareResult
where
    F: FnOnce(&mut Write) -> io::Result<()>,
{
    let mut stream = res.start()?;
    match do_render(&mut stream) {
        Ok(()) => Ok(Halt(stream)),
        Err(e) => stream.bail(format!("Problem rendering template: {:?}", e)),
    }
}

/// This method can be used as a "template tag", that is a method that
/// can be called directly from a template.
fn footer(out: &mut Write) -> io::Result<()> {
    templates::footer(
        out,
        &[
            ("ructe", "https://crates.io/crates/ructe"),
            ("nickel", "https://crates.io/crates/nickel"),
        ],
    )
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
