use super::map::AvlTreeMap;

pub struct AvlTreeSet<T: Ord> {
    tree: AvlTreeMap<T, ()>,
}

impl<T: Ord> AvlTreeSet<T> {
    /// Creates an empty set.
    /// No memory is allocated until the first item is inserted.
    pub fn new() -> Self {
        Self {
            tree: AvlTreeMap::new(),
        }
    }

    /// Returns true if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    /// Clears the set, deallocating all memory.
    pub fn clear(&mut self) {
        self.tree.clear();
    }

    /// Returns a reference to the value in the set that is equal to the given value.
    pub fn get(&self, value: &T) -> Option<&T> {
        self.tree.get_key_value(value).map(|kv| kv.0)
    }

    /// Inserts a value pair into the set.
    pub fn insert(&mut self, value: T) -> bool {
        self.tree.insert(value, ())
    }

    /// Removes a value from the set.
    /// Returns whether the value was previously in the set.
    pub fn remove(&mut self, value: &T) -> bool {
        self.tree.remove(value)
    }

    #[cfg(test)]
    pub fn check_consistency(&self) {
        self.tree.check_consistency()
    }
}
