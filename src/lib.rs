//! Rust Compiled Templates is a HTML template system for Rust.
//!
//! Ructe works by converting your templates (and static files) to
//! rust source code, which is then compiled with your project.
//! This has the benefits that:
//!
//! 1. Many syntactical and logical errors in templates are caught
//! compile-time, rather than in a running server.
//! 2. No extra latency on the first request, since the templates are
//! fully compiled before starting the program.
//! 3. The template files does not have to be distributed / installed.
//! Templates (and static assets) are included in the compiled
//! program, which can be a single binary.
//!
//! The template syntax, which is inspired by [Twirl], the Scala-based
//! template engine in [Play framework], is documented in
//! [the _Template syntax_ module].
//! A sample template may look like this:
//!
//! ```html
//! @use any::rust::Type;
//!
//! @(name: &str, items: &[Type])
//!
//! <html>
//!   <head><title>@name</title></head>
//!   <body>
//!     @if items.is_empty() {
//!       <p>There are no items.</p>
//!     } else {
//!       <p>There are @items.len() items.</p>
//!       <ul>
//!       @for item in items {
//!         <li>@item</li>
//!       }
//!       </ul>
//!   <body>
//! </html>
//! ```
//!
//! There are some [examples in the repository].
//! There is also a separate example of
//! [using ructe with warp and diesel].
//!
//! [Twirl]: https://github.com/playframework/twirl
//! [Play framework]: https://www.playframework.com/
//! [the _Template syntax_ module]: Template_syntax/index.html
//! [examples in the repository]: https://github.com/kaj/ructe/tree/master/examples
//! [using ructe with warp and diesel]: https://github.com/kaj/warp-diesel-ructe-sample
//!
//! To be able to use this template in your rust code, you need a
//! `build.rs` that transpiles the template to rust code.
//! A minimal such build script looks like the following.
//! See the [`Ructe`] struct documentation for details.
//!
//! [`Ructe`]: struct.Ructe.html
//!
//! ```rust,no_run
//! use ructe::{Result, Ructe};
//!
//! fn main() -> Result<()> {
//!     Ructe::from_env()?.compile_templates("templates")
//! }
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
//!     templates::hello_html(&mut buf, "World").unwrap();
//!     assert_eq!(buf, b"<h1>Hello World!</h1>\n");
//! }
//! ```
//!
//! # Optional features
//!
//! Ructe has some options that can be enabled from `Cargo.toml`.
//!
//! * `sass` -- Compile sass and include the compiled css as static assets.
//! * `mime03` -- Static files know their mime types, compatible with
//! version 0.3.x of the [mime] crate.
//! * `mime02` -- Static files know their mime types, compatible with
//! version 0.2.x of the [mime] crate.
//! * `warp02` -- Provide an extension to [`Response::Builder`] to
//! simplify template rendering in the [warp] framework, versions 0.2.x.
//! * `warp01` -- Provide an extension to [`Response::Builder`] to
//! simplify template rendering in the [warp] framework, versions 0.1.x.
//!
//! [`response::Builder`]: ../http/response/struct.Builder.html
//! [mime]: https://crates.rs/crates/mime
//! [warp]: https://crates.rs/crates/warp
//!
//! The `mime02` and `mime03` features are mutually exclusive and
//! requires a dependency on a matching version of `mime`.
//! Any of them can be combined with the `sass` feature.
//!
//! ```toml
//! build = "src/build.rs"
//!
//! [build-dependencies]
//! ructe = { version = "0.6.0", features = ["sass", "mime03"] }
//!
//! [dependencies]
//! mime = "0.3.13"
//! ```
#![warn(missing_docs)]
extern crate base64;
extern crate bytecount;
extern crate itertools;
extern crate md5;
#[cfg(feature = "mime")]
extern crate mime;
extern crate nom;
#[cfg(feature = "sass")]
extern crate rsass;
#[cfg(feature = "warp")]
extern crate warp;

pub mod Template_syntax;
mod expression;
#[doc(hidden)] // public for doctest to work, but hide from docs.
pub mod nom_delimited_list;
mod parseresult;
mod spacelike;
mod staticfiles;
mod template;
mod templateexpression;

use parseresult::show_errors;
use std::env;
use std::error::Error;
use std::fmt::{self, Display};
use std::fs::{create_dir_all, read_dir, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use template::template;

pub use staticfiles::StaticFiles;

/// The main build-time interface of ructe.
///
/// Your build script should create an instance of `Ructe` and use it
/// to compile templates and possibly get access to the static files
/// handler.
///
/// Ructe compiles your templates to rust code that should be compiled
/// with your other rust code, so it needs to be called before
/// compiling.
/// Assuming you use [cargo], it can be done like this:
///
/// First, specify a build script and ructe as a build dependency in
/// `Cargo.toml`:
///
/// ```toml
/// build = "src/build.rs"
///
/// [build-dependencies]
/// ructe = "0.6.0"
/// ```
///
/// Then, in `build.rs`, compile all templates found in the templates
/// directory and put the output where cargo tells it to:
///
/// ```rust,no_run
/// use ructe::{Result, Ructe};
///
/// fn main() -> Result<()> {
///     Ructe::from_env()?.compile_templates("templates")
/// }
/// ```
///
/// And finally, include and use the generated code in your code.
/// The file `templates.rs` will contain `mod templates { ... }`,
/// so I just include it in my `main.rs`:
///
/// ```rust,ignore
/// include!(concat!(env!("OUT_DIR"), "/templates.rs"));
/// ```
///
///
/// When creating a `Ructe` it will create a file called
/// `templates.rs` in your `$OUT_DIR` (which is normally created and
/// specified by `cargo`).
/// The methods will add content, and when the `Ructe` goes of of
/// scope, the file will be completed.
///
/// [cargo]: https://doc.rust-lang.org/cargo/
pub struct Ructe {
    f: File,
    outdir: PathBuf,
}

impl Ructe {
    /// Create a Ructe instance suitable for a [cargo]-built project.
    ///
    /// A file called `templates.rs` (and a directory called
    /// `templates` containing sub-modules) will be created in the
    /// directory that cargo specifies with the `OUT_DIR` environment
    /// variable.
    ///
    /// [cargo]: https://doc.rust-lang.org/cargo/
    pub fn from_env() -> Result<Ructe> {
        Ructe::new(PathBuf::from(get_env("OUT_DIR")?))
    }

    /// Create  a ructe instance writing to a given directory.
    ///
    /// The `out_dir` path is assumed to be a directory that exists
    /// and is writable.
    /// A file called `templates.rs` (and a directory called
    /// `templates` containing sub-modules) will be created in
    /// `out_dir`.
    ///
    /// If you are using Ructe in a project that uses [cargo],
    /// you should probably use [`from_env`] instead.
    ///
    /// [cargo]: https://doc.rust-lang.org/cargo/
    /// [`from_env`]: #method.from_env
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

    /// Create a `templates` module in `outdir` containing rust code for
    /// all templates found in `indir`.
    ///
    /// If indir is a relative path, it should be relative to the main
    /// directory of your crate, i.e. the directory containing your
    /// `Cargo.toml` file.
    ///
    /// Files with suffix `.rs.html`, `.rs.svg`, or `.rs.xml` are
    /// considered templates.
    /// A templete file called `template.rs.html`, `template.rs.svg`,
    /// etc, will result in a callable function named `template_html`,
    /// `template_svg`, etc.
    /// The `template_html` function will get a `template` alias for
    /// backwards compatibility, but that will be removed in a future
    /// release.
    pub fn compile_templates<P>(&mut self, indir: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        handle_entries(&mut self.f, indir.as_ref(), &self.outdir)
    }

    /// Create a [`StaticFiles`] handler for this Ructe instance.
    ///
    /// This will create a `statics` module inside the generated
    /// `templates` module.
    ///
    /// # Examples
    ///
    /// This code goes into the `build.rs`:
    ///
    /// ```no_run
    /// # extern crate ructe;
    /// # use ructe::{Ructe, RucteError};
    /// # fn main() -> Result<(), RucteError> {
    /// let mut ructe = Ructe::from_env()?;
    /// ructe.statics()?.add_files("static")
    /// # }
    /// ```
    ///
    /// Assuming your project have a directory named `static` that
    /// contains e.g. a file called `logo.svg` and you have included
    /// the generated `templates.rs`, you can now use
    /// `templates::statics::logo_png` as a [`StaticFile`] in your
    /// project.
    ///
    /// [`StaticFiles`]: struct.StaticFiles.html
    /// [`StaticFile`]: templates/struct.StaticFile.html
    pub fn statics(&mut self) -> Result<StaticFiles> {
        self.f.write_all(b"pub mod statics;")?;
        Ok(StaticFiles::for_template_dir(
            &self.outdir,
            &PathBuf::from(get_env("CARGO_MANIFEST_DIR")?),
        )?)
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
        if cfg!(feature = "warp02") {
            self.f
                .write_all(include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/template_utils_warp02.rs"
                )))
                .unwrap();
        }
        self.f.write_all(b"\n}\n").unwrap();
    }
}

fn handle_entries(
    f: &mut impl Write,
    indir: &Path,
    outdir: &Path,
) -> Result<()> {
    println!("cargo:rerun-if-changed={}", indir.display());
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
            for suffix in &[".rs.html", ".rs.svg", ".rs.xml"] {
                if filename.ends_with(suffix) {
                    println!("cargo:rerun-if-changed={}", path.display());
                    let prename = &filename[..filename.len() - suffix.len()];
                    let name =
                        format!("{}_{}", prename, &suffix[".rs.".len()..]);
                    if handle_template(&name, &path, outdir)? {
                        writeln!(
                            f,
                            "mod template_{name};\n\
                             pub use self::template_{name}::{name};\n",
                            name = name,
                        )?;
                        // Backwards compatibility to 0.7.2 and earlier.
                        if suffix == &".rs.html" {
                            writeln!(
                                f,
                                "#[deprecated(since=\"0.7.4\", \
                                 note=\"please use `{name}` instead\")]\n\
                                 pub use self::{name} as {alias};\n",
                                alias = prename,
                                name = name,
                            )?;
                        }
                    }
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
    match template(&buf) {
        Ok((_, t)) => {
            File::create(outdir.join(format!("template_{}.rs", name)))
                .and_then(|mut f| t.write_rust(&mut f, name))?;
            Ok(true)
        }
        Err(error) => {
            println!("cargo:warning=Template parse error in {:?}:", path);
            show_errors(&mut io::stdout(), &buf, &error, "cargo:warning=");
            Ok(false)
        }
    }
}

/// The module containing your generated template code will also
/// contain everything from here.
///
/// The name `ructe::templates` should never be used.  Instead, you
/// should use the module templates created when compiling your
/// templates.
/// If you include the generated `templates.rs` in your `main.rs` (or
/// `lib.rs` in a library crate), this module will be
/// `crate::templates`.
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

fn get_env(name: &str) -> Result<String> {
    env::var(name).map_err(|e| RucteError::Env(name.into(), e))
}

/// The build-time error type for Ructe.
#[derive(Debug)]
pub enum RucteError {
    /// A build-time IO error in Ructe
    Io(io::Error),
    /// Error resolving a given environment variable.
    Env(String, env::VarError),
}

impl Error for RucteError {}

impl Display for RucteError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RucteError::Io(err) => err.fmt(out),
            RucteError::Env(var, err) => write!(out, "{:?}: {}", var, err),
        }
    }
}

impl From<io::Error> for RucteError {
    fn from(e: io::Error) -> RucteError {
        RucteError::Io(e)
    }
}

/// A result where the error type is a [`RucteError`].
///
/// [`RucteError`]: enum.RucteError.html
pub type Result<T> = std::result::Result<T, RucteError>;
