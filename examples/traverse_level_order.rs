use avl::AvlTreeMap;

fn main() {
    let mut map = AvlTreeMap::new();
    map.insert(1, "1");
    map.insert(2, "2");
    map.insert(3, "3");
    map.insert(4, "4");
    map.insert(5, "5");
    map.insert(6, "6");

    println!("Level-order traversal:");
    map.traverse_level_order(|k, v| {
        println!("Key: {}, Value: {}", k, v);
    });
}
