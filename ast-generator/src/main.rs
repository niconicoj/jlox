#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::process;

use clap::App;
use clap::Arg;

type GenResult<T> = Result<T, Box<dyn Error>>;

struct Config {
    source: String,
    directory: String,
}

fn main() {
    let config = match get_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    let ast_config = match parse_source(config.source) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to parse ast config file : {}", e);
            process::exit(1);
        }
    };
    for (basename, types) in ast_config.into_iter() {
        if let Err(e) = define_ast(&config.directory, &basename, types) {
            eprint!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn parse_source(source_path: String) -> GenResult<HashMap<String, Vec<String>>> {
    let source_file = File::open(source_path)?;
    let reader = BufReader::new(source_file);

    let config = serde_json::from_reader(reader)?;

    Ok(config)
}

fn get_args() -> GenResult<Config> {
    let matches = App::new("ast-generator")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("source")
                .help("source file")
                .short("s")
                .long("source")
                .default_value("./ast.json"),
        )
        .arg(
            Arg::with_name("directory")
                .help("target directory")
                .short("d")
                .long("directory")
                .default_value("./"),
        )
        .get_matches();
    let source = matches.value_of("source").unwrap();
    let directory = matches.value_of("directory").unwrap();
    Ok(Config {
        source: source.to_string(),
        directory: directory.to_string(),
    })
}

fn define_ast(directory: &String, basename: &String, types: Vec<String>) -> GenResult<()> {
    let mut writer = File::create(format!("{}/{}.java", directory, basename))?;
    writer.write(b"import java.util.List;\n\n")?;
    writer.write(format!("abstract class {} {{\n", basename).as_bytes())?;
    define_visitor(&mut writer, &basename, &types)?;
    for t in types.iter() {
        let mut t = t.split(":");
        let classname = t.next().ok_or("no class name")?.trim();
        let fields = t.next().ok_or("no fields")?.trim();
        define_type(
            &mut writer,
            &basename,
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
    basename: &String,
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

fn define_visitor(writer: &mut File, basename: &String, types: &Vec<String>) -> GenResult<()> {
    writer.write(b"    interface Visitor<R> {\n")?;
    for t in types.iter() {
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
