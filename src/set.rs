use std::fmt;

use super::map::{IntoIter as MapIntoIter, Iter as MapIter, Map};

/// An ordered set implemented with a nearly balanced binary search tree.
pub struct Set<T> {
    map: Map<T, ()>,
}

/// An iterator over the values of a set.
pub struct Iter<'a, T> {
    map_iter: MapIter<'a, T, ()>,
}

/// An owning iterator over the values of a set.
pub struct IntoIter<T> {
    map_into_iter: MapIntoIter<T, ()>,
}

impl<T: Ord> Set<T> {
    /// Creates an empty set.
    /// No memory is allocated until the first item is inserted.
    pub fn new() -> Self {
        Self { map: Map::new() }
    }

    /// Returns a reference to the value in the set that is equal to the given value.
    pub fn get(&self, value: &T) -> Option<&T> {
        self.map.get_key_value(value).map(|kv| kv.0)
    }

    /// Inserts a value into the set.
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ())
    }

    /// Removes a value from the set.
    /// Returns whether the value was previously in the set.
    pub fn remove(&mut self, value: &T) -> bool {
        self.map.remove(value).is_some()
    }

    /// Removes a value from the set.
    /// Returns the value if it was previously in the set.
    pub fn take(&mut self, value: &T) -> Option<T> {
        self.map.remove_entry(value).map(|(k, _)| k)
    }

    /// Asserts that the internal tree structure is consistent.
    #[cfg(any(test, feature = "consistency_check"))]
    pub fn check_consistency(&self) {
        self.map.check_consistency()
    }
}

impl<T> Set<T> {
    /// Returns true if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Clears the set, deallocating all memory.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Gets an iterator over the values of the map in sorted order.
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            map_iter: self.map.iter(),
        }
    }
}

impl<T: Ord> Default for Set<T> {
    /// Creates an empty set.
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for Set<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_set().entries(self.iter()).finish()
    }
}

impl<'a, T> IntoIterator for &'a Set<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for Set<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            map_into_iter: self.map.into_iter(),
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.map_iter.next().map(|item| item.0)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.map_into_iter.next().map(|(k, _)| k)
    }
}
