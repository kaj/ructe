use std::io;
use std::str::from_utf8;

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
use templates::*;

fn main() {
    hello_use_templates(&mut io::stdout(), &Html("<p>this is foo</p>"))
        .unwrap();
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
    assert_eq!(from_utf8(&buf).unwrap(),
               "<p class=\"foo\" data-n=\"56\">Is 56 a prime? false!</p>\n");
}

#[test]
fn test_hello_utf8() {
    let mut buf = Vec::new();
    hello_utf8(&mut buf, "δ", "ε", "δ < ε", "δ &lt; ε").unwrap();
    assert_eq!(from_utf8(&buf).unwrap(),
               "<p>δ &lt; ε</p>\n\
                <p>δ &lt; ε</p>\n\
                <p>δ &lt; ε</p>\n\
                <p>δ &lt; ε</p>\n");
}

mod models {
    use std::fmt;
    use templates::Html;

    #[allow(dead_code)]
    pub struct User<'a> {
        pub name: &'a str,
        pub email: &'a str,
    }
    impl<'a> User<'a> {
        #[allow(dead_code)]
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
    let mut buf = Vec::new();
    hello_fields(&mut buf, &user).unwrap();
    assert_eq!(from_utf8(&buf).unwrap(),
               "<h1>Hello Tom Puss!</h1>\n<p>Your email is \
                tom@example.nl</p>\n");
}

#[test]
fn test_hello_method() {
    let user = models::User {
        name: "Tom Puss",
        email: "tom@example.nl",
    };
    let mut buf = Vec::new();
    hello_method(&mut buf, &user).unwrap();
    assert_eq!(from_utf8(&buf).unwrap(),
               "<h1>Hello Tom Puss!</h1>\n<p>Your email is \
                <a href=\"mailto:tom@example.nl\">tom@example.nl</a></p>\n");
}

#[test]
fn test_hello_code() {
    use templates::Html;
    let mut buf = Vec::new();
    hello_code(&mut buf, &"Paragraph:", &Html("<p>Hello.</p>")).unwrap();
    assert_eq!(from_utf8(&buf).unwrap(),
               "<h2>Paragraph:</h2>\n<p>Hello.</p>\n");
}

#[test]
fn test_for_loop() {
    let mut buf = Vec::new();
    for_loop(&mut buf, &vec!["Hello", "World"]).unwrap();
    assert_eq!(from_utf8(&buf).unwrap(),
               "<h1>Looped paragraphs</h1>\n<p>Hello</p>\n<p>World</p>\n\n");
}

#[test]
fn test_explicit_formatting() {
    let mut buf = Vec::new();
    explicit_formatting(&mut buf, 5.212432234, "one\ntwo").unwrap();
    assert_eq!(from_utf8(&buf).unwrap(),
               "<p>Value 1 is 5.2 (or really 5.212432234),\n\
                while value 2 is \"one\\ntwo\".</p>\n");
}

#[test]
fn test_hello_use_templates() {
    let mut buf = Vec::new();
    hello_use_templates(&mut buf, &Html("<p>this is foo</p>")).unwrap();
    assert_eq!(from_utf8(&buf).unwrap(),
               "<h1>Hello World!</h1>\n\n\
                <h2>foo</h2>\n<p>this is foo</p>\n\n");
}

#[test]
fn test_page_with_base() {
    let mut buf = Vec::new();
    page_page(&mut buf, "World").unwrap();
    assert_eq!(from_utf8(&buf).unwrap(),
               "<html>\n  \
                <head><title>Hello World!</title></head>\n  \
                <body>\n    \
                <h1>Hello World!</h1>\n    \n  \
                <p>This is page content for World</p>\n\n  \
                </body>\n\
                </html>\n\n");
}
