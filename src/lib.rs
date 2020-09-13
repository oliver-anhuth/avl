//! Dictionary data structures implemented with an AVL tree (nearly balanced binary search tree).

#[doc(inline)]
pub use map::AvlTreeMap;

#[doc(inline)]
pub use set::AvlTreeSet;

#[doc(inline)]
pub mod map;

#[doc(inline)]
pub use map::set;

#[cfg(test)]
mod tests;
