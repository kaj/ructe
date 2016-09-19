extern crate ructe;

use ructe::compile_templates;
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let in_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("templates");
    let templates = ["hello", "hello_args", "hello_args_two",
                     "hello_fields", "hello_code"];
    for t in templates.iter() {
        println!("cargo:rerun-if-changed=templates/{}.rs.html", t);
    }
    compile_templates(&in_dir, &out_dir, &templates).expect("foo");
}
