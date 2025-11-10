use gpui::{
    div, prelude::*, px, rgb, App, Application, Bounds, Context, Entity, FocusHandle, Focusable,
    Point, Size, Window, WindowOptions,
};
use gpui_component::button::ButtonVariants;
use gpui_component::Root;
use gpui_inspector::inspector_view::InspectorView;
use gpui_inspector::{auto_inspector, inspector_main};

use navbar::Navbar;
use tab1::Tab1;
use tab2::Tab2;

mod navbar;
mod tab1;
mod tab2;
mod text;

enum TabMode {
    Tab1,
    Tab2,
}

struct MainView {
    mode: TabMode,
    focus_handle: FocusHandle,
    tab1_view: Entity<Tab1>,
    tab2_view: Entity<Tab2>,
}

impl MainView {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let tab1_view = cx.new(|_| Tab1::new());
        let tab2_view = cx.new(|_| Tab2::new());
        Self {
            mode: TabMode::Tab1,
            focus_handle,
            tab1_view,
            tab2_view,
        }
    }

    fn switch_to_tab1(&mut self, cx: &mut Context<Self>) {
        if matches!(self.mode, TabMode::Tab1) {
            return;
        }
        self.mode = TabMode::Tab1;
        cx.notify();
    }

    fn switch_to_tab2(&mut self, cx: &mut Context<Self>) {
        if matches!(self.mode, TabMode::Tab2) {
            return;
        }
        self.mode = TabMode::Tab2;
        cx.notify();
    }
}

impl Focusable for MainView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[auto_inspector]
impl Render for MainView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_tab = match self.mode {
            TabMode::Tab1 => 1,
            TabMode::Tab2 => 2,
        };

        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size_full()
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(
                Navbar::new("main-navbar", active_tab)
                    .tab_button(
                        gpui_component::button::Button::new("tab1")
                            .when(active_tab == 1, |btn| btn.primary())
                            .label("Tab 1")
                            .on_click(cx.listener(|this, _, _, cx| this.switch_to_tab1(cx)))
                            .into_any_element(),
                    )
                    .tab_button(
                        gpui_component::button::Button::new("tab2")
                            .when(active_tab == 2, |btn| btn.primary())
                            .label("Tab 2")
                            .on_click(cx.listener(|this, _, _, cx| this.switch_to_tab2(cx)))
                            .into_any_element(),
                    ),
            )
            .child(match self.mode {
                TabMode::Tab1 => div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .child(self.tab1_view.clone()),
                TabMode::Tab2 => div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .child(self.tab2_view.clone()),
            })
    }
}

#[inspector_main]
fn main() {
    let app = Application::new();

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|cx| MainView::new(window, cx));
                // This first level on the window, should be a Root.
                cx.new(|cx| Root::new(view.into(), window, cx))
            })?;

            let inspector_options = gpui::WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: px(0.0),
                        y: px(0.0),
                    },
                    size: Size {
                        width: px(400.0),
                        height: px(600.0),
                    },
                })),
                ..Default::default()
            };
            cx.open_window(inspector_options, |_, cx| cx.new(InspectorView::new))?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
