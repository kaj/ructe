use std::io;
use std::str::from_utf8;

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
use templates::*;

fn main() {
    println!("### Page:");
    page(&mut io::stdout()).unwrap();
    for s in statics::STATICS {
        println!("### /static/{}:\n{}",
                 s.name,
                 from_utf8(s.content).unwrap_or("(non-utf8 content)"));
    }
}

#[cfg(test)]
mod test {
    use std::io::{self, Write};
    use templates::*;

    #[test]
    fn page_w_static() {
        assert_eq!(r2s(|o| page(o)),
                   "<html>\n  \
                    <head>\n    \
                    <title>Example with stylesheet</title>\n    \
                    <link rel=\"stylesheet\" \
                    href=\"/static/style-dp91gNUn.css\" \
                    type=\"text/css\"/>\n  \
                    </head>\n  \
                    <body>\n    \
                    Hello world!\n  \
                    </body>\n\
                    </html>\n");
    }

    #[test]
    fn static_css_data() {
        use templates::statics::style_css;
        use std::str::from_utf8;
        assert_eq!(from_utf8(&style_css.content).unwrap(),
                   "body{background:\"burlap-oPfjAg2n.jpg\"}\
                    greeting{hello:world}\n");
    }

    #[test]
    fn get_static_by_name() {
        use templates::statics::StaticFile;
        assert_eq!(StaticFile::get("style-dp91gNUn.css").map(|s| s.name),
                   Some("style-dp91gNUn.css"))
    }

    #[test]
    fn get_static_unknown() {
        use templates::statics::StaticFile;
        assert_eq!(StaticFile::get("style-bar.css").map(|s| s.name), None)
    }

    #[test]
    fn all_statics_known() {
        use templates::statics::STATICS;
        assert_eq!(STATICS.iter().map(|s| s.name).collect::<Vec<_>>(),
                   ["burlap-oPfjAg2n.jpg", "style-dp91gNUn.css"]);
    }

    fn r2s<Call>(call: Call) -> String
        where Call: FnOnce(&mut Write) -> io::Result<()>
    {
        let mut buf = Vec::new();
        call(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}
