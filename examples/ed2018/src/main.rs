use std::io::{self, Write};

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
use self::templates::*;

fn main() {
    println!("### Page:");
    page(&mut io::stdout()).unwrap();
    for s in statics::STATICS {
        println!("### /static/{} is {}:", s.name, s.mime);
        io::stdout().write_all(s.content).unwrap();
    }
}

#[test]
fn test_page_w_static() {
    assert_eq!(
        r2s(|o| page(o)),
        "<html>\n  \
         <head>\n    \
         <title>Example with stylesheet</title>\n    \
         <link rel=\"stylesheet\" href=\"/static/style-BeQlLiwh.css\" \
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
    use self::templates::statics::style_css;
    use std::str::from_utf8;
    assert_eq!(
        from_utf8(&style_css.content).unwrap(),
        "body{background:white;color:#efefef}\n"
    );
}

#[test]
fn test_get_static_by_name() {
    use self::templates::statics::StaticFile;
    assert_eq!(
        StaticFile::get("style-BeQlLiwh.css").map(|s| s.name),
        Some("style-BeQlLiwh.css")
    )
}

#[test]
fn test_get_static_unknown() {
    use self::templates::statics::StaticFile;
    assert_eq!(StaticFile::get("foo-bar.css").map(|s| s.name), None)
}

#[test]
fn test_all_statics_known() {
    use self::templates::statics::STATICS;
    assert_eq!(
        STATICS.iter().map(|s| s.name).collect::<Vec<_>>(),
        ["foo-JckCHvyv.css", "foo-R-7hhHLr.js", "style-BeQlLiwh.css"]
    );
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
