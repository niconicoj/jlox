use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::process;

use clap::App;
use clap::Arg;

type GenResult<T> = Result<T, Box<dyn Error>>;

struct Config {
    directory: String,
    basename: String,
}

const TYPES: [&str; 4] = [
    "Binary : Expr left, Token operator, Expr right",
    "Grouping : Expr expression",
    "Literal : Object value",
    "Unary : Token operator, Expr right",
];

fn main() {
    let config = match get_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    if let Err(e) = define_ast(config) {
        eprint!("Error: {}", e);
        process::exit(1);
    }
}

fn get_args() -> GenResult<Config> {
    let matches = App::new("ast-generator")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("basename")
                .value_name("BASENAME")
                .help("class Base name")
                .default_value("Expr"),
        )
        .arg(
            Arg::with_name("directory")
                .help("Output directory")
                .short("d")
                .long("directory")
                .default_value("./"),
        )
        .get_matches();
    let directory = matches.value_of("directory").unwrap();
    let basename = matches.value_of("basename").unwrap();
    Ok(Config {
        directory: directory.to_string(),
        basename: basename.to_string(),
    })
}

fn define_ast(config: Config) -> GenResult<()> {
    let mut writer = File::create(format!("{}/{}.java", config.directory, config.basename))?;
    writer.write(b"import java.util.List;\n\n")?;
    writer.write(format!("abstract class {} {{\n", config.basename).as_bytes())?;
    define_visitor(&mut writer, config.basename.clone())?;
    for t in TYPES.iter() {
        let mut t = t.split(":");
        let classname = t.next().ok_or("no class name")?.trim();
        let fields = t.next().ok_or("no fields")?.trim();
        define_type(
            &mut writer,
            config.basename.clone(),
            classname.to_string(),
            fields.to_string(),
        )?;
    }
    writer.write(b"    abstract <R> R accept(Visitor<R> visitor);")?;
    writer.write(b"\n}\n")?;
    writer.flush()?;
    Ok(())
}

fn define_type(
    writer: &mut File,
    basename: String,
    classname: String,
    fields: String,
) -> GenResult<()> {
    writer.write(format!("    static class {} extends {} {{\n", classname, basename).as_bytes())?;
    writer.write(format!("        {}({}) {{\n", classname, fields).as_bytes())?;
    for field in fields.split(", ") {
        let name = field.split(" ").nth(1).ok_or("field doesn't have name")?;
        writer.write(format!("            this.{} = {};\n", name, name).as_bytes())?;
    }
    writer.write(format!("        }}\n\n").as_bytes())?;
    writer.write(b"        @Override\n")?;
    writer.write(b"        <R> R accept(Visitor<R> visitor) {\n")?;
    writer.write(
        format!(
            "            return visitor.visit{}{}(this);\n",
            classname, basename
        )
        .as_bytes(),
    )?;
    writer.write(b"        }\n\n")?;
    for field in fields.split(", ") {
        writer.write(format!("        final {};\n", field).as_bytes())?;
    }
    writer.write(b"    }\n\n")?;
    Ok(())
}

fn define_visitor(writer: &mut File, basename: String) -> GenResult<()> {
    writer.write(b"    interface Visitor<R> {\n")?;
    for t in TYPES.iter() {
        let type_name = t
            .split(":")
            .nth(0)
            .ok_or("type doesn't have a name")?
            .trim();
        writer.write(
            format!(
                "        R visit{}{}({} {});\n",
                type_name,
                basename,
                type_name,
                basename.to_lowercase()
            )
            .as_bytes(),
        )?;
    }
    writer.write(b"    }\n\n")?;
    Ok(())
}
