use avl::{AvlTreeMap, AvlTreeSet};

fn main() {
    let mut map = AvlTreeMap::new();
    map.insert(0, "zero");
    map.insert(1, "one");
    map.insert(2, "two");
    assert_eq!(map.get(&1), Some(&"one"));
    map.remove(&1);
    assert!(map.get(&1).is_none());

    let mut set = AvlTreeSet::new();
    set.insert(format!("{}", 0));
    set.insert(format!("{}", 1));
    set.insert(format!("{}", 2));
    assert!(set.contains("1"));
    set.remove("1");
    assert!(!set.contains("1"));
}
