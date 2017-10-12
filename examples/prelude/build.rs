extern crate ructe;

fn main() {
    ructe::compile_templates_cargo("pages").expect("Failed to compile templates");
}
