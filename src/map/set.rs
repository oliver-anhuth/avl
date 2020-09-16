//! An ordered set implemented with an AVL tree.

#![allow(dead_code)]

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::iter::FromIterator;
use std::ops::{BitAnd, BitOr, BitXor, RangeBounds, Sub};

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
    lhs: &'a AvlTreeSet<T>,
    lhs_range: Range<'a, T>,
    rhs: &'a AvlTreeSet<T>,
    rhs_range: Range<'a, T>,
}

/// A lazy iterator for the values in the difference of two sets.
///
/// This `struct` is created by the [`difference`] method on [`AvlTreeSet`].
///
/// [`AvlTreeSet`]: struct.AvlTreeSet.html
/// [`difference`]: struct.AvlTreeSet.html#method.difference
pub struct Difference<'a, T> {
    lhs_range: Range<'a, T>,
    rhs: &'a AvlTreeSet<T>,
    rhs_range: Range<'a, T>,
}

/// A lazy iterator for the values in the symmetric difference of two sets.
///
/// This `struct` is created by the [`symmetric_difference`] method on [`AvlTreeSet`].
///
/// [`AvlTreeSet`]: struct.AvlTreeSet.html
/// [`symmetric_difference`]: struct.AvlTreeSet.html#method.symmetric_difference
pub struct SymmetricDifference<'a, T> {
    lhs_iter: Iter<'a, T>,
    rhs_iter: Iter<'a, T>,
}

impl<T: Ord> AvlTreeSet<T> {
    /// Creates an empty set.
    /// No memory is allocated until the first item is inserted.
    pub fn new() -> Self {
        Self {
            map: AvlTreeMap::new(),
        }
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
}

impl<T: Ord> AvlTreeSet<T> {
    /// Inserts a value into the set.
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Moves all values from other into self, leaving other empty.
    pub fn append(&mut self, other: &mut Self) {
        self.map.append(&mut other.map);
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

    /// Gets an iterator over the values of the difference between two sets,
    /// i.e., all values that are in `self` but not in `other`,
    /// in ascending order.
    pub fn difference<'a>(&'a self, other: &'a Self) -> Difference<'a, T> {
        Difference::new(self, other)
    }

    /// Gets an iterator over the values of the symmectric difference of two sets,
    /// i.e., all values in `self` or `other`, but not in both,
    /// in ascending order.
    pub fn symmetric_difference<'a>(&'a self, other: &'a Self) -> SymmetricDifference<'a, T> {
        SymmetricDifference::new(self, other)
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.intersection(other).next().is_none()
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e., `other` contains at least all the values in `self`.
    pub fn is_subset(&self, other: &Self) -> bool {
        if self.len() > other.len() {
            return false;
        }
        for v in self {
            if !other.contains(v) {
                return false;
            }
        }
        true
    }

    /// Returns `true` if the set is a superset of another,
    /// i.e., `self` contains at least all the values in `other`.
    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    /// Asserts that the internal tree structure is consistent.
    #[cfg(any(test, feature = "consistency_check"))]
    pub fn check_consistency(&self) {
        self.map.check_consistency()
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

impl<T: Ord> Extend<T> for AvlTreeSet<T> {
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

impl<T: Ord + Clone> BitOr<&AvlTreeSet<T>> for &AvlTreeSet<T> {
    type Output = AvlTreeSet<T>;

    /// Returns the union of `self` and `rhs` as a new set.
    fn bitor(self, rhs: &AvlTreeSet<T>) -> AvlTreeSet<T> {
        self.union(rhs).cloned().collect()
    }
}

impl<T: Ord + Clone> BitAnd<&AvlTreeSet<T>> for &AvlTreeSet<T> {
    type Output = AvlTreeSet<T>;

    /// Returns the intersection of `self` and `rhs` as a new set.
    fn bitand(self, rhs: &AvlTreeSet<T>) -> AvlTreeSet<T> {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T: Ord + Clone> Sub<&AvlTreeSet<T>> for &AvlTreeSet<T> {
    type Output = AvlTreeSet<T>;

    /// Returns the difference of `self` and `rhs` as a new set.
    fn sub(self, rhs: &AvlTreeSet<T>) -> AvlTreeSet<T> {
        self.difference(rhs).cloned().collect()
    }
}

impl<T: Ord + Clone> BitXor<&AvlTreeSet<T>> for &AvlTreeSet<T> {
    type Output = AvlTreeSet<T>;

    /// Returns the symmetric difference of `self` and `rhs` as a new set.
    fn bitxor(self, rhs: &AvlTreeSet<T>) -> AvlTreeSet<T> {
        self.symmetric_difference(rhs).cloned().collect()
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

impl<'a, T> Iter<'a, T> {
    fn peek(&self) -> Option<<Self as Iterator>::Item> {
        self.map_iter.peek().map(|(k, _)| k)
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

impl<'a, T> Range<'a, T> {
    fn peek(&self) -> Option<<Self as Iterator>::Item> {
        self.map_range.peek().map(|(k, _)| k)
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

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.map_into_iter.fmt_keys(f)
    }
}

impl<'a, T: Ord> Union<'a, T> {
    fn new(lhs: &'a AvlTreeSet<T>, rhs: &'a AvlTreeSet<T>) -> Self {
        Self {
            lhs_iter: lhs.iter(),
            rhs_iter: rhs.iter(),
        }
    }
}

// Auto derived clone seems to have an invalid type bound of T: Clone
impl<'a, T> Clone for Union<'a, T> {
    fn clone(&self) -> Self {
        Self {
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
        match (self.lhs_iter.peek(), self.rhs_iter.peek()) {
            (None, None) => None,
            (Some(lhs), None) => {
                self.lhs_iter.next();
                Some(lhs)
            }
            (None, Some(rhs)) => {
                self.rhs_iter.next();
                Some(rhs)
            }
            (Some(lhs), Some(rhs)) => match lhs.cmp(rhs) {
                Ordering::Less => {
                    self.lhs_iter.next();
                    Some(lhs)
                }
                Ordering::Equal => {
                    self.lhs_iter.next();
                    self.rhs_iter.next();
                    Some(lhs)
                }
                Ordering::Greater => {
                    self.rhs_iter.next();
                    Some(rhs)
                }
            },
        }
    }
}

impl<'a, T: Ord> Intersection<'a, T> {
    fn new(lhs: &'a AvlTreeSet<T>, rhs: &'a AvlTreeSet<T>) -> Self {
        Self {
            lhs,
            lhs_range: lhs.range(..),
            rhs,
            rhs_range: rhs.range(..),
        }
    }
}

// Auto derived Clone seems to have an invalid type bound of T: Clone
impl<'a, T> Clone for Intersection<'a, T> {
    fn clone(&self) -> Self {
        Self {
            lhs: self.lhs,
            lhs_range: self.lhs_range.clone(),
            rhs: self.rhs,
            rhs_range: self.rhs_range.clone(),
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
            match (self.lhs_range.peek(), self.rhs_range.peek()) {
                (None, _) | (_, None) => return None,
                (Some(lhs), Some(rhs)) => match lhs.cmp(rhs) {
                    Ordering::Equal => {
                        self.lhs_range.next();
                        self.rhs_range.next();
                        return Some(lhs);
                    }
                    Ordering::Less => {
                        self.lhs
                            .map
                            .reset_range_start_bound_included(&mut self.lhs_range.map_range, rhs);
                    }
                    Ordering::Greater => {
                        self.rhs
                            .map
                            .reset_range_start_bound_included(&mut self.rhs_range.map_range, lhs);
                    }
                },
            }
        }
    }
}

impl<'a, T: Ord> Difference<'a, T> {
    fn new(lhs: &'a AvlTreeSet<T>, rhs: &'a AvlTreeSet<T>) -> Self {
        Self {
            lhs_range: lhs.range(..),
            rhs,
            rhs_range: rhs.range(..),
        }
    }
}

// Auto derived Clone seems to have an invalid type bound of T: Clone
impl<'a, T> Clone for Difference<'a, T> {
    fn clone(&self) -> Self {
        Self {
            lhs_range: self.lhs_range.clone(),
            rhs: self.rhs,
            rhs_range: self.rhs_range.clone(),
        }
    }
}

impl<'a, T: Ord + fmt::Debug> fmt::Debug for Difference<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Difference")?;
        f.debug_set().entries(self.clone()).finish()
    }
}

impl<'a, T: Ord> Iterator for Difference<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.lhs_range.peek(), self.rhs_range.peek()) {
                (None, _) => return None,
                (Some(lhs), None) => {
                    self.lhs_range.next();
                    return Some(lhs);
                }
                (Some(lhs), Some(rhs)) => match lhs.cmp(rhs) {
                    Ordering::Equal => {
                        self.lhs_range.next();
                        self.rhs_range.next();
                    }
                    Ordering::Less => {
                        self.lhs_range.next();
                        return Some(lhs);
                    }
                    Ordering::Greater => {
                        self.rhs
                            .map
                            .reset_range_start_bound_included(&mut self.rhs_range.map_range, lhs);
                    }
                },
            }
        }
    }
}

impl<'a, T: Ord> SymmetricDifference<'a, T> {
    fn new(lhs: &'a AvlTreeSet<T>, rhs: &'a AvlTreeSet<T>) -> Self {
        Self {
            lhs_iter: lhs.iter(),
            rhs_iter: rhs.iter(),
        }
    }
}

// Auto derived clone seems to have an invalid type bound of T: Clone
impl<'a, T> Clone for SymmetricDifference<'a, T> {
    fn clone(&self) -> Self {
        Self {
            lhs_iter: self.lhs_iter.clone(),
            rhs_iter: self.rhs_iter.clone(),
        }
    }
}

impl<'a, T: Ord + fmt::Debug> fmt::Debug for SymmetricDifference<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SymmetricDifference")?;
        f.debug_set().entries(self.clone()).finish()
    }
}

impl<'a, T: Ord> Iterator for SymmetricDifference<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.lhs_iter.peek(), self.rhs_iter.peek()) {
                (None, None) => return None,
                (Some(lhs), None) => {
                    self.lhs_iter.next();
                    return Some(lhs);
                }
                (None, Some(rhs)) => {
                    self.rhs_iter.next();
                    return Some(rhs);
                }
                (Some(lhs), Some(rhs)) => match lhs.cmp(rhs) {
                    Ordering::Less => {
                        self.lhs_iter.next();
                        return Some(lhs);
                    }
                    Ordering::Equal => {
                        self.lhs_iter.next();
                        self.rhs_iter.next();
                    }
                    Ordering::Greater => {
                        self.rhs_iter.next();
                        return Some(rhs);
                    }
                },
            }
        }
    }
}
