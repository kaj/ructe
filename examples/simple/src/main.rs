#![allow(dead_code)] // Most templates here are only used in tests.
use std::io::{self, Write};

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
use templates::*;

fn main() {
    page::page(&mut io::stdout(), "sample page").unwrap();
}

fn r2s<Call>(call: Call) -> String
    where Call: FnOnce(&mut Write) -> io::Result<()>
{
    let mut buf = Vec::new();
    call(&mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

#[test]
fn test_hello() {
    assert_eq!(r2s(|o| hello(o)), "<h1>Hello World!</h1>\n");
}

#[test]
fn test_hello_args() {
    assert_eq!(r2s(|o| hello_args(o, "World")), "<h1>Hello World!</h1>\n");
}

#[test]
fn test_hello_encodes_args() {
    assert_eq!(r2s(|o| hello_args(o, "encoded < & >")),
               "<h1>Hello encoded &lt; &amp; &gt;!</h1>\n");
}

#[test]
fn test_hello_args_two() {
    assert_eq!(r2s(|o| hello_args_two(o, 56, "prime".to_string(), false)),
               "<p class=\"foo\" data-n=\"56\">Is 56 a prime? false!</p>\n");
}

#[test]
fn test_list() {
    assert_eq!(r2s(|o| list(o, &["foo", "bar"])),
               "<ul>\n  \n    <li>foo</li>\n  \n    <li>bar</li>\n  </ul>\n");
}

#[test]
fn test_uselist() {
    assert_eq!(r2s(|o| uselist(o)),
               "<h1>Two items</h1>\n\
                <ul>\n  \n    <li>foo</li>\n  \
                \n    <li>bar</li>\n  </ul>\n\n\
                <h2>No items</h2>\n\
                <ul>\n  </ul>\n\n");
}

#[test]
fn test_hello_utf8() {
    assert_eq!(r2s(|o| hello_utf8(o, "δ", "ε", "δ < ε", "δ &lt; ε")),
               "<p>δ &lt; ε</p>\n\
                <p>δ &lt; ε</p>\n\
                <p>δ &lt; ε</p>\n\
                <p>δ &lt; ε</p>\n");
}

mod models {
    use std::fmt;
    use templates::Html;

    pub struct User<'a> {
        pub name: &'a str,
        pub email: &'a str,
    }
    impl<'a> User<'a> {
        pub fn mailto(&self) -> Html<String> {
            Html(format!("<a href=\"mailto:{0}\">{0}</a>", self.email))
        }
    }
    impl<'a> fmt::Display for User<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.name)
        }
    }
}

#[test]
fn test_hello_fields() {
    let user = models::User {
        name: "Tom Puss",
        email: "tom@example.nl",
    };
    assert_eq!(r2s(|o| hello_fields(o, &user)),
               "<h1>Hello Tom Puss!</h1>\n<p>Your email is \
                tom@example.nl</p>\n");
}

#[test]
fn test_hello_method() {
    let user = models::User {
        name: "Tom Puss",
        email: "tom@example.nl",
    };
    assert_eq!(r2s(|o| hello_method(o, &user)),
               "<h1>Hello Tom Puss!</h1>\n<p>Your email is \
                <a href=\"mailto:tom@example.nl\">tom@example.nl</a></p>\n");
}

#[test]
fn test_hello_code() {
    use templates::Html;
    assert_eq!(r2s(|o| hello_code(o, &"Paragraph:", &Html("<p>Hello.</p>"))),
               "<h2>Paragraph:</h2>\n<p>Hello.</p>\n");
}

#[test]
fn test_for_loop() {
    assert_eq!(r2s(|o| for_loop(o, &vec!["Hello", "World"])),
               "<h1>Looped paragraphs</h1>\n\n  <p>Hello</p>\n\n  <p>World</p>\n");
}

#[test]
fn test_explicit_formatting() {
    assert_eq!(r2s(|o| explicit_formatting(o, 5.212432234, "one\ntwo")),
               "<p>Value 1 is 5.2 (or really 5.212432234),\n\
                while value 2 is \"one\\ntwo\".</p>\n");
}

#[test]
fn test_hello_use_templates() {
    assert_eq!(r2s(|o| hello_use_templates(o, &Html("<p>this is foo</p>"))),
               "<h1>Hello World!</h1>\n\n\
                <h2>foo</h2>\n<p>this is foo</p>\n\n");
}

#[test]
fn test_page_with_base() {
    assert_eq!(r2s(|o| page::page(o, "World")),
               "<!doctype html>\n\
                <html>\n  \
                <head><title>Hello World!</title></head>\n  \
                <body>\n    \
                <h1>Hello World!</h1>\n    \n  \
                <p>This is page content for World</p>\n\n  \
                </body>\n\
                </html>\n\n");
}
