extern crate lemon_tree;

use lemon_tree::{lem_fn, LemonTree};

type Expr = f64;
type Exprs = Vec<f64>;

#[lem_fn("NUM(value)")] pub fn expr_1(value: f64) -> Expr {value}
#[lem_fn("MINUS Expr(a)")] pub fn expr_2(a: Expr) -> Expr {-a}
#[lem_fn("PLUS Expr(a)")] pub fn expr_3(a: Expr) -> Expr {a}
#[lem_fn("Expr(a) PLUS Expr(b)")] pub fn expr_4(a: Expr, b: Expr) -> Expr {a + b}
#[lem_fn("Expr(a) MINUS Expr(b)")] pub fn expr_5(a: Expr, b: Expr) -> Expr {a - b}
#[lem_fn("Expr(a) TIMES Expr(b)")] pub fn expr_6(a: Expr, b: Expr) -> Expr {a * b}
#[lem_fn("Expr(a) DIVIDE Expr(b)")] pub fn expr_7(a: Expr, b: Expr) -> Expr {a / b}
#[lem_fn("PAR_OPEN Expr(0) PAR_CLOSE")] pub fn expr_8(a: Expr) -> Expr {a}
#[lem_fn("Expr(item)")] pub fn exprs_1(item: Expr) -> Exprs {vec![item]}
#[lem_fn("Exprs(items) SEMICOLON Expr(item)")] pub fn exprs_2(mut items: Exprs, item: Expr) -> Exprs {items.push(item); items}

#[derive(LemonTree)]
#[lem_opt(token_type="f64", left="PLUS MINUS", left="DIVIDE TIMES", trace=">>")]
#[lem("Exprs(exprs) [SEMICOLON]")]
pub struct Program
{	exprs: Exprs,
}

fn parse(parser: &mut <Program as LemonTree>::Parser, mut input: &str) -> Exprs
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
fn calc_1()
{	let mut parser = Program::get_parser(());

	assert_eq!(parse(&mut parser, "2 + 2 * 2; (2+2) * 2"), vec![6.0, 8.0]);
	assert_eq!(parse(&mut parser, "2 * 2 + 2; (2*2) + 2"), vec![6.0, 6.0]);
	assert_eq!(parse(&mut parser, "-1*30"), vec![-30.0]);
	assert_eq!(parse(&mut parser, "0--1;"), vec![1.0]);
	assert_eq!(parse(&mut parser, "(((0)))"), vec![0.0]);
	assert_eq!(parse(&mut parser, "0.123 + 10"), vec![10.123]);
	assert_eq!(parse(&mut parser, "0.123 / (1.0-1.0)"), vec![f64::INFINITY]);
}
