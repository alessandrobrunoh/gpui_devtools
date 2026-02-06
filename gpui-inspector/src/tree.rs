use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use gpui::SharedString;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RenderedNode {
    pub version: u64,
    pub component_name: String,
    pub element_tree: Option<ElementNode>,
    /// Mapping from local element IDs to GPUI element IDs (as strings)
    #[serde(skip)]
    pub gpui_element_ids: HashMap<u64, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElementNode {
    pub id: u64,
    pub name: String,
    pub properties: BTreeMap<String, String>,
    pub children: Vec<ElementNode>,
    /// GPUI element ID for this element (when available)
    #[serde(skip)]
    pub global_element_id: Option<String>,
    /// GPUI element path for this element (when available)
    #[serde(skip)]
    pub global_element_path: Option<String>,
}

impl ElementNode {
    /// Create a new element node with GPUI integration
    pub fn with_global_id(mut self, id: String) -> Self {
        self.global_element_id = Some(id);
        self
    }

    /// Create a new element node with GPUI path
    pub fn with_global_path(mut self, path: String) -> Self {
        self.global_element_path = Some(path);
        self
    }

    /// Check if this element has a GPUI element ID
    pub fn has_global_id(&self) -> bool {
        self.global_element_id.is_some()
    }

    /// Check if this element has a GPUI element path
    pub fn has_global_path(&self) -> bool {
        self.global_element_path.is_some()
    }

    /// Check if this element is focusable (has a GPUI ID)
    pub fn is_focusable(&self) -> bool {
        self.global_element_id.is_some()
    }

    /// Get the global element ID if available
    pub fn global_id(&self) -> Option<&String> {
        self.global_element_id.as_ref()
    }

    /// Get the global element path if available
    pub fn global_path(&self) -> Option<&String> {
        self.global_element_path.as_ref()
    }
}

/// Trait to convert GPUI property values to strings for the inspector.
/// Uses autoref-based specialization to provide a fallback for types that don't implement it.
pub trait Inspectable {
    fn inspect(&self) -> String;
}

pub trait InspectableFallback {
    fn inspect(&self) -> String;
}

impl<T> InspectableFallback for T {
    fn inspect(&self) -> String {
        "Value".to_string()
    }
}

macro_rules! impl_inspectable_display {
    ($($t:ty),*) => {
        $(
            impl Inspectable for $t {
                fn inspect(&self) -> String {
                    self.to_string()
                }
            }
        )*
    };
}

impl_inspectable_display!(
    String, &str, SharedString, 
    bool, i8, i16, i32, i64, isize, 
    u8, u16, u32, u64, usize, 
    f32, f64
);

impl Inspectable for gpui::Hsla {
    fn inspect(&self) -> String {
        format!("hsla({:.0}, {:.0}%, {:.0}%, {:.1})", self.h * 360.0, self.s * 100.0, self.l * 100.0, self.a)
    }
}

impl<T: Inspectable> Inspectable for Option<T> {
    fn inspect(&self) -> String {
        match self {
            Some(v) => v.inspect(),
            None => "None".to_string(),
        }
    }
}

/// A guard that exits a node in the TreeBuilder when dropped.
pub struct NodeGuard;
impl Drop for NodeGuard {
    fn drop(&mut self) {
        crate::builder_exit();
    }
}

/// Helper for capturing values that might or might not implement Debug.
pub struct InspectorDebugValue<'a, T>(pub &'a T);

impl<'a, T: std::fmt::Debug> InspectorDebugValue<'a, T> {
    pub fn inspect(&self) -> String {
        format!("{:?}", self.0)
    }
}

pub trait InspectorDebugFallback {
    fn inspect(&self) -> String;
}

impl<'a, T> InspectorDebugFallback for InspectorDebugValue<'a, T> {
    fn inspect(&self) -> String {
        "...".to_string()
    }
}

/// A wrapper that carries both a GPUI element and its inspector metadata.
/// This allows the inspector to "see" inside helper methods.
pub struct InspectorWrapper<E> {
    pub element: E,
    pub node: ElementNode,
}

impl<E: gpui::IntoElement> gpui::IntoElement for InspectorWrapper<E> {
    type Element = E::Element;
    fn into_element(self) -> Self::Element {
        self.element.into_element()
    }
}

impl<E: gpui::RenderOnce> gpui::RenderOnce for InspectorWrapper<E> {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl gpui::IntoElement {
        self.element.render(window, cx)
    }
}
