# lemon-tree
Famous [Lemon Parser Generator](https://www.hwaci.com/sw/lemon/), designed as library that builds your parser transparently during cargo build. To describe parser rules you add annotation attributes to rust functions, structs and enums.

This crate uses [lemon-mint](https://crates.io/crates/lemon-mint) as backend.

## Installation

Put this to your project's Cargo.toml:

```
[dependencies]
lemon-tree = "0.1"
```

## Example

As first example, lets create a calculator program. Create empty cargo project, and put the following to it's main.rs:

```rust
extern crate lemon_tree;

use lemon_tree::{lem_fn, LemonTree};

type Expr = f64; // in lemon this corresponds to: %type Expr {f64}
type Exprs = Vec<f64>; // %type Exprs {Vec<f64>}

#[lem_fn("NUM(value)")] pub fn expr_1(value: f64) -> Expr {value} // Expr ::= NUM(value). value
#[lem_fn("MINUS Expr(a)")] pub fn expr_2(a: Expr) -> Expr {-a} // Expr ::= MINUS Expr(a). -a
#[lem_fn("PLUS Expr(a)")] pub fn expr_3(a: Expr) -> Expr {a} // Expr ::= PLUS Expr(a). a
#[lem_fn("Expr(a) PLUS Expr(b)")] pub fn expr_4(a: Expr, b: Expr) -> Expr {a + b} // Expr ::= Expr(a) PLUS Expr(b). a + b
#[lem_fn("Expr(a) MINUS Expr(b)")] pub fn expr_5(a: Expr, b: Expr) -> Expr {a - b} // Expr ::= Expr(a) MINUS Expr(b). a - b
#[lem_fn("Expr(a) TIMES Expr(b)")] pub fn expr_6(a: Expr, b: Expr) -> Expr {a * b} // Expr ::= Expr(a) TIMES Expr(b). a * b
#[lem_fn("Expr(a) DIVIDE Expr(b)")] pub fn expr_7(a: Expr, b: Expr) -> Expr {a / b} // Expr ::= Expr(a) DIVIDE Expr(b). a / b
#[lem_fn("PAR_OPEN Expr(a) PAR_CLOSE")] pub fn expr_8(a: Expr) -> Expr {a} // Expr ::= PAR_OPEN Expr(a) PAR_CLOSE. a
#[lem_fn("Expr(item)")] pub fn exprs_1(item: Expr) -> Exprs {vec![item]} // Exprs ::= Expr(item). vec![item]
#[lem_fn("Exprs(items) SEMICOLON Expr(item)")] pub fn exprs_2(mut items: Exprs, item: Expr) -> Exprs {items.push(item); items} // Exprs ::= Exprs(items) SEMICOLON Expr(item). items.push(item); items

// The start symbol is marked with #[derive(LemonTree)]
#[derive(LemonTree)]
#[lem_opt(token_type="f64", left="PLUS MINUS", left="DIVIDE TIMES", trace=">>")]
#[lem("Exprs(exprs) [SEMICOLON]")]
pub struct Program
{	exprs: Exprs,
}
// The above code generates the following lemon directives:
// %start_symbol {Program}
// %token_type {f64}
// %left PLUS MINUS.
// %left DIVIDE TIMES.
// Program ::= Exprs(exprs). Program {exprs}
// Program ::= Exprs(exprs) SEMICOLON. Program {exprs}

// This is tokenizer function. It takes "input", and feeds tokens to the "parser".
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
			{	// End of input
				// parser.end() returns Result<Program>
				return parser.end().unwrap().exprs;
			}
		}
		input = &input[1 ..];
	}
}

fn main()
{	let mut parser = Program::get_parser(());

	assert_eq!(parse(&mut parser, "2 + 2 * 2; (2+2) * 2"), vec![6.0, 8.0]);
	assert_eq!(parse(&mut parser, "2 * 2 + 2; (2*2) + 2"), vec![6.0, 6.0]);
	assert_eq!(parse(&mut parser, "-1*30"), vec![-30.0]);
	assert_eq!(parse(&mut parser, "0--1;"), vec![1.0]);
	assert_eq!(parse(&mut parser, "(((0)))"), vec![0.0]);
	assert_eq!(parse(&mut parser, "0.123 + 10"), vec![10.123]);
	assert_eq!(parse(&mut parser, "0.123 / (1.0-1.0)"), vec![f64::INFINITY]);
}
```

You can have several parsers in your project. Each parser must be completely described in one rust file, and `#[derive(LemonTree)]` (the start symbol) must appear the last in the file.

This crate exports 3 symbols: `lem_fn`, `LemonTree` and `LemonTreeNode`.

Need to mark start symbol with `#[derive(LemonTree)]`. This automatic derive trait allows to set parser options with `#[lem_opt()]` attribute, and parser rules with `#[lem()]` attribute.

Symbols other than the start symbol can be declared with module-global functions annotated with `#[lem_fn()]` attribute. This attribute must be exported to the current namespace with `use lemon_tree::lem_fn`.

Another option is to use `LemonTreeNode`:

```rust
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

#[derive(LemonTreeNode, Debug, PartialEq)]
pub enum Exprs
{	#[lem("Expr(0)")]
	Expr(Expr),

	#[lem("Exprs(0) SEMICOLON Expr(1)")]
	Exprs(Box<Exprs>, Expr),
}

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

fn main()
{	use Expr::*;
	let mut parser = Program::get_parser(());

	assert_eq!
	(	parse(&mut parser, "2 + 2 * 2; (2+2) * 2"),
		vec!
		[	Times
			(	Box::new(Plus(Box::new(Num(2.0)), Box::new(Num(2.0)))),
				Box::new(Num(2.0))
			),
			Plus
			(	Box::new(Num(2.0)),
				Box::new(Times(Box::new(Num(2.0)), Box::new(Num(2.0))))
			)
		]
	);
}
```

This allows to build syntax trees easily.
