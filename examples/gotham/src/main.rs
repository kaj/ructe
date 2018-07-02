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
use gotham::router::Router;
use gotham::state::{FromState, State};
use hyper::header::Expires;
use hyper::{Response, StatusCode};
use ructe_response::RucteResponse;
use std::io::{self, Write};
use std::time::{Duration, SystemTime};
use templates::*;

fn main() {
    let addr = "127.0.0.1:3000";
    println!("Starting server on http://{}/", addr);
    gotham::start(addr, router())
}

pub fn router() -> Router {
    build_simple_router(|route| {
        route.get("/").to(homepage);
        route.get("/robots.txt").to(robots);
        route
            .get("/static/:name")
            .with_path_extractor::<FilePath>()
            .to(static_file);
    })
}

fn homepage(state: State) -> (State, Response) {
    state.html(|o| page(o, &[("first", 3), ("second", 7), ("third", 2)]))
}

fn footer(out: &mut Write) -> io::Result<()> {
    templates::footer(out, &[
        ("ructe", "https://crates.io/crates/ructe"),
        ("gotham", "https://gotham.rs/"),
    ])
}

fn robots(state: State) -> (State, Response) {
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((b"".to_vec(), mime::TEXT_PLAIN)),
    );
    (state, res)
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct FilePath {
    pub name: String,
}

static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);

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

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
