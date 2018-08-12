#[macro_use]
extern crate nom;
extern crate railroad;

pub use railroad::svg::encode;

macro_rules! complete_named (
  ($name:ident, $submac:ident!( $($args:tt)* )) => (
    fn $name( i: nom::types::CompleteStr ) -> nom::IResult<nom::types::CompleteStr, nom::types::CompleteStr, u32> {
      $submac!(i, $($args)*)
    }
  );
  ($name:ident<$o:ty>, $submac:ident!( $($args:tt)* )) => (
    fn $name( i: nom::types::CompleteStr ) -> nom::IResult<nom::types::CompleteStr, $o, u32> {
      $submac!(i, $($args)*)
    }
  );
);


/*
// {} is a Stack
// [] is a Sequence
// <> is a Choice
// "" is a Term
// '' is a NonTerm
// `` is a Comment
// ? is a Option
// * is a Repeat with sep
// ! is Empty
*/

#[derive(Debug)]
enum Expr {
    Term(String),
    NonTerm(String),
    Comment(String),
    Sequence(Vec<Expr>),
    Stack(Vec<Expr>),
    Choice(Vec<Expr>),
    Optional(Box<Expr>),
    LabeledBox(Box<Expr>, Box<Expr>),
    Repeat(Box<Expr>, Box<Expr>),
    Empty
}

impl Into<Box<railroad::RailroadNode>> for Expr {
    fn into(self) -> Box<railroad::RailroadNode> {
        match self {
            Expr::Term(s) => Box::new(railroad::Terminal::new(s)),
            Expr::NonTerm(s) => Box::new(railroad::NonTerminal::new(s)),
            Expr::Comment(s) => Box::new(railroad::Comment::new(s)),
            Expr::Sequence(v) => Box::new(railroad::Sequence::new(v.into_iter().map(|e| e.into()).collect())),
            Expr::Stack(v) => Box::new(railroad::Stack::new(v.into_iter().map(|e| e.into()).collect())),
            Expr::Choice(v) => Box::new(railroad::Choice::new(v.into_iter().map(|e| e.into()).collect())),
            Expr::LabeledBox(i, o) => {
                Box::new(railroad::LabeledBox::new(
                        Into::<Box<railroad::RailroadNode>>::into(*i),
                        Into::<Box<railroad::RailroadNode>>::into(*o))
                    )
            },
            Expr::Optional(e) => Box::new(railroad::Optional::new(Into::<Box<railroad::RailroadNode>>::into(*e))),
            Expr::Repeat(i, o) => {
                Box::new(railroad::Repeat::new(
                        Into::<Box<railroad::RailroadNode>>::into(*i),
                        Into::<Box<railroad::RailroadNode>>::into(*o))
                    )
            },
            Expr::Empty => Box::new(railroad::Empty),
        }
    }
}

complete_named!(term<Expr>,
    do_parse!(
        s: ws!(delimited!(tag!("\""), take_until!("\""), tag!("\""))) >>
        ( Expr::Term(s.0.to_owned()) )
    )
);

complete_named!(nonterm<Expr>,
    do_parse!(
        s: ws!(delimited!(tag!("'"), take_until!("'"), tag!("'"))) >>
        ( Expr::NonTerm(s.0.to_owned()) )
    )
);

complete_named!(comment<Expr>,
    do_parse!(
        s: ws!(delimited!(tag!("`"), take_until!("`"), tag!("`"))) >>
        ( Expr::Comment(s.0.to_owned()) )
    )
);

complete_named!(empty<Expr>, do_parse!( tag!("!") >> ( Expr::Empty )));

complete_named!(list<Vec<Expr>>,
    separated_list_complete!(ws!(tag!(",")), lbox_expr)
);

complete_named!(sequence<Expr>,
    do_parse!(
        ws!(tag!("[")) >>
        l: list >>
        ws!(tag!("]")) >>
        ( Expr::Sequence(l) )
    )
);

complete_named!(stack<Expr>,
    do_parse!(
        ws!(tag!("{")) >>
        l: list >>
        ws!(tag!("}")) >>
        ( Expr::Stack(l) )
    )
);

complete_named!(choice<Expr>,
    do_parse!(
        ws!(tag!("<")) >>
        l: list >>
        ws!(tag!(">")) >>
        ( Expr::Choice(l) )
    )
);

complete_named!(simple_expr<Expr>,
    ws!(alt!(term | nonterm | comment | sequence | stack | choice | empty))
);


complete_named!(opt_expr<Expr>,
    do_parse!(
        e: simple_expr >>
        o: many0!(tag!("?")) >>
        ( o.into_iter().fold(e, |cur, _| Expr::Optional(Box::new(cur))) )
    )
);

complete_named!(rpt_tail<Expr>,
    do_parse!(
        complete!(tag!("*")) >>
        e: opt_expr >>
        ( e )
    )
);

complete_named!(rpt_expr<Expr>,
    do_parse!(
        e: opt_expr >>
        rpt: opt!(rpt_tail) >>
        ( match rpt {
            None => e,
            Some(rpt) => Expr::Repeat(Box::new(e), Box::new(rpt))
        })
    )
);

complete_named!(lbox_tail<Expr>,
    do_parse!(
        complete!(tag!("#")) >>
        e: rpt_expr >>
        ( e )
    )
);

complete_named!(lbox_expr<Expr>,
    do_parse!(
        e: rpt_expr >>
        lbox: opt!(lbox_tail) >>
        ( match lbox {
            None => e,
            Some(lbox) => Expr::LabeledBox(Box::new(e), Box::new(lbox))
        })
    )
);

complete_named!(root_expr<Expr>, terminated!(lbox_expr, eof!()));

pub fn compile(src: &str) -> Result<(i64, i64, railroad::Diagram<railroad::Sequence>), nom::Err<nom::types::CompleteStr>> {
    let (unparsed, tree) = root_expr(nom::types::CompleteStr(src))?;
    assert!(unparsed.is_empty());
    let root = railroad::Sequence::new(vec![Box::new(railroad::SimpleStart), tree.into(), Box::new(railroad::SimpleEnd)]);
    let dia = railroad::Diagram::with_default_css(root);
    let width = (&dia as &railroad::RailroadNode).width();
    let height = (&dia as &railroad::RailroadNode).height();
    Ok((width, height, dia))
}
