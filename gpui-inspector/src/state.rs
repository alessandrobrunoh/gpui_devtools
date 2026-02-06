use crate::tree::RenderedNode;
use gpui::EntityId;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::Arc;

pub static RENDER_TREE: Lazy<Arc<Mutex<Option<Arc<RenderedNode>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub static ACTIVE_INSPECTORS: Lazy<Arc<Mutex<HashSet<EntityId>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));

use crate::tree::ElementNode;
use std::cell::RefCell;

thread_local! {
    pub static TREE_BUILDER: RefCell<TreeBuilder> = RefCell::new(TreeBuilder::new());
}

pub struct TreeBuilder {
    pub stack: Vec<ElementNode>,
    pub root: Option<ElementNode>,
    pub next_id: u64,
}

impl TreeBuilder {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            root: None,
            next_id: 0,
        }
    }

    pub fn start(&mut self) {
        self.stack.clear();
        self.root = None;
        self.next_id = 0;
    }

    pub fn enter_node(&mut self, name: &str) {
        let node = ElementNode {
            id: self.next_id,
            name: name.to_string(),
            properties: std::collections::BTreeMap::new(),
            children: Vec::new(),
            global_element_id: None,
            global_element_path: None,
        };
        self.next_id += 1;
        self.stack.push(node);
    }

    pub fn exit_node(&mut self) {
        if let Some(node) = self.stack.pop() {
            if let Some(parent) = self.stack.last_mut() {
                parent.children.push(node);
            } else {
                self.root = Some(node);
            }
        }
    }

    pub fn add_property(&mut self, name: &str, value: String) {
        if let Some(node) = self.stack.last_mut() {
            node.properties.insert(name.to_string(), value);
        }
    }

    pub fn set_global_id(&mut self, id: String) {
        if let Some(node) = self.stack.last_mut() {
            node.global_element_id = Some(id);
        }
    }

    pub fn set_global_path(&mut self, path: String) {
        if let Some(node) = self.stack.last_mut() {
            node.global_element_path = Some(path);
        }
    }
}
