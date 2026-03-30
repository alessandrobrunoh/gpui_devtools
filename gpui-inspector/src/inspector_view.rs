use crate::hooks::handle_element_selection;
use crate::state::RENDER_TREE;
use crate::tree::{ElementNode, RenderedNode};
use gpui::prelude::*;
use gpui::*;
use std::collections::HashSet;

pub struct InspectorView {
    selected_node_id: Option<u64>,
    collapsed_nodes: HashSet<u64>,
    expanded_nodes: HashSet<u64>, // Manually expanded nodes (overrides auto-collapse)
    expand_all: bool,
    search_query: SharedString,
    last_version: u64,
}

// Modern color palette
struct Theme {
    // Backgrounds
    bg_primary: Hsla,
    bg_secondary: Hsla,
    bg_tertiary: Hsla,
    bg_hover: Hsla,
    bg_selected: Hsla,
    bg_accent: Hsla,

    // Borders
    border_subtle: Hsla,
    border_default: Hsla,

    // Text
    text_primary: Hsla,
    text_secondary: Hsla,
    text_muted: Hsla,
    text_accent: Hsla,

    // Accents
    accent_blue: Hsla,
    accent_green: Hsla,
    accent_orange: Hsla,
    accent_purple: Hsla,
    accent_cyan: Hsla,
    accent_pink: Hsla,
}

impl Theme {
    fn dark() -> Self {
        Self {
            // Backgrounds - deeper, more refined dark
            bg_primary: hsla(220.0 / 360.0, 0.13, 0.10, 1.0),
            bg_secondary: hsla(220.0 / 360.0, 0.13, 0.12, 1.0),
            bg_tertiary: hsla(220.0 / 360.0, 0.13, 0.14, 1.0),
            bg_hover: hsla(220.0 / 360.0, 0.20, 0.18, 1.0),
            bg_selected: hsla(210.0 / 360.0, 0.50, 0.25, 1.0),
            bg_accent: hsla(210.0 / 360.0, 0.60, 0.20, 1.0),

            // Borders
            border_subtle: hsla(220.0 / 360.0, 0.10, 0.20, 1.0),
            border_default: hsla(220.0 / 360.0, 0.10, 0.25, 1.0),

            // Text
            text_primary: hsla(0.0, 0.0, 0.93, 1.0),
            text_secondary: hsla(220.0 / 360.0, 0.10, 0.65, 1.0),
            text_muted: hsla(220.0 / 360.0, 0.10, 0.45, 1.0),
            text_accent: hsla(210.0 / 360.0, 0.80, 0.65, 1.0),

            // Accents - vibrant, modern colors
            accent_blue: hsla(210.0 / 360.0, 0.90, 0.55, 1.0),
            accent_green: hsla(145.0 / 360.0, 0.70, 0.50, 1.0),
            accent_orange: hsla(30.0 / 360.0, 0.90, 0.55, 1.0),
            accent_purple: hsla(270.0 / 360.0, 0.70, 0.60, 1.0),
            accent_cyan: hsla(185.0 / 360.0, 0.80, 0.50, 1.0),
            accent_pink: hsla(330.0 / 360.0, 0.70, 0.60, 1.0),
        }
    }
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
            expanded_nodes: HashSet::new(),
            expand_all: false,
            search_query: SharedString::default(),
            last_version: 0,
        };

        this.poll_updates(_window, cx);
        this
    }

    fn poll_updates(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Use a simple counter to reduce polling frequency
        static FRAME_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

        cx.on_next_frame(window, |this, window, cx| {
            let count = FRAME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            // Only check every 6 frames (~10hz at 60fps)
            if count % 6 == 0 {
                let current_version = crate::state::RENDER_TREE
                    .lock()
                    .as_ref()
                    .map(|t| t.version)
                    .unwrap_or(0);

                if current_version > this.last_version {
                    this.last_version = current_version;
                    cx.notify();
                }
            }

            this.poll_updates(window, cx);
        });
    }

    fn get_element_icon(&self, name: &str, has_children: bool) -> &'static str {
        match name.to_lowercase().as_str() {
            "div" if has_children => "󰉋",              // folder icon
            "div" => "󰉌",                              // empty folder
            "button" => "󰍽",                           // button icon
            "input" | "textfield" => "󰷈",              // text input
            "img" | "image" => "󰋩",                    // image
            "svg" => "󰕙",                              // vector
            "text" | "label" => "󰊄",                   // text
            "list" | "scrollview" => "󰙀",              // list
            "stack" | "vstack" | "hstack" => "󰕴",      // stack
            name if name.contains("root") => "󰙅",      // tree root
            name if name.contains("view") => "󰕰",      // view
            name if name.contains("container") => "󰆧", // container
            _ => "󰆧",                                  // default box
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
        let current = if node.global_element_id.is_some() {
            1
        } else {
            0
        };
        current
            + node
                .children
                .iter()
                .map(|child| self.count_gpui_elements(child))
                .sum::<usize>()
    }

    fn count_children(&self, node: &ElementNode) -> usize {
        node.children.len()
    }

    fn count_descendants(&self, node: &ElementNode) -> usize {
        node.children
            .iter()
            .map(|child| 1 + self.count_descendants(child))
            .sum()
    }

    fn toggle_expand_all(&mut self, cx: &mut Context<Self>) {
        self.expand_all = !self.expand_all;
        if self.expand_all {
            self.collapsed_nodes.clear();
            self.expanded_nodes.clear();
        } else {
            self.collapsed_nodes.clear();
            self.expanded_nodes.clear();
        }
        cx.notify();
    }

    fn toggle_collapse(&mut self, node_id: u64, cx: &mut Context<Self>) {
        // If node is in expanded_nodes (was manually expanded), remove it to collapse
        if self.expanded_nodes.contains(&node_id) {
            self.expanded_nodes.remove(&node_id);
            self.collapsed_nodes.insert(node_id);
        }
        // If node is in collapsed_nodes, remove it to expand
        else if self.collapsed_nodes.contains(&node_id) {
            self.collapsed_nodes.remove(&node_id);
            self.expanded_nodes.insert(node_id);
        }
        // Otherwise, toggle based on current visible state - add to expanded_nodes to expand
        else {
            // This means it's auto-collapsed, so expand it
            self.expanded_nodes.insert(node_id);
        }
        cx.notify();
    }

    fn node_matches_search(&self, node: &ElementNode) -> bool {
        if self.search_query.is_empty() {
            return true;
        }
        if node
            .name
            .to_lowercase()
            .contains(&self.search_query.to_lowercase())
        {
            return true;
        }
        if let Some(ref id) = node.global_element_id {
            if id
                .to_lowercase()
                .contains(&self.search_query.to_lowercase())
            {
                return true;
            }
        }
        node.children
            .iter()
            .any(|child| self.node_matches_search(child))
    }

    fn render_header(
        &self,
        rendered_node: &RenderedNode,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = Theme::dark();
        let total_elements = rendered_node
            .element_tree
            .as_ref()
            .map(|root| self.count_elements(root))
            .unwrap_or(0);
        let gpui_elements = rendered_node
            .element_tree
            .as_ref()
            .map(|root| self.count_gpui_elements(root))
            .unwrap_or(0);

        div()
            .flex()
            .flex_col()
            .bg(theme.bg_secondary)
            .border_b_1()
            .border_color(theme.border_subtle)
            // Title bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(44.0))
                    .px_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .size(px(28.0))
                                    .rounded(px(6.0))
                                    .bg(theme.bg_accent)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(div().text_sm().child("🔍")),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary)
                                            .child("GPUI Inspector"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_muted)
                                            .child(rendered_node.component_name.clone()),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.render_stat_badge(
                                "Elements",
                                total_elements,
                                theme.accent_blue,
                            ))
                            .child(self.render_stat_badge(
                                "GPUI",
                                gpui_elements,
                                theme.accent_green,
                            )),
                    ),
            )
            // Toolbar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .gap_3()
                    // Search bar
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .h(px(32.0))
                            .px_3()
                            .bg(theme.bg_primary)
                            .border_1()
                            .border_color(theme.border_subtle)
                            .rounded(px(6.0))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.text_muted)
                                    .mr_2()
                                    .child("󰍉"),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .text_color(if self.search_query.is_empty() {
                                        theme.text_muted
                                    } else {
                                        theme.text_primary
                                    })
                                    .child(if self.search_query.is_empty() {
                                        "Search elements...".into()
                                    } else {
                                        self.search_query.clone()
                                    }),
                            ),
                    )
                    // Action buttons
                    .child(div().flex().gap_1().child(self.render_toolbar_button(
                        if self.expand_all { "󰅀" } else { "󰅃" },
                        if self.expand_all {
                            "Collapse All"
                        } else {
                            "Expand All"
                        },
                        &theme,
                        cx,
                    ))),
            )
    }

    fn render_stat_badge(
        &self,
        label: &'static str,
        count: usize,
        color: Hsla,
    ) -> impl IntoElement {
        let theme = Theme::dark();
        div()
            .flex()
            .items_center()
            .gap_1()
            .px_2()
            .py_1()
            .bg(color.opacity(0.15))
            .rounded(px(4.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(color)
                    .child(format!("{}", count)),
            )
            .child(div().text_xs().text_color(theme.text_muted).child(label))
    }

    fn render_toolbar_button(
        &self,
        icon: &str,
        _tooltip: &str,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let icon_owned = icon.to_string();
        div()
            .size(px(28.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(4.0))
            .bg(theme.bg_tertiary)
            .border_1()
            .border_color(theme.border_subtle)
            .cursor_pointer()
            .hover(|s| {
                s.bg(Theme::dark().bg_hover)
                    .border_color(Theme::dark().border_default)
            })
            .child(
                div()
                    .text_sm()
                    .text_color(theme.text_secondary)
                    .child(icon_owned),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.toggle_expand_all(cx)),
            )
    }

    fn render_section_header(&self, title: &str, icon: &str, theme: &Theme) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .h(px(36.0))
            .px_4()
            .bg(theme.bg_secondary)
            .border_b_1()
            .border_color(theme.border_subtle)
            .child(
                div()
                    .text_sm()
                    .text_color(theme.accent_blue)
                    .child(icon.to_string()),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text_secondary)
                    .child(title.to_uppercase()),
            )
    }

    fn render_element_node(
        &mut self,
        node: &ElementNode,
        indent_level: usize,
        cx: &mut Context<Self>,
        rendered_count: &mut usize,
    ) -> impl IntoElement {
        // Performance limits
        const MAX_VISIBLE_ELEMENTS: usize = 150;
        const MAX_DEPTH: usize = 8;

        // Skip if we've rendered too many elements
        if *rendered_count >= MAX_VISIBLE_ELEMENTS {
            return div().into_any_element();
        }

        // Skip if too deep
        if indent_level > MAX_DEPTH {
            return div().into_any_element();
        }

        *rendered_count += 1;

        let theme = Theme::dark();
        let is_selected = self.selected_node_id == Some(node.id);
        // Determine collapse state:
        // 1. If in expanded_nodes -> NOT collapsed (user manually expanded)
        // 2. If in collapsed_nodes -> collapsed (user manually collapsed)
        // 3. If depth > 2 and not expand_all -> auto-collapsed
        // 4. Otherwise -> not collapsed
        let is_collapsed = if self.expanded_nodes.contains(&node.id) {
            false // Manually expanded
        } else if self.collapsed_nodes.contains(&node.id) {
            true // Manually collapsed
        } else {
            indent_level > 2 && !self.expand_all // Auto-collapse deep nodes
        };
        let has_children = !node.children.is_empty();
        let node_id = node.id;
        let name = node.name.clone();
        let child_count = node.children.len();
        let has_gpui_id = node.global_element_id.is_some();

        // Simple color logic - avoid expensive lowercase checks
        let element_color = if has_gpui_id {
            theme.accent_green
        } else if has_children {
            theme.accent_orange
        } else {
            theme.text_primary
        };

        let bg_color = if is_selected {
            theme.bg_selected
        } else {
            theme.bg_primary
        };

        // Simplified indentation - just use padding
        let indent_px = px((indent_level * 16) as f32);

        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .id(SharedString::from(format!("n{}", node_id)))
                    .flex()
                    .items_center()
                    .h(px(24.0))
                    .pl(indent_px)
                    .pr_2()
                    .bg(bg_color)
                    .cursor_pointer()
                    .when(is_selected, |this| {
                        this.border_l_2().border_color(theme.accent_blue)
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.selected_node_id = Some(node_id);
                            handle_element_selection(node_id, cx);
                            cx.notify();
                        }),
                    )
                    // Arrow + Icon + Name in one line
                    .child(
                        div()
                            .w(px(16.0))
                            .text_xs()
                            .text_color(theme.text_muted)
                            .when(has_children, |this| {
                                this.cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.toggle_collapse(node_id, cx);
                                        }),
                                    )
                                    .child(if is_collapsed { "▶" } else { "▼" })
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(element_color)
                            .child(if has_gpui_id {
                                "●"
                            } else if has_children {
                                "○"
                            } else {
                                "·"
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .ml_1()
                            .text_sm()
                            .text_color(if is_selected {
                                theme.text_primary
                            } else {
                                element_color
                            })
                            .child(name),
                    )
                    .when(has_children, |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(format!("({})", child_count)),
                        )
                    })
                    .when(has_gpui_id, |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.accent_blue)
                                .ml_1()
                                .child("#"),
                        )
                    }),
            )
            .when(!is_collapsed && has_children, |this| {
                let children_to_render: Vec<_> = node
                    .children
                    .iter()
                    .take(50) // Limit children per node
                    .map(|child| {
                        self.render_element_node(child, indent_level + 1, cx, rendered_count)
                    })
                    .collect();

                this.children(children_to_render)
                    .when(node.children.len() > 50, |this| {
                        this.child(
                            div()
                                .pl(px(((indent_level + 1) * 16) as f32))
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(format!("... and {} more", node.children.len() - 50)),
                        )
                    })
            })
            .into_any_element()
    }

    fn render_details_panel(&mut self, rendered_node: &RenderedNode) -> impl IntoElement {
        let theme = Theme::dark();
        let selected_element = self.selected_node_id.and_then(|id| {
            rendered_node
                .element_tree
                .as_ref()
                .and_then(|root| find_element_by_id(root, id))
        });

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_primary)
            .when_some(selected_element, |this, node| {
                this
                    // Element name header
                    .child(
                        div()
                            .p_4()
                            .border_b_1()
                            .border_color(theme.border_subtle)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .size(px(40.0))
                                            .rounded(px(8.0))
                                            .bg(theme.accent_blue.opacity(0.15))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                div()
                                                    .text_xl()
                                                    .text_color(theme.accent_blue)
                                                    .child(self.get_element_icon(
                                                        &node.name,
                                                        !node.children.is_empty(),
                                                    )),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_weight(FontWeight::BOLD)
                                                    .font_family(
                                                        "JetBrains Mono, Consolas, monospace",
                                                    )
                                                    .text_color(theme.text_primary)
                                                    .child(node.name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_2()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme.text_muted)
                                                            .child(format!("ID: {}", node.id)),
                                                    )
                                                    .when(!node.children.is_empty(), |this| {
                                                        this.child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(theme.text_muted)
                                                                .child(format!(
                                                                    "• {} children",
                                                                    node.children.len()
                                                                )),
                                                        )
                                                    }),
                                            ),
                                    ),
                            ),
                    )
                    // GPUI Element ID section
                    .when_some(node.global_element_id.as_ref(), |this, id| {
                        this.child(
                            div()
                                .p_4()
                                .border_b_1()
                                .border_color(theme.border_subtle)
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(theme.accent_green)
                                                        .child("󰐕"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(theme.text_secondary)
                                                        .child("GPUI ELEMENT ID"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .px_3()
                                                .py_2()
                                                .bg(theme.bg_secondary)
                                                .border_1()
                                                .border_color(theme.border_subtle)
                                                .rounded(px(6.0))
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_family(
                                                            "JetBrains Mono, Consolas, monospace",
                                                        )
                                                        .text_color(theme.accent_green)
                                                        .child(id.clone()),
                                                ),
                                        ),
                                ),
                        )
                    })
                    // Properties section
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .overflow_hidden()
                            .p_4()
                            .gap_3()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div().text_sm().text_color(theme.accent_orange).child("󰆧"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text_secondary)
                                            .child("PROPERTIES"),
                                    )
                                    .child(
                                        div()
                                            .px_1()
                                            .py(px(1.0))
                                            .rounded(px(3.0))
                                            .bg(theme.bg_tertiary)
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.text_muted)
                                                    .child(format!("{}", node.properties.len())),
                                            ),
                                    ),
                            )
                            .child(if node.properties.is_empty() {
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .py_6()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_2xl()
                                                    .text_color(theme.text_muted)
                                                    .child("󰆧"),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(theme.text_muted)
                                                    .child("No properties"),
                                            ),
                                    )
                                    .into_any_element()
                            } else {
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .children(
                                        node.properties
                                            .iter()
                                            .map(|(k, v)| self.render_property_row(k, v, &theme)),
                                    )
                                    .into_any_element()
                            }),
                    )
            })
            .when(selected_element.is_none(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .h_full()
                        .gap_3()
                        .child(
                            div()
                                .size(px(64.0))
                                .rounded(px(12.0))
                                .bg(theme.bg_secondary)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(div().text_2xl().text_color(theme.text_muted).child("󰆧")),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.text_secondary)
                                .child("Select an element"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child("to view its properties"),
                        ),
                )
            })
    }

    fn render_property_row(&self, key: &str, value: &str, theme: &Theme) -> impl IntoElement {
        div()
            .flex()
            .items_start()
            .py_2()
            .px_3()
            .rounded(px(4.0))
            .bg(theme.bg_secondary)
            .border_1()
            .border_color(theme.border_subtle)
            .hover(|s| s.border_color(Theme::dark().border_default))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .w_full()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.accent_cyan)
                            .child(key.to_string()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_family("JetBrains Mono, Consolas, monospace")
                            .text_color(theme.text_primary)
                            .child(value.to_string()),
                    ),
            )
    }
}

impl Render for InspectorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::dark();
        let rendered_node_option = RENDER_TREE.lock().clone();

        div()
            .size_full()
            .bg(theme.bg_primary)
            .text_color(theme.text_primary)
            .flex()
            .overflow_hidden()
            // Left panel - Element Tree
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_3_5()
                    .h_full()
                    .border_r_1()
                    .border_color(theme.border_subtle)
                    // Header with search
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .when_some(rendered_node_option.as_ref(), |this, rn| {
                                this.child(self.render_header(rn, cx))
                            }),
                    )
                    // Section header
                    .child(self.render_section_header("Elements", "󰙅", &theme))
                    // Tree content
                    .child(
                        div()
                            .id("tree_container")
                            .flex_1()
                            .overflow_y_scroll()
                            .bg(theme.bg_primary)
                            .children(rendered_node_option.as_ref().and_then(|rn| {
                                rn.element_tree.as_ref().map(|root| {
                                    let mut rendered_count = 0usize;
                                    self.render_element_node(root, 0, cx, &mut rendered_count)
                                })
                            })),
                    ),
            )
            // Right panel - Properties
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_2_5()
                    .h_full()
                    .bg(theme.bg_primary)
                    // Section header
                    .child(self.render_section_header("Properties", "󰆧", &theme))
                    // Properties content
                    .child(
                        div()
                            .id("properties_container")
                            .flex_1()
                            .overflow_y_scroll()
                            .children(
                                rendered_node_option
                                    .as_ref()
                                    .map(|rn| self.render_details_panel(rn)),
                            ),
                    ),
            )
    }
}

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
