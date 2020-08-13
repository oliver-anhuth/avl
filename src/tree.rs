use std::cmp;
use std::ptr::NonNull;

pub struct Tree<K>
where
    K: PartialEq + PartialOrd,
{
    root: Link<K>,
    num_nodes: usize,
}

impl<K> Tree<K>
where
    K: PartialEq + PartialOrd,
{
    pub fn new() -> Self {
        Self {
            root: None,
            num_nodes: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn len(&self) -> usize {
        self.num_nodes
    }

    pub fn height(&self) -> usize {
        match self.root {
            None => 0,
            Some(root_ptr) => unsafe { root_ptr.as_ref().height },
        }
    }

    pub fn clear(&mut self) {
        self.postorder(|node_ptr| unsafe { Node::destroy(node_ptr) });
        self.root = None;
        self.num_nodes = 0;
    }

    pub fn get(&self, key: &K) -> Option<&K> {
        if let Some(node_ptr) = self.find(key) {
            return Some(&unsafe { &*node_ptr.as_ptr() }.key);
        }
        None
    }

    pub fn insert(&mut self, key: K) -> bool {
        if let Some((parent, mut link_ptr)) = self.find_insert_pos(&key) {
            unsafe {
                *link_ptr.as_mut() = Some(Node::create(parent, key));
            }
            self.num_nodes += 1;
            self.rebalance_once(parent);
            return true;
        }
        false
    }

    pub fn remove(&mut self, key: &K) -> bool {
        // Find node to-be-removed
        if let Some(node_ptr) = self.find(key) {
            debug_assert!(self.num_nodes >= 1);
            self.unlink_node(node_ptr);
            unsafe { Node::destroy(node_ptr) };
            self.num_nodes -= 1;
            debug_assert!(self.get(key).is_none());
            return true;
        }
        false
    }

    #[cfg(test)]
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
                    left_height = left_ptr.as_ref().height;
                    height = cmp::max(height, left_height + 1);
                }

                // Check link for right child node
                if let Some(right_ptr) = node_ptr.as_ref().right {
                    assert!(right_ptr.as_ref().parent == Some(node_ptr));
                    assert!(right_ptr.as_ref().key > node_ptr.as_ref().key);
                    right_height = right_ptr.as_ref().height;
                    height = cmp::max(height, right_height + 1);
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

    fn find(&self, key: &K) -> Link<K> {
        let mut current = self.root;
        while let Some(node_ptr) = current {
            unsafe {
                if *key == node_ptr.as_ref().key {
                    break;
                } else if *key < node_ptr.as_ref().key {
                    current = node_ptr.as_ref().left;
                } else {
                    current = node_ptr.as_ref().right;
                }
            }
        }
        current
    }

    fn find_insert_pos(&mut self, key: &K) -> Option<(Link<K>, LinkPtr<K>)> {
        let mut parent: Link<K> = None;
        let mut link_ptr: LinkPtr<K> = unsafe { LinkPtr::new_unchecked(&mut self.root) };
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

    fn unlink_node(&mut self, node_ptr: NodePtr<K>) {
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
                self.rebalance(Some(rebalance_from));
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
                        self.rebalance(Some(parent_ptr));
                    }
                }
            }
        }
    }

    fn left_height(node_ptr: NodePtr<K>) -> usize {
        unsafe {
            match node_ptr.as_ref().left {
                None => 0,
                Some(left_ptr) => left_ptr.as_ref().height,
            }
        }
    }

    fn right_height(node_ptr: NodePtr<K>) -> usize {
        unsafe {
            match node_ptr.as_ref().right {
                None => 0,
                Some(right_ptr) => right_ptr.as_ref().height,
            }
        }
    }

    fn adjust_height(mut node_ptr: NodePtr<K>) {
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

    fn rotate_left(&mut self, mut node_ptr: NodePtr<K>) {
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

    fn rotate_right(&mut self, mut node_ptr: NodePtr<K>) {
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

    // Rebalance nodes starting from given position up to the root node.
    fn rebalance(&mut self, start_from: Link<K>) {
        let mut current = start_from;
        while let Some(node_ptr) = current {
            let parent = unsafe { node_ptr.as_ref().parent };
            self.rebalance_node(node_ptr);
            current = parent;
        }
    }

    // Rebalance nodes starting from given position up to the root node.
    // Stop after first rebalance operation.
    // This is enough to restore balance after a single insert operation.
    fn rebalance_once(&mut self, start_from: Link<K>) {
        let mut current = start_from;
        while let Some(node_ptr) = current {
            let parent = unsafe { node_ptr.as_ref().parent };
            let did_rebalance = self.rebalance_node(node_ptr);
            if did_rebalance {
                break;
            }
            current = parent;
        }
    }

    // Rebalance nodes starting from given position up to the root node.
    fn rebalance_node(&mut self, node_ptr: NodePtr<K>) -> bool {
        unsafe {
            let left_height = Self::left_height(node_ptr);
            let right_height = Self::right_height(node_ptr);
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

    #[cfg(test)]
    fn preorder<F: FnMut(NodePtr<K>)>(&self, f: F) {
        self.traverse(f, |_| {}, |_| {});
    }

    fn postorder<F: FnMut(NodePtr<K>)>(&self, f: F) {
        self.traverse(|_| {}, |_| {}, f);
    }

    fn traverse<Pre, In, Post>(&self, mut preorder: Pre, mut inorder: In, mut postorder: Post)
    where
        Pre: FnMut(NodePtr<K>),
        In: FnMut(NodePtr<K>),
        Post: FnMut(NodePtr<K>),
    {
        if let Some(mut node_ptr) = self.root {
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

impl<K> Drop for Tree<K>
where
    K: PartialEq + PartialOrd,
{
    fn drop(&mut self) {
        self.clear();
    }
}

type NodePtr<K> = NonNull<Node<K>>;
type Link<K> = Option<NodePtr<K>>;
type LinkPtr<K> = NonNull<Link<K>>;

struct Node<K> {
    key: K,
    left: Link<K>,
    right: Link<K>,
    parent: Link<K>,
    height: usize,
}

impl<K> Node<K>
where
    K: PartialOrd,
{
    fn create(parent: Link<K>, key: K) -> NodePtr<K> {
        let boxed = Box::new(Node {
            key,
            parent,
            left: None,
            right: None,
            height: 0,
        });
        unsafe { NodePtr::new_unchecked(Box::into_raw(boxed)) }
    }

    unsafe fn destroy(node_ptr: NodePtr<K>) {
        Box::from_raw(node_ptr.as_ptr());
    }
}

enum Direction {
    FromParent,
    FromLeft,
    FromRight,
}
