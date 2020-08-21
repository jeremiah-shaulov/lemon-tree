use lemon_tree::{lem_fn, LemonTree, LemonTreeNode};

#[derive(LemonTreeNode, Debug, PartialEq)]
pub enum Expr
{	#[lem("NUM(0)")] Num(f64),
	#[lem("MINUS Expr(0)")] UnaryMinus(Box<Expr>),
	#[lem("PLUS Expr(0)")] UnaryPlus(Box<Expr>),
	#[lem("Expr(0) PLUS Expr(1)")] Plus(Box<Expr>, Box<Expr>),
	#[lem("Expr(0) MINUS Expr(1)")] Minus(Box<Expr>, Box<Expr>),
	#[lem("Expr(0) TIMES Expr(1)")] Times(Box<Expr>, Box<Expr>),
	#[lem("Expr(0) DIVIDE Expr(1)")] Divide(Box<Expr>, Box<Expr>),
}
#[lem_fn("PAR_OPEN Expr(0) PAR_CLOSE")] pub fn expr_from_par(a: Expr) -> Expr {a}

pub type Exprs = Vec<Expr>;
#[lem_fn("Expr(item)")] fn exprs_item(item: Expr) -> Exprs {vec![item]}
#[lem_fn("Exprs(items) SEMICOLON Expr(item)")] fn exprs_items(mut items: Exprs, item: Expr) -> Exprs {items.push(item); items}

#[derive(LemonTree, Debug)]
#[lem_opt(token_type="f64", left="PLUS MINUS", left="DIVIDE TIMES")]
#[lem("Exprs(exprs) [SEMICOLON]")]
pub struct Program
{	exprs: Vec<Expr>,
	flag: bool,
}

fn parse(parser: &mut <Program as LemonTree>::Parser, mut input: &str) -> Vec<Expr>
{	loop
	{	input = input.trim_start();
		match input.bytes().next()
		{	Some(c) => match c
			{	b'+' => parser.add_token(<Program as LemonTree>::Token::PLUS, 0.0).unwrap(),
				b'-' => parser.add_token(<Program as LemonTree>::Token::MINUS, 0.0).unwrap(),
				b'*' => parser.add_token(<Program as LemonTree>::Token::TIMES, 0.0).unwrap(),
				b'/' => parser.add_token(<Program as LemonTree>::Token::DIVIDE, 0.0).unwrap(),
				b'(' => parser.add_token(<Program as LemonTree>::Token::PAR_OPEN, 0.0).unwrap(),
				b')' => parser.add_token(<Program as LemonTree>::Token::PAR_CLOSE, 0.0).unwrap(),
				b';' => parser.add_token(<Program as LemonTree>::Token::SEMICOLON, 0.0).unwrap(),
				b'0' ..= b'9' | b'.' =>
				{	let pos = input.bytes().position(|c| !c.is_ascii_digit() && c!=b'.').unwrap_or(input.len());
					let value = input[.. pos].parse().unwrap();
					parser.add_token(<Program as LemonTree>::Token::NUM, value).unwrap();
					input = &input[pos-1 ..];
				}
				_ => panic!("Invalid token")
			}
			None =>
			{	return parser.end().unwrap().exprs;
			}
		}
		input = &input[1 ..];
	}
}

#[test]
fn tree_1()
{	use Expr::*;
	let mut parser = Program::get_parser(());

	assert_eq!
	(	parse(&mut parser, "2 + 2 * 2; (3+3) * 3"),
		vec!
		[	Plus
			(	Box::new(Num(2.0)),
				Box::new(Times(Box::new(Num(2.0)), Box::new(Num(2.0))))
			),
			Times
			(	Box::new(Plus(Box::new(Num(3.0)), Box::new(Num(3.0)))),
				Box::new(Num(3.0))
			)
		]
	);
}
