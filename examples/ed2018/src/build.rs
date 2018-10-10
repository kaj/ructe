extern crate ructe;

use ructe::{compile_templates, StaticFiles};
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let in_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut statics = StaticFiles::new(&out_dir).unwrap();
    statics.add_files(&in_dir.join("static")).unwrap();
    statics.add_sass_file(&in_dir.join("style.scss")).unwrap();
    compile_templates(&in_dir.join("templates"), &out_dir).unwrap();
}
