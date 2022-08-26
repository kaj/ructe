use ructe::{Ructe, RucteError};

fn main() -> Result<(), RucteError> {
    let mut ructe = Ructe::from_env()?;
    ructe
        .statics()?
        .add_files("static")?
        .add_sass_file("scss/style.scss")?;
    ructe.compile_templates("templates")
}
