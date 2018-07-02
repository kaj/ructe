//! This job builds rust source from static files and templates,
//! which can then be `include!`d in `main.rs`.
extern crate ructe;

use ructe::{compile_templates, StaticFiles};
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let base_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut statics = StaticFiles::new(&out_dir).unwrap();
    statics.add_files(&base_dir.join("statics")).unwrap();
    statics.add_sass_file(&base_dir.join("style.scss")).unwrap();
    compile_templates(&base_dir.join("templates"), &out_dir).unwrap();
}
