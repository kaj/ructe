extern crate ructe;

use ructe::{StaticFiles, compile_templates};
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let base_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut statics = StaticFiles::new(&out_dir).unwrap();
    statics.add_files(&base_dir.join("static")).unwrap();
    statics.add_sass_file("scss/style.scss".as_ref()).unwrap();

    let template_dir = base_dir.join("templates");
    compile_templates(&template_dir, &out_dir).expect("templates");
}
