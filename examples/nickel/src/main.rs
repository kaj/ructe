extern crate hyper;
extern crate mime;
extern crate nickel;
extern crate time;

use hyper::header::{ContentType, Expires, HttpDate};
use nickel::status::StatusCode;
use nickel::{Halt, HttpRouter, MiddlewareResult, Nickel, Request, Response};
use std::io::{self, Write};
use time::{now, Duration};

fn main() {
    let mut server = Nickel::new();
    server.get("/static/:name.:ext", static_file);
    server.get("/", page);
    server.listen("127.0.0.1:6767").expect("listen");
}

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

fn page<'mw>(
    _req: &mut Request,
    res: Response<'mw>,
) -> MiddlewareResult<'mw> {
    use templates::page;
    render(res, |o| page(o, &[("silly", 4), ("long", 7), ("final", 3)]))
}

fn render<'mw, F>(res: Response<'mw>, do_render: F) -> MiddlewareResult<'mw>
where
    F: FnOnce(&mut Write) -> io::Result<()>,
{
    let mut stream = res.start()?;
    match do_render(&mut stream) {
        Ok(()) => Ok(Halt(stream)),
        Err(e) => stream.bail(format!("Problem rendering template: {:?}", e)),
    }
}

fn footer(out: &mut Write) -> io::Result<()> {
    templates::footer(
        out,
        &[
            ("ructe", "https://crates.io/crates/ructe"),
            ("nickel", "https://crates.io/crates/nickel"),
        ],
    )
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
