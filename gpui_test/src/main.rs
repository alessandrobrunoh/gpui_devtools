use gpui::{
    div, prelude::*, px, rgb, Application, Bounds, Context, Point, SharedString, Size, Window,
    WindowOptions,
};
use gpui_component::{button::*, Root};
use gpui_inspector::{auto_inspector, inspector_main};
use gpui_inspector::inspector_view::InspectorView;

mod text;
use text::Text;

struct MainView {
    text: SharedString,
}

#[auto_inspector]
impl Render for MainView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()

            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(px(500.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
            .child(
                Button::new("ok")
                    .primary()
                    .label("Let's Go!")
                    .on_click(|_, _, _| println!("Clicked!")),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().size_8().bg(gpui::red()))
                    .child(div().size_8().bg(gpui::green()))
                    .child(div().size_8().bg(gpui::blue()))
                    .child(div().size_8().bg(gpui::yellow()))
                    .child(div().size_8().bg(gpui::black()))
                    .child(div().size_8().bg(gpui::white())),
            )
            .child(Text::new("hello-worldo").label("Hello World2"))
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
                let view = cx.new(|_| MainView {
                    text: "World".into(),
                });
                // This first level on the window, should be a Root.
                cx.new(|cx| Root::new(view.into(), window, cx))
            })?;

            let inspector_options = WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(Bounds {
                    origin: Point { x: px(0.0), y: px(0.0) },
                    size: Size { width: px(400.0), height: px(600.0) },
                })),
                ..Default::default()
            };
            cx.open_window(inspector_options, |_, cx| {
                cx.new(InspectorView::new)
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
