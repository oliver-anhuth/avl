#![allow(dead_code)]

use std::cmp::{self, Ordering};
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

/// An ordered map implemented with a nearly balanced binary search tree.
///
/// ```
/// use avl::AvlTreeMap;
/// let mut map = AvlTreeMap::new();
/// map.insert(0, "zero");
/// map.insert(1, "one");
/// map.insert(2, "two");
/// assert_eq!(map.get(&1), Some(&"one"));
/// map.remove(&1);
/// assert!(map.get(&1).is_none());
/// ```
pub struct Map<K, V> {
    root: Link<K, V>,
    num_nodes: usize,
}

/// A node in the binary search tree, containing links to its parent node, left child, right child,
/// its height (== maximum number of links to a leaf node) and a key, a value.
struct Node<K, V> {
    parent: Link<K, V>,
    left: Link<K, V>,
    right: Link<K, V>,
    height: u16,
    key: K,
    value: V,
}

type NodePtr<K, V> = NonNull<Node<K, V>>;
type Link<K, V> = Option<NodePtr<K, V>>;
type LinkPtr<K, V> = NonNull<Link<K, V>>;

/// Insert position is parent + link to modify.
type InsertPos<K, V> = (Link<K, V>, LinkPtr<K, V>);

/// An iterator over the entries of a map.
pub struct Iter<'a, K, V> {
    node_iter: NodeIter<'a, K, V>,
}

/// An iterator over the keys of a map.
pub struct Keys<'a, K, V> {
    node_iter: NodeIter<'a, K, V>,
}

/// An iterator over the values of a map.
pub struct Values<'a, K, V> {
    node_iter: NodeIter<'a, K, V>,
}

/// A mutable iterator over the entries of a map.
pub struct IterMut<'a, K, V> {
    node_iter: NodeIter<'a, K, V>,
}

/// An iterator over the values of a map.
pub struct ValuesMut<'a, K, V> {
    node_iter: NodeIter<'a, K, V>,
}

/// An owning iterator over the entries of a map.
pub struct IntoIter<K, V> {
    node_eater: NodeEater<K, V>,
}

/// Specifies a range [first, last] of tree nodes.
/// Allows iteration by successively narrowing the range from either end.
struct NodeIter<'a, K, V> {
    first: Link<K, V>,
    last: Link<K, V>,
    marker: PhantomData<&'a Node<K, V>>,
}

struct NodeEater<K, V> {
    first: Link<K, V>,
    last: Link<K, V>,
}

impl<K: Ord, V> Map<K, V> {
    /// Creates an empty map.
    /// No memory is allocated until the first item is inserted.
    pub fn new() -> Self {
        Self {
            root: None,
            num_nodes: 0,
        }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(node_ptr) = self.find(key) {
            return Some(&unsafe { &*node_ptr.as_ptr() }.value);
        }
        None
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(node_ptr) = self.find(key) {
            return Some(&mut unsafe { &mut *node_ptr.as_ptr() }.value);
        }
        None
    }

    /// Returns references to the key-value pair corresponding to the key.
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        if let Some(node_ptr) = self.find(key) {
            return Some((
                &unsafe { &*node_ptr.as_ptr() }.key,
                &unsafe { &*node_ptr.as_ptr() }.value,
            ));
        }
        None
    }

    /// Returns true if the key is in the map, else false.
    pub fn contains_key(&self, key: &K) -> bool {
        self.find(key).is_some()
    }

    /// Inserts a key-value pair into the map.
    /// Returns None if the key is not in the map.
    /// Updates the value if the key is already in the map and returns the old value.
    pub fn insert(&mut self, key: K, mut value: V) -> Option<V> {
        match self.find_insert_pos(&key) {
            Ok((parent, mut link_ptr)) => {
                unsafe {
                    *link_ptr.as_mut() = Some(Node::create(parent, key, value));
                }
                self.num_nodes += 1;
                if let Some(parent_ptr) = parent {
                    self.rebalance_once(parent_ptr);
                }
                None
            }
            Err(mut node_ptr) => {
                unsafe {
                    mem::swap(&mut node_ptr.as_mut().value, &mut value);
                }
                Some(value)
            }
        }
    }

    /// Removes a key from the map.
    /// Returns the value at the key if the key was previously in the map.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.remove_entry(key).map(|(_, v)| v)
    }

    /// Removes a key from the map.
    /// Returns the stored key and value if the key was previously in the map.
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        // Find node to-be-removed
        if let Some(node_ptr) = self.find(key) {
            debug_assert!(self.num_nodes > 0);
            self.num_nodes -= 1;
            self.unlink_node(node_ptr);
            let kv = unsafe { Node::destroy(node_ptr) };
            debug_assert!(self.find(key).is_none());
            return Some(kv);
        }
        None
    }

    /// Asserts that the internal tree structure is consistent.
    #[cfg(any(test, feature = "consist^ency_check"))]
    pub fn check_consistency(&self) {
        unsafe {
            // Check root link
            if let Some(root_node_ptr) = self.root {
                assert!(root_node_ptr.as_ref().parent.is_none());
            }

            // Check tree nodes
            let mut num_nodes = 0;
            self.preorder(|node_ptr| {
                let mut height = 0;
                let mut left_height = 0;
                let mut right_height = 0;

                // Check link for left child node
                if let Some(left_ptr) = node_ptr.as_ref().left {
                    assert!(left_ptr.as_ref().parent == Some(node_ptr));
                    assert!(left_ptr.as_ref().key < node_ptr.as_ref().key);
                    left_height = left_ptr.as_ref().height + 1;
                    height = cmp::max(height, left_height);
                }

                // Check link for right child node
                if let Some(right_ptr) = node_ptr.as_ref().right {
                    assert!(right_ptr.as_ref().parent == Some(node_ptr));
                    assert!(right_ptr.as_ref().key > node_ptr.as_ref().key);
                    right_height = right_ptr.as_ref().height + 1;
                    height = cmp::max(height, right_height);
                }

                // Check height
                assert_eq!(node_ptr.as_ref().height, height);
                assert!(height <= 128, "Should hold for all 64 bit address spaces");

                // Check AVL condition (nearly balance)
                assert!(left_height <= right_height + 1);
                assert!(right_height <= left_height + 1);

                num_nodes += 1;
            });

            // Check number of nodes
            assert_eq!(num_nodes, self.num_nodes);
        }
    }
}

impl<K, V> Map<K, V> {
    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.num_nodes
    }

    #[cfg(test)]
    pub fn height(&self) -> u16 {
        match self.root {
            None => 0,
            Some(root_ptr) => unsafe { root_ptr.as_ref().height },
        }
    }

    /// Clears the map, deallocating all memory.
    pub fn clear(&mut self) {
        self.postorder(|node_ptr| unsafe {
            Node::destroy(node_ptr);
        });
        self.root = None;
        self.num_nodes = 0;
    }

    /// Gets an iterator over the entries of the map, sorted by key.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            node_iter: NodeIter::new(self.find_min(), self.find_max()),
        }
    }

    /// Gets an iterator over the keys of the map, in sorted order.
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            node_iter: NodeIter::new(self.find_min(), self.find_max()),
        }
    }

    /// Gets an iterator over the values of the map, in order by key.
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            node_iter: NodeIter::new(self.find_min(), self.find_max()),
        }
    }

    /// Gets a mutable iterator over the values of the map, in order by key.
    pub fn values_mut(&self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            node_iter: NodeIter::new(self.find_min(), self.find_max()),
        }
    }

    /// Gets a mutable iterator over the entries of the map, sorted by key.
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            node_iter: NodeIter::new(self.find_min(), self.find_max()),
        }
    }
}

impl<K: Ord, V> Map<K, V> {
    fn find(&self, key: &K) -> Link<K, V> {
        let mut current = self.root;
        while let Some(node_ptr) = current {
            current = unsafe {
                match key.cmp(&node_ptr.as_ref().key) {
                    Ordering::Equal => break,
                    Ordering::Less => node_ptr.as_ref().left,
                    Ordering::Greater => node_ptr.as_ref().right,
                }
            }
        }
        current
    }

    fn find_insert_pos(&mut self, key: &K) -> Result<InsertPos<K, V>, NodePtr<K, V>> {
        let mut parent: Link<K, V> = None;
        let mut link_ptr: LinkPtr<K, V> = unsafe { LinkPtr::new_unchecked(&mut self.root) };
        unsafe {
            while let Some(mut node_ptr) = link_ptr.as_ref() {
                if *key == node_ptr.as_ref().key {
                    return Err(node_ptr);
                } else {
                    parent = *link_ptr.as_ref();
                    if *key < node_ptr.as_ref().key {
                        link_ptr = LinkPtr::new_unchecked(&mut node_ptr.as_mut().left);
                    } else {
                        link_ptr = LinkPtr::new_unchecked(&mut node_ptr.as_mut().right);
                    }
                }
            }
        }
        Ok((parent, link_ptr))
    }
}

impl<K, V> Map<K, V> {
    fn find_min(&self) -> Link<K, V> {
        match self.root {
            None => None,
            Some(mut min_ptr) => {
                while let Some(left_ptr) = unsafe { min_ptr.as_ref().left } {
                    min_ptr = left_ptr;
                }
                Some(min_ptr)
            }
        }
    }

    fn find_max(&self) -> Link<K, V> {
        match self.root {
            None => None,
            Some(mut max_ptr) => {
                while let Some(right_ptr) = unsafe { max_ptr.as_ref().right } {
                    max_ptr = right_ptr;
                }
                Some(max_ptr)
            }
        }
    }

    fn unlink_node(&mut self, node_ptr: NodePtr<K, V>) {
        unsafe {
            // Check if node to-unlink has right sub tree
            if let Some(mut min_child_ptr) = node_ptr.as_ref().right {
                // Replace node by smallest child in right sub tree
                //  |             |
                //  *             1
                // / \           / \
                //    A             A
                //   / \    =>     / \
                //  1             B
                //   \
                //    B
                let mut min_child_parent_ptr = node_ptr;
                while let Some(left_ptr) = min_child_ptr.as_ref().left {
                    min_child_parent_ptr = min_child_ptr;
                    min_child_ptr = left_ptr;
                }

                // Smallest child node is stem or leaf, unlink from tree
                debug_assert!(min_child_ptr.as_ref().left.is_none());
                if min_child_parent_ptr.as_ref().left == Some(min_child_ptr) {
                    min_child_parent_ptr.as_mut().left = min_child_ptr.as_ref().right;
                } else {
                    min_child_parent_ptr.as_mut().right = min_child_ptr.as_ref().right;
                }
                if let Some(mut right_ptr) = min_child_ptr.as_ref().right {
                    right_ptr.as_mut().parent = min_child_ptr.as_ref().parent;
                }

                // Replace node to-unlink by smallest child node (up to 6 links)
                min_child_ptr.as_mut().left = node_ptr.as_ref().left;
                if let Some(mut left_ptr) = node_ptr.as_ref().left {
                    left_ptr.as_mut().parent = Some(min_child_ptr);
                }

                min_child_ptr.as_mut().right = node_ptr.as_ref().right;
                if let Some(mut right_ptr) = node_ptr.as_ref().right {
                    right_ptr.as_mut().parent = Some(min_child_ptr);
                }

                min_child_ptr.as_mut().parent = node_ptr.as_ref().parent;
                match node_ptr.as_ref().parent {
                    None => self.root = Some(min_child_ptr),
                    Some(mut parent_ptr) => {
                        if parent_ptr.as_ref().left == Some(node_ptr) {
                            parent_ptr.as_mut().left = Some(min_child_ptr);
                        } else {
                            parent_ptr.as_mut().right = Some(min_child_ptr);
                        }
                    }
                }

                // Parent of smallest child node might be out of balance now
                let mut rebalance_from = min_child_parent_ptr;
                if rebalance_from == node_ptr {
                    // Parent is node to-unlink and has been replaced by smallest child
                    rebalance_from = min_child_ptr;
                }
                self.rebalance(rebalance_from);
            } else {
                // Node to-unlink is stem or leaf, unlink from tree.
                //   |        |
                //   *   =>   A
                //  /
                // A
                debug_assert!(node_ptr.as_ref().right.is_none());
                if let Some(mut left_ptr) = node_ptr.as_ref().left {
                    left_ptr.as_mut().parent = node_ptr.as_ref().parent;
                }
                match node_ptr.as_ref().parent {
                    None => self.root = node_ptr.as_ref().left,
                    Some(mut parent_ptr) => {
                        if parent_ptr.as_ref().left == Some(node_ptr) {
                            parent_ptr.as_mut().left = node_ptr.as_ref().left;
                        } else {
                            parent_ptr.as_mut().right = node_ptr.as_ref().left
                        }
                        // Parent node might be out of balance now
                        self.rebalance(parent_ptr);
                    }
                }
            }
        }
    }

    fn left_height(node_ptr: NodePtr<K, V>) -> u16 {
        unsafe {
            match node_ptr.as_ref().left {
                None => 0,
                Some(left_ptr) => left_ptr.as_ref().height + 1,
            }
        }
    }

    fn right_height(node_ptr: NodePtr<K, V>) -> u16 {
        unsafe {
            match node_ptr.as_ref().right {
                None => 0,
                Some(right_ptr) => right_ptr.as_ref().height + 1,
            }
        }
    }

    fn adjust_height(mut node_ptr: NodePtr<K, V>) {
        unsafe {
            node_ptr.as_mut().height = cmp::max(
                match node_ptr.as_ref().left {
                    None => 0,
                    Some(left_ptr) => left_ptr.as_ref().height + 1,
                },
                match node_ptr.as_ref().right {
                    None => 0,
                    Some(right_ptr) => right_ptr.as_ref().height + 1,
                },
            );
        }
    }

    /// Rotate given node to the left.
    /// ```none
    ///  |                |
    ///  *                1
    /// / \              / \
    ///    1      =>    *   2
    ///   / \          /   / \
    ///      2
    ///     / \
    /// ```
    fn rotate_left(&mut self, mut node_ptr: NodePtr<K, V>) {
        unsafe {
            if let Some(mut right_ptr) = node_ptr.as_ref().right {
                node_ptr.as_mut().right = right_ptr.as_ref().left;
                if let Some(mut right_left_ptr) = right_ptr.as_mut().left {
                    right_left_ptr.as_mut().parent = Some(node_ptr);
                }

                right_ptr.as_mut().parent = node_ptr.as_ref().parent;
                match node_ptr.as_ref().parent {
                    None => self.root = Some(right_ptr),
                    Some(mut parent_ptr) => {
                        if parent_ptr.as_ref().left == Some(node_ptr) {
                            parent_ptr.as_mut().left = Some(right_ptr);
                        } else {
                            parent_ptr.as_mut().right = Some(right_ptr);
                        }
                    }
                }

                right_ptr.as_mut().left = Some(node_ptr);
                node_ptr.as_mut().parent = Some(right_ptr);

                Self::adjust_height(node_ptr);
                Self::adjust_height(right_ptr);
            }
        }
    }

    /// Rotate given node to the right.
    /// ```none
    ///      *            1
    ///     / \          / \
    ///    1      =>    2   *
    ///   / \          / \ / \
    ///  2
    /// / \
    /// ```
    fn rotate_right(&mut self, mut node_ptr: NodePtr<K, V>) {
        unsafe {
            if let Some(mut left_ptr) = node_ptr.as_ref().left {
                node_ptr.as_mut().left = left_ptr.as_ref().right;
                if let Some(mut right_ptr) = left_ptr.as_ref().right {
                    right_ptr.as_mut().parent = Some(node_ptr);
                }

                left_ptr.as_mut().parent = node_ptr.as_ref().parent;
                match node_ptr.as_ref().parent {
                    None => self.root = Some(left_ptr),
                    Some(mut parent_ptr) => {
                        if parent_ptr.as_ref().left == Some(node_ptr) {
                            parent_ptr.as_mut().left = Some(left_ptr);
                        } else {
                            parent_ptr.as_mut().right = Some(left_ptr);
                        }
                    }
                }

                left_ptr.as_mut().right = Some(node_ptr);
                node_ptr.as_mut().parent = Some(left_ptr);

                Self::adjust_height(node_ptr);
                Self::adjust_height(left_ptr);
            }
        }
    }

    /// Rebalances nodes starting from given position up to the root node.
    fn rebalance(&mut self, start_from: NodePtr<K, V>) {
        let mut current = Some(start_from);
        while let Some(node_ptr) = current {
            let parent = unsafe { node_ptr.as_ref().parent };
            self.rebalance_node(node_ptr);
            current = parent;
        }
    }

    /// Rebalances nodes starting from given position up to the root node.
    /// Stops after first rebalance operation.
    /// This is enough to restore balance after a single insert operation.
    fn rebalance_once(&mut self, start_from: NodePtr<K, V>) {
        let mut current = Some(start_from);
        while let Some(node_ptr) = current {
            let parent = unsafe { node_ptr.as_ref().parent };
            let did_rebalance = self.rebalance_node(node_ptr);
            if did_rebalance {
                break;
            }
            current = parent;
        }
    }

    /// Restores AVL condition (balance) at given node if necessary and adjusts height.
    /// Resulting balance will be +1, 0 or -1 height difference between left and right subtree.
    /// Initial balance must node exceed +2 or -2, which always holds after a single update.
    /// Returns whether rebalancing had been necessary.
    fn rebalance_node(&mut self, node_ptr: NodePtr<K, V>) -> bool {
        unsafe {
            let left_height = Self::left_height(node_ptr);
            let right_height = Self::right_height(node_ptr);
            debug_assert!(left_height <= right_height + 2);
            debug_assert!(right_height <= left_height + 2);
            if left_height > right_height + 1 {
                // Rebalance right
                let left_ptr = node_ptr.as_ref().left.unwrap();
                if Self::right_height(left_ptr) > Self::left_height(left_ptr) {
                    self.rotate_left(left_ptr);
                }
                self.rotate_right(node_ptr);
                true
            } else if right_height > left_height + 1 {
                // Rebalance left
                let right_ptr = node_ptr.as_ref().right.unwrap();
                if Self::left_height(right_ptr) > Self::right_height(right_ptr) {
                    self.rotate_right(right_ptr);
                }
                self.rotate_left(node_ptr);
                true
            } else {
                Self::adjust_height(node_ptr);
                false
            }
        }
    }

    #[allow(dead_code)]
    fn preorder<F: FnMut(NodePtr<K, V>)>(&self, f: F) {
        Self::traverse(self.root, f, |_| {}, |_| {});
    }

    #[allow(dead_code)]
    fn inorder<F: FnMut(NodePtr<K, V>)>(&self, f: F) {
        Self::traverse(self.root, |_| {}, f, |_| {});
    }

    fn postorder<F: FnMut(NodePtr<K, V>)>(&self, f: F) {
        Self::traverse(self.root, |_| {}, |_| {}, f);
    }

    fn traverse<Pre, In, Post>(
        start: Link<K, V>,
        mut preorder: Pre,
        mut inorder: In,
        mut postorder: Post,
    ) where
        Pre: FnMut(NodePtr<K, V>),
        In: FnMut(NodePtr<K, V>),
        Post: FnMut(NodePtr<K, V>),
    {
        #[allow(clippy::enum_variant_names)]
        enum Direction {
            FromParent,
            FromLeft,
            FromRight,
        }

        if let Some(mut node_ptr) = start {
            let mut dir = Direction::FromParent;
            loop {
                match dir {
                    Direction::FromParent => {
                        preorder(node_ptr);
                        if let Some(left_ptr) = unsafe { node_ptr.as_ref().left } {
                            node_ptr = left_ptr;
                        } else {
                            dir = Direction::FromLeft;
                        }
                    }
                    Direction::FromLeft => {
                        inorder(node_ptr);
                        if let Some(right_ptr) = unsafe { node_ptr.as_ref().right } {
                            node_ptr = right_ptr;
                            dir = Direction::FromParent;
                        } else {
                            dir = Direction::FromRight;
                        }
                    }
                    Direction::FromRight => {
                        // Post order traversal is used for node deletion,
                        // so make sure not to use node pointer after postorder call.
                        if let Some(parent_ptr) = unsafe { node_ptr.as_ref().parent } {
                            if Some(node_ptr) == unsafe { parent_ptr.as_ref().left } {
                                dir = Direction::FromLeft;
                            } else {
                                dir = Direction::FromRight;
                            }
                            postorder(node_ptr);
                            node_ptr = parent_ptr;
                        } else {
                            postorder(node_ptr);
                            break;
                        }
                    }
                }
            }
        }
    }
}

impl<K, V> Drop for Map<K, V> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K: Ord, V> Default for Map<K, V> {
    /// Creates an empty map.
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> fmt::Debug for Map<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_map().entries(self.iter()).finish()
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut Map<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            node_eater: NodeEater::new(self),
        }
    }
}

impl<K, V> Node<K, V> {
    unsafe fn create(parent: Link<K, V>, key: K, value: V) -> NodePtr<K, V> {
        let boxed = Box::new(Node {
            parent,
            left: None,
            right: None,
            height: 0,
            key,
            value,
        });
        NodePtr::new_unchecked(Box::into_raw(boxed))
    }

    unsafe fn destroy(node_ptr: NodePtr<K, V>) -> (K, V) {
        let boxed = Box::from_raw(node_ptr.as_ptr());
        (boxed.key, boxed.value)
    }
}

impl<K, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            node_iter: NodeIter::new(self.node_iter.first, self.node_iter.last),
        }
    }
}

impl<K, V> fmt::Debug for Iter<'_, K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        let mut sep = "";
        for (key, value) in self.clone() {
            write!(f, "{}({:?}, {:?})", sep, key, value)?;
            sep = ", ";
        }
        write!(f, "]")
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        match self.node_iter.pop_first() {
            None => None,
            Some(node_ptr) => unsafe {
                let key: &'a K = &(*node_ptr.as_ptr()).key;
                let value: &'a V = &(*node_ptr.as_ptr()).value;
                Some((key, value))
            },
        }
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.node_iter.pop_last() {
            None => None,
            Some(node_ptr) => unsafe {
                let key: &'a K = &(*node_ptr.as_ptr()).key;
                let value: &'a V = &(*node_ptr.as_ptr()).value;
                Some((key, value))
            },
        }
    }
}

impl<K, V> Clone for Keys<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            node_iter: NodeIter::new(self.node_iter.first, self.node_iter.last),
        }
    }
}

impl<K, V> fmt::Debug for Keys<'_, K, V>
where
    K: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        let mut sep = "";
        for key in self.clone() {
            write!(f, "{}{:?}", sep, key)?;
            sep = ", ";
        }
        write!(f, "]")
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        match self.node_iter.pop_first() {
            None => None,
            Some(node_ptr) => unsafe {
                let key: &'a K = &(*node_ptr.as_ptr()).key;
                Some(key)
            },
        }
    }
}

impl<'a, K, V> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.node_iter.pop_last() {
            None => None,
            Some(node_ptr) => unsafe {
                let key: &'a K = &(*node_ptr.as_ptr()).key;
                Some(key)
            },
        }
    }
}

impl<K, V> Clone for Values<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            node_iter: NodeIter::new(self.node_iter.first, self.node_iter.last),
        }
    }
}

impl<K, V> fmt::Debug for Values<'_, K, V>
where
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        let mut sep = "";
        for value in self.clone() {
            write!(f, "{}{:?}", sep, value)?;
            sep = ", ";
        }
        write!(f, "]")
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        match self.node_iter.pop_first() {
            None => None,
            Some(node_ptr) => unsafe {
                let value: &'a V = &(*node_ptr.as_ptr()).value;
                Some(value)
            },
        }
    }
}

impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.node_iter.pop_last() {
            None => None,
            Some(node_ptr) => unsafe {
                let value: &'a V = &(*node_ptr.as_ptr()).value;
                Some(value)
            },
        }
    }
}
impl<K, V> fmt::Debug for IterMut<'_, K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        let mut sep = "";
        for (key, value) in (Iter {
            node_iter: NodeIter::new(self.node_iter.first, self.node_iter.last),
        }) {
            write!(f, "{}({:?}, {:?})", sep, key, value)?;
            sep = ", ";
        }
        write!(f, "]")
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        match self.node_iter.pop_first() {
            None => None,
            Some(node_ptr) => unsafe {
                let key: &'a K = &(*node_ptr.as_ptr()).key;
                let value: &'a mut V = &mut (*node_ptr.as_ptr()).value;
                Some((key, value))
            },
        }
    }
}

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.node_iter.pop_last() {
            None => None,
            Some(node_ptr) => unsafe {
                let key: &'a K = &(*node_ptr.as_ptr()).key;
                let value: &'a mut V = &mut (*node_ptr.as_ptr()).value;
                Some((key, value))
            },
        }
    }
}

impl<K, V> fmt::Debug for ValuesMut<'_, K, V>
where
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        let mut sep = "";
        for value in (Values {
            node_iter: NodeIter::new(self.node_iter.first, self.node_iter.last),
        }) {
            write!(f, "{}{:?}", sep, value)?;
            sep = ", "
        }
        write!(f, "]")
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<Self::Item> {
        match self.node_iter.pop_first() {
            None => None,
            Some(node_ptr) => unsafe {
                let value: &'a mut V = &mut (*node_ptr.as_ptr()).value;
                Some(value)
            },
        }
    }
}

impl<'a, K, V> DoubleEndedIterator for ValuesMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.node_iter.pop_last() {
            None => None,
            Some(node_ptr) => unsafe {
                let value: &'a mut V = &mut (*node_ptr.as_ptr()).value;
                Some(value)
            },
        }
    }
}

impl<'a, K, V> NodeIter<'a, K, V> {
    fn new(first: Link<K, V>, last: Link<K, V>) -> Self {
        NodeIter {
            first,
            last,
            marker: PhantomData,
        }
    }

    fn pop_first(&mut self) -> Link<K, V> {
        let first = self.first;
        if let Some(node_ptr) = first {
            if self.first == self.last {
                // Last remaining node in the range -> end iteration
                self.first = None;
                self.last = None;
            } else if let Some(mut next_ptr) = unsafe { node_ptr.as_ref().right } {
                // Next node in range is smallest child in right sub tree
                while let Some(left_ptr) = unsafe { next_ptr.as_ref().left } {
                    next_ptr = left_ptr;
                }
                self.first = Some(next_ptr);
            } else {
                // No right sub tree, walk upwards in the chain of parents,
                // next node is the first parent which is reached from a left child.
                // Note that we should not fall off the root, i.e. have a None parent,
                // this would indicate an invalid node range (last before first, not same tree, etc).
                let mut next_ptr = node_ptr;
                loop {
                    let from = Some(next_ptr);
                    match unsafe { next_ptr.as_ref().parent } {
                        None => panic!("Invalid node range"),
                        Some(parent_ptr) => {
                            next_ptr = parent_ptr;
                            if unsafe { next_ptr.as_ref().left } == from {
                                // We came back from left child -> this is the next node
                                break;
                            }
                        }
                    }
                }
                self.first = Some(next_ptr);
            }
        }
        first
    }

    fn pop_last(&mut self) -> Link<K, V> {
        let last = self.last;
        if let Some(node_ptr) = last {
            if self.last == self.first {
                // Last remaining node in the range -> end iteration
                self.first = None;
                self.last = None;
            } else if let Some(mut next_ptr) = unsafe { node_ptr.as_ref().left } {
                // Next node in range is biggest child in left sub tree
                while let Some(right_ptr) = unsafe { next_ptr.as_ref().right } {
                    next_ptr = right_ptr;
                }
                self.last = Some(next_ptr);
            } else {
                // No left sub tree, walk upwards in the chain of parents,
                // next node is the first parent which is reached from a right child.
                // Note that we should not fall off the root, i.e. have a None parent,
                // this would indicate an invalid node range (last before first, not same tree, etc).
                let mut next_ptr = node_ptr;
                loop {
                    let from = Some(next_ptr);
                    match unsafe { next_ptr.as_ref().parent } {
                        None => panic!("Invalid node range"),
                        Some(parent_ptr) => {
                            next_ptr = parent_ptr;
                            if unsafe { next_ptr.as_ref().right } == from {
                                // We came back from right child -> this is the next node
                                break;
                            }
                        }
                    }
                }
                self.last = Some(next_ptr);
            }
        }
        last
    }
}

impl<K, V> IntoIter<K, V>
where
    K: fmt::Debug,
{
    pub fn fmt_keys(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let keys = Keys {
            node_iter: NodeIter::new(self.node_eater.first, self.node_eater.last),
        };
        write!(f, "{:?}", keys)
    }
}

impl<K, V> fmt::Debug for IntoIter<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        let mut sep = "";
        for (key, value) in (Iter {
            node_iter: NodeIter::new(self.node_eater.first, self.node_eater.last),
        }) {
            write!(f, "{}({:?}, {:?})", sep, key, value)?;
            sep = ", ";
        }
        write!(f, "]")
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        self.node_eater.pop_first()
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.node_eater.pop_last()
    }
}

impl<K, V> NodeEater<K, V> {
    fn new(mut map: Map<K, V>) -> Self {
        let node_eater = Self {
            first: map.find_min(),
            last: map.find_max(),
        };
        map.root.take();
        node_eater
    }

    fn pop_first(&mut self) -> Option<(K, V)> {
        let mut first = self.first;
        if let Some(node_ptr) = first {
            if self.first == self.last {
                // Last remaining node in the range -> end iteration
                self.first = None;
                self.last = None;
            } else {
                unsafe {
                    // Replace node with right sub tree
                    match node_ptr.as_ref().parent {
                        None => {
                            // Current node has no parent, right child becomes root of the tree
                            // *            1
                            //  \     =>   / \
                            //   1
                            //  / \
                            first = node_ptr.as_ref().right;
                            if let Some(mut root_ptr) = first {
                                root_ptr.as_mut().parent = None;
                            }
                        }
                        Some(mut parent_ptr) => {
                            // Right child becomes parent's left sub tree
                            //   A            A
                            //  / \          / \
                            // *      =>    1
                            //  \          / \
                            //   1
                            //  / \
                            parent_ptr.as_mut().left = node_ptr.as_ref().right;
                            if let Some(mut left_ptr) = parent_ptr.as_ref().left {
                                left_ptr.as_mut().parent = Some(parent_ptr);
                                first = Some(left_ptr);
                            } else {
                                first = Some(parent_ptr);
                            }
                        }
                    }

                    // Next node is smallest child in sub tree.
                    if let Some(mut next_ptr) = first {
                        while let Some(left_ptr) = next_ptr.as_ref().left {
                            next_ptr = left_ptr;
                        }
                        first = Some(next_ptr);
                    }

                    self.first = first;
                }
            }

            return Some(unsafe { Node::destroy(node_ptr) });
        }

        None
    }

    fn pop_last(&mut self) -> Option<(K, V)> {
        let mut last = self.last;
        if let Some(node_ptr) = last {
            if self.last == self.first {
                // Last remaining node in the range -> end iteration
                self.first = None;
                self.last = None;
            } else {
                unsafe {
                    // Replace node with left sub tree
                    match node_ptr.as_ref().parent {
                        None => {
                            // Current node has no parent, left child becomes root of the tree
                            // Current node has no parent, right child becomes root of the tree
                            //    *         1
                            //   /    =>   / \
                            //  1
                            // / \
                            last = node_ptr.as_ref().left;
                            if let Some(mut root_ptr) = last {
                                root_ptr.as_mut().parent = None;
                            }
                        }
                        Some(mut parent_ptr) => {
                            // Left child becomes parent's right sub tree
                            //   A           A
                            //  / \         / \
                            //     *   =>      1
                            //    /           / \
                            //   1
                            //  / \
                            parent_ptr.as_mut().right = node_ptr.as_ref().left;
                            if let Some(mut right_ptr) = parent_ptr.as_ref().right {
                                right_ptr.as_mut().parent = Some(parent_ptr);
                                last = Some(right_ptr);
                            } else {
                                last = Some(parent_ptr);
                            }
                        }
                    }

                    // Next node is biggest child in sub tree.
                    if let Some(mut next_ptr) = last {
                        while let Some(right_ptr) = next_ptr.as_ref().right {
                            next_ptr = right_ptr;
                        }
                        last = Some(next_ptr);
                    }

                    self.last = last;
                }
            }

            return Some(unsafe { Node::destroy(node_ptr) });
        }

        None
    }

    fn postorder<F: FnMut(NodePtr<K, V>)>(&self, f: F) {
        Map::traverse(self.first, |_| {}, |_| {}, f);
    }
}

impl<K, V> Drop for NodeEater<K, V> {
    /// Drops all nodes which have not been consumed.
    fn drop(&mut self) {
        self.postorder(|node_ptr| unsafe {
            Node::destroy(node_ptr);
        });
    }
}
