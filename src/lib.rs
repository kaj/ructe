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
use template::template;


pub fn compile_templates(indir: &Path, outdir: &Path) -> io::Result<()> {
    File::create(outdir.join("templates.rs")).and_then(|mut f| {
        try!(write!(f,
                    "mod templates {{\n\
                     use std::io::{{self, Write}};\n\
                     use std::fmt::Display;\n\n"));

        let outdir = outdir.join("templates");
        try!(create_dir_all(&outdir));
        try!(handle_entries(&mut f, indir, &outdir));
        write!(f,
               "{}\n}}\n",
               include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                                    "/src/template_utils.rs")))
    })
}

fn handle_entries(f: &mut Write,
                  indir: &Path,
                  outdir: &Path)
                  -> io::Result<()> {
    let suffix = ".rs.html";
    for entry in try!(read_dir(indir)) {
        let entry = try!(entry);
        let path = entry.path();
        if try!(entry.file_type()).is_dir() {
            if let Some(filename) = entry.file_name().to_str() {
                let outdir = outdir.join(filename);
                try!(create_dir_all(&outdir));
                try!(File::create(outdir.join("mod.rs"))
                     .and_then(|mut f| handle_entries(&mut f, &path, &outdir)));
                try!(write!(f, "pub mod {name};\n\n", name=filename));
            }

        } else if let Some(filename) = entry.file_name().to_str() {
            if filename.ends_with(suffix) {
                println!("cargo:rerun-if-changed={}",
                         path.to_string_lossy());
                let name = &filename[..filename.len() - suffix.len()];
                if try!(handle_template(name, &path, &outdir)) {
                    try!(write!(f,
                                "mod template_{name};\npub use \
                                 self::template_{name}\
                                 ::{name};\n\n",
                                name = name));
                }
            }
        }
    }
    Ok(())
}

fn handle_template(name: &str, path: &Path, outdir: &Path) -> io::Result<bool> {
    let mut input = try!(File::open(path));
    let mut buf = Vec::new();
    try!(input.read_to_end(&mut buf));
    match template(&buf) {
        Done(_, t) => {
            let fname = outdir.join(format!("template_{}.rs", name));
            try!(File::create(fname)
                 .and_then(|mut f| t.write_rust(&mut f, name)));
            Ok(true)
        }
        Error(err) => {
            println!("cargo:warning=\
                      Template parse error in {:?}: {}",
                     path,
                     err);
            Ok(false)
        }
        Incomplete(needed) => {
            println!("cargo:warning=\
                      Failed to parse template {:?}: \
                      {:?} needed",
                     path,
                     needed);
            Ok(false)
        }
    }
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
