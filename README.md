# 🎨 GPUI Inspector

**A real-time component inspector for the `gpui` framework, inspired by browser devtools.**

[![Crates.io](https://img.shields.io/crates/v/gpui-inspector.svg?style=for-the-badge&label=)](https://crates.io/crates/gpui-inspector)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=for-the-badge)](https://rust-lang.org)

`gpui-inspector` is a developer tool that helps you debug and understand your `gpui` applications by providing a live, interactive view of your component tree. It runs in a separate window and automatically updates as your application's state changes, making UI development faster and more intuitive.

---

## ✨ Features

-   **Live Component Tree:** View the hierarchical structure of your rendered `gpui` elements in real-time.
-   **Component Properties:** Inspect the properties and values passed to your components.
-   **Minimal Setup:** Integrate the inspector into your project with just two macros.
-   **Automatic Updates:** The inspector view automatically reflects changes in your application without any manual refresh.
-   **Entity Expansion:** Automatically resolves and displays the content of rendered `Entity<T>` views for a complete picture.

## 🚀 Getting Started

### 1. Add Dependencies

Add `gpui-inspector` to your `Cargo.toml`:

```toml
[dependencies]
gpui = "0.2" # Or your required version
gpui-inspector = { path = "../gpui-inspector" } # Or use a git/crates.io version
```
*(Note: This example uses a local path. For external projects, use a git or crates.io dependency.)*

### 2. Instrument Your Code

To make a component inspectable, add the `#[auto_inspector]` attribute to its `Render` implementation.

```rust
use gpui_inspector::auto_inspector;

struct MyComponent { /* ... */ }

#[auto_inspector]
impl Render for MyComponent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .child("Hello, Inspector!")
            // ...
    }
}
```

### 3. Update Your Main Function

Replace your standard `main` function with one annotated with `#[inspector_main]`. This macro handles starting both your application and the inspector window.

```rust
use gpui_inspector::inspector_main;

#[inspector_main]
fn main() {
    let app = Application::new();

    app.run(move |cx| {
        // Your regular app setup
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(MyComponent::new)
        }).unwrap();
    });
}
```

### 4. Run Your Application

Run your application with `cargo run`. Two windows will appear: your main app and the GPUI Inspector. As you interact with your application, the component tree in the inspector will update live.

---

## 🤔 Future Ideas / TODO

This project is just getting started. Here are some ideas for future development:

-   [ ] **Interactive Element Highlighting:** Hovering an element in the inspector highlights its bounds in the application window.
-   [ ] **Live Property Editing:** Edit simple properties (colors, text, sizes) directly from the inspector to see immediate visual feedback.
-   [ ] **Component Search & Filter:** Add a search bar to quickly find components by name or property.
-   [ ] **Performance Metrics:** Display render times for each component to help diagnose performance bottlenecks.
-   [ ] **Action & Event Logging:** Create a log of dispatched actions and events to trace the flow of data and user interactions.
-   [ ] **Improved UI/UX:** Enhance the inspector's own UI to be more powerful and user-friendly.

## 🤝 Contribution

Contributions are welcome! Please feel free to open an issue to discuss ideas or submit a pull request with improvements.

## 📜 License

This project is licensed under the [MIT License](LICENSE).
