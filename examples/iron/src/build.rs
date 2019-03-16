//! This job builds rust source from static files and templates,
//! which can then be `include!`d in `main.rs`.
//!
//! This build scritps uses deprecated functionality, mainly to have
//! something still use it until I actually remove it.
extern crate ructe;

use ructe::{compile_templates, Result, StaticFiles};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let base_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut statics = StaticFiles::new(&out_dir)?;
    statics.add_files(&base_dir.join("statics"))?;
    statics.add_sass_file(&base_dir.join("style.scss"))?;
    compile_templates(&base_dir.join("templates"), &out_dir)
}
