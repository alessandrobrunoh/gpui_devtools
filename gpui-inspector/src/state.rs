use crate::tree::RenderedNode;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;

pub static RENDER_TREE: Lazy<Arc<Mutex<Option<RenderedNode>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));
