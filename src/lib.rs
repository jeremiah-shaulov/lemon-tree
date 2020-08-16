extern crate lemon_mint;
extern crate lemon_tree_derive;

pub use lemon_mint::{LemonMint, LemonMintBuilder, LemonMintError};
pub use lemon_tree_derive::{lem_fn, LemonTree, LemonTreeNode};

pub trait LemonTree
{   type Token;
	type Parser;
}

pub trait LemonTreeNode
{
}
