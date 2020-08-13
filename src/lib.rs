mod tree;

pub use tree::Tree as AvlTreeMap;

pub struct AvlTreeSet<T: Ord> {
    tree: AvlTreeMap<T, ()>,
}

impl<T: Ord> AvlTreeSet<T> {
    pub fn new() -> Self {
        Self {
            tree: AvlTreeMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tree.len()
    }

    pub fn clear(&mut self) {
        self.tree.clear();
    }

    pub fn get(&self, value: &T) -> Option<&()> {
        self.tree.get(value)
    }

    pub fn insert(&mut self, value: T) -> bool {
        self.tree.insert(value, ())
    }

    pub fn remove(&mut self, value: &T) -> bool {
        self.tree.remove(value)
    }

    #[cfg(test)]
    pub fn check_consistency(&self) {
        self.tree.check_consistency()
    }
}

#[cfg(test)]
mod tests {
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
    }

    #[test]
    fn test_get() {
        use rand::{rngs::StdRng, Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(0);
        let values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();

        let mut map = AvlTreeMap::new();
        assert!(map.get(&42).is_none());
        for value in &values {
            map.insert(*value, *value + 1);
        }

        for value in &values {
            let got = map.get(value);
            assert!(got.is_some());
            assert_eq!(*got.unwrap(), *value + 1);
        }
        assert!(map.get(&-42).is_none());
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
            map.insert(*value, ());
        }

        values.shuffle(&mut rng);
        for value in &values {
            assert!(map.get(value).is_some());
            assert!(map.remove(value));
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

        values.shuffle(&mut rng);
        values.resize(values.len() / 2, 0);
        for value in &values {
            set.remove(value);
        }
        set.check_consistency();
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
}
