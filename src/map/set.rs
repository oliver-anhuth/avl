//! An ordered set implemented with an AVL tree.

#![allow(dead_code)]

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::iter::FromIterator;
use std::ops::RangeBounds;

use crate::map::{AvlTreeMap, IntoIter as MapIntoIter, Iter as MapIter, Range as MapRange};

/// An ordered set implemented with an AVL tree.
///
/// ```
/// use avl::AvlTreeSet;
/// let mut set = AvlTreeSet::new();
/// set.insert(0);
/// set.insert(1);
/// set.insert(2);
/// assert_eq!(set.get(&1), Some(&1));
/// set.remove(&1);
/// assert!(set.get(&1).is_none());
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AvlTreeSet<T> {
    map: AvlTreeMap<T, ()>,
}

/// An iterator over the values of a set.
pub struct Iter<'a, T> {
    map_iter: MapIter<'a, T, ()>,
}

/// An iterator over a range of values of a set.
pub struct Range<'a, T> {
    map_range: MapRange<'a, T, ()>,
}

/// An owning iterator over the values of a set.
pub struct IntoIter<T> {
    map_into_iter: MapIntoIter<T, ()>,
}

/// A lazy iterator for the values in the union of two sets.
///
/// This `struct` is created by the [`union`] method on [`AvlTreeSet`].
///
/// [`AvlTreeSet`]: struct.AvlTreeSet.html
/// [`union`]: struct.AvlTreeSet.html#method.union
pub struct Union<'a, T> {
    lhs_peek: Option<&'a T>,
    rhs_peek: Option<&'a T>,
    lhs_iter: Iter<'a, T>,
    rhs_iter: Iter<'a, T>,
}

/// A lazy iterator for the values in the intersection of two sets.
///
/// This `struct` is created by the [`intersection`] method on [`AvlTreeSet`].
///
/// [`AvlTreeSet`]: struct.AvlTreeSet.html
/// [`intersection`]: struct.AvlTreeSet.html#method.intersection
pub struct Intersection<'a, T> {
    lhs_peek: Option<&'a T>,
    rhs_peek: Option<&'a T>,
    lhs_iter: Range<'a, T>,
    rhs_iter: Range<'a, T>,
}

impl<T: Ord> AvlTreeSet<T> {
    /// Creates an empty set.
    /// No memory is allocated until the first item is inserted.
    pub fn new() -> Self {
        Self {
            map: AvlTreeMap::new(),
        }
    }

    /// Returns a reference to the value in the set that is equal to the given value.
    ///
    /// The value may be any borrowed form of the set's value type, but the ordering
    /// on the borrowed form *must* match the ordering on the value type.
    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.get_key_value(value).map(|kv| kv.0)
    }

    /// Returns true if the set contains a value.
    ///
    /// The value may be any borrowed form of the set's value type, but the ordering
    /// on the borrowed form *must* match the ordering on the value type.
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.contains_key(value)
    }

    /// Inserts a value into the set.
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Removes a value from the set.
    /// Returns whether the value was previously in the set.
    ///
    /// The value may be any borrowed form of the set's value type, but the ordering
    /// on the borrowed form *must* match the ordering on the value type.
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.remove(value).is_some()
    }

    /// Removes a value from the set.
    /// Returns the value if it was previously in the set.
    ///
    /// The value may be any borrowed form of the set's value type, but the ordering
    /// on the borrowed form *must* match the ordering on the value type.
    pub fn take<Q>(&mut self, value: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.remove_entry(value).map(|(k, _)| k)
    }

    /// Moves all values from other into self, leaving other empty.
    pub fn append(&mut self, other: &mut Self) {
        self.map.append(&mut other.map);
    }

    /// Gets an iterator over a sub-range of values in the set in sorted order.
    ///
    /// The value may be any borrowed form of the set's value type, but the ordering
    /// on the borrowed form *must* match the ordering on the value type.
    ///
    /// # Panics
    ///
    /// Panics if range `start > end`.
    /// Panics if range `start == end` and both bounds are `Excluded`.
    pub fn range<Q, R>(&self, range: R) -> Range<'_, T>
    where
        T: Borrow<Q>,
        R: RangeBounds<Q>,
        Q: Ord + ?Sized,
    {
        Range {
            map_range: self.map.range(range),
        }
    }

    /// Gets an iterator over the values of the union set,
    /// i.e., all values in `self` or `other`, without duplicates,
    /// in ascending order.
    pub fn union<'a>(&'a self, other: &'a Self) -> Union<'a, T> {
        Union::new(self, other)
    }

    /// Gets an iterator over the values of the intersection set,
    /// i.e., all values that are botih in `self` and `other`,
    /// in ascending order.
    pub fn intersection<'a>(&'a self, other: &'a Self) -> Intersection<'a, T> {
        Intersection::new(self, other)
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.intersection(other).next().is_none()
    }

    /// Asserts that the internal tree structure is consistent.
    #[cfg(any(test, feature = "consistency_check"))]
    pub fn check_consistency(&self) {
        self.map.check_consistency()
    }
}

impl<T> AvlTreeSet<T> {
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
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            map_iter: self.map.iter(),
        }
    }
}

impl<T: Ord> Default for AvlTreeSet<T> {
    /// Creates an empty set.
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> FromIterator<T> for AvlTreeSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = Self::new();
        for value in iter {
            set.insert(value);
        }
        set
    }
}

impl<T: fmt::Debug> fmt::Debug for AvlTreeSet<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_set().entries(self.iter()).finish()
    }
}

impl<'a, T> IntoIterator for &'a AvlTreeSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for AvlTreeSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            map_into_iter: self.map.into_iter(),
        }
    }
}

impl<T> Extend<T> for AvlTreeSet<T>
where
    T: Ord + Copy,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().for_each(move |value| {
            self.insert(value);
        });
    }
}

impl<'a, T> Extend<&'a T> for AvlTreeSet<T>
where
    T: Ord + Copy,
    T: 'a,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a T>,
    {
        self.extend(iter.into_iter().copied());
    }
}

// Auto derived clone seems to have an invalid type bound of T: Clone
impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Self {
            map_iter: self.map_iter.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.map_iter.fmt_keys(f)
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.map_iter.next().map(|(k, _)| k)
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.map_iter.next_back().map(|(k, _)| k)
    }
}

// Auto derived clone seems to have an invalid type bound of T: Clone
impl<'a, T> Clone for Range<'a, T> {
    fn clone(&self) -> Self {
        Self {
            map_range: self.map_range.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Range<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.map_range.fmt_keys(f)
    }
}

impl<'a, T> Iterator for Range<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.map_range.next().map(|(k, _)| k)
    }
}

impl<'a, T> DoubleEndedIterator for Range<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.map_range.next_back().map(|(k, _)| k)
    }
}

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.map_into_iter.fmt_keys(f)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.map_into_iter.next().map(|(k, _)| k)
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.map_into_iter.next_back().map(|(k, _)| k)
    }
}

impl<'a, T: Ord> Union<'a, T> {
    fn new(lhs: &'a AvlTreeSet<T>, rhs: &'a AvlTreeSet<T>) -> Self {
        let mut lhs_iter = lhs.iter();
        let mut rhs_iter = rhs.iter();
        Self {
            lhs_peek: lhs_iter.next(),
            rhs_peek: rhs_iter.next(),
            lhs_iter,
            rhs_iter,
        }
    }
}

// Auto derived clone seems to have an invalid type bound of T: Clone
impl<'a, T> Clone for Union<'a, T> {
    fn clone(&self) -> Self {
        Self {
            lhs_peek: self.lhs_peek,
            rhs_peek: self.rhs_peek,
            lhs_iter: self.lhs_iter.clone(),
            rhs_iter: self.rhs_iter.clone(),
        }
    }
}

impl<'a, T: Ord + fmt::Debug> fmt::Debug for Union<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Union")?;
        f.debug_set().entries(self.clone()).finish()
    }
}

impl<'a, T: Ord> Iterator for Union<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lhs_peek, self.rhs_peek) {
            (None, None) => None,
            (Some(lhs), None) => {
                self.lhs_peek = self.lhs_iter.next();
                Some(lhs)
            }
            (None, Some(rhs)) => {
                self.rhs_peek = self.rhs_iter.next();
                Some(rhs)
            }
            (Some(lhs), Some(rhs)) => match lhs.cmp(rhs) {
                Ordering::Less => {
                    self.lhs_peek = self.lhs_iter.next();
                    Some(lhs)
                }
                Ordering::Equal => {
                    self.lhs_peek = self.lhs_iter.next();
                    self.rhs_peek = self.rhs_iter.next();
                    Some(lhs)
                }
                Ordering::Greater => {
                    self.rhs_peek = self.rhs_iter.next();
                    Some(rhs)
                }
            },
        }
    }
}

impl<'a, T: Ord> Intersection<'a, T> {
    fn new(lhs: &'a AvlTreeSet<T>, rhs: &'a AvlTreeSet<T>) -> Self {
        let mut lhs_iter = lhs.range(..);
        let mut rhs_iter = rhs.range(..);
        Self {
            lhs_peek: lhs_iter.next(),
            rhs_peek: rhs_iter.next(),
            lhs_iter,
            rhs_iter,
        }
    }
}

// Auto derived Clone seems to have an invalid type bound of T: Clone
impl<'a, T> Clone for Intersection<'a, T> {
    fn clone(&self) -> Self {
        Self {
            lhs_peek: self.lhs_peek,
            rhs_peek: self.rhs_peek,
            lhs_iter: self.lhs_iter.clone(),
            rhs_iter: self.rhs_iter.clone(),
        }
    }
}

impl<'a, T: Ord + fmt::Debug> fmt::Debug for Intersection<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Intersection")?;
        f.debug_set().entries(self.clone()).finish()
    }
}

impl<'a, T: Ord> Iterator for Intersection<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.lhs_peek, self.rhs_peek) {
                (None, _) | (_, None) => return None,
                (Some(lhs), Some(rhs)) => match lhs.cmp(rhs) {
                    Ordering::Equal => {
                        self.lhs_peek = self.lhs_iter.next();
                        self.rhs_peek = self.rhs_iter.next();
                        return Some(lhs);
                    }
                    Ordering::Less => {
                        self.lhs_peek = self.lhs_iter.next();
                    }
                    Ordering::Greater => {
                        self.rhs_peek = self.rhs_iter.next();
                    }
                },
            }
        }
    }
}
