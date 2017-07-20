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

extern crate base64;
extern crate md5;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate lazy_static;
#[cfg(feature = "sass")]
extern crate rsass;

pub mod How_to_use_ructe;
mod spacelike;
mod engine;
#[macro_use]
mod errors;
mod expression;
#[macro_use]
mod templateexpression;
mod template;
pub mod Template_syntax;
pub mod Using_static_files;

pub use engine::{RenderEngine, TEMPLATE_UTILS};
use engine::Engine;
use errors::get_error;
use nom::{ErrorKind, prepare_errors};
use nom::IResult::*;
use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs::{File, create_dir_all, read_dir};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::str::from_utf8;
pub use template::Template;

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
    /// Rust source file `statics.rs` beeing written.
    src: File,
    /// Maps rust names to public names (foo_jpg -> foo-abc123.jpg)
    names: BTreeMap<String, String>,
    /// Maps public names to rust names (foo-abc123.jpg -> foo_jpg)
    names_r: BTreeMap<String, String>,
}

impl StaticFiles {
    /// Create a new set of static files.
    ///
    /// There should only be one `StaticFiles` for a set of compiled templates.
    /// The `outdir` should be the same as in the call to `compile_templates`.
    pub fn new(outdir: &Path) -> io::Result<Self> {
        let outdir = outdir.join("templates");
        create_dir_all(&outdir)?;
        let mut src = File::create(outdir.join("statics.rs"))?;
        if cfg!(feature = "mime03") {
            write!(src,
                   "extern crate mime;\n\
                    use self::mime::Mime;\n\n")?;
        }
        write!(src,
               "/// A static file has a name (so its url can be recognized) \
                and the\n\
                /// actual file contents.\n\
                ///\n\
                /// The name includes a short (48 bits as 8 base64 characters) \
                hash of\n\
                /// the content, to enable long-time caching of static \
                resourses in\n\
                /// the clients.\n\
                #[allow(dead_code)]\n\
                pub struct StaticFile {{\n    \
                    pub content: &'static [u8],\n    \
                    pub name: &'static str,\n")?;
        if cfg!(feature = "mime02") {
            write!(src, "    _mime: &'static str,\n")?;
        }
        if cfg!(feature = "mime03") {
            write!(src, "    pub mime: &'static Mime,\n")?;
        }
        write!(src,
               "}}\n\n\
                #[allow(dead_code)]\n\
                impl StaticFile {{\n    \
                /// Get a single `StaticFile` by name, if it exists.\n    \
                pub fn get(name: &str) -> Option<&'static Self> {{\n        \
                if let Ok(pos) = STATICS.\
                binary_search_by_key(&name, |s| s.name) {{\n            \
                return Some(STATICS[pos]);\n        \
                }} else {{\n            \
                None\n        \
                }}\n    \
                }}\n\
                }}\n")?;
        if cfg!(feature = "mime02") {
            write!(src,
                   "extern crate mime;\n\
                    use self::mime::Mime;\n\n\
                    impl StaticFile {{\n    \
                    /// Get the mime type of this static file.\n    \
                    ///\n    \
                    /// Currently, this method parses a (static) string every \
                    time.\n    \
                    /// A future release of `mime` may support statically \
                    created\n    \
                    /// `Mime` structs, which will make this nicer.\n    \
                    #[allow(unused)]\n    \
                    pub fn mime(&self) -> Mime {{\n        \
                    self._mime.parse().unwrap()\n    \
                    }}\n\
                    }}\n")?;
        }
        Ok(StaticFiles {
               src: src,
               names: BTreeMap::new(),
               names_r: BTreeMap::new(),
           })
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
            let from_name = format!("{}_{}", name, ext);
            let to_name = format!("{}-{}.{}", name, checksum_slug(&buf), &ext);
            self.write_static_file(path, name, &buf, ext)?;
            self.names.insert(from_name.clone(), to_name.clone());
            self.names_r.insert(to_name, from_name.clone());
        }
        Ok(())
    }

    /// Add a file by its name and content.
    ///
    /// The `path` parameter is used only to create a file name, the actual
    /// content of the static file will be the `data` parameter.
    pub fn add_file_data(&mut self,
                         path: &Path,
                         data: &[u8])
                         -> io::Result<()> {
        if let Some((name, ext)) = name_and_ext(path) {
            let from_name = format!("{}_{}", name, ext);
            let to_name = format!("{}-{}.{}", name, checksum_slug(data), &ext);
            self.write_static_buf(path, name, data, ext)?;
            self.names.insert(from_name.clone(), to_name.clone());
            self.names_r.insert(to_name, from_name.clone());
        }
        Ok(())
    }

    /// Compile a sass file and add the resulting css.
    ///
    /// If `src` is `"somefile.sass"`, then that file will be copiled
    /// with rsass (using the `Comressed` output style).
    /// The result will be addes as if if was an existing
    /// `"somefile.css"` file.
    ///
    /// This method is only available when ructe is built with the
    /// "sass" feature.
    #[cfg(feature = "sass")]
    pub fn add_sass_file(&mut self, src: &Path) -> io::Result<()> {
        use rsass::*;
        use std::sync::Arc;
        let mut scope = GlobalScope::new();

        // TODO Find any referenced files!
        println!("cargo:rerun-if-changed={}", src.to_string_lossy());

        let existing_statics = Arc::new(self.get_names().clone());
        scope.define_function(
            "static_name",
            SassFunction::builtin(
                vec![("name".into(), Value::Null)],
                false,
                Arc::new(move |s| match s.get("name") {
                    Value::Literal(name, _) => {
                        let name = name.replace('-', "_").replace('.', "_");
                        for (n, v) in existing_statics.as_ref() {
                            if name == *n {
                                return Ok(Value::Literal(v.clone(),
                                                         Quotes::Double));
                            }
                        }
                        Err(Error::S(format!("Static file {} not found", name)))
                    }
                    name => Err(Error::badarg("string", &name)),
                }),
            ),
        );

        let file_context = FileContext::new();
        let (file_context, src) = file_context.file(src);
        let scss = parse_scss_file(&src).unwrap();
        let style = OutputStyle::Compressed;
        let css = style.write_root(&scss, &mut scope, file_context).unwrap();
        self.add_file_data(&src.with_extension("css"), &css)
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
                content: include_bytes!({path:?}),\n  \
                name: \"{name}-{hash}.{suf}\",\n\
                {mime}\
                }};\n",
               path = path,
               name = name,
               hash = checksum_slug(content),
               suf = suffix,
               mime = mime_arg(suffix))
    }

    fn write_static_buf(&mut self,
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
                {mime}\
                }};\n",
               path = path,
               name = name,
               content = content,
               hash = checksum_slug(content),
               suf = suffix,
               mime = mime_arg(suffix))
    }

    /// Get a mapping of names, from without hash to with.
    ///
    /// ````
    /// # use ructe::StaticFiles;
    /// # use std::path::PathBuf;
    /// # use std::vec::Vec;
    /// # let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    /// #     .join("target").join("test-tmp");
    /// let mut statics = StaticFiles::new(&p).unwrap();
    /// statics.add_file_data("black.css".as_ref(), b"body{color:black}\n");
    /// statics.add_file_data("blue.css".as_ref(), b"body{color:blue}\n");
    /// assert_eq!(statics.get_names().iter()
    ///                .map(|(a, b)| format!("{} -> {}", a, b))
    ///                .collect::<Vec<_>>(),
    ///            vec!["black_css -> black-r3rltVhW.css".to_string(),
    ///                 "blue_css -> blue-GZGxfXag.css".to_string()])
    /// ````
    pub fn get_names(&self) -> &BTreeMap<String, String> {
        &self.names
    }
}

#[cfg(not(feature = "mime02"))]
#[cfg(not(feature = "mime03"))]
fn mime_arg(_: &str) -> String {
    "".to_string()
}
#[cfg(feature = "mime02")]
fn mime_arg(suffix: &str) -> String {
    format!("_mime: {:?},\n", mime_from_suffix(suffix))
}

#[cfg(feature = "mime02")]
fn mime_from_suffix(suffix: &str) -> &'static str {
    // TODO This is just enough for some examples.  Need more types.
    // Should probably look at content as well.
    match suffix.to_lowercase().as_ref() {
        "css" => "text/css",
        "eot" => "application/vnd.ms-fontobject",
        "jpg" | "jpeg" => "image/jpeg",
        "js" => "application/javascript",
        "png" => "image/png",
        "woff" => "application/font-woff",
        _ => "Application/OctetStream",
    }
}

#[cfg(feature = "mime03")]
fn mime_arg(suffix: &str) -> String {
    format!("mime: &mime::{},\n", mime_from_suffix(suffix))
}

#[cfg(feature = "mime03")]
fn mime_from_suffix(suffix: &str) -> &'static str {
    // TODO This is just enough for some examples.  Need more types.
    // Should probably look at content as well.
    // This is limited to the constants that is defined in mime 0.3.
    match suffix.to_lowercase().as_ref() {
        "bmp" => "IMAGE_BMP",
        "css" => "TEXT_CSS",
        "gif" => "IMAGE_GIF",
        "jpg" | "jpeg" => "IMAGE_JPEG",
        "js" => "TEXT_JAVASCRIPT",
        "json" => "APPLICATION_JSON",
        "png" => "IMAGE_PNG",
        _ => "APPLICATION_OCTET_STREAM",
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
                       self.names_r
                           .iter()
                           .map(|s| format!("&{}", s.1))
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

pub struct TraverseConf<'a> {
    pub suffix: &'a str,
    pub prelude_name: &'a OsString,
    pub parser: &'a Fn(&[u8], &Path) -> Option<Template>,
}

pub fn parse_and_report_cargo(buf: &[u8], path: &Path) -> Option<Template> {
    match template::template(&buf) {
        Done(_, tpl) => {
            println!("cargo:rerun-if-changed={}", path.display());
            Some(tpl)
        }
        result => {
            println!("cargo:warning=Template parse error in {:?}:", path);
            show_errors(&mut io::stdout(), &buf, result, "cargo:warning=");

            None
        }
    }
}

/// Create a `templates` module in `outdir` containing rust code for
/// all templates found in `indir`.
pub fn compile_templates(indir: &Path, outdir: &Path) -> io::Result<()> {
    let conf = TraverseConf {
        suffix: ".rs.html",
        prelude_name: &OsString::new(),
        parser: &parse_and_report_cargo,
    };

    let en = Engine::new(outdir, "templates")?;
    if outdir.join("templates").join("statics.rs").exists() {
        en.add_mod("statics")?;
    }

    traverse_dir(indir, &en, &conf, &[])
}

pub fn compile_templates_cargo(tmpl_dir: &str) -> io::Result<()> {
    let outdir = PathBuf::from(env::var("OUT_DIR").expect(
        "Environment variable OUT_DIR not set by cargo",
    ));

    let indir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect(
        "Environment variable CARGO_MANIFEST_DIR not set by cargo",
    )).join(tmpl_dir);

    let conf = TraverseConf {
        suffix: ".rs.html",
        prelude_name: &OsString::from("_prelude.rs.html"),
        parser: &parse_and_report_cargo,
    };

    let en = Engine::new(&outdir, tmpl_dir)?;
    if outdir.join(tmpl_dir).join("statics.rs").exists() {
        en.add_mod("statics")?;
    }

    traverse_dir(&indir, &en, &conf, &[])
}

pub fn traverse_dir<E: RenderEngine>(
    dir: &Path,
    en: &E,
    conf: &TraverseConf,
    prelude: &[u8],
) -> io::Result<()> {
    let mut buf = Vec::from(prelude);
    if conf.prelude_name.len() > 0 {
        if let Ok(mut f) = File::open(dir.join(conf.prelude_name)) {
            f.read_to_end(&mut buf)?;
        }
    }

    for e in dir.read_dir()? {
        let e = e?;
        let path = e.path();
        let file_name = e.file_name();
        if file_name == *conf.prelude_name {
            continue;
        }
        let name = match file_name.to_str() {
            Some(x) => x,
            _ => {
                let msg = format!(
                    "Broken file name {} in {:?}",
                    e.file_name().to_string_lossy(),
                    path
                );
                return Err(io::Error::new(io::ErrorKind::Other, msg));
            }
        };

        if path.is_dir() {
            traverse_dir(&path, &*en.sublevel(name)?, conf, &buf)?;
            continue;
        }

        if name.ends_with(conf.suffix) {
            let old_len = buf.len();
            match File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    return Err(wrap_err(
                        e,
                        format!("Error while opening {:?}", path),
                    ))
                }
            }.read_to_end(&mut buf)?;

            if let Some(tpl) = (conf.parser)(&buf, &path) {
                let name = &name[..name.len() - conf.suffix.len()];
                en.render(name, &tpl)?;
            }

            buf.truncate(old_len);
        }
    }

    Ok(())
}

fn wrap_err(err: io::Error, msg: String) -> io::Error {
    io::Error::new(err.kind(), ErrorWrapper::new(err, msg))
}

#[derive(Debug)]
pub struct ErrorWrapper<T: Error> {
    description: String,
    inner: T,
}

impl<T: Error> ErrorWrapper<T> {
    pub fn new(err: T, msg: String) -> Self {
        ErrorWrapper { description: msg, inner: err }
    }
}

impl<T: Error> Error for ErrorWrapper<T> {
    fn description(&self) -> &str {
        &self.description
    }

    fn cause(&self) -> Option<&Error> {
        Some(&self.inner)
    }
}

impl<T: Error> fmt::Display for ErrorWrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
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
