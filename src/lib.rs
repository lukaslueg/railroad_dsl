#[macro_use]
extern crate pest_derive;
use railroad as rr;

use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser.pest"]
struct RRParser;

pub struct Diagram {
    pub width: i64,
    pub height: i64,
    pub diagram: rr::Diagram<Box<dyn rr::Node>>,
}

fn unescape(pair: &Pair<'_, Rule>) -> String {
    let s = pair.as_str();
    let mut result = String::with_capacity(s.len());
    let mut iter = s[1..s.len() - 1].chars();
    while let Some(ch) = iter.next() {
        result.push(match ch {
            '\\' => iter.next().expect("no escaped char?"),
            _ => ch,
        });
    }
    result
}

fn binary<F, T>(pair: Pair<'_, Rule>, f: F) -> Box<dyn rr::Node>
where
    T: rr::Node + 'static,
    F: FnOnce(Box<dyn rr::Node>, Pair<'_, Rule>) -> T,
{
    let mut inner = pair.into_inner();
    let node = make_node(inner.next().expect("pair cannot be empty"));
    if let Some(pair) = inner.next() {
        Box::new(f(node, pair))
    } else {
        node
    }
}

fn make_node(pair: Pair<'_, Rule>) -> Box<dyn rr::Node> {
    use Rule::*;
    match pair.as_rule() {
        term => Box::new(rr::Terminal::new(unescape(&pair))),
        nonterm => Box::new(rr::NonTerminal::new(unescape(&pair))),
        comment => Box::new(rr::Comment::new(unescape(&pair))),
        empty => Box::new(rr::Empty),
        sequence => Box::new(rr::Sequence::new(
            pair.into_inner().map(make_node).collect(),
        )),
        stack => Box::new(rr::Stack::new(pair.into_inner().map(make_node).collect())),
        choice => Box::new(rr::Choice::new(pair.into_inner().map(make_node).collect())),
        opt_expr => binary(pair, |node, _| rr::Optional::new(node)),
        rpt_expr => binary(pair, |first, second| {
            rr::Repeat::new(first, make_node(second))
        }),
        lbox_expr => binary(pair, |first, second| {
            rr::LabeledBox::new(first, make_node(second))
        }),
        _ => unreachable!(),
    }
}

fn start_to_end(root: Box<dyn rr::Node>) -> Box<dyn rr::Node> {
    Box::new(rr::Sequence::new(vec![
        Box::new(rr::SimpleStart) as Box<dyn rr::Node>,
        root,
        Box::new(rr::SimpleEnd),
    ]))
}

pub fn compile(src: &str) -> Result<Diagram, Box<pest::error::Error<Rule>>> {
    let mut result = RRParser::parse(Rule::input, src)?;
    let trees = result.next().expect("expected root_expr").into_inner();
    let mut trees: Vec<_> = trees.map(|p| start_to_end(make_node(p))).collect();
    let root = if trees.len() == 1 {
        trees.remove(0)
    } else {
        Box::new(rr::VerticalGrid::new(trees))
    };
    let diagram = rr::Diagram::with_default_css(root);
    let width = (&diagram as &dyn rr::Node).width();
    let height = (&diagram as &dyn rr::Node).height();
    Ok(Diagram {
        width,
        height,
        diagram,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::io::Read;
    use std::path;

    #[test]
    fn examples_must_parse() {
        let home = env::var_os("CARGO_MANIFEST_DIR").unwrap();
        let mut exmpl_dir = path::PathBuf::from(home);
        exmpl_dir.push("examples");
        for path in fs::read_dir(exmpl_dir).unwrap().filter_map(Result::ok) {
            if let Some(filename) = path.file_name().to_str() {
                if filename.ends_with("diagram.txt") {
                    eprintln!("Compiling `{filename}`");
                    let mut buffer = String::new();
                    fs::File::open(path.path())
                        .unwrap()
                        .read_to_string(&mut buffer)
                        .unwrap();
                    if let Err(e) = compile(&buffer) {
                        panic!("Failed to compile {}", e.with_path(filename));
                    }
                }
            }
        }
    }
}
