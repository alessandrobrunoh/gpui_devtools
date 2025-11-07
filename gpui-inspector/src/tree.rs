use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RenderedNode {
    pub component_name: String,
    pub element_tree: Option<ElementNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElementNode {
    pub id: u64,
    pub name: String,
    pub properties: HashMap<String, String>,
    pub children: Vec<ElementNode>,
}
