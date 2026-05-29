# lemon-tree

[![crates.io](https://img.shields.io/crates/v/lemon-tree.svg)](https://crates.io/crates/lemon-tree)
[![docs.rs](https://docs.rs/lemon-tree/badge.svg)](https://docs.rs/lemon-tree)

The famous [Lemon Parser Generator](https://www.hwaci.com/sw/lemon/), exposed as a Rust library that builds an LALR(1) parser transparently during `cargo build`.

Instead of maintaining a separate grammar file and running a code-generation step, you describe the grammar directly in Rust by adding annotation attributes to ordinary functions, structs and enums. The generated parser produces your own Rust types, so a successful parse yields a ready-to-use syntax tree (or a computed value) with no glue code in between.

This crate uses [lemon-mint](https://crates.io/crates/lemon-mint) as the parser-generation backend.

## How it works

The annotation attributes are procedural macros. As `cargo` compiles your source file, the macros accumulate the grammar rules they encounter, and the final `#[derive(LemonTree)]` (the start symbol) triggers generation of a complete LALR(1) parser. The parser is emitted into a private submodule and connected to your types through the `LemonTree` and `LemonTreeNode` traits. Because everything happens at compile time, grammar conflicts and errors surface as ordinary build errors.

## Installation

Add this to your project's `Cargo.toml`:

```toml
[dependencies]
lemon-tree = "0.2"
```

The crate exports three items: `lem_fn`, `LemonTree` and `LemonTreeNode`.

## Core concepts

A grammar is made of **terminal** symbols (tokens, supplied by your tokenizer) and **nonterminal** symbols (produced by reducing rules). By convention, names containing a lowercase letter are nonterminals, and all-uppercase names like `PLUS` or `NUM` are tokens.

Each nonterminal is backed by a Rust type. There are three ways to attach rules to it:

* **A struct** annotated with `#[derive(LemonTreeNode)]` (or `#[derive(LemonTree)]` for the start symbol). Rules are listed in `#[lem("...")]` attributes on the struct, and each rule constructs the struct.
* **An enum** annotated with `#[derive(LemonTreeNode)]` (or `#[derive(LemonTree)]`). The `#[lem("...")]` attributes are placed on the individual variants.
* **A function** annotated with `#[lem_fn("...")]`. The function's return type is the left-hand-side nonterminal, and the body is the action run when the rule reduces.

The **start symbol** is marked with `#[derive(LemonTree)]` and represents the final result of a parse. Parser-wide options are set with `#[lem_opt(...)]` attributes placed next to it.

### Rule syntax

The string inside `#[lem(...)]` / `#[lem_fn(...)]` is the right-hand side of a Lemon rule — a whitespace-separated sequence of symbols. The left-hand side is implied by where the attribute sits (the struct/enum/variant type, or the function return type).

* **Multiple rules.** An attribute may list several alternatives, and may be repeated: `#[lem("A(value)", "B(value)")]`.
* **Aliases.** A symbol can be followed by an alias in parentheses to bind its value. For structs, the alias is a field name (`"VALUE(value)"`). For enum variants and `#[lem_fn]` functions it is an argument/tuple-field name or a zero-based index (`"Expr(0) PLUS Expr(1)"`).
* **Value conversion.** Bound values are moved into their target via `.into()`, so the target type only needs `From` for the matched type (this is why an `Expr` flows into a `Box<Expr>` field automatically). Implement `Into<TargetType>` yourself for arbitrary conversions. Unbound fields/arguments are filled with `Default::default()`.
* **Optional symbols.** Square brackets mark an optional part and expand into alternatives: `"Exprs(exprs) [SEMICOLON]"` means both `"Exprs(exprs)"` and `"Exprs(exprs) SEMICOLON"`. Brackets may be nested.

### Parser options — `#[lem_opt(...)]`

| Option           | Lemon directive    | Meaning                                                                |
|------------------|--------------------|------------------------------------------------------------------------|
| `token_type`     | `%token_type`      | The Rust type carried by every token's value.                          |
| `extra_argument` | `%extra_argument`  | Type of a user value made available to every action.                   |
| `left`           | `%left`            | Declare tokens left-associative (precedence grows with each line).     |
| `right`          | `%right`           | Declare tokens right-associative.                                      |
| `nonassoc`       | `%nonassoc`        | Declare tokens non-associative.                                        |
| `fallback`       | `%fallback`        | `"FALLBACK_TOK TOK_A TOK_B ..."` — fall back the listed tokens.        |
| `trace`          | `%trace`           | Print a parser trace to stderr, prefixed with the given prompt.        |

Associativity options may be repeated; rules declared earlier have lower precedence than later ones, exactly as in Lemon.

### The generated parser API

Deriving `LemonTree` on a start symbol `S` generates:

* `S::get_parser(extra)` — create a parser (`extra` is the `%extra_argument`; use `()` when none is set).
* `<S as LemonTree>::Token` — an enum with one variant per terminal symbol.
* `parser.add_token(token, value)` — feed one token; returns `Result<(), ()>` (`Err` on syntax error).
* `parser.try_add_token(token, value)` — like `add_token`, but returns `Ok(false)` when the token is not accepted instead of erroring.
* `parser.end()` — signal end of input; returns `Result<S, ()>`.
* `parser.extra` — public field holding the `extra_argument` value.

## Example: a calculator

This first example builds a calculator that evaluates expressions to `f64`. Create an empty cargo project and put the following in its `main.rs`:

```rust
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

// This is the tokenizer function. It takes "input", and feeds tokens to the "parser".
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

## Example: building a syntax tree

Rather than computing a value, you can produce an AST. Here `Expr` is an enum whose variants carry the rules, so the parser hands back a tree of `Expr` nodes. Note how each matched `Expr` is automatically wrapped in a `Box<Expr>` via `.into()`.

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

pub type Exprs = Vec<Expr>;
#[lem_fn("Expr(item)")] fn exprs_item(item: Expr) -> Exprs {vec![item]}
#[lem_fn("Exprs(items) SEMICOLON Expr(item)")] fn exprs_items(mut items: Exprs, item: Expr) -> Exprs {items.push(item); items}

#[derive(LemonTree, Debug)]
#[lem_opt(token_type="f64", left="PLUS MINUS", left="DIVIDE TIMES")]
#[lem("Exprs(exprs) [SEMICOLON]")]
pub struct Program
{	exprs: Vec<Expr>,
	flag: bool, // not bound by any rule, so filled with Default::default()
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
```

## Constraints

* All attributes describing a single parser must live in **one** Rust file.
* `#[derive(LemonTree)]` (the start symbol) must be the **last** parser attribute in that file — it is what triggers parser generation.
* A file may define only one parser (one `#[derive(LemonTree)]`).
* Unions are not supported as symbol types.

You can have several parsers in one crate, as long as each lives in its own file.

## Cargo features

These features (forwarded to `lemon-tree-derive`) help debug the grammar at build time:

* `dump-grammar` — print the generated grammar, with Rust actions, to stderr during the build.
* `dump-lemon-grammar` — print the grammar in classic Lemon `.y` syntax to stderr.
* `debug-parser-to-file` — write the generated parser source to a file next to your source instead of inlining it, making the generated code easy to inspect.

## License

Licensed under the [MIT license](LICENSE).
