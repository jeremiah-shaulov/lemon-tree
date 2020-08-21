//! Famous [Lemon Parser Generator](https://www.hwaci.com/sw/lemon/), designed as library that builds your parser transparently during cargo build.
//! To describe parser rules you add annotation attributes to rust functions, structs and enums.
//!
//! You can find usage examples [here](https://github.com/jeremiah-shaulov/lemon-tree).
//!
//! Let's say we want to create Lemon parser like this:
//!
//! ```ignore
//! %token_type {f64}
//! %left PLUS
//!
//! Unit ::= Expr(expr).
//! Expr ::= VALUE(value).
//! Expr ::= Expr(a) PLUS Expr(b).
//! ```
//!
//! We want that Unit and Expr will be represented by the following types in rust:
//!
//! ```ignore
//! struct Unit
//! {	expr: Expr,
//! }
//!
//! struct Expr
//! {	value: f64,
//! }
//! ```
//!
//! Every symbol except the start symbol we need to annotate with `#[derive(LemonTreeNode)]`, and the start symbol with `#[derive(LemonTree)]`.
//! Parser rules that return this symbol we put into `#[lem()]` annotation attributes.
//! All `#[derive(LemonTreeNode)]`, `#[derive(LemonTree)]` and `#[lem_fn()]` attributes that describe single Lemon parser must be contained in single rust file,
//! and the `#[derive(LemonTree)]` must come the last.
//!
//! ```ignore
//! #[derive(LemonTreeNode)]
//! #[lem("VALUE(value)")]
//! pub struct Expr
//! {	value: f64,
//! }
//!
//! #[derive(LemonTree)]
//! #[lem("Expr(expr)")]
//! pub struct Unit
//! {	expr: Expr,
//! }
//! ```
//!
//! The `#[lem()]` attribute can appear multiple times, and each attribute can contain multiple rules, like `#[lem("A(value)", "B(value)")]`.
//!
//! Each rule will produce code that creates new struct instance. Aliases given in parentheses will be assigned to struct fields.
//! If a struct has more fields that appear in expression, the remaining fields will be set to `Default::default()`, so they need to implement `std::default::Default` trait.
//! Existing fields will be assigned like this: `Type {field: value.into()}`. So field type in struct can be the type of value, or compatible with it.
//!
//! In example above, there's one Lemon rule, that doesn't return the final result, but needs to perform some calculation.
//! We expect that rule `Expr ::= Expr(a) PLUS Expr(b)` will produce type `Expr {value: a.value + b.value}`.
//! We can implement this rule as rust function:
//!
//! ```ignore
//! #[lem_fn("Expr(a) PLUS Expr(b)")]
//! pub fn expr_1(a: Expr, b: Expr) -> Expr
//! {	Expr {value: a.value + b.value}
//! }
//! ```
//!
//! So `#[lem_fn()]` attribute creates parser rule, whose action is module-global function call.
//! The return type of such function will be the left-hand side symbol in Lemon rule, like `Expr ::= Expr(a) PLUS Expr(b)`.
//!
//! To specify Lemon parser directives, like `%token_type {f64}`, need to use `#[lem_opt()]` attributes near start symbol, like `#[lem_opt(token_type="f64")]`.
//!
//! Here is complete example:
//!
//! ```
//! use lemon_tree::{lem_fn, LemonTree, LemonTreeNode};
//!
//! #[derive(LemonTreeNode, Debug)]
//! #[lem("VALUE(value)")]
//! pub struct Expr
//! {	value: f64,
//! }
//!
//! #[lem_fn("Expr(a) PLUS Expr(b)")]
//! pub fn expr_1(a: Expr, b: Expr) -> Expr
//! {	Expr {value: a.value + b.value}
//! }
//!
//! #[derive(LemonTree, Debug)]
//! #[lem("Expr(expr)")]
//! #[lem_opt(token_type="f64", left="PLUS")]
//! pub struct Unit
//! {	expr: Expr,
//! }
//!
//! fn main()
//! {	let mut parser = Unit::get_parser(());
//! 	parser.add_token(<Unit as LemonTree>::Token::VALUE, 10.0).unwrap();
//! 	parser.add_token(<Unit as LemonTree>::Token::PLUS, 0.0).unwrap();
//! 	parser.add_token(<Unit as LemonTree>::Token::VALUE, 20.0).unwrap();
//! 	let result = parser.end().unwrap();
//! 	assert_eq!(result.expr.value, 30.0);
//! 	println!("Result: {:?}", result);
//! }
//! ```
//!
//! Enums can be used as symbol types as well. With enums need to put `#[lem()]` parser rules near enum variants.
//! Example:
//!
//! ```
//! use lemon_tree::{lem_fn, LemonTree, LemonTreeNode};
//!
//! #[derive(LemonTreeNode, Debug, PartialEq)]
//! pub enum Expr
//! {	#[lem("VALUE(0)")]
//! 	Value(f64),
//!
//! 	#[lem("Expr(0) PLUS Expr(1)")]
//! 	Plus(Box<Expr>, Box<Expr>), // the generated action will look like: Expr::Plus(arg_0.into(), arg_1.into())
//! }
//!
//! #[derive(LemonTree, Debug, PartialEq)]
//! #[lem("Expr(expr)")]
//! #[lem_opt(token_type="f64", left="PLUS")]
//! pub struct Unit
//! {	expr: Expr,
//! }
//!
//! fn main()
//! {	let mut parser = Unit::get_parser(());
//! 	parser.add_token(<Unit as LemonTree>::Token::VALUE, 10.0).unwrap();
//! 	parser.add_token(<Unit as LemonTree>::Token::PLUS, 0.0).unwrap();
//! 	parser.add_token(<Unit as LemonTree>::Token::VALUE, 20.0).unwrap();
//! 	let result = parser.end().unwrap();
//! 	assert_eq!
//! 	(	result,
//! 		Unit
//! 		{	expr: Expr::Plus
//! 			(	Box::new(Expr::Value(10.0)),
//! 				Box::new(Expr::Value(20.0)),
//! 			)
//! 		}
//! 	);
//! 	println!("Result: {:?}", result);
//! }
//! ```
//!
//! Notice, that in `Expr::Plus` action, `Expr` object magically converted to `Box<Expr>`, because `Box<T>` implements `From<T>`, so `into()` can be used to convert.
//!
//! What if we want to do more complex conversion? Actually we can convert anything to anything, if we manually implement an `Into<T>` trait.
//! Example:
//!
//! ```
//! use lemon_tree::{lem_fn, LemonTree, LemonTreeNode};
//!
//! #[derive(LemonTreeNode, Debug, PartialEq)]
//! pub enum Expr
//! {	#[lem("VALUE(0)")]
//! 	Value(f64),
//!
//! 	#[lem("Expr(0) PLUS Expr(1)")]
//! 	Plus(String, String),
//! }
//!
//! impl Into<String> for Expr
//! {	fn into(self) -> String
//! 	{	match self
//! 		{	Expr::Value(v) => format!("{}", v),
//! 			Expr::Plus(a, b) => format!("{} + {}", a, b),
//! 		}
//! 	}
//! }
//!
//! #[derive(LemonTree, Debug, PartialEq)]
//! #[lem("Expr(expr)")]
//! #[lem_opt(token_type="f64", left="PLUS")]
//! pub struct Unit
//! {	expr: Expr,
//! }
//!
//! fn main()
//! {	let mut parser = Unit::get_parser(());
//! 	parser.add_token(<Unit as LemonTree>::Token::VALUE, 10.0).unwrap();
//! 	parser.add_token(<Unit as LemonTree>::Token::PLUS, 0.0).unwrap();
//! 	parser.add_token(<Unit as LemonTree>::Token::VALUE, 20.0).unwrap();
//! 	let result = parser.end().unwrap();
//! 	assert_eq!
//! 	(	result,
//! 		Unit
//! 		{	expr: Expr::Plus("10".to_string(), "20".to_string())
//! 		}
//! 	);
//! 	println!("Result: {:?}", result);
//! }
//! ```
//!

extern crate lemon_tree_derive;

pub use lemon_tree_derive::{lem_fn, LemonTree, LemonTreeNode};

/// Parser "start symbol" can be represented as a struct or enum. You need to annotate it with `#[derive(LemonTree)]`, and implementation of this trait will be generated.
///
/// The implementation contains 2 associated types:
/// * Parser - the parser, that will accept tokens, and finally return the start symbol.
/// * Token - enum with token names. All the terminal symbols (tokens) that appear in your grammar (in #[lem()] and #[lem_fn()] attributes) will become variants in this enum.
///
/// If you annotate a struct like this:
///
/// ```ignore
/// #[derive(LemonTree)]
/// struct Unit
/// {
/// }
/// ```
///
/// And you have terminal symbols `HELLO` and `WORLD`, then you can:
///
/// ```ignore
/// let mut parser = Unit::get_parser(()); // where () is initializer for %extra_argument
/// // the type of parser is <Unit as LemonTree>::Parser
/// parser.add_token(<Unit as LemonTree>::Token::HELLO, ()).unwrap();
/// parser.add_token(<Unit as LemonTree>::Token::WORLD, ()).unwrap();
/// let resulting_unit = parser.end().unwrap(); // returns Unit
/// ```
pub trait LemonTree
{   type Parser;
	type Token;
}

/// For nonterminal symbols, except start symbol.
pub trait LemonTreeNode
{
}
