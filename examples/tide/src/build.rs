//! This job builds rust source from templates,
//! which can then be `include!`d in `main.rs`.

use ructe::{Ructe, RucteError};

fn main() -> Result<(), RucteError> {
    let mut ructe = Ructe::from_env()?;
    let mut statics = ructe.statics()?;
    statics.add_files("statics")?;
    statics.add_sass_file("style.scss")?;
    ructe.compile_templates("templates")
}
