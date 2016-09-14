use std::io;
use std::str::from_utf8;

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
use templates::*;

fn main() {
    hello(&mut io::stdout()).unwrap();
}

#[test]
fn test_hello() {
    let mut buf = Vec::new();
    hello(&mut buf).unwrap();
    assert_eq!(from_utf8(&buf).unwrap(), "<h1>Hello World!</h1>\n");
}

#[test]
fn test_hello_args() {
    let mut buf = Vec::new();
    hello_args(&mut buf, "World").unwrap();
    assert_eq!(from_utf8(&buf).unwrap(), "<h1>Hello World!</h1>\n");
}

#[test]
fn test_hello_encodes_args() {
    let mut buf = Vec::new();
    hello_args(&mut buf, "encoded < & >").unwrap();
    assert_eq!(from_utf8(&buf).unwrap(),
               "<h1>Hello encoded &lt; &amp; &gt;!</h1>\n");
}

#[test]
fn test_hello_args_two() {
    let mut buf = Vec::new();
    hello_args_two(&mut buf, 56, "prime".to_string(), false).unwrap();
    assert_eq!(from_utf8(&buf).unwrap(), "<p>Is 56 a prime? false!</p>\n");
}
