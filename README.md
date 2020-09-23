# AVL Tree Map and Set in Rust

[![Build and test](https://github.com/oliver-anhuth/avl/workflows/BuildAndTest/badge.svg)](https://github.com/oliver-anhuth/avl/actions)

An ordered map and set implemented with an AVL tree (nearly balanced binary search tree) in Rust.

```rust
use avl::AvlTreeMap;

let mut map = AvlTreeMap::new();
map.insert(0, "zero");
map.insert(1, "one");
map.insert(2, "two");
assert_eq!(map.get(&1), Some(&"one"));
map.remove(&1);
assert!(map.get(&1).is_none());


use avl::AvlTreeSet;

let mut set = AvlTreeSet::new();
set.insert(0);
set.insert(1);
set.insert(2);
assert_eq!(set.get(&1), Some(&1));
set.remove(&1);
assert!(set.get(&1).is_none());
```

This is solely to get practice with the dark art of unsafe Rust. For all common purposes one of the standard library collections should be preferable.
