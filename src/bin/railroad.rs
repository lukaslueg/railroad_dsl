extern crate railroad_dsl;
// keeps compatibility with current stable, but unused on nightly
#[macro_use]
extern crate structopt;

use std::fs;
use std::path::PathBuf;
use std::io::{self, Read};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about = "Process railroad diagrams according to DSL.

If no input files are given, act as a pipe from stdin to stdout.
Otherwise, process each input file into an output file with
the file extension replaced by `.svg`.")]
struct Options {
    #[structopt(help = "Input files to process")]
    inputs: Vec<String>,
}


fn main() {
    let args = Options::from_args();
    if args.inputs.is_empty() {
        let mut buf = String::new();
        match io::stdin().read_to_string(&mut buf) {
            Err(e) => eprintln!("error reading stdin: {}", e),
            Ok(_) => match railroad_dsl::compile(&buf) {
                Err(e) => eprintln!("syntax error:\n{}", e.with_path("<stdin>")),
                Ok((_, _, diagram)) => println!("{}", diagram),
            }
        }
    } else {
        for input in args.inputs {
            let output = PathBuf::from(&input).with_extension("svg");
            match fs::read_to_string(&input) {
                Err(e) => eprintln!("error reading file {}: {}", input, e),
                Ok(buf) => match railroad_dsl::compile(&buf) {
                    Err(e) => eprintln!("syntax error:\n{}", e.with_path(&input)),
                    Ok((_, _, diagram)) => match fs::write(&output, format!("{}", diagram)) {
                        Err(e) => eprintln!("error writing file {}: {}", output.display(), e),
                        Ok(_) => ()
                    }
                }
            }
        }
    }
}
