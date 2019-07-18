//! This job builds rust source from static files and templates,
//! which can then be `include!`d in `main.rs`.
extern crate ructe;
use ructe::{Result, Ructe};

fn main() -> Result<()> {
    let mut ructe = Ructe::from_env()?;
    let mut statics = ructe.statics()?;
    statics.add_files("statics")?;
    statics.add_sass_file("style.scss")?;
    ructe.compile_templates("templates")
}
