use std::ops::Bound;

use super::{AvlTreeMap, AvlTreeSet};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

const N: i32 = 1_000;
const LARGE_N: i32 = 10_000_000;

#[test]
fn test_new() {
    let map_i32: AvlTreeMap<i32, ()> = AvlTreeMap::new();
    assert!(map_i32.is_empty());
    map_i32.check_consistency();
    assert_eq!(format!("{:?}", map_i32), String::from("{}"));

    let mut map_i8 = AvlTreeMap::<i8, &str>::new();
    assert!(map_i8.is_empty());
    map_i8.insert(0, "foo");
    map_i8.insert(1, "bar");
    map_i8.insert(2, "baz");
    map_i8.check_consistency();
    assert_eq!(
        format!("{:?}", map_i8),
        String::from(r#"{0: "foo", 1: "bar", 2: "baz"}"#)
    );
    assert_eq!(map_i8[&1], "bar");

    let map_string: AvlTreeMap<String, String> = AvlTreeMap::default();
    assert!(map_string.is_empty());
    map_string.check_consistency();

    let set_i32: AvlTreeSet<i32> = AvlTreeSet::new();
    assert!(set_i32.is_empty());
    assert_eq!(format!("{:?}", set_i32), String::from("{}"));

    let mut set_i8 = AvlTreeSet::<i8>::new();
    assert!(set_i8.is_empty());
    set_i8.insert(1);
    set_i8.insert(2);
    set_i8.insert(0);
    set_i8.check_consistency();
    assert_eq!(format!("{:?}", set_i8), String::from("{0, 1, 2}"));
}

#[test]
fn test_rebalance() {
    {
        //     3          2
        //    /          / \
        //   2     =>   1   3
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
        //     3              3          2
        //    / \            /          / \
        //   2   4   =>     2     =>   1   3
        //  /              /
        // 1              1
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
        //   3          2
        //  /          / \
        // 1     =>   1   3
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
        //   3            3          2
        //  / \          /          / \
        // 1   4   =>   1     =>   1   3
        //  \            \
        //   2            2
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
        // 1              2
        //  \            / \
        //   2     =>   1   3
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
        //   1            1              2
        //  / \            \            / \
        // 0   2     =>     2     =>   1   3
        //      \            \
        //       3            3
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
        // 1            2
        //  \          / \
        //   3   =>   1   3
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
        //   1          1            2
        //  / \          \          / \
        // 0   3   =>     3   =>   1   3
        //    /          /
        //   2          2
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
    let mut rng = StdRng::seed_from_u64(0);
    let mut values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();
    values.sort();
    values.dedup();

    let mut map = AvlTreeMap::new();
    for value in &values {
        assert!(map.insert(*value, *value).is_none());
        map.check_consistency();
    }
    assert!(map.len() == values.len());

    for value in &values {
        assert_eq!(map.insert(*value, *value), Some(*value));
        assert!(map.contains_key(value));
    }
    assert!(map.len() == values.len());
}

#[test]
fn test_insert_sorted_range() {
    let mut map = AvlTreeMap::new();
    for value in 0..N {
        assert!(map.insert(value, value).is_none());
        map.check_consistency();
    }
    assert!(map.len() == N as usize);
    assert!(map.get(&-42).is_none());
}

#[test]
fn test_insert_shuffled_range() {
    let mut values: Vec<i32> = (0..N).collect();
    let mut rng = StdRng::seed_from_u64(0);
    values.shuffle(&mut rng);

    let mut map = AvlTreeMap::new();
    for value in &values {
        assert!(map.insert(*value, "foo").is_none());
        map.check_consistency();
    }
    assert!(map.len() == values.len());

    for value in &values {
        assert_eq!(map.insert(*value, "bar"), Some("foo"));
    }
    assert!(map.len() == values.len());
    assert!(map.get(&-42).is_none());
}

#[test]
fn test_get() {
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

    for value in &values {
        let got = map.get_mut(value);
        assert_eq!(got, Some(&mut (value.wrapping_add(1))));
        *got.unwrap() = value.wrapping_add(2);
        let got = map.get_key_value(value);
        assert_eq!(got, Some((value, &(value.wrapping_add(2)))));
    }
}

#[test]
#[should_panic(expected = "no entry found for key")]
fn test_index_panic() {
    let mut map = AvlTreeMap::new();
    map.insert(1, "foo");
    map.insert(42, "bar");
    map.insert(512, "baz");
    map[&13];
}

#[test]
fn test_clear() {
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
        assert!(map.insert(*value, String::from("bar")).is_none());
    }
    assert!(!map.is_empty());
    assert!(map.len() == values.len());
    map.check_consistency();
}

#[test]
fn test_remove() {
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
fn test_append() {
    let mut rng = StdRng::seed_from_u64(0);
    let n = N & 1;
    let mut values: Vec<i32> = (0..n).map(|_| rng.gen()).collect();

    let mut map = AvlTreeMap::new();
    let mut map2 = AvlTreeMap::new();
    for chunk in values.chunks_exact(2) {
        map.insert(chunk[0], "foo");
        map2.insert(chunk[1], "bar");
    }

    values.sort();
    values.dedup();

    map.append(&mut map2);
    assert!(map2.is_empty());
    let mut map_keys = map.keys();
    for value in values {
        assert_eq!(map_keys.next(), Some(&value));
    }
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

    // Test non mutable iterators
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

    let mut key_iter = map.keys();
    for value in &values {
        let key = key_iter.next();
        assert!(key.is_some());
        let &key = key.unwrap();
        assert_eq!(key, *value);
    }
    assert!(map_iter.next().is_none());

    let mut mapped_iter = map.values();
    for value in &values {
        let mapped = mapped_iter.next();
        assert!(mapped.is_some());
        let &mapped = mapped.unwrap();
        assert_eq!(mapped, value.wrapping_add(42));
    }
    assert!(map_iter.next().is_none());

    // Test mutable iterators
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
        *mapped_mut = *value;
    }
    assert!(value_iter.next().is_none());

    // Test consuming iterator
    let mut value_iter = values.iter();
    for (key, mapped) in map.clone() {
        let value = value_iter.next();
        assert!(value.is_some());
        let value = value.unwrap();
        assert_eq!(key, *value);
        assert_eq!(mapped, *value);
    }
    assert!(value_iter.next().is_none());

    let mut into_iter = map.clone().into_iter();
    for _ in 0..N / 10 {
        into_iter.next();
    }

    // Test reverse iterator
    let mut values_iter = values.iter();
    let mut map_iter = map.iter();
    for _ in 1..=10 {
        values_iter.next();
        values_iter.next_back();
        map_iter.next();
        map_iter.next_back();
    }
    while let Some(value) = values_iter.next_back() {
        let kv = map_iter.next_back();
        assert_eq!(kv, Some((value, value)));
    }

    // Test owning reverse iterator
    let mut values_iter = values.iter();
    let mut map_iter = map.clone().into_iter();
    for _ in 1..=10 {
        values_iter.next();
        values_iter.next_back();
        map_iter.next();
        map_iter.next_back();
    }
    while let Some(value) = values_iter.next_back() {
        let kv = map_iter.next_back();
        assert_eq!(kv, Some((*value, *value)));
    }

    // Test debug formatting for non owning iterator
    let mut map: AvlTreeMap<i32, &str> = AvlTreeMap::new();
    map.extend(vec![(1, "one"), (2, "two"), (3, "three")].into_iter());
    assert_eq!(
        format!("{:?}", map.iter()),
        r#"[(1, "one"), (2, "two"), (3, "three")]"#
    );
    assert_eq!(format!("{:?}", map.keys()), "[1, 2, 3]");
    assert_eq!(format!("{:?}", map.values()), r#"["one", "two", "three"]"#);
    assert_eq!(
        format!("{:?}", map.iter_mut()),
        r#"[(1, "one"), (2, "two"), (3, "three")]"#
    );
    assert_eq!(
        format!("{:?}", map.values_mut()),
        r#"["one", "two", "three"]"#
    );

    // Test debug formatting for owning iterator
    let mut map_into_iter = map.clone().into_iter();
    assert_eq!(
        format!("{:?}", map_into_iter),
        r#"[(1, "one"), (2, "two"), (3, "three")]"#
    );
    assert_eq!(
        format!("{:?}", map_into_iter),
        r#"[(1, "one"), (2, "two"), (3, "three")]"#
    );
    map_into_iter.next();
    assert_eq!(
        format!("{:?}", map_into_iter),
        r#"[(2, "two"), (3, "three")]"#
    );

    map_into_iter.next_back();
    assert_eq!(format!("{:?}", map_into_iter), r#"[(2, "two")]"#);

    map_into_iter.next();
    assert_eq!(format!("{:?}", map_into_iter), "[]");

    map_into_iter.next();
    map_into_iter.next();
    map_into_iter.next_back();
    assert_eq!(format!("{:?}", map_into_iter), "[]");
}

#[test]
fn test_map_range_iter() {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0);
    let mut values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();

    let mut map = AvlTreeMap::new();
    for value in &values {
        map.insert(*value, value.wrapping_add(42));
    }

    values.sort();
    values.dedup();

    let start_idx = (N / 4) as usize;
    let end_idx = (N - N / 4) as usize;

    let mut range = map.range(values[start_idx]..values[end_idx]);
    for value in &values[start_idx..end_idx] {
        let kv = range.next();
        assert!(kv.is_some());
        let (&key, &mapped) = kv.unwrap();
        assert_eq!(key, *value);
        assert_eq!(mapped, value.wrapping_add(42));
    }
    assert!(range.next().is_none());

    let mut range = map.range_mut((
        Bound::Excluded(values[start_idx]),
        Bound::Included(values[end_idx]),
    ));
    for value in &values[start_idx + 1..=end_idx] {
        let kv = range.next();
        assert!(kv.is_some());
        let (&key, &mut mapped) = kv.unwrap();
        assert_eq!(key, *value);
        assert_eq!(mapped, value.wrapping_add(42));
    }
    assert!(range.next().is_none());

    let mut range = map.range(values[end_idx]..values[start_idx]);
    assert!(range.next().is_none());

    let mut range = map.range(values[start_idx]..=values[start_idx]);
    let kv = range.next();
    assert!(kv.is_some());
    let (&key, &mapped) = kv.unwrap();
    assert_eq!(key, values[start_idx]);
    assert_eq!(mapped, values[start_idx].wrapping_add(42));
    assert!(range.next().is_none());

    let mut range = map.range(values[start_idx]..values[start_idx]);
    assert!(range.next().is_none());

    let mut range = map.range((
        Bound::Excluded(values[start_idx]),
        Bound::Included(values[start_idx]),
    ));
    assert!(range.next().is_none());

    let mut range = map.range((
        Bound::Excluded(values[start_idx]),
        Bound::Excluded(values[start_idx + 1]),
    ));
    assert!(range.next().is_none());
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

    let mut value_iter = values.iter();
    for key in set.clone() {
        let value = value_iter.next();
        assert!(value.is_some());
        let value = value.unwrap();
        assert_eq!(key, *value);
    }
    assert!(value_iter.next().is_none());

    // Test debug formatting
    let mut set: AvlTreeSet<i32> = (1..4).collect();
    set.extend(4..8);
    set.extend([8, 9].iter());

    assert_eq!(format!("{:?}", set.iter()), "[1, 2, 3, 4, 5, 6, 7, 8, 9]");
    assert_eq!(
        format!("{:?}", set.clone().into_iter()),
        "[1, 2, 3, 4, 5, 6, 7, 8, 9]"
    );
    assert_eq!(format!("{:?}", set.range(3..8)), "[3, 4, 5, 6, 7]");
    assert_eq!(format!("{:?}", set.range(3..=8)), "[3, 4, 5, 6, 7, 8]");
    assert_eq!(format!("{:?}", set.range(3..=3)), "[3]");
    assert_eq!(format!("{:?}", set.range(3..3)), "[]");
    assert_eq!(format!("{:?}", set.range(4..=3)), "[]");
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
