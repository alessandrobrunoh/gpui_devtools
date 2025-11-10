use gpui::{div, prelude::*, Context, IntoElement, Render, Window};
use gpui_component::{
    button::{Button, ButtonVariants},
    label::Label,
};
use gpui_inspector::auto_inspector;

use crate::text::Text;

pub struct Tab2 {
    state: bool,
}

impl Tab2 {
    pub fn new() -> Self {
        Tab2 { state: true }
    }

    fn toggle_state(&mut self, cx: &mut Context<Self>) {
        self.state = !self.state;
        // cx.notify();
    }
}

#[auto_inspector]
impl Render for Tab2 {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_text = if self.state { "true" } else { "false" };

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_xl()
                    .text_color(gpui::white())
                    .child(Label::new(format!("Stato: {}", state_text))),
            )
            .child(
                Button::new("toggle-button")
                    .primary()
                    .label("Cambia Stato")
                    .on_click(cx.listener(|tab, _, _, cx| tab.toggle_state(cx))),
            )
    }
}
