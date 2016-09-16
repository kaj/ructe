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

mod models {
    use std::fmt;

    pub struct User<'a> {
        pub name: &'a str,
        pub email: &'a str,
    }
    impl<'a> fmt::Display for User<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.name)
        }
    }
}

#[test]
fn test_hello_fields() {
    let user = models::User { name: "Tom Puss", email: "tom@example.nl" };
    let mut buf = Vec::new();
    hello_fields(&mut buf, &user).unwrap();
    assert_eq!(from_utf8(&buf).unwrap(),
               "<h1>Hello Tom Puss!</h1>\n<p>Your email is tom@example.nl</p>\n");
}
