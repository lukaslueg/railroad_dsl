use clap::Parser;
use railroad::DEFAULT_CSS;
use std::borrow;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(clap::Parser)]
#[command(
    author,
    version,
    about,
    long_about = "A small DSL to generate syntax-diagrams.

If no input files are given, act as a pipe from stdin to stdout. Otherwise, \
process each input file into an output file with the file extension replaced by `.svg`"
)]
struct Options {
    // Input files to process
    inputs: Vec<String>,
    // Alternative CSS file for the SVG
    #[arg(long, help = "Alternative CSS file")]
    css: Option<PathBuf>,
}

#[derive(Debug)]
enum Error {
    Parser(#[allow(dead_code)] Box<pest::error::Error<railroad_dsl::Rule>>),
    IO(#[allow(dead_code)] io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<Box<pest::error::Error<railroad_dsl::Rule>>> for Error {
    fn from(err: Box<pest::error::Error<railroad_dsl::Rule>>) -> Self {
        Self::Parser(err)
    }
}

fn dia_from_stdin(css: &str) -> Result<(), Error> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let diagram = railroad_dsl::compile(&buf, css).map_err(|e| Box::new(e.with_path("<stdin>")))?;
    println!("{}", diagram.diagram);
    Ok(())
}

fn dia_from_files(inputs: &[String], css: &str) -> Result<(), Error> {
    let mut err = Ok(());
    for input in inputs {
        let output = PathBuf::from(&input).with_extension("svg");
        let buf = match fs::read_to_string(input) {
            Err(e) => {
                eprintln!("error reading file {input}: {e}");
                err = Err(Error::IO(e));
                continue;
            }
            Ok(buf) => buf,
        };
        let diagram = match railroad_dsl::compile(&buf, css) {
            Err(e) => {
                eprintln!("syntax error:\n{}", e.clone().with_path(input));
                err = Err(Error::Parser(e));
                continue;
            }
            Ok(diagram) => diagram,
        };
        if let Err(e) = fs::write(&output, format!("{}", diagram.diagram)) {
            eprintln!("error writing file {}: {}", output.display(), e);
            err = Err(Error::IO(e));
        }
    }
    err
}

fn run(args: &Options) -> Result<(), Error> {
    let css = args
        .css
        .as_deref()
        .map(|f| fs::read_to_string(f).map(borrow::Cow::Owned))
        .unwrap_or_else(|| Ok(borrow::Cow::Borrowed(DEFAULT_CSS)))
        .map_err(Error::IO)?;

    if args.inputs.is_empty() {
        dia_from_stdin(&css)
    } else {
        dia_from_files(&args.inputs, &css)
    }
}

fn main() -> Result<(), Error> {
    let opts = Options::parse();
    run(&opts)
}
