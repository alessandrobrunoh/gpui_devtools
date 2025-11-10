use gpui::{div, prelude::*, AnyElement, App, ElementId, IntoElement, RenderOnce, Window};
use gpui_inspector::auto_inspector;

#[derive(IntoElement)]
pub struct Navbar {
    _id: ElementId,
    _active_tab: u32,
    children: Vec<AnyElement>,
}

impl Navbar {
    pub fn new(id: impl Into<ElementId>, active_tab: u32) -> Self {
        Self {
            _id: id.into(),
            _active_tab: active_tab,
            children: Vec::new(),
        }
    }

    pub fn tab_button(mut self, child: AnyElement) -> Self {
        self.children.push(child);
        self
    }
}

#[auto_inspector]
impl RenderOnce for Navbar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .gap_4()
            .p_4()
            .bg(gpui::rgb(0x2a2a2a))
            .children(self.children)
    }
}
