// Core module: Contains the data models and state management for inspected data

pub mod state;
pub mod tree;

// Re-export commonly used items
pub use state::{
    get_tree_version, increment_tree_version, set_tree_update_callback, ALL_TREES, RENDER_TREE,
    TREE_UPDATE_CALLBACK, TREE_VERSION,
};
pub use tree::{ElementNode, RenderedNode};
