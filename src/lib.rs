mod tree;
pub use tree::Tree;

#[cfg(test)]
mod tests {
    use super::Tree;

    #[test]
    fn test_new() {
        let tree_i32 = Tree::<i32>::new();
        assert!(tree_i32.is_empty());

        let tree_i8 = Tree::<i8>::new();
        assert!(tree_i8.is_empty());
    }

    #[test]
    fn test_insert() {
        use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

        let mut values: Vec<i32> = (0..100).collect();
        let mut rng = StdRng::seed_from_u64(0);
        values.shuffle(&mut rng);

        let mut tree = Tree::new();
        for value in values.iter() {
            assert!(tree.insert(*value));
        }
        assert!(tree.len() == values.len());

        for value in values.iter() {
            assert!(!tree.insert(*value));
        }
        assert!(tree.len() == values.len());
    }

    #[test]
    fn test_get() {
        use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

        let mut values: Vec<i32> = (0..1000).collect();
        let mut rng = StdRng::seed_from_u64(0);
        values.shuffle(&mut rng);

        let mut tree = Tree::new();
        assert!(tree.get(&42).is_none());
        for value in values.iter() {
            tree.insert(*value);
        }

        for value in values.iter() {
            let got = tree.get(value);
            assert!(got.is_some());
            assert_eq!(got.unwrap(), value);
        }
        assert!(tree.get(&-42).is_none());
    }

    #[test]
    fn test_clear() {
        use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

        let mut values: Vec<i32> = (0..1000).collect();
        let mut rng = StdRng::seed_from_u64(0);
        values.shuffle(&mut rng);

        let mut tree = Tree::new();
        for value in values.iter() {
            tree.insert(*value);
        }
        assert!(!tree.is_empty());
        assert!(tree.len() == values.len());

        tree.clear();
        assert!(tree.is_empty());
        assert!(tree.len() == 0);

        for value in values.iter() {
            assert!(tree.insert(*value));
        }
    }

    #[test]
    fn test_remove() {
        use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

        let mut values: Vec<i32> = (0..1000).collect();
        let mut rng = StdRng::seed_from_u64(0);
        values.shuffle(&mut rng);

        let mut tree = Tree::new();
        for value in values.iter() {
            tree.insert(*value);
        }

        values.shuffle(&mut rng);
        for (idx, value) in values.iter().enumerate() {
            assert!(tree.len() == values.len() - idx);
            assert!(tree.get(value).is_some());
            assert!(tree.remove(value));
            assert!(tree.get(value).is_none());
            assert!(tree.len() == values.len() - idx - 1);
        }
        assert!(tree.is_empty());
        assert!(tree.len() == 0);
    }
}
