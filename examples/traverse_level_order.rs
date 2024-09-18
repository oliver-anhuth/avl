use avl::{AvlTreeMap, AvlTreeSet};

fn main() {
    let mut map = AvlTreeMap::new();
    map.insert(1, "1");
    map.insert(2, "2");
    map.insert(3, "3");
    map.insert(4, "4");
    map.insert(5, "5");
    map.insert(6, "6");

    println!("Level-order map traversal:");
    map.traverse_level_order(|lv, k, v| {
        println!("Level: {}, Key: {}, Value: {}", lv, k, v);
    });

    println!();

    let mut set = AvlTreeSet::new();
    set.insert(1);
    set.insert(2);
    set.insert(3);
    set.insert(4);
    set.insert(5);
    set.insert(6);

    println!("Level-order set traversal:");
    set.traverse_level_order(|lv, v| {
        println!("Level: {}, Value: {}", lv, v);
    });
}
