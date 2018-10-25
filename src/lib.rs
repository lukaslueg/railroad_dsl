extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate railroad as rr;

pub use rr::svg::encode;

use pest::Parser;
use pest::iterators::Pair;

#[derive(Parser)]
#[grammar = "parser.pest"]
struct RRParser;

fn unescape(pair: &Pair<Rule>) -> String {
    let s = pair.as_str();
    let mut result = String::with_capacity(s.len());
    let mut iter = s[1..s.len()-1].chars();
    while let Some(ch) = iter.next() {
        result.push(match ch {
            '\\' => iter.next().expect("no escaped char?"),
            _ => ch
        });
    }
    result
}

fn binary<F, T>(pair: Pair<Rule>, f: F) -> Box<rr::RailroadNode>
where T: rr::RailroadNode + 'static,
      F: FnOnce(Box<rr::RailroadNode>, Pair<Rule>) -> T
{
    let mut inner = pair.into_inner();
    let node = make_node(inner.next().expect("pair cannot be empty"));
    if let Some(pair) = inner.next() {
        Box::new(f(node, pair))
    } else {
        node
    }
}

fn make_node(pair: Pair<Rule>) -> Box<rr::RailroadNode> {
    use Rule::*;
    match pair.as_rule() {
        term      => Box::new(rr::Terminal::new(unescape(&pair))),
        nonterm   => Box::new(rr::NonTerminal::new(unescape(&pair))),
        comment   => Box::new(rr::Comment::new(unescape(&pair))),
        empty     => Box::new(rr::Empty),
        sequence  => Box::new(rr::Sequence::new(pair.into_inner().map(make_node).collect())),
        stack     => Box::new(rr::Stack::new(pair.into_inner().map(make_node).collect())),
        choice    => Box::new(rr::Choice::new(pair.into_inner().map(make_node).collect())),
        opt_expr  => binary(pair, |node, _| rr::Optional::new(node)),
        rpt_expr  => binary(pair, |first, second| rr::Repeat::new(first, make_node(second))),
        lbox_expr => binary(pair, |first, second| rr::LabeledBox::new(first, make_node(second))),
        _ => unreachable!(),
    }
}

fn start_to_end(root: Box<rr::RailroadNode>) -> Box<rr::RailroadNode> {
    Box::new(rr::Sequence::new(vec![Box::new(rr::SimpleStart), root, Box::new(rr::SimpleEnd)]))
}

pub fn compile(src: &str) -> Result<(i64, i64, rr::Diagram<Box<rr::RailroadNode>>),
                                    pest::error::Error<Rule>> {
    let mut result = RRParser::parse(Rule::input, src)?;
    let trees = result.next().expect("expected root_expr").into_inner();
    let mut trees: Vec<_> = trees.map(|p| start_to_end(make_node(p))).collect();
    let root = if trees.len() == 1 {
        trees.remove(0)
    } else {
        Box::new(rr::VerticalGrid::new(trees))
    };
    let dia = rr::Diagram::with_default_css(root);
    let width = (&dia as &rr::RailroadNode).width();
    let height = (&dia as &rr::RailroadNode).height();
    Ok((width, height, dia))
}
