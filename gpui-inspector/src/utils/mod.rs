// Utility functions and helpers

use crate::core::ElementNode;

/// Helper function to find an element by ID in the tree
pub fn find_element_by_id(root: &ElementNode, id: u64) -> Option<&ElementNode> {
    if root.id == id {
        return Some(root);
    }
    for child in &root.children {
        if let Some(found) = find_element_by_id(child, id) {
            return Some(found);
        }
    }
    None
}
