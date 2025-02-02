use super::Result;
use itertools::Itertools;
use std::ascii::escape_default;
use std::collections::BTreeMap;
use std::fmt::{self, Display};
use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Handler for static files.
///
/// Apart from handling templates for dynamic content, ructe also
/// helps with constants for static content.
///
/// Most sites that need HTML templates also needs some static resources.
/// Maybe one or several CSS files, some javascript, and / or pictures.
/// A good way to reduce network round-trips is to use a far expires
/// header to tell the browser it can cache those files and don't need
/// to check if they have changed.
/// But what if the files do change?
/// Then pretty much the only way to make sure the browser gets the
/// updated file is to change the URL to the file as well.
///
/// Ructe can create content-dependent file names for static files.
/// If you have an `image.png`, ructe may call it `image-SomeHash.png`
/// where `SomeHash` is 8 url-safe base64 characters encoding 48 bits
/// of a md5 sum of the file.
///
/// Each static file will be available as a
/// [`StaticFile`](templates/statics/index.html) struct instance in
/// your `templates::statics` module.
/// Also, the const `STATICS` array in the same module will contain a
/// reference to each of those instances.
///
/// Actually serving the file is a job for a web framework like
/// [axum](https://lib.rs/crates/axum),
/// [warp](https://lib.rs/crates/warp), or
/// [rocket](https://rocket.rs/), but ructe helps by packing the file
/// contents into a constant struct that you can access from rust
/// code.
///
/// # Overview
///
/// This section describes how to set up your project to serve
/// static content using ructe.
///
/// To do this, the first step is to add a line in `build.rs` telling
/// ructe to find and transpile your static files:
///
/// ```no_run
/// # use ructe::{Ructe, RucteError};
/// # fn main() -> Result<(), RucteError> {
/// let mut ructe = Ructe::from_env()?;
/// ructe.statics()?.add_files("static")?;
/// # Ok(())
/// # }
/// ```
///
/// Then you need to link to the encoded file.
/// For an image, you probably want to link it from an `<img>` tag in
/// a template.  That can be done like this:
///
/// ```html
/// @use super::statics::image_png;
/// @()
/// <img alt="Something" src="/static/@image_png.name">
/// ```
///
/// So, what has happened here?
/// First, assuming the `static` directory in your
/// `$CARGO_MANIFEST_DIR` contained a file name `image.png`, your
/// `templates::statics` module (which is reachable as `super::statics`
/// from inside a template) will contain a
/// `pub static image_png: StaticFile` which can be imported and used
/// in both templates and rust code.
/// A `StaticFile` has a field named `name` which is a `&'static str`
/// containing the name with the generated hash, `image-SomeHash.png`.
///
/// The next step is that a browser actually sends a request for
/// `/static/image-SomeHash.png` and your server needs to deliver it.
/// Here, things depend on your web framework, so we start with some
/// pseudo code.
/// Full examples for [axum], [warp], and [gotham] is
/// available [in the ructe repository].
///
/// [axum]: https://lib.rs/crates/axum
/// [warp]: https://crates.rs/crates/warp
/// [gotham]: https://crates.rs/crates/gotham
/// [in the ructe repository]: https://github.com/kaj/ructe/tree/master/examples
///
/// ```ignore
/// /// A hypothetical web framework calls this each /static/... request,
/// /// with the name component of the URL as the name argument.
/// fn serve_static(name: &str) -> Response {
///     if let Some(data) = StaticFile::get(name) {
///         Response::Ok(data.content)
///     } else {
///         Response::NotFound
///     }
/// }
/// ```
///
/// The `StaticFile::get` function returns the `&'static StaticFile`
/// for a given file name if the file exists.
/// This is a reference to the same struct that we used by the name
/// `image_png` in the template.
/// Besides the `name` field (which will be equal to the argument, or
/// `get` would not have returned this `StaticFile`), there is a
/// `content: &'static [u8]` field which contains the actual file
/// data.
///
/// # Content-types
///
/// How to get the content type of static files.
///
/// Ructe has support for making the content-type of each static
/// file availiable using the
/// [mime](https://crates.io/crates/mime) crate.
///
/// To use the mime 0.3.x support, enable the `mime3` feature and
/// add mime 0.3.x as a dependency:
///
/// ```toml
/// [build-dependencies]
/// ructe = { version = "^0.3.2", features = ["mime03"] }
///
/// [dependencies]
/// mime = "~0.3"
/// ```
///
/// From version 0.3, the `mime` crates supports creating const
/// static `Mime` objects, so with this feature, a `StaticFile`
/// simply has a `pub mime: &'static Mime` field.
///
/// ```
/// # // Test and doc even without the feature, so mock functionality.
/// # pub mod templates { pub mod statics {
/// # pub struct FakeFile { pub mime: &'static str }
/// # pub static image_png: FakeFile = FakeFile { mime: "image/png", };
/// # }}
/// use templates::statics::image_png;
///
/// # fn main() {
/// assert_eq!(format!("Type is {}", image_png.mime),
///            "Type is image/png");
/// # }
/// ```
pub struct StaticFiles {
    /// Rust source file `statics.rs` beeing written.
    src: Vec<u8>,
    /// Path for writing the file `statics.rs`.
    src_path: PathBuf,
    /// Base path for finding static files with relative paths
    base_path: PathBuf,
    /// Maps rust names to public names (foo_jpg -> foo-abc123.jpg)
    names: BTreeMap<String, String>,
    /// Maps public names to rust names (foo-abc123.jpg -> foo_jpg)
    names_r: BTreeMap<String, String>,
}

impl StaticFiles {
    pub(crate) fn for_template_dir(
        outdir: &Path,
        base_path: &Path,
    ) -> Result<Self> {
        let mut src = Vec::with_capacity(512);
        if cfg!(feature = "mime03") {
            src.write_all(b"use mime::Mime;\n\n")?;
        }
        if cfg!(feature = "tide013") {
            src.write_all(b"use tide::http::mime::{self, Mime};\n\n")?;
        } else if cfg!(feature = "http-types") {
            src.write_all(b"use http_types::mime::{self, Mime};\n\n")?;
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
        if cfg!(feature = "mime03") {
            src.write_all(b"    pub mime: &'static Mime,\n")?;
        }
        if cfg!(feature = "http-types") {
            src.write_all(b"    pub mime: &'static Mime,\n")?;
        }
        src.write_all(
            b"}
#[allow(dead_code)]
impl StaticFile {
    /// Get a single `StaticFile` by name, if it exists.
    #[must_use]
    pub fn get(name: &str) -> Option<&'static Self> {
        if let Ok(pos) = STATICS.binary_search_by_key(&name, |s| s.name) {
            Some(STATICS[pos])
        } else {None}
    }
}
",
        )?;
        Ok(StaticFiles {
            src,
            src_path: outdir.join("statics.rs"),
            base_path: base_path.into(),
            names: BTreeMap::new(),
            names_r: BTreeMap::new(),
        })
    }

    // Should the return type be some kind of cow path?
    fn path_for(&self, path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();
        if path.is_relative() {
            self.base_path.join(path)
        } else {
            path.into()
        }
    }

    /// Add all files from a specific directory, `indir`, as static files.
    pub fn add_files(
        &mut self,
        indir: impl AsRef<Path>,
    ) -> Result<&mut Self> {
        let indir = self.path_for(indir);
        println!("cargo:rerun-if-changed={}", indir.display());
        for entry in read_dir(indir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                self.add_file(entry.path())?;
            }
        }
        Ok(self)
    }

    /// Add all files from a specific directory, `indir`, as static files.
    ///
    /// The `to` string is used as a directory path of the resulting
    /// urls, the file names are taken as is, without adding any hash.
    /// This is usefull for resources used by preexisting javascript
    /// packages, where it might be hard to change the used urls.
    ///
    /// Note that some way of changing the url when the content
    /// changes is still needed if you serve the files with far
    /// expire, and using this method makes that your responsibility
    /// rather than ructes.
    /// Either the file may have hashed names as is, or you may use
    /// the version number of a 3:rd party package as part of the `to`
    /// parameter.
    ///
    /// The `to` parameter may be an empty string.
    /// In that case, no extra slash is added.
    pub fn add_files_as(
        &mut self,
        indir: impl AsRef<Path>,
        to: &str,
    ) -> Result<&mut Self> {
        for entry in read_dir(self.path_for(indir))? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let to = if to.is_empty() {
                entry.file_name().to_string_lossy().to_string()
            } else {
                format!("{}/{}", to, entry.file_name().to_string_lossy())
            };
            if file_type.is_file() {
                self.add_file_as(entry.path(), &to)?;
            } else if file_type.is_dir() {
                self.add_files_as(entry.path(), &to)?;
            }
        }
        Ok(self)
    }

    /// Add one specific file as a static file.
    ///
    /// Create a name to use in the url like `name-hash.ext` where
    /// name and ext are the name and extension from `path` and has is
    /// a few url-friendly bytes from a hash of the file content.
    ///
    pub fn add_file(&mut self, path: impl AsRef<Path>) -> Result<&mut Self> {
        let path = self.path_for(path);
        if let Some((name, ext)) = name_and_ext(&path) {
            println!("cargo:rerun-if-changed={}", path.display());
            let mut input = File::open(&path)?;
            let mut buf = Vec::new();
            input.read_to_end(&mut buf)?;
            let rust_name = format!("{name}_{ext}");
            let url_name = format!("{name}-{}.{ext}", checksum_slug(&buf));
            self.add_static(
                &path,
                &rust_name,
                &url_name,
                &FileContent(&path),
                ext,
            )?;
        }
        Ok(self)
    }

    /// Add one specific file as a static file.
    ///
    /// Use `url_name` in the url without adding any hash characters.
    pub fn add_file_as(
        &mut self,
        path: impl AsRef<Path>,
        url_name: &str,
    ) -> Result<&mut Self> {
        let path = &self.path_for(path);
        let ext = name_and_ext(path).map_or("", |(_, e)| e);
        println!("cargo:rerun-if-changed={}", path.display());
        self.add_static(path, url_name, url_name, &FileContent(path), ext)?;
        Ok(self)
    }

    /// Add a resource by its name and content, without reading an actual file.
    ///
    /// The `path` parameter is used only to create a file name, the actual
    /// content of the static file will be the `data` parameter.
    /// A hash will be added to the file name, just as for
    /// file-sourced statics.
    ///
    /// # Examples
    ///
    /// With the folloing code in `build.rs`:
    /// ````
    /// # use ructe::{Result, Ructe, StaticFiles};
    /// # use std::fs::create_dir_all;
    /// # use std::path::PathBuf;
    /// # use std::vec::Vec;
    /// # fn main() -> Result<()> {
    /// # let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("test-tmp").join("add-file");
    /// # create_dir_all(&p);
    /// # let mut ructe = Ructe::new(p)?;
    /// let mut statics = ructe.statics()?;
    /// statics.add_file_data("black.css", b"body{color:black}\n");
    /// # Ok(())
    /// # }
    /// ````
    ///
    /// A `StaticFile` named `black_css` will be defined in the
    /// `templates::statics` module of your crate:
    ///
    /// ````
    /// # mod statics {
    /// # use ructe::templates::StaticFile;
    /// # pub static black_css: StaticFile = StaticFile {
    /// #     content: b"body{color:black}\n",
    /// #     name: "black-r3rltVhW.css",
    /// #     #[cfg(feature = "mime03")]
    /// #     mime: &mime::TEXT_CSS,
    /// # };
    /// # }
    /// assert_eq!(statics::black_css.name, "black-r3rltVhW.css");
    /// ````
    pub fn add_file_data<P>(
        &mut self,
        path: P,
        data: &[u8],
    ) -> Result<&mut Self>
    where
        P: AsRef<Path>,
    {
        let path = &self.path_for(path);
        if let Some((name, ext)) = name_and_ext(path) {
            let rust_name = format!("{name}_{ext}");
            let url_name = format!("{name}-{}.{ext}", checksum_slug(data));
            self.add_static(
                path,
                &rust_name,
                &url_name,
                &ByteString(data),
                ext,
            )?;
        }
        Ok(self)
    }

    /// Compile a sass file and add the resulting css.
    ///
    /// If `src` is `"somefile.sass"`, then that file will be copiled
    /// with [rsass] (using the `Comressed` output style).
    /// The result will be added as if if was an existing
    /// `"somefile.css"` file.
    ///
    /// While handling the scss input, rsass is extended with a
    /// `static_name` sass function that takes a file name as given to
    /// [`add_file()`][Self::add_file] (or simliar) and returns the
    /// `name-hash.ext` filename that ructe creates for it.
    /// Note that only files that are added to the `StaticFiles`
    /// _before_ the call to `add_sass_files` are supported by the
    /// `static_name` function.
    ///
    /// This method is only available when ructe is built with the
    /// "sass" feature.
    #[cfg(feature = "sass")]
    pub fn add_sass_file<P>(&mut self, src: P) -> Result<&mut Self>
    where
        P: AsRef<Path>,
    {
        use rsass::css::CssString;
        use rsass::input::CargoContext;
        use rsass::output::{Format, Style};
        use rsass::sass::{CallError, FormalArgs};
        use rsass::value::Quotes;
        use rsass::*;
        use std::sync::Arc;
        let format = Format {
            style: Style::Compressed,
            precision: 4,
        };

        let src = self.path_for(src);
        let (context, scss) =
            CargoContext::for_path(&src).map_err(rsass::Error::from)?;
        let mut context = context.with_format(format);
        let existing_statics = self.get_names().clone();
        context.get_scope().define_function(
            "static_name".into(),
            sass::Function::builtin(
                "",
                &"static_name".into(),
                FormalArgs::new(vec![("name".into(), None)]),
                Arc::new(move |s| {
                    let name: String = s.get("name".into())?;
                    let rname = name.replace('-', "_").replace('.', "_");
                    existing_statics
                        .iter()
                        .find(|(n, _v)| *n == &rname)
                        .map(|(_n, v)| {
                            CssString::new(v.into(), Quotes::Double).into()
                        })
                        .ok_or_else(|| {
                            CallError::msg(format!(
                                "Static file {name:?} not found",
                            ))
                        })
                }),
            ),
        );

        let css = context.transform(scss)?;
        self.add_file_data(&src.with_extension("css"), &css)
    }

    fn add_static(
        &mut self,
        path: &Path,
        rust_name: &str,
        url_name: &str,
        content: &impl Display,
        suffix: &str,
    ) -> Result<&mut Self> {
        let mut rust_name =
            rust_name.replace(|c: char| !c.is_alphanumeric(), "_");
        if rust_name
            .as_bytes()
            .first()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(true)
        {
            rust_name.insert(0, 'n');
        }
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
        Ok(self)
    }

    /// Get a mapping of names, from without hash to with.
    ///
    /// ````
    /// # use ructe::{Result, Ructe, StaticFiles};
    /// # use std::fs::create_dir_all;
    /// # use std::path::PathBuf;
    /// # use std::vec::Vec;
    /// # fn main() -> Result<()> {
    /// # let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("test-tmp").join("get-names");
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

impl Drop for StaticFiles {
    /// Write the ending of the statics source code, declaring the
    /// `STATICS` variable.
    fn drop(&mut self) {
        // Ignore a possible write failure, rather than a panic in drop.
        let _ = writeln!(
            self.src,
            "\npub static STATICS: &[&StaticFile] \
             = &[{}];",
            self.names_r
                .iter()
                .map(|s| format!("&{}", s.1))
                .format(", "),
        );
        let _ = super::write_if_changed(&self.src_path, &self.src);
    }
}

struct FileContent<'a>(&'a Path);

impl Display for FileContent<'_> {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        write!(out, "include_bytes!({:?})", self.0)
    }
}

struct ByteString<'a>(&'a [u8]);

impl Display for ByteString<'_> {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        out.write_str("b\"")?;
        for byte in self.0 {
            escape_default(*byte).fmt(out)?;
        }
        out.write_str("\"")
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

#[cfg(all(feature = "mime03", feature = "http-types"))]
compile_error!(
    r#"Only one of the features "http-types" or "mime03" can be enabled at a time."#
);

/// A short and url-safe checksum string from string data.
fn checksum_slug(data: &[u8]) -> String {
    use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
    BASE64_URL_SAFE_NO_PAD.encode(&md5::compute(data)[..6])
}
fn mime_arg(#[allow(unused)] suffix: &str) -> String {
    #[cfg(not(any(feature = "mime03", feature = "http-types")))]
    let result = String::new();
    #[cfg(any(feature = "mime03", feature = "http-types"))]
    let result = format!("  mime: &mime::{},\n", mime_from_suffix(suffix));
    result
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

#[cfg(feature = "http-types")]
fn mime_from_suffix(suffix: &str) -> &'static str {
    match suffix.to_lowercase().as_ref() {
        "css" => "CSS",
        "html" | "htm" => "CSS",
        "ico" => "ICO",
        "jpg" | "jpeg" => "JPEG",
        "js" | "jsonp" => "JAVASCRIPT",
        "json" => "JSON",
        "png" => "PNG",
        "svg" => "SVG",
        "txt" => "PLAIN",
        "wasm" => "WASM",
        "xml" => "XML",
        _ => "mime::BYTE_STREAM",
    }
}
