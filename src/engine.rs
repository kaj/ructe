use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use template;

pub trait RenderEngine {
    fn sublevel(&self, name: &str) -> io::Result<Box<Self>>;
    fn render(&self, name: &str, tpl: &template::Template) -> io::Result<()>;
}

pub struct Engine<'a> {
    top_level: bool,
    top_level_name: &'a str,
    outpath: PathBuf,
    mod_file: File,
}

pub const TEMPLATE_UTILS: &'static [u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/template_utils.rs"
));

impl<'a> Engine<'a> {
    pub fn new(target_path: &Path, name: &'a str) -> io::Result<Self> {
        fs::create_dir_all(target_path.join(name))?;

        let mut f = File::create(target_path.join(name.to_string() + ".rs"))?;

        write!(
            &f,
            "pub mod {} {{\n\
            use std::io::{{self, Write}};\n\
            use std::fmt::Display;\n\n",
            name
        )?;

        f.write(TEMPLATE_UTILS)?;

        Ok(Engine {
            top_level: true,
            top_level_name: name,
            outpath: target_path.join(name),
            mod_file: f,
        })
    }

    pub fn add_mod(&self, name: &str) -> io::Result<()> {
        write!(&self.mod_file, "pub mod {};\n\n", name)
    }
}

impl<'a> RenderEngine for Engine<'a> {
    fn sublevel(&self, name: &str) -> io::Result<Box<Self>> {
        let path = self.outpath.join(name);
        fs::create_dir_all(&path)?;

        let ret = Engine {
            top_level: false,
            top_level_name: self.top_level_name,
            mod_file: File::create(path.join("mod.rs"))?,
            outpath: path,
        };

        self.add_mod(name)?;

        Ok(Box::new(ret))
    }

    fn render(&self, name: &str, tpl: &template::Template) -> io::Result<()> {
        File::create(self.outpath.join(format!("template_{}.rs", name)))
            .and_then(
                |mut f| write_rust(&mut f, tpl, name, self.top_level_name),
            )?;

        write!(
            &self.mod_file,
            "mod template_{name};\n\
                pub use self::template_{name}::{name};\n\n",
            name = name
        )
    }
}

impl<'a> Drop for Engine<'a> {
    fn drop(&mut self) {
        if self.top_level {
            self.mod_file.write(b"}\n").unwrap();
        }
    }
}

pub fn write_rust(
    out: &mut Write,
    tpl: &template::Template,
    name: &str,
    top_level_name: &str,
) -> io::Result<()> {
    write!(
        out,
        "use std::io::{{self, Write}};\n\
            #[allow(unused)]\n\
            use ::{}::{{Html,ToHtml}};\n",
        top_level_name
    )?;

    for l in &tpl.preamble {
        write!(out, "{};\n", l)?;
    }

    let type_args = if tpl.args.contains(&"content: Content".to_owned()) {
        (
            "<Content>",
            "\nwhere Content: FnOnce(&mut Write) \
          -> io::Result<()>",
        )
    } else {
        ("", "")
    };

    write!(
        out,
        "\n\
            pub fn {name}{type_args}(out: &mut Write{args})\n\
            -> io::Result<()> {type_spec}{{\n\
            {body}\
            Ok(())\n\
            }}\n",
        name = name,
        type_args = type_args.0,
        args = tpl.args.iter().map(|a| format!(", {}", a)).collect::<String>(),
        type_spec = type_args.1,
        body = tpl.body.iter().map(|b| b.code()).collect::<String>()
    )
}
