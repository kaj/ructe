#[macro_use]
extern crate nom;

mod spacelike;
mod expression;
mod templateexpression;
mod template;

use nom::IResult::*;
use std::fs::{File, create_dir_all, read_dir};
use std::io::{self, Read, Write};
use std::path::Path;
use std::str::from_utf8;
use template::template;


pub fn compile_templates(indir: &Path, outdir: &Path) -> io::Result<()> {
    let suffix = ".rs.html";

    File::create(outdir.join("templates.rs")).and_then(|mut f| {
        try!(write!(f,
                    "mod templates {{\n\
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
                            let fname =
                                outdir.join(format!("template_{}.rs", name));
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
                                     e,
                                     path,
                                     from_utf8(pos).unwrap())
                        }
                        Error(err) => {
                            println!("cargo:warning=\
                                      Template parse error in {:?}: {}",
                                     path,
                                     err)
                        }
                        Incomplete(needed) => {
                            println!("cargo:warning=\
                                      Failed to parse template {:?}: \
                                      {:?} needed",
                                     path,
                                     needed)
                        }
                    }
                }
            }
        }
        write!(f,
               "{}\n}}\n",
               include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                                    "/src/template_utils.rs")))
    })
}

#[cfg(test)]
mod template_utils_test {
    use std::fmt::Display;
    use std::io::{self, Write};
    include!("template_utils.rs");

    #[test]
    fn encoded() {
        let mut buf = Vec::new();
        "a < b".to_html(&mut buf).unwrap();
        assert_eq!(b"a &lt; b", &buf[..]);
    }
    #[test]
    fn raw_html() {
        let mut buf = Vec::new();
        Html("a<b>c</b>").to_html(&mut buf).unwrap();
        assert_eq!(b"a<b>c</b>", &buf[..]);
    }
}
