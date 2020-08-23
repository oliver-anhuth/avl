#![allow(dead_code)]

use std::cmp::{self, Ordering};
use std::fmt;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A sorted map implemented with a nearly balanced binary search tree.
pub struct Map<K, V> {
    root: Link<K, V>,
    num_nodes: usize,
}

/// A node in the binary search tree, containing a key, a value, links to its left child,
/// right child and parent node, and its height (== maximum number of links to a leaf node).
struct Node<K, V> {
    key: K,
    value: V,
    left: Link<K, V>,
    right: Link<K, V>,
    parent: Link<K, V>,
    height: usize,
}

type NodePtr<K, V> = NonNull<Node<K, V>>;
type Link<K, V> = Option<NodePtr<K, V>>;
type LinkPtr<K, V> = NonNull<Link<K, V>>;

#[allow(clippy::enum_variant_names)]
enum Direction {
    FromParent,
    FromLeft,
    FromRight,
}

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

struct NodeIter<'a, K, V> {
    next: Link<K, V>,
    dir: Direction,
    marker: std::marker::PhantomData<&'a Node<K, V>>,
}

struct NodeEater<K, V> {
    next: Link<K, V>,
    root: Link<K, V>,
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

    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, key: K, value: V) -> bool {
        if let Some((parent, mut link_ptr)) = self.find_insert_pos(&key) {
            unsafe {
                *link_ptr.as_mut() = Some(Node::create(parent, key, value));
            }
            self.num_nodes += 1;
            if let Some(parent_ptr) = parent {
                self.rebalance_once(parent_ptr);
            }
            return true;
        }
        false
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
    pub fn height(&self) -> usize {
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
}

impl<K, V> Map<K, V> {
    /// Gets an iterator over the entries of the map, sorted by key.
    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        Iter {
            node_iter: NodeIter::new(self.find_min(), Direction::FromLeft),
        }
    }

    /// Gets an iterator over the keys of the map, in sorted order.
    pub fn keys<'a>(&'a self) -> Keys<'a, K, V> {
        Keys {
            node_iter: NodeIter::new(self.find_min(), Direction::FromLeft),
        }
    }

    /// Gets an iterator over the values of the map, in order by key.
    pub fn values<'a>(&'a self) -> Values<'a, K, V> {
        Values {
            node_iter: NodeIter::new(self.find_min(), Direction::FromLeft),
        }
    }

    /// Gets a mutable iterator over the values of the map, in order by key.
    pub fn values_mut<'a>(&'a self) -> ValuesMut<'a, K, V> {
        ValuesMut {
            node_iter: NodeIter::new(self.find_min(), Direction::FromLeft),
        }
    }

    /// Gets a mutable iterator over the entries of the map, sorted by key.
    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, K, V> {
        IterMut {
            node_iter: NodeIter::new(self.find_min(), Direction::FromLeft),
        }
    }
}

impl<K: Ord, V> Map<K, V> {
    #[cfg(any(test, feature = "consistency_check"))]
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

                // Check AVL condition (nearly balance)
                assert!(left_height <= right_height + 1);
                assert!(right_height <= left_height + 1);

                num_nodes += 1;
            });

            // Check number of nodes
            assert_eq!(num_nodes, self.num_nodes);
        }
    }

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

    fn find_insert_pos(&mut self, key: &K) -> Option<(Link<K, V>, LinkPtr<K, V>)> {
        let mut parent: Link<K, V> = None;
        let mut link_ptr: LinkPtr<K, V> = unsafe { LinkPtr::new_unchecked(&mut self.root) };
        unsafe {
            while let Some(mut node_ptr) = link_ptr.as_ref() {
                if *key == node_ptr.as_ref().key {
                    return None;
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
        Some((parent, link_ptr))
    }
}

impl<K, V> Map<K, V> {
    fn find_min(&self) -> Link<K, V> {
        if let Some(mut min_ptr) = self.root {
            unsafe {
                while let Some(left_ptr) = min_ptr.as_ref().left {
                    min_ptr = left_ptr;
                }
                return Some(min_ptr);
            }
        }
        None
    }

    fn unlink_node(&mut self, node_ptr: NodePtr<K, V>) {
        unsafe {
            // Check if node to-unlink has right sub tree
            if let Some(mut min_child_ptr) = node_ptr.as_ref().right {
                // Find smallest child node in right sub tree
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

    fn left_height(node_ptr: NodePtr<K, V>) -> usize {
        unsafe {
            match node_ptr.as_ref().left {
                None => 0,
                Some(left_ptr) => left_ptr.as_ref().height + 1,
            }
        }
    }

    fn right_height(node_ptr: NodePtr<K, V>) -> usize {
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
        root: Link<K, V>,
        mut preorder: Pre,
        mut inorder: In,
        mut postorder: Post,
    ) where
        Pre: FnMut(NodePtr<K, V>),
        In: FnMut(NodePtr<K, V>),
        Post: FnMut(NodePtr<K, V>),
    {
        if let Some(mut node_ptr) = root {
            debug_assert!(unsafe { node_ptr.as_ref().parent }.is_none());
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
            key,
            value,
            parent,
            left: None,
            right: None,
            height: 0,
        });
        NodePtr::new_unchecked(Box::into_raw(boxed))
    }

    unsafe fn destroy(node_ptr: NodePtr<K, V>) -> (K, V) {
        let boxed = Box::from_raw(node_ptr.as_ptr());
        (boxed.key, boxed.value)
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        match self.node_iter.next {
            None => None,
            Some(node_ptr) => unsafe {
                let current_ptr = node_ptr;
                self.node_iter.next();
                Some((&(*current_ptr.as_ptr()).key, &(*current_ptr.as_ptr()).value))
            },
        }
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        match self.node_iter.next {
            None => None,
            Some(node_ptr) => unsafe {
                let current_ptr = node_ptr;
                self.node_iter.next();
                Some(&(*current_ptr.as_ptr()).key)
            },
        }
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        match self.node_iter.next {
            None => None,
            Some(node_ptr) => unsafe {
                let current_ptr = node_ptr;
                self.node_iter.next();
                Some(&(*current_ptr.as_ptr()).value)
            },
        }
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        match self.node_iter.next {
            None => None,
            Some(node_ptr) => unsafe {
                let current_ptr = node_ptr;
                self.node_iter.next();
                Some((
                    &(*current_ptr.as_ptr()).key,
                    &mut (*current_ptr.as_ptr()).value,
                ))
            },
        }
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<Self::Item> {
        match self.node_iter.next {
            None => None,
            Some(node_ptr) => unsafe {
                let current_ptr = node_ptr;
                self.node_iter.next();
                Some(&mut (*current_ptr.as_ptr()).value)
            },
        }
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        self.node_eater.munch()
    }
}

impl<'a, K, V> NodeIter<'a, K, V> {
    fn new(start: Link<K, V>, dir: Direction) -> Self {
        Self {
            next: start,
            dir,
            marker: PhantomData,
        }
    }

    fn next(&mut self) {
        while let Some(node_ptr) = self.next {
            match self.dir {
                Direction::FromParent => {
                    let left = unsafe { node_ptr.as_ref().left };
                    if left.is_some() {
                        self.next = left;
                    } else {
                        self.dir = Direction::FromLeft;
                        break;
                    }
                }
                Direction::FromLeft => {
                    let right = unsafe { node_ptr.as_ref().right };
                    if right.is_some() {
                        self.next = right;
                        self.dir = Direction::FromParent;
                    } else {
                        self.dir = Direction::FromRight;
                    }
                }
                Direction::FromRight => {
                    let from = self.next;
                    let parent = unsafe { node_ptr.as_ref().parent };
                    self.next = parent;
                    if let Some(parent_ptr) = parent {
                        if from == unsafe { parent_ptr.as_ref().left } {
                            self.dir = Direction::FromLeft;
                            break;
                        } else {
                            self.dir = Direction::FromRight;
                        }
                    }
                }
            }
        }
    }
}

impl<K, V> NodeEater<K, V> {
    fn new(mut map: Map<K, V>) -> Self {
        let mut node_eater = Self {
            next: map.root,
            root: map.root.take(),
        };
        node_eater.find_next();
        node_eater
    }

    fn find_next(&mut self) {
        if let Some(mut next_ptr) = self.next {
            while let Some(left_ptr) = unsafe { next_ptr.as_ref().left } {
                next_ptr = left_ptr;
            }
            self.next = Some(next_ptr);
        }
    }

    fn munch(&mut self) -> Option<(K, V)> {
        if let Some(node_ptr) = self.next {
            unsafe {
                match node_ptr.as_ref().parent {
                    None => {
                        self.root = node_ptr.as_ref().right;
                        if let Some(mut root_ptr) = self.root {
                            root_ptr.as_mut().parent = None;
                        }
                        self.next = self.root;
                    }
                    Some(mut parent_ptr) => {
                        parent_ptr.as_mut().left = node_ptr.as_ref().right;
                        if let Some(mut left_ptr) = parent_ptr.as_ref().left {
                            left_ptr.as_mut().parent = Some(parent_ptr);
                            self.next = Some(left_ptr);
                        } else {
                            self.next = Some(parent_ptr);
                        }
                    }
                }
                self.find_next();
                return Some(Node::destroy(node_ptr));
            }
        }
        None
    }

    fn postorder<F: FnMut(NodePtr<K, V>)>(&self, f: F) {
        Map::traverse(self.root, |_| {}, |_| {}, f);
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
