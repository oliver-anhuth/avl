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

        let mut values: Vec<i32> = (0..99).collect();
        let mut rng = StdRng::seed_from_u64(0);
        values.shuffle(&mut rng);

        let mut tree = Tree::new();
        for value in values.iter() {
            assert!(tree.insert(*value));
        }
        for value in values.iter() {
            assert!(!tree.insert(*value));
        }
    }

    #[test]
    fn test_clear() {
        use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

        let mut values: Vec<i32> = (0..99).collect();
        let mut rng = StdRng::seed_from_u64(0);
        values.shuffle(&mut rng);

        let mut tree = Tree::new();
        for value in values.iter() {
            assert!(tree.insert(*value));
        }
        tree.clear();
        for value in values.iter() {
            assert!(tree.insert(*value));
        }
    }
}
