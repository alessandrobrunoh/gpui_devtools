use gpui::{div, prelude::*, App, ElementId, IntoElement, RenderOnce, SharedString, Window};

use crate::inspector;

#[derive(IntoElement)]
pub struct Text {
    id: ElementId,
    label: Option<SharedString>,
}

impl Text {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[inspector]
impl RenderOnce for Text {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().child(self.label.unwrap_or_default())
    }
}
