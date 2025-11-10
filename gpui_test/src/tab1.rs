use gpui::{div, prelude::*, Context, IntoElement, Render, Window};
use gpui_inspector::auto_inspector;

pub struct Tab1;

impl Tab1 {
    pub fn new() -> Self {
        Tab1
    }
}

#[auto_inspector]
impl Render for Tab1 {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_2xl()
                    .text_color(gpui::white())
                    .child("Benvenuto in Tab 1!"),
            )
            .child(
                div()
                    .text_lg()
                    .text_color(gpui::rgb(0xcccccc))
                    .child("Questa è la prima tab dell'applicazione."),
            )
    }
}
