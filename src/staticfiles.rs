use super::Result;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::env;
use std::fmt::{self, Display};
use std::fs::{create_dir_all, read_dir, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

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
        since = "0.6.0",
        note = "Use the statics() method of struct Ructe instead"
    )]
    pub fn new(outdir: &Path) -> io::Result<Self> {
        let outdir = outdir.join("templates");
        create_dir_all(&outdir)?;
        StaticFiles::for_template_dir(&outdir)
    }

    pub fn for_template_dir(outdir: &Path) -> io::Result<Self> {
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
