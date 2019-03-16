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
//! Templates (and static assets) are included in the compiled
//! program, which can be a single binary.
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
//! @use super::page_base;
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
//! There are [some examples in the
//! repository](https://github.com/kaj/ructe/tree/master/examples).
//! There is also [a separate example of using ructe with warp and
//! diesel](https://github.com/kaj/warp-diesel-ructe-sample).
//!
//! # Optional features
//!
//! Ructe has some options that can be enabled from `Cargo.toml`.
//!
//! * `sass` -- Compile sass and include the compiled css as static assets.
//! * `mime02` -- Static files know their mime types, compatible with
//! version 0.2.x of the `mime` crate.
//! * `mime03` -- Static files know their mime types, compatible with
//! version 0.3.x of the `mime` crate.
//!
//! The `mime02` and `mime03` features are mutually exclusive and
//! requires a dependency on a matching version of `mime`.
//! Any of them can be combined with the `sass` feature.
//!
//! ```toml
//! build = "src/build.rs"
//!
//! [build-dependencies]
//! ructe = { version = "^0.3", features = ["sass", "mime02"]
//!
//! [dependencies]
//! mime = "0.2.6"
//! ```
#[warn(missing_docs)]

extern crate base64;
extern crate bytecount;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate md5;
#[cfg(feature = "mime")]
extern crate mime;
#[macro_use]
extern crate nom;
#[cfg(feature = "sass")]
extern crate rsass;
#[cfg(feature = "warp")]
extern crate warp;

pub mod How_to_use_ructe;
mod spacelike;
#[macro_use]
mod errors;
mod expression;
#[macro_use]
mod templateexpression;
pub mod Template_syntax;
pub mod Using_static_files;
mod staticfiles;
mod template;

use errors::get_error;
use nom::types::CompleteByteSlice as Input;
use nom::{Context, Err, ErrorKind};
use std::env;
use std::fmt::Debug;
use std::fs::{create_dir_all, read_dir, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use template::template;

pub use staticfiles::StaticFiles;

/// Create a `statics` module inside `outdir`, containing static file data
/// for all files in `indir`.
///
/// This must be called *before* `compile_templates`.
#[deprecated(
    since = "0.6.0",
    note = "Use the statics() method of struct Ructe instead"
)]
pub fn compile_static_files(indir: &Path, outdir: &Path) -> Result<()> {
    #[allow(deprecated)]
    let mut out = StaticFiles::new(outdir)?;
    out.add_files(indir)
}

/// The main build-time interface of ructe.
///
/// Your build script should create an instance of `Ructe` and use it
/// to compile templates and possibly get access to the static files
/// handler.
///
/// When creating a `Ructe` it will create a file called
/// `templates.rs` in your `$OUT_DIR` (which is normally created and
/// specified by `cargo`).
/// The methods will and content, and when the `Ructe` goes of of
/// scope, the file will be completed.
pub struct Ructe {
    f: File,
    outdir: PathBuf,
}

impl Ructe {
    /// Create  a ructe instance from the `OUT_DIR` environment variable.
    ///
    /// This should be correct when using ructe from a build script in
    /// your project.
    pub fn from_env() -> Result<Ructe> {
        Ructe::new(PathBuf::from(env::var("OUT_DIR")?))
    }

    /// Create  a ructe instance from the `OUT_DIR` environment variable.
    ///
    /// This should be correct when using ructe from a build script in
    /// your project.
    pub fn new(out_dir: PathBuf) -> Result<Ructe> {
        let mut f = File::create(out_dir.join("templates.rs"))?;
        f.write_all(
            b"pub mod templates {\n\
              use std::io::{self, Write};\n\
              use std::fmt::Display;\n\n",
        )?;
        let outdir = out_dir.join("templates");
        create_dir_all(&outdir)?;
        Ok(Ructe { f, outdir })
    }

    pub fn statics(&mut self) -> Result<StaticFiles> {
        self.f.write_all(b"pub mod statics;")?;
        Ok(StaticFiles::for_template_dir(
            &self.outdir,
            &PathBuf::from(env::var("CARGO_MANIFEST_DIR")?),
        )?)
    }

    /// Create a `templates` module in `outdir` containing rust code for
    /// all templates found in `indir`.
    pub fn compile_templates<P>(&mut self, indir: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        handle_entries(&mut self.f, indir.as_ref(), &self.outdir)
    }
}

impl Drop for Ructe {
    fn drop(&mut self) {
        self.f
            .write_all(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/template_utils.rs"
            )))
            .unwrap();
        if cfg!(feature = "warp") {
            self.f
                .write_all(include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/template_utils_warp.rs"
                )))
                .unwrap();
        }
        self.f.write_all(b"\n}\n").unwrap();
    }
}

/// Create a `templates` module in `outdir` containing rust code for
/// all templates found in `indir`.
#[deprecated(since = "0.6.0", note = "Use method of struct Ructe instead")]
pub fn compile_templates(indir: &Path, outdir: &Path) -> Result<()> {
    let mut ructe = Ructe::new(outdir.into())?;

    ructe.compile_templates(indir)?;

    if ructe.outdir.join("statics.rs").exists() {
        ructe.f.write_all(b"pub mod statics;")?;
    }

    Ok(())
}

fn handle_entries(f: &mut Write, indir: &Path, outdir: &Path) -> Result<()> {
    println!("cargo:rerun-if-changed={}", indir.display());
    let suffix = ".rs.html";
    for entry in read_dir(indir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            if let Some(filename) = entry.file_name().to_str() {
                let outdir = outdir.join(filename);
                create_dir_all(&outdir)?;
                let mut modrs = File::create(outdir.join("mod.rs"))?;
                modrs.write_all(
                    b"#[allow(renamed_and_removed_lints)]\n\
                      #[cfg_attr(feature=\"cargo-clippy\", \
                      allow(useless_attribute))]\n\
                      #[allow(unused)]\n\
                      use super::{Html,ToHtml};\n",
                )?;
                handle_entries(&mut modrs, &path, &outdir)?;
                writeln!(f, "pub mod {name};\n", name = filename)?;
            }
        } else if let Some(filename) = entry.file_name().to_str() {
            if filename.ends_with(suffix) {
                println!("cargo:rerun-if-changed={}", path.display());
                let name = &filename[..filename.len() - suffix.len()];
                if handle_template(name, &path, outdir)? {
                    writeln!(
                        f,
                        "mod template_{name};\n\
                         pub use self::template_{name}::{name};\n",
                        name = name,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn handle_template(
    name: &str,
    path: &Path,
    outdir: &Path,
) -> io::Result<bool> {
    let mut input = File::open(path)?;
    let mut buf = Vec::new();
    input.read_to_end(&mut buf)?;
    match template(Input(&buf)) {
        Ok((_, t)) => {
            File::create(outdir.join(format!("template_{}.rs", name)))
                .and_then(|mut f| t.write_rust(&mut f, name))?;
            Ok(true)
        }
        result => {
            println!("cargo:warning=Template parse error in {:?}:", path);
            show_errors(&mut io::stdout(), &buf, result, "cargo:warning=");
            Ok(false)
        }
    }
}

fn show_errors<E>(
    out: &mut Write,
    buf: &[u8],
    result: nom::IResult<Input, E>,
    prefix: &str,
) where
    E: Debug,
{
    match result {
        Ok(_) => (),
        Err(Err::Error(Context::Code(_before, e))) => {
            show_error(out, buf, 0, &format!("error {:?}", e), prefix);
        }
        Err(Err::Error(Context::List(mut v))) => {
            v.reverse();
            for (i, e) in v {
                let pos = buf.len() - i.len();
                show_error(out, buf, pos, &get_message(&e), prefix);
            }
        }
        Err(Err::Failure(Context::List(mut v))) => {
            v.reverse();
            for (i, e) in v {
                let pos = buf.len() - i.len();
                show_error(out, buf, pos, &get_message(&e), prefix);
            }
        }
        Err(Err::Failure(e)) => {
            show_error(out, buf, 0, &format!("failure {:?}", e), prefix);
        }
        Err(_) => show_error(out, buf, 0, "xyzzy", prefix),
    }
}

fn get_message(err: &ErrorKind) -> String {
    match err {
        &ErrorKind::Custom(n) => match get_error(n) {
            Some(msg) => msg,
            None => format!("Unknown error #{}", n),
        },
        err => format!("{:?}", err),
    }
}

fn show_error(
    out: &mut Write,
    buf: &[u8],
    pos: usize,
    msg: &str,
    prefix: &str,
) {
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
    writeln!(
        out,
        "{prefix}{:>4}:{}\n\
         {prefix}     {:>pos$} {}",
        line_no,
        line,
        "^",
        msg,
        pos = pos_in_line,
        prefix = prefix,
    )
    .unwrap();
}

fn what_line(buf: &[u8], pos: usize) -> usize {
    1 + bytecount::count(&buf[0..pos], b'\n')
}

/// The module containing your generated template code will also
/// contain everything from here.
///
/// The name `ructe::templates` should never be used.  Instead, you
/// should use the module templates created when compiling your
/// templates.
pub mod templates {
    #[cfg(feature = "mime03")]
    use mime::Mime;
    use std::fmt::Display;
    use std::io::{self, Write};

    #[cfg(feature = "mime02")]
    /// Documentation mock.  The real Mime type comes from the `mime` crate.
    pub type Mime = u8; // mock

    /// A static file has a name (so its url can be recognized) and the
    /// actual file contents.
    ///
    /// The content-type (mime type) of the file is available as a
    /// static field when building ructe with the `mime03` feature or
    /// as the return value of a method when building ructe with the
    /// `mime02` feature (in `mime` version 0.2.x, a Mime cannot be
    /// defined as a part of a const static value.
    pub struct StaticFile {
        /// The actual static file contents.
        pub content: &'static [u8],
        /// The file name as used in a url, including a short (48 bits
        /// as 8 base64 characters) hash of the content, to enable
        /// long-time caching of static resourses in the clients.
        pub name: &'static str,
        /// The Mime type of this static file, as defined in the mime
        /// crate version 0.3.x.
        #[cfg(feature = "mime03")]
        pub mime: &'static Mime,
    }

    impl StaticFile {
        /// Get the mime type of this static file.
        ///
        /// Currently, this method parses a (static) string every time.
        /// A future release of `mime` may support statically created
        /// `Mime` structs, which will make this nicer.
        #[allow(unused)]
        #[cfg(feature = "mime02")]
        pub fn mime(&self) -> Mime {
            unimplemented!()
        }
    }

    include!("template_utils.rs");

    #[cfg(feature = "warp")]
    include!("template_utils_warp.rs");

    #[test]
    fn encoded() {
        let mut buf = Vec::new();
        "a < b\0\n".to_html(&mut buf).unwrap();
        assert_eq!(b"a &lt; b\0\n", &buf[..]);

        let mut buf = Vec::new();
        "'b".to_html(&mut buf).unwrap();
        assert_eq!(b"&#39;b", &buf[..]);

        let mut buf = Vec::new();
        "xxxxx>&".to_html(&mut buf).unwrap();
        assert_eq!(b"xxxxx&gt;&amp;", &buf[..]);
    }

    #[test]
    fn encoded_empty() {
        let mut buf = Vec::new();
        "".to_html(&mut buf).unwrap();
        "".to_html(&mut buf).unwrap();
        "".to_html(&mut buf).unwrap();
        assert_eq!(b"", &buf[..]);
    }

    #[test]
    fn double_encoded() {
        let mut buf = Vec::new();
        "&amp;".to_html(&mut buf).unwrap();
        "&lt;".to_html(&mut buf).unwrap();
        assert_eq!(b"&amp;amp;&amp;lt;", &buf[..]);
    }

    #[test]
    fn encoded_only() {
        let mut buf = Vec::new();
        "&&&&&&&&&&&&&&&&".to_html(&mut buf).unwrap();
        assert_eq!(b"&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;&amp;" as &[u8], &buf[..]);

        let mut buf = Vec::new();
        "''''''''''''''".to_html(&mut buf).unwrap();
        assert_eq!(b"&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;&#39;" as &[u8], &buf[..]);
    }

    #[test]
    fn raw_html() {
        let mut buf = Vec::new();
        Html("a<b>c</b>").to_html(&mut buf).unwrap();
        assert_eq!(b"a<b>c</b>", &buf[..]);
    }
}

/// The build-time error type for Ructe.
#[derive(Debug)]
pub enum RucteError {
    Io(io::Error),
    Var(env::VarError),
}

impl From<io::Error> for RucteError {
    fn from(e: io::Error) -> RucteError {
        RucteError::Io(e)
    }
}

impl From<env::VarError> for RucteError {
    fn from(e: env::VarError) -> RucteError {
        RucteError::Var(e)
    }
}

/// A result where the error type is a [`RucteError`].
///
/// [`RucteError`]: enum.RucteError.html
pub type Result<T> = std::result::Result<T, RucteError>;
