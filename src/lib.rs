//! Rust Compiled Templates is a HTML template system for Rust.
//!
//! Ructe works by converting your templates (and static files) to
//! rust source code, which is then compiled with your project.
//! This has the benefits that:
//!
//! 1. Many syntactical and logical errors in templates are caught
//!    compile-time, rather than in a running server.
//! 2. No extra latency on the first request, since the templates are
//!    fully compiled before starting the program.
//! 3. The template files does not have to be distributed / installed.
//!    Templates (and static assets) are included in the compiled
//!    program, which can be a single binary.
//!
//! The template syntax, which is inspired by [Twirl], the Scala-based
//! template engine in [Play framework], is documented in
//! the [Template_syntax] module.
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
//!     }
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
//! [examples in the repository]: https://github.com/kaj/ructe/tree/master/examples
//! [using ructe with warp and diesel]: https://github.com/kaj/warp-diesel-ructe-sample
//!
//! To be able to use this template in your rust code, you need a
//! `build.rs` that transpiles the template to rust code.
//! A minimal such build script looks like the following.
//! See the [`Ructe`] struct documentation for details.
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
//! # // mock
//! # mod templates {
//! #   use std::io::{Write, Result};
//! #   pub fn hello_html(buf: &mut impl Write, arg: &str) -> Result<()> {
//! #     writeln!(buf, "<h1>Hello {arg}!</h1>")
//! #   }
//! # }
//! let mut buf = Vec::new();
//! templates::hello_html(&mut buf, "World").unwrap();
//! assert_eq!(buf, b"<h1>Hello World!</h1>\n");
//! ```
//!
//! # Optional features
//!
//! Ructe has some options that can be enabled from `Cargo.toml`.
//!
//! * `sass` -- Compile sass and include the compiled css as static assets.
//! * `mime03` -- Static files know their mime types, compatible with
//!   version 0.3.x of the [mime] crate.
//! * `warp03` -- Provide an extension to `Response::Builder` of the [warp]
//!   framework (versions 0.3.x) to simplify template rendering.
//! * `http-types` -- Static files know their mime types, compatible with
//!   the [http-types] crate.
//! * `tide013`, `tide014`, `tide015`, `tide016` -- Support for the
//!   [tide] framework version 0.13.x through 0.16.x.  Implies the
//!   `http-types` feature (but does not require a direct http-types
//!   requirement, as that is reexported by tide).
//!   (these versions of tide is compatible enough that the features
//!   are actually just aliases for the first one, but a future tide
//!   version may require a modified feature.)
//!
//! [mime]: https://crates.rs/crates/mime
//! [warp]: https://crates.rs/crates/warp
//! [tide]: https://crates.rs/crates/tide
//! [http-types]: https://crates.rs/crates/http-types
//!
//! The `mime03`, and `http-types` features are mutually
//! exclusive and requires a dependency on a matching version of
//! `mime` or `http-types`.
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
#![forbid(unsafe_code, missing_docs)]
#![allow(clippy::manual_strip)] // Until MSR is 1.45.0

pub mod Template_syntax;
mod expression;
mod parseresult;
mod spacelike;
mod staticfiles;
mod template;
mod templateexpression;

use parseresult::show_errors;
use std::env;
use std::error::Error;
use std::fmt::{self, Debug, Display};
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
    f: Vec<u8>,
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
    /// you should probably use [`Ructe::from_env`] instead.
    ///
    /// [cargo]: https://doc.rust-lang.org/cargo/
    pub fn new(outdir: PathBuf) -> Result<Ructe> {
        let mut f = Vec::with_capacity(512);
        let outdir = outdir.join("templates");
        create_dir_all(&outdir)?;
        f.write_all(b"pub mod templates {\n")?;
        write_if_changed(
            &outdir.join("_utils.rs"),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/utils.rs"
            )),
        )?;
        f.write_all(
            b"#[doc(hidden)]\nmod _utils;\n\
              #[doc(inline)]\npub use self::_utils::*;\n\n",
        )?;
        if cfg!(feature = "warp03") {
            write_if_changed(
                &outdir.join("_utils_warp03.rs"),
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/templates/utils_warp03.rs"
                )),
            )?;
            f.write_all(
                b"#[doc(hidden)]\nmod _utils_warp03;\n\
                  #[doc(inline)]\npub use self::_utils_warp03::*;\n\n",
            )?;
        }
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
    /// # use ructe::{Ructe, RucteError};
    /// # fn main() -> Result<(), RucteError> {
    /// let mut ructe = Ructe::from_env()?;
    /// ructe.statics()?.add_files("static")?;
    /// Ok(())
    /// # }
    /// ```
    ///
    /// Assuming your project have a directory named `static` that
    /// contains e.g. a file called `logo.svg` and you have included
    /// the generated `templates.rs`, you can now use
    /// `templates::statics::logo_png` as a [`StaticFile`] in your
    /// project.
    ///
    /// [`StaticFile`]: templates::StaticFile
    pub fn statics(&mut self) -> Result<StaticFiles> {
        self.f.write_all(b"pub mod statics;")?;
        StaticFiles::for_template_dir(
            &self.outdir,
            &PathBuf::from(get_env("CARGO_MANIFEST_DIR")?),
        )
    }
}

impl Drop for Ructe {
    fn drop(&mut self) {
        let _ = self.f.write_all(b"}\n");
        let _ =
            write_if_changed(&self.outdir.join("../templates.rs"), &self.f);
    }
}

fn write_if_changed(path: &Path, content: &[u8]) -> io::Result<()> {
    use std::fs::{read, write};
    if let Ok(old) = read(path) {
        if old == content {
            return Ok(());
        }
    }
    write(path, content)
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
                let mut modrs = Vec::with_capacity(512);
                modrs.write_all(
                    b"#[allow(clippy::useless_attribute, unused)]\n\
                      use super::{Html,ToHtml};\n",
                )?;
                handle_entries(&mut modrs, &path, &outdir)?;
                write_if_changed(&outdir.join("mod.rs"), &modrs)?;
                writeln!(f, "pub mod {filename};\n")?;
            }
        } else if let Some(filename) = entry.file_name().to_str() {
            for suffix in &[".rs.html", ".rs.svg", ".rs.xml"] {
                if filename.ends_with(suffix) {
                    println!("cargo:rerun-if-changed={}", path.display());
                    let prename = &filename[..filename.len() - suffix.len()];
                    let name =
                        format!("{prename}_{}", &suffix[".rs.".len()..]);
                    if handle_template(&name, &path, outdir)? {
                        writeln!(
                            f,
                            "#[doc(hidden)]\n\
                             mod template_{name};\n\
                             #[doc(inline)]\n\
                             pub use self::template_{name}::{name};\n",
                        )?;
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
            let mut data = Vec::new();
            t.write_rust(&mut data, name)?;
            write_if_changed(
                &outdir.join(format!("template_{name}.rs")),
                &data,
            )?;
            Ok(true)
        }
        Err(error) => {
            println!("cargo:warning=Template parse error in {path:?}:");
            show_errors(&mut io::stdout(), &buf, &error, "cargo:warning=");
            Ok(false)
        }
    }
}

pub mod templates;

fn get_env(name: &str) -> Result<String> {
    env::var(name).map_err(|e| RucteError::Env(name.into(), e))
}

/// The build-time error type for Ructe.
pub enum RucteError {
    /// A build-time IO error in Ructe
    Io(io::Error),
    /// Error resolving a given environment variable.
    Env(String, env::VarError),
    /// Error bundling a sass stylesheet as css.
    #[cfg(feature = "sass")]
    Sass(rsass::Error),
}

impl Error for RucteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            RucteError::Io(e) => Some(e),
            RucteError::Env(_, e) => Some(e),
            #[cfg(feature = "sass")]
            RucteError::Sass(e) => Some(e),
        }
    }
}

impl Display for RucteError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        write!(out, "Error: {self:?}")
    }
}
impl Debug for RucteError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RucteError::Io(err) => Display::fmt(err, out),
            RucteError::Env(var, err) => write!(out, "{var:?}: {err}"),
            #[cfg(feature = "sass")]
            RucteError::Sass(err) => Debug::fmt(err, out),
        }
    }
}

impl From<io::Error> for RucteError {
    fn from(e: io::Error) -> RucteError {
        RucteError::Io(e)
    }
}

#[cfg(feature = "sass")]
impl From<rsass::Error> for RucteError {
    fn from(e: rsass::Error) -> RucteError {
        RucteError::Sass(e)
    }
}

/// A result where the error type is a [`RucteError`].
pub type Result<T> = std::result::Result<T, RucteError>;
