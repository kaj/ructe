#![allow(dead_code)] // Most templates here are only used in tests.

use std::io::{self, Write};

include!(concat!(env!("OUT_DIR"), "/pages.rs"));

fn main() {
    pages::pi(&mut io::stdout()).unwrap();
    pages::max::index(&mut io::stdout()).unwrap();
}

fn r2s<Call>(call: Call) -> String
    where Call: FnOnce(&mut Write) -> io::Result<()>
{
    let mut buf = Vec::new();
    call(&mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

#[test]
fn test_pi() {
    assert_eq!(r2s(|o| pages::pi(o)), "PI is 3.1415927\n");
}

#[test]
fn test_sublevel() {
    assert_eq!(r2s(|o| pages::max::index(o)), "MAX is 65535 and PI is 3.1415927\n");
}
