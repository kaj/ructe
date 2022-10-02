use ructe::{Ructe, RucteError};

fn main() -> Result<(), RucteError> {
    let mut ructe = Ructe::from_env()?;
    ructe
        .statics()?
        .add_files("statics")?
        .add_sass_file("style.scss")?;
    ructe.compile_templates("templates")
}
