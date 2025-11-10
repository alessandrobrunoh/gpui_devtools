use crate::core::tree::RenderedNode;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub static RENDER_TREE: Lazy<Arc<Mutex<Option<RenderedNode>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

// Store all rendered trees by component name
pub static ALL_TREES: Lazy<Arc<Mutex<HashMap<String, RenderedNode>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub static TREE_VERSION: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));

// Global callback that gets called when tree updates
type TreeUpdateCallback = Box<dyn Fn() + Send + Sync>;
pub static TREE_UPDATE_CALLBACK: Lazy<Arc<Mutex<Option<TreeUpdateCallback>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub fn set_tree_update_callback<F: Fn() + Send + Sync + 'static>(callback: F) {
    *TREE_UPDATE_CALLBACK.lock() = Some(Box::new(callback));
}

pub fn increment_tree_version() {
    TREE_VERSION.fetch_add(1, Ordering::Relaxed);

    // Call the callback if one is registered
    if let Some(callback) = TREE_UPDATE_CALLBACK.lock().as_ref() {
        callback();
    }
}

pub fn get_tree_version() -> u64 {
    TREE_VERSION.load(Ordering::Relaxed)
}
