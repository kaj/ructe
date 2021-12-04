use std::io;
use std::str::from_utf8;

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
use crate::templates::*;

fn main() {
    println!("### Page:");
    page(&mut io::stdout()).unwrap();
    for s in statics::STATICS {
        println!(
            "### /static/{}:\n{}",
            s.name,
            from_utf8(s.content).unwrap_or("(non-utf8 content)"),
        );
    }
}

#[cfg(test)]
mod test {
    use crate::templates::statics::{StaticFile, STATICS};
    use crate::templates::*;
    use std::io;

    #[test]
    fn page_w_static() {
        assert_eq!(
            r2s(|o| page(o)),
            "<html>\n  \
             <head>\n    \
             <title>Example with stylesheet</title>\n    \
             <link rel=\"stylesheet\" \
             href=\"/static/style-uNrEkqKN.css\" \
             type=\"text/css\"/>\n  \
             </head>\n  \
             <body>\n    \
             Hello world!\n  \
             </body>\n\
             </html>\n"
        );
    }

    #[test]
    fn static_css_data() {
        use crate::templates::statics::style_css;
        use std::str::from_utf8;
        assert_eq!(
            from_utf8(&style_css.content).unwrap(),
            "\u{feff}body{background:\"burlap-oPfjAg2n.jpg\"}\
             greeting{hello:wörld}\n"
        );
    }

    #[test]
    fn get_static_by_name() {
        assert_eq!(
            StaticFile::get("style-uNrEkqKN.css").map(|s| s.name),
            Some("style-uNrEkqKN.css")
        )
    }

    #[test]
    fn get_static_unknown() {
        assert_eq!(StaticFile::get("style-bar.css").map(|s| s.name), None)
    }

    #[test]
    fn all_statics_known() {
        assert_eq!(
            STATICS.iter().map(|s| s.name).collect::<Vec<_>>(),
            ["burlap-oPfjAg2n.jpg", "style-uNrEkqKN.css"]
        );
    }

    fn r2s<Call>(call: Call) -> String
    where
        Call: FnOnce(&mut Vec<u8>) -> io::Result<()>,
    {
        let mut buf = Vec::new();
        call(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}
