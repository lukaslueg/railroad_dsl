use clap::Parser;
use std::borrow;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Clone, clap::ValueEnum)]
#[allow(clippy::upper_case_acronyms)]
enum Format {
    SVG,
    PNG,
}

#[derive(Clone, clap::ValueEnum)]
enum Theme {
    Light,
    Dark,
}

impl Theme {
    fn to_stylesheet(&self, fmt: &Format) -> railroad::Stylesheet {
        match (self, fmt) {
            (Self::Light, Format::SVG) => railroad::Stylesheet::Light,
            (Self::Dark, Format::SVG) => railroad::Stylesheet::Dark,
            (Self::Light, Format::PNG) => railroad::Stylesheet::LightRendersafe,
            (Self::Dark, Format::PNG) => railroad::Stylesheet::DarkRendersafe,
        }
    }
}

impl Format {
    fn file_extension(&self) -> &'static str {
        match self {
            Self::SVG => "svg",
            Self::PNG => "png",
        }
    }
}

#[derive(clap::Parser)]
#[command(
    author,
    version,
    about,
    long_about = "A small DSL to generate syntax-diagrams.

If no input files are given, act as a pipe from stdin to stdout. Otherwise, \
process each input file into an output file with the file extension replaced"
)]
struct Options {
    // Input files to process
    inputs: Vec<String>,
    // Alternative CSS file for the SVG
    #[arg(long, help = "Alternative CSS file")]
    css: Option<PathBuf>,
    #[arg(value_enum, long, help = "Output format", default_value_t=Format::SVG)]
    format: Format,
    #[arg(long, help = "Maximum width of the final image")]
    max_width: Option<u32>,
    #[arg(long, help = "Maximum height of the final image")]
    max_height: Option<u32>,
    #[arg(value_enum, long, help = "Theme to use", default_value_t=Theme::Light)]
    theme: Theme,
}

#[derive(Debug)]
enum Error {
    Parser(#[allow(dead_code)] Box<pest::error::Error<railroad_dsl::Rule>>),
    IO(#[allow(dead_code)] io::Error),
    Render(#[allow(dead_code)] railroad::render::Error),
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

impl From<railroad::render::Error> for Error {
    fn from(err: railroad::render::Error) -> Self {
        Self::Render(err)
    }
}

fn dia_from_stdin(
    css: &str,
    format: &Format,
    fit_to: &railroad::render::FitTo,
) -> Result<(), Error> {
    use std::io::Write;

    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let diagram = railroad_dsl::compile(&buf, css).map_err(|e| Box::new(e.with_path("<stdin>")))?;
    match format {
        Format::SVG => {
            println!("{}", diagram.diagram);
        }
        Format::PNG => {
            let png_buf = railroad::render::to_png(&diagram.diagram.to_string(), fit_to)?;
            std::io::stdout().write_all(&png_buf)?;
        }
    }
    Ok(())
}

fn dia_from_files(
    inputs: &[String],
    css: &str,
    format: &Format,
    fit_to: &railroad::render::FitTo,
) -> Result<(), Error> {
    let mut err = Ok(());
    for input in inputs {
        let output = PathBuf::from(&input).with_extension(format.file_extension());
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
        let buf = match format {
            Format::SVG => diagram.diagram.to_string().into_bytes(),
            Format::PNG => railroad::render::to_png(&diagram.diagram.to_string(), fit_to)?,
        };
        if let Err(e) = fs::write(&output, buf) {
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
        .unwrap_or(Ok(borrow::Cow::Borrowed(
            args.theme.to_stylesheet(&args.format).stylesheet(),
        )))
        .map_err(Error::IO)?;

    let fit_to = railroad::render::FitTo::from_size(args.max_width, args.max_height);

    if args.inputs.is_empty() {
        dia_from_stdin(&css, &args.format, &fit_to)
    } else {
        dia_from_files(&args.inputs, &css, &args.format, &fit_to)
    }
}

fn main() -> Result<(), Error> {
    let opts = Options::parse();
    run(&opts)
}
