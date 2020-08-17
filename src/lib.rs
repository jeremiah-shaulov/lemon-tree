//! Famous [Lemon Parser Generator](https://www.hwaci.com/sw/lemon/), designed as library that builds your parser transparently during cargo build.
//! To describe parser rules you add annotation attributes to rust functions, structs and enums.
//!
//! You can find usage examples [here](https://github.com/jeremiah-shaulov/lemon-tree)

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
