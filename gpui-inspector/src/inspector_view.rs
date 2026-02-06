use gpui::prelude::*;
use gpui::*;
use crate::hooks::handle_element_selection;
use crate::state::RENDER_TREE;
use crate::tree::{ElementNode, RenderedNode};
use std::collections::HashSet;
use std::sync::Arc;

pub struct InspectorView {
    selected_node_id: Option<u64>,
    collapsed_nodes: HashSet<u64>,
    expand_all: bool,
    search_query: String,
    last_version: u64,
}

impl InspectorView {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let entity_id = cx.entity_id();
        crate::state::ACTIVE_INSPECTORS.lock().insert(entity_id);

        cx.on_release(move |_this, _cx| {
            crate::state::ACTIVE_INSPECTORS.lock().remove(&entity_id);
        })
        .detach();

        let mut this = Self {
            selected_node_id: None,
            collapsed_nodes: HashSet::new(),
            expand_all: false,
            search_query: String::new(),
            last_version: 0,
        };

        this.poll_updates(_window, cx);
        this
    }

    fn poll_updates(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        cx.on_next_frame(window, |this, window, cx| {
            let current_version = crate::state::RENDER_TREE
                .lock()
                .as_ref()
                .map(|t| t.version)
                .unwrap_or(0);

            if current_version > this.last_version {
                this.last_version = current_version;
                cx.notify();
            }

            this.poll_updates(window, cx);
        });
    }

    fn get_element_icon(&self, name: &str, has_children: bool) -> &'static str {
        match name {
            "div" | "Div" if has_children => "📁",
            "div" | "Div" => "⬜",
            "button" | "Button" => "🔘",
            "input" | "Input" => "📝",
            "img" | "Img" => "🖼️",
            "svg" | "Svg" => "🎨",
            "Text" | "text" => "📄",
            _ if name.contains("Root") => "🏠",
            _ => "⚡",
        }
    }

    fn count_elements(&self, node: &ElementNode) -> usize {
        1 + node
            .children
            .iter()
            .map(|child| self.count_elements(child))
            .sum::<usize>()
    }

    fn count_gpui_elements(&self, node: &ElementNode) -> usize {
        let current = if node.global_element_id.is_some() { 1 } else { 0 };
        current + node.children.iter().map(|child| self.count_gpui_elements(child)).sum::<usize>()
    }

    fn toggle_expand_all(&mut self, cx: &mut Context<Self>) {
        self.expand_all = !self.expand_all;
        if self.expand_all {
            self.collapsed_nodes.clear();
        } else {
            self.collapsed_nodes.clear();
        }
        cx.notify();
    }

    fn toggle_collapse(&mut self, node_id: u64, cx: &mut Context<Self>) {
        if self.collapsed_nodes.contains(&node_id) {
            self.collapsed_nodes.remove(&node_id);
        } else {
            self.collapsed_nodes.insert(node_id);
        }
        cx.notify();
    }

    fn node_matches_search(&self, node: &ElementNode) -> bool {
        if node.name.to_lowercase().contains(&self.search_query.to_lowercase()) {
            return true;
        }
        node.children.iter().any(|child| self.node_matches_search(child))
    }

    fn render_tree_header(
        &self,
        rendered_node: &RenderedNode,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let total_elements = rendered_node.element_tree.as_ref().map(|root| self.count_elements(root)).unwrap_or(0);
        let gpui_elements = rendered_node.element_tree.as_ref().map(|root| self.count_gpui_elements(root)).unwrap_or(0);

        div()
            .flex()
            .flex_col()
            .bg(rgb(0x21252b))
            .border_b_1()
            .border_color(rgb(0x181a1f))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_2()
                    .px_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(div().text_xs().text_color(rgb(0x9ca3af)).child(format!("Total: {}", total_elements)))
                            .child(div().text_xs().text_color(rgb(0x10b981)).child(format!("GPUI: {}", gpui_elements))),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .bg(rgb(0x374151))
                                    .rounded(px(3.0))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(0x4b5563)))
                                    .child(div().text_xs().child(if self.expand_all { "Collapse All" } else { "Expand All" }))
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| this.toggle_expand_all(cx))),
                            )
                    ),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .bg(rgb(0x181a1f))
                            .rounded(px(4.0))
                            .px_2()
                            .py_1()
                            .child(div().text_xs().text_color(rgb(0x6b7280)).mr_2().child("🔍"))
                            .child(div().flex_1().text_xs().child(if self.search_query.is_empty() { "Filter...".to_string() } else { self.search_query.clone() }))
                    )
            )
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

        if !self.search_query.is_empty() && !self.node_matches_search(node) {
            return div().into_any_element();
        }

        let bg_color = rgb(0x282c34);
        let selected_row_bg = rgb(0x3e4451);
        let active_row_bg = rgb(0x2c313a);
        let text_primary = rgb(0xabb2bf);
        let text_secondary = rgb(0x5c6370);
        let accent_green = rgb(0x98c379);
        let accent_orange = rgb(0xd19a66);

        let row_bg = if is_selected { selected_row_bg } else { bg_color };
        let text_color = if is_selected { rgb(0xffffff) } else if node.global_element_id.is_some() { accent_green } else if has_children { accent_orange } else { text_primary };

        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(24.0))
                    .px_2()
                    .bg(row_bg)
                    .hover(|s| s.bg(if is_selected { selected_row_bg } else { active_row_bg }))
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.selected_node_id = Some(node_id);
                        handle_element_selection(node_id, cx);
                        cx.notify();
                    }))
                    .child(
                        div().flex().h_full().children((0..indent_level).map(|i| {
                            div().w(px(12.0)).h_full().flex().justify_center().child(
                                div().w(px(1.0)).h_full().bg(rgb(0x3e4451)).opacity(if i == indent_level - 1 && has_children { 0.5 } else { 0.2 })
                            )
                        }))
                    )
                    .child(
                        div().w(px(16.0)).flex().justify_center().child(
                            div().text_xs().text_color(text_secondary).when(has_children, |this| {
                                this.on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                    this.toggle_collapse(node_id, cx);
                                    cx.stop_propagation();
                                })).child(if is_collapsed { "▸" } else { "▾" })
                            })
                        )
                    )
                    .child(div().mr_2().text_xs().child(self.get_element_icon(&name, has_children)))
                    .child(div().flex_1().text_sm().font_family("Monospace").text_color(text_color).child(name))
                    .when(node.global_element_id.is_some(), |this| {
                        this.child(div().ml_2().text_xs().text_color(rgb(0x61afef)).child("#"))
                    })
            )
            .when(!is_collapsed && has_children, |this| {
                this.children(node.children.iter().map(|child| self.render_element_node(child, indent_level + 1, cx)))
            })
            .into_any_element()
    }

    fn render_details_panel(&mut self, rendered_node: &RenderedNode) -> impl IntoElement {
        let selected_element = self.selected_node_id.and_then(|id| {
            rendered_node.element_tree.as_ref().and_then(|root| find_element_by_id(root, id))
        });

        let text_primary = rgb(0xabb2bf);
        let text_secondary = rgb(0x5c6370);
        let accent_green = rgb(0x98c379);

        div()
            .flex()
            .flex_col()
            .p_4()
            .gap_4()
            .when_some(selected_element, |this, node| {
                this.child(
                    div().flex().flex_col().gap_1()
                        .child(div().text_xs().font_weight(gpui::FontWeight::BOLD).text_color(rgb(0xe06c75)).child("ELEMENT"))
                        .child(div().text_lg().font_weight(gpui::FontWeight::BOLD).text_color(rgb(0xffffff)).child(node.name.clone()))
                )
                .when_some(node.global_element_id.as_ref(), |this, id| {
                    this.child(
                        div().flex().flex_col().gap_1()
                            .child(div().text_xs().font_weight(gpui::FontWeight::BOLD).text_color(rgb(0x61afef)).child("GPUI ID"))
                            .child(div().px_2().py_1().bg(rgb(0x21252b)).rounded(px(4.0)).child(div().text_sm().font_family("Monospace").text_color(accent_green).child(id.clone())))
                    )
                })
                .child(
                    div().flex().flex_col().gap_2()
                        .child(div().text_xs().font_weight(gpui::FontWeight::BOLD).text_color(rgb(0xd19a66)).child("PROPERTIES"))
                        .child(if node.properties.is_empty() {
                            div().text_sm().italic().text_color(text_secondary).child("None")
                        } else {
                            div().flex().flex_col().gap_1().children(node.properties.iter().map(|(k, v)| {
                                div().flex().justify_between().py_1().px_2().rounded(px(4.0)).hover(|s| s.bg(rgb(0x2c313a)))
                                    .child(div().text_sm().text_color(text_primary).child(k.clone()))
                                    .child(div().max_w_1_2().text_sm().font_family("Monospace").text_color(accent_green).child(v.clone()))
                            }))
                        })
                )
            })
            .when(selected_element.is_none(), |this| {
                this.child(div().flex().flex_col().items_center().justify_center().h_full().child(div().text_sm().text_color(text_secondary).child("Select an element to inspect")))
            })
    }
}

impl Render for InspectorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rendered_node_option = RENDER_TREE.lock().clone();

        div()
            .size_full()
            .bg(rgb(0x282c34))
            .text_color(rgb(0xabb2bf))
            .flex()
            .child(
                div()
                    .flex().flex_col().w_3_5().h_full().border_r_1().border_color(rgb(0x181a1f))
                    .child(
                        div().flex().items_center().h(px(32.0)).px_4().bg(rgb(0x21252b)).border_b_1().border_color(rgb(0x181a1f))
                            .child(div().text_xs().font_weight(gpui::FontWeight::BOLD).text_color(rgb(0x5c6370)).child("ELEMENT TREE"))
                    )
                    .child(div().flex().flex_col().when_some(rendered_node_option.as_ref(), |this, rn| {
                        this.child(self.render_tree_header(rn, cx))
                    }))
                    .child(
                        div().id("tree_container").flex_1().overflow_y_scroll().bg(rgb(0x282c34))
                            .children(rendered_node_option.as_ref().and_then(|rn| {
                                rn.element_tree.as_ref().map(|root| self.render_element_node(root, 0, cx))
                            }))
                    )
            )
            .child(
                div()
                    .flex().flex_col().w_2_5().h_full()
                    .child(
                        div().flex().items_center().h(px(32.0)).px_4().bg(rgb(0x21252b)).border_b_1().border_color(rgb(0x181a1f))
                            .child(div().text_xs().font_weight(gpui::FontWeight::BOLD).text_color(rgb(0x5c6370)).child("PROPERTIES"))
                    )
                    .child(
                        div().id("properties_container").flex_1().overflow_y_scroll().bg(rgb(0x282c34))
                            .children(rendered_node_option.as_ref().map(|rn| self.render_details_panel(rn)))
                    )
            )
    }
}

fn find_element_by_id<'a>(root: &'a ElementNode, id: u64) -> Option<&'a ElementNode> {
    if root.id == id { return Some(root); }
    for child in &root.children { if let Some(found) = find_element_by_id(child, id) { return Some(found); } }
    None
}
