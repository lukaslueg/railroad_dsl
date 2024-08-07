use railroad_dsl;
use std::fs;
use std::io;
use std::io::{Read, Write};

fn output(outname: &str, style: &railroad::Stylesheet) -> Result<(), io::Error> {
    let mut outp = fs::File::create(outname)?;
    outp.write_all(b"<html>")?;

    let mut paths = fs::read_dir("./examples")?
        .into_iter()
        .filter_map(|d| d.ok())
        .collect::<Vec<_>>();
    paths.sort_by_key(|e| e.file_name());

    for path in paths {
        if let Some(filename) = path.file_name().to_str() {
            if filename.ends_with("diagram.txt") {
                println!("Generating from `{}`", filename);
                let mut buffer = String::new();
                fs::File::open(path.path())?.read_to_string(&mut buffer)?;
                let diagram = railroad_dsl::compile(&buffer, style.stylesheet()).unwrap();
                write!(outp, "<h3>Generated from <i>`{}`</i></h3>", filename)?;
                write!(
                    outp,
                    "<pre>{}</pre><br>",
                    railroad::svg::encode_minimal(&buffer)
                )?;
                write!(outp, "<div style=\"width: {}px; height: auto; max-height: 100%, max-width: 100%\">{}</div>", diagram.width, diagram.diagram)?;
                outp.write_all(b"<hr>")?;
            }
        }
    }

    outp.write_all(b"</html>")?;

    println!("Written result to `{outname}`");

    Ok(())
}

fn main() -> Result<(), io::Error> {
    output(
        "examples/example_diagrams_light.html",
        &railroad::Stylesheet::Light,
    )?;
    output(
        "examples/example_diagrams_dark.html",
        &railroad::Stylesheet::Dark,
    )
}
