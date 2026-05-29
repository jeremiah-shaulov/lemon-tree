//! The famous [Lemon Parser Generator](https://www.hwaci.com/sw/lemon/), exposed as a Rust library
//! that builds an LALR(1) parser transparently during `cargo build`. Instead of writing a separate
//! grammar file and running a code generator, you describe the grammar directly in Rust by adding
//! annotation attributes to ordinary functions, structs and enums. The resulting parser produces
//! your own Rust types, so parsing yields a ready-to-use syntax tree (or computed value) with no
//! glue code in between.
//!
//! This crate uses [lemon-mint](https://crates.io/crates/lemon-mint) as the parser-generation backend.
//! You can find more usage examples [here](https://github.com/jeremiah-shaulov/lemon-tree).
//!
//! # How it works
//!
//! The annotation attributes are procedural macros. As `cargo` compiles your source file, the macros
//! accumulate the grammar rules they see, and the final `#[derive(LemonTree)]` (the start symbol)
//! triggers generation of a complete LALR(1) parser. The generated parser is emitted into a private
//! submodule and wired to your types through the [`LemonTree`] and [`LemonTreeNode`] traits. Because
//! everything happens at compile time, grammar conflicts and errors are reported as ordinary build
//! errors.
//!
//! # Mapping to Lemon
//!
//! If you are familiar with Lemon's `.y` grammar files, this is the correspondence. Let's say we want
//! to create a Lemon parser like this:
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
//! If a struct has more fields than appear in expression, the remaining fields will be set to `Default::default()`, so they need to implement `std::default::Default` trait.
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
//! # Defining grammar symbols
//!
//! A grammar is made of *terminal* symbols (tokens, fed in by your tokenizer) and *nonterminal*
//! symbols (produced by reducing rules). By convention, names that contain a lowercase letter are
//! treated as nonterminals, and all-uppercase names like `PLUS` or `VALUE` are treated as tokens.
//!
//! Each nonterminal is backed by a Rust type, and there are three ways to attach rules to it:
//!
//! * **A struct** annotated with `#[derive(LemonTreeNode)]` (or `#[derive(LemonTree)]` for the start
//!   symbol). Rules are listed in `#[lem("...")]` attributes on the struct. Each rule constructs the
//!   struct; aliases in the rule name the fields to fill.
//! * **An enum** annotated with `#[derive(LemonTreeNode)]` (or `#[derive(LemonTree)]`). Here the
//!   `#[lem("...")]` attributes are placed on the individual variants, and each rule constructs that
//!   variant.
//! * **A function** annotated with `#[lem_fn("...")]`. The function's return type is the left-hand
//!   side nonterminal, and the function body is the action executed when the rule reduces. This is the
//!   most flexible form, useful when a reduction needs to compute something rather than just build a value.
//!
//! All rules for one parser must live in a single Rust file, and the `#[derive(LemonTree)]` start
//! symbol must come last in that file (see [Constraints](#constraints)).
//!
//! # Rule syntax
//!
//! The string inside `#[lem(...)]` / `#[lem_fn(...)]` is the right-hand side of a Lemon rule — a
//! sequence of terminal and nonterminal symbols separated by whitespace. The left-hand side is implied
//! by where the attribute is placed (the struct/enum/variant type, or the function return type).
//!
//! A single attribute may contain several alternative rules, and the attribute may be repeated:
//!
//! ```ignore
//! #[lem("A(value)", "B(value)")]   // two rules, both producing this symbol
//! #[lem("C(value)")]               // another rule
//! ```
//!
//! ## Aliases
//!
//! A symbol in the rule can be followed by an alias in parentheses, which binds the symbol's value so
//! the action can use it:
//!
//! * For **structs**, the alias is a field name: `"VALUE(value)"` assigns the matched value to the
//!   `value` field.
//! * For **enum variants** and **`#[lem_fn]` functions**, the alias may be a function argument /
//!   tuple-field name, or a zero-based positional index: `"Expr(0) PLUS Expr(1)"` binds the first and
//!   second `Expr` to tuple fields `0` and `1`.
//!
//! ## Value conversion
//!
//! Bound values are moved into the target field via `.into()`, so the field type only needs to
//! implement `From` for the matched value's type (for example `Box<T>` implements `From<T>`, which is
//! why `Expr` can flow into a `Box<Expr>` field automatically). For arbitrary conversions you can
//! implement `Into<TargetType>` yourself. Fields and arguments that are *not* bound by an alias are
//! filled with [`Default::default()`], so those types must implement [`Default`].
//!
//! ## Optional symbols `[...]`
//!
//! Square brackets mark an optional part of a rule and expand into multiple alternatives. For example
//! `"Exprs(exprs) [SEMICOLON]"` is shorthand for the two rules `"Exprs(exprs)"` and
//! `"Exprs(exprs) SEMICOLON"`. Brackets may be nested.
//!
//! # Parser options — `#[lem_opt(...)]`
//!
//! Lemon directives are set with `#[lem_opt(...)]` attributes placed next to the start symbol
//! (`#[derive(LemonTree)]`). Each option takes a string value:
//!
//! | Option           | Lemon directive    | Meaning                                                                 |
//! |------------------|--------------------|-------------------------------------------------------------------------|
//! | `token_type`     | `%token_type`      | The Rust type carried by every token's value (the `minor` value).       |
//! | `extra_argument` | `%extra_argument`  | Type of a user value made available to all actions (see below).         |
//! | `left`           | `%left`            | Declare tokens left-associative (precedence increases with each line).  |
//! | `right`          | `%right`           | Declare tokens right-associative.                                       |
//! | `nonassoc`       | `%nonassoc`        | Declare tokens non-associative.                                         |
//! | `fallback`       | `%fallback`        | `"FALLBACK_TOK TOK_A TOK_B ..."` — fall back the listed tokens.         |
//! | `trace`          | `%trace`           | Print a parser trace to stderr, prefixed with the given prompt string.  |
//!
//! Associativity / precedence options may be repeated; rules declared earlier have lower precedence
//! than those declared later, exactly as in Lemon. Example:
//!
//! ```ignore
//! #[lem_opt(token_type="f64", left="PLUS MINUS", left="DIVIDE TIMES", trace=">>")]
//! ```
//!
//! # The generated parser API
//!
//! Deriving [`LemonTree`] on the start symbol `S` generates:
//!
//! * `S::get_parser(extra)` — create a parser. `extra` is the `%extra_argument` value (use `()` when
//!   no `extra_argument` is set). Its type is `<S as LemonTree>::Parser`.
//! * `<S as LemonTree>::Token` — an enum with one variant per terminal symbol used anywhere in the
//!   grammar.
//! * `parser.add_token(token, value)` — feed one token. `value` has the `token_type` type. Returns
//!   `Result<(), ()>`; an `Err` means a syntax error at that token.
//! * `parser.try_add_token(token, value)` — like `add_token`, but returns `Result<bool, ()>` where
//!   `Ok(false)` indicates the token was not accepted in the current state instead of erroring.
//! * `parser.end()` — signal end of input. Returns `Result<S, ()>`: the constructed start symbol on
//!   success, or `Err(())` on a syntax error.
//! * `parser.extra` — public field holding the `extra_argument` value, readable and writable between
//!   tokens.
//!
//! You drive the parser from your own tokenizer: repeatedly call `add_token`, then call `end` to get
//! the result. See the [`README`](https://github.com/jeremiah-shaulov/lemon-tree) for a complete
//! calculator with a hand-written tokenizer.
//!
//! # The `extra_argument`
//!
//! Setting `#[lem_opt(extra_argument="MyType")]` gives every action access to a shared value. In
//! `#[lem_fn]` functions, add a final argument literally named `extra` to receive it:
//!
//! ```ignore
//! #[lem_fn("Expr(a) PLUS Expr(b)")]
//! pub fn expr_plus(a: Expr, b: Expr, extra: &mut Context) -> Expr
//! {	extra.count += 1;
//! 	Expr {value: a.value + b.value}
//! }
//! ```
//!
//! The value is supplied when constructing the parser via `get_parser(extra)` and is also accessible
//! as `parser.extra`.
//!
//! # Constraints
//!
//! * All attributes describing a single parser must live in **one** Rust file.
//! * `#[derive(LemonTree)]` (the start symbol) must be the **last** parser attribute in that file; it
//!   is what triggers parser generation.
//! * A file may define only one parser (one `#[derive(LemonTree)]`).
//! * Unions are not supported as symbol types.
//!
//! Different parsers can coexist in the same crate as long as each lives in its own file.
//!
//! # Cargo features
//!
//! These features (forwarded to `lemon-tree-derive`) help with debugging the grammar at build time:
//!
//! * `dump-grammar` — print the generated grammar, with Rust actions, to stderr during the build.
//! * `dump-lemon-grammar` — print the grammar in classic Lemon `.y` syntax to stderr.
//! * `debug-parser-to-file` — write the generated parser source to a file next to your source instead
//!   of inlining it, which makes the generated code easy to inspect.

pub use lemon_tree_derive::{lem_fn, LemonTree, LemonTreeNode};

/// Implemented for the parser's *start symbol* — the type that a successful parse produces.
///
/// You don't implement this trait by hand: annotate a struct or enum with `#[derive(LemonTree)]`
/// and the implementation, together with a `get_parser` constructor, is generated for you. This
/// derive must be the last parser attribute in the file, because it is what triggers parser
/// generation (see the [crate-level documentation](crate) for the full picture).
///
/// The implementation provides two associated types:
/// * [`Parser`](LemonTree::Parser) — the parser, which accepts tokens and finally returns the start symbol.
/// * [`Token`](LemonTree::Token) — an enum whose variants are all the terminal symbols (tokens) that
///   appear anywhere in your grammar (in `#[lem()]` and `#[lem_fn()]` attributes).
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
{	/// The generated parser. Create one with `get_parser(extra)`, feed it tokens with
	/// `add_token` / `try_add_token`, and call `end` to obtain the start symbol.
	type Parser;

	/// Enum of all terminal symbols (tokens) used in the grammar; pass its variants to `add_token`.
	type Token;
}

/// Marker trait implemented by every nonterminal symbol other than the start symbol.
///
/// Derive it with `#[derive(LemonTreeNode)]` on a struct or enum to make that type a nonterminal
/// of the current parser. Attach the rules that produce it with `#[lem("...")]` attributes — on the
/// struct itself, or on each enum variant. The start symbol uses [`LemonTree`] instead.
pub trait LemonTreeNode
{
}
