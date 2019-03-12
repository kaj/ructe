// This module is only a chapter of the documentation.
//! This module describes how to configure and use Rust Compiled Templates.
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
//! ructe = "^0.3"
//! ```
//!
//! Then, in the build script, compile all templates found in the templates
//! directory and put the output where cargo tells it to:
//!
//! ```rust,no_run
//! use ructe::{Result, Ructe};
//!
//! fn main() -> Result<()> {
//!     Ructe::from_env()?.compile_templates("templates")
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
#![allow(non_snake_case)]
