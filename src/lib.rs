mod tree;
pub use tree::Tree;

#[cfg(test)]
mod tests {
    use super::Tree;

    #[test]
    fn test_new() {
        let _ = Tree::new();
    }
}
