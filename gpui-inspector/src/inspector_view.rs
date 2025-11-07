use crate::state::RENDER_TREE;
use crate::tree::{ElementNode, RenderedNode};
use gpui::{
    div, prelude::*, px, rgb, Context, IntoElement, MouseButton, ParentElement, Render, Window,
};
use std::collections::HashSet;

pub struct InspectorView {
    selected_node_id: Option<u64>,
    collapsed_nodes: HashSet<u64>,
}

impl InspectorView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            selected_node_id: None,
            collapsed_nodes: HashSet::new(),
        }
    }

    fn toggle_collapse(&mut self, node_id: u64, cx: &mut Context<Self>) {
        if self.collapsed_nodes.contains(&node_id) {
            self.collapsed_nodes.remove(&node_id);
        } else {
            self.collapsed_nodes.insert(node_id);
        }
        cx.notify();
    }

    fn render_element_node(
        &mut self,
        node: &ElementNode,
        indent_level: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_selected = self.selected_node_id == Some(node.id);
        let is_collapsed = self.collapsed_nodes.contains(&node.id);
        let has_children = !node.children.is_empty();

        let node_id = node.id;
        let name = node.name.clone();

        // Zed colors
        let row_bg = if is_selected {
            rgb(0x2b4f6a) // Zed blue selection
        } else {
            rgb(0x282c34) // Dark background
        };

        let hover_bg = if is_selected {
            rgb(0x2b4f6a)
        } else {
            rgb(0x2f3440)
        };

        div()
            .flex()
            .flex_col()
            .child(
                // Main row
                div()
                    .flex()
                    .items_center()
                    .h(px(24.0))
                    .pl(px(indent_level as f32 * 20.0 + 8.0))
                    .bg(row_bg)
                    .hover(|style| style.bg(hover_bg))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event, _window, cx| {
                            this.selected_node_id = Some(node_id);
                            cx.notify();
                        }),
                    )
                    .child(
                        // Expand/collapse icon
                        div()
                            .w(px(20.0))
                            .h(px(20.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(has_children, |this| {
                                this.on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _event, _window, cx| {
                                        this.toggle_collapse(node_id, cx);
                                        cx.stop_propagation();
                                    }),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x8a8f98))
                                        .child(if is_collapsed { "▶" } else { "▼" }),
                                )
                            }),
                    )
                    .child(
                        // Element icon
                        div()
                            .mr(px(8.0))
                            .text_sm()
                            .text_color(rgb(0x61afef)) // Zed blue for icons
                            .child("⬡"),
                    )
                    .child(
                        // Element name
                        div()
                            .text_sm()
                            .text_color(if is_selected {
                                rgb(0xe5e9f0)
                            } else {
                                rgb(0xabb2bf)
                            })
                            .child(name),
                    ),
            )
            .when(!is_collapsed && has_children, |this| {
                this.children(node.children.iter().map(|child| {
                    self.render_element_node(child, indent_level + 1, cx)
                }))
            })
    }

    fn render_details_panel(
        &mut self,
        rendered_node: &RenderedNode,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected_element = self.selected_node_id.and_then(|id| {
            rendered_node
                .element_tree
                .as_ref()
                .and_then(|root| find_element_by_id(root, id))
        });

        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .when(selected_element.is_some(), |this| {
                this.child(
                    div()
                        .text_base()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .mb_2()
                        .text_color(rgb(0xe5e9f0))
                        .child(if let Some(node) = selected_element.as_ref() {
                            node.name.clone()
                        } else {
                            String::new()
                        }),
                )
            })
            .when(selected_element.is_some(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(selected_element.map(|node| {
                            if node.properties.is_empty() {
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x5c6370))
                                    .italic()
                                    .child("No properties")
                            } else {
                                div().flex().flex_col().gap_1().children(
                                    node.properties.iter().map(|(key, value)| {
                                        div()
                                            .flex()
                                            .items_start()
                                            .gap_2()
                                            .px_2()
                                            .py_1()
                                            .rounded_sm()
                                            .hover(|style| style.bg(rgb(0x2f3440)))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0xc678dd)) // Zed purple for keys
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .child(format!("{}", key)),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0x5c6370))
                                                    .child(":"),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0x98c379)) // Zed green for values
                                                    .child(value.clone()),
                                            )
                                    }),
                                )
                            }
                        })),
                )
            })
            .when(selected_element.is_none(), |this| {
                this.child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x5c6370))
                        .italic()
                        .child("Select an element to view properties"),
                )
            })
    }
}

impl Render for InspectorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rendered_node_option = RENDER_TREE.lock().clone();

        div()
            .size_full()
            .bg(rgb(0x282c34)) // Zed dark background
            .text_color(rgb(0xabb2bf))
            .flex()
            .child(
                // Left panel: Element Tree
                div()
                    .flex_col()
                    .w_1_2()
                    .h_full()
                    .border_r_1()
                    .border_color(rgb(0x181a1f))
                    .child(
                        // Header
                        div()
                            .flex()
                            .items_center()
                            .h(px(32.0))
                            .px_3()
                            .border_b_1()
                            .border_color(rgb(0x181a1f))
                            .bg(rgb(0x21252b))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(0x8a8f98))
                                    .child("ELEMENT TREE"),
                            ),
                    )
                    .child(
                        // Tree content
                        div()
                            .flex_col()
                            .children(rendered_node_option.as_ref().and_then(|rn| {
                                rn.element_tree
                                    .as_ref()
                                    .map(|root| self.render_element_node(root, 0, cx))
                            })),
                    ),
            )
            .child(
                // Right panel: Properties
                div()
                    .flex_col()
                    .w_1_2()
                    .h_full()
                    .child(
                        // Header
                        div()
                            .flex()
                            .items_center()
                            .h(px(32.0))
                            .px_3()
                            .border_b_1()
                            .border_color(rgb(0x181a1f))
                            .bg(rgb(0x21252b))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(0x8a8f98))
                                    .child("PROPERTIES"),
                            ),
                    )
                    .child(
                        // Properties content
                        div()
                            .flex_col()
                            .children(
                                rendered_node_option
                                    .as_ref()
                                    .map(|rn| self.render_details_panel(rn, cx)),
                            ),
                    ),
            )
    }
}

// Helper function to find an element by ID in the tree
fn find_element_by_id<'a>(root: &'a ElementNode, id: u64) -> Option<&'a ElementNode> {
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
