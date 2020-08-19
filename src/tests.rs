use super::{AvlTreeMap, AvlTreeSet};

const N: i32 = 1_000;
const LARGE_N: i32 = 10_000_000;

#[test]
fn test_new() {
    let map_i32 = AvlTreeMap::<i32, ()>::new();
    assert!(map_i32.is_empty());
    map_i32.check_consistency();

    let map_i8 = AvlTreeMap::<i8, ()>::new();
    assert!(map_i8.is_empty());
    map_i8.check_consistency();

    let map_string = AvlTreeMap::<String, String>::new();
    assert!(map_string.is_empty());
    map_string.check_consistency();
}

#[test]
fn test_rebalance() {
    {
        //     3 ->   2
        //    /      / \
        //   2      1   3
        //  /
        // 1
        let mut map = AvlTreeMap::new();
        map.insert(3, ());
        map.insert(2, ());
        map.insert(1, ());
        map.check_consistency();
        assert_eq!(map.height(), 1);
    }
    {
        //     3   ->     3 ->   2
        //    / \        /      / \
        //   2   4      2      1   3
        //  /          /
        // 1          1
        let mut map = AvlTreeMap::new();
        map.insert(3, ());
        map.insert(2, ());
        map.insert(4, ());
        map.insert(1, ());
        map.check_consistency();
        assert_eq!(map.height(), 2);
        map.remove(&4);
        map.check_consistency();
        assert_eq!(map.height(), 1);
    }
    {
        //   3  ->   2
        //  /       / \
        // 1       1   3
        //  \
        //   2
        let mut map = AvlTreeMap::new();
        map.insert(3, ());
        map.insert(1, ());
        map.insert(2, ());
        map.check_consistency();
        assert_eq!(map.height(), 1);
    }
    {
        //   3   ->   3  ->   2
        //  / \      /       / \
        // 1   4    1       1   3
        //  \        \
        //   2        2
        let mut map = AvlTreeMap::new();
        map.insert(3, ());
        map.insert(1, ());
        map.insert(4, ());
        map.insert(2, ());
        map.check_consistency();
        assert_eq!(map.height(), 2);
        map.remove(&4);
        map.check_consistency();
        assert_eq!(map.height(), 1);
    }
    {
        // 1 ->    2
        //  \     / \
        //   2   1   3
        //    \
        //     3
        let mut map = AvlTreeMap::new();
        map.insert(1, ());
        map.insert(2, ());
        map.insert(3, ());
        map.check_consistency();
        assert_eq!(map.height(), 1);
    }
    {
        //   1     -> 1     ->    2
        //  / \        \         / \
        // 0   2        2       1   3
        //      \        \
        //       3        3
        let mut map = AvlTreeMap::new();
        map.insert(1, ());
        map.insert(0, ());
        map.insert(2, ());
        map.insert(3, ());
        map.check_consistency();
        assert_eq!(map.height(), 2);
        map.remove(&0);
        map.check_consistency();
        assert_eq!(map.height(), 1);
    }
    {
        // 1   ->  2
        //  \     / \
        //   3   1   3
        //  /
        // 2
        let mut map = AvlTreeMap::new();
        map.insert(1, ());
        map.insert(3, ());
        map.insert(2, ());
        map.check_consistency();
        assert_eq!(map.height(), 1);
    }
    {
        //   1   ->  1   ->  2
        //  / \       \     / \
        // 0   3       3   1   3
        //    /       /
        //   2       2
        let mut map = AvlTreeMap::new();
        map.insert(1, ());
        map.insert(0, ());
        map.insert(3, ());
        map.insert(2, ());
        map.check_consistency();
        assert_eq!(map.height(), 2);
        map.remove(&0);
        map.check_consistency();
        assert_eq!(map.height(), 1);
    }
}

#[test]
fn test_insert() {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0);
    let mut values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();
    values.sort();
    values.dedup();

    let mut map = AvlTreeMap::new();
    for value in &values {
        assert!(map.insert(*value, *value));
        map.check_consistency();
    }
    assert!(map.len() == values.len());

    for value in &values {
        assert!(!map.insert(*value, *value));
    }
    assert!(map.len() == values.len());
}

#[test]
fn test_insert_sorted_range() {
    let mut map = AvlTreeMap::new();
    for value in 0..N {
        assert!(map.insert(value, value));
        map.check_consistency();
    }
    assert!(map.len() == N as usize);
    assert!(map.height() > 0);
    assert!(map.height() < N as usize / 2);
    assert!(map.get(&-42).is_none());
}

#[test]
fn test_insert_shuffled_range() {
    use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

    let mut values: Vec<i32> = (0..N).collect();
    let mut rng = StdRng::seed_from_u64(0);
    values.shuffle(&mut rng);

    let mut map = AvlTreeMap::new();
    for value in &values {
        assert!(map.insert(*value, "foo"));
        map.check_consistency();
    }
    assert!(map.len() == values.len());

    for value in &values {
        assert!(!map.insert(*value, "bar"));
    }
    assert!(map.len() == values.len());
    assert!(map.get(&-42).is_none());
}

#[test]
fn test_get() {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0);
    let values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();

    let mut map = AvlTreeMap::new();
    assert!(map.get(&42).is_none());
    for value in &values {
        map.insert(*value, value.wrapping_add(1));
    }

    for value in &values {
        let got = map.get(value);
        assert_eq!(got, Some(&(*value + 1)));
        let got = map.get_key_value(value);
        assert_eq!(got, Some((value, &(value.wrapping_add(1)))));
    }
}

#[test]
fn test_clear() {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0);
    let mut values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();
    values.sort();
    values.dedup();

    let mut map = AvlTreeMap::new();
    for value in &values {
        map.insert(*value, String::from("foo"));
    }
    assert!(!map.is_empty());
    assert!(map.len() == values.len());

    map.clear();
    assert!(map.is_empty());
    assert!(map.len() == 0);

    for value in &values {
        assert!(map.insert(*value, String::from("bar")));
    }
    assert!(!map.is_empty());
    assert!(map.len() == values.len());
    map.check_consistency();
}

#[test]
fn test_remove() {
    use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0);
    let mut values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();
    values.sort();
    values.dedup();

    let mut map = AvlTreeMap::new();
    for value in &values {
        map.insert(*value, 42);
    }

    values.shuffle(&mut rng);
    for value in &values {
        assert!(map.get(value).is_some());
        assert_eq!(map.remove(value), Some(42));
        assert!(map.get(value).is_none());
        map.check_consistency();
    }
    assert!(map.is_empty());
    assert!(map.len() == 0);
}

#[test]
fn test_set() {
    use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0);
    let mut values: Vec<i32> = (0..N).map(|_| rng.gen_range(0, N)).collect();

    let mut set = AvlTreeSet::new();
    for value in &values {
        set.insert(*value);
    }
    set.check_consistency();

    for value in &values {
        let got = set.get(value);
        assert_eq!(got, Some(value));
    }

    values.shuffle(&mut rng);
    values.resize(values.len() / 2, 0);
    for value in &values {
        set.remove(value);
    }
    set.check_consistency();
}

#[test]
fn test_map_iter() {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0);
    let mut values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();

    let mut map = AvlTreeMap::new();
    for value in &values {
        map.insert(*value, value.wrapping_add(42));
    }

    values.sort();
    values.dedup();

    let mut map_iter = map.iter();
    for value in &values {
        let kv = map_iter.next();
        assert!(kv.is_some());
        let (&key, &mapped) = kv.unwrap();
        assert_eq!(key, *value);
        assert_eq!(mapped, value.wrapping_add(42));
    }
    assert!(map_iter.next().is_none());

    let mut value_iter = values.iter();
    for (&key, &mapped) in &map {
        let value = value_iter.next();
        assert!(value.is_some());
        let value = value.unwrap();
        assert_eq!(key, *value);
        assert_eq!(mapped, value.wrapping_add(42));
    }
    assert!(value_iter.next().is_none());

    let mut map_iter_mut = map.iter_mut();
    for value in &values {
        let kv = map_iter_mut.next();
        assert!(kv.is_some());
        let (&key, mapped_mut) = kv.unwrap();
        assert_eq!(key, *value);
        assert_eq!(*mapped_mut, value.wrapping_add(42));
        *mapped_mut = value.wrapping_sub(42);
    }
    assert!(map_iter_mut.next().is_none());

    let mut value_iter = values.iter();
    for (&key, mapped_mut) in &mut map {
        let value = value_iter.next();
        assert!(value.is_some());
        let value = value.unwrap();
        assert_eq!(key, *value);
        assert_eq!(*mapped_mut, value.wrapping_sub(42));
        *mapped_mut = 42;
    }
    assert!(value_iter.next().is_none());
}

#[test]
fn test_set_iter() {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0);
    let mut values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();

    let mut set = AvlTreeSet::new();
    for value in &values {
        set.insert(*value);
    }

    values.sort();
    values.dedup();

    let mut set_iter = set.iter();
    for value in &values {
        let value_in_set = set_iter.next();
        assert!(value_in_set.is_some());
        let &value_in_set = value_in_set.unwrap();
        assert_eq!(value_in_set, *value);
    }
    assert!(set_iter.next().is_none());

    let mut value_iter = values.iter();
    for &value_in_set in &set {
        let value = value_iter.next();
        assert!(value.is_some());
        let value = value.unwrap();
        assert_eq!(value_in_set, *value);
    }
    assert!(value_iter.next().is_none());
}

#[test]
#[ignore]
fn test_large() {
    use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0);
    let mut values: Vec<i32> = (0..LARGE_N).map(|_| rng.gen_range(0, LARGE_N)).collect();

    let mut map = AvlTreeMap::new();
    for value in &values {
        map.insert(*value, *value);
    }
    map.check_consistency();

    values.shuffle(&mut rng);
    values.resize(values.len() / 2, 0);
    for value in &values {
        map.remove(value);
    }
    map.check_consistency();
}
