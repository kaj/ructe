#![allow(dead_code)] // Most templates here are only used in tests.

use std::io;

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
use crate::templates::*;

fn main() {
    page::page_html(&mut io::stdout(), "sample page").unwrap();
}

fn r2s<Call>(call: Call) -> String
where
    Call: FnOnce(&mut Vec<u8>) -> io::Result<()>,
{
    let mut buf = Vec::new();
    call(&mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

#[test]
fn test_hello() {
    assert_eq!(
        r2s(|o| hello_html(o)),
        "<h1>Hello World!</h1>\
         \n<p>Note: Brackets and @ signs needs to be escaped: { ... }</p>\n"
    );
}

#[test]
fn test_hello_args() {
    assert_eq!(
        r2s(|o| hello_args_html(o, "World")),
        "<h1>Hello World!</h1>\n",
    );
}

#[test]
fn test_hello_encodes_args() {
    assert_eq!(
        r2s(|o| hello_args_html(o, "encoded < & >")),
        "<h1>Hello encoded &lt; &amp; &gt;!</h1>\n"
    );
}

#[test]
fn test_hello_args_two() {
    assert_eq!(
        r2s(|o| hello_args_two_html(o, 56, "prime", false)),
        "<p class=\"foo\" data-n=\"56\">Is 56 a prime? false!</p>\n"
    );
}

#[test]
fn test_hello_args_three() {
    assert_eq!(
        r2s(|o| hello_args_three_html(o, 56, &56, &56)),
        "<p>56 56 56</p>\n"
    );
}

#[test]
fn test_if_let_some() {
    assert_eq!(
        r2s(|o| if_let_html(o, Some("thing"))),
        "<p> The item is thing </p>\n"
    )
}
#[test]
fn test_if_let_none() {
    assert_eq!(r2s(|o| if_let_html(o, None)), "<p> Got nothing </p>\n")
}

#[test]
fn test_if_let_destructure() {
    assert_eq!(
        r2s(|o| if_let_destructure_html(o, &Some((47, 11)))),
        "<p> We have 47 and 11 </p>\n"
    )
}

#[test]
fn test_if_else_if() {
    let f = |n| r2s(|o| if_else_if_html(o, n));
    assert_eq!(f(0), "There are none\n");
    assert_eq!(f(1), "There is one\n");
    assert_eq!(f(7), "There are 7\n");
}

#[test]
fn test_list() {
    assert_eq!(
        r2s(|o| list_html(o, &["foo", "bar"])),
        "\n<ul>\n  \n    <li>foo</li>\n  \n    <li>bar</li>\n  \n</ul>\n\n"
    );
}

#[test]
fn test_list_empty() {
    assert_eq!(r2s(|o| list_html(o, &[])), "\n<p>No items</p>\n\n");
}

#[test]
fn test_list_destructure() {
    assert_eq!(
        r2s(|o| list_destructure_html(o, &["foo", "bar"])),
        "<ul>\n  \n    <li>0: foo</li>\n  \n    \
         <li>1: bar</li>\n  \n</ul>\n"
    );
}

#[test]
fn test_list_destructure_2() {
    assert_eq!(
        r2s(|o| list_destructure_2_html(o)),
        "\n    <p>Rasmus is 44 years old.</p>\n\n    \
         <p>Mike is 36 years old.</p>\n\n"
    );
}

#[test]
fn test_uselist() {
    assert_eq!(
        r2s(|o| uselist_html(o)),
        "<h1>Two items</h1>\n\n\
         <ul>\n  \n    <li>foo</li>\n  \
         \n    <li>bar</li>\n  \n</ul>\n\n\n\
         <h2>No items</h2>\n\n\
         <p>No items</p>\n\n\n"
    );
}

#[test]
fn test_hello_utf8() {
    assert_eq!(
        r2s(|o| hello_utf8_html(o, "δ", "ε", "δ < ε", "δ &lt; ε")),
        "<p>δ &lt; ε</p>\n\
         <p>δ &lt; ε</p>\n\
         <p>δ &lt; ε</p>\n\
         <p>δ &lt; ε</p>\n"
    );
}

#[test]
fn test_comments() {
    assert_eq!(
        r2s(|o| comments_html(o)),
        "<!-- this is a real HTML comment, which gets send to the client -->\n\
         <p>This is visible</p>\n\n"
    );
}

mod models {
    use crate::templates::Html;
    use std::fmt;

    pub struct User<'a> {
        pub name: &'a str,
        pub email: &'a str,
    }
    impl User<'_> {
        pub fn mailto(&self) -> Html<String> {
            Html(format!("<a href=\"mailto:{0}\">{0}</a>", self.email))
        }
    }
    impl fmt::Display for User<'_> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str(self.name)
        }
    }
}

#[test]
fn test_hello_fields() {
    let user = models::User {
        name: "Tom Puss",
        email: "tom@example.nl",
    };
    assert_eq!(
        r2s(|o| hello_fields_html(o, &user)),
        "<h1>Hello Tom Puss!</h1>\n<p>Your email is \
         tom@example.nl</p>\n"
    );
}

#[test]
fn test_hello_method() {
    let user = models::User {
        name: "Tom Puss",
        email: "tom@example.nl",
    };
    assert_eq!(
        r2s(|o| hello_method_html(o, &user)),
        "<h1>Hello Tom Puss!</h1>\n<p>Your email is \
         <a href=\"mailto:tom@example.nl\">tom@example.nl</a></p>\n"
    );
}

#[test]
fn test_hello_code() {
    use templates::Html;
    assert_eq!(
        r2s(|o| hello_code_html(o, &"Paragraph:", &Html("<p>Hello.</p>"))),
        "<h2>Paragraph:</h2>\n<p>Hello.</p>\n"
    );
}

#[test]
fn test_for_loop() {
    assert_eq!(
        r2s(|o| for_loop_html(o, &vec!["Hello", "World"])),
        "<h1>Looped paragraphs</h1>\n\n  \
         <p>Hello</p>\n\n  <p>World</p>\n\n"
    );
}

#[test]
fn test_for_destructure() {
    let users = vec![
        models::User {
            name: "Tom Puss",
            email: "tom@example.nl",
        },
        models::User {
            name: "Heloise Walker",
            email: "helwa@briarson.edu",
        },
    ];
    assert_eq!(
        r2s(|o| for_destructure_html(o, &users)),
        "<ul><li>Tom Puss</li><li>Heloise Walker</li></ul>\n",
    )
}

#[test]
fn test_explicit_formatting() {
    assert_eq!(
        r2s(|o| explicit_formatting_html(o, 5.212432234, "one\ntwo")),
        "<p>Value 1 is 5.2 (or really 5.212432234),\n\
         while value 2 is &quot;one\\ntwo&quot;.</p>\n"
    );
}

#[test]
fn test_hello_use_templates() {
    assert_eq!(
        r2s(|o| hello_use_templates_html(o, &Html("<p>this is foo</p>"))),
        "<h1>Hello World!</h1>\
         \n<p>Note: Brackets and @ signs needs to be escaped: { ... }</p>\n\
         \n<h2>foo</h2>\n<p>this is foo</p>\n\n"
    );
}

#[test]
fn test_page_with_base() {
    assert_eq!(
        r2s(|o| page::page_html(o, "World")),
        "<!doctype html>\
         \n<html>\
         \n  <head><title>Hello World!</title>\n\
         \n  <meta property=\"og:description\" content=\"A simple example\"/>\n\
         \n</head>\
         \n  <body>\
         \n    <h1>Hello World!</h1>\
         \n    \n<div>\
         \n  \
         \n  <p>This is page content for World</p>\n\
         \n</div>\n<footer>A footer common to some pages.</footer>\
         \n\n  </body>\
         \n</html>\n\n\n"
    );
}

#[test]
fn test_page_two() {
    assert_eq!(
        r2s(|o| page::page_two_html(o, 2019)),
        "<!doctype html>\
         \n<html lang=\"sv\">\
         \n  <head>\
         \n    <title>Year 2019 - Example page</title>\
         \n  </head>\
         \n  <body>\
         \n    <header>\
         \n      <h1>Year 2019</h1>\
         \n      \n  <p>Welcome to this page about the year 2019.</p>\n\
         \n    </header>\n\
         \n    <main>\
         \n  <p>This is the main page content.</p>\
         \n</main>\n\
         \n    <footer>\
         \n      <p>A simple example page.</p>\
         \n    </footer>\
         \n  </body>\
         \n</html>\n\n",
    )
}

#[test]
fn test_some_expressions() {
    assert_eq!(
        r2s(|o| some_expressions_html(o, "name")),
        "<p>name</p>\
         \n<p>name.name</p>\
         \n<p>4</p>\
         \n<p>name.len()</p>\
         \n<p>-1</p>\
         \n<p>1</p>\n"
    )
}

#[test]
fn test_issue_66() {
    assert_eq!(r2s(|o| issue_66_html(o)), "ABC\n");
}

#[test]
fn test_issue_68() {
    assert_eq!(
        r2s(|o| issue_68_html(o)),
        "Hello!\n\nThe 0 number.\n\nThe 1 number.\n\nThe 2 number.\n\nGood bye!\n",
    );
}

/// [Issue #106](https://github.com/kaj/ructe/issues/106)
#[test]
fn lifetimes() {
    assert_eq!(
        r2s(|o| with_lifetime_html(o, &["foo", "bar"])),
        "\n  <p>foo</p>\n\n  <p>bar</p>\n\n",
    );
}

/// [Issue #106](https://github.com/kaj/ructe/issues/106)
#[test]
fn lifetimes2() {
    assert_eq!(
        r2s(|o| with_lifetime2_html(o, &["foo", "bar"])),
        "\n  <p>foo</p>\n\n  <p>bar</p>\n\n",
    );
}

#[test]
fn test_list_join() {
    assert_eq!(
        r2s(|o| list_joins_html(o, &[2, 3, 7])),
        "<p>Items: 2, 3, 7.</p>\n",
    )
}
