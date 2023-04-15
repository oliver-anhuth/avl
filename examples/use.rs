use avl::{AvlTreeMap, AvlTreeSet};

fn main() {
    let mut map = AvlTreeMap::new();
    map.insert(0, "zero");
    map.insert(1, "one");
    map.insert(2, "two");
    map.insert(2, "two");
    map.insert(3, "three");
    map.insert(4, "four");
    map.insert(5, "five");
    assert_eq!(map.get(&1), Some(&"one"));
    map.remove(&1);
    assert!(map.get(&1).is_none());

    for (k, v) in &map {
        println!("{k} => {v}");
    }

    let mut set = AvlTreeSet::new();
    for x in 0..5 {
        set.insert(x);
    }
    assert!(set.contains(&1));
    set.remove(&1);
    assert!(!set.contains(&1));

    print!("{{ ");
    for x in &set {
        print!("{x}, ");
    }
    println!("}}");
}
