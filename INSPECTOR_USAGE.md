# GPUI Inspector - Usage Guide

## 🎯 Overview

The GPUI Inspector now supports **automatic inspection** using the `#[inspector_main]` macro! You no longer need to manually add `#[inspector]` to every `impl Render` and `impl RenderOnce` block.

## 🚀 Quick Start

### 1. Add `#[inspector_main]` to your main function

```rust
use gpui_inspector::inspector_main;

#[inspector_main]
fn main() {
    let app = Application::new();
    
    app.run(move |cx| {
        // Your app code here
    });
}
```

### 2. Add `#[auto_inspector]` to your Render implementations

```rust
use gpui_inspector::auto_inspector;

struct MyView {
    // fields...
}

#[auto_inspector]
impl Render for MyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Hello, World!")
    }
}
```

### 3. Also works with RenderOnce!

```rust
use gpui_inspector::auto_inspector;

#[derive(IntoElement)]
struct MyComponent {
    label: SharedString,
}

#[auto_inspector]
impl RenderOnce for MyComponent {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        div().child(self.label)
    }
}
```

## 📚 Complete Example

```rust
use gpui::{
    div, prelude::*, px, rgb, Application, Context, SharedString, Window, WindowOptions,
};
use gpui_inspector::{auto_inspector, inspector_main};
use gpui_inspector::inspector_view::InspectorView;

struct MainView {
    text: SharedString,
}

// Just add this attribute!
#[auto_inspector]
impl Render for MainView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0x505050))
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
    }
}

// Add this to main!
#[inspector_main]
fn main() {
    let app = Application::new();

    app.run(move |cx| {
        cx.spawn(async move |cx| {
            // Create your main window
            cx.open_window(WindowOptions::default(), |window, cx| {
                cx.new(|_| MainView {
                    text: "World".into(),
                })
            })?;

            // Create the inspector window
            cx.open_window(WindowOptions::default(), |_, cx| {
                cx.new(InspectorView::new)
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
```

## 🔧 How It Works

1. **`#[inspector_main]`** - Enables auto-inspection globally when your app starts
2. **`#[auto_inspector]`** - Marks a `Render` or `RenderOnce` impl for inspection
3. The inspector automatically captures the render tree and displays it in the inspector window

## 🎨 Inspector UI Features

- **Collapsible tree view** - Click ▶/▼ to expand/collapse nodes
- **Element selection** - Click on any element to view its properties
- **Zed-inspired design** - Dark theme with syntax highlighting
- **Real-time updates** - The tree updates as your UI renders

## 📖 Old Method (Still Supported)

If you prefer manual control, you can still use the old `#[inspector]` macro:

```rust
use gpui_inspector::inspector;

#[inspector]
impl Render for MyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Hello!")
    }
}
```

The old method doesn't require `#[inspector_main]` but you need to add `#[inspector]` to every impl manually.

## 🆚 Comparison

| Feature | Old (`#[inspector]`) | New (`#[inspector_main]` + `#[auto_inspector]`) |
|---------|---------------------|--------------------------------------------------|
| Manual annotation per impl | ✅ Required | ✅ Required |
| Global enable/disable | ❌ No | ✅ Yes (via `#[inspector_main]`) |
| Runtime control | ❌ No | ✅ Yes (can enable/disable) |
| Cleaner code | ⚠️ Okay | ✅ Better separation |

## 🐛 Troubleshooting

### Inspector window is empty
- Make sure you added `#[inspector_main]` to your `main()` function
- Make sure you added `#[auto_inspector]` to your `impl Render` blocks
- Check the console for `[Inspector] Auto-inspection enabled` message

### Elements not showing up
- Ensure your views implement `Render` or `RenderOnce`
- The `#[auto_inspector]` must be placed directly above the `impl` block
- Make sure you're creating the inspector window after enabling inspection

### Compilation errors
- Ensure `gpui-inspector` is in your `Cargo.toml` dependencies
- Make sure you imported the macro: `use gpui_inspector::auto_inspector;`
- Check that you're using `#[inspector_main]` not `#[inspector]` on the main function

## 📝 Notes

- The inspector captures the element tree structure during render
- Properties are captured as strings for display
- Closures and complex types show as `<closure>` or their token representation
- The inspector window is independent and doesn't affect your main app's performance

## 🎉 Benefits of the New Approach

✨ **Less boilerplate** - One `#[inspector_main]` instead of multiple `#[inspector]`  
✨ **Runtime control** - Can enable/disable inspection programmatically  
✨ **Cleaner separation** - Main function clearly marks inspection mode  
✨ **Future-proof** - Easier to add global inspection features  

Enjoy debugging your GPUI apps! 🚀