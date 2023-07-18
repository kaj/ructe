use std::io::{self, Write};

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
use templates::*;

fn main() {
    println!("### Page:");
    page_html(&mut io::stdout()).unwrap();
    for s in statics::STATICS {
        println!("### /static/{} is {}:", s.name, s.mime);
        io::stdout().write_all(s.content).unwrap();
    }
}

#[test]
fn test_page_w_static() {
    assert_eq!(
        r2s(|o| page_html(o)),
        "<html>\n  \
         <head>\n    \
         <title>Example with stylesheet</title>\n    \
         <link rel=\"stylesheet\" href=\"/static/style-o2rFo1lI.css\" \
         type=\"text/css\"/>\n  \
         </head>\n  \
         <body>\n    \
         Hello world!\n  \
         </body>\n\
         </html>\n"
    );
}

#[test]
fn test_static_css_data() {
    use std::str::from_utf8;
    use templates::statics::style_css;
    assert_eq!(
        from_utf8(&style_css.content).unwrap(),
        "body {\n    background: white;\n    color: black;\n}\n"
    );
}

#[test]
fn test_get_static_by_name() {
    use templates::statics::StaticFile;
    assert_eq!(
        StaticFile::get("style-o2rFo1lI.css").map(|s| s.name),
        Some("style-o2rFo1lI.css")
    )
}

#[test]
fn test_get_static_unknown() {
    use templates::statics::StaticFile;
    assert_eq!(StaticFile::get("foo-bar.css").map(|s| s.name), None)
}

#[test]
fn test_all_statics_known() {
    use templates::statics::STATICS;
    assert_eq!(
        STATICS.iter().map(|s| s.name).collect::<Vec<_>>(),
        [
            "17-C0lZWdZX.css",
            "foo-JckCHvyv.css",
            "foo-R-7hhHLr.js",
            "style-o2rFo1lI.css"
        ]
    );
}

/// Test for issue #82.
#[test]
fn test_num_gets_prefixed() {
    // The rust name is usable in rust.
    let style = &templates::statics::n17_css;
    // Url name is not prefixed (but has a content hash).
    assert_eq!(style.name, "17-C0lZWdZX.css");
}

#[cfg(test)]
fn r2s<Call>(call: Call) -> String
where
    Call: FnOnce(&mut Vec<u8>) -> io::Result<()>,
{
    let mut buf = Vec::new();
    call(&mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}
