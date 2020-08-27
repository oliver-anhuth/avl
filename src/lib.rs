//! Dictionary data structures implemented with an AVL tree (nearly balanced binary search tree).

#[doc(inline)]
pub use map::AvlTreeMap;

#[doc(inline)]
pub use set::AvlTreeSet;

#[doc(inline)]
pub mod map;
pub mod set;

#[cfg(test)]
mod tests;
