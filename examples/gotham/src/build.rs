//! This job builds rust source from static files and templates,
//! which can then be `include!`d in `main.rs`.
use ructe::{Result, Ructe};

fn main() -> Result<()> {
    let mut ructe = Ructe::from_env()?;
    ructe
        .statics()?
        .add_files("statics")?
        .add_sass_file("style.scss")?;
    ructe.compile_templates("templates")
}
