use gpui::{
    div, prelude::*, px, rgb, Application, Bounds, Context, Point, SharedString, Size, Window,
    WindowOptions,
};
use gpui_component::{button::*, Root, Sizable};
use gpui_inspector::inspector_view::InspectorView;
use gpui_inspector::{auto_inspector, inspector_main};

mod text;

#[derive(Clone)]
struct TodoItem {
    id: usize,
    text: String,
    completed: bool,
}

struct MainView {
    title: SharedString,
    counter: i32,
    show_colors: bool,
    todos: Vec<TodoItem>,
    next_todo_id: usize,
    dark_mode: bool,
    todo_filter: TodoFilter,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum TodoFilter {
    All,
    Active,
    Completed,
}

#[auto_inspector]
impl Render for MainView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg_color = if self.dark_mode {
            rgb(0x1a1a1a)
        } else {
            rgb(0xf3f4f6)
        };

        let text_color = if self.dark_mode {
            rgb(0xffffff)
        } else {
            rgb(0x111827)
        };

        div()
            .id("main_container")
            .flex()
            .flex_col()
            .gap_4()
            .bg(bg_color)
            .text_color(text_color)
            .size_full()
            .p_6()
            .child(self.render_header(cx))
            .child(
                div()
                    .flex()
                    .gap_4()
                    .flex_1()
                    .child(
                        div()
                            .flex_col()
                            .w_1_2()
                            .gap_4()
                            .child(self.render_counter_section(cx))
                            .child(self.render_colors_section(cx)),
                    )
                    .child(
                        div()
                            .flex_col()
                            .w_1_2()
                            .child(self.render_todo_section(cx)),
                    ),
            )
            .child(self.render_footer(cx))
    }
}

#[auto_inspector]
impl MainView {
    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .id("header")
            .flex()
            .items_center()
            .justify_between()
            .mb_4()
            .child(
                div()
                    .id("title_container")
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .id("title")
                            .text_2xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child(
                                Button::new("edit_title_1")
                                    .small()
                                    .label("A")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.title = "GPUI Inspector Demo".into();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("edit_title_2")
                                    .small()
                                    .label("B")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.title = "Awesome GPUI App".into();
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("toggle_theme")
                            .label(if self.dark_mode { "🌞 Light" } else { "🌙 Dark" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dark_mode = !this.dark_mode;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .id("subtitle")
                            .text_sm()
                            .text_color(rgb(0x888888))
                            .child("v0.2.0"),
                    ),
            )
    }

    fn render_todo_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let filtered_todos: Vec<_> = self
            .todos
            .iter()
            .filter(|t| match self.todo_filter {
                TodoFilter::All => true,
                TodoFilter::Active => !t.completed,
                TodoFilter::Completed => t.completed,
            })
            .collect();

        div()
            .id("todo_section")
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .bg(if self.dark_mode {
                rgb(0x2d2d2d)
            } else {
                rgb(0xffffff)
            })
            .border_1()
            .border_color(if self.dark_mode {
                rgb(0x404040)
            } else {
                rgb(0xe5e7eb)
            })
            .rounded_md()
            .shadow_sm()
            .child(
                div()
                    .id("todo_header")
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .id("todo_title")
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(format!("Tasks ({})", filtered_todos.len())),
                    )
                    .child(
                        Button::new("add_todo")
                            .primary()
                            .label("＋ New Task")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let new_todo = TodoItem {
                                    id: this.next_todo_id,
                                    text: format!("Task {}", this.next_todo_id),
                                    completed: false,
                                };
                                this.todos.push(new_todo);
                                this.next_todo_id += 1;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(self.render_filter_button("All", TodoFilter::All, cx))
                    .child(self.render_filter_button("Active", TodoFilter::Active, cx))
                    .child(self.render_filter_button("Done", TodoFilter::Completed, cx)),
            )
            .child(
                div()
                    .id("todo_list")
                    .flex()
                    .flex_col()
                    .gap_2()
                    .max_h(px(300.0))
                    .overflow_y_scroll()
                    .children(filtered_todos.into_iter().map(|todo| {
                        let todo_id = todo.id;
                        let text_color = if self.dark_mode {
                            rgb(0xffffff)
                        } else {
                            rgb(0x111827)
                        };
                        div()
                            .id(SharedString::from(format!("todo_{}", todo_id)))
                            .flex()
                            .items_center()
                            .gap_3()
                            .p_2()
                            .bg(if todo.completed {
                                if self.dark_mode {
                                    rgb(0x1f2937)
                                } else {
                                    rgb(0xf9fafb)
                                }
                            } else {
                                if self.dark_mode {
                                    rgb(0x374151)
                                } else {
                                    rgb(0xffffff)
                                }
                            })
                            .border_1()
                            .border_color(if self.dark_mode {
                                rgb(0x4b5563)
                            } else {
                                rgb(0xf3f4f6)
                            })
                            .rounded_sm()
                            .child(
                                div()
                                    .id(SharedString::from(format!("check_{}", todo_id)))
                                    .size(px(18.0))
                                    .rounded(px(9.0))
                                    .border_2()
                                    .border_color(if todo.completed {
                                        rgb(0x10b981)
                                    } else {
                                        rgb(0x9ca3af)
                                    })
                                    .when(todo.completed, |this| this.bg(rgb(0x10b981)))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(if todo.completed { "✓" } else { "" })
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if let Some(todo) =
                                            this.todos.iter_mut().find(|t| t.id == todo_id)
                                        {
                                            todo.completed = !todo.completed;
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .text_color(if todo.completed {
                                        rgb(0x9ca3af)
                                    } else {
                                        text_color
                                    })
                                    .child(todo.text.clone()),
                            )
                            .child(
                                Button::new(SharedString::from(format!("del_{}", todo_id)))
                                    .danger()
                                    .small()
                                    .label("×")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.todos.retain(|t| t.id != todo_id);
                                        cx.notify();
                                    })),
                            )
                    })),
            )
    }

    fn render_filter_button(
        &self,
        label: &'static str,
        filter: TodoFilter,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let active = self.todo_filter == filter;
        div()
            .id(label)
            .px_2()
            .py_1()
            .text_xs()
            .rounded(px(4.0))
            .bg(if active {
                rgb(0x3b82f6)
            } else if self.dark_mode {
                rgb(0x374151)
            } else {
                rgb(0xe5e7eb)
            })
            .text_color(if active {
                rgb(0xffffff)
            } else if self.dark_mode {
                rgb(0xd1d5db)
            } else {
                rgb(0x4b5563)
            })
            .cursor_pointer()
            .child(label)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.todo_filter = filter;
                cx.notify();
            }))
    }

    fn render_footer(&self, _cx: &Context<Self>) -> impl IntoElement {
        div()
            .id("footer")
            .mt_4()
            .pt_4()
            .border_t_1()
            .border_color(if self.dark_mode {
                rgb(0x374151)
            } else {
                rgb(0xe5e7eb)
            })
            .flex()
            .justify_between()
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6b7280))
                    .child(format!("Total tasks: {}", self.todos.len())),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6b7280))
                    .child("Built with GPUI"),
            )
    }

    fn render_counter_section(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .id("counter_section")
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .bg(if self.dark_mode {
                rgb(0x2d2d2d)
            } else {
                rgb(0xffffff)
            })
            .border_1()
            .border_color(if self.dark_mode {
                rgb(0x404040)
            } else {
                rgb(0xe5e7eb)
            })
            .rounded_md()
            .child(
                div()
                    .id("counter_title")
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Counter"),
            )
            .child(
                div()
                    .id("counter_display")
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .id("counter_value")
                            .text_3xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(if self.counter > 0 {
                                rgb(0x10b981)
                            } else if self.counter < 0 {
                                rgb(0xef4444)
                            } else {
                                rgb(0xf59e0b)
                            })
                            .child(format!("{}", self.counter)),
                    )
                    .child(
                        div()
                            .id("counter_buttons")
                            .flex()
                            .gap_2()
                            .child(Button::new("decrement").label("-").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.counter -= 1;
                                    cx.notify();
                                }),
                            ))
                            .child(Button::new("increment").primary().label("+").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.counter += 1;
                                    cx.notify();
                                }),
                            )),
                    ),
            )
    }

    fn render_colors_section(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .id("colors_section")
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .bg(if self.dark_mode {
                rgb(0x2d2d2d)
            } else {
                rgb(0xffffff)
            })
            .border_1()
            .border_color(if self.dark_mode {
                rgb(0x404040)
            } else {
                rgb(0xe5e7eb)
            })
            .rounded_md()
            .child(
                div()
                    .id("colors_header")
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .id("colors_title")
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Palette"),
                    )
                    .child(
                        Button::new("toggle_colors")
                            .small()
                            .label(if self.show_colors { "Hide" } else { "Show" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_colors = !this.show_colors;
                                cx.notify();
                            })),
                    ),
            )
            .when(self.show_colors, |this| {
                this.child(
                    div()
                        .id("color_grid")
                        .grid()
                        .grid_cols(6)
                        .gap_2()
                        .children(
                            [
                                ("red", rgb(0xef4444)),
                                ("green", rgb(0x10b981)),
                                ("blue", rgb(0x3b82f6)),
                                ("yellow", rgb(0xf59e0b)),
                                ("purple", rgb(0x8b5cf6)),
                                ("pink", rgb(0xec4899)),
                            ]
                            .into_iter()
                            .map(|(name, color)| {
                                div()
                                    .id(SharedString::from(format!("color_{}", name)))
                                    .size(px(24.0))
                                    .bg(color)
                                    .rounded_sm()
                                    .cursor_pointer()
                            }),
                        ),
                )
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
            let main_window = cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| MainView {
                    title: "GPUI Interactive Demo".into(),
                    counter: 0,
                    show_colors: true,
                    dark_mode: true,
                    todo_filter: TodoFilter::All,
                    todos: vec![
                        TodoItem {
                            id: 1,
                            text: "Learn GPUI Inspector".to_string(),
                            completed: false,
                        },
                        TodoItem {
                            id: 2,
                            text: "Build interactive UI".to_string(),
                            completed: true,
                        },
                        TodoItem {
                            id: 3,
                            text: "Test inspector updates".to_string(),
                            completed: false,
                        },
                    ],
                    next_todo_id: 4,
                });
                // This first level on the window, should be a Root.
                cx.new(|cx| Root::new(view.into(), window, cx))
            })?;

            // Register the main window with the inspector
            gpui_inspector::hooks::gpui_inspector_integration().set_main_window(main_window.into());

            let inspector_options = gpui::WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: px(650.0),
                        y: px(100.0),
                    },
                    size: Size {
                        width: px(450.0),
                        height: px(700.0),
                    },
                })),
                ..Default::default()
            };
            cx.open_window(inspector_options, |window, cx| {
                cx.new(|cx| InspectorView::new(window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
