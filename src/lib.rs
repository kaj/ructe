//! Rust Compiled Templates is a HTML template system for Rust.
//!
//! Ructe works by converting your templates (and static files) to
//! rust source code, which is then compiled with your project.
//! This has the benefits that:
//!
//! 1. Many syntactical and logical errors in templates are caught
//! compile-time, rather than in a running server.
//! 2. No extra latency on the first request, since the template are
//! compiled before starting the program.
//! 3. The template files does not have to be distributed / installed.
//! Templates are included in the compiled program, which can be a
//! single binary.
//!
//! The template syntax, which is inspired by
//! [Twirl](https://github.com/playframework/twirl), the Scala-based
//! template engine in
//! [Play framework](https://www.playframework.com/),
//! is documented in [the _Template syntax_ module](Template_syntax/index.html).
//! A sample template may look like this:
//!
//! ```html
//! @use ::Group;
//! @use templates::page_base;
//!
//! @(title: &str, user: Option<String>, groups: &[Group])
//!
//! @:page_base(title, &user, {
//!   <div class="group">
//!     @if groups.is_empty() {
//!       <p>No pictures.</p>
//!     }
//!     @for g in groups {
//!       <div class="item"><h2>@g.title</h2>
//!         <p><a href="@g.url"><img src="/img/@g.photo.id-s.jpg"></a></p>
//!         <p>@g.count pictures</p>
//!       </div>
//!     }
//!   </div>
//! })
//! ```
//!
//! # How to use ructe
//!
//! Ructe compiles your templates to rust code that should be compiled with
//! your other rust code, so it needs to be called before compiling.
//! Assuming you use [cargo](http://doc.crates.io/), it can be done like
//! this:
//!
//! First, specify a build script and ructe as a build dependency in
//! `Cargo.toml`:
//!
//! ```toml
//! build = "src/build.rs"
//!
//! [build-dependencies]
//! ructe = "^0.2"
//! ```
//!
//! Then, in the build script, compile all templates found in the templates
//! directory and put the output where cargo tells it to:
//!
//! ```rust,no_run
//! extern crate ructe;
//!
//! use ructe::compile_templates;
//! use std::env;
//! use std::path::PathBuf;
//!
//! fn main() {
//!     let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
//!     let in_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
//!         .join("templates");
//!     compile_templates(&in_dir, &out_dir).expect("compile templates");
//! }
//! ```
//!
//! And finally, include and use the generated code in your code.
//! The file `templates.rs` will contain `mod templates { ... }`,
//! so I just include it in my `main.rs`:
//!
//! ```rust,ignore
//! include!(concat!(env!("OUT_DIR"), "/templates.rs"));
//! ```
//!
//! When calling a template, the arguments declared in the template will be
//! prepended by a `Write` argument to write the output to.
//! It can be a `Vec<u8>` as a buffer or for testing, or an actual output
//! destination.
//! The return value of a template is `std::io::Result<()>`, which should be
//! `Ok(())` unless writing to the destination fails.
//!
//! ```
//! #[test]
//! fn test_hello() {
//!     let mut buf = Vec::new();
//!     templates::hello(&mut buf, "World").unwrap();
//!     assert_eq!(buf, b"<h1>Hello World!</h1>\n");
//! }
//! ```

extern crate base64;
extern crate md5;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate lazy_static;

mod spacelike;
mod expression;
#[macro_use]
mod errors;
#[macro_use]
mod templateexpression;
mod template;
pub mod Template_syntax;

use errors::get_error;
use nom::{ErrorKind, prepare_errors};
use nom::IResult::*;
use std::collections::BTreeSet;
use std::fs::{File, create_dir_all, read_dir};
use std::io::{self, Read, Write};
use std::path::Path;
use std::str::from_utf8;
use template::template;

/// Create a `statics` module inside `outdir`, containing static file data
/// for all files in `indir`.
///
/// This must be called *before* `compile_templates`.
pub fn compile_static_files(indir: &Path, outdir: &Path) -> io::Result<()> {
    let mut out = StaticFiles::new(outdir)?;
    out.add_files(indir)
}

/// Handler for static files.
///
/// To just add all files in a single directory, there is a shorthand method
/// `compile_static_files`.
/// For more complex setups (static files in more than one directory,
/// generated static files, etc), use this struct.
///
/// Each static file will be available as a
/// [`StaticFile`](templates/statics/index.html) struct instance in
/// your `templates::statics` module.
/// Also, the const `STATICS` array in the same module will contain a
/// reference to each of those instances.
pub struct StaticFiles {
    src: File,
    statics: BTreeSet<String>,
}

impl StaticFiles {
    /// Create a new set of static files.
    ///
    /// There should only be one `StaticFiles` for a set of compiled templates.
    /// The `outdir` should be the same as in the call to `compile_templates`.
    pub fn new(outdir: &Path) -> io::Result<Self> {
        let outdir = outdir.join("templates");
        try!(create_dir_all(&outdir));
        let mut src = File::create(outdir.join("statics.rs"))?;
        write!(src,
               "{}\n",
               include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                                    "/src/statics_utils.rs")))?;
        Ok(StaticFiles { src: src, statics: BTreeSet::new() })
    }

    /// Add all files from a specific directory, `indir`, as static files.
    pub fn add_files(&mut self, indir: &Path) -> io::Result<()> {
        for entry in read_dir(indir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                self.add_file(&entry.path())?;
            }
        }
        Ok(())
    }

    /// Add one specific file as a static file.
    pub fn add_file(&mut self, path: &Path) -> io::Result<()> {
        if let Some((name, ext)) = name_and_ext(path) {
            println!("cargo:rerun-if-changed={}", path.to_string_lossy());
            let mut input = File::open(&path)?;
            let mut buf = Vec::new();
            input.read_to_end(&mut buf)?;

            self.write_static_file(path, name, &buf, ext)?;
            self.statics.insert(format!("{}_{}", name, ext));
        }
        Ok(())
    }

    fn write_static_file(&mut self,
                         path: &Path,
                         name: &str,
                         content: &[u8],
                         suffix: &str)
                         -> io::Result<()> {
        write!(self.src,
               "\n/// From {path:?}\n\
                #[allow(non_upper_case_globals)]\n\
                pub static {name}_{suf}: StaticFile = \
                StaticFile {{\n  \
                content: &{content:?},\n  \
                name: \"{name}-{hash}.{suf}\",\n\
                }};\n",
               path = path,
               name = name,
               content = content,
               hash = checksum_slug(&content),
               suf = suffix)
    }
}

impl Drop for StaticFiles {
    /// Write the ending of the statics source code, declaring the
    /// `STATICS` variable.
    fn drop(&mut self) {
        // Ignore a possible write failure, rather than a panic in drop.
        let _ = write!(self.src,
                       "\npub static STATICS: &'static [&'static StaticFile] \
                        = &[{}];\n",
                       self.statics
                           .iter()
                           .map(|s| format!("&{}", s))
                           .collect::<Vec<_>>()
                           .join(", "));
    }
}

fn name_and_ext(path: &Path) -> Option<(&str, &str)> {
    if let (Some(name), Some(ext)) = (path.file_name(), path.extension()) {
        if let (Some(name), Some(ext)) = (name.to_str(), ext.to_str()) {
            return Some((&name[..name.len() - ext.len() - 1], ext));
        }
    }
    None
}

/// A short and url-safe checksum string from string data.
fn checksum_slug(data: &[u8]) -> String {
    base64::encode_mode(&md5::compute(data)[..6], base64::Base64Mode::UrlSafe)
}


/// Create a `templates` module in `outdir` containing rust code for
/// all templates found in `indir`.
pub fn compile_templates(indir: &Path, outdir: &Path) -> io::Result<()> {
    File::create(outdir.join("templates.rs")).and_then(|mut f| {
        try!(write!(f,
                    "pub mod templates {{\n\
                     use std::io::{{self, Write}};\n\
                     use std::fmt::Display;\n\n"));

        let outdir = outdir.join("templates");
        try!(create_dir_all(&outdir));

        try!(handle_entries(&mut f, indir, &outdir));

        if outdir.join("statics.rs").exists() {
            try!(write!(f, "pub mod statics;\n"));
        }

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
                try!(write!(f, "pub mod {name};\n\n", name = filename));
            }

        } else if let Some(filename) = entry.file_name().to_str() {
            if filename.ends_with(suffix) {
                println!("cargo:rerun-if-changed={}", path.to_string_lossy());
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
            File::create(fname).and_then(|mut f| t.write_rust(&mut f, name))?;
            Ok(true)
        }
        result => {
            println!("cargo:warning=\
                      Template parse error in {:?}:",
                     path);
            show_errors(&mut io::stdout(), &buf, result, "cargo:warning=");
            Ok(false)
        }
    }
}

fn show_errors<E>(out: &mut Write,
                  buf: &[u8],
                  result: nom::IResult<&[u8], E>,
                  prefix: &str) {
    if let Some(errors) = prepare_errors(buf, result) {
        for &(ref kind, ref from, ref _to) in &errors {
            show_error(out, buf, *from, &get_message(kind), prefix);
        }
    }
}

fn get_message(err: &ErrorKind) -> String {
    match err {
        &ErrorKind::Custom(n) => {
            match get_error(n) {
                Some(msg) => msg,
                None => format!("Unknown error #{}", n),
            }
        }
        err => format!("{:?}", err),
    }
}

fn show_error(out: &mut Write,
              buf: &[u8],
              pos: usize,
              msg: &str,
              prefix: &str) {
    let mut line_start = buf[0..pos].rsplitn(2, |c| *c == b'\n');
    let _ = line_start.next();
    let line_start =
        line_start.next().map(|bytes| bytes.len() + 1).unwrap_or(0);
    let line = buf[line_start..]
        .splitn(2, |c| *c == b'\n')
        .next()
        .and_then(|s| from_utf8(s).ok())
        .unwrap_or("(Failed to display line)");
    let line_no = what_line(buf, line_start);
    let pos_in_line =
        from_utf8(&buf[line_start..pos]).unwrap().chars().count() + 1;
    writeln!(out,
             "{prefix}{:>4}:{}\n\
              {prefix}     {:>pos$} {}",
             line_no,
             line,
             "^",
             msg,
             pos = pos_in_line,
             prefix = prefix)
            .unwrap();
}

fn what_line(buf: &[u8], pos: usize) -> usize {
    1 + buf[0..pos].iter().filter(|c| **c == b'\n').count()
}

/// The module containing your generated template code will also
/// contain everything from here.
///
/// The name `ructe::templates` should never be used.  Instead, you
/// should use the module templates created when compiling your
/// templates.
pub mod templates {
    use std::fmt::Display;
    use std::io::{self, Write};
    include!("template_utils.rs");

    /// Your `templates::statics` module will mirror this module.
    ///
    /// The name `ructe::templates::statics` should never be used.
    /// Instead, you should use the module `templates::statics` created
    /// when compiling your templates.
    pub mod statics {
        include!("statics_utils.rs");

        /// An array of all known `StaticFile`s.
        pub static STATICS: &'static [&'static StaticFile] = &[];
    }

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
