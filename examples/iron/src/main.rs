#[macro_use]
extern crate mime;
extern crate iron;
extern crate router;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::io::{self, Write};

fn main() {
    let mut router = Router::new();
    router.get("/", page, "index");
    router.get("/static/:name", static_file, "static_file");
    let server = Iron::new(router).http("localhost:3000").unwrap();
    println!("Listening on {}.", server.socket);
}

fn page(_: &mut Request) -> IronResult<Response> {
    let mut buf = Vec::new();
    templates::page(&mut buf, &[("serious", 3), ("hard", 7), ("final", 3)])
        .expect("render template");
    Ok(Response::with((
        status::Ok,
        mime!(Text / Html; Charset=Utf8),
        buf,
    )))
}

fn footer(out: &mut Write) -> io::Result<()> {
    templates::footer(
        out,
        &[
            ("ructe", "https://crates.io/crates/ructe"),
            ("iron", "https://crates.io/crates/iron"),
        ],
    )
}

fn static_file(req: &mut Request) -> IronResult<Response> {
    let router = req.extensions.get::<Router>().expect("router");
    let name = router.find("name").expect("name");
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

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
