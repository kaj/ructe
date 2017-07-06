// This module is only a chapter of the documentation.
#![allow(non_snake_case)]
//! Apart from handling templates for dynamic content, ructe also
//! helps with constants for static content.
//!
//! Most sites that need HTML templates also needs some static resources.
//! Maybe one or several CSS files, some javascript, and / or pictures.
//! A good way to reduce network round-trips is to use a far expires
//! header to tell the browser it can cache those files and don't need
//! to check if they have changed.
//! But what if the files do change?
//! Then pretty much the only way to make sure the browser gets the
//! updated file is to change the URL to the file as well.
//!
//! Ructe can create content-dependent file names for static files.
//! If you have an `image.png`, ructe may call it `image-SomeHash.png`
//! where `SomeHash` is 8 url-safe base64 characters encoding 48 bits
//! of a md5 sum of the file.
//!
//! Actually serving the file is a job for a web framework like
//! [iron](https://github.com/iron/iron),
//! [nickel](https://github.com/nickel-org/nickel.rs) or
//! [rocket](https://rocket.rs/), but ructe helps by packing the file
//! contents into a constant struct that you can access from rust
//! code.


pub mod a_Overview {
    //! This section describes how to set up your project to serve
    //! static content using ructe.
    //!
    //! To do this, the first step is to add a line in `build.rs` telling
    //! ructe to find and transpile your static files:
    //!
    //! ```no_run
    //! # use ructe::{compile_static_files, compile_templates};
    //! # use std::env;
    //! # use std::path::PathBuf;
    //! let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    //! let in_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    //! compile_static_files(&in_dir.join("static"), &out_dir).unwrap();
    //! compile_templates(&in_dir.join("templates"), &out_dir).unwrap();
    //! ```
    //!
    //! Then you need to link to the encoded file.
    //! For an image, you probably want to link it from an `<img>` tag in
    //! a template.  That can be done like this:
    //!
    //! ```html
    //! @use templates::statics::image_png;
    //! @()
    //! <img alt="Something" src="/static/@image_png.name">
    //! ```
    //!
    //! So, what has happened here?
    //! First, assuming the `static` directory in your
    //! `$CARGO_MANIFEST_DIR` contained a file name `image.png`, your
    //! `templates::statics` module will contain a
    //! `pub static image_png: StaticFile` which can be imported and used
    //! in both templates and rust code.
    //! A `StaticFile` has a field named `name` which is a `&'static str`
    //! containing the name with the generated hash, `image-SomeHash.png`.
    //!
    //! The next step is that a browser actually sends a request for
    //! `/static/image-SomeHash.png` and your server needs to deliver it.
    //! Here, things depend on your web framework, so we start with some
    //! pseudo code, and then goes on to working examples for iron and
    //! nickel.
    //!
    //! ```ignore
    //! /// A hypothetical web framework calls this each /static/ request,
    //! /// with the name component of the URL as the name argument.
    //! fn serve_static(name: &str) -> HttpResult {
    //!     if let Some(data) = StaticFile::get(name) {
    //!         HttpResult::Ok(data.content)
    //!     } else {
    //!         HttpResult::NotFound
    //!     }
    //! }
    //! ```
    //!
    //! The `StaticFile::get` function returns the `&'static StaticFile`
    //! for a given file name if the file exists.
    //! This is a reference to the same struct that we used by the name
    //! `image_png` in the template.
    //! Besides the `name` field (which will be equal to the argument, or
    //! `get` would not have returned this `StaticFile`), there is a
    //! `content: &'static [u8]` field which contains the actual file
    //! data.
}

pub mod b_Content_types {
    //! TODO: Write this section
}

pub mod c_Iron {
    //! How to serve static files with the Iron web framework.
    //!
    //! Somewhere (maybe in `main`), you probably create a router.
    //! To add a static handler could look something like this:
    //!
    //! ```ignore
    //! // Somewhere (maybe in main) you create a router
    //! let mut router = Router::new();
    //! // Among the routes, you add this:
    //! router.get("/static/:name", static_file, "static_file");
    //! // Go on to start an Iron server with the router
    //! ```
    //!
    //! Then the actual handler needs to be implemented.
    //! Here's one implementation.
    //!
    //! ```ignore
    //! fn static_file(req: &mut Request) -> IronResult<Response> {
    //!     // Extract the requested file name from the router
    //!     let router = req.extensions.get::<Router>().expect("router");
    //!     let name = router.find("name").expect("name");
    //!     // If the static files exists, serve it
    //!     if let Some(data) = statics::StaticFile::get(name) {
    //!         Ok(Response::with((status::Ok, data.mime(), data.content)))
    //!     } else {
    //!         debug!("Static file {} not found", name);
    //!         Ok(Response::with((
    //!             status::NotFound,
    //!             mime!(Text / Plain),
    //!             "not found",
    //!         )))
    //!     }
    //! }
    //! ```
    //!
    //! This implementation uses the `mime02` feature of ructe.
    //! Relevant parts of `Cargo.toml` might look like this:
    //!
    //! ```toml
    //! [build-dependencies]
    //! ructe = { version = "^0.3.2", features = ["sass", "mime02"] }
    //!
    //! [dependencies]
    //! iron = "^0.5.1"
    //! router = "^0.5.1"
    //! mime = "0.2.6"
    //! ```
}

pub mod d_Nickel {
    //! TODO: Write this section
}
