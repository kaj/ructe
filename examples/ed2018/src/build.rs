use ructe::{Result, Ructe};

fn main() -> Result<()> {
    let mut ructe = Ructe::from_env()?;
    let mut statics = ructe.statics()?;
    statics.add_files("static")?;
    statics.add_sass_file("style.scss")?;
    ructe.compile_templates("templates")
}
