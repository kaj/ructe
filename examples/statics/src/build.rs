use ructe::{Ructe, RucteError};

fn main() -> Result<(), RucteError> {
    let mut ructe = Ructe::from_env()?;
    ructe.statics()?.add_files("static")?;
    ructe.compile_templates("templates")
}
