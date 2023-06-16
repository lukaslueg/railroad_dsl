use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about = "Process railroad diagrams according to DSL.

If no input files are given, act as a pipe from stdin to stdout.
Otherwise, process each input file into an output file with
the file extension replaced by `.svg`.")]
struct Options {
    #[structopt(help = "Input files to process")]
    inputs: Vec<String>,
    #[structopt(
        help = "Alternative CSS file for the SVG",
        long = "css",
        parse(from_os_str)
    )]
    css: Option<PathBuf>,
}

enum Error {
    Parser(Box<pest::error::Error<railroad_dsl::Rule>>),
    IO(io::Error),
}

fn read_css(css: &Option<PathBuf>) -> Result<Option<String>, Error> {
    if let Some(f) = css {
        let css = fs::read_to_string(f).map_err(Error::IO)?;
        Ok(Some(css))
    } else {
        Ok(None)
    }
}

fn dia_from_stdin(css: Option<String>) -> Result<(), Error> {
    let mut buf = String::new();
    match io::stdin().read_to_string(&mut buf) {
        Err(e) => {
            eprintln!("error reading stdin: {e}");
            Err(Error::IO(e))
        }
        Ok(_) => match railroad_dsl::compile(&buf, css.as_deref()) {
            Err(e) => {
                eprintln!("syntax error:\n{}", e.clone().with_path("<stdin>"));
                Err(Error::Parser(e))
            }
            Ok(diagram) => {
                println!("{}", diagram.diagram);
                Ok(())
            }
        },
    }
}

fn dia_from_files(inputs: &[String], css: Option<String>) -> Result<(), Error> {
    let mut err = Ok(());
    for input in inputs {
        let output = PathBuf::from(&input).with_extension("svg");
        match fs::read_to_string(input) {
            Err(e) => {
                eprintln!("error reading file {input}: {e}");
                err = Err(Error::IO(e));
            }
            Ok(buf) => match railroad_dsl::compile(&buf, css.as_deref()) {
                Err(e) => {
                    eprintln!("syntax error:\n{}", e.clone().with_path(input));
                    err = Err(Error::Parser(e));
                }
                Ok(diagram) => {
                    if let Err(e) = fs::write(&output, format!("{}", diagram.diagram)) {
                        eprintln!("error writing file {}: {}", output.display(), e);
                        err = Err(Error::IO(e));
                    }
                }
            },
        }
    }
    err
}

fn run(args: &Options) -> Result<(), Error> {
    let css = read_css(&args.css)?;
    if args.inputs.is_empty() {
        dia_from_stdin(css)
    } else {
        dia_from_files(&args.inputs, css)
    }
}

fn main() {
    let opts = Options::from_args();
    if run(&opts).is_err() {
        ::std::process::exit(1);
    }
}
