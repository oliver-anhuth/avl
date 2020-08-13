mod tree;

pub use tree::Tree as AvlTreeMap;

pub struct AvlTreeSet<T: PartialOrd> {
    tree: AvlTreeMap<T, ()>,
}

impl<T: PartialOrd> AvlTreeSet<T> {
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

        let set_i32 = AvlTreeSet::<i32>::new();
        assert!(set_i32.is_empty());
        set_i32.check_consistency();

        let map_i8 = AvlTreeMap::<i8, ()>::new();
        assert!(map_i8.is_empty());
        map_i8.check_consistency();

        let set_i8 = AvlTreeSet::<i8>::new();
        assert!(set_i8.is_empty());
        set_i8.check_consistency();

        let map_string = AvlTreeMap::<String, String>::new();
        assert!(map_string.is_empty());
        map_string.check_consistency();

        let set_string = AvlTreeSet::<String>::new();
        assert!(set_string.is_empty());
        set_string.check_consistency();
    }

    #[test]
    fn test_insert() {
        use rand::{rngs::StdRng, Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(0);
        let mut values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();
        values.sort();
        values.dedup();

        let mut tree = AvlTreeMap::new();
        for value in &values {
            assert!(tree.insert(*value, *value));
            tree.check_consistency();
        }
        assert!(tree.len() == values.len());

        for value in &values {
            assert!(!tree.insert(*value, *value));
        }
        assert!(tree.len() == values.len());
    }

    #[test]
    fn test_insert_sorted_range() {
        let mut tree = AvlTreeMap::new();
        for value in 0..N {
            assert!(tree.insert(value, value));
            tree.check_consistency();
        }
        assert!(tree.len() == N as usize);
        assert!(tree.height() > 0);
        assert!(tree.height() < N as usize / 2);
    }

    #[test]
    fn test_insert_shuffled_range() {
        use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

        let mut values: Vec<i32> = (0..N).collect();
        let mut rng = StdRng::seed_from_u64(0);
        values.shuffle(&mut rng);

        let mut tree = AvlTreeMap::new();
        for value in &values {
            assert!(tree.insert(*value, "foo"));
            tree.check_consistency();
        }
        assert!(tree.len() == values.len());

        for value in &values {
            assert!(!tree.insert(*value, "bar"));
        }
        assert!(tree.len() == values.len());
    }

    #[test]
    fn test_get() {
        use rand::{rngs::StdRng, Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(0);
        let values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();

        let mut tree = AvlTreeMap::new();
        assert!(tree.get(&42).is_none());
        for value in &values {
            tree.insert(*value, *value + 1);
        }

        for value in &values {
            let got = tree.get(value);
            assert!(got.is_some());
            assert_eq!(*got.unwrap(), *value + 1);
        }
        assert!(tree.get(&-42).is_none());
    }

    #[test]
    fn test_clear() {
        use rand::{rngs::StdRng, Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(0);
        let mut values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();
        values.sort();
        values.dedup();

        let mut tree = AvlTreeMap::new();
        for value in &values {
            tree.insert(*value, String::from("foo"));
        }
        assert!(!tree.is_empty());
        assert!(tree.len() == values.len());

        tree.clear();
        assert!(tree.is_empty());
        assert!(tree.len() == 0);

        for value in &values {
            assert!(tree.insert(*value, String::from("bar")));
        }
        assert!(!tree.is_empty());
        assert!(tree.len() == values.len());
        tree.check_consistency();
    }

    #[test]
    fn test_remove() {
        use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(0);
        let mut values: Vec<i32> = (0..N).map(|_| rng.gen()).collect();
        values.sort();
        values.dedup();

        let mut tree = AvlTreeMap::new();
        for value in &values {
            tree.insert(*value, ());
        }

        values.shuffle(&mut rng);
        for value in &values {
            assert!(tree.get(value).is_some());
            assert!(tree.remove(value));
            assert!(tree.get(value).is_none());
            tree.check_consistency();
        }
        assert!(tree.is_empty());
        assert!(tree.len() == 0);
    }

    #[test]
    #[ignore]
    fn test_large() {
        use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(0);
        let mut values: Vec<i32> = (0..LARGE_N).map(|_| rng.gen_range(0, LARGE_N)).collect();

        let mut tree = AvlTreeMap::new();
        for value in &values {
            tree.insert(*value, *value);
        }
        tree.check_consistency();

        values.shuffle(&mut rng);
        values.resize(values.len() / 2, 0);
        for value in &values {
            tree.remove(value);
        }
        tree.check_consistency();
    }
}
