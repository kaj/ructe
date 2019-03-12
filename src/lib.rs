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

extern crate base64;
extern crate bytecount;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate md5;
#[macro_use]
extern crate nom;
#[cfg(feature = "sass")]
extern crate rsass;

pub mod How_to_use_ructe;
mod spacelike;
#[macro_use]
mod errors;
mod expression;
#[macro_use]
mod templateexpression;
pub mod Template_syntax;
pub mod Using_static_files;
mod template;

use errors::get_error;
use itertools::Itertools;
use nom::types::CompleteByteSlice as Input;
use nom::{Context, Err, ErrorKind};
use std::collections::BTreeMap;
use std::env;
use std::fmt::{self, Debug, Display};
use std::fs::{create_dir_all, read_dir, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use template::template;

/// Create a `statics` module inside `outdir`, containing static file data
/// for all files in `indir`.
///
/// This must be called *before* `compile_templates`.
#[deprecated(
    since = "0.6",
    note = "Use the statics() method of struct Ructe instead"
)]
pub fn compile_static_files(indir: &Path, outdir: &Path) -> Result<()> {
    #[allow(deprecated)]
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
    ///
    /// From version 0.6 of ructe,
    /// [the `statics()` method of `struct Ructe`](struct.Ructe.html#method.statics)
    /// should be used instead of this method.
    #[deprecated(
        since = "0.6",
        note = "Use the statics() method of struct Ructe instead"
    )]
    pub fn new(outdir: &Path) -> io::Result<Self> {
        let outdir = outdir.join("templates");
        create_dir_all(&outdir)?;
        StaticFiles::for_template_dir(&outdir)
    }

    fn for_template_dir(outdir: &Path) -> io::Result<Self> {
        let mut src = File::create(outdir.join("statics.rs"))?;
        if cfg!(feature = "mime03") {
            src.write_all(b"extern crate mime;\nuse self::mime::Mime;\n\n")?;
        }
        src.write_all(
b"/// A static file has a name (so its url can be recognized) and the
/// actual file contents.
///
/// The name includes a short (48 bits as 8 base64 characters) hash of
/// the content, to enable long-time caching of static resourses in
/// the clients.
#[allow(dead_code)]
pub struct StaticFile {
    pub content: &'static [u8],
    pub name: &'static str,
")?;
        if cfg!(feature = "mime02") {
            src.write_all(b"    _mime: &'static str,\n")?;
        }
        if cfg!(feature = "mime03") {
            src.write_all(b"    pub mime: &'static Mime,\n")?;
        }
        src.write_all(
            b"}
#[allow(dead_code)]
impl StaticFile {
    /// Get a single `StaticFile` by name, if it exists.
    pub fn get(name: &str) -> Option<&'static Self> {
        if let Ok(pos) = STATICS.binary_search_by_key(&name, |s| s.name) {
            return Some(STATICS[pos]);
        } else {None}
    }
}
",
        )?;
        if cfg!(feature = "mime02") {
            src.write_all(
                b"extern crate mime;
use self::mime::Mime;
impl StaticFile {
    /// Get the mime type of this static file.
    ///
    /// Currently, this method parses a (static) string every time.
    /// A future release of `mime` may support statically created
    /// `Mime` structs, which will make this nicer.
    #[allow(unused)]
    pub fn mime(&self) -> Mime {
        self._mime.parse().unwrap()
    }
}
",
            )?;
        }
        Ok(StaticFiles {
            src,
            names: BTreeMap::new(),
            names_r: BTreeMap::new(),
        })
    }

    /// Add all files from a specific directory, `indir`, as static files.
    pub fn add_files<P: AsRef<Path>>(&mut self, indir: P) -> Result<()> {
        let indir = indir.as_ref();
        let indir = if indir.is_relative() {
            PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join(indir)
        } else {
            indir.into()
        };
        println!("cargo:rerun-if-changed={}", indir.display());
        for entry in read_dir(indir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                self.add_file(&entry.path())?;
            }
        }
        Ok(())
    }

    /// Add all files from a specific directory, `indir`, as static files.
    ///
    /// The `to` string is used as a directory path of the resulting
    /// urls, the file names are taken as is, without adding any hash.
    /// This is usefull for resources used by preexisting javascript
    /// packages, where it might be hard to change the used urls.
    pub fn add_files_as(&mut self, indir: &Path, to: &str) -> io::Result<()> {
        for entry in read_dir(indir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let to =
                format!("{}/{}", to, entry.file_name().to_string_lossy());
            if file_type.is_file() {
                self.add_file_as(&entry.path(), &to)?;
            } else if file_type.is_dir() {
                self.add_files_as(&entry.path(), &to)?;
            }
        }
        Ok(())
    }

    /// Add one specific file as a static file.
    ///
    /// Create a name to use in the url like `name-hash.ext` where
    /// name and ext are the name and extension from `path` and has is
    /// a few url-friendly bytes from a hash of the file content.
    pub fn add_file(&mut self, path: &Path) -> io::Result<()> {
        if let Some((name, ext)) = name_and_ext(path) {
            println!("cargo:rerun-if-changed={}", path.display());
            let mut input = File::open(&path)?;
            let mut buf = Vec::new();
            input.read_to_end(&mut buf)?;
            let rust_name = format!("{}_{}", name, ext);
            let url_name =
                format!("{}-{}.{}", name, checksum_slug(&buf), &ext);
            self.add_static(
                path,
                &rust_name,
                &url_name,
                &FileContent(path),
                ext,
            )?;
        }
        Ok(())
    }

    /// Add one specific file as a static file.
    ///
    /// Use `url_name` in the url without adding any hash characters.
    pub fn add_file_as(
        &mut self,
        path: &Path,
        url_name: &str,
    ) -> io::Result<()> {
        let ext = name_and_ext(path).map(|(_, e)| e).unwrap_or("");
        println!("cargo:rerun-if-changed={}", path.display());
        self.add_static(path, url_name, url_name, &FileContent(path), ext)?;
        Ok(())
    }

    /// Add a file by its name and content.
    ///
    /// The `path` parameter is used only to create a file name, the actual
    /// content of the static file will be the `data` parameter.
    pub fn add_file_data<P>(&mut self, path: P, data: &[u8]) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if let Some((name, ext)) = name_and_ext(path) {
            let rust_name = format!("{}_{}", name, ext);
            let url_name =
                format!("{}-{}.{}", name, checksum_slug(data), &ext);
            self.add_static(
                path,
                &rust_name,
                &url_name,
                &ByteString(data),
                ext,
            )?;
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
    pub fn add_sass_file<P>(&mut self, src: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let src = src.as_ref();
        use rsass::*;
        use std::sync::Arc;
        let mut scope = GlobalScope::new();

        // TODO Find any referenced files!
        println!("cargo:rerun-if-changed={}", src.display());

        let existing_statics = Arc::new(self.get_names().clone());
        scope.define_function(
            "static_name",
            SassFunction::builtin(
                vec![("name".into(), sass::Value::Null)],
                false,
                Arc::new(move |s| match s.get("name")? {
                    css::Value::Literal(name, _) => {
                        let name = name.replace('-', "_").replace('.', "_");
                        for (n, v) in existing_statics.as_ref() {
                            if name == *n {
                                return Ok(css::Value::Literal(
                                    v.clone(),
                                    Quotes::Double,
                                ));
                            }
                        }
                        Err(Error::S(format!(
                            "Static file {} not found",
                            name,
                        )))
                    }
                    name => Err(Error::badarg("string", &name)),
                }),
            ),
        );

        let file_context = FileContext::new();
        let (file_context, src) = file_context.file(src);
        let scss = parse_scss_file(&src).unwrap();
        let style = OutputStyle::Compressed;
        let css = style.write_root(&scss, &mut scope, &file_context).unwrap();
        self.add_file_data(&src.with_extension("css"), &css)
    }

    fn add_static(
        &mut self,
        path: &Path,
        rust_name: &str,
        url_name: &str,
        content: &Display,
        suffix: &str,
    ) -> io::Result<()> {
        let rust_name = rust_name
            .replace("/", "_")
            .replace("-", "_")
            .replace(".", "_");
        writeln!(
            self.src,
            "\n/// From {path:?}\
             \n#[allow(non_upper_case_globals)]\
             \npub static {rust_name}: StaticFile = StaticFile {{\
             \n  content: {content},\
             \n  name: \"{url_name}\",\
             \n{mime}\
             }};",
            path = path,
            rust_name = rust_name,
            url_name = url_name,
            content = content,
            mime = mime_arg(suffix),
        )?;
        self.names.insert(rust_name.clone(), url_name.into());
        self.names_r.insert(url_name.into(), rust_name);
        Ok(())
    }

    /// Get a mapping of names, from without hash to with.
    ///
    /// ````
    /// # use ructe::{Result, Ructe, StaticFiles};
    /// # use std::fs::create_dir_all;
    /// # use std::path::PathBuf;
    /// # use std::vec::Vec;
    /// # fn main() -> Result<()> {
    /// # let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("test-tmp");
    /// # create_dir_all(&p);
    /// # let mut ructe = Ructe::new(p)?;
    /// let mut statics = ructe.statics()?;
    /// statics.add_file_data("black.css", b"body{color:black}\n");
    /// statics.add_file_data("blue.css", b"body{color:blue}\n");
    /// assert_eq!(
    ///     statics.get_names().iter()
    ///         .map(|(a, b)| format!("{} -> {}", a, b))
    ///         .collect::<Vec<_>>(),
    ///     vec!["black_css -> black-r3rltVhW.css".to_string(),
    ///          "blue_css -> blue-GZGxfXag.css".to_string()],
    /// );
    /// # Ok(())
    /// # }
    /// ````
    pub fn get_names(&self) -> &BTreeMap<String, String> {
        &self.names
    }
}

struct FileContent<'a>(&'a Path);

impl<'a> Display for FileContent<'a> {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        write!(out, "include_bytes!({:?})", self.0)
    }
}

struct ByteString<'a>(&'a [u8]);

impl<'a> Display for ByteString<'a> {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use std::ascii::escape_default;
        use std::str::from_utf8_unchecked;
        let escaped = self
            .0
            .iter()
            .flat_map(|c| escape_default(*c))
            .collect::<Vec<u8>>();
        write!(
            out,
            "b\"{}\"",
            // The above escaping makes sure t contains only printable ascii,
            // which is always valid utf8.
            unsafe { from_utf8_unchecked(&escaped) },
        )
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
    match suffix.to_lowercase().as_ref() {
        "bmp" => "image/bmp",
        "css" => "text/css",
        "eot" => "application/vnd.ms-fontobject",
        "gif" => "image/gif",
        "jpg" | "jpeg" => "image/jpeg",
        "js" | "jsonp" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

#[cfg(feature = "mime03")]
fn mime_arg(suffix: &str) -> String {
    format!("mime: &mime::{},\n", mime_from_suffix(suffix))
}

#[cfg(feature = "mime03")]
fn mime_from_suffix(suffix: &str) -> &'static str {
    // This is limited to the constants that is defined in mime 0.3.
    match suffix.to_lowercase().as_ref() {
        "bmp" => "IMAGE_BMP",
        "css" => "TEXT_CSS",
        "gif" => "IMAGE_GIF",
        "jpg" | "jpeg" => "IMAGE_JPEG",
        "js" | "jsonp" => "TEXT_JAVASCRIPT",
        "json" => "APPLICATION_JSON",
        "png" => "IMAGE_PNG",
        "svg" => "IMAGE_SVG",
        "woff" => "FONT_WOFF",
        "woff2" => "FONT_WOFF",
        _ => "APPLICATION_OCTET_STREAM",
    }
}

impl Drop for StaticFiles {
    /// Write the ending of the statics source code, declaring the
    /// `STATICS` variable.
    fn drop(&mut self) {
        // Ignore a possible write failure, rather than a panic in drop.
        let _ = writeln!(
            self.src,
            "\npub static STATICS: &'static [&'static StaticFile] \
             = &[{}];",
            self.names_r
                .iter()
                .map(|s| format!("&{}", s.1))
                .format(", "),
        );
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
    base64::encode_config(&md5::compute(data)[..6], base64::URL_SAFE)
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
        Ok(StaticFiles::for_template_dir(&self.outdir)?)
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
            .write_all(
                concat!(
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/src/template_utils.rs"
                    )),
                    "\n}\n"
                )
                .as_bytes(),
            )
            .unwrap();
    }
}

/// Create a `templates` module in `outdir` containing rust code for
/// all templates found in `indir`.
#[deprecated(since = "0.6", note = "Use method of struct Ructe instead")]
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
