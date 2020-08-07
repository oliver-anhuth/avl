#![allow(dead_code, unused_variables, unused_must_use)]

use std::ptr::NonNull;

pub struct Tree<K>
where
    K: PartialEq + PartialOrd,
{
    root: Link<K>,
}

impl<K> Tree<K>
where
    K: PartialEq + PartialOrd,
{
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn clear(&mut self) {
        self.postorder(|node_ptr| Node::destroy(node_ptr));
        self.root = None;
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
            return true;
        }
        false
    }

    pub fn remove(&mut self, key: &K) -> bool {
        if let Some(mut node_ptr) = self.find(key) {
            unsafe {
                if let Some(mut min_child_ptr) = node_ptr.as_mut().right {
                    let mut min_child_parent_ptr = node_ptr;
                    while let Some(left_ptr) = min_child_ptr.as_mut().left {
                        min_child_parent_ptr = min_child_ptr;
                        min_child_ptr = left_ptr;
                    }
                    if min_child_parent_ptr.as_mut().left == Some(min_child_ptr) {
                        min_child_parent_ptr.as_mut().left = min_child_ptr.as_mut().right;
                    } else {
                        min_child_parent_ptr.as_mut().right = min_child_ptr.as_mut().right;
                    }
                    if let Some(mut right_ptr) = min_child_ptr.as_mut().right {
                        right_ptr.as_mut().parent = min_child_ptr.as_mut().parent;
                    }
                    std::mem::swap(&mut node_ptr.as_mut().key, &mut min_child_ptr.as_mut().key);
                    Node::destroy(min_child_ptr);
                    debug_assert!(self.get(key).is_none());
                } else {
                    debug_assert!(node_ptr.as_mut().right.is_none());
                    if let Some(mut parent_ptr) = node_ptr.as_mut().parent {
                        if parent_ptr.as_mut().left == Some(node_ptr) {
                            parent_ptr.as_mut().left = node_ptr.as_mut().left;
                        } else {
                            parent_ptr.as_mut().right = node_ptr.as_mut().left;
                        }
                    } else {
                        self.root = node_ptr.as_mut().left;
                    }
                    if let Some(mut left_ptr) = node_ptr.as_mut().left {
                        left_ptr.as_mut().parent = node_ptr.as_mut().parent;
                    }
                    Node::destroy(node_ptr);
                    debug_assert!(self.get(key).is_none());
                }
            }
            return true;
        }
        false
    }

    fn find(&self, key: &K) -> Link<K> {
        let mut current = self.root;
        while let Some(mut node_ptr) = current {
            unsafe {
                if *key == node_ptr.as_mut().key {
                    break;
                } else if *key < node_ptr.as_mut().key {
                    current = node_ptr.as_mut().left;
                } else {
                    current = node_ptr.as_mut().right;
                }
            }
        }
        current
    }

    fn find_insert_pos(&mut self, key: &K) -> Option<(Link<K>, LinkPtr<K>)> {
        let mut parent: Link<K> = None;
        let mut link_ptr: LinkPtr<K> = unsafe { LinkPtr::new_unchecked(&mut self.root) };
        unsafe {
            while let Some(mut node_ptr) = link_ptr.as_mut() {
                if *key == node_ptr.as_mut().key {
                    return None;
                } else {
                    parent = *link_ptr.as_mut();
                    if *key < node_ptr.as_mut().key {
                        link_ptr = LinkPtr::new_unchecked(&mut node_ptr.as_mut().left);
                    } else {
                        link_ptr = LinkPtr::new_unchecked(&mut node_ptr.as_mut().right);
                    }
                }
            }
        }
        Some((parent, link_ptr))
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
                        if let Some(left_ptr) = unsafe { node_ptr.as_mut().left } {
                            node_ptr = left_ptr;
                        } else {
                            dir = Direction::FromLeft;
                        }
                    }
                    Direction::FromLeft => {
                        inorder(node_ptr);
                        if let Some(right_ptr) = unsafe { node_ptr.as_mut().right } {
                            node_ptr = right_ptr;
                            dir = Direction::FromParent;
                        } else {
                            dir = Direction::FromRight;
                        }
                    }
                    Direction::FromRight => {
                        // Post order traversal is used for node deletion,
                        // so make sure not to use node pointer after postorder call.
                        if let Some(mut parent_ptr) = unsafe { node_ptr.as_mut().parent } {
                            if Some(node_ptr) == unsafe { parent_ptr.as_mut().left } {
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
        });
        unsafe { NodePtr::new_unchecked(Box::into_raw(boxed)) }
    }

    fn destroy(node_ptr: NodePtr<K>) {
        unsafe {
            Box::from_raw(node_ptr.as_ptr());
        }
    }
}

enum Direction {
    FromParent,
    FromLeft,
    FromRight,
}
