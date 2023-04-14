//! Dictionary data structures implemented with an AVL tree (nearly balanced binary search tree).

#![no_std]
extern crate alloc;

#[doc(inline)]
pub use map::AvlTreeMap;

#[doc(inline)]
pub use set::AvlTreeSet;

pub mod map;
pub mod set;

#[cfg(test)]
mod tests;
