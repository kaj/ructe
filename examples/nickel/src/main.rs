extern crate nickel;
extern crate hyper;
extern crate time;
#[macro_use]
extern crate mime;

use hyper::header::{ContentType, Expires, HttpDate};
use nickel::{HttpRouter, MiddlewareResult, Nickel, Request, Response};
use nickel::status::StatusCode;
use time::{Duration, now};

fn main() {
    let mut server = Nickel::new();
    server.get("/static/:name.:ext", static_file);
    server.get("/", page);
    server.listen("127.0.0.1:6767").expect("listen");
}


fn static_file<'mw>(req: &mut Request,
                    mut res: Response<'mw>)
                    -> MiddlewareResult<'mw> {

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

fn page<'mw>(_req: &mut Request,
             mut res: Response<'mw>)
             -> MiddlewareResult<'mw> {
    use templates;
    let mut buf = Vec::new();
    templates::page(&mut buf).unwrap();
    res.set(ContentType(mime!(Text/Html; Charset=Utf8)));
    res.send(buf)
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
