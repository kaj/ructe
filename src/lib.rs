#[macro_use]
extern crate nom;

mod spacelike;
use spacelike::spacelike;
mod expression;
mod templateexpression;
use templateexpression::{TemplateExpression, template_expression};

use nom::eof;
use nom::IResult::*;
use std::fs::{File, create_dir_all, read_dir};
use std::io::{self, Read, Write};
use std::path::Path;
use std::str::from_utf8;

#[derive(Debug, PartialEq, Eq)]
struct Template {
    preamble: Vec<String>,
    args: Vec<String>,
    body: Vec<TemplateExpression>,
}

impl Template {
    fn write_rust(&self, out: &mut Write, name: &str) -> io::Result<()> {
        write!(out,
               "use std::io::{{self, Write}};\n\
                #[allow(unused)]\n\
                use ::templates::{{Html,ToHtml}};\n\
                {preamble}\n\
                pub fn {name}{type_args}(out: &mut Write{args})\n\
                -> io::Result<()> {type_spec}{{\n\
                {body}\
                Ok(())\n\
                }}\n",
               preamble = self.preamble
                   .iter()
                   .map(|l| format!("{};\n", l))
                   .collect::<String>(),
               name = name,
               type_args = self.args
                   .iter()
                   .filter(|a| a.as_str() == "content: Content")
                   .map(|_a| format!("<Content>"))
                   .collect::<String>(),
               args = self.args
                   .iter()
                   .map(|a| format!(", {}", a))
                   .collect::<String>(),
               type_spec = self.args
                   .iter()
                   .filter(|a| a.as_str() == "content: Content")
                   .map(|_a| {
                       format!("\nwhere Content: FnOnce(&mut Write) \
                                -> io::Result<()>")
                   })
                   .collect::<String>(),
               body = self.body
                   .iter()
                   .map(|b| b.code())
                   .collect::<String>())
    }
}

named!(template<&[u8], Template>,
       chain!(
           spacelike ~
           preamble: many0!(chain!(tag!("@") ~
                                   code: is_not!(";()") ~
                                   tag!(";") ~
                                   spacelike,
                                   ||from_utf8(code).unwrap().to_string()
                                   )) ~
           tag!("@(") ~
           args: separated_list!(tag!(", "), formal_argument) ~
           tag!(")") ~
           spacelike ~
           body: many0!(template_expression) ~
           eof,
           || { Template { preamble: preamble, args: args, body: body } }
           )
);

// TODO Actually parse arguments!
named!(formal_argument<&[u8], String>,
       chain!(
           raw: is_not!(",)"),
           || from_utf8(raw).unwrap().to_string()
               )
       );




pub fn compile_templates(indir: &Path, outdir: &Path) -> io::Result<()> {
    let suffix = ".rs.html";

    File::create(outdir.join("templates.rs")).and_then(|mut f| {
        try!(write!(f, "mod templates {{\n\
                        use std::io::{{self, Write}};\n\
                        use std::fmt::Display;\n\n"));

        let outdir = outdir.join("templates");
        try!(create_dir_all(&outdir));

        for entry in try!(read_dir(indir)) {
            let entry = try!(entry);
            let path = entry.path();
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(suffix) {
                    println!("cargo:rerun-if-changed={}",
                             path.to_string_lossy());
                    let name = &filename[..filename.len() - suffix.len()];
                    let mut input = try!(File::open(&path));
                    let mut buf = Vec::new();
                    try!(input.read_to_end(&mut buf));
                    match template(&buf) {
                        Done(_, t) => {
                            let fname = outdir.join(format!("template_{}.rs",
                                                            name));
                            try!(File::create(fname)
                                 .and_then(|mut f| t.write_rust(&mut f, name)));
                            try!(write!(f,
                                        "mod template_{name};\n\
                                         pub use ::templates::template_{name}\
                                         ::{name};\n\n",
                                        name = name));
                        }
                        Error(nom::Err::Position(e, pos)) => {
                            println!("cargo:warning=\
                                      Template parse error {:?} in {:?}: {:?}",
                                     e, path, from_utf8(pos).unwrap())
                        }
                        Error(err) => {
                            println!("cargo:warning=\
                                      Template parse error in {:?}: {}",
                                     path, err)
                        }
                        Incomplete(needed) => {
                            println!("cargo:warning=\
                                      Failed to parse template {:?}: \
                                      {:?} needed",
                                     path, needed)
                        }
                    }
                }
            }
        }
        write!(f, "{}\n}}\n", include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                                                   "/src/template_utils.rs")))
    })
}

mod foo {
    use std::fmt::Display;
    use std::io::{self, Write};
    include!("template_utils.rs");

    #[test]
    fn test_encoded() {
        let mut buf = Vec::new();
        "a < b".to_html(&mut buf).unwrap();
        assert_eq!(b"a &lt; b", &buf[..]);
    }
    #[test]
    fn test_raw_html() {
        let mut buf = Vec::new();
        Html("a<b>c</b>").to_html(&mut buf).unwrap();
        assert_eq!(b"a<b>c</b>", &buf[..]);
    }
}
